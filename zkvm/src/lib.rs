use anyhow::Result;
use backend::BackendType;
use powdr::number::GoldilocksField;
use powdr::pipeline::{Pipeline, Stage};
use powdr::riscv::continuations::{
    bootloader::default_input, rust_continuations, rust_continuations_dry_run,
};
use powdr::riscv::{compile_rust, CoProcessors};
use powdr::riscv_executor;
use std::path::Path;
use std::time::Instant;

pub fn zkvm_evm_prove_one(task: &str, suite_json: String, output_path: &str) -> Result<()> {
    log::debug!("Compiling Rust...");
    let force_overwrite = true;
    let with_bootloader = true;
    let (asm_file_path, asm_contents) = compile_rust(
        &format!("vm/{task}"),
        Path::new(output_path),
        force_overwrite,
        &CoProcessors::base().with_poseidon(),
        with_bootloader,
    )
    .ok_or_else(|| vec!["could not compile rust".to_string()])
    .unwrap();

    let mk_pipeline = || {
        Pipeline::<GoldilocksField>::default()
            .from_asm_string(asm_contents.clone(), Some(asm_file_path.clone()))
            .with_prover_inputs(vec![])
    };

    log::debug!("Creating pipeline from powdr-asm...");
    let start = Instant::now();
    let pipeline = mk_pipeline();
    let duration = start.elapsed();
    log::debug!("Pipeline from powdr-asm took: {:?}", duration);

    log::debug!("Advancing pipeline to fixed columns...");
    let start = Instant::now();
    let pil_with_evaluated_fixed_cols = pipeline.pil_with_evaluated_fixed_cols().unwrap();
    let duration = start.elapsed();
    log::debug!("Advancing pipeline took: {:?}", duration);

    let mk_pipeline_with_data = || mk_pipeline().add_data(666, &suite_json);

    let mk_pipeline_opt = || {
        mk_pipeline_with_data()
            .from_pil_with_evaluated_fixed_cols(pil_with_evaluated_fixed_cols.clone())
    };

    log::debug!("Running powdr-riscv executor in fast mode...");
    let start = Instant::now();
    let (trace, _mem) = riscv_executor::execute::<GoldilocksField>(
        &asm_contents,
        mk_pipeline_with_data().data_callback().unwrap(),
        &default_input(&[]),
        riscv_executor::ExecMode::Fast,
    );
    let duration = start.elapsed();
    log::debug!("Fast executor took: {:?}", duration);
    log::debug!("Trace length: {}", trace.len);

    log::debug!("Running powdr-riscv executor in trace mode for continuations...");
    let start = Instant::now();
    let bootloader_inputs = rust_continuations_dry_run(mk_pipeline_with_data());
    let duration = start.elapsed();
    log::debug!("Trace executor took: {:?}", duration);

    let prove_with = Some(BackendType::EStark);

    let generate_witness_and_prove =
        |mut pipeline: Pipeline<GoldilocksField>| -> Result<(), Vec<String>> {
            let start = Instant::now();
            log::debug!("Generating witness...");
            pipeline.advance_to(Stage::GeneratedWitness)?;
            let duration = start.elapsed();
            log::debug!("Generating witness took: {:?}", duration);

            let start = Instant::now();
            log::debug!("Proving ...");
            prove_with.map(|backend| pipeline.with_backend(backend).proof().unwrap());
            let duration = start.elapsed();
            log::debug!("Proving took: {:?}", duration);
            Ok(())
        };

    log::debug!("Running witness generation...");
    let start = Instant::now();
    rust_continuations(
        mk_pipeline_opt,
        generate_witness_and_prove,
        bootloader_inputs,
    )
    .unwrap();
    let duration = start.elapsed();
    log::debug!("Witness generation took: {:?}", duration);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::zkvm_evm_prove_one;

    //use revm::primitives::address;

    // RUST_MIN_STACK=2073741821 RUST_LOG=debug proxychains nohup cargo test --release test_zkvm_evm_prove -- --nocapture  &
    #[test]
    #[ignore]
    fn test_zkvm_evm_prove() {
        env_logger::try_init().unwrap_or_default();
        //let test_file = "test-vectors/blockInfo.json";
        let test_file = "test-vectors/solidityExample.json";
        let suite_json = std::fs::read_to_string(test_file).unwrap();

        zkvm_evm_prove_one("evm", suite_json, "/tmp/test_evm").unwrap();
    }

    #[test]
    #[ignore]
    fn test_zkvm_lr_prove() {
        env_logger::try_init().unwrap_or_default();
        //let test_file = "test-vectors/blockInfo.json";
        let test_file = "test-vectors/solidityExample.json";
        let suite_json = std::fs::read_to_string(test_file).unwrap();

        zkvm_evm_prove_one("lr", suite_json, "/tmp/test_lr").unwrap();
    }
}
