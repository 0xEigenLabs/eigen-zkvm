use algebraic::witness::witness_calculator::WitnessCalculator;
use algebraic::{
    circom_circuit::CircomCircuit,
    errors::{EigenError, Result},
    reader::load_r1cs,
    witness::load_input_for_witness,
    Field, PrimeField,
};
use groth16::bellman_ce::pairing::{
    bls12_381::{Bls12, Fr as Fr_bls12381},
    bn256::{Bn256, Fr},
};
use groth16::bellman_ce::plonk::better_cs::keys::{read_fr_vec, write_fr_vec};
use groth16::bellman_ce::{
    groth16::{Parameters, Proof, VerifyingKey},
    Engine,
};
use groth16::groth16::Groth16;
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
            write_pk_vk_to_files(pk, vk, pk_file, vk_file)?
        }
        "bls12381" => {
            let circuit = create_circuit_from_file::<Bls12>(circuit_file, None);
            let (pk, vk) = Groth16::circuit_specific_setup(circuit, &mut rng)?;
            write_pk_vk_to_files(pk, vk, pk_file, vk_file)?
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
    let inputs = load_input_for_witness(input_file);
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
                        // println!("wi: {}", wi);
                        Fr::from_str(&wi.to_string()).unwrap()
                    }
                })
                .collect::<Vec<_>>();
            let circuit = create_circuit_from_file::<Bn256>(circuit_file, Some(w));
            let inputs = circuit.get_public_inputs().unwrap();
            let proof = Groth16::prove(&pk, circuit.clone(), &mut rng)?;
            let writer = std::fs::File::create(proof_file)?;
            proof.write(writer)?;
            let mut writer1 = std::fs::File::create(public_input_file)?;
            write_fr_vec(&inputs, &mut writer1)?;
        }
        "bls12381" => {
            let pk: Parameters<Bls12> = read_pk_from_file(pk_file, false)?;
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
            let circuit = create_circuit_from_file::<Bls12>(circuit_file, Some(w));
            let inputs = circuit.get_public_inputs().unwrap();
            let proof = Groth16::prove(&pk, circuit.clone(), &mut rng)?;
            let writer = std::fs::File::create(proof_file)?;
            proof.write(writer)?;
            let mut writer1 = std::fs::File::create(public_input_file)?;
            write_fr_vec(&inputs, &mut writer1)?;
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
            let inputs = read_public_input_from_file(public_input_file)?;
            let proof = read_proof_from_file(proof_file)?;

            let verification_result =
                Groth16::<_, CircomCircuit<Bn256>>::verify_with_processed_vk(&vk, &inputs, &proof);

            if verification_result.is_err() || !verification_result.unwrap() {
                return Err(EigenError::Unknown("verify failed".to_string()));
            }
        }

        "bls12381" => {
            let vk = read_vk_from_file(vk_file)?;
            let inputs = read_public_input_from_file_bls12381(public_input_file)?;
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
        r1cs: load_r1cs(circuit_file),
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

fn read_vk_from_file<E: Engine>(file_path: &str) -> Result<VerifyingKey<E>> {
    let file = std::fs::File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    Ok(VerifyingKey::<E>::read(&mut reader)?)
}

fn read_public_input_from_file(file_path: &str) -> Result<Vec<Fr>> {
    let file = std::fs::File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    Ok(read_fr_vec::<Fr, _>(&mut reader)?)
}

fn read_public_input_from_file_bls12381(file_path: &str) -> Result<Vec<Fr_bls12381>> {
    let file = std::fs::File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    Ok(read_fr_vec::<Fr_bls12381, _>(&mut reader)?)
}

fn read_proof_from_file<E: Engine>(file_path: &str) -> Result<Proof<E>> {
    let file = std::fs::File::open(file_path)?;
    let mut reader = std::io::BufReader::new(file);
    Ok(Proof::<E>::read(&mut reader)?)
}

fn write_pk_vk_to_files<E: Engine>(
    pk: Parameters<E>,
    vk: VerifyingKey<E>,
    pk_file: &str,
    vk_file: &str,
) -> Result<()> {
    let writer = std::fs::File::create(pk_file)?;
    pk.write(writer)?;
    let writer1 = std::fs::File::create(vk_file)?;
    vk.write(writer1)?;
    Ok(())
}
