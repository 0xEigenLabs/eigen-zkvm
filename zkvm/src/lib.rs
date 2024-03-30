use anyhow::Result;
use powdr::backend::BackendType;
use powdr::number::{DegreeType, FieldElement, GoldilocksField};
use powdr::riscv::continuations::{rust_continuations, rust_continuations_dry_run};
use powdr::riscv::{compile_rust, CoProcessors};
use powdr::Pipeline;
use recursion::pilcom::export as pil_export;
use starky::{
    merklehash::MerkleTreeGL,
    pil2circom,
    stark_setup::StarkSetup,
    types::{StarkStruct, Step},
};
use std::fs;
use std::path::Path;
use std::time::Instant;

const TEST_CHANNEL: u32 = 1;

fn generate_witness_and_prove<F: FieldElement>(
    mut pipeline: Pipeline<F>,
) -> Result<(), Vec<String>> {
    let start = Instant::now();
    log::debug!("Generating witness...");
    pipeline.compute_witness().unwrap();
    let duration = start.elapsed();
    log::debug!("Generating witness took: {:?}", duration);

    let start = Instant::now();
    log::debug!("Proving ...");

    pipeline = pipeline.with_backend(BackendType::EStark);
    pipeline.compute_proof().unwrap();
    let duration = start.elapsed();
    log::debug!("Proving took: {:?}", duration);
    Ok(())
}

fn generate_verifier<F: FieldElement, W: std::io::Write>(
    mut pipeline: Pipeline<F>,
    mut writer: W,
) -> Result<()> {
    // TODO: don't write it to disk, we should discuss with powdr-labs to provide a function for
    //pipeline to return the vk directly.
    let mut tf = tempfile::tempfile().unwrap();
    pipeline = pipeline.with_backend(BackendType::EStark);
    pipeline.export_verification_key(&mut tf).unwrap();
    let mut setup: StarkSetup<MerkleTreeGL> = serde_json::from_reader(tf).unwrap();

    let pil = pipeline.optimized_pil().unwrap();

    let degree = pil.degree();
    assert!(degree > 1);
    let n_bits = (DegreeType::BITS - (degree - 1).leading_zeros()) as usize;
    let n_bits_ext = n_bits + 1;

    let steps = (2..=n_bits_ext)
        .rev()
        .step_by(4)
        .map(|b| Step { nBits: b })
        .collect();

    let params = StarkStruct {
        nBits: n_bits,
        nBitsExt: n_bits_ext,
        nQueries: 2,
        verificationHashType: "GL".to_owned(),
        steps,
    };

    // generate circom
    let opt = pil2circom::StarkOption {
        enable_input: false,
        verkey_input: false,
        skip_main: true,
        agg_stage: false,
    };
    if !setup.starkinfo.qs.is_empty() {
        let pil_json = pil_export::<F>(pil);
        let str_ver = pil2circom::pil2circom(
            &pil_json,
            &setup.const_root,
            &params,
            &mut setup.starkinfo,
            &mut setup.program,
            &opt,
        )
        .unwrap();
        writer.write_fmt(format_args!("{}", str_ver))?;
    }
    Ok(())
}

pub fn zkvm_evm_execute_and_prove(task: &str, suite_json: String, output_path: &str) -> Result<()> {
    log::debug!("Compiling Rust...");
    let force_overwrite = true;
    let with_bootloader = true;
    let (asm_file_path, asm_contents) = compile_rust::<GoldilocksField>(
        &format!("program/{task}"),
        Path::new(output_path),
        force_overwrite,
        &CoProcessors::base().with_poseidon(),
        with_bootloader,
    )
    .ok_or_else(|| vec!["could not compile rust".to_string()])
    .unwrap();

    let mut pipeline = Pipeline::<GoldilocksField>::default()
        .with_output(output_path.into(), true)
        .from_asm_string(asm_contents.clone(), Some(asm_file_path.clone()))
        .with_prover_inputs(Default::default())
        .add_data(TEST_CHANNEL, &suite_json);

    log::debug!("Computing fixed columns...");
    let start = Instant::now();

    pipeline.compute_fixed_cols().unwrap();

    let duration = start.elapsed();
    log::debug!("Computing fixed columns took: {:?}", duration);

    /*
    log::debug!("Running powdr-riscv executor in fast mode...");
    let start = Instant::now();

    let (trace, _mem) = powdr::riscv_executor::execute::<GoldilocksField>(
        &asm_contents,
        powdr::riscv_executor::MemoryState::new(),
        pipeline.data_callback().unwrap(),
        &default_input(&[]),
        powdr::riscv_executor::ExecMode::Fast,
    );
    let duration = start.elapsed();
    log::debug!("Fast executor took: {:?}", duration);
    log::debug!("Trace length: {}", trace.len);
    */

    log::debug!("Running powdr-riscv executor in trace mode for continuations...");
    let start = Instant::now();

    let bootloader_inputs = rust_continuations_dry_run(&mut pipeline);

    let duration = start.elapsed();
    log::debug!("Trace executor took: {:?}", duration);

    log::debug!("Running witness generation...");
    let start = Instant::now();

    rust_continuations(pipeline, generate_witness_and_prove, bootloader_inputs).unwrap();

    let duration = start.elapsed();
    log::debug!("Witness generation took: {:?}", duration);

    Ok(())
}

pub fn zkvm_evm_generate_chunks(
    workspace: &str,
    suite_json: &String,
    output_path: &str,
) -> Result<Vec<(Vec<GoldilocksField>, u64)>> {
    log::debug!("Compiling Rust...");
    let force_overwrite = true;
    let with_bootloader = true;
    let (asm_file_path, asm_contents) = compile_rust::<GoldilocksField>(
        workspace,
        Path::new(output_path),
        force_overwrite,
        &CoProcessors::base().with_poseidon(),
        with_bootloader,
    )
    .ok_or_else(|| vec!["could not compile rust".to_string()])
    .unwrap();

    let mut pipeline = Pipeline::<GoldilocksField>::default()
        .with_output(output_path.into(), true)
        .from_asm_string(asm_contents.clone(), Some(asm_file_path.clone()))
        .with_prover_inputs(Default::default())
        .add_data(TEST_CHANNEL, suite_json);

    log::debug!("Running powdr-riscv executor in fast mode...");

    /*
    let (trace, _mem) = powdr::riscv_executor::execute::<GoldilocksField>(
        &asm_contents,
        powdr::riscv_executor::MemoryState::new(),
        pipeline.data_callback().unwrap(),
        &default_input(&[]),
        powdr::riscv_executor::ExecMode::Fast,
    );

    log::debug!("Trace length: {}", trace.len);
    */
    log::debug!("Running powdr-riscv executor in trace mode for continuations...");
    let start = Instant::now();

    let bootloader_inputs = rust_continuations_dry_run(&mut pipeline);

    let duration = start.elapsed();
    log::debug!(
        "Trace executor took: {:?}, input size: {:?}",
        duration,
        bootloader_inputs.len()
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

    let pipeline = Pipeline::<GoldilocksField>::default()
        .with_output(output_path.into(), true)
        .from_asm_file(asm_file_path.clone())
        .with_prover_inputs(Default::default())
        .add_data(TEST_CHANNEL, suite_json);

    log::debug!("Running witness generation and proof computation...");
    let start = Instant::now();

    //TODO: if we clone it, we lost the information gained from this function
    rust_continuation(
        pipeline.clone(),
        generate_witness_and_prove,
        bootloader_input,
        i,
    )
    .unwrap();

    let verifier_file = Path::new(output_path).join(format!("{}_chunk_{}.circom", task, i));
    log::debug!(
        "Running circom verifier generation to {:?}...",
        verifier_file
    );
    let f = fs::File::create(verifier_file)?;
    generate_verifier(pipeline, f).unwrap();

    let duration = start.elapsed();
    log::debug!(
        "Witness generation and proof computation took: {:?}",
        duration
    );

    Ok(())
}

pub fn rust_continuation<F: FieldElement, PipelineCallback, E>(
    mut pipeline: Pipeline<F>,
    pipeline_callback: PipelineCallback,
    bootloader_inputs: Vec<F>,
    i: usize,
) -> Result<(), E>
where
    PipelineCallback: Fn(Pipeline<F>) -> Result<(), E>,
{
    // Here the fixed columns most likely will have been computed already,
    // in which case this will be a no-op.
    pipeline.compute_fixed_cols().unwrap();

    log::info!("\nRunning chunk {}...", i + 1);
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
        let workspace = format!("program/{}", task);
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
                for d in &data.0 {
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
