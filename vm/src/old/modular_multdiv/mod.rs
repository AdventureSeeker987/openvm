use std::sync::Arc;

use afs_primitives::{
    bigint::{check_carry_mod_to_zero::CheckCarryModToZeroSubAir, utils::big_uint_mod_inverse},
    var_range::VariableRangeCheckerChip,
};
use air::ModularMultDivAir;
use hex_literal::hex;
use num_bigint_dig::BigUint;
use once_cell::sync::Lazy;
use p3_field::PrimeField32;

use crate::{
    arch::{
        instructions::{ModularArithmeticOpcode, UsizeOpcode},
        ExecutionBridge, ExecutionBus, ExecutionState, InstructionExecutor,
    },
    system::{
        memory::{MemoryControllerRef, MemoryHeapReadRecord, MemoryHeapWriteRecord},
        program::{bridge::ProgramBus, ExecutionError, Instruction},
    },
    utils::{biguint_to_limbs, limbs_to_biguint},
};

mod air;
mod bridge;
mod columns;
mod trace;

pub use columns::*;

#[cfg(test)]
mod tests;

// Max bits that can fit into our field element.
pub const FIELD_ELEMENT_BITS: usize = 30;

pub static SECP256K1_COORD_PRIME: Lazy<BigUint> = Lazy::new(|| {
    BigUint::from_bytes_be(&hex!(
        "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE FFFFFC2F"
    ))
});

pub static SECP256K1_SCALAR_PRIME: Lazy<BigUint> = Lazy::new(|| {
    BigUint::from_bytes_be(&hex!(
        "FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141"
    ))
});

#[derive(Debug, Clone)]
pub struct ModularMultDivRecord<T, const NUM_LIMBS: usize> {
    pub from_state: ExecutionState<u32>,
    pub instruction: Instruction<T>,

    pub x_array_read: MemoryHeapReadRecord<T, NUM_LIMBS>,
    pub y_array_read: MemoryHeapReadRecord<T, NUM_LIMBS>,
    pub z_array_write: MemoryHeapWriteRecord<T, NUM_LIMBS>,
}

// This chip is for modular multiplication and division of usually 256 bit numbers
// represented as 32 8 bit limbs in little endian format.
// Note: CARRY_LIMBS = 2 * NUM_LIMBS - 1 is required
// Warning: The chip can break if NUM_LIMBS * LIMB_SIZE is not equal to the number of bits in the modulus.
#[derive(Debug, Clone)]
pub struct ModularMultDivChip<
    T: PrimeField32,
    const CARRY_LIMBS: usize,
    const NUM_LIMBS: usize,
    const LIMB_SIZE: usize,
> {
    pub air: ModularMultDivAir<CARRY_LIMBS, NUM_LIMBS, LIMB_SIZE>,
    data: Vec<ModularMultDivRecord<T, NUM_LIMBS>>,
    memory_controller: MemoryControllerRef<T>,
    pub range_checker_chip: Arc<VariableRangeCheckerChip>,
    modulus: BigUint,

    offset: usize,
}

impl<T: PrimeField32, const CARRY_LIMBS: usize, const NUM_LIMBS: usize, const LIMB_SIZE: usize>
    ModularMultDivChip<T, CARRY_LIMBS, NUM_LIMBS, LIMB_SIZE>
{
    pub fn new(
        execution_bus: ExecutionBus,
        program_bus: ProgramBus,
        memory_controller: MemoryControllerRef<T>,
        modulus: BigUint,
        offset: usize,
    ) -> Self {
        let range_checker_chip = memory_controller.borrow().range_checker.clone();
        let memory_bridge = memory_controller.borrow().memory_bridge();
        let bus = range_checker_chip.bus();
        assert!(
            bus.range_max_bits >= LIMB_SIZE,
            "range_max_bits {} < LIMB_SIZE {}",
            bus.range_max_bits,
            LIMB_SIZE
        );
        let subair = CheckCarryModToZeroSubAir::new(
            modulus.clone(),
            LIMB_SIZE,
            bus.index,
            bus.range_max_bits,
            FIELD_ELEMENT_BITS,
        );
        Self {
            air: ModularMultDivAir {
                execution_bridge: ExecutionBridge::new(execution_bus, program_bus),
                memory_bridge,
                subair,
                offset,
            },
            data: vec![],
            memory_controller,
            range_checker_chip,
            modulus,
            offset,
        }
    }
}

impl<T: PrimeField32, const CARRY_LIMBS: usize, const NUM_LIMBS: usize, const LIMB_SIZE: usize>
    InstructionExecutor<T> for ModularMultDivChip<T, CARRY_LIMBS, NUM_LIMBS, LIMB_SIZE>
{
    fn execute(
        &mut self,
        instruction: Instruction<T>,
        from_state: ExecutionState<u32>,
    ) -> Result<ExecutionState<u32>, ExecutionError> {
        let Instruction {
            opcode,
            op_a: z_address_ptr,
            op_b: x_address_ptr,
            op_c: y_address_ptr,
            d,
            e,
            ..
        } = instruction;
        let local_opcode_index = opcode - self.offset;
        assert_eq!(CARRY_LIMBS, NUM_LIMBS * 2 - 1);
        assert!(LIMB_SIZE <= 10); // refer to [primitives/src/bigint/README.md]

        let mut memory_controller = self.memory_controller.borrow_mut();
        debug_assert_eq!(from_state.timestamp, memory_controller.timestamp());

        let x_array_read = memory_controller.read_heap::<NUM_LIMBS>(d, e, x_address_ptr);
        let y_array_read = memory_controller.read_heap::<NUM_LIMBS>(d, e, y_address_ptr);

        let x = x_array_read.data_read.data.map(|x| x.as_canonical_u32());
        let y = y_array_read.data_read.data.map(|x| x.as_canonical_u32());

        let x_biguint = limbs_to_biguint(&x, LIMB_SIZE);
        let y_biguint = limbs_to_biguint(&y, LIMB_SIZE);

        let z_biguint = self.solve(
            ModularArithmeticOpcode::from_usize(local_opcode_index),
            x_biguint,
            y_biguint,
        );
        let z_limbs = biguint_to_limbs(z_biguint, LIMB_SIZE);

        let z_array_write = memory_controller.write_heap::<NUM_LIMBS>(
            d,
            e,
            z_address_ptr,
            z_limbs.map(|x| T::from_canonical_u32(x)),
        );

        self.data.push(ModularMultDivRecord {
            from_state,
            instruction: Instruction {
                opcode: local_opcode_index,
                ..instruction
            },
            x_array_read,
            y_array_read,
            z_array_write,
        });

        Ok(ExecutionState {
            pc: from_state.pc + 1,
            timestamp: memory_controller.timestamp(),
        })
    }

    fn get_opcode_name(&self, opcode: usize) -> String {
        let local_opcode_index = ModularArithmeticOpcode::from_usize(opcode - self.offset);
        format!(
            "{local_opcode_index:?}<{:?},{NUM_LIMBS},{LIMB_SIZE}>",
            self.modulus
        )
    }
}

impl<T: PrimeField32, const CARRY_LIMBS: usize, const NUM_LIMBS: usize, const LIMB_SIZE: usize>
    ModularMultDivChip<T, CARRY_LIMBS, NUM_LIMBS, LIMB_SIZE>
{
    pub fn solve(&self, opcode: ModularArithmeticOpcode, x: BigUint, y: BigUint) -> BigUint {
        match opcode {
            ModularArithmeticOpcode::MUL => (x * y) % self.modulus.clone(),
            ModularArithmeticOpcode::DIV => {
                let y_inv = big_uint_mod_inverse(&y, &self.modulus);
                (x * y_inv) % &self.modulus
            }
            _ => unreachable!(),
        }
    }
}