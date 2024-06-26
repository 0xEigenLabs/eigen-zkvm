use anyhow::Result;
use bellperson::{gpu, groth16::*, Circuit};
use group::WnafGroup;
use pairing::{Engine, MultiMillerLoop};
use rand_core::RngCore;
use std::marker::PhantomData;

pub struct Groth16<E: Engine, C: Circuit<E::Fr>> {
    _engine: PhantomData<E>,
    _circuit: PhantomData<C>,
}

impl<E, C> Groth16<E, C>
where
    E: MultiMillerLoop,
    E::G1: WnafGroup,
    E::G2: WnafGroup,
    C: Circuit<E::Fr> + std::marker::Send,
    E::Fr: gpu::GpuName,
{
    pub fn circuit_specific_setup<R: RngCore>(
        circuit: C,
        rng: &mut R,
    ) -> Result<(Parameters<E>, VerifyingKey<E>)> {
        let pk: Parameters<E> = generate_random_parameters::<E, C, R>(circuit, rng)?;
        let vk = pk.vk.clone();

        Ok((pk, vk))
    }

    pub fn prove<R: RngCore>(
        circuit_pk: &Parameters<E>,
        input_and_witness: C,
        rng: &mut R,
    ) -> Result<Proof<E>>
    where
        E::G1Affine: gpu::GpuName,
        E::G2Affine: gpu::GpuName,
    {
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
    use super::*;
    use crate::circuit::CircomCircuit;
    use crate::reader;
    use crate::witness::{load_input_for_witness, WitnessCalculator};
    use blstrs::{Bls12, Scalar};
    use ff::{Field, PrimeField};
    use num_traits::Zero;
    use rand::rngs::OsRng;
    const INPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.input.json");
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
        let circuit: CircomCircuit<Scalar> = CircomCircuit {
            r1cs: reader::load_r1cs(CIRCUIT_FILE_BLS12),
            witness: None,
            wire_mapping: None,
            aux_offset: 0,
        };
        let params = Groth16::circuit_specific_setup(circuit, &mut OsRng)?;
        let elapsed = t.elapsed().as_secs_f64();
        println!("1-groth16-bls12381 setup run time: {} secs", elapsed);

        //2. Prove
        let t1 = std::time::Instant::now();
        let mut wtns = WitnessCalculator::from_file(WASM_FILE_BLS12)?;
        let inputs = load_input_for_witness(INPUT_FILE);
        let w = wtns.calculate_witness(inputs, false).unwrap();
        let w = w
            .iter()
            .map(|wi| {
                if wi.is_zero() {
                    <Bls12 as Engine>::Fr::ZERO
                } else {
                    // println!("wi: {}", wi);
                    <Bls12 as Engine>::Fr::from_str_vartime(&wi.to_string()).unwrap()
                }
            })
            .collect::<Vec<_>>();
        let circuit1: CircomCircuit<Scalar> = CircomCircuit {
            r1cs: reader::load_r1cs(CIRCUIT_FILE_BLS12),
            witness: Some(w),
            wire_mapping: None,
            aux_offset: 0,
        };
        let inputs = circuit1.get_public_inputs().unwrap();
        let proof: bellperson::groth16::Proof<Bls12> =
            Groth16::prove(&params.0, circuit1, &mut OsRng)?;
        let elapsed1 = t1.elapsed().as_secs_f64();
        println!("2-groth16-bls12381 prove run time: {} secs", elapsed1);

        //3. Verify
        let t2 = std::time::Instant::now();
        let verified = Groth16::<_, CircomCircuit<Scalar>>::verify_with_processed_vk(
            &params.1, &inputs, &proof,
        )?;
        let elapsed2 = t2.elapsed().as_secs_f64();
        println!("3-groth16-bls12381 verify run time: {} secs", elapsed2);

        assert!(verified);

        Ok(())
    }
}
