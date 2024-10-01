use std::{cell::RefCell, rc::Rc, sync::Arc};

use afs_primitives::{
    bigint::{check_carry_mod_to_zero::CheckCarryModToZeroSubAir, utils::*},
    ecc::SampleEcPoints,
    sub_chip::LocalTraceInstructions,
    var_range::{bus::VariableRangeCheckerBus, VariableRangeCheckerChip},
};
use ax_sdk::{
    any_rap_vec, config::baby_bear_blake3::BabyBearBlake3Engine, engine::StarkFriEngine,
    utils::create_seeded_rng,
};
use num_bigint_dig::BigUint;
use p3_air::BaseAir;
use p3_baby_bear::BabyBear;
use p3_matrix::dense::RowMajorMatrix;
use rand::RngCore;

use super::{super::utils::*, ExprBuilder, FieldExprChip, SymbolicExpr};

const LIMB_BITS: usize = 8;

pub fn generate_random_biguint(prime: &BigUint) -> BigUint {
    let mut rng = create_seeded_rng();
    let len = 32;
    let x = (0..len).map(|_| rng.next_u32()).collect();
    let x = BigUint::new(x);
    x % prime
}

fn get_sub_air(prime: &BigUint) -> (CheckCarryModToZeroSubAir, Arc<VariableRangeCheckerChip>) {
    let field_element_bits = 30;
    let range_bus = 1;
    let range_decomp = 17; // double needs 17, rests need 16.
    let range_checker = Arc::new(VariableRangeCheckerChip::new(VariableRangeCheckerBus::new(
        range_bus,
        range_decomp,
    )));
    let subair = CheckCarryModToZeroSubAir::new(
        prime.clone(),
        LIMB_BITS,
        range_bus,
        range_decomp,
        field_element_bits,
    );
    (subair, range_checker)
}

#[test]
fn test_add() {
    let prime = secp256k1_coord_prime();
    let (subair, range_checker) = get_sub_air(&prime);

    let builder = ExprBuilder::new(prime.clone(), LIMB_BITS, 32);
    let builder = Rc::new(RefCell::new(builder));
    let x1 = ExprBuilder::new_input(builder.clone());
    let x2 = ExprBuilder::new_input(builder.clone());
    let mut x3 = x1 + x2;
    x3.save();
    let builder = builder.borrow().clone();

    let chip = FieldExprChip {
        builder,
        check_carry_mod_to_zero: subair,
        range_checker: range_checker.clone(),
    };

    let x = generate_random_biguint(&prime);
    let y = generate_random_biguint(&prime);
    let expected = (&x + &y) % prime;
    let inputs = vec![x, y];

    let row = chip.generate_trace_row((inputs, range_checker.clone()));
    let (_, _, vars, _, _) = chip.load_vars(&row);
    assert_eq!(vars.len(), 1);
    let generated = evaluate_biguint(&vars[0], LIMB_BITS);
    assert_eq!(generated, expected);

    let trace = RowMajorMatrix::new(row, BaseAir::<BabyBear>::width(&chip));
    let range_trace = range_checker.generate_trace();

    BabyBearBlake3Engine::run_simple_test_no_pis(
        &any_rap_vec![&chip, &range_checker.air],
        vec![trace, range_trace],
    )
    .expect("Verification failed");
}

#[test]
fn test_div() {
    let prime = secp256k1_coord_prime();
    let (subair, range_checker) = get_sub_air(&prime);

    let builder = ExprBuilder::new(prime.clone(), LIMB_BITS, 32);
    let builder = Rc::new(RefCell::new(builder));
    let x1 = ExprBuilder::new_input(builder.clone());
    let x2 = ExprBuilder::new_input(builder.clone());
    let _x3 = x1 / x2; // auto save on division.
    let builder = builder.borrow().clone();

    let chip = FieldExprChip {
        builder,
        check_carry_mod_to_zero: subair,
        range_checker: range_checker.clone(),
    };

    let x = generate_random_biguint(&prime);
    let y = generate_random_biguint(&prime);
    let y_inv = big_uint_mod_inverse(&y, &prime);
    let expected = (&x * &y_inv) % prime;
    let inputs = vec![x, y];

    let row = chip.generate_trace_row((inputs, range_checker.clone()));
    let (_, _, vars, _, _) = chip.load_vars(&row);
    assert_eq!(vars.len(), 1);
    let generated = evaluate_biguint(&vars[0], LIMB_BITS);
    assert_eq!(generated, expected);

    let trace = RowMajorMatrix::new(row, BaseAir::<BabyBear>::width(&chip));
    let range_trace = range_checker.generate_trace();

    BabyBearBlake3Engine::run_simple_test_no_pis(
        &any_rap_vec![&chip, &range_checker.air],
        vec![trace, range_trace],
    )
    .expect("Verification failed");
}

#[test]
fn test_ec_add() {
    let prime = secp256k1_coord_prime();
    let (subair, range_checker) = get_sub_air(&prime);

    let builder = ExprBuilder::new(prime.clone(), LIMB_BITS, 32);
    let builder = Rc::new(RefCell::new(builder));
    let x1 = ExprBuilder::new_input(builder.clone());
    let y1 = ExprBuilder::new_input(builder.clone());
    let x2 = ExprBuilder::new_input(builder.clone());
    let y2 = ExprBuilder::new_input(builder.clone());
    let dx = x2.clone() - x1.clone();
    let dy = y2.clone() - y1.clone();
    let lambda = dy / dx; // auto save on division.
    let mut x3 = lambda.clone() * lambda.clone() - x1.clone() - x2;
    x3.save();
    let mut y3 = lambda * (x1 - x3) - y1;
    y3.save();
    let builder = builder.borrow().clone();

    let chip = FieldExprChip {
        builder,
        check_carry_mod_to_zero: subair,
        range_checker: range_checker.clone(),
    };

    let (x1, y1) = SampleEcPoints[0].clone();
    let (x2, y2) = SampleEcPoints[1].clone();
    let (expected_x3, expected_y3) = SampleEcPoints[2].clone();
    let inputs = vec![x1, y1, x2, y2];

    let row = chip.generate_trace_row((inputs, range_checker.clone()));
    let (_, _, vars, _, _) = chip.load_vars(&row);
    assert_eq!(vars.len(), 3); // lambda, x3, y3
    let generated_x3 = evaluate_biguint(&vars[1], LIMB_BITS);
    let generated_y3 = evaluate_biguint(&vars[2], LIMB_BITS);
    assert_eq!(generated_x3, expected_x3);
    assert_eq!(generated_y3, expected_y3);

    let trace = RowMajorMatrix::new(row, BaseAir::<BabyBear>::width(&chip));
    let range_trace = range_checker.generate_trace();

    BabyBearBlake3Engine::run_simple_test_no_pis(
        &any_rap_vec![&chip, &range_checker.air],
        vec![trace, range_trace],
    )
    .expect("Verification failed");
}

#[test]
fn test_ec_double() {
    let prime = secp256k1_coord_prime();
    let (subair, range_checker) = get_sub_air(&prime);

    let builder = ExprBuilder::new(prime.clone(), LIMB_BITS, 32);
    let builder = Rc::new(RefCell::new(builder));
    let x1 = ExprBuilder::new_input(builder.clone());
    let y1 = ExprBuilder::new_input(builder.clone());
    let nom = (x1.clone() * x1.clone()).int_mul(3);
    let denom = y1.int_mul(2);
    let lambda = nom / denom;
    let mut x3 = lambda.clone() * lambda.clone() - x1.clone() - x1.clone();
    x3.save();
    let mut y3 = lambda * (x1 - x3) - y1;
    y3.save();
    let builder = builder.borrow().clone();

    let chip = FieldExprChip {
        builder,
        check_carry_mod_to_zero: subair,
        range_checker: range_checker.clone(),
    };

    let (x1, y1) = SampleEcPoints[1].clone();
    let (expected_x3, expected_y3) = SampleEcPoints[3].clone();
    let inputs = vec![x1, y1];

    let row = chip.generate_trace_row((inputs, range_checker.clone()));
    let (_, _, vars, _, _) = chip.load_vars(&row);
    assert_eq!(vars.len(), 3); // lambda, x3, y3
    let generated_x3 = evaluate_biguint(&vars[1], LIMB_BITS);
    let generated_y3 = evaluate_biguint(&vars[2], LIMB_BITS);
    assert_eq!(generated_x3, expected_x3);
    assert_eq!(generated_y3, expected_y3);

    let trace = RowMajorMatrix::new(row, BaseAir::<BabyBear>::width(&chip));
    let range_trace = range_checker.generate_trace();

    BabyBearBlake3Engine::run_simple_test_no_pis(
        &any_rap_vec![&chip, &range_checker.air],
        vec![trace, range_trace],
    )
    .expect("Verification failed");
}

fn test_symbolic_limbs(expr: SymbolicExpr, expected_q: usize, expected_carry: usize) {
    let prime = secp256k1_coord_prime();
    let (q, carry) = expr.constraint_limbs(&prime, LIMB_BITS, 32);
    assert_eq!(q, expected_q);
    assert_eq!(carry, expected_carry);
}

#[test]
fn test_symbolic_limbs_add() {
    let expr = SymbolicExpr::Add(
        Box::new(SymbolicExpr::Var(0)),
        Box::new(SymbolicExpr::Var(1)),
    );
    // x + y = pq, q should fit in q limb.
    // x+y should have 32 limbs, pq also 32 limbs.
    let expected_q = 1;
    let expected_carry = 32;
    test_symbolic_limbs(expr, expected_q, expected_carry);
}

#[test]
fn test_symbolic_limbs_sub() {
    let expr = SymbolicExpr::Sub(
        Box::new(SymbolicExpr::Var(0)),
        Box::new(SymbolicExpr::Var(1)),
    );
    // x - y = pq, q should fit in q limb.
    // x - y should have 32 limbs, pq also 32 limbs.
    let expected_q = 1;
    let expected_carry = 32;
    test_symbolic_limbs(expr, expected_q, expected_carry);
}

#[test]
fn test_symbolic_limbs_mul() {
    let expr = SymbolicExpr::Mul(
        Box::new(SymbolicExpr::Var(0)),
        Box::new(SymbolicExpr::Var(1)),
    );
    // x * y = pq, q can be up to p so can limbs as p.
    // x * y and p * q  both have 63 limbs.
    let expected_q = 32;
    let expected_carry = 63;
    test_symbolic_limbs(expr, expected_q, expected_carry);
}