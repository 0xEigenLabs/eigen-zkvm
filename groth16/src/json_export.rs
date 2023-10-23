use crate::bellman_ce::pairing::{
    bls12_381::{Bls12, Fr as Fr_bls12381},
    bn256::{Bn256, Fr},
};
use algebraic::{PrimeField, PrimeFieldRepr};
use franklin_crypto::bellman::{
    bn256,
    groth16::{Proof, VerifyingKey},
    CurveAffine, Engine,
};
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
}

pub fn render_scalar_to_hex<F: PrimeField>(el: &F) -> String {
    let mut buff = vec![];
    let repr = el.into_repr();
    repr.write_be(&mut buff).unwrap();

    format!("0x{}", hex::encode(buff))
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
}

pub fn serialize_vk<E: Engine + Parser>(vk: VerifyingKey<E>, curve_type: &str) -> String {
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
        E::parse_g1_json(&vk.alpha_g1),
        E::parse_g1_json(&vk.beta_g1),
        E::parse_g2_json(&vk.beta_g2),
        E::parse_g2_json(&vk.gamma_g2),
        E::parse_g1_json(&vk.delta_g1),
        E::parse_g2_json(&vk.delta_g2),
        &vk.ic
            .iter()
            .map(E::parse_g1_json)
            .collect::<Vec<_>>()
            .join(", "),
    )
}

pub fn serialize_proof<E: Engine + Parser>(p: &Proof<E>, curve_type: &str) -> String {
    format!(
        "{{
\"pi_a\": {},
\"pi_b\": {},
\"pi_c\": {},
\"protocol\": \"groth16\",
\"curve\": \"{}\"
}}",
        E::parse_g1_json(&p.a),
        E::parse_g2_json(&p.b),
        E::parse_g1_json(&p.c),
        curve_type
    )
}

pub trait FieldElement: Sized + PrimeField {}

impl FieldElement for Fr {}
impl FieldElement for Fr_bls12381 {}

pub fn serialize_input<T: FieldElement>(inputs: &Vec<T>) -> String {
    format!(
        "[\"{}\"]",
        inputs
            .iter()
            .map(render_scalar_to_hex)
            .collect::<Vec<_>>()
            .join("\", \""),
    )
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
}
