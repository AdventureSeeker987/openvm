/// A E2E benchmark to aggregate a small program with common VM chips.
/// Proofs:
/// 1. Prove a program with some common operations.
/// 2. Verify the proof of 1. in the inner config.
/// 2. Verify the proof of 2. in the outer config.
/// 3. Verify the proof of 3. using a Halo2 static verifier.
/// 4. Wrapper Halo2 circuit to reduce the size of 4.
use afs_compiler::{
    asm::AsmBuilder,
    conversion::CompilerOptions,
    ir::{Ext, Felt, RVar, Var},
};
use afs_recursion::testing_utils::inner::build_verification_program;
use ax_sdk::{
    bench::run_with_metric_collection,
    config::{baby_bear_poseidon2::BabyBearPoseidon2Engine, FriParameters},
    engine::{StarkForTest, StarkFriEngine},
};
use p3_baby_bear::BabyBear;
use p3_commit::PolynomialSpace;
use p3_field::{extension::BinomialExtensionField, AbstractField};
use p3_uni_stark::{Domain, StarkGenericConfig};
use stark_vm::{program::Program, sdk::gen_vm_program_stark_for_test, vm::config::VmConfig};
use tracing::info_span;

/// A simple benchmark program to run most operations: keccak256, field arithmetic, field extension,
/// for loop, if-then statement
fn bench_program() -> Program<BabyBear> {
    type F = BabyBear;
    type EF = BinomialExtensionField<BabyBear, 4>;

    let mut builder = AsmBuilder::<F, EF>::default();

    let n: Var<_> = builder.eval(F::from_canonical_u32(2));
    let arr = builder.dyn_array(n);
    let v: Var<_> = builder.eval(F::from_canonical_u32(0));
    builder.range(RVar::zero(), n).for_each(|i, builder| {
        builder.if_eq(i, F::from_canonical_u32(0)).then(|builder| {
            builder.assign(&v, v + F::from_canonical_u32(2));
        });
        builder.assign(&v, v + F::from_canonical_u32(3));
        builder.set_value(&arr, i, v);
    });
    builder.keccak256(&arr);
    let f1: Felt<_> = builder.eval(F::from_canonical_u32(1));
    let f2: Felt<_> = builder.eval(F::from_canonical_u32(2));
    let _: Felt<_> = builder.eval(f1 + f2);
    let ext1: Ext<_, _> = builder.eval(F::from_canonical_u32(1));
    let ext2: Ext<_, _> = builder.eval(F::from_canonical_u32(2));
    let _: Ext<_, _> = builder.eval(ext1 + ext2);

    builder.halt();

    builder.compile_isa()
}

fn bench_program_stark_for_test<SC: StarkGenericConfig>() -> StarkForTest<SC>
where
    Domain<SC>: PolynomialSpace<Val = BabyBear>,
{
    let fib_program = bench_program();

    let vm_config = VmConfig {
        compress_poseidon2_enabled: false,
        perm_poseidon2_enabled: false,
        keccak_enabled: true,
        field_arithmetic_enabled: true,
        ..Default::default()
    };
    gen_vm_program_stark_for_test(fib_program, vec![], vm_config)
}

fn main() {
    run_with_metric_collection("OUTPUT_PATH", || {
        let (vdata, pvs) = info_span!("Bench Program Inner", group = "bench_program_inner")
            .in_scope(|| {
                let program_stark = bench_program_stark_for_test();
                let pvs = program_stark.pvs.clone();
                (
                    program_stark
                        .run_simple_test(&BabyBearPoseidon2Engine::new(FriParameters {
                            log_blowup: 4,
                            num_queries: 24,
                            proof_of_work_bits: 16,
                        }))
                        .unwrap(),
                    pvs,
                )
            });

        let compiler_options = CompilerOptions {
            enable_cycle_tracker: true,
            ..Default::default()
        };
        let (vdata, pvs) = info_span!("Inner Verifier", group = "inner_verifier").in_scope(|| {
            let (program, witness_stream) =
                build_verification_program(pvs, vdata, compiler_options);
            let inner_verifier_stf = gen_vm_program_stark_for_test(
                program,
                witness_stream,
                VmConfig {
                    num_public_values: 4,
                    ..Default::default()
                },
            );
            let pvs = inner_verifier_stf.pvs.clone();
            (
                inner_verifier_stf
                    .run_simple_test(&BabyBearPoseidon2Engine::new(
                        // log_blowup = 3 because of poseidon2 chip.
                        FriParameters {
                            log_blowup: 3,
                            num_queries: 33,
                            proof_of_work_bits: 16,
                        },
                    ))
                    .unwrap(),
                pvs,
            )
        });

        #[cfg(feature = "static-verifier")]
        info_span!("Recursive Verify e2e", group = "recursive_verify_e2e").in_scope(|| {
            let (program, witness_stream) =
                build_verification_program(pvs, vdata, compiler_options);
            let outer_verifier_sft = gen_vm_program_stark_for_test(
                program,
                witness_stream,
                VmConfig {
                    num_public_values: 4,
                    ..Default::default()
                },
            );
            afs_recursion::halo2::testing_utils::run_evm_verifier_e2e_test(
                &outer_verifier_sft,
                // log_blowup = 3 because of poseidon2 chip.
                Some(FriParameters {
                    log_blowup: 3,
                    num_queries: 33,
                    proof_of_work_bits: 16,
                }),
            );
        });
    });
}