use backend::BackendType;
use powdr::executor::witgen::QueryCallback;
use powdr::number::{FieldElement, GoldilocksField};
use powdr::pipeline::{parse_query, Pipeline, Stage};
use powdr::riscv::continuations::{
    bootloader::default_input, rust_continuations, rust_continuations_dry_run,
};
use powdr::riscv::{compile_rust, CoProcessors};
use powdr::riscv_executor;
use revm::primitives::Address;
use std::collections::HashMap as STDHashMap;
use std::path::Path;
use std::time::Instant;

pub fn zkvm_evm_prove_one(
    suite_json: String,
    _addr: Address,
    _chain_id: u64,
    output_path: &str,
) -> Result<(), String> {
    println!("Compiling Rust...");
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

    println!("Creating data callback...");
    let mut suite_json_bytes: Vec<GoldilocksField> = suite_json
        .into_bytes()
        .iter()
        .map(|b| (*b as u32).into())
        .collect();
    suite_json_bytes.insert(0, (suite_json_bytes.len() as u32).into());

    let mut data: STDHashMap<GoldilocksField, Vec<GoldilocksField>> = STDHashMap::default();
    data.insert(666.into(), suite_json_bytes);

    println!("Running powdr-riscv executor in fast mode...");
    let start = Instant::now();
    let (trace, _mem) = riscv_executor::execute::<GoldilocksField>(
        &asm_contents,
        &data,
        &default_input(),
        riscv_executor::ExecMode::Fast,
    );
    let duration = start.elapsed();
    println!("Fast executor took: {:?}", duration);
    println!("Trace length: {}", trace.len);

    println!("Running powdr-riscv executor in trace mode for continuations...");
    let start = Instant::now();
    let bootloader_inputs = rust_continuations_dry_run(mk_pipeline(), data.clone());
    let duration = start.elapsed();
    println!("Trace executor took: {:?}", duration);

    let prove_with = Some(BackendType::EStark);
    let generate_witness = |pipeline: Pipeline<GoldilocksField>| -> Result<(), Vec<String>> {
        let data = data_to_query_callback(data.clone());
        let mut pipeline = pipeline.add_query_callback(Box::new(data));
        pipeline.advance_to(Stage::GeneratedWitness)?;
        prove_with.map(|backend| pipeline.with_backend(backend).proof().unwrap());
        Ok(())
    };

    println!("Running witness generation...");
    let start = Instant::now();
    rust_continuations(mk_pipeline, generate_witness, bootloader_inputs).unwrap();
    let duration = start.elapsed();
    println!("Witness generation took: {:?}", duration);
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

    use revm::primitives::address;

    #[test]
    #[ignore = "Too long"]
    fn test_zkvm_evm_prove() {
        //let test_file = "test-vectors/blockInfo.json";
        let test_file = "test-vectors/solidityExample.json";
        let suite_json = std::fs::read_to_string(test_file).unwrap();

        let addr = address!("a94f5374fce5edbc8e2a8697c15331677e6ebf0b");
        zkvm_evm_prove_one(suite_json, addr, 1, "/tmp/test").unwrap();
    }
}
