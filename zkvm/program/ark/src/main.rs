#![no_std]
#![no_main]

// use std::fs::File;
// use std::io::{Read, Write};
// use std::io::{BufReader, BufWriter};
use ark_ec::bn::Bn;
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize, SerializationError};
use ark_std::vec;

// For randomness (during paramgen and proof generation)
// use ark_std::rand::{Rng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};
use ark_ff::vec::Vec;
use powdr_riscv_runtime::{print, io};
// For benchmarking

// Bring in some tools for using pairing-friendly curves
// We're going to use the BLS12-377 pairing-friendly elliptic curve.
// use ark_bls12_377::{Bls12_377, Fr};
use ark_bn254::{Bn254, Config, Fr};
use ark_ff::{BigInt, Field, Fp};
use ark_std::test_rng;
use ark_groth16::{ProvingKey, PreparedVerifyingKey};
use ark_crypto_primitives::snark::{CircuitSpecificSetupSNARK, SNARK};
// We'll use these interfaces to construct our circuit.
use ark_relations::{
    lc, ns,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

const TEST_CHANNEL: u32 = 666;

// static PROOF_BYTES: &[u8] = include_bytes!("proof.bin");
static PVK_BYTES: &[u8] = include_bytes!("pvk.bin");

#[no_mangle]
fn main() {
    // use ark_groth16::Groth16;
    // use rand_chacha::ChaCha20Rng;
    // use rand_core::{RngCore, SeedableRng};
    // use ark_ff::Field;
    // use core::time::Duration;

    // let rand: u64 = 10393729187455219830;
    // // // Generate the MiMC round constants as finite field elements
    // let constants = (0..MIMC_ROUNDS)
    //     .map(|_| Fr::from(rand))
    //     .collect::<Vec<_>>();

    let pvk_bytes: Vec<u8> = io::read(TEST_CHANNEL);
    let big_int_value = BigInt::<4>::new([
        1875955372304588914,
        12194129877466962247,
        15183177813418508560,
        2843644298302705624,
    ]);

    let image: Fp<ark_ff::MontBackend<ark_bn254::FrConfig, 4>, 4> = Fr::from(big_int_value);
    
    let deserialized_pvk: PreparedVerifyingKey<Bn254> = {
        PreparedVerifyingKey::deserialize_uncompressed(&mut &pvk_bytes[..]).unwrap()
    };
    // let deserialized_proof: ark_groth16::Proof<Bn<Config>> = {
    //     ark_groth16::Proof::deserialize_uncompressed(&mut &PVK_BYTES[..]).unwrap()
    // };
    // let xl = Fr::from(rand);
    // let xr = Fr::from(rand);
    // let image = mimc(xl, xr, &constants);
    // let deserialized_proof: ark_groth16::Proof<Bn<Config>> = {
    //     ark_groth16::Proof::deserialize_uncompressed(&mut &PROOF_BYTES[..]).unwrap()
    // };
    // let deserialized_proof: ark_groth16::Proof<Bn<Config>> = {
    //     ark_groth16::Proof::deserialize_uncompressed(&mut &PROOF_BYTES[..]).unwrap()
    // };

    // let deserialized_proof: ark_groth16::Proof<Bn<Config>> = deserialized.unwrap_or_default();

    // Groth16::<Bn254>::verify_with_processed_vk(&deserialized_pvk, &[image], &deserialized_proof).unwrap();
}
