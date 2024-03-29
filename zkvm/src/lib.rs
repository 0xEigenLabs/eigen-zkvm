use anyhow::Result;
use powdr::backend::BackendType;
use powdr::number::FieldElement;
use powdr::number::GoldilocksField;
use powdr::pipeline::{Pipeline, Stage};
use powdr::riscv::continuations::{
    bootloader::default_input, rust_continuations, rust_continuations_dry_run,
};
use powdr_riscv::{compile_rust, CoProcessors};
use std::path::Path;
use std::time::Instant;

pub fn zkvm_evm_execute_and_prove(task: &str, suite_json: String, output_path: &str) -> Result<()> {
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
            .with_output(output_path.into(), true)
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
    let (trace, _mem) = powdr_riscv_executor::execute::<GoldilocksField>(
        &asm_contents,
        mk_pipeline_with_data().data_callback().unwrap(),
        &default_input(&[]),
        powdr_riscv_executor::ExecMode::Fast,
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

pub fn zkvm_evm_generate_chunks(
    workspace: &str,
    suite_json: &String,
    output_path: &str,
) -> Result<Vec<Vec<GoldilocksField>>> {
    log::debug!("Compiling Rust...");
    let force_overwrite = true;
    let with_bootloader = true;
    let (asm_file_path, asm_contents) = compile_rust(
        workspace,
        Path::new(output_path),
        force_overwrite,
        &CoProcessors::base().with_poseidon(),
        with_bootloader,
    )
    .ok_or_else(|| vec!["could not compile rust".to_string()])
    .unwrap();

    let mk_pipeline = || {
        Pipeline::<GoldilocksField>::default()
            .with_output(output_path.into(), true)
            .from_asm_string(asm_contents.clone(), Some(asm_file_path.clone()))
            .with_prover_inputs(vec![])
    };

    let mk_pipeline_with_data = || mk_pipeline().add_data(666, suite_json);

    log::debug!("Running powdr-riscv executor in fast mode...");

    let (trace, _mem) = powdr_riscv_executor::execute::<GoldilocksField>(
        &asm_contents,
        mk_pipeline_with_data().data_callback().unwrap(),
        &default_input(&[]),
        powdr_riscv_executor::ExecMode::Fast,
    );
    log::debug!("Trace length: {}", trace.len);

    log::debug!("Running powdr-riscv executor in trace mode for continuations...");
    let start = Instant::now();
    let bootloader_inputs = rust_continuations_dry_run(mk_pipeline_with_data());
    let duration = start.elapsed();
    log::debug!(
        "Trace executor took: {:?}, input size: {:?}",
        duration,
        bootloader_inputs[0].len()
    );
    Ok(bootloader_inputs)
}

pub fn zkvm_evm_prove_only(
    task: &str,
    suite_json: &String,
    bootloader_input: Vec<GoldilocksField>,
    i: usize,
    output_path: &str,
) -> Result<()> {
    log::debug!("Compiling Rust...");
    let asm_file_path = Path::new(output_path).join(format!("{}.asm", task));

    let mk_pipeline = || {
        Pipeline::<GoldilocksField>::default()
            .with_output(output_path.into(), true)
            .from_asm_file(asm_file_path.clone())
            .with_prover_inputs(vec![])
    };
    let mk_pipeline_with_data = || mk_pipeline().add_data(666, suite_json);

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
    rust_continuation(
        mk_pipeline_with_data,
        generate_witness_and_prove,
        bootloader_input,
        i,
    )
    .unwrap();
    let duration = start.elapsed();
    log::debug!("Witness generation took: {:?}", duration);
    Ok(())
}

pub fn rust_continuation<F: FieldElement, PipelineFactory, PipelineCallback, E>(
    pipeline_factory: PipelineFactory,
    pipeline_callback: PipelineCallback,
    bootloader_inputs: Vec<F>,
    i: usize,
) -> Result<(), E>
where
    PipelineFactory: Fn() -> Pipeline<F>,
    PipelineCallback: Fn(Pipeline<F>) -> Result<(), E>,
{
    let num_chunks = bootloader_inputs.len();

    log::info!("Advancing pipeline to PilWithEvaluatedFixedCols stage...");
    let pipeline = pipeline_factory();
    let pil_with_evaluated_fixed_cols = pipeline.pil_with_evaluated_fixed_cols().unwrap();

    // This returns the same pipeline as pipeline_factory() (with the same name, output dir, etc...)
    // but starting from the PilWithEvaluatedFixedCols stage. This is more efficient, because we can advance
    // to that stage once before we branch into different chunks.
    let optimized_pipeline_factory = || {
        pipeline_factory().from_pil_with_evaluated_fixed_cols(pil_with_evaluated_fixed_cols.clone())
    };

    log::info!("\nRunning chunk {} / {}...", i + 1, num_chunks);
    let pipeline = optimized_pipeline_factory();
    let name = format!("{}_chunk_{}", pipeline.name(), i);
    let pipeline = pipeline.with_name(name);
    let pipeline = pipeline.add_external_witness_values(vec![(
        "main.bootloader_input_value".to_string(),
        bootloader_inputs,
    )]);
    pipeline_callback(pipeline)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::identities::Zero;
    use std::io::{Read, Write};

    use std::fs;

    //use revm::primitives::address;

    // RUST_MIN_STACK=2073741821 RUST_LOG=debug proxychains nohup cargo test --release test_zkvm_evm_prove -- --nocapture  &
    #[test]
    #[ignore]
    fn test_zkvm_evm_prove() {
        env_logger::try_init().unwrap_or_default();
        //let test_file = "test-vectors/blockInfo.json";
        let test_file = "test-vectors/solidityExample.json";
        let suite_json = fs::read_to_string(test_file).unwrap();

        zkvm_evm_execute_and_prove("evm", suite_json, "/tmp/test_evm").unwrap();
    }

    #[test]
    #[ignore]
    fn test_zkvm_lr_prove() {
        env_logger::try_init().unwrap_or_default();
        //let test_file = "test-vectors/blockInfo.json";
        let test_file = "test-vectors/solidityExample.json";
        let suite_json = fs::read_to_string(test_file).unwrap();

        zkvm_evm_execute_and_prove("lr", suite_json, "/tmp/test_lr").unwrap();
    }

    #[test]
    #[ignore]
    fn test_zkvm_lr_execute_then_prove() {
        env_logger::try_init().unwrap_or_default();
        //let test_file = "test-vectors/blockInfo.json";
        let test_file = "test-vectors/solidityExample.json";
        let suite_json = fs::read_to_string(test_file).unwrap();

        let output_path = "/tmp/test_lr";
        let task = "lr";
        let workspace = format!("vm/{}", task);
        let bootloader_inputs =
            zkvm_evm_generate_chunks(workspace.as_str(), &suite_json, output_path).unwrap();
        // save the chunks
        let bi_files: Vec<_> = (0..bootloader_inputs.len())
            .map(|i| Path::new(output_path).join(format!("{task}_chunks_{i}.data")))
            .collect();
        bootloader_inputs
            .iter()
            .zip(&bi_files)
            .for_each(|(data, filename)| {
                let mut f = fs::File::create(filename).unwrap();
                for d in data {
                    f.write_all(&d.to_bytes_le()[0..8]).unwrap();
                }
            });

        // load each chunk, generate witness and prove
        bi_files.iter().enumerate().for_each(|(i, filename)| {
            let mut f = fs::File::open(filename).unwrap();
            let metadata = fs::metadata(filename).unwrap();
            let file_size = metadata.len() as usize;
            assert!(file_size % 8 == 0);
            let mut buffer = vec![0; file_size];
            f.read_exact(&mut buffer).unwrap();
            let mut bi = vec![GoldilocksField::zero(); file_size / 8];
            bi.iter_mut().zip(buffer.chunks(8)).for_each(|(out, bin)| {
                *out = GoldilocksField::from_bytes_le(bin);
            });

            zkvm_evm_prove_only(task, &suite_json, bi, i, output_path).unwrap();
        });
    }
}
