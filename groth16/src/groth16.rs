#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use crate::bellman_ce::{groth16::*, Circuit};
use anyhow::Result;
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use franklin_crypto::bellman::pairing::Engine;
#[allow(unused_imports)]
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
use rand_old::{self as rand, Rng};

#[cfg(any(feature = "cuda", feature = "opencl"))]
use bellperson::{gpu, groth16::*, Circuit};
#[cfg(any(feature = "cuda", feature = "opencl"))]
use group::WnafGroup;
#[cfg(any(feature = "cuda", feature = "opencl"))]
use pairing::{Engine, MultiMillerLoop};
#[cfg(any(feature = "cuda", feature = "opencl"))]
use rand_core::RngCore;
use std::marker::PhantomData;

#[cfg(any(feature = "cuda", feature = "opencl"))]
pub struct Groth16<E: Engine, C: Circuit<E::Fr>> {
    _engine: PhantomData<E>,
    _circuit: PhantomData<C>,
}

#[cfg(any(feature = "cuda", feature = "opencl"))]
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

#[cfg(not(any(feature = "cuda", feature = "opencl")))]
pub struct Groth16<E: Engine, C: Circuit<E>> {
    _engine: PhantomData<E>,
    _circuit: PhantomData<C>,
}
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
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
#[cfg(not(any(feature = "cuda", feature = "opencl")))]
mod tests {
    use anyhow::Ok;
    use franklin_crypto::bellman::{Field, PrimeField};
    use num_traits::Zero;

    use super::*;
    use crate::api::create_circuit_add_witness;
    use crate::api::SetupResult;
    use crate::api::{groth16_prove_inplace, groth16_setup_inplace, groth16_verify_inplace};
    use crate::bellman_ce::bls12_381::Bls12;
    use crate::bellman_ce::bn256::{Bn256, Fr};
    use algebraic::circom_circuit::CircomCircuit;
    use algebraic::reader;
    use algebraic::witness::{load_input_for_witness, WitnessCalculator};
    const INPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.input.json");
    const CIRCUIT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.r1cs");
    const WASM_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.wasm");
    const CIRCUIT_FILE_BLS12: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/test-vectors/mycircuit_bls12381.r1cs");
    const WASM_FILE_BLS12: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/test-vectors/mycircuit_bls12381.wasm");

    #[test]
    fn groth16_proof() -> Result<()> {
        //1. SRS
        let t = std::time::Instant::now();
        let circuit: CircomCircuit<Bn256> = CircomCircuit {
            r1cs: reader::load_r1cs(CIRCUIT_FILE),
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
        let mut wtns = WitnessCalculator::from_file(WASM_FILE)?;
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
            r1cs: reader::load_r1cs(CIRCUIT_FILE),
            witness: Some(w),
            wire_mapping: None,
            aux_offset: 0,
        };
        let inputs = circuit1.get_public_inputs().unwrap();
        let proof = Groth16::prove(&params.0, circuit1, &mut rng)?;
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
    fn groth16_proof_bls12381_inpace() -> Result<()> {
        //1. SRS
        let t = std::time::Instant::now();
        let setup_result = groth16_setup_inplace("BLS12381", CIRCUIT_FILE_BLS12)?;
        let (circuit, pk, vk) = match setup_result {
            SetupResult::BLS12381(circuit, pk, vk) => (circuit, pk, vk),
            _ => panic!("Expected BLS12381 setup result"),
        };
        let elapsed = t.elapsed().as_secs_f64();
        println!("1-groth16-bls12381 setup run time: {} secs", elapsed);

        //2. Prove
        let t1 = std::time::Instant::now();
        let mut rng = rand::thread_rng();
        let mut wtns = WitnessCalculator::from_file(WASM_FILE_BLS12)?;
        let inputs = load_input_for_witness(INPUT_FILE);
        let w = wtns.calculate_witness(inputs, false).unwrap();
        let circuit1: CircomCircuit<Bls12> = create_circuit_add_witness::<Bls12>(circuit, w);
        let proof = Groth16::prove(&pk, circuit1.clone(), &mut rng)?;
        let elapsed1 = t1.elapsed().as_secs_f64();
        println!("2-groth16-bls12381 prove run time: {} secs", elapsed1);

        //3. Verify
        let t2 = std::time::Instant::now();
        let inputs = circuit1.get_public_inputs().unwrap();
        let verified =
            Groth16::<_, CircomCircuit<Bls12>>::verify_with_processed_vk(&vk, &inputs, &proof)?;
        let elapsed2 = t2.elapsed().as_secs_f64();
        println!("3-groth16-bls12381 verify run time: {} secs", elapsed2);

        assert!(verified);

        Ok(())
    }

    #[test]
    fn groth16_api_proof_inpace() -> Result<()> {
        //1. SRS
        let t = std::time::Instant::now();
        let curve_type = "BLS12381";
        let public_input_file = "/tmp/public_input.json";
        let proof_file = "/tmp/proof.json";
        let setup_result = groth16_setup_inplace(curve_type, CIRCUIT_FILE_BLS12)?;
        let (circuit, pk, vk) = match setup_result {
            SetupResult::BLS12381(circuit, pk, vk) => (circuit, pk, vk),
            _ => panic!("Expected BLS12381 setup result"),
        };
        let elapsed = t.elapsed().as_secs_f64();
        println!("1-groth16-bls12381 setup run time: {} secs", elapsed);

        //2. Prove
        let t1 = std::time::Instant::now();
        let _ = groth16_prove_inplace(
            curve_type,
            circuit,
            WASM_FILE_BLS12,
            pk,
            INPUT_FILE,
            public_input_file,
            proof_file,
            false,
        );
        let elapsed1 = t1.elapsed().as_secs_f64();
        println!("2-groth16-bls12381 prove run time: {} secs", elapsed1);

        //3. Verify
        let t2 = std::time::Instant::now();
        let _ = groth16_verify_inplace(vk, public_input_file, proof_file);
        let elapsed2 = t2.elapsed().as_secs_f64();
        println!("3-groth16-bls12381 verify run time: {} secs", elapsed2);

        Ok(())
    }
}

#[cfg(test)]
#[cfg(any(feature = "cuda", feature = "opencl"))]
mod tests {
    use super::*;
    use crate::api::{create_circuit_add_witness, groth16_setup_inplace, SetupResult};
    use algebraic::witness::{load_input_for_witness, WitnessCalculator};
    use algebraic_gpu::circom_circuit::CircomCircuit;
    use algebraic_gpu::reader;
    use blstrs::{Bls12, Scalar};
    use ff::{Field, PrimeField};
    use log::info;
    use num_traits::Zero;
    use rand_new::rngs::OsRng;
    const INPUT_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/multiplier.input.json");
    const CIRCUIT_FILE_BLS12: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/test-vectors/mycircuit_bls12381.r1cs");
    const WASM_FILE_BLS12: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/test-vectors/mycircuit_bls12381.wasm");

    #[test]
    fn groth16_proof_bls12381_inplace() -> Result<()> {
        let _ = env_logger::try_init();
        //1. SRS
        let t = std::time::Instant::now();
        let setup_result = groth16_setup_inplace("BLS12381", CIRCUIT_FILE_BLS12)?;
        let (circuit, pk, vk) = match setup_result {
            SetupResult::BLS12381(circuit, pk, vk) => (circuit, pk, vk),
            _ => panic!("Expected BLS12381 setup result"),
        };
        let elapsed = t.elapsed().as_secs_f64();
        info!("1-groth16-bls12381 setup run time: {} secs", elapsed);

        //2. Prove
        let t1 = std::time::Instant::now();
        let mut wtns = WitnessCalculator::from_file(WASM_FILE_BLS12)?;
        let inputs = load_input_for_witness(INPUT_FILE);
        let w = wtns.calculate_witness(inputs, false).unwrap();
        let circuit1: CircomCircuit<Scalar> = create_circuit_add_witness(circuit, w);
        let proof = Groth16::prove(&pk, circuit1.clone(), &mut OsRng)?;
        let elapsed1 = t1.elapsed().as_secs_f64();
        info!("2-groth16-bls12381 prove run time: {} secs", elapsed1);

        //3. Verify
        let t2 = std::time::Instant::now();
        let inputs = circuit1.get_public_inputs().unwrap();
        let verified =
            Groth16::<_, CircomCircuit<Scalar>>::verify_with_processed_vk(&vk, &inputs, &proof)?;
        let elapsed2 = t2.elapsed().as_secs_f64();
        info!("3-groth16-bls12381 verify run time: {} secs", elapsed2);

        assert!(verified);

        Ok(())
    }
}
