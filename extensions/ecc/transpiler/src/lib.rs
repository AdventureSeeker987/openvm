use openvm_ecc_guest::{SwBaseFunct7, OPCODE, SW_FUNCT3};
use openvm_instructions::{
    instruction::Instruction, riscv::RV32_REGISTER_NUM_LIMBS, PhantomDiscriminant, UsizeOpcode,
    VmOpcode,
};
use openvm_instructions_derive::UsizeOpcode;
use openvm_stark_backend::p3_field::PrimeField32;
use openvm_transpiler::{util::from_r_type, TranspilerExtension};
use rrs_lib::instruction_formats::RType;
use strum::{EnumCount, EnumIter, FromRepr};

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, EnumCount, EnumIter, FromRepr, UsizeOpcode,
)]
#[opcode_offset = 0x600]
#[allow(non_camel_case_types)]
#[repr(usize)]
pub enum Rv32WeierstrassOpcode {
    EC_ADD_NE,
    SETUP_EC_ADD_NE,
    EC_DOUBLE,
    SETUP_EC_DOUBLE,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, FromRepr)]
#[repr(u16)]
pub enum EccPhantom {
    HintDecompress = 0x40,
}

#[derive(Default)]
pub struct EccTranspilerExtension;

impl<F: PrimeField32> TranspilerExtension<F> for EccTranspilerExtension {
    fn process_custom(&self, instruction_stream: &[u32]) -> Option<(Instruction<F>, usize)> {
        if instruction_stream.is_empty() {
            return None;
        }
        let instruction_u32 = instruction_stream[0];
        let opcode = (instruction_u32 & 0x7f) as u8;
        let funct3 = ((instruction_u32 >> 12) & 0b111) as u8;

        if opcode != OPCODE {
            return None;
        }
        if funct3 != SW_FUNCT3 {
            return None;
        }

        let instruction = {
            // short weierstrass ec
            assert!(
                Rv32WeierstrassOpcode::COUNT <= SwBaseFunct7::SHORT_WEIERSTRASS_MAX_KINDS as usize
            );
            let dec_insn = RType::new(instruction_u32);
            let base_funct7 = (dec_insn.funct7 as u8) % SwBaseFunct7::SHORT_WEIERSTRASS_MAX_KINDS;
            let curve_idx =
                ((dec_insn.funct7 as u8) / SwBaseFunct7::SHORT_WEIERSTRASS_MAX_KINDS) as usize;
            let curve_idx_shift = curve_idx * Rv32WeierstrassOpcode::COUNT;
            if let Some(SwBaseFunct7::HintDecompress) = SwBaseFunct7::from_repr(base_funct7) {
                assert_eq!(dec_insn.rd, 0);
                return Some((
                    Instruction::phantom(
                        PhantomDiscriminant(EccPhantom::HintDecompress as u16),
                        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * dec_insn.rs1),
                        F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * dec_insn.rs2),
                        curve_idx as u16,
                    ),
                    1,
                ));
            }
            if base_funct7 == SwBaseFunct7::SwSetup as u8 {
                let local_opcode = match dec_insn.rs2 {
                    0 => Rv32WeierstrassOpcode::SETUP_EC_DOUBLE,
                    _ => Rv32WeierstrassOpcode::SETUP_EC_ADD_NE,
                };
                Some(Instruction::new(
                    VmOpcode::from_usize(local_opcode.with_default_offset() + curve_idx_shift),
                    F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * dec_insn.rd),
                    F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * dec_insn.rs1),
                    F::from_canonical_usize(RV32_REGISTER_NUM_LIMBS * dec_insn.rs2),
                    F::ONE, // d_as = 1
                    F::TWO, // e_as = 2
                    F::ZERO,
                    F::ZERO,
                ))
            } else {
                let global_opcode = match SwBaseFunct7::from_repr(base_funct7) {
                    Some(SwBaseFunct7::SwAddNe) => {
                        Rv32WeierstrassOpcode::EC_ADD_NE as usize
                            + Rv32WeierstrassOpcode::default_offset()
                    }
                    Some(SwBaseFunct7::SwDouble) => {
                        assert!(dec_insn.rs2 == 0);
                        Rv32WeierstrassOpcode::EC_DOUBLE as usize
                            + Rv32WeierstrassOpcode::default_offset()
                    }
                    _ => unimplemented!(),
                };
                let global_opcode = global_opcode + curve_idx_shift;
                Some(from_r_type(global_opcode, 2, &dec_insn))
            }
        };
        instruction.map(|instruction| (instruction, 1))
    }
}
