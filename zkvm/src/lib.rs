use backend::BackendType;
use powdr::executor::witgen::QueryCallback;
use powdr::number::{FieldElement, GoldilocksField};
use powdr::pipeline::{parse_query, Pipeline, Stage};
use powdr::riscv::continuations::{
    bootloader::default_input, rust_continuations, rust_continuations_dry_run,
};
use powdr::riscv::{compile_rust, CoProcessors};
use powdr::riscv_executor;
use std::collections::HashMap as STDHashMap;
use std::path::Path;
use std::time::Instant;

pub fn zkvm_evm_prove_one(
    suite_json: String,
    addr: &str,
    chain_id: u32,
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
    };

    log::debug!("Creating data callback...");
    let mut suite_json_bytes: Vec<GoldilocksField> = suite_json
        .into_bytes()
        .iter()
        .map(|b| (*b as u32).into())
        .collect();
    suite_json_bytes.insert(0, (suite_json_bytes.len() as u32).into());

    let mut data: STDHashMap<GoldilocksField, Vec<GoldilocksField>> = STDHashMap::default();
    data.insert(666.into(), suite_json_bytes);
    data.insert(
        667.into(),
        [1, chain_id].iter().map(|b| (*b).into()).collect(),
    );
    let mut addr: Vec<_> = addr.as_bytes().iter().map(|b| (*b as u32).into()).collect();
    addr.insert(0, (addr.len() as u32).into());
    data.insert(668.into(), addr);

    log::debug!("Running powdr-riscv executor in fast mode...");
    let start = Instant::now();
    let (trace, _mem) = riscv_executor::execute::<GoldilocksField>(
        &asm_contents,
        &data,
        &default_input(),
        riscv_executor::ExecMode::Fast,
    );
    let duration = start.elapsed();
    log::debug!("Fast executor took: {:?}", duration);
    log::debug!("Trace length: {}", trace.len);

    log::debug!("Running powdr-riscv executor in trace mode for continuations...");
    let start = Instant::now();
    let bootloader_inputs = rust_continuations_dry_run(mk_pipeline(), data.clone());
    let duration = start.elapsed();
    log::debug!("Trace executor took: {:?}", duration);

    let prove_with = Some(BackendType::EStark);
    let generate_witness = |pipeline: Pipeline<GoldilocksField>| -> Result<(), Vec<String>> {
        let data = data_to_query_callback(data.clone());
        let mut pipeline = pipeline.add_query_callback(Box::new(data));
        pipeline.advance_to(Stage::GeneratedWitness)?;
        prove_with.map(|backend| pipeline.with_backend(backend).proof().unwrap());
        Ok(())
    };

    log::debug!("Running witness generation...");
    let start = Instant::now();
    rust_continuations(mk_pipeline, generate_witness, bootloader_inputs).unwrap();
    let duration = start.elapsed();
    log::debug!("Witness generation took: {:?}", duration);
    Ok(())
}

fn access_element<T: FieldElement>(
    name: &str,
    elements: &[T],
    index_str: &str,
) -> Result<Option<T>, String> {
    let index = index_str
        .parse::<usize>()
        .map_err(|e| format!("Error parsing index: {e})"))?;
    let value = elements.get(index).cloned();
    if let Some(value) = value {
        Ok(Some(value))
    } else {
        Err(format!(
            "Error accessing {name}: Index {index} out of bounds {}",
            elements.len()
        ))
    }
}

#[allow(clippy::print_stdout)]
fn data_to_query_callback<T: FieldElement>(data: STDHashMap<T, Vec<T>>) -> impl QueryCallback<T> {
    move |query: &str| -> Result<Option<T>, String> {
        // TODO In the future, when match statements need to be exhaustive,
        // This function probably gets an Option as argument and it should
        // answer None by Ok(None).

        match &parse_query(query)?[..] {
            ["\"input\"", index] => access_element("prover inputs", &data[&T::zero()], index),
            ["\"data\"", index, what] => {
                let what = what
                    .parse::<usize>()
                    .map_err(|e| format!("Error parsing what: {e})"))?;

                access_element("prover inputs", &data[&(what as u64).into()], index)
            }
            ["\"print_char\"", ch] => {
                print!(
                    "{}",
                    ch.parse::<u8>()
                        .map_err(|e| format!("Invalid char to print: {e}"))?
                        as char
                );
                // We do not answer None because we don't want this function to be
                // called again.
                Ok(Some(0.into()))
            }
            ["\"hint\"", value] => Ok(Some(T::from_str(value))),
            _k => Err("Unsupported query".to_string()),
        }
    }
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

        let addr = "a94f5374fce5edbc8e2a8697c15331677e6ebf0b";
        zkvm_evm_prove_one(suite_json, addr, 1, "/tmp/test").unwrap();
    }
}
