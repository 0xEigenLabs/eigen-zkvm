pub mod constraint;
pub mod custom_gate;
pub mod header;
pub mod r1cs_file;
pub(crate) mod utils;

use itertools::Itertools;
use std::collections::BTreeMap;
use std::str;

use crate::bellman_ce::{
    pairing::Engine, Circuit, ConstraintSystem, Index, LinearCombination, PrimeField, ScalarEngine,
    SynthesisError, Variable,
};
use crate::r1cs::constraint::Constraint;
use crate::r1cs::custom_gate::{CustomGates, CustomGatesUses};

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
