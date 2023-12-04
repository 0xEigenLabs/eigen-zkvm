#![allow(deprecated, dead_code)]
#![allow(clippy::derived_hash_with_manual_eq, clippy::too_many_arguments)]
use crate::constant::POSEIDON_BLS12381_CONSTANTS;
use crate::field_bls12381::Fr;
use crate::poseidon_bls12381_constants as constants;
use ff::*;

/// Using recommended parameters from whitepaper https://eprint.iacr.org/2019/458.pdf (table 2, table 8)
/// Generated by https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/master/code/calc_round_numbers.py
/// And rounded up to nearest integer that divides by t
#[derive(Debug)]
pub struct Constants {
    pub c: Vec<Vec<Fr>>,
    pub m: Vec<Vec<Vec<Fr>>>,
    pub p: Vec<Vec<Vec<Fr>>>,
    pub s: Vec<Vec<Fr>>,
    pub n_rounds_f: usize,
    pub n_rounds_p: Vec<usize>,
}

pub fn load_constants() -> Constants {
    let (c_str, m_str) = constants::constants();
    let mut c: Vec<Vec<Fr>> = Vec::new();
    for v1 in c_str {
        let mut cci: Vec<Fr> = Vec::new();
        for v2 in v1 {
            let b: Fr = from_hex(v2).unwrap();
            cci.push(b);
        }
        c.push(cci);
    }
    let mut m: Vec<Vec<Vec<Fr>>> = Vec::new();
    for v1 in m_str {
        let mut mi: Vec<Vec<Fr>> = Vec::new();
        for v2 in v1 {
            let mut mij: Vec<Fr> = Vec::new();
            for s in v2 {
                let b: Fr = from_hex(s).unwrap();
                mij.push(b);
            }
            mi.push(mij);
        }
        m.push(mi);
    }
    Constants {
        c,
        m,
        p: Vec::new(),
        s: Vec::new(),
        n_rounds_f: 8,
        n_rounds_p: vec![
            56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
        ],
    }
}

#[deprecated(
    since = "0.1.0",
    note = "please use `poseidon_bls12381_opt::Poseidon` instead"
)]
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
    pub fn ark(&self, state: &mut Vec<Fr>, c: &[Fr], it: usize) {
        for i in 0..state.len() {
            state[i].add_assign(&c[it + i]);
        }
    }

    #[inline(always)]
    fn pow5(x: &mut Fr) {
        let aux = *x;
        x.square();
        x.square();
        x.mul_assign(&aux);
    }

    pub fn sbox(&self, n_rounds_f: usize, n_rounds_p: usize, state: &mut Vec<Fr>, i: usize) {
        if i < n_rounds_f / 2 || i >= n_rounds_f / 2 + n_rounds_p {
            for x in state {
                Self::pow5(x);
            }
        } else {
            Self::pow5(&mut state[0])
        }
    }

    pub fn mix(&self, state: &[Fr], m: &[Vec<Fr>]) -> Vec<Fr> {
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

    /// Hash function
    /// init_state would be Fr::zero() initially
    pub fn hash(&self, inp: &Vec<Fr>, init_state: &Fr) -> Result<Fr, String> {
        let result = self.hash_inner(inp, init_state, 1)?;
        Ok(result[0])
    }

    pub fn hash_ex(&self, inp: &[Fr], init_state: &Fr, out: usize) -> Result<Vec<Fr>, String> {
        self.hash_inner(inp, init_state, out)
    }

    fn hash_inner(&self, inp: &[Fr], init_state: &Fr, out: usize) -> Result<Vec<Fr>, String> {
        if inp.is_empty() || inp.len() > POSEIDON_BLS12381_CONSTANTS.n_rounds_p.len() {
            return Err(format!(
                "Wrong inputs length {} > {}",
                inp.len(),
                POSEIDON_BLS12381_CONSTANTS.n_rounds_p.len()
            ));
        }

        let t = inp.len() + 1;
        let n_rounds_f = POSEIDON_BLS12381_CONSTANTS.n_rounds_f;
        let n_rounds_p = POSEIDON_BLS12381_CONSTANTS.n_rounds_p[t - 2];

        let mut state = vec![*init_state; t];
        state[1..].clone_from_slice(inp);

        for i in 0..(n_rounds_f + n_rounds_p) {
            self.ark(&mut state, &POSEIDON_BLS12381_CONSTANTS.c[t - 2], i * t);
            self.sbox(n_rounds_f, n_rounds_p, &mut state, i);
            state = self.mix(&state, &POSEIDON_BLS12381_CONSTANTS.m[t - 2]);
        }
        Ok(state[0..out].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::field_bls12381::Fr;
    use crate::poseidon_bls12381::*;

    #[test]
    fn test_load_constants() {
        let cons = load_constants();
        assert_eq!(
            cons.c[0][0].to_string(),
            "Fr(0x6267f5556c88257324c1c8b00d5871b2eba13cc39d72aa10dde6b69bc44c41c7)"
        );
        assert_eq!(
            cons.c[cons.c.len() - 1][0].to_string(),
            "Fr(0x13100d2b1511c87a14eb5fd8412a134abb770736d90f135e699360b9ba852335)"
        );
        assert_eq!(
            cons.m[0][0][0].to_string(),
            "Fr(0x1e6d0cd936714f2124fc4c78321266174fe2855e689c6511a36ecadc3cccc268)"
        );
        assert_eq!(
            cons.m[cons.m.len() - 1][0][0].to_string(),
            "Fr(0x2afb03f1f7a6085f9bf017982ed578fe6185c0355d4affd686b77f84f582f3c3)"
        );
    }

    #[test]
    fn test_poseidon_hash() {
        let poseidon = Poseidon::new();

        let b0: Fr = Fr::from_str("0").unwrap();
        let b1: Fr = Fr::from_str("1").unwrap();
        let b2: Fr = Fr::from_str("2").unwrap();
        let b3: Fr = Fr::from_str("3").unwrap();
        let b4: Fr = Fr::from_str("4").unwrap();

        let is = Fr::zero();
        let h = poseidon.hash(&vec![b1], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x49a66f6b01dbc6440d1a5f920e027b94429916f2c821a920cf6203ad3de56cea)"
        );

        let h = poseidon.hash(&vec![b1, b2], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x28ce19420fc246a05553ad1e8c98f5c9d67166be2c18e9e4cb4b4e317dd2a78a)"
        );

        let h = poseidon.hash(&vec![b1, b2, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x38a4dfeeb62c8ddc28f907fff9658ad10495c587433646531f57d7741c372226)"
        );

        let h = poseidon.hash(&vec![b1, b2, b0, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x4a300ef0358077f7277ded221e20d5967013c62a653bee2db65e162b6143321c)"
        );

        let h = poseidon.hash(&vec![b3, b4, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x631d5456b4cf350c218dfb3c8de41ae9d05ede09f46be5982eed8d3f1d6c7c2a)"
        );

        let h = poseidon.hash(&vec![b3, b4, b0, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x4946ee12005b4c8f9fc919b4768499cfeccb4d35168c1db4509eae7ea5055483)"
        );

        let h = poseidon.hash(&vec![b1, b2, b3, b4], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x2a918b9c9f9bd7bb509331c81e297b5707f6fc7393dcee1b13901a0b22202e18)"
        );
    }

    #[test]
    fn test_batch_hash() {
        let poseidon = Poseidon::new();

        let inputs: Vec<_> = (0..16).collect::<Vec<u64>>();
        let inp: Vec<Fr> = inputs
            .iter()
            .map(|e| Fr::from_str(&e.to_string()).unwrap())
            .collect();

        let is = Fr::zero();
        let h = poseidon.hash(&inp, &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x193396627b6574b4dd5285df3191bcc692f69a9fd7d4d1d7fe98063b2e7cd3a8)",
        );
    }
}
