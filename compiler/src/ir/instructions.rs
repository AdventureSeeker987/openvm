use super::{Array, Config, Ext, Felt, MemIndex, Ptr, RVar, TracedVec, Var};
use crate::ir::modular_arithmetic::BigUintVar;

/// An intermeddiate instruction set for implementing programs.
///
/// Programs written in the DSL can compile both to the recursive zkVM and the R1CS or Plonk-ish
/// circuits.
#[derive(Debug, Clone, strum_macros::Display)]
pub enum DslIr<C: Config> {
    // Immediates.
    /// Assigns an immediate to a variable (var = imm).
    ImmV(Var<C::N>, C::N),
    /// Assigns a field immediate to a field element (felt = field imm).
    ImmF(Felt<C::F>, C::F),
    /// Assigns an ext field immediate to an extension field element (ext = ext field imm).
    ImmE(Ext<C::F, C::EF>, C::EF),

    // Additions.
    /// Add two variables (var = var + var).
    AddV(Var<C::N>, Var<C::N>, Var<C::N>),
    /// Add a variable and an immediate (var = var + imm).
    AddVI(Var<C::N>, Var<C::N>, C::N),
    /// Add two field elements (felt = felt + felt).
    AddF(Felt<C::F>, Felt<C::F>, Felt<C::F>),
    /// Add a field element and a field immediate (felt = felt + field imm).
    AddFI(Felt<C::F>, Felt<C::F>, C::F),
    /// Add two extension field elements (ext = ext + ext).
    AddE(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Ext<C::F, C::EF>),
    /// Add an extension field element and an ext field immediate (ext = ext + ext field imm).
    AddEI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::EF),
    /// Add an extension field element and a field element (ext = ext + felt).
    AddEF(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Felt<C::F>),
    /// Add an extension field element and a field immediate (ext = ext + field imm).
    AddEFI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::F),
    /// Add a field element and an ext field immediate (ext = felt + ext field imm).
    AddEFFI(Ext<C::F, C::EF>, Felt<C::F>, C::EF),
    /// Add two modular BigInts over coordinate field.
    AddSecp256k1Coord(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Add two modular BigInts over scalar field.
    AddSecp256k1Scalar(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Add two 256-bit integers
    Add256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),

    // Subtractions.
    /// Subtracts two variables (var = var - var).
    SubV(Var<C::N>, Var<C::N>, Var<C::N>),
    /// Subtracts a variable and an immediate (var = var - imm).
    SubVI(Var<C::N>, Var<C::N>, C::N),
    /// Subtracts an immediate and a variable (var = imm - var).
    SubVIN(Var<C::N>, C::N, Var<C::N>),
    /// Subtracts two field elements (felt = felt - felt).
    SubF(Felt<C::F>, Felt<C::F>, Felt<C::F>),
    /// Subtracts a field element and a field immediate (felt = felt - field imm).
    SubFI(Felt<C::F>, Felt<C::F>, C::F),
    /// Subtracts a field immediate and a field element (felt = field imm - felt).
    SubFIN(Felt<C::F>, C::F, Felt<C::F>),
    /// Subtracts two extension field elements (ext = ext - ext).
    SubE(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Ext<C::F, C::EF>),
    /// Subtrancts an extension field element and an extension field immediate (ext = ext - ext field imm).
    SubEI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::EF),
    /// Subtracts an extension field immediate and an extension field element (ext = ext field imm - ext).
    SubEIN(Ext<C::F, C::EF>, C::EF, Ext<C::F, C::EF>),
    /// Subtracts an extension field element and a field immediate (ext = ext - field imm).
    SubEFI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::F),
    /// Subtracts an extension field element and a field element (ext = ext - felt).
    SubEF(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Felt<C::F>),
    /// Subtracts two modular BigInts over coordinate field.
    SubSecp256k1Coord(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Subtracts two modular BigInts over scalar field.
    SubSecp256k1Scalar(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Subtract two 256-bit integers
    Sub256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),

    // Multiplications.
    /// Multiplies two variables (var = var * var).
    MulV(Var<C::N>, Var<C::N>, Var<C::N>),
    /// Multiplies a variable and an immediate (var = var * imm).
    MulVI(Var<C::N>, Var<C::N>, C::N),
    /// Multiplies two field elements (felt = felt * felt).
    MulF(Felt<C::F>, Felt<C::F>, Felt<C::F>),
    /// Multiplies a field element and a field immediate (felt = felt * field imm).
    MulFI(Felt<C::F>, Felt<C::F>, C::F),
    /// Multiplies two extension field elements (ext = ext * ext).
    MulE(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Ext<C::F, C::EF>),
    /// Multiplies an extension field element and an extension field immediate (ext = ext * ext field imm).
    MulEI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::EF),
    /// Multiplies an extension field element and a field immediate (ext = ext * field imm).
    MulEFI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::F),
    /// Multiplies an extension field element and a field element (ext = ext * felt).
    MulEF(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Felt<C::F>),
    /// Multiplies two modular BigInts over coordinate field.
    MulSecp256k1Coord(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Multiplies two modular BigInts over scalar field.
    MulSecp256k1Scalar(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Multiply two 256-bit integers
    Mul256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),

    // Divisions.
    /// Divides two variables (var = var / var).
    DivF(Felt<C::F>, Felt<C::F>, Felt<C::F>),
    /// Divides a field element and a field immediate (felt = felt / field imm).
    DivFI(Felt<C::F>, Felt<C::F>, C::F),
    /// Divides a field immediate and a field element (felt = field imm / felt).
    DivFIN(Felt<C::F>, C::F, Felt<C::F>),
    /// Divides two extension field elements (ext = ext / ext).
    DivE(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Ext<C::F, C::EF>),
    /// Divides an extension field element and an extension field immediate (ext = ext / ext field imm).
    DivEI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::EF),
    /// Divides and extension field immediate and an extension field element (ext = ext field imm / ext).
    DivEIN(Ext<C::F, C::EF>, C::EF, Ext<C::F, C::EF>),
    /// Divides an extension field element and a field immediate (ext = ext / field imm).
    DivEFI(Ext<C::F, C::EF>, Ext<C::F, C::EF>, C::F),
    /// Divides an extension field element and a field element (ext = ext / felt).
    DivEF(Ext<C::F, C::EF>, Ext<C::F, C::EF>, Felt<C::F>),
    /// Subtracts two modular BigInts over coordinate field.
    DivSecp256k1Coord(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Multiplies two modular BigInts over scalar field.
    DivSecp256k1Scalar(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),

    // Negations.
    /// Negates a variable (var = -var).
    NegV(Var<C::N>, Var<C::N>),
    /// Negates a field element (felt = -felt).
    NegF(Felt<C::F>, Felt<C::F>),
    /// Negates an extension field element (ext = -ext).
    NegE(Ext<C::F, C::EF>, Ext<C::F, C::EF>),

    // Comparisons.
    /// Compares two variables
    LessThanV(Var<C::N>, Var<C::N>, Var<C::N>),
    /// Compares a variable and an immediate
    LessThanVI(Var<C::N>, Var<C::N>, C::N),
    /// Compare two u256 for <
    LessThanU256(Ptr<C::N>, BigUintVar<C>, BigUintVar<C>),
    /// Compare two 256-bit integers for ==
    EqualTo256(Ptr<C::N>, BigUintVar<C>, BigUintVar<C>),
    /// Compare two signed 256-bit integers for <
    LessThanI256(Ptr<C::N>, BigUintVar<C>, BigUintVar<C>),

    // Bitwise operations.
    /// Bitwise XOR on two 256-bit integers
    Xor256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Bitwise AND on two 256-bit integers
    And256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Bitwise OR on two 256-bit integers
    Or256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),

    // Shifts.
    /// Shift left on 256-bit integers
    ShiftLeft256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Shift right logical on 256-bit integers
    ShiftRightLogic256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),
    /// Shift right arithmetic on 256-bit integers
    ShiftRightArith256(BigUintVar<C>, BigUintVar<C>, BigUintVar<C>),

    // =======

    // Control flow.
    /// Executes a for loop with the parameters (start step value, end step value, step size, step variable, body).
    For(RVar<C::N>, RVar<C::N>, C::N, Var<C::N>, TracedVec<DslIr<C>>),
    /// Executes an indefinite loop.
    Loop(TracedVec<DslIr<C>>),
    /// Executes an equal conditional branch with the parameters (lhs var, rhs var, then body, else body).
    IfEq(
        Var<C::N>,
        Var<C::N>,
        TracedVec<DslIr<C>>,
        TracedVec<DslIr<C>>,
    ),
    /// Executes a not equal conditional branch with the parameters (lhs var, rhs var, then body, else body).
    IfNe(
        Var<C::N>,
        Var<C::N>,
        TracedVec<DslIr<C>>,
        TracedVec<DslIr<C>>,
    ),
    /// Executes an equal conditional branch with the parameters (lhs var, rhs imm, then body, else body).
    IfEqI(Var<C::N>, C::N, TracedVec<DslIr<C>>, TracedVec<DslIr<C>>),
    /// Executes a not equal conditional branch with the parameters (lhs var, rhs imm, then body, else body).
    IfNeI(Var<C::N>, C::N, TracedVec<DslIr<C>>, TracedVec<DslIr<C>>),
    /// Break out of a loop.
    Break,

    // Assertions.
    /// Assert that two variables are equal (var == var).
    AssertEqV(Var<C::N>, Var<C::N>),
    /// Assert that two variables are not equal (var != var).
    AssertNeV(Var<C::N>, Var<C::N>),
    /// Assert that two field elements are equal (felt == felt).
    AssertEqF(Felt<C::F>, Felt<C::F>),
    /// Assert that two field elements are not equal (felt != felt).
    AssertNeF(Felt<C::F>, Felt<C::F>),
    /// Assert that two extension field elements are equal (ext == ext).
    AssertEqE(Ext<C::F, C::EF>, Ext<C::F, C::EF>),
    /// Assert that two extension field elements are not equal (ext != ext).
    AssertNeE(Ext<C::F, C::EF>, Ext<C::F, C::EF>),
    /// Assert that a variable is equal to an immediate (var == imm).
    AssertEqVI(Var<C::N>, C::N),
    /// Assert that a variable is not equal to an immediate (var != imm).
    AssertNeVI(Var<C::N>, C::N),
    /// Assert that a field element is equal to a field immediate (felt == field imm).
    AssertEqFI(Felt<C::F>, C::F),
    /// Assert that a field element is not equal to a field immediate (felt != field imm).
    AssertNeFI(Felt<C::F>, C::F),
    /// Assert that an extension field element is equal to an extension field immediate (ext == ext field imm).
    AssertEqEI(Ext<C::F, C::EF>, C::EF),
    /// Assert that an extension field element is not equal to an extension field immediate (ext != ext field imm).
    AssertNeEI(Ext<C::F, C::EF>, C::EF),

    // Memory instructions.
    /// Allocate (ptr, len, size) a memory slice of length len
    Alloc(Ptr<C::N>, RVar<C::N>, usize),
    /// Load variable (var, ptr, index)
    LoadV(Var<C::N>, Ptr<C::N>, MemIndex<C::N>),
    /// Load field element (var, ptr, index)
    LoadF(Felt<C::F>, Ptr<C::N>, MemIndex<C::N>),
    /// Load extension field
    LoadE(Ext<C::F, C::EF>, Ptr<C::N>, MemIndex<C::N>),
    /// Store variable at address
    StoreV(Var<C::N>, Ptr<C::N>, MemIndex<C::N>),
    /// Store field element at address
    StoreF(Felt<C::F>, Ptr<C::N>, MemIndex<C::N>),
    /// Store extension field at address
    StoreE(Ext<C::F, C::EF>, Ptr<C::N>, MemIndex<C::N>),

    // Bits.
    /// Decompose a variable into size bits (bits = num2bits(var, size)). Should only be used when target is a gnark circuit.
    CircuitNum2BitsV(Var<C::N>, usize, Vec<Var<C::N>>),
    /// Decompose a field element into bits (bits = num2bits(felt)). Should only be used when target is a gnark circuit.
    CircuitNum2BitsF(Felt<C::F>, Vec<Var<C::N>>),

    // Hashing.
    /// Permutes an array of baby bear elements using Poseidon2 (output = p2_permute(array)).
    Poseidon2PermuteBabyBear(Array<C, Felt<C::F>>, Array<C, Felt<C::F>>),
    /// Compresses two baby bear element arrays using Poseidon2 (output = p2_compress(array1, array2)).
    Poseidon2CompressBabyBear(
        Array<C, Felt<C::F>>,
        Array<C, Felt<C::F>>,
        Array<C, Felt<C::F>>,
    ),
    /// Permutes an array of Bn254 elements using Poseidon2 (output = p2_permute(array)). Should only
    /// be used when target is a gnark circuit.
    CircuitPoseidon2Permute([Var<C::N>; 3]),
    /// Permutates an array of BabyBear elements in the circuit.
    // CircuitPoseidon2PermuteBabyBear([Felt<C::F>; 16]),

    /// ```ignore
    /// Keccak256(output, input)
    /// ```
    ///
    /// Computes the keccak256 hash of variable length `input` where `input` does not have the
    /// keccak padding bits. `input` will be constrained to be bytes. The `output` pointers can
    /// overwrite the `input` memory. The `output` is in `u16` limbs, with conversion to bytes being
    /// **little-endian**. The `output` is exactly 16 limbs (32 bytes).
    Keccak256(Array<C, Var<C::N>>, Array<C, Var<C::N>>),

    /// ```ignore
    /// Secp256k1AddUnequal(dst, p, q)
    /// ```
    /// Reads `p,q` from heap and writes `dst = p + q` to heap. A point is represented on the heap
    /// as two affine coordinates concatenated together into a byte array.
    /// Assumes that `p.x != q.x` which is equivalent to `p != +-q`.
    Secp256k1AddUnequal(
        Array<C, Var<C::N>>,
        Array<C, Var<C::N>>,
        Array<C, Var<C::N>>,
    ),
    /// ```ignore
    /// Secp256k1Double(dst, p)
    /// ```
    /// Reads `p` from heap and writes `dst = p + p` to heap. A point is represented on the heap
    /// as two affine coordinates concatenated together into a byte array.
    Secp256k1Double(Array<C, Var<C::N>>, Array<C, Var<C::N>>),

    // Miscellaneous instructions.
    /// Prints a variable.
    PrintV(Var<C::N>),
    /// Prints a field element.
    PrintF(Felt<C::F>),
    /// Prints an extension field element.
    PrintE(Ext<C::F, C::EF>),
    /// Throws an error.
    Error(),

    /// Prepare next input vector (preceded by its length) for hinting.
    HintInputVec(),
    /// Prepare bit decomposition for hinting.
    HintBitsU(RVar<C::N>),
    /// Prepare bit decomposition for hinting.
    HintBitsV(Var<C::N>, u32),
    /// Prepare bit decomposition for hinting.
    HintBitsF(Felt<C::F>, u32),
    /// Prepare byte decomposition for hinting.
    HintBytes(Var<C::N>, u32),

    StoreHintWord(Ptr<C::N>, MemIndex<C::N>),

    /// Witness a variable. Should only be used when target is a gnark circuit.
    WitnessVar(Var<C::N>, u32),
    /// Witness a field element. Should only be used when target is a gnark circuit.
    WitnessFelt(Felt<C::F>, u32),
    /// Witness an extension field element. Should only be used when target is a gnark circuit.
    WitnessExt(Ext<C::F, C::EF>, u32),
    /// Label a field element as the ith public input.
    Publish(Felt<C::F>, Var<C::N>),
    /// Operation to halt the program. Should be the last instruction in the program.
    Halt,

    // Public inputs for circuits.
    /// Asserts that the inputted var is equal the circuit's vkey hash public input. Should only be
    /// used when target is a gnark circuit.
    CircuitCommitVkeyHash(Var<C::N>),
    /// Asserts that the inputted var is equal the circuit's commited values digest public input. Should
    /// only be used when target is a gnark circuit.
    CircuitCommitCommitedValuesDigest(Var<C::N>),

    // FRI specific instructions.
    /// Select's a variable based on a condition. (select(cond, true_val, false_val) => output).
    /// Should only be used when target is a gnark circuit.
    CircuitSelectV(Var<C::N>, Var<C::N>, Var<C::N>, Var<C::N>),
    /// Select's a field element based on a condition. (select(cond, true_val, false_val) => output).
    /// Should only be used when target is a gnark circuit.
    CircuitSelectF(Var<C::N>, Felt<C::F>, Felt<C::F>, Felt<C::F>),
    /// Select's an extension field element based on a condition. (select(cond, true_val, false_val) => output).
    /// Should only be used when target is a gnark circuit.
    CircuitSelectE(
        Var<C::N>,
        Ext<C::F, C::EF>,
        Ext<C::F, C::EF>,
        Ext<C::F, C::EF>,
    ),
    /// Converts an ext to a slice of felts. Should only be used when target is a gnark circuit.
    CircuitExt2Felt([Felt<C::F>; 4], Ext<C::F, C::EF>),
    /// Converts a slice of felts to an ext. Should only be used when target is a gnark circuit.
    CircuitFelts2Ext([Felt<C::F>; 4], Ext<C::F, C::EF>),

    // Debugging instructions.
    /// Executes less than (var = var < var).  This operation is NOT constrained.
    LessThan(Var<C::N>, Var<C::N>, Var<C::N>),

    /// Start the cycle tracker used by a block of code annotated by the string input. Calling this with the same
    /// string will end the open cycle tracker instance and start a new one with an increasing numeric postfix.
    CycleTrackerStart(String),
    /// End the cycle tracker used by a block of code annotated by the string input.
    CycleTrackerEnd(String),
}

impl<C: Config> Default for DslIr<C> {
    fn default() -> Self {
        Self::Halt
    }
}
