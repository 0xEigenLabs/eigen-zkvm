use crate::bellman_ce::pairing::bn256::Bn256;
use crate::{circom_circuit::CircomCircuit, plonk, reader};

#[cfg(not(feature = "wasm"))]
use crate::{aggregation, verifier};

use anyhow::Result;
use std::path::Path;

// generate a monomial_form SRS, and save it to a file
pub fn setup(power: u32, srs_monomial_form: &String) -> Result<()> {
    let srs = plonk::gen_key_monomial_form(power)?;
    let path = Path::new(srs_monomial_form);
    assert!(
        !path.exists(),
        "duplicate srs_monomial_form file: {}",
        path.display()
    );
    let writer = std::fs::File::create(srs_monomial_form)?;
    srs.write(writer)?;
    log::info!("srs_monomial_form saved to {}", srs_monomial_form);
    Result::Ok(())
}

// circuit filename default resolver
pub fn analyse(circuit_file: &String, output: &String) -> Result<()> {
    let circuit = CircomCircuit::<Bn256> {
        r1cs: reader::load_r1cs(&circuit_file),
        witness: None,
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };
    let mut stats = plonk::analyse(circuit)?;
    let writer = std::fs::File::create(output)?;
    serde_json::to_writer_pretty(writer, &stats)?;
    stats.constraint_stats.clear();
    log::info!(
        "analyse result: {}",
        serde_json::to_string_pretty(&stats).unwrap_or_else(|_| "<failed>".to_owned())
    );
    Result::Ok(())
}

pub fn prove(
    circuit_file: &String,
    witness: &String,
    srs_monomial_form: &String,
    srs_lagrange_form: Option<String>,
    transcript: &String,
    proof_bin: &String,
    proof_json: &String,
    public_json: &String,
) -> Result<()> {
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(circuit_file),
        witness: Some(reader::load_witness_from_file::<Bn256>(witness)),
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit.clone(),
        reader::load_key_monomial_form(srs_monomial_form),
        reader::maybe_load_key_lagrange_form(srs_lagrange_form),
    )?;

    let proof = setup.prove(circuit, transcript)?;
    let writer = std::fs::File::create(proof_bin)?;
    proof.write(writer)?;

    let (inputs, serialized_proof) = bellman_vk_codegen::serialize_proof(&proof);
    let ser_proof_str = serde_json::to_string_pretty(&serialized_proof)?;
    let ser_inputs_str = serde_json::to_string_pretty(&inputs)?;

    std::fs::write(proof_json, ser_proof_str.as_bytes())?;
    std::fs::write(public_json, ser_inputs_str.as_bytes())?;

    Result::Ok(())
}

pub fn export_verification_key(
    srs_monomial_form: &String,
    circuit_file: &String,
    output_vk: &String,
) -> Result<()> {
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(circuit_file),
        witness: None,
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit,
        reader::load_key_monomial_form(srs_monomial_form),
        None,
    )?;
    let vk = setup.make_verification_key()?;
    let writer = std::fs::File::create(output_vk)?;
    vk.write(writer)?;
    Result::Ok(())
}

pub fn verify(vk_file: &String, proof_bin: &String, transcript: &String) -> Result<()> {
    let vk = reader::load_verification_key::<Bn256>(vk_file);
    let proof = reader::load_proof::<Bn256>(proof_bin);
    let ok = plonk::verify(&vk, &proof, transcript)?;
    anyhow::ensure!(ok, "Proof is invalid");
    Result::Ok(())
}

pub fn generate_verifier(vk_file: &String, sol: &String) -> Result<()> {
    let vk = reader::load_verification_key::<Bn256>(vk_file);
    bellman_vk_codegen::render_verification_key_from_default_template(&vk, sol, true);
    Result::Ok(())
}

#[cfg(not(feature = "wasm"))]
pub fn export_aggregation_verification_key(
    num_proofs_to_check: usize,
    num_inputs: usize,
    srs_monomial_form: &String,
    vk_file: &String,
) -> Result<()> {
    let big_crs = reader::load_key_monomial_form(srs_monomial_form);
    let vk = aggregation::export_vk(num_proofs_to_check, num_inputs, &big_crs)?;
    let path = Path::new(vk_file);
    assert!(!path.exists(), "dumpcate proof file: {}", path.display());
    let writer = std::fs::File::create(vk_file)?;
    vk.write(writer)?;
    Result::Ok(())
}

#[cfg(not(feature = "wasm"))]
pub fn aggregation_prove(
    srs_monomial_form: &String,
    old_proof_list: &String,
    old_vk: &String,
    new_proof: &String,
    proofjson: &String,
) -> Result<()> {
    let big_crs = reader::load_key_monomial_form(srs_monomial_form);
    let old_proofs = reader::load_proofs_from_list::<Bn256>(old_proof_list);
    let old_vk = reader::load_verification_key::<Bn256>(old_vk);
    let proof = aggregation::prove(big_crs, old_proofs, old_vk)?;
    let path = Path::new(new_proof);
    assert!(
        !path.exists(),
        "dumpcate new proof file: {}",
        path.display()
    );
    let path = Path::new(proofjson);
    assert!(!path.exists(), "dumpcate proofjson: {}", path.display());

    let writer = std::fs::File::create(new_proof)?;
    proof.write(writer)?;

    let ser_proof_str = serde_json::to_string_pretty(&proof)?;
    std::fs::write(proofjson, ser_proof_str.as_bytes())?;
    Result::Ok(())
}

#[cfg(not(feature = "wasm"))]
pub fn aggregation_verify(proof: &String, vk: &String) -> Result<()> {
    let vk = reader::load_aggregation_verification_key(vk);
    let proof = reader::load_aggregated_proof(proof);
    let correct = aggregation::verify(vk, proof)?;
    anyhow::ensure!(correct, "Proof is invalid");
    Result::Ok(())
}

// check an aggregated proof is corresponding to the original proofs
#[cfg(not(feature = "wasm"))]
pub fn aggregation_check(
    old_proof_list: &String,
    old_vk: &String,
    new_proof: &String,
) -> Result<()> {
    let old_proofs = reader::load_proofs_from_list::<Bn256>(old_proof_list);
    let old_vk = reader::load_verification_key::<Bn256>(old_vk);
    let new_proof = reader::load_aggregated_proof(new_proof);

    let expected = aggregation::get_aggregated_input(old_proofs, old_vk)?;

    anyhow::ensure!(
        expected == new_proof.proof.inputs[0],
        "Aggregation hash input mismatch",
    );
    Result::Ok(())
}

#[cfg(not(feature = "wasm"))]
pub fn generate_aggregation_verifier(
    raw_vk_file: &String,
    aggregation_vk_file: &String,
    num_inputs: usize,
    sol: &String,
) -> Result<()> {
    let old_vk = reader::load_verification_key::<Bn256>(raw_vk_file);
    let aggregation_vk = reader::load_aggregation_verification_key(aggregation_vk_file);
    let config = aggregation::Config {
        vk_tree_root: aggregation::get_vk_tree_root_hash(old_vk)?,
        //vk_max_index: 0, //because we has aggregated only 1 vk
        individual_input_num: num_inputs,
        aggregation_vk,
    };
    verifier::recursive_plonk_verifier::create_verifier_contract_from_default_template(config, sol);
    Result::Ok(())
}
