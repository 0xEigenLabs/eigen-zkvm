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

pub fn zkvm_evm_prove_one(
    suite_json: String,
    output_path: &str,
) -> Result<(), String> {
    log::debug!("Compiling Rust...");
    let force_overwrite = true;
    let with_bootloader = true;
    let (asm_file_path, asm_contents) = compile_rust(
        "vm/evm",
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

    #[test]
    #[ignore]
    fn test_zkvm_evm_prove() {
        env_logger::try_init().unwrap_or_default();
        //let test_file = "test-vectors/blockInfo.json";
        let test_file = "test-vectors/solidityExample.json";
        let suite_json = std::fs::read_to_string(test_file).unwrap();

        /*
        let map_caller_keys: HashMap<_, _> = [
            (
                b256!("45a915e4d060149eb4365960e6a7a45f334393093061116b197e3240065ff2d8"),
                address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b"),
            ),
            (
                b256!("c85ef7d79691fe79573b1a7064c19c1a9819ebdbd1faaab1a8ec92344438aaf4"),
                address!("cd2a3d9f938e13cd947ec05abc7fe734df8dd826"),
            ),
            (
                b256!("044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d"),
                address!("82a978b3f5962a5b0957d9ee9eef472ee55b42f1"),
            ),
            (
                b256!("6a7eeac5f12b409d42028f66b0b2132535ee158cfda439e3bfdd4558e8f4bf6c"),
                address!("c9c5a15a403e41498b6f69f6f89dd9f5892d21f7"),
            ),
            (
                b256!("a95defe70ebea7804f9c3be42d20d24375e2a92b9d9666b832069c5f3cd423dd"),
                address!("3fb1cd2cd96c6d5c0b5eb3322d807b34482481d4"),
            ),
            (
                b256!("fe13266ff57000135fb9aa854bbfe455d8da85b21f626307bf3263a0c2a8e7fe"),
                address!("dcc5ba93a1ed7e045690d722f2bf460a51c61415"),
            ),
        ]
        .into();
        */

        zkvm_evm_prove_one(suite_json, "/tmp/test").unwrap();
    }
}
