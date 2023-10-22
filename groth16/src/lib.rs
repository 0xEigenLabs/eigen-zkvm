pub mod export_solidity_verifier;
pub mod groth16;

pub use bellman_ce::pairing::ff;
pub use ff::*;
pub use franklin_crypto::bellman as bellman_ce;

mod parse {
    use crate::bellman_ce::pairing::{bls12_381::Bls12, bn256::Bn256};
    use algebraic::{PrimeField, PrimeFieldRepr};
    use franklin_crypto::bellman::CurveAffine;

    pub fn render_scalar_to_hex<F: PrimeField>(el: &F) -> String {
        let mut buff = vec![];
        let repr = el.into_repr();
        repr.write_be(&mut buff).unwrap();

        format!("0x{}", hex::encode(buff))
    }

    fn parse_g1(
        e: &<Bn256 as franklin_crypto::bellman::pairing::Engine>::G1Affine,
    ) -> (String, String) {
        let (x, y) = e.into_xy_unchecked();

        (render_scalar_to_hex(&x), render_scalar_to_hex(&y))
    }

    fn parse_g2(
        e: &<Bn256 as franklin_crypto::bellman::pairing::Engine>::G2Affine,
    ) -> (String, String, String, String) {
        let (x, y) = e.into_xy_unchecked();
        (
            render_scalar_to_hex(&x.c0),
            render_scalar_to_hex(&x.c1),
            render_scalar_to_hex(&y.c0),
            render_scalar_to_hex(&y.c1),
        )
    }

    pub fn parse_g1_json(
        e: &<Bn256 as franklin_crypto::bellman::pairing::Engine>::G1Affine,
    ) -> String {
        let parsed = parse_g1(e);

        format!("[\"{}\", \"{}\"]", parsed.0, parsed.1)
    }

    pub fn parse_g2_json(
        e: &<Bn256 as franklin_crypto::bellman::pairing::Engine>::G2Affine,
    ) -> String {
        let parsed = parse_g2(e);

        format!(
            "[[\"{}\", \"{}\"], [\"{}\", \"{}\"]]",
            parsed.0, parsed.1, parsed.2, parsed.3,
        )
    }

    fn parse_g1_bls12381(
        e: &<Bls12 as franklin_crypto::bellman::pairing::Engine>::G1Affine,
    ) -> (String, String) {
        let (x, y) = e.into_xy_unchecked();

        (render_scalar_to_hex(&x), render_scalar_to_hex(&y))
    }

    fn parse_g2_bls12381(
        e: &<Bls12 as franklin_crypto::bellman::pairing::Engine>::G2Affine,
    ) -> (String, String, String, String) {
        let (x, y) = e.into_xy_unchecked();
        (
            render_scalar_to_hex(&x.c0),
            render_scalar_to_hex(&x.c1),
            render_scalar_to_hex(&y.c0),
            render_scalar_to_hex(&y.c1),
        )
    }

    pub fn parse_g1_json_bls12381(
        e: &<Bls12 as franklin_crypto::bellman::pairing::Engine>::G1Affine,
    ) -> String {
        let parsed = parse_g1_bls12381(e);

        format!("[\"{}\", \"{}\"]", parsed.0, parsed.1)
    }

    pub fn parse_g2_json_bls12381(
        e: &<Bls12 as franklin_crypto::bellman::pairing::Engine>::G2Affine,
    ) -> String {
        let parsed = parse_g2_bls12381(e);

        format!(
            "[[\"{}\", \"{}\"], [\"{}\", \"{}\"]]",
            parsed.0, parsed.1, parsed.2, parsed.3,
        )
    }
}

pub mod serialize {
    use crate::bellman_ce::pairing::{
        bls12_381::{Bls12, Fr as Fr_bls12381},
        bn256::{Bn256, Fr},
    };
    use crate::bellman_ce::{
        groth16::{Proof, VerifyingKey},
        PrimeField,
    };
    use crate::parse::*;

    pub fn serialize_vk(vk: VerifyingKey<Bn256>, curve_type: &str) -> String {
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
            parse_g1_json(&vk.alpha_g1),
            parse_g1_json(&vk.beta_g1),
            parse_g2_json(&vk.beta_g2),
            parse_g2_json(&vk.gamma_g2),
            parse_g1_json(&vk.delta_g1),
            parse_g2_json(&vk.delta_g2),
            &vk.ic
                .iter()
                .map(parse_g1_json)
                .collect::<Vec<_>>()
                .join(", "),
        )
    }

    pub fn serialize_proof(p: &Proof<Bn256>, curve_type: &str) -> String {
        format!(
            "{{
\"pi_a\": {},
\"pi_b\": {},
\"pi_c\": {},
\"protocol\": \"groth16\",
\"curve\": \"{}\"
}}",
            parse_g1_json(&p.a),
            parse_g2_json(&p.b),
            parse_g1_json(&p.c),
            curve_type
        )
    }

    pub fn serialize_input(inputs: &Vec<Fr>) -> String {
        format!(
            "[\"{}\"]",
            inputs
                .iter()
                .map(render_scalar_to_hex)
                .collect::<Vec<_>>()
                .join("\", \""),
        )
    }

    pub fn serialize_vk_bls12381(vk: VerifyingKey<Bls12>, curve_type: &str) -> String {
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
            parse_g1_json_bls12381(&vk.alpha_g1),
            parse_g1_json_bls12381(&vk.beta_g1),
            parse_g2_json_bls12381(&vk.beta_g2),
            parse_g2_json_bls12381(&vk.gamma_g2),
            parse_g1_json_bls12381(&vk.delta_g1),
            parse_g2_json_bls12381(&vk.delta_g2),
            &vk.ic
                .iter()
                .map(parse_g1_json_bls12381)
                .collect::<Vec<_>>()
                .join(", "),
        )
    }

    pub fn serialize_proof_bls12381(p: &Proof<Bls12>, curve_type: &str) -> String {
        format!(
            "{{
\"pi_a\": {},
\"pi_b\": {},
\"pi_c\": {},
\"protocol\": \"groth16\",
\"curve\": \"{}\"
}}",
            parse_g1_json_bls12381(&p.a),
            parse_g2_json_bls12381(&p.b),
            parse_g1_json_bls12381(&p.c),
            curve_type
        )
    }

    pub fn serialize_input_bls1231(inputs: &Vec<Fr_bls12381>) -> String {
        format!(
            "[\"{}\"]",
            inputs
                .iter()
                .map(render_scalar_to_hex)
                .collect::<Vec<_>>()
                .join("\", \""),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::bellman_ce::groth16::{Proof, VerifyingKey};
    use crate::bellman_ce::plonk::better_cs::keys::read_fr_vec;
    use crate::serialize::*;
    use crate::{
        bellman_ce::pairing::{
            bls12_381::Bls12,
            bn256::{Bn256, Fr},
        },
        serialize::serialize_vk_bls12381,
    };

    #[test]
    fn test_serialize_vk() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/verification_key.bin").unwrap(),
        );
        let vk = VerifyingKey::<Bn256>::read(&mut reader).unwrap();

        let result = serialize_vk(vk, "bn128");
        std::fs::write("./test-vectors/verification_key.json", result).expect("Unable to write data to file");
    }

    #[test]
    fn test_serialize_vk_bls12381() {
        let mut reader = std::io::BufReader::with_capacity(
            1 << 24,
            std::fs::File::open("./test-vectors/verification_key_bls12381.bin").unwrap(),
        );
        let vk = VerifyingKey::<Bls12>::read(&mut reader).unwrap();

        let result = serialize_vk_bls12381(vk, "bls12381");
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
        let result = serialize_input(&input);
        std::fs::write("./test-vectors/public_input.json", result).expect("Unable to write data to file");
    }
}
