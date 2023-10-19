use crate::bellman_ce::pairing::{bls12_381::Bls12, bn256::Bn256};
use algebraic::errors::{EigenError, Result};
use algebraic::witness::{load_input_for_witness, WitnessCalculator};
use algebraic::circom_circuit::CircomCircuit;
use algebraic::reader::load_r1cs;
use crate::groth16::Groth16;
use crate::snark::SNARK;

// TODO plz move this function into zkit and remove this file
pub fn groth16_setup(circuit_file: &String, curve_type: &str) -> Result<()> {
    let mut rng = rand::thread_rng();
    match curve_type {
        "bn256" => {
            let circuit: CircomCircuit<Bn256> = CircomCircuit {
                r1cs: load_r1cs(circuit_file),
                witness: None,
                wire_mapping: None,
                aux_offset: 0,
            };
            let (pk, pvk) = Groth16::circuit_specific_setup(circuit, &mut rng)?;

            // let pk_json = serde_json::to_string(&pk)?;
            // let pk_file = std::fs::File::create("pk.json")?;
            // std::fs::write(pk_file, pk_json.as_bytes())?;

            // let pvk_json = serde_json::to_string(&pvk)?;
            // let mut pvk_file = std::fs::File::create("pvk.json")?;
            // std::fs::write(pvk_json.as_bytes())?;
        }
        "bls12381" => {
            let circuit: CircomCircuit<Bls12> = CircomCircuit {
                r1cs: load_r1cs(circuit_file),
                witness: None,
                wire_mapping: None,
                aux_offset: 0,
            };
            let (pk, pvk) = Groth16::circuit_specific_setup(circuit, &mut rng)?;

            // let pk_json = serde_json::to_string(&pk)?;
            // let mut pk_file = std::fs::File::create("pk.json")?;
            // std::fs::write(pk_json.as_bytes())?;

            // let pvk_json = serde_json::to_string(&pvk)?;
            // let mut pvk_file = std::fs::File::create("pvk.json")?;
            // std::fs::write(pvk_json.as_bytes())?;
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
