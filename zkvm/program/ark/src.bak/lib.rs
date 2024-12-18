#![no_std]
extern crate alloc;
use alloc::{vec, vec::Vec};
use bn::Fr;
use sha2::{Digest, Sha256};
use core::error::Error;
use sp1_sdk::SP1ProofWithPublicValues;

mod ark_converter;

//use runtime::get_prover_input;

pub fn load<Path>(path: impl AsRef<Path>) -> Result<SP1ProofWithPublicValues> {
    bincode::deserialize_from(File::open(path).expect("failed to open file"))
        .map_err(Into::into)
}

/// Hashes the public inputs in the same format as the Plonk and Groth16 verifiers.
pub fn hash_public_inputs(public_inputs: &[u8]) -> [u8; 32] {
    let mut result = Sha256::digest(public_inputs);

    // The Plonk and Groth16 verifiers operate over a 254 bit field, so we need to zero
    // out the first 3 bits. The same logic happens in the SP1 Ethereum verifier contract.
    result[0] &= 0x1F;

    result.into()
}

pub fn decode_sp1_vkey_hash(sp1_vkey_hash: &str) -> Result<[u8; 32], dyn Error> {
    let bytes = hex::decode(&sp1_vkey_hash[2..]).map_err("err")?;
    bytes.try_into().map_err("err")
}

#[no_mangle]
pub fn main() {
    use ark_bn254::Bn254;
    use ark_groth16::{r1cs_to_qap::LibsnarkReduction, Groth16};
    let GROTH16_VK_BYTES: &'static [u8] = include_bytes!("groth16_vk.bin");
    // Location of the serialized SP1ProofWithPublicValues. See README.md for more information.
    // let proof_file = "test_binaries/fibonacci-groth16.bin";
    // let proof_file = "test_binaries/proof-with-is-prime.bin";
    // let proof_file = "test_binaries/proof-with-io.bin";
    let proof_file = "test_binaries/proof-with-json.bin";

    // Load the saved proof and extract the proof and public inputs.
    let sp1_proof_with_public_values = load(proof_file).unwrap();

    let proof = sp1_proof_with_public_values.bytes();
    let public_inputs = sp1_proof_with_public_values.public_values.to_vec();

    // This vkey hash was derived by calling `vk.bytes32()` on the verifying key.
    // let vkey_hash = "0x00d6052e6398c0190c062a92e52072fb98f1104cbb5c3a99893a2cdf0edd233d"; // fib
    // let vkey_hash = "0x00b609323ad6326a41d4f787c1fda6a319cbc8c03d1f4f6abb7c6b303e6d676c"; // prime
    // let vkey_hash = "0x00e66619227acf37db237ff093103c471ccf2cdc1533ae256423f7a21b4cdfd3"; // io
    let vkey_hash = "0x00df8bddc7ae33a58a0fd7037aff1121f1a8c9a50eae6e6c19d9d4fc45be10e7"; // json

    let proof = ark_converter::load_ark_proof_from_bytes(&proof[4..]).unwrap();
    let vkey = ark_converter::load_ark_groth16_verifying_key_from_bytes(&GROTH16_VK_BYTES).unwrap();

    let public_inputs = ark_converter::load_ark_public_inputs_from_bytes(
        &decode_sp1_vkey_hash(vkey_hash).unwrap(),
        &hash_public_inputs(&public_inputs),
    );

    let res = Groth16::<Bn254, LibsnarkReduction>::verify_proof(&vkey.into(), &proof, &public_inputs)
        .unwrap();
}
