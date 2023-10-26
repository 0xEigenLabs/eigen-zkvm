use algebraic::circom_witness::WitnessCalculator;
use algebraic::r1cs::R1CS;
use algebraic::{
    bellman_ce::Engine,
    circom_circuit::CircomCircuit,
    errors::{EigenError, Result},
    Field, PrimeField,
};
use groth16::{
    bellman_ce::{
        groth16::{Parameters, Proof, VerifyingKey},
        pairing::{
            bls12_381::{Bls12, Fr as Fr_bls12381},
            bn256::{Bn256, Fr},
        },
    },
    groth16::Groth16,
    json_utils::*,
};
use num_traits::Zero;
use rand;

pub fn groth16_setup(
    curve_type: &str,
    circuit_file: &str,
    pk_file: &str,
    vk_file: &str,
) -> Result<()> {
    let mut rng = rand::thread_rng();
    match curve_type {
        "bn128" => {
            let circuit = create_circuit_from_file::<Bn256>(circuit_file, None);
            let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut rng)?;
            write_pk_vk_to_files(curve_type, pk, vk, pk_file, vk_file)?
        }
        "bls12381" => {
            let circuit = create_circuit_from_file::<Bls12>(circuit_file, None);
            let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut rng)?;
            write_pk_vk_to_files(curve_type, pk, vk, pk_file, vk_file)?
        }
        _ => {
            return Err(EigenError::Unknown(format!(
                "Unknown curve type: {}",
                curve_type
            )))
        }
    };
    Ok(())
}

pub fn groth16_prove(
    curve_type: &str,
    circuit_file: &str,
    wtns_file: &str,
    pk_file: &str,
    input_file: &str,
    public_input_file: &str,
    proof_file: &str,
) -> Result<()> {
    let mut rng = rand::thread_rng();

    let mut wtns = WitnessCalculator::new(wtns_file)?;
    let inputs = WitnessCalculator::load_input_for_witness(input_file);
    let w = wtns.calculate_witness(inputs, false)?;
    match curve_type {
        "bn128" => {
            let pk: Parameters<Bn256> = read_pk_from_file(pk_file, false)?;
            let w = w
                .iter()
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
            let proof_json = serialize_proof(&proof, curve_type, false)?;
            std::fs::write(proof_file, proof_json)?;
            let input_json = circuit.get_public_inputs_json();
            std::fs::write(public_input_file, input_json)?;
        }
        "bls12381" => {
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
            let proof_json = serialize_proof(&proof, curve_type, false)?;
            std::fs::write(proof_file, proof_json)?;
            let input_json = circuit.get_public_inputs_json();
            std::fs::write(public_input_file, input_json)?;
        }
        _ => {
            return Err(EigenError::Unknown(format!(
                "Unknown curve type: {}",
                curve_type
            )))
        }
    };

    Ok(())
}

pub fn groth16_verify(
    curve_type: &str,
    vk_file: &str,
    public_input_file: &str,
    proof_file: &str,
) -> Result<()> {
    match curve_type {
        "bn128" => {
            let vk = read_vk_from_file(vk_file)?;
            let inputs = read_public_input_from_file::<Fr>(public_input_file)?;
            let proof = read_proof_from_file(proof_file)?;

            let verification_result =
                Groth16::<_, CircomCircuit<Bn256>>::verify_with_processed_vk(&vk, &inputs, &proof);

            if verification_result.is_err() || !verification_result.unwrap() {
                return Err(EigenError::Unknown("verify failed".to_string()));
            }
        }

        "bls12381" => {
            let vk = read_vk_from_file(vk_file)?;
            let inputs = read_public_input_from_file::<Fr_bls12381>(public_input_file)?;
            let proof = read_proof_from_file(proof_file)?;

            let verification_result =
                Groth16::<_, CircomCircuit<Bls12>>::verify_with_processed_vk(&vk, &inputs, &proof);

            if verification_result.is_err() || !verification_result.unwrap() {
                return Err(EigenError::Unknown("verify failed".to_string()));
            }
        }

        _ => {
            return Err(EigenError::Unknown(format!(
                "Unknown curve type: {}",
                curve_type
            )))
        }
    }

    Ok(())
}

fn create_circuit_from_file<E: Engine>(
    circuit_file: &str,
    witness: Option<Vec<E::Fr>>,
) -> CircomCircuit<E> {
    CircomCircuit {
        r1cs: R1CS::load_r1cs(circuit_file),
        witness,
        wire_mapping: None,
        aux_offset: 0,
    }
}

fn read_pk_from_file<E: Engine>(file_path: &str, checked: bool) -> Result<Parameters<E>> {
    let file = std::fs::File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    Ok(Parameters::<E>::read(&mut reader, checked)?)
}

fn read_vk_from_file<P: Parser>(file_path: &str) -> Result<VerifyingKey<P>> {
    let json_data = std::fs::read_to_string(file_path)?;
    Ok(to_verification_key::<P>(&json_data))
}

fn read_public_input_from_file<T: PrimeField>(file_path: &str) -> Result<Vec<T>> {
    let json_data = std::fs::read_to_string(file_path)?;
    Ok(to_public_input::<T>(&json_data))
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
) -> Result<()> {
    let writer = std::fs::File::create(pk_file)?;
    pk.write(writer)?;
    let vk_json = serialize_vk(&vk, curve_type, false)?;
    std::fs::write(vk_file, vk_json)?;
    Ok(())
}
