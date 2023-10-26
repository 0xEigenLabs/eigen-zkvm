use crate::bellman_ce::{groth16::*, Circuit};
use algebraic::errors::Result;
use franklin_crypto::bellman::pairing::Engine;
use rand::Rng;
use std::marker::PhantomData;

pub struct Groth16<E: Engine, C: Circuit<E>> {
    _engine: PhantomData<E>,
    _circuit: PhantomData<C>,
}

impl<E: Engine, C: Circuit<E>> Groth16<E, C> {
    pub fn circuit_specific_setup<R: Rng>(
        circuit: C,
        rng: &mut R,
    ) -> Result<(Parameters<E>, VerifyingKey<E>)> {
        let pk: Parameters<E> = generate_random_parameters::<E, C, R>(circuit, rng)?;
        let vk = pk.vk.clone();

        Ok((pk, vk))
    }

    pub fn prove<R: Rng>(
        circuit_pk: &Parameters<E>,
        input_and_witness: C,
        rng: &mut R,
    ) -> Result<Proof<E>> {
        let result = create_random_proof::<E, _, _, _>(input_and_witness, circuit_pk, rng)?;

        Ok(result)
    }

    pub fn verify_with_processed_vk(
        circuit_vk: &VerifyingKey<E>,
        public_input: &[E::Fr],
        proof: &Proof<E>,
    ) -> Result<bool> {
        let circuit_pvk = prepare_verifying_key(circuit_vk);
        let result = verify_proof(&circuit_pvk, proof, public_input)?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use franklin_crypto::bellman::{Field, PrimeField};
    use num_traits::Zero;

    use super::*;
    use crate::bellman_ce::bls12_381::Bls12;
    use crate::bellman_ce::bls12_381::Fr as Fr_bls12381;
    use crate::bellman_ce::bn256::{Bn256, Fr};
    use algebraic::circom_circuit::CircomCircuit;
    use algebraic::r1cs_reader;
    use algebraic::r1cs_witness::{load_input_for_witness, WitnessCalculator};
    const INPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.input.json");
    const CIRCUIT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.r1cs");
    const WASM_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.wasm");
    const CIRCUIT_FILE_BLS12: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-vectors/mycircuit_bls12381.r1cs"
    );
    const WASM_FILE_BLS12: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/test-vectors/mycircuit_bls12381.wasm"
    );

    #[test]
    fn groth16_proof() -> Result<()> {
        //1. SRS
        let t = std::time::Instant::now();
        let circuit: CircomCircuit<Bn256> = CircomCircuit {
            r1cs: r1cs_reader::load_r1cs(CIRCUIT_FILE),
            witness: None,
            wire_mapping: None,
            aux_offset: 0,
        };
        let mut rng = rand::thread_rng();
        let params = Groth16::circuit_specific_setup(circuit, &mut rng)?;
        let elapsed = t.elapsed().as_secs_f64();
        println!("1-groth16-bn128 setup run time: {} secs", elapsed);

        //2. Prove
        let t1 = std::time::Instant::now();
        let mut wtns = WitnessCalculator::new(WASM_FILE).unwrap();
        let inputs = load_input_for_witness(INPUT_FILE);
        let w = wtns.calculate_witness(inputs, false).unwrap();
        let w = w
            .iter()
            .map(|wi| {
                if wi.is_zero() {
                    Fr::zero()
                } else {
                    // println!("wi: {}", wi);
                    Fr::from_str(&wi.to_string()).unwrap()
                }
            })
            .collect::<Vec<_>>();
        let circuit1: CircomCircuit<Bn256> = CircomCircuit {
            r1cs: r1cs_reader::load_r1cs(CIRCUIT_FILE),
            witness: Some(w),
            wire_mapping: None,
            aux_offset: 0,
        };
        let inputs = circuit1.get_public_inputs().unwrap();
        let proof = Groth16::prove(&params.0, circuit1.clone(), &mut rng)?;
        let elapsed1 = t1.elapsed().as_secs_f64();
        println!("2-groth16-bn128 prove run time: {} secs", elapsed1);

        //3. Verify
        let t2 = std::time::Instant::now();
        let verified = Groth16::<_, CircomCircuit<Bn256>>::verify_with_processed_vk(
            &params.1, &inputs, &proof,
        )?;
        let elapsed2 = t2.elapsed().as_secs_f64();
        println!("3-groth16-bn128 verify run time: {} secs", elapsed2);

        assert!(verified);

        Ok(())
    }

    #[test]
    fn groth16_proof_bls12381() -> Result<()> {
        //1. SRS
        let t = std::time::Instant::now();
        let circuit: CircomCircuit<Bls12> = CircomCircuit {
            r1cs: r1cs_reader::load_r1cs(CIRCUIT_FILE_BLS12),
            witness: None,
            wire_mapping: None,
            aux_offset: 0,
        };
        let mut rng = rand::thread_rng();
        let params = Groth16::circuit_specific_setup(circuit, &mut rng)?;
        let elapsed = t.elapsed().as_secs_f64();
        println!("1-groth16-bls12381 setup run time: {} secs", elapsed);

        //2. Prove
        let t1 = std::time::Instant::now();
        let mut wtns = WitnessCalculator::new(WASM_FILE_BLS12).unwrap();
        let inputs = load_input_for_witness(INPUT_FILE);
        let w = wtns.calculate_witness(inputs, false).unwrap();
        let w = w
            .iter()
            .map(|wi| {
                if wi.is_zero() {
                    Fr_bls12381::zero()
                } else {
                    // println!("wi: {}", wi);
                    Fr_bls12381::from_str(&wi.to_string()).unwrap()
                }
            })
            .collect::<Vec<_>>();
        let circuit1: CircomCircuit<Bls12> = CircomCircuit {
            r1cs: r1cs_reader::load_r1cs(CIRCUIT_FILE_BLS12),
            witness: Some(w),
            wire_mapping: None,
            aux_offset: 0,
        };
        let inputs = circuit1.get_public_inputs().unwrap();
        let proof = Groth16::prove(&params.0, circuit1.clone(), &mut rng)?;
        let elapsed1 = t1.elapsed().as_secs_f64();
        println!("2-groth16-bls12381 prove run time: {} secs", elapsed1);

        //3. Verify
        let t2 = std::time::Instant::now();
        let verified = Groth16::<_, CircomCircuit<Bls12>>::verify_with_processed_vk(
            &params.1, &inputs, &proof,
        )?;
        let elapsed2 = t2.elapsed().as_secs_f64();
        println!("3-groth16-bls12381 verify run time: {} secs", elapsed2);

        assert!(verified);

        Ok(())
    }
}
