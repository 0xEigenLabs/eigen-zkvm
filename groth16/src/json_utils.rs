#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use crate::bellman_ce::pairing::{bls12_381::Bls12, bn256::Bn256};
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use algebraic::utils::repr_to_big;
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use algebraic::PrimeField;
#[cfg(any(feature = "cuda", feature = "opencl"))]
use algebraic_gpu::circom_circuit::repr_to_big;
use anyhow::Result;
#[cfg(any(feature = "cuda", feature = "opencl"))]
use bellperson::groth16::*;
#[cfg(any(feature = "cuda", feature = "opencl"))]
use blstrs::{Bls12, Fp, Fp2, G1Affine, G2Affine, Scalar};
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use franklin_crypto::bellman::{
    bls12_381::{
        Fq2 as Fq2_bls12381, G1Affine as G1Affine_bls12381, G2Affine as G2Affine_bls12381,
    },
    bn256::{Fq2, G1Affine, G2Affine},
    groth16::{Proof, VerifyingKey},
    CurveAffine,
};
use num_bigint::BigUint;
use num_traits::Num;
#[cfg(any(feature = "cuda", feature = "opencl"))]
use pairing::MultiMillerLoop;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::fmt;
#[derive(Debug, Serialize, Deserialize)]
pub struct G1 {
    pub x: String,
    pub y: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct G2 {
    pub x: [String; 2],
    pub y: [String; 2],
}

impl fmt::Display for G1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {}", self.x, self.y)
    }
}

impl fmt::Display for G2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}], [{}, {}]", self.x[0], self.x[1], self.y[0], self.y[1])
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyingKeyFile {
    #[serde(rename = "protocol")]
    pub protocol: String,
    #[serde(rename = "curve")]
    pub curve: String,
    #[serde(rename = "vk_alpha_1")]
    pub alpha_g1: G1,
    #[serde(rename = "vk_beta_1")]
    pub beta_g1: G1,
    #[serde(rename = "vk_beta_2")]
    pub beta_g2: G2,
    #[serde(rename = "vk_gamma_2")]
    pub gamma_g2: G2,
    #[serde(rename = "vk_delta_1")]
    pub delta_g1: G1,
    #[serde(rename = "vk_delta_2")]
    pub delta_g2: G2,
    #[serde(rename = "IC")]
    pub ic: Vec<G1>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProofFile {
    #[serde(rename = "pi_a")]
    pub a: G1,
    #[serde(rename = "pi_b")]
    pub b: G2,
    #[serde(rename = "pi_c")]
    pub c: G1,
    #[serde(rename = "protocol")]
    pub protocol: String,
    #[serde(rename = "curve")]
    pub curve: String,
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub trait Parser: franklin_crypto::bellman::pairing::Engine {
    fn parse_g1(e: &Self::G1Affine, to_hex: bool) -> (String, String);
    fn parse_g2(e: &Self::G2Affine, to_hex: bool) -> (String, String, String, String);
    fn parse_g1_json(e: &Self::G1Affine, to_hex: bool) -> G1 {
        let parsed = Self::parse_g1(e, to_hex);
        G1 { x: parsed.0, y: parsed.1 }
    }
    fn parse_g2_json(e: &Self::G2Affine, to_hex: bool) -> G2 {
        let parsed = Self::parse_g2(e, to_hex);
        G2 { x: (parsed.0, parsed.1).into(), y: (parsed.2, parsed.3).into() }
    }
    fn to_g1(x: &str, y: &str) -> Self::G1Affine;
    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine;
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub trait Parser: MultiMillerLoop {
    fn parse_g1(e: &Self::G1Affine, to_hex: bool) -> (String, String);
    fn parse_g2(e: &Self::G2Affine, to_hex: bool) -> (String, String, String, String);
    fn parse_g1_json(e: &Self::G1Affine, to_hex: bool) -> G1 {
        let parsed = Self::parse_g1(e, to_hex);
        G1 { x: parsed.0, y: parsed.1 }
    }
    fn parse_g2_json(e: &Self::G2Affine, to_hex: bool) -> G2 {
        let parsed = Self::parse_g2(e, to_hex);
        G2 { x: (parsed.0, parsed.1).into(), y: (parsed.2, parsed.3).into() }
    }
    fn to_g1(x: &str, y: &str) -> Self::G1Affine;
    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine;
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn render_scalar_to_str<F: PrimeField>(el: &F, to_hex: bool) -> String {
    let repr = el.into_repr();
    if to_hex {
        repr.to_string()
    } else {
        repr_to_big(repr)
    }
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn render_str_to_scalar<F: PrimeField>(value: &str) -> F {
    let value = match value.starts_with("0x") {
        true => BigUint::from_str_radix(&value[2..], 16).unwrap().to_str_radix(10),
        _ => value.to_string(),
    };
    F::from_str(&value).unwrap()
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub fn to_public_input<T: PrimeField>(s: &str) -> Vec<T> {
    let input: Vec<String> = serde_json::from_str(s).unwrap();
    input.iter().map(|hex_str| render_str_to_scalar::<T>(hex_str)).collect()
}
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
impl Parser for Bn256 {
    fn parse_g1(e: &Self::G1Affine, to_hex: bool) -> (String, String) {
        let (x, y) = e.into_xy_unchecked();
        (render_scalar_to_str(&x, to_hex), render_scalar_to_str(&y, to_hex))
    }

    fn parse_g2(e: &Self::G2Affine, to_hex: bool) -> (String, String, String, String) {
        let (x, y) = e.into_xy_unchecked();
        (
            render_scalar_to_str(&x.c0, to_hex),
            render_scalar_to_str(&x.c1, to_hex),
            render_scalar_to_str(&y.c0, to_hex),
            render_scalar_to_str(&y.c1, to_hex),
        )
    }

    fn to_g1(x: &str, y: &str) -> Self::G1Affine {
        G1Affine::from_xy_unchecked(render_str_to_scalar(x), render_str_to_scalar(y))
    }

    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine {
        let x = Fq2 { c0: render_str_to_scalar(x0), c1: render_str_to_scalar(x1) };
        let y = Fq2 { c0: render_str_to_scalar(y0), c1: render_str_to_scalar(y1) };
        G2Affine::from_xy_unchecked(x, y)
    }
}

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
impl Parser for Bls12 {
    fn parse_g1(e: &Self::G1Affine, to_hex: bool) -> (String, String) {
        let (x, y) = e.into_xy_unchecked();
        (render_scalar_to_str(&x, to_hex), render_scalar_to_str(&y, to_hex))
    }

    fn parse_g2(e: &Self::G2Affine, to_hex: bool) -> (String, String, String, String) {
        let (x, y) = e.into_xy_unchecked();
        (
            render_scalar_to_str(&x.c0, to_hex),
            render_scalar_to_str(&x.c1, to_hex),
            render_scalar_to_str(&y.c0, to_hex),
            render_scalar_to_str(&y.c1, to_hex),
        )
    }

    fn to_g1(x: &str, y: &str) -> Self::G1Affine {
        G1Affine_bls12381::from_xy_unchecked(render_str_to_scalar(x), render_str_to_scalar(y))
    }

    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine {
        let x = Fq2_bls12381 { c0: render_str_to_scalar(x0), c1: render_str_to_scalar(x1) };
        let y = Fq2_bls12381 { c0: render_str_to_scalar(y0), c1: render_str_to_scalar(y1) };
        G2Affine_bls12381::from_xy_unchecked(x, y)
    }
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn render_fp_to_str(fp: &Fp, to_hex: bool) -> String {
    let be_bytes = fp.to_bytes_be();
    let mut hex_string = String::new();
    hex_string.push_str("0x");
    for &b in be_bytes.iter() {
        hex_string.push_str(&format!("{:02x}", b));
    }
    if to_hex {
        hex_string
    } else {
        repr_to_big(hex_string)
    }
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn render_str_to_fp(value: &str) -> Fp {
    let mut be_bytes = [0u8; 48];
    let hex_str;
    if value.starts_with("0x") {
        hex_str = value[2..].to_string();
    } else {
        let big_uint = BigUint::from_str_radix(value, 10).unwrap();
        hex_str = big_uint.to_str_radix(16);
    }
    let final_hex_str = if hex_str.len() % 2 != 0 { format!("0{}", hex_str) } else { hex_str };
    let bytes = hex::decode(final_hex_str).expect("Invalid hex string");
    let start = 48 - bytes.len();
    be_bytes[start..].copy_from_slice(&bytes);
    Fp::from_bytes_be(&be_bytes).unwrap()
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn render_str_to_scalar(value: &str) -> Scalar {
    let mut be_bytes = [0u8; 32];
    let hex_str;
    if value.starts_with("0x") {
        hex_str = value[2..].to_string();
    } else {
        let big_uint = BigUint::from_str_radix(value, 10).unwrap();
        hex_str = big_uint.to_str_radix(16);
    }
    let final_hex_str = if hex_str.len() % 2 != 0 { format!("0{}", hex_str) } else { hex_str };
    let bytes = hex::decode(final_hex_str).expect("Invalid hex string");
    let start = 32 - bytes.len();
    be_bytes[start..].copy_from_slice(&bytes);
    Scalar::from_bytes_be(&be_bytes).unwrap()
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub fn to_public_input(s: &str) -> Vec<Scalar> {
    let input: Vec<String> = serde_json::from_str(s).unwrap();
    input.iter().map(|hex_str| render_str_to_scalar(hex_str)).collect()
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
impl Parser for Bls12 {
    fn parse_g1(e: &Self::G1Affine, to_hex: bool) -> (String, String) {
        let (x, y) = (e.x(), e.y());
        (render_fp_to_str(&x, to_hex), render_fp_to_str(&y, to_hex))
    }

    fn parse_g2(e: &Self::G2Affine, to_hex: bool) -> (String, String, String, String) {
        let (x, y) = (e.x(), e.y());
        (
            render_fp_to_str(&x.c0(), to_hex),
            render_fp_to_str(&x.c1(), to_hex),
            render_fp_to_str(&y.c0(), to_hex),
            render_fp_to_str(&y.c1(), to_hex),
        )
    }

    fn to_g1(x: &str, y: &str) -> Self::G1Affine {
        G1Affine::from_raw_unchecked(render_str_to_fp(x), render_str_to_fp(y), false)
    }

    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine {
        let x = Fp2::new(render_str_to_fp(x0), render_str_to_fp(x1));
        let y = Fp2::new(render_str_to_fp(y0), render_str_to_fp(y1));
        G2Affine::from_raw_unchecked(x, y, false)
    }
}

pub fn serialize_vk<P: Parser>(
    vk: &VerifyingKey<P>,
    curve_type: &str,
    to_hex: bool,
) -> Result<String> {
    let verifying_key_file = VerifyingKeyFile {
        protocol: "groth16".to_string(),
        curve: curve_type.to_string(),
        alpha_g1: P::parse_g1_json(&vk.alpha_g1, to_hex),
        beta_g1: P::parse_g1_json(&vk.beta_g1, to_hex),
        beta_g2: P::parse_g2_json(&vk.beta_g2, to_hex),
        gamma_g2: P::parse_g2_json(&vk.gamma_g2, to_hex),
        delta_g1: P::parse_g1_json(&vk.delta_g1, to_hex),
        delta_g2: P::parse_g2_json(&vk.delta_g2, to_hex),
        ic: vk.ic.iter().map(|e| P::parse_g1_json(e, to_hex)).collect::<Vec<_>>(),
    };

    Ok(to_string(&verifying_key_file)?)
}

pub fn serialize_proof<P: Parser>(p: &Proof<P>, curve_type: &str, to_hex: bool) -> Result<String> {
    let proof_file = ProofFile {
        a: P::parse_g1_json(&p.a, to_hex),
        b: P::parse_g2_json(&p.b, to_hex),
        c: P::parse_g1_json(&p.c, to_hex),
        protocol: "groth16".to_string(),
        curve: curve_type.to_string(),
    };

    Ok(to_string(&proof_file)?)
}

pub fn to_verification_key<P: Parser>(s: &str) -> VerifyingKey<P> {
    let vk_file: VerifyingKeyFile =
        serde_json::from_str(s).expect("Error during deserialization of the JSON data");

    let convert_g1 = |point: &G1| P::to_g1(&point.x, &point.y);
    let convert_g2 = |point: &G2| P::to_g2(&point.x[0], &point.x[1], &point.y[0], &point.y[1]);

    VerifyingKey {
        alpha_g1: convert_g1(&vk_file.alpha_g1),
        beta_g1: convert_g1(&vk_file.beta_g1),
        beta_g2: convert_g2(&vk_file.beta_g2),
        gamma_g2: convert_g2(&vk_file.gamma_g2),
        delta_g1: convert_g1(&vk_file.delta_g1),
        delta_g2: convert_g2(&vk_file.delta_g2),
        ic: vk_file.ic.iter().map(convert_g1).collect(),
    }
}

pub fn to_proof<P: Parser>(s: &str) -> Proof<P> {
    let proof: ProofFile =
        serde_json::from_str(s).expect("Error during deserialization of the JSON data");

    let convert_g1 = |point: &G1| P::to_g1(&point.x, &point.y);
    let convert_g2 = |point: &G2| P::to_g2(&point.x[0], &point.x[1], &point.y[0], &point.y[1]);

    Proof { a: convert_g1(&proof.a), b: convert_g2(&proof.b), c: convert_g1(&proof.c) }
}

#[cfg(test)]
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_vk() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/verification_key.bin").unwrap(),
        );
        let vk_from_bin = VerifyingKey::<Bn256>::read(&mut reader).unwrap();
        let result = serialize_vk(&vk_from_bin, "BN128", false).unwrap();
        std::fs::write("./test-vectors/verification_key.json", result)
            .expect("Unable to write data to file");

        let json_data = std::fs::read_to_string("./test-vectors/verification_key.json")
            .expect("Unable to read the JSON file");
        let verifying_key_from_json = to_verification_key::<Bn256>(&json_data);
        assert_eq!(
            vk_from_bin.alpha_g1, verifying_key_from_json.alpha_g1,
            "VerificationKey are not equal"
        );
    }

    #[test]
    fn test_serialize_vk_bls12381() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/verification_key_bls12381.bin").unwrap(),
        );
        let vk_from_bin = VerifyingKey::<Bls12>::read(&mut reader).unwrap();
        let result = serialize_vk(&vk_from_bin, "BLS12381", false).unwrap();
        std::fs::write("./test-vectors/verification_key_bls12381.json", result)
            .expect("Unable to write data to file");
        let json_data = std::fs::read_to_string("./test-vectors/verification_key_bls12381.json")
            .expect("Unable to read the JSON file");
        let verifying_key_from_json = to_verification_key::<Bls12>(&json_data);
        assert_eq!(
            vk_from_bin.alpha_g1, verifying_key_from_json.alpha_g1,
            "VerificationKey are not equal"
        );
    }

    #[test]
    fn test_serialize_proof() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/proof.bin").unwrap(),
        );
        let proof_from_bin = Proof::<Bn256>::read(&mut reader).unwrap();
        let result = serialize_proof(&proof_from_bin, "BN128", false).unwrap();
        std::fs::write("./test-vectors/proof.json", result).expect("Unable to write data to file");

        let json_data = std::fs::read_to_string("./test-vectors/proof.json")
            .expect("Unable to read the JSON file");
        let proof_from_json = to_proof::<Bn256>(&json_data);
        assert_eq!(proof_from_bin.a, proof_from_json.a, "Proofs are not equal");
    }
}

#[cfg(test)]
#[cfg(any(feature = "cuda", feature = "opencl"))]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_vk_bls12381() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/verification_key_bls12381.bin").unwrap(),
        );
        let vk_from_bin = VerifyingKey::<Bls12>::read(&mut reader).unwrap();
        let result = serialize_vk(&vk_from_bin, "BLS12381", false).unwrap();
        std::fs::write("./test-vectors/verification_key_bls12381.json", result)
            .expect("Unable to write data to file");
        let json_data = std::fs::read_to_string("./test-vectors/verification_key_bls12381.json")
            .expect("Unable to read the JSON file");
        let verifying_key_from_json = to_verification_key::<Bls12>(&json_data);
        assert_eq!(
            vk_from_bin.alpha_g1, verifying_key_from_json.alpha_g1,
            "VerificationKey are not equal"
        );
    }
}
