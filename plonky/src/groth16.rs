use crate::bellman_ce::{groth16::*, Circuit};
use crate::errors::Result;
use crate::franklin_crypto::bellman::pairing::Engine;
use crate::snark::SNARK;
use rand::Rng;
use std::marker::PhantomData;

pub struct Groth16<E: Engine, C: Circuit<E>> {
    _engine: PhantomData<E>,
    _circuit: PhantomData<C>,
}

impl<E: Engine, C: Circuit<E>> SNARK<E::Fr> for Groth16<E, C> {
    type Circuit = C;
    type AssignedCircuit = C;
    type ProvingKey = Parameters<E>;
    type VerificationKey = VerifyingKey<E>;
    type PreparedVerificationKey = PreparedVerifyingKey<E>;
    type Proof = Proof<E>;

    fn circuit_specific_setup<R: Rng>(
        circuit: Self::Circuit,
        rng: &mut R,
    ) -> Result<(Self::ProvingKey, Self::PreparedVerificationKey)> {
        #[cfg(debug_assertions)]
        let t = std::time::Instant::now();

        let pk = generate_random_parameters::<E, Self::Circuit, R>(circuit, rng)?;
        let pvk = prepare_verifying_key(&pk.vk);

        #[cfg(debug_assertions)]
        {
            let elapsed = t.elapsed().as_secs_f64();
            log::debug!("groth16 setup run time: {} secs", elapsed);
        }

        Ok((pk, pvk))
    }

    fn prove<R: Rng>(
        circuit_pk: &Self::ProvingKey,
        input_and_witness: Self::AssignedCircuit,
        rng: &mut R,
    ) -> Result<Self::Proof> {
        #[cfg(debug_assertions)]
        let t = std::time::Instant::now();

        let result = create_random_proof::<E, _, _, _>(input_and_witness, circuit_pk, rng)?;

        #[cfg(debug_assertions)]
        {
            let elapsed = t.elapsed().as_secs_f64();
            log::debug!("groth16 generate proof run time: {} secs", elapsed);
        }

        Ok(result)
    }

    fn verify_with_processed_vk(
        circuit_pvk: &Self::PreparedVerificationKey,
        public_input: &[E::Fr],
        proof: &Self::Proof,
    ) -> Result<bool> {
        #[cfg(debug_assertions)]
        let t = std::time::Instant::now();

        let result = verify_proof(&circuit_pvk, proof, &public_input)?;

        #[cfg(debug_assertions)]
        {
            let elapsed = t.elapsed().as_secs_f64();
            log::debug!("groth16 verify run time: {} secs", elapsed);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bellman_ce::bn256::Bn256;
    use crate::circom_circuit::CircomCircuit;
    use crate::reader;
    const CIRCUIT_FILE: &'static str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.r1cs");
    const WITNESS_FILE: &'static str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../test/single/witness.wtns");

    #[test]
    fn groth16_proof() -> Result<()> {
        env_logger::init();
        let circuit: CircomCircuit<Bn256> = CircomCircuit {
            r1cs: reader::load_r1cs(CIRCUIT_FILE),
            witness: Some(reader::load_witness_from_file::<Bn256>(WITNESS_FILE)),
            wire_mapping: None,
            aux_offset: 0,
        };
        let mut rng = rand::thread_rng();
        let params = Groth16::circuit_specific_setup(circuit.clone(), &mut rng)?;

        let proof = Groth16::prove(&params.0, circuit.clone(), &mut rng)?;

        let inputs: Vec<franklin_crypto::bellman::bn256::Fr> = circuit.get_public_inputs().unwrap();
        let verified = Groth16::<_, CircomCircuit<Bn256>>::verify_with_processed_vk(
            &params.1, &inputs, &proof,
        )?;

        assert!(verified);

        Ok(())
    }
}
