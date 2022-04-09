use EigenZKit::{reader, plonk, circom_circuit::CircomCircuit};
use EigenZKit::bellman_ce::pairing::bn256::Bn256;
extern crate bellman_vk_codegen;
use std::env;

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

fn main() {
    let arguments: Vec<String> = env::args().collect();
    // generate proof
    let circuit_file = &arguments[1];
    let witness = &arguments[2];
    let srs_monomial_form = &arguments[3];
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
}
