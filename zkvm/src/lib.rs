use anyhow::Result;
use powdr::backend::{
    composite::{split, CompositeProof, CompositeVerificationKey},
    BackendType,
};
use powdr::executor::constant_evaluator::get_uniquely_sized;
use powdr::number::{DegreeType, FieldElement, GoldilocksField};
use powdr::riscv::continuations::{rust_continuations, rust_continuations_dry_run};
use powdr::riscv::{compile_rust, Runtime};
use powdr::Pipeline;
use recursion::pilcom::export as pil_export;
use starky::{
    merklehash::MerkleTreeGL,
    pil2circom,
    stark_setup::StarkSetup,
    types::{StarkStruct, Step},
};
use std::fs::{self, create_dir_all /*, remove_dir_all*/};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

const TEST_CHANNEL: u32 = 1;

fn generate_witness_and_prove<F: FieldElement>(
    mut pipeline: Pipeline<F>,
) -> Result<Pipeline<F>, Vec<String>> {
    let start = Instant::now();
    log::debug!("Generating witness...");
    pipeline.compute_witness()?;
    let duration = start.elapsed();
    log::debug!("Generating witness took: {:?}", duration);

    let start = Instant::now();
    log::debug!("Proving ...");

    pipeline =
        pipeline.with_backend(BackendType::EStarkStarkyComposite, Some("stark_gl".to_string()));
    pipeline.compute_proof()?;
    let duration = start.elapsed();
    log::debug!("Proving took: {:?}", duration);
    Ok(pipeline)
}

fn generate_witness_and_prove_raw<F: FieldElement>(
    mut pipeline: Pipeline<F>,
) -> Result<(), Vec<String>> {
    let start = Instant::now();
    log::debug!("Generating witness...");
    pipeline.compute_witness()?;
    let duration = start.elapsed();
    log::debug!("Generating witness took: {:?}", duration);

    let start = Instant::now();
    log::debug!("Proving ...");

    pipeline =
        pipeline.with_backend(BackendType::EStarkStarkyComposite, Some("stark_gl".to_string()));
    pipeline.compute_proof()?;
    let duration = start.elapsed();
    log::debug!("Proving took: {:?}", duration);
    Ok(())
}

fn generate_verifier<F: FieldElement>(
    mut pipeline: Pipeline<F>,
    output_path: &str,
    task: &str,
    chunk_idx: usize,
) -> Result<Vec<usize>> {
    let buf = Vec::new();
    let mut vw = BufWriter::new(buf);
    pipeline =
        pipeline.with_backend(BackendType::EStarkStarkyComposite, Some("stark_gl".to_string()));
    pipeline.export_verification_key(&mut vw).unwrap();

    log::debug!("Init CompositeVerificationKey");
    let cvk: CompositeVerificationKey = bincode::deserialize(&vw.into_inner()?)?;

    log::debug!("Init CompositeProof");
    let proof_data = pipeline.proof().unwrap();
    let cf: CompositeProof = bincode::deserialize(proof_data)?;

    let full_pil = pipeline.optimized_pil().unwrap();
    let pils = split::split_pil((*full_pil).clone());

    log::debug!("Generate verifier for each proof");
    let mut ids: Vec<usize> = vec![];
    for (idx, (vk, machine_proof)) in
        cvk.verification_keys.iter().zip(cf.proofs.into_iter()).enumerate()
    {
        if vk.is_none() {
            continue;
        }
        let pil = pils.get(&machine_proof.machine).unwrap();
        let proof_file = Path::new(output_path)
            .join(format!("{}_chunk_{}_submachine_{}.json", task, chunk_idx, idx));

        log::debug!("Running proof generation to {:?}...", proof_file);
        fs::write(proof_file, machine_proof.proof)?;

        let verifier_file = Path::new(output_path)
            .join(format!("{}_chunk_{}_submachine_{}.circom", task, chunk_idx, idx));
        log::debug!("Running circom verifier generation to {:?}...", verifier_file);
        let mut writer = fs::File::create(verifier_file)?;

        let vk_data = vk.as_ref().unwrap().get(&machine_proof.size).unwrap();
        let mut setup: StarkSetup<MerkleTreeGL> = serde_json::from_slice(vk_data)?;
        log::debug!(
            "Load StarkSetup, machien={}, size={}",
            machine_proof.machine,
            machine_proof.size
        );

        // FIXME: get the sub machine PIL
        //let pil = pipeline.optimized_pil().unwrap();
        //let degree = pil.degree();
        let fixed_cols = pipeline.fixed_cols().unwrap();
        let degree = get_uniquely_sized(&fixed_cols)
            .unwrap()
            .iter()
            .find(|(col, _)| col == "main.STEP")
            .unwrap()
            .1
            .len() as u64;

        assert!(degree > 1);
        let n_bits = (DegreeType::BITS - (degree - 1).leading_zeros()) as usize;
        let n_bits_ext = n_bits + 1;

        let steps = (2..=n_bits_ext).rev().step_by(4).map(|b| Step { nBits: b }).collect();

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
            ids.push(idx);
        } else {
            log::info!("No public vars in {}", machine_proof.machine);
        }
    }
    Ok(ids)
}

pub fn zkvm_execute_and_prove(task: &str, suite_json: String, output_path: &str) -> Result<()> {
    log::debug!("Compiling Rust...");
    let force_overwrite = true;
    let with_bootloader = true;
    let (asm_file_path, asm_contents) = compile_rust::<GoldilocksField>(
        &format!("program/{task}"),
        Path::new(output_path),
        force_overwrite,
        &Runtime::base().with_poseidon(),
        false,
        with_bootloader,
    )
    .unwrap();

    let mut pipeline = Pipeline::<GoldilocksField>::default()
        .with_output(output_path.into(), force_overwrite)
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

    let bootloader_inputs = rust_continuations_dry_run(&mut pipeline, Default::default());

    let duration = start.elapsed();
    log::debug!("Trace executor took: {:?}", duration);

    log::debug!("Running witness generation...");
    let start = Instant::now();

    rust_continuations(pipeline, generate_witness_and_prove_raw, bootloader_inputs).unwrap();

    let duration = start.elapsed();
    log::debug!("Witness generation took: {:?}", duration);

    Ok(())
}

pub fn zkvm_generate_chunks(
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
        &Runtime::base().with_poseidon(),
        false,
        with_bootloader,
    )
    .unwrap();

    let mut pipeline = Pipeline::<GoldilocksField>::default()
        .with_output(output_path.into(), force_overwrite)
        .from_asm_string(asm_contents.clone(), Some(asm_file_path.clone()))
        .with_prover_inputs(Default::default())
        .add_data(TEST_CHANNEL, suite_json);

    log::debug!("Running powdr-riscv executor in fast mode...");
    pipeline.compute_fixed_cols().unwrap();

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

    let bootloader_inputs = rust_continuations_dry_run(&mut pipeline, Default::default());

    let duration = start.elapsed();
    log::debug!("Trace executor took: {:?}, input size: {:?}", duration, bootloader_inputs.len());

    Ok(bootloader_inputs)
}

pub fn zkvm_prove_only(
    task: &str,
    suite_json: &String,
    bootloader_input: Vec<GoldilocksField>,
    start_of_shutdown_routine: u64,
    i: usize,
    output_path: &str,
) -> Result<Vec<usize>> {
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
    let pipeline = rust_continuation(
        task,
        pipeline,
        generate_witness_and_prove,
        bootloader_input,
        start_of_shutdown_routine,
        i,
    )
    .unwrap();

    let ids = generate_verifier(pipeline, output_path, task, i)?;

    let duration = start.elapsed();
    log::debug!("Witness generation and proof computation took: {:?}", duration);

    Ok(ids)
}

pub fn rust_continuation<F: FieldElement, PipelineCallback, E>(
    task: &str,
    mut pipeline: Pipeline<F>,
    pipeline_callback: PipelineCallback,
    bootloader_inputs: Vec<F>,
    start_of_shutdown_routine: u64,
    i: usize,
) -> Result<Pipeline<F>, E>
where
    PipelineCallback: Fn(Pipeline<F>) -> Result<Pipeline<F>, E>,
{
    let fixed_cols = pipeline.compute_fixed_cols().unwrap();

    // Advance the pipeline to the optimized PIL stage, so that it doesn't need to be computed
    // in every chunk.
    pipeline.compute_optimized_pil().unwrap();

    let length = get_uniquely_sized(&fixed_cols)
        .unwrap()
        .iter()
        .find(|(col, _)| col == "main.STEP")
        .unwrap()
        .1
        .len() as u64;

    let name = format!("{}_chunk_{}", task, i);
    log::debug!("\nRunning chunk {} in {}...", i + 1, name);

    // we used to do
    //let pipeline = pipeline.with_name(name);

    // now we should do
    let parent_path = pipeline.output_dir().as_ref().unwrap();
    let chunk_dir = parent_path.join(name);
    //remove_dir_all(&chunk_dir).unwrap();
    create_dir_all(&chunk_dir).unwrap();
    let pipeline = pipeline.with_output(chunk_dir, true);

    let jump_to_shutdown_routine =
        (0..length).map(|i| (i == start_of_shutdown_routine - 1).into()).collect();

    let pipeline = pipeline.add_external_witness_values(vec![
        ("main_bootloader_inputs.value".to_string(), bootloader_inputs),
        ("main.jump_to_shutdown_routine".to_string(), jump_to_shutdown_routine),
    ]);
    pipeline_callback(pipeline)
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::identities::Zero;
    use std::io::{Read, Write};

    // RUST_MIN_STACK=2073741821 RUST_LOG=debug nohup cargo test --release test_zkvm_prove -- --nocapture  &
    #[test]
    #[ignore]
    fn test_zkvm_prove() {
        env_logger::try_init().unwrap_or_default();
        let test_file = "test-vectors/reth.block.json";
        let suite_json = fs::read_to_string(test_file).unwrap();

        zkvm_execute_and_prove("evm", suite_json, "/tmp/test_evm").unwrap();
    }

    #[test]
    fn test_zkvm_lr_prove() {
        env_logger::try_init().unwrap_or_default();
        zkvm_execute_and_prove("lr", "".to_string(), "/tmp/test_lr").unwrap();
    }

    #[test]
    #[ignore]
    fn test_zkvm_lr_execute_then_prove() {
        env_logger::try_init().unwrap_or_default();
        let test_file = "test-vectors/reth.block.json";
        let suite_json = fs::read_to_string(test_file).unwrap();

        let task = "evm";
        let output_path = "/tmp/test_evm";
        let workspace = format!("program/{}", task);
        let bootloader_inputs =
            zkvm_generate_chunks(workspace.as_str(), &suite_json, output_path).unwrap();
        // save the chunks
        let bi_files: Vec<_> = (0..bootloader_inputs.len())
            .map(|i| Path::new(output_path).join(format!("{task}_chunks_{i}.data")))
            .collect();
        bootloader_inputs.iter().zip(&bi_files).for_each(|(data, filename)| {
            let mut f = fs::File::create(filename).unwrap();
            // write the start_of_shutdown_routine
            f.write_all(&data.1.to_le_bytes()).unwrap();
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
            // read the start_of_shutdown_routine
            let mut buffer = [0u8; 8];
            f.read_exact(&mut buffer).unwrap();
            let start_of_shutdown_routine: u64 = u64::from_le_bytes(buffer);
            let file_size = file_size - 8;
            let mut buffer = vec![0; file_size];
            f.read_exact(&mut buffer).unwrap();
            let mut bi = vec![GoldilocksField::zero(); file_size / 8];
            bi.iter_mut().zip(buffer.chunks(8)).for_each(|(out, bin)| {
                *out = GoldilocksField::from_bytes_le(bin);
            });

            let submachine_ids =
                zkvm_prove_only(task, &suite_json, bi, start_of_shutdown_routine, i, output_path)
                    .unwrap();
            log::info!("submachine ids: {:?}", submachine_ids);
        });
    }
}
