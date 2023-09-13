#![allow(deprecated, dead_code)]
#![allow(clippy::derive_hash_xor_eq, clippy::too_many_arguments)]
use crate::field_bls12381::Fr;
use crate::matrix::Matrix;
use crate::mds::{create_mds_matrices, factor_to_sparse_matrixes, MdsMatrices, quintic_s_box};
use crate::round_constants::generate_constants;
use ff::*;

/// Using recommended parameters from whitepaper https://eprint.iacr.org/2019/458.pdf (table 2, table 8)
#[derive(Debug)]
pub struct Constants{
    pub mds_matrices: MdsMatrices<Fr>,
    pub round_constants: Option<Vec<Fr>>,
    pub full_rounds: usize,
    pub half_full_rounds: usize,
    pub partial_rounds: usize,
    pub constants_offset: usize,
}

fn round_numbers(arity: usize) -> (usize, usize) {
    let full_rounds = 8;
    let partial_rounds = match arity {
        2 => 55,
        3 => 55,
        4 => 56,
        5 => 56,
        6 => 56,
        7 => 56,
        8 => 57,
        9 => 57,
        10 => 57,
        11 => 57,
        12 => 57,
        13 => 57,
        14 => 57,
        15 => 57,
        16 => 59,
        17 => 59,
        25 => 59,
        37 => 60,
        65 => 61,
        _ => panic!("Invalid arity value provided: {}", arity),
    };
    (full_rounds, partial_rounds)
}

const SBOX: u8 = 1; // x^5
const FIELD: u8 = 1; // Gf(p)

pub fn load_bls12381_constants(arity: usize) -> Constants {
    let width = arity + 1;
    let mds_matrices = create_mds_matrices(width);
    let (full_rounds, partial_rounds) = round_numbers(arity);
    let half_full_rounds = full_rounds / 2;
    let constants_offset:usize = 0;
    let r_f = full_rounds as u16;
    let r_p = partial_rounds as u16;
    let round_constants = generate_constants::<Fr>(FIELD,SBOX,255 as u16,width as u16,r_f,r_p);

    let (pre_sparse_matrix, sparse_matrixes) =
        factor_to_sparse_matrixes(mds_matrices.m.clone(), partial_rounds);
    // Ensure we have enough constants for the sbox rounds
    assert!(
        width * (full_rounds + partial_rounds) <= round_constants.len(),
        "Not enough round constants"
    );

    Constants {
        mds_matrices,
        round_constants: Some(round_constants),
        full_rounds,
        half_full_rounds,
        partial_rounds,
        constants_offset,
    }
}

pub struct Poseidon;

impl Default for Poseidon {
    fn default() -> Self {
        Self::new()
    }
}

impl Poseidon {
    pub fn new() -> Poseidon {
        Self {}
    }

    pub fn mix(&self, state: &Vec<Fr>, m: &[Vec<Fr>]) -> Vec<Fr> {
        let mut new_state: Vec<Fr> = Vec::new();
        for i in 0..state.len() {
            new_state.push(Fr::zero());
            for (j, x) in state.iter().enumerate() {
                let mut mij = m[i][j];
                mij.mul_assign(x);
                new_state[i].add_assign(&mij);
            }
        }
        new_state
    }

    pub fn hash(&self, inp: &Vec<Fr>, mut p: Constants) -> Result<Fr, String> 
    {
        // This counter is incremented when a round constants is read. Therefore, the round constants never repeat.
        // The first full round should use the initial constants.
        self.full_round(inp, p);
        
        for _ in 1..p.half_full_rounds {
            self.full_round(inp, p);
        }

        partial_round(p);

        for _ in 1..p.partial_rounds {
            partial_round(p);
        }

        for _ in 0..p.half_full_rounds {
            self.full_round(inp, p);
        }

        p.elements[1]
    }

    pub fn full_round(&self, inp: &Vec<Fr>, mut p: Constants)
    {
        // Apply the quintic S-Box to all elements, after adding the round key.
        // Round keys are added in the S-box to match circuits (where the addition is free)
        // and in preparation for the shift to adding round keys after (rather than before) applying the S-box.
        let t = inp.len() + 1;
        let pre_round_keys = p
            .round_constants
            .as_ref()
            .unwrap()
            .iter()
            .map(Some);

        inp
            .iter_mut()
            .zip(pre_round_keys)
            .for_each(|(l, pre)| {
                quintic_s_box(l, pre, None);
            });

        p.constants_offset += inp.len();

        let mut state = vec![pre_round_keys.clone(), t];
        state[1..].clone_from_slice(&inp);

        // M(B)
        // Multiply the elements by the constant MDS matrix
        self.mix(&state, &[p.mds_matrices.m[t-1]]);
    }

    /// The partial round is the same as the full round, with the difference that we apply the S-Box only to the first bitflags poseidon leaf.
    pub fn partial_round(&self, inp: &Vec<Fr>, mut p: Constants)
    {
        // Every element of the hash buffer is incremented by the round constants
        self.add_round_constants(inp, p);

        // Apply the quintic S-Box to the first element
        quintic_s_box(&mut inp[0], None, None);

        let mut state = vec![pre_round_keys.clone(), t];
        state[1..].clone_from_slice(&inp);
        // Multiply the elements by the constant MDS matrix
        p.product_mds();
    }

    /// For every leaf, add the round constants with index defined by the constants offset, and increment the
    /// offset.
    fn add_round_constants(&self, inp: &Vec<Fr>, mut p: Constants)
    {
        for (element, round_constant) in inp.iter_mut().zip(
            p.round_constants
                .as_ref()
                .unwrap()
                .iter()
                .skip(p.constants_offset),
        ) {
            element.add_assign(round_constant);
        }

        p.constants_offset += inp.len();
    }

}