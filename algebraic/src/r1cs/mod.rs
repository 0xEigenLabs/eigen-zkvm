pub mod constraint;
pub mod custom_gate;
pub mod header;
pub mod r1cs_file;
pub(crate) mod utils;

use crate::bellman_ce::{PrimeField, ScalarEngine};

use crate::r1cs::constraint::Constraint;
use crate::r1cs::custom_gate::{CustomGates, CustomGatesUses};
use crate::r1cs::r1cs_file::R1CSFile;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, Seek};
use std::str;

/// R1CS spec: https://www.sikoba.com/docs/SKOR_GD_R1CS_Format.pdf
#[derive(Clone, Debug)]
pub struct R1CS<E: ScalarEngine> {
    pub num_inputs: usize,
    pub num_aux: usize,
    pub num_variables: usize,
    pub num_outputs: usize,
    pub constraints: Vec<Constraint<E>>,
    pub custom_gates: Vec<CustomGates<E>>,
    pub custom_gates_uses: Vec<CustomGatesUses>,
}

#[derive(Serialize, Deserialize)]
pub struct CircuitJson {
    pub constraints: Vec<Vec<BTreeMap<String, String>>>,
    #[serde(rename = "nPubInputs")]
    pub num_inputs: usize,
    #[serde(rename = "nOutputs")]
    pub num_outputs: usize,
    #[serde(rename = "nVars")]
    pub num_variables: usize,
}

impl<E: ScalarEngine> R1CS<E> {
    /// load r1cs file by filename with autodetect encoding (bin or json)
    pub fn load_r1cs(filename: &str) -> R1CS<E> {
        let file = OpenOptions::new()
            .read(true)
            .open(filename)
            .unwrap_or_else(|_| panic!("unable to open {}.", filename));

        let render = BufReader::new(file);

        if filename.ends_with("json") {
            Self::load_r1cs_from_json(render)
        } else {
            let (r1cs, _wire_mapping) = Self::load_r1cs_from_bin(render);

            r1cs
        }
    }

    /// load r1cs from bin by a reader
    fn load_r1cs_from_bin<R: Read + Seek>(reader: R) -> (R1CS<E>, Vec<usize>) {
        let file = R1CSFile::from_reader::<R>(reader).expect("unable to read.");
        let num_inputs = (1 + file.header.n_pub_in + file.header.n_pub_out) as usize;
        let num_variables = file.header.n_wires as usize;
        let num_aux = num_variables - num_inputs;
        (
            R1CS {
                num_aux,
                num_inputs,
                num_variables,
                num_outputs: file.header.n_pub_out as usize,
                constraints: file.constraints,
                custom_gates: file.custom_gates,
                custom_gates_uses: file.custom_gates_uses,
            },
            file.wire_mapping.iter().map(|e| *e as usize).collect_vec(),
        )
    }

    /// load r1cs from json by a reader
    fn load_r1cs_from_json<R: Read>(reader: R) -> R1CS<E> {
        let circuit_json: CircuitJson = serde_json::from_reader(reader).expect("unable to read.");

        let num_inputs = circuit_json.num_inputs + circuit_json.num_outputs + 1;
        let num_aux = circuit_json.num_variables - num_inputs;

        let convert_constraint = |lc: &BTreeMap<String, String>| {
            lc.iter()
                .map(|(index, coeff)| (index.parse().unwrap(), E::Fr::from_str(coeff).unwrap()))
                .collect_vec()
        };

        let constraints = circuit_json
            .constraints
            .iter()
            .map(|c| {
                (
                    convert_constraint(&c[0]),
                    convert_constraint(&c[1]),
                    convert_constraint(&c[2]),
                )
            })
            .collect_vec();

        R1CS {
            num_inputs,
            num_aux,
            num_variables: circuit_json.num_variables,
            num_outputs: circuit_json.num_outputs,
            constraints,
            custom_gates: vec![],
            custom_gates_uses: vec![],
        }
    }
}
