#![allow(unused_variables)]
#![allow(unused_imports)]
use std::rc::Rc;

use ax_stark_backend::p3_field::AbstractField;
use ax_stark_sdk::{
    bench::run_with_metric_collection,
    config::{baby_bear_poseidon2::BabyBearPoseidon2Engine, FriParameters},
    engine::StarkFriEngine,
    p3_baby_bear::BabyBear,
};
use axvm_benchmarks::utils::{bench_from_exe, build_bench_program, BenchmarkCli};
use axvm_circuit::arch::instructions::exe::AxVmExe;
use axvm_keccak256_circuit::Keccak256Rv32Config;
use axvm_native_compiler::conversion::CompilerOptions;
use axvm_native_recursion::testing_utils::inner::build_verification_program;
use axvm_rv32im_transpiler::{
    Rv32ITranspilerExtension, Rv32IoTranspilerExtension, Rv32MTranspilerExtension,
};
use axvm_sdk::{config::AppConfig, StdIn};
use axvm_transpiler::{transpiler::Transpiler, FromElf};
use clap::Parser;
use eyre::Result;
use tracing::info_span;

fn main() -> Result<()> {
    let cli_args = BenchmarkCli::parse();
    let app_log_blowup = cli_args.app_log_blowup.unwrap_or(2);
    // let agg_log_blowup = cli_args.agg_log_blowup.unwrap_or(2);

    let elf = build_bench_program("revm_transfer")?;
    let exe = AxVmExe::from_elf(
        elf,
        Transpiler::<BabyBear>::default()
            .with_extension(Rv32ITranspilerExtension)
            .with_extension(Rv32MTranspilerExtension)
            .with_extension(Rv32IoTranspilerExtension),
    )?;
    let app_config = AppConfig {
        app_fri_params: FriParameters::standard_with_100_bits_conjectured_security(app_log_blowup),
        app_vm_config: Keccak256Rv32Config::default(),
        leaf_fri_params: FriParameters::standard_with_100_bits_conjectured_security(1).into(),
        compiler_options: CompilerOptions::default().with_cycle_tracker(),
    };
    run_with_metric_collection("OUTPUT_PATH", || -> Result<()> {
        info_span!("revm 100 transfers").in_scope(|| {
            bench_from_exe("revm_transfer", app_config, exe, StdIn::default(), false)
        })?;
        Ok(())
    })
}
