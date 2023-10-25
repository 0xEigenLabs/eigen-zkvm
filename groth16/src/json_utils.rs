use crate::bellman_ce::pairing::{
    bls12_381::{Bls12, Fr as Fr_bls12381},
    bn256::{Bn256, Fr},
};
use algebraic::{PrimeField, PrimeFieldRepr};
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
use serde::Deserialize;

pub trait FieldElement: Sized + PrimeField {}

impl FieldElement for Fr {}
impl FieldElement for Fr_bls12381 {}

pub trait Parser: franklin_crypto::bellman::pairing::Engine {
    fn parse_g1(e: &Self::G1Affine) -> (String, String);
    fn parse_g2(e: &Self::G2Affine) -> (String, String, String, String);
    fn parse_g1_json(e: &Self::G1Affine) -> String {
        let parsed = Self::parse_g1(e);
        format!("[\"{}\", \"{}\"]", parsed.0, parsed.1)
    }
    fn parse_g2_json(e: &Self::G2Affine) -> String {
        let parsed = Self::parse_g2(e);
        format!(
            "[[\"{}\", \"{}\"], [\"{}\", \"{}\"]]",
            parsed.0, parsed.1, parsed.2, parsed.3,
        )
    }
    fn to_g1(x: &str, y: &str) -> Self::G1Affine;
    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine;
}

pub fn render_scalar_to_hex<F: PrimeField>(el: &F) -> String {
    let mut buff = vec![];
    let repr = el.into_repr();
    repr.write_be(&mut buff).unwrap();

    format!("0x{}", hex::encode(buff))
}

pub fn render_hex_to_scalar<F: PrimeField>(value: &str) -> F {
    let value = BigUint::from_str_radix(&value[2..], 16)
        .unwrap()
        .to_str_radix(10);
    F::from_str(&value).unwrap()
}

impl Parser for Bn256 {
    fn parse_g1(e: &Self::G1Affine) -> (String, String) {
        let (x, y) = e.into_xy_unchecked();
        (render_scalar_to_hex(&x), render_scalar_to_hex(&y))
    }

    fn parse_g2(e: &Self::G2Affine) -> (String, String, String, String) {
        let (x, y) = e.into_xy_unchecked();
        (
            render_scalar_to_hex(&x.c0),
            render_scalar_to_hex(&x.c1),
            render_scalar_to_hex(&y.c0),
            render_scalar_to_hex(&y.c1),
        )
    }

    fn to_g1(x: &str, y: &str) -> Self::G1Affine {
        G1Affine::from_xy_unchecked(render_hex_to_scalar(x), render_hex_to_scalar(y))
    }

    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine {
        let x = Fq2 {
            c0: render_hex_to_scalar(x0),
            c1: render_hex_to_scalar(x1),
        };
        let y = Fq2 {
            c0: render_hex_to_scalar(y0),
            c1: render_hex_to_scalar(y1),
        };
        G2Affine::from_xy_unchecked(x, y)
    }
}

impl Parser for Bls12 {
    fn parse_g1(e: &Self::G1Affine) -> (String, String) {
        let (x, y) = e.into_xy_unchecked();
        (render_scalar_to_hex(&x), render_scalar_to_hex(&y))
    }

    fn parse_g2(e: &Self::G2Affine) -> (String, String, String, String) {
        let (x, y) = e.into_xy_unchecked();
        (
            render_scalar_to_hex(&x.c0),
            render_scalar_to_hex(&x.c1),
            render_scalar_to_hex(&y.c0),
            render_scalar_to_hex(&y.c1),
        )
    }

    fn to_g1(x: &str, y: &str) -> Self::G1Affine {
        G1Affine_bls12381::from_xy_unchecked(render_hex_to_scalar(x), render_hex_to_scalar(y))
    }

    fn to_g2(x0: &str, x1: &str, y0: &str, y1: &str) -> Self::G2Affine {
        let x = Fq2_bls12381 {
            c0: render_hex_to_scalar(x0),
            c1: render_hex_to_scalar(x1),
        };
        let y = Fq2_bls12381 {
            c0: render_hex_to_scalar(y0),
            c1: render_hex_to_scalar(y1),
        };
        G2Affine_bls12381::from_xy_unchecked(x, y)
    }
}

pub fn serialize_vk<P: Parser>(vk: VerifyingKey<P>, curve_type: &str) -> String {
    format!(
        "{{
\"protocol\": \"groth16\",
\"curve\": \"{}\",
\"vk_alpha_1\": {},
\"vk_beta_1\": {},
\"vk_beta_2\": {},
\"vk_gamma_2\": {},
\"vk_delta_1\": {},
\"vk_delta_2\": {},
\"IC\": [{}]
}}",
        curve_type,
        P::parse_g1_json(&vk.alpha_g1),
        P::parse_g1_json(&vk.beta_g1),
        P::parse_g2_json(&vk.beta_g2),
        P::parse_g2_json(&vk.gamma_g2),
        P::parse_g1_json(&vk.delta_g1),
        P::parse_g2_json(&vk.delta_g2),
        &vk.ic
            .iter()
            .map(P::parse_g1_json)
            .collect::<Vec<_>>()
            .join(", "),
    )
}

pub fn serialize_proof<P: Parser>(p: &Proof<P>, curve_type: &str) -> String {
    format!(
        "{{
\"pi_a\": {},
\"pi_b\": {},
\"pi_c\": {},
\"protocol\": \"groth16\",
\"curve\": \"{}\"
}}",
        P::parse_g1_json(&p.a),
        P::parse_g2_json(&p.b),
        P::parse_g1_json(&p.c),
        curve_type
    )
}

pub fn serialize_input<T: FieldElement>(inputs: &[T]) -> String {
    format!(
        "[\"{}\"]",
        inputs
            .iter()
            .map(render_scalar_to_hex)
            .collect::<Vec<_>>()
            .join("\", \""),
    )
}

#[derive(Debug, Deserialize)]
pub struct G1 {
    pub x: String,
    pub y: String,
}

#[derive(Debug, Deserialize)]
pub struct G2 {
    pub x: [String; 2],
    pub y: [String; 2],
}

#[derive(Debug, Deserialize)]
pub struct VerifyingKeyFile {
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

#[derive(Debug, Deserialize)]
pub struct ProofFile {
    #[serde(rename = "pi_a")]
    pub a: G1,
    #[serde(rename = "pi_b")]
    pub b: G2,
    #[serde(rename = "pi_c")]
    pub c: G1,
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

    Proof {
        a: convert_g1(&proof.a),
        b: convert_g2(&proof.b),
        c: convert_g1(&proof.c),
    }
}

pub fn to_public_input<T: FieldElement>(s: &str) -> Vec<T> {
    let input: Vec<String> =
        serde_json::from_str(s).expect("Error during deserialization of the JSON data");
    input
        .iter()
        .map(|hex_str| render_hex_to_scalar::<T>(hex_str))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bellman_ce::groth16::{Proof, VerifyingKey};
    use crate::bellman_ce::pairing::{
        bls12_381::Bls12,
        bn256::{Bn256, Fr},
    };
    use crate::bellman_ce::plonk::better_cs::keys::read_fr_vec;

    #[test]
    fn test_serialize_vk() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/verification_key.bin").unwrap(),
        );
        let vk = VerifyingKey::<Bn256>::read(&mut reader).unwrap();

        let result = serialize_vk(vk, "bn128");
        std::fs::write("./test-vectors/verification_key.json", result)
            .expect("Unable to write data to file");
    }

    #[test]
    fn test_to_verification_key() {
        let json_data = std::fs::read_to_string("./test-vectors/verification_key.json")
            .expect("Unable to read the JSON file");

        let verifying_key = to_verification_key::<Bn256>(&json_data);

        assert!(
            !verifying_key.ic.is_empty(),
            "IC vector should not be empty"
        );
    }

    #[test]
    fn test_serialize_vk_bls12381() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/verification_key_bls12381.bin").unwrap(),
        );
        let vk = VerifyingKey::<Bls12>::read(&mut reader).unwrap();

        let result = serialize_vk(vk, "bls12381");
        std::fs::write("./test-vectors/verification_key_bls12381.json", result)
            .expect("Unable to write data to file");
    }

    #[test]
    fn test_serialize_proof() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/proof.bin").unwrap(),
        );
        let proof = Proof::<Bn256>::read(&mut reader).unwrap();

        let result = serialize_proof(&proof, "bn128");
        std::fs::write("./test-vectors/proof.json", result).expect("Unable to write data to file");
    }

    #[test]
    fn test_to_proof() {
        let json_data = std::fs::read_to_string("./test-vectors/proof.json")
            .expect("Unable to read the JSON file");

        let proof = to_proof::<Bn256>(&json_data);

        println!("{:?}", proof);
    }

    #[test]
    fn test_serialize_input() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/public_input.bin").unwrap(),
        );

        let input = read_fr_vec::<Fr, _>(&mut reader).unwrap();
        let result = serialize_input::<Fr>(&input);
        std::fs::write("./test-vectors/public_input.json", result)
            .expect("Unable to write data to file");
    }

    #[test]
    fn test_to_public_input() {
        let json_data = std::fs::read_to_string("./test-vectors/public_input.json")
            .expect("Unable to read the JSON file");

        let input = to_public_input::<Fr>(&json_data);

        println!("{:?}", input);
    }
}
