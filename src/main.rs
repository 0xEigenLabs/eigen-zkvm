use eigen_zkit::{reader, plonk, circom_circuit::CircomCircuit};
use eigen_zkit::bellman_ce::pairing::bn256::Bn256;
use eigen_zkit::{recursive, verifier};
use std::path::Path;
use std::env;

pub fn analyse(circuit_file: &String, output: &String) {
    let circuit = CircomCircuit{
        r1cs: reader::load_r1cs(&circuit_file),
        witness: None,
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };
    let mut stats = plonk::analyse(circuit).expect("plonk analyse failed");
    let writer = std::fs::File::create(output).unwrap();
    serde_json::to_writer_pretty(writer, &stats).expect("write failed");
    stats.constraint_stats.clear();
    log::info!(
        "analyse result: {}",
        serde_json::to_string_pretty(&stats).unwrap_or_else(|_| "<failed>".to_owned())
    );
}

pub fn prove(
    circuit_file: &String,
    witness: &String,
    srs_monomial_form: &String,
    srs_lagrange_form: Option<String>,
    transcript: &String,
    proof_bin: &String,
    proof_json: &String,
    public_json: &String
    ) {
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(circuit_file),
        witness: Some(reader::load_witness_from_file::<Bn256>(witness)),
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };
    println!("load circuit finished");

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit.clone(),
        reader::load_key_monomial_form(srs_monomial_form),
        reader::maybe_load_key_lagrange_form(srs_lagrange_form),
    ).expect("setup error");

    let proof = setup.prove(circuit, transcript).expect("prove error");
    let writer = std::fs::File::create(proof_bin).unwrap();
    proof.write(writer).unwrap();

    let (inputs, serialized_proof) = bellman_vk_codegen::serialize_proof(&proof);
    let ser_proof_str = serde_json::to_string_pretty(&serialized_proof).unwrap();
    let ser_inputs_str = serde_json::to_string_pretty(&inputs).unwrap();

    std::fs::write(proof_json, ser_proof_str.as_bytes()).expect("save proof json error");
    std::fs::write(public_json, ser_inputs_str.as_bytes()).expect("save public json error");
}

pub fn export_verification_key(
    srs_monomial_form: &String,
    circuit_file: &String,
    output_vk: &String
) {
    let circuit = CircomCircuit {
        r1cs: reader::load_r1cs(circuit_file),
        witness: None,
        wire_mapping: None,
        aux_offset: plonk::AUX_OFFSET,
    };

    let setup = plonk::SetupForProver::prepare_setup_for_prover(
        circuit,
        reader::load_key_monomial_form(srs_monomial_form),
        None).expect("setup error");
    let vk = setup.make_verification_key().unwrap();
    let writer = std::fs::File::create(output_vk).unwrap();
    vk.write(writer).unwrap();
    println!("Verification key saved to {}", output_vk);
}

pub fn verify(
    vk_file: &String,
    proof_bin: &String,
    transcript: &String
) {
    let vk = reader::load_verification_key::<Bn256>(vk_file);
    let proof = reader::load_proof::<Bn256>(proof_bin);
    let ok = plonk::verify(&vk, &proof, transcript).expect("failed to verify proof");
    if ok {
        println!("Proof is valid");
    } else {
        println!("Proof is invalid");
        std::process::exit(400);
    }
}

pub fn generate_verifier(
    vk_file: &String,
    sol: &String,
) {
    let vk = reader::load_verification_key::<Bn256>(vk_file);
    bellman_vk_codegen::render_verification_key_from_default_template(&vk, sol);
    println!("Generate verifier {} done", sol);
}

pub fn export_recursive_verification_key(num_proofs_to_check: usize, num_inputs: usize, srs_monomial_form: &String, vk_file: &String) {
    let big_crs = reader::load_key_monomial_form(srs_monomial_form);
    let vk = recursive::export_vk(num_proofs_to_check, num_inputs, &big_crs).expect("must export recursive vk");
    let path = Path::new(vk_file);
    assert!(!path.exists(), "dumpcate proof file: {}", path.display());
    let writer = std::fs::File::create(vk_file).unwrap();
    vk.write(writer).unwrap();
}

pub fn recursive_prove(srs_monomial_form: &String, old_proof_list: &String, old_vk: &String, new_proof: &String, proofjson: &String) {
    let big_crs = reader::load_key_monomial_form(srs_monomial_form);
    let old_proofs = reader::load_proofs_from_list::<Bn256>(old_proof_list);
    let old_vk = reader::load_verification_key::<Bn256>(old_vk);
    let proof = recursive::prove(big_crs, old_proofs, old_vk).unwrap();
    let path = Path::new(new_proof);
    assert!(!path.exists(), "dumpcate new proof file: {}", path.display());
    let path = Path::new(proofjson);
    assert!(!path.exists(), "dumpcate proofjson: {}", path.display());

    let writer = std::fs::File::create(new_proof).unwrap();
    proof.write(writer).unwrap();

    let ser_proof_str = serde_json::to_string_pretty(&proof).unwrap();
    std::fs::write(proofjson, ser_proof_str.as_bytes()).expect("save proofjson error");
}

pub fn recursive_verify(proof: &String, vk: &String) {
    let vk = reader::load_recursive_verification_key(vk);
    let proof = reader::load_aggregated_proof(proof);
    let correct = recursive::verify(vk, proof).expect("fail to verify recursive proof");
    if correct {
        log::info!("Proof is valid");
    } else {
        log::info!("Proof is invalid");
        std::process::exit(400);
    }
}

// check an aggregated proof is corresponding to the original proofs
pub fn check_aggregation(old_proof_list: &String, old_vk: &String, new_proof: &String) {
    let old_proofs = reader::load_proofs_from_list::<Bn256>(old_proof_list);
    let old_vk = reader::load_verification_key::<Bn256>(old_vk);
    let new_proof = reader::load_aggregated_proof(new_proof);

    let expected = recursive::get_aggregated_input(old_proofs, old_vk).expect("fail to get aggregated input");
    log::info!("hash to input: {:?}", expected);
    log::info!("new_proof's input: {:?}", new_proof.proof.inputs[0]);

    if expected == new_proof.proof.inputs[0] {
        log::info!("Aggregation hash input match");
    } else {
        log::error!("Aggregation hash input mismatch");
    }
}

pub fn generate_recursive_verifier(
    raw_vk_file: &String,
    recursive_vk_file: &String,
    num_inputs: usize,
    sol: &String,
) {
    let old_vk = reader::load_verification_key::<Bn256>(raw_vk_file);
    let recursive_vk = reader::load_recursive_verification_key(recursive_vk_file);
    let config = recursive::Config {
        vk_tree_root: recursive::get_vk_tree_root_hash(old_vk).unwrap(),
        //vk_max_index: 0, //because we has aggregated only 1 vk
        individual_input_num: num_inputs,
        recursive_vk,
    };
    verifier::recursive_plonk_verifier::create_verifier_contract_from_default_template(config, sol);
}



fn main() {
    use std::time::{SystemTime};

    let arguments: Vec<String> = env::args().collect();
    // generate proof
    let circuit_file = &arguments[1];
    let witness = &arguments[2];
    let srs_monomial_form = &arguments[3];
    let start = SystemTime::now();
    prove(circuit_file, witness, srs_monomial_form, None,
        &String::from("keccak"),
        &String::from("proof.bin"),
        &String::from("proof.json"),
        &String::from("public.json"));

    export_verification_key(srs_monomial_form, circuit_file, &String::from("vk.bin"));

    verify(
        &String::from("vk.bin"),
        &String::from("proof.bin"),
        &String::from("keccak"),
    );

    generate_verifier(
        &String::from("vk.bin"),
        &String::from("verifier.sol")
    );

    let end = SystemTime::now();
    println!("time cost: {}", end.duration_since(start).unwrap().as_secs());

}
