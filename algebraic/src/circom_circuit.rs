// copy from https://github.com/poma/zkutil/blob/master/src/circom_circuit.rs partially
#![allow(clippy::needless_range_loop)]
extern crate rand;

use itertools::Itertools;
use std::collections::BTreeMap;
use std::str;

use crate::bellman_ce::{
    pairing::Engine, Circuit, ConstraintSystem, Index, LinearCombination, PrimeField, ScalarEngine,
    SynthesisError, Variable,
};

use crate::utils::repr_to_big;

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

pub type Constraint<E> = (
    Vec<(usize, <E as ScalarEngine>::Fr)>,
    Vec<(usize, <E as ScalarEngine>::Fr)>,
    Vec<(usize, <E as ScalarEngine>::Fr)>,
);

// R1CSfile's CustomGates
#[derive(Debug, Default, Clone)]
pub struct CustomGates<E: ScalarEngine> {
    pub template_name: String,
    pub parameters: Vec<E::Fr>,
}

// R1CSfile's CustomGatesUses
#[derive(Debug, Default, Clone)]
pub struct CustomGatesUses {
    pub id: u64,
    pub signals: Vec<u64>,
}

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

#[derive(Clone, Debug)]
pub struct CircomCircuit<E: ScalarEngine> {
    pub r1cs: R1CS<E>,
    pub witness: Option<Vec<E::Fr>>,
    pub wire_mapping: Option<Vec<usize>>,
    pub aux_offset: usize,
    // debug symbols
}

impl<E: ScalarEngine> CircomCircuit<E> {
    pub fn get_public_inputs(&self) -> Option<Vec<E::Fr>> {
        match &self.witness {
            None => None,
            Some(w) => match &self.wire_mapping {
                None => Some(w[1..self.r1cs.num_inputs].to_vec()),
                Some(m) => Some(m[1..self.r1cs.num_inputs].iter().map(|i| w[*i]).collect_vec()),
            },
        }
    }

    pub fn get_public_inputs_json(&self) -> String {
        let inputs = self.get_public_inputs();
        let inputs = match inputs {
            None => return String::from("[]"),
            Some(inp) => inp.iter().map(|x| repr_to_big(x.into_repr())).collect_vec(),
        };
        serde_json::to_string_pretty(&inputs).unwrap()
    }
}

/// Our demo circuit implements this `Circuit` trait which
/// is used during paramgen and proving in order to
/// synthesize the constraint system.
impl<E: Engine> Circuit<E> for CircomCircuit<E> {
    //noinspection RsBorrowChecker
    fn synthesize<CS: ConstraintSystem<E>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        let witness = &self.witness;
        let wire_mapping = &self.wire_mapping;
        for i in 1..self.r1cs.num_inputs {
            cs.alloc_input(
                || format!("variable {}", i),
                || {
                    Ok(match witness {
                        None => E::Fr::from_str(&format!("alloc input {} error", i)).unwrap(),
                        Some(w) => match wire_mapping {
                            None => w[i],
                            Some(m) => w[m[i]],
                        },
                    })
                },
            )?;
        }
        for i in 0..self.r1cs.num_aux {
            cs.alloc(
                || format!("aux {}", i + self.aux_offset),
                || {
                    Ok(match witness {
                        None => {
                            E::Fr::from_str(&format!("alloc aux {} error", i + self.aux_offset))
                                .unwrap()
                        }
                        Some(w) => match wire_mapping {
                            None => w[i + self.r1cs.num_inputs],
                            Some(m) => w[m[i + self.r1cs.num_inputs]],
                        },
                    })
                },
            )?;
        }

        let make_index = |index| {
            if index < self.r1cs.num_inputs {
                Index::Input(index)
            } else {
                Index::Aux(index - self.r1cs.num_inputs + self.aux_offset)
            }
        };
        let make_lc = |lc_data: Vec<(usize, E::Fr)>| {
            lc_data.iter().fold(
                LinearCombination::<E>::zero(),
                |lc: LinearCombination<E>, (index, coeff)| {
                    lc + (*coeff, Variable::new_unchecked(make_index(*index)))
                },
            )
        };
        for (i, constraint) in self.r1cs.constraints.iter().enumerate() {
            // 0 * LC = 0 must be ignored
            if !((constraint.0.is_empty() || constraint.1.is_empty()) && constraint.2.is_empty()) {
                cs.enforce(
                    || format!("{}", i),
                    |_| make_lc(constraint.0.clone()),
                    |_| make_lc(constraint.1.clone()),
                    |_| make_lc(constraint.2.clone()),
                );
            }
        }
        Ok(())
    }
}
