#[cfg(test)]
mod tests {
    use std::env;
    const INPUT_FILE: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/data/public_inputs_bls12381.json");
    const VK_FILE_BLS12: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/data/groth16_vk_bls12381.json");
    const PROOF_FILE_BLS12: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/data/proof_bls12381.json");

    #[test]
    fn test_ark_groth16_bls12381() {
        use std::fs;

        use crate::json_util::{Groth16Proof, JsonPublicInput, JsonVerificationKey};
        use ark_bls12_381::Bls12_381;
        use ark_groth16::Groth16;
        use ark_groth16::Proof;
        use ark_groth16::VerifyingKey;

        let public_input =
            fs::read_to_string(INPUT_FILE).expect("Failed to read public inputs JSON file");
        let public_input =
            serde_json::from_str::<JsonPublicInput<ark_bls12_381::Fr>>(&public_input)
                .expect("Failed to parse JSON public input");
        println!("Public Input: {:?}", public_input);
        let vk_string =
            fs::read_to_string(VK_FILE_BLS12).expect("Failed to read public inputs JSON file");
        let vk = serde_json::from_str::<JsonVerificationKey<Bls12_381>>(&vk_string).unwrap();
        println!("vk: {:?}", vk);
        let proof_string =
            fs::read_to_string(PROOF_FILE_BLS12).expect("Failed to read public inputs JSON file");
        let proof = serde_json::from_str::<Groth16Proof<Bls12_381>>(&proof_string).unwrap();
        println!("proof: {:?}", proof);

        let vk = VerifyingKey::<Bls12_381> {
            alpha_g1: vk.alpha_1,
            beta_g2: vk.beta_2,
            gamma_g2: vk.gamma_2,
            delta_g2: vk.delta_2,
            gamma_abc_g1: vk.ic[..vk.ic.len() - 1].to_vec(),
        };
        let proof = Proof { a: proof.pi_a, b: proof.pi_b, c: proof.pi_c };
        let public_inputs: &[ark_bls12_381::Fr] = &public_input.values;
        println!("Expected public inputs: {}", vk.gamma_abc_g1.len());
        println!("Provided public inputs: {}", public_inputs.len());
        let vk = ark_groth16::prepare_verifying_key(&vk);

        let res = Groth16::<Bls12_381>::verify_proof(&vk, &proof, public_inputs);
        assert!(res.is_ok(), "Groth16 proof verification failed: {:?}", res.err());
    }
}
