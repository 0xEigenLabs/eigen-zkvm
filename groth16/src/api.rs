#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use crate::bellman_ce::{
    groth16::{Parameters, Proof, VerifyingKey},
    pairing::{
        bls12_381::{Bls12, Fr as Fr_bls12381},
        bn256::{Bn256, Fr},
    },
};
use crate::{groth16::Groth16, json_utils::*, template::CONTRACT_TEMPLATE};
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use algebraic::{
    bellman_ce::Engine,
    circom_circuit::CircomCircuit,
    reader::load_r1cs,
    witness::{load_input_for_witness, WitnessCalculator},
    Field, PrimeField,
};
#[cfg(any(feature = "cuda", feature = "opencl"))]
use algebraic_gpu::{
    circom_circuit::CircomCircuit,
    reader::load_r1cs,
    witness::{load_input_for_witness, WitnessCalculator},
    Field, PrimeField,
};
use anyhow::{anyhow, bail, Result};
#[cfg(any(feature = "cuda", feature = "opencl"))]
use bellperson::{gpu, groth16::*};
#[cfg(any(feature = "cuda", feature = "opencl"))]
use blstrs::{Bls12, Scalar};
#[cfg(any(feature = "cuda", feature = "opencl"))]
use group::WnafGroup;
use num_traits::Zero;
#[cfg(any(feature = "cuda", feature = "opencl"))]
use pairing::{Engine, MultiMillerLoop};
#[cfg(any(feature = "cuda", feature = "opencl"))]
use rand_new as rand;
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use rand_old as rand;
use regex::Regex;

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn groth16_setup(
    curve_type: &str,
    circuit_file: &str,
    pk_file: &str,
    vk_file: &str,
    to_hex: bool,
) -> Result<()> {
    let mut rng = rand::thread_rng();
    match curve_type {
        "BN128" => {
            let circuit = create_circuit_from_file::<Bn256>(circuit_file, None);
            let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut rng)?;
            write_pk_vk_to_files(curve_type, pk, vk, pk_file, vk_file, to_hex)?
        }
        "BLS12381" => {
            let circuit = create_circuit_from_file::<Bls12>(circuit_file, None);
            let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut rng)?;
            write_pk_vk_to_files(curve_type, pk, vk, pk_file, vk_file, to_hex)?
        }
        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    };
    Ok(())
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
#[allow(clippy::large_enum_variant)]
pub enum SetupResult {
    BN128(CircomCircuit<Bn256>, Parameters<Bn256>, VerifyingKey<Bn256>),
    BLS12381(CircomCircuit<Bls12>, Parameters<Bls12>, VerifyingKey<Bls12>),
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn groth16_setup_inplace(curve_type: &str, circuit_file: &str) -> Result<SetupResult> {
    let mut rng = rand::thread_rng();
    let result = match curve_type {
        "BN128" => {
            let circuit = create_circuit_from_file::<Bn256>(circuit_file, None);
            let (pk, vk) = Groth16::circuit_specific_setup(circuit.clone(), &mut rng)?;
            SetupResult::BN128(circuit, pk, vk)
        }
        "BLS12381" => {
            let circuit = create_circuit_from_file::<Bls12>(circuit_file, None);
            let (pk, vk) = Groth16::circuit_specific_setup(circuit.clone(), &mut rng)?;
            SetupResult::BLS12381(circuit, pk, vk)
        }
        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    };
    Ok(result)
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn groth16_setup(
    curve_type: &str,
    circuit_file: &str,
    pk_file: &str,
    vk_file: &str,
    to_hex: bool,
) -> Result<()> {
    let mut rng = rand::thread_rng();
    match curve_type {
        "BLS12381" => {
            let circuit = create_circuit_from_file::<Scalar>(circuit_file, None);
            let (pk, vk): (Parameters<Bls12>, VerifyingKey<Bls12>) =
                Groth16::circuit_specific_setup(circuit, &mut rng)?;
            write_pk_vk_to_files(curve_type, pk, vk, pk_file, vk_file, to_hex)?
        }
        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    };
    Ok(())
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
#[allow(clippy::large_enum_variant)]
pub enum SetupResult {
    BLS12381(CircomCircuit<Scalar>, Parameters<Bls12>, VerifyingKey<Bls12>),
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn groth16_setup_inplace(curve_type: &str, circuit_file: &str) -> Result<SetupResult> {
    let mut rng = rand::thread_rng();
    let result = match curve_type {
        "BLS12381" => {
            let circuit = create_circuit_from_file::<Scalar>(circuit_file, None);
            let (pk, vk): (Parameters<Bls12>, VerifyingKey<Bls12>) =
                Groth16::circuit_specific_setup(circuit.clone(), &mut rng)?;
            SetupResult::BLS12381(circuit, pk, vk)
        }
        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    };
    Ok(result)
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
#[allow(clippy::too_many_arguments)]
pub fn groth16_prove(
    curve_type: &str,
    circuit_file: &str,
    wtns_file: &str,
    pk_file: &str,
    input_file: &str,
    public_input_file: &str,
    proof_file: &str,
    to_hex: bool,
) -> Result<()> {
    let mut rng = rand::thread_rng();

    let mut wtns = WitnessCalculator::from_file(wtns_file)?;
    let inputs = load_input_for_witness(input_file);
    let w = wtns.calculate_witness(inputs, false)?;
    match curve_type {
        "BN128" => {
            let pk: Parameters<Bn256> = read_pk_from_file(pk_file, false)?;
            let w =
                w.iter()
                    .map(|wi| {
                        if wi.is_zero() {
                            Fr::zero()
                        } else {
                            Fr::from_str(&wi.to_string()).unwrap()
                        }
                    })
                    .collect::<Vec<_>>();
            let circuit = create_circuit_from_file::<Bn256>(circuit_file, Some(w));
            let proof = Groth16::prove(&pk, circuit.clone(), &mut rng)?;
            let proof_json = serialize_proof(&proof, curve_type, to_hex)?;
            std::fs::write(proof_file, proof_json)?;
            let input_json = circuit.get_public_inputs_json();
            std::fs::write(public_input_file, input_json)?;
        }
        "BLS12381" => {
            let pk: Parameters<Bls12> = read_pk_from_file(pk_file, false)?;
            let w = w
                .iter()
                .map(|wi| {
                    if wi.is_zero() {
                        Fr_bls12381::zero()
                    } else {
                        Fr_bls12381::from_str(&wi.to_string()).unwrap()
                    }
                })
                .collect::<Vec<_>>();
            let circuit = create_circuit_from_file::<Bls12>(circuit_file, Some(w));
            let proof = Groth16::prove(&pk, circuit.clone(), &mut rng)?;
            let proof_json = serialize_proof(&proof, curve_type, to_hex)?;
            std::fs::write(proof_file, proof_json)?;
            let input_json = circuit.get_public_inputs_json();
            std::fs::write(public_input_file, input_json)?;
        }
        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    };
    Ok(())
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
#[allow(clippy::too_many_arguments)]
pub fn groth16_prove_inplace<E: Engine + crate::json_utils::Parser>(
    curve_type: &str,
    circuit: CircomCircuit<E>,
    wtns_file: &str,
    pk: Parameters<E>,
    input_file: &str,
    public_input_file: &str,
    proof_file: &str,
    to_hex: bool,
) -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut wtns = WitnessCalculator::from_file(wtns_file)?;
    let inputs = load_input_for_witness(input_file);
    let w = wtns.calculate_witness(inputs, false)?;
    let circuit1 = create_circuit_add_witness(circuit, w);
    let proof = Groth16::prove(&pk, circuit1.clone(), &mut rng)?;
    let proof_json = serialize_proof(&proof, curve_type, to_hex)?;
    std::fs::write(proof_file, proof_json)?;
    let input_json = circuit1.get_public_inputs_json();
    std::fs::write(public_input_file, input_json)?;
    Ok(())
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
#[allow(clippy::too_many_arguments)]
pub fn groth16_prove(
    curve_type: &str,
    circuit_file: &str,
    wtns_file: &str,
    pk_file: &str,
    input_file: &str,
    public_input_file: &str,
    proof_file: &str,
    to_hex: bool,
) -> Result<()> {
    let mut rng = rand::thread_rng();

    let mut wtns = WitnessCalculator::from_file(wtns_file)?;
    let inputs = load_input_for_witness(input_file);
    let w = wtns.calculate_witness(inputs, false)?;
    match curve_type {
        "BLS12381" => {
            let pk: Parameters<Bls12> = read_pk_from_file(pk_file, false)?;
            let w = w
                .iter()
                .map(|wi| {
                    if wi.is_zero() {
                        Scalar::ZERO
                    } else {
                        Scalar::from_str_vartime(&wi.to_string()).unwrap()
                    }
                })
                .collect::<Vec<_>>();
            let circuit: CircomCircuit<Scalar> =
                create_circuit_from_file::<Scalar>(circuit_file, Some(w));
            let proof = Groth16::prove(&pk, circuit.clone(), &mut rng)?;
            let proof_json = serialize_proof(&proof, curve_type, to_hex)?;
            std::fs::write(proof_file, proof_json)?;
            let input_json = circuit.get_public_inputs_json();
            std::fs::write(public_input_file, input_json)?;
        }
        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    };

    Ok(())
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
#[allow(clippy::too_many_arguments)]
pub fn groth16_prove_inplace(
    curve_type: &str,
    circuit: CircomCircuit<Scalar>,
    wtns_file: &str,
    pk: Parameters<Bls12>,
    input_file: &str,
    public_input_file: &str,
    proof_file: &str,
    to_hex: bool,
) -> Result<()> {
    let mut rng = rand::thread_rng();
    let mut wtns = WitnessCalculator::from_file(wtns_file)?;
    let inputs = load_input_for_witness(input_file);
    let w = wtns.calculate_witness(inputs, false)?;
    let circuit1 = create_circuit_add_witness(circuit, w);
    let proof = Groth16::prove(&pk, circuit1.clone(), &mut rng)?;
    let proof_json = serialize_proof(&proof, curve_type, to_hex)?;
    std::fs::write(proof_file, proof_json)?;
    let input_json = circuit1.get_public_inputs_json();
    std::fs::write(public_input_file, input_json)?;
    Ok(())
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn groth16_verify(
    curve_type: &str,
    vk_file: &str,
    public_input_file: &str,
    proof_file: &str,
) -> Result<()> {
    match curve_type {
        "BN128" => {
            let vk = read_vk_from_file(vk_file)?;
            let inputs = read_public_input_from_file::<Fr>(public_input_file)?;
            let proof = read_proof_from_file(proof_file)?;

            let verification_result =
                Groth16::<_, CircomCircuit<Bn256>>::verify_with_processed_vk(&vk, &inputs, &proof);

            if verification_result.is_err() || !verification_result.unwrap() {
                bail!("verify failed");
            }
        }

        "BLS12381" => {
            let vk = read_vk_from_file(vk_file)?;
            let inputs = read_public_input_from_file::<Fr_bls12381>(public_input_file)?;
            let proof = read_proof_from_file(proof_file)?;

            let verification_result =
                Groth16::<_, CircomCircuit<Bls12>>::verify_with_processed_vk(&vk, &inputs, &proof);

            if verification_result.is_err() || !verification_result.unwrap() {
                bail!("verify failed");
            }
        }

        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    }

    Ok(())
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn groth16_verify_inplace<E: Engine + crate::json_utils::Parser>(
    vk: VerifyingKey<E>,
    public_input_file: &str,
    proof_file: &str,
) -> Result<()> {
    let inputs: Vec<E::Fr> = read_public_input_from_file::<E::Fr>(public_input_file)?;
    let proof = read_proof_from_file(proof_file)?;
    let verification_result =
        Groth16::<_, CircomCircuit<E>>::verify_with_processed_vk(&vk, &inputs[..], &proof);

    if verification_result.is_err() || !verification_result.unwrap() {
        bail!("verify failed");
    }

    Ok(())
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn groth16_verify(
    curve_type: &str,
    vk_file: &str,
    public_input_file: &str,
    proof_file: &str,
) -> Result<()> {
    match curve_type {
        "BLS12381" => {
            let vk: VerifyingKey<Bls12> = read_vk_from_file(vk_file)?;
            let inputs: Vec<Scalar> = read_public_input_from_file(public_input_file)?;
            let proof = read_proof_from_file(proof_file)?;
            let verification_result =
                Groth16::<_, CircomCircuit<Scalar>>::verify_with_processed_vk(&vk, &inputs, &proof);

            if verification_result.is_err() || !verification_result.unwrap() {
                bail!("verify failed");
            }
        }

        _ => {
            bail!(format!("Unknown curve type: {}", curve_type))
        }
    }

    Ok(())
}

// Acknowledgement: The Solidity verifier template was modified from ZoKrates implementation.
pub fn generate_verifier(vk_file_path: &str, sol_file_path: &str) -> Result<()> {
    let json_data = std::fs::read_to_string(vk_file_path)?;
    let vk_file: VerifyingKeyFile =
        serde_json::from_str(&json_data).expect("Error during deserialization of the JSON data");

    let vk_alpha = vk_file.alpha_g1.to_string();
    let vk_beta = vk_file.beta_g2.to_string();
    let vk_gamma = vk_file.gamma_g2.to_string();
    let vk_delta = vk_file.delta_g2.to_string();
    let vk_gamma_abc = vk_file.ic;

    let (mut template_text, solidity_pairing_lib_sans_bn256g2) =
        (String::from(CONTRACT_TEMPLATE), solidity_pairing_lib(false));

    let vk_regex = Regex::new(r#"(<%vk_[^i%]*%>)"#).unwrap();
    let vk_gamma_abc_len_regex = Regex::new(r#"(<%vk_gamma_abc_length%>)"#).unwrap();
    let vk_gamma_abc_repeat_regex = Regex::new(r#"(<%vk_gamma_abc_pts%>)"#).unwrap();
    let vk_input_len_regex = Regex::new(r#"(<%vk_input_length%>)"#).unwrap();
    let input_loop = Regex::new(r#"(<%input_loop%>)"#).unwrap();
    let input_argument = Regex::new(r#"(<%input_argument%>)"#).unwrap();

    template_text = vk_regex.replace(template_text.as_str(), vk_alpha.as_str()).into_owned();

    template_text =
        vk_regex.replace(template_text.as_str(), vk_beta.to_string().as_str()).into_owned();

    template_text =
        vk_regex.replace(template_text.as_str(), vk_gamma.to_string().as_str()).into_owned();

    template_text =
        vk_regex.replace(template_text.as_str(), vk_delta.to_string().as_str()).into_owned();

    let gamma_abc_count: usize = vk_gamma_abc.len();
    template_text = vk_gamma_abc_len_regex
        .replace(template_text.as_str(), format!("{}", gamma_abc_count).as_str())
        .into_owned();

    template_text = vk_input_len_regex
        .replace(template_text.as_str(), format!("{}", gamma_abc_count - 1).as_str())
        .into_owned();

    // feed input values only if there are any
    template_text = if gamma_abc_count > 1 {
        input_loop.replace(
            template_text.as_str(),
            r#"
        for(uint i = 0; i < input.length; i++){
            inputValues[i] = input[i];
        }"#,
        )
    } else {
        input_loop.replace(template_text.as_str(), "")
    }
    .to_string();

    // take input values as argument only if there are any
    template_text = if gamma_abc_count > 1 {
        input_argument.replace(
            template_text.as_str(),
            format!(", uint[{}] memory input", gamma_abc_count - 1).as_str(),
        )
    } else {
        input_argument.replace(template_text.as_str(), "")
    }
    .to_string();

    let mut gamma_abc_repeat_text = String::new();
    for (i, g1) in vk_gamma_abc.iter().enumerate() {
        gamma_abc_repeat_text.push_str(
            format!("vk.gamma_abc[{}] = Pairing.G1Point({});", i, g1.to_string().as_str()).as_str(),
        );
        if i < gamma_abc_count - 1 {
            gamma_abc_repeat_text.push_str("\n        ");
        }
    }

    template_text = vk_gamma_abc_repeat_regex
        .replace(template_text.as_str(), gamma_abc_repeat_text.as_str())
        .into_owned();

    let re = Regex::new(r"(?P<v>0[xX][0-9a-fA-F]{64})").unwrap();
    template_text = re.replace_all(&template_text, "uint256($v)").to_string();

    match std::fs::write(
        sol_file_path,
        format!("{}{}", solidity_pairing_lib_sans_bn256g2, template_text),
    ) {
        Ok(()) => println!("Generate solidity verifier successfully!"),
        Err(e) => {
            bail!("write sol file failed, {:?}", e)
        }
    }
    Ok(())
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
fn create_circuit_from_file<E: Engine>(
    circuit_file: &str,
    witness: Option<Vec<E::Fr>>,
) -> CircomCircuit<E> {
    CircomCircuit { r1cs: load_r1cs(circuit_file), witness, wire_mapping: None, aux_offset: 0 }
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn create_circuit_add_witness<E: Engine>(
    mut circuit: CircomCircuit<E>,
    witness: Vec<num_bigint::BigInt>,
) -> CircomCircuit<E> {
    let witness: Vec<E::Fr> =
        witness
            .iter()
            .map(|wi| {
                if wi.is_zero() {
                    E::Fr::zero()
                } else {
                    E::Fr::from_str(&wi.to_string()).unwrap()
                }
            })
            .collect::<Vec<_>>();
    circuit.witness = Some(witness);
    circuit.wire_mapping = None;
    circuit.aux_offset = 0;
    circuit
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
fn create_circuit_from_file<E: PrimeField>(
    circuit_file: &str,
    witness: Option<Vec<E>>,
) -> CircomCircuit<E> {
    CircomCircuit { r1cs: load_r1cs(circuit_file), witness, wire_mapping: None, aux_offset: 0 }
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn create_circuit_add_witness(
    mut circuit: CircomCircuit<Scalar>,
    witness: Vec<num_bigint::BigInt>,
) -> CircomCircuit<Scalar> {
    let w = witness
        .iter()
        .map(|wi| {
            if wi.is_zero() {
                Scalar::ZERO
            } else {
                Scalar::from_str_vartime(&wi.to_string()).unwrap()
            }
        })
        .collect::<Vec<_>>();
    circuit.witness = Some(w);
    circuit.wire_mapping = None;
    circuit.aux_offset = 0;
    circuit
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn read_pk_from_file<E: Engine>(file_path: &str, checked: bool) -> Result<Parameters<E>> {
    let file =
        std::fs::File::open(file_path).map_err(|e| anyhow!("Open {}, {:?}", file_path, e))?;
    let mut reader = std::io::BufReader::new(file);
    Ok(Parameters::<E>::read(&mut reader, checked)?)
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
fn read_pk_from_file<E: Engine>(file_path: &str, checked: bool) -> Result<Parameters<E>>
where
    E: MultiMillerLoop,
    E::G1: WnafGroup,
    E::G2: WnafGroup,
    E::Fr: gpu::GpuName,
{
    let file =
        std::fs::File::open(file_path).map_err(|e| anyhow!("Open {}, {:?}", file_path, e))?;
    let mut reader = std::io::BufReader::new(file);
    Ok(Parameters::<E>::read(&mut reader, checked)?)
}

pub fn read_vk_from_file<P: Parser>(file_path: &str) -> Result<VerifyingKey<P>> {
    let json_data = std::fs::read_to_string(file_path)?;
    Ok(to_verification_key::<P>(&json_data))
}
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
fn read_public_input_from_file<T: PrimeField>(file_path: &str) -> Result<Vec<T>> {
    let json_data = std::fs::read_to_string(file_path)?;
    Ok(to_public_input::<T>(&json_data))
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
fn read_public_input_from_file(file_path: &str) -> Result<Vec<Scalar>> {
    let json_data = std::fs::read_to_string(file_path)?;
    Ok(to_public_input(&json_data))
}

fn read_proof_from_file<P: Parser>(file_path: &str) -> Result<Proof<P>> {
    let json_data = std::fs::read_to_string(file_path)?;
    Ok(to_proof::<P>(&json_data))
}

fn write_pk_vk_to_files<P: Parser>(
    curve_type: &str,
    pk: Parameters<P>,
    vk: VerifyingKey<P>,
    pk_file: &str,
    vk_file: &str,
    to_hex: bool,
) -> Result<()> {
    let writer = std::fs::File::create(pk_file)?;
    pk.write(writer)?;
    let vk_json = serialize_vk(&vk, curve_type, to_hex)?;
    std::fs::write(vk_file, vk_json)?;
    Ok(())
}

fn solidity_pairing_lib(with_g2_addition: bool) -> String {
    let pairing_lib_beginning = r#"// This file is MIT Licensed.
//
// Copyright 2017 Christian Reitwiessner
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
pragma solidity ^0.8.16;
library Pairing {
    struct G1Point {
        uint X;
        uint Y;
    }
    // Encoding of field elements is: X[0] * z + X[1]
    struct G2Point {
        uint[2] X;
        uint[2] Y;
    }
    /// @return the generator of G1
    function P1() pure internal returns (G1Point memory) {
        return G1Point(1, 2);
    }
    /// @return the generator of G2
    function P2() pure internal returns (G2Point memory) {
        return G2Point(
            [10857046999023057135944570762232829481370756359578518086990519993285655852781,
             11559732032986387107991004021392285783925812861821192530917403151452391805634],
            [8495653923123431417604973247489272438418190587263600148770280649306958101930,
             4082367875863433681332203403145435568316851327593401208105741076214120093531]
        );
    }
    /// @return the negation of p, i.e. p.addition(p.negate()) should be zero.
    function negate(G1Point memory p) pure internal returns (G1Point memory) {
        // The prime q in the base field F_q for G1
        uint q = 21888242871839275222246405745257275088696311157297823662689037894645226208583;
        if (p.X == 0 && p.Y == 0)
            return G1Point(0, 0);
        return G1Point(p.X, q - (p.Y % q));
    }
    /// @return r the sum of two points of G1
    function addition(G1Point memory p1, G1Point memory p2) internal view returns (G1Point memory r) {
        uint[4] memory input;
        input[0] = p1.X;
        input[1] = p1.Y;
        input[2] = p2.X;
        input[3] = p2.Y;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 6, input, 0xc0, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success);
    }
"#;

    let pairing_lib_g2_addition = r#"
    /// @return r the sum of two points of G2
    function addition(G2Point memory p1, G2Point memory p2) internal view returns (G2Point memory r) {
        (r.X[0], r.X[1], r.Y[0], r.Y[1]) = BN256G2.ECTwistAdd(p1.X[0],p1.X[1],p1.Y[0],p1.Y[1],p2.X[0],p2.X[1],p2.Y[0],p2.Y[1]);
    }
"#;

    let pairing_lib_ending = r#"
    /// @return r the product of a point on G1 and a scalar, i.e.
    /// p == p.scalar_mul(1) and p.addition(p) == p.scalar_mul(2) for all points p.
    function scalar_mul(G1Point memory p, uint s) internal view returns (G1Point memory r) {
        uint[3] memory input;
        input[0] = p.X;
        input[1] = p.Y;
        input[2] = s;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 7, input, 0x80, r, 0x60)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require (success);
    }
    /// @return the result of computing the pairing check
    /// e(p1[0], p2[0]) *  .... * e(p1[n], p2[n]) == 1
    /// For example pairing([P1(), P1().negate()], [P2(), P2()]) should
    /// return true.
    function pairing(G1Point[] memory p1, G2Point[] memory p2) internal view returns (bool) {
        require(p1.length == p2.length);
        uint elements = p1.length;
        uint inputSize = elements * 6;
        uint[] memory input = new uint[](inputSize);
        for (uint i = 0; i < elements; i++)
        {
            input[i * 6 + 0] = p1[i].X;
            input[i * 6 + 1] = p1[i].Y;
            input[i * 6 + 2] = p2[i].X[1];
            input[i * 6 + 3] = p2[i].X[0];
            input[i * 6 + 4] = p2[i].Y[1];
            input[i * 6 + 5] = p2[i].Y[0];
        }
        uint[1] memory out;
        bool success;
        assembly {
            success := staticcall(sub(gas(), 2000), 8, add(input, 0x20), mul(inputSize, 0x20), out, 0x20)
            // Use "invalid" to make gas estimation work
            switch success case 0 { invalid() }
        }
        require(success);
        return out[0] != 0;
    }
    /// Convenience method for a pairing check for two pairs.
    function pairingProd2(G1Point memory a1, G2Point memory a2, G1Point memory b1, G2Point memory b2) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](2);
        G2Point[] memory p2 = new G2Point[](2);
        p1[0] = a1;
        p1[1] = b1;
        p2[0] = a2;
        p2[1] = b2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for three pairs.
    function pairingProd3(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](3);
        G2Point[] memory p2 = new G2Point[](3);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        return pairing(p1, p2);
    }
    /// Convenience method for a pairing check for four pairs.
    function pairingProd4(
            G1Point memory a1, G2Point memory a2,
            G1Point memory b1, G2Point memory b2,
            G1Point memory c1, G2Point memory c2,
            G1Point memory d1, G2Point memory d2
    ) internal view returns (bool) {
        G1Point[] memory p1 = new G1Point[](4);
        G2Point[] memory p2 = new G2Point[](4);
        p1[0] = a1;
        p1[1] = b1;
        p1[2] = c1;
        p1[3] = d1;
        p2[0] = a2;
        p2[1] = b2;
        p2[2] = c2;
        p2[3] = d2;
        return pairing(p1, p2);
    }
}
"#;

    if !with_g2_addition {
        [pairing_lib_beginning, pairing_lib_ending].join("\n")
    } else {
        [pairing_lib_beginning, pairing_lib_g2_addition, pairing_lib_ending].join("\n")
    }
}
