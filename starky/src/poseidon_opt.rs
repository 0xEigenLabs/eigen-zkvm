#![allow(non_snake_case)]

use crate::constant::POSEIDON_CONSTANTS_OPT;
use crate::poseidon_constants_opt as constants;
use anyhow::bail;
use anyhow::Result;
use fields::field_gl::Fr as FGL;
use fields::Field;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct Constants {
    pub c: Vec<FGL>,
    pub m: Vec<Vec<FGL>>,
    pub p: Vec<Vec<FGL>>,
    pub s: Vec<FGL>,
    pub n_rounds_f: usize,
    pub n_rounds_p: usize,
}

pub fn load_constants() -> Constants {
    let (c_str, m_str, p_str, s_str) = constants::constants();
    let mut c: Vec<FGL> = Vec::new();
    for v1 in c_str {
        c.push(FGL::from(v1));
    }
    let mut m: Vec<Vec<FGL>> = Vec::new();
    for v1 in m_str {
        let mut mi: Vec<FGL> = Vec::new();
        for v2 in v1 {
            mi.push(FGL::from(v2));
        }
        m.push(mi);
    }

    let mut p: Vec<Vec<FGL>> = Vec::new();
    for v1 in p_str {
        let mut mi: Vec<FGL> = Vec::new();
        for v2 in v1 {
            mi.push(FGL::from(v2));
        }
        p.push(mi);
    }

    let mut s: Vec<FGL> = Vec::new();
    for v1 in s_str {
        s.push(FGL::from(v1));
    }

    Constants { c, m, p, s, n_rounds_f: 8, n_rounds_p: 22 }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
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

    #[inline(always)]
    fn pow7(x: &mut FGL) {
        let aux = *x;
        x.square();
        x.mul_assign(&aux);
        x.square();
        x.mul_assign(&aux);
    }

    pub fn hash(&self, inp: &[FGL], init_state: &[FGL], out: usize) -> Result<Vec<FGL>> {
        self.hash_inner(inp, init_state, out)
    }

    fn hash_inner(&self, inp: &[FGL], init_state: &[FGL], out: usize) -> Result<Vec<FGL>> {
        if inp.len() != 8 {
            bail!(format!("Wrong inputs length {} != 8", inp.len(),));
        }

        let t = 12;
        let n_rounds_f = POSEIDON_CONSTANTS_OPT.n_rounds_f;
        let n_rounds_p = POSEIDON_CONSTANTS_OPT.n_rounds_p;
        let C = &POSEIDON_CONSTANTS_OPT.c;
        let S = &POSEIDON_CONSTANTS_OPT.s;
        let M = &POSEIDON_CONSTANTS_OPT.m;
        let P = &POSEIDON_CONSTANTS_OPT.p;

        let mut state = vec![FGL::ZERO; t];
        if init_state.len() != 4 {
            bail!(format!("Capacity inputs length {} != 4", init_state.len(),));
        }

        state[0..8].clone_from_slice(inp);
        state[8..].clone_from_slice(init_state);

        state.iter_mut().enumerate().for_each(|(i, a)| a.add_assign(&C[i]));

        let mut tmp_state = vec![FGL::ZERO; t];
        for r in 0..(n_rounds_f / 2 - 1) {
            state.iter_mut().for_each(Self::pow7);
            state.iter_mut().enumerate().for_each(|(i, a)| {
                a.add_assign(&C[(r + 1) * t + i]);
            });

            let sz = state.len();
            tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
                let mut acc = FGL::ZERO;
                for j in 0..sz {
                    let mut tmp = M[j][i];
                    tmp.mul_assign(&state[j]);
                    acc.add_assign(&tmp);
                }
                *out = acc;
            });
            state.iter_mut().zip(tmp_state.iter()).for_each(|(out, inp)| {
                *out = *inp;
            });
        }

        state.iter_mut().for_each(Self::pow7);
        state.iter_mut().enumerate().for_each(|(i, a)| {
            a.add_assign(&C[(n_rounds_f / 2 - 1 + 1) * t + i]);
        }); //opt

        let sz = state.len();
        tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
            let mut acc = FGL::ZERO;
            for j in 0..sz {
                let mut tmp = P[j][i];
                tmp.mul_assign(&state[j]);
                acc.add_assign(&tmp);
            }
            *out = acc;
        });
        state.iter_mut().zip(tmp_state.iter()).for_each(|(out, inp)| {
            *out = *inp;
        });

        for r in 0..n_rounds_p {
            Self::pow7(&mut state[0]);
            state[0].add_assign(&C[(n_rounds_f / 2 + 1) * t + r]);

            let sz = state.len();
            let mut s0 = FGL::ZERO;
            for j in 0..sz {
                let mut tmp = S[(t * 2 - 1) * r + j];
                tmp.mul_assign(&state[j]);
                s0.add_assign(&tmp);
            }

            for k in 1..t {
                let mut tmp = S[(t * 2 - 1) * r + t + k - 1];
                tmp.mul_assign(&state[0]);
                state[k].add_assign(&tmp);
            }

            state[0] = s0;
        }

        for r in 0..(n_rounds_f / 2 - 1) {
            state.iter_mut().for_each(Self::pow7);
            state.iter_mut().enumerate().for_each(|(i, a)| {
                a.add_assign(&C[(n_rounds_f / 2 + 1) * t + n_rounds_p + r * t + i]);
            });

            let sz = state.len();
            tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
                let mut acc = FGL::ZERO;
                for j in 0..sz {
                    let mut tmp = M[j][i];
                    tmp.mul_assign(&state[j]);
                    acc.add_assign(&tmp);
                }
                *out = acc;
            });
            state.iter_mut().zip(tmp_state.iter()).for_each(|(out, inp)| {
                *out = *inp;
            });
        }

        state.iter_mut().for_each(Self::pow7);
        let sz = state.len();
        tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
            let mut acc = FGL::ZERO;
            for j in 0..sz {
                let mut tmp = M[j][i];
                tmp.mul_assign(&state[j]);
                acc.add_assign(&tmp);
            }
            *out = acc;
        });
        state = tmp_state;

        Ok(state[0..out].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::poseidon_opt::*;
    use fields::field_gl::Fr as FGL;
    use rand::Rand;

    #[test]
    fn test_pow7() {
        let mut rng = rand::thread_rng();
        let mut x = FGL::rand(&mut rng);
        let x7 = x * x * x * x * x * x * x;
        Poseidon::pow7(&mut x);
        assert_eq!(x, x7);
    }

    #[test]
    fn test_poseidon_opt_hash_all_0() {
        let poseidon = Poseidon::new();
        let input = vec![FGL::ZERO; 8];
        let state = vec![FGL::ZERO; 4];
        let res = poseidon.hash(&input, &state, 4).unwrap();
        let expected = vec![
            FGL::from(0x3c18a9786cb0b359u64),
            FGL::from(0xc4055e3364a246c3u64),
            FGL::from(0x7953db0ab48808f4u64),
            FGL::from(0xc71603f33a1144cau64),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_poseidon_opt_hash_1_11() {
        let poseidon = Poseidon::new();
        let input = (0u64..8).map(FGL::from).collect::<Vec<FGL>>();
        let state = (8u64..12).map(FGL::from).collect::<Vec<FGL>>();
        let res = poseidon.hash(&input, &state, 4).unwrap();
        let expected = vec![
            FGL::from(0xd64e1e3efc5b8e9eu64),
            FGL::from(0x53666633020aaa47u64),
            FGL::from(0xd40285597c6a8825u64),
            FGL::from(0x613a4f81e81231d2u64),
        ];
        assert_eq!(res, expected);
    }

    #[test]
    fn test_poseidon_opt_hash_all_neg_1() {
        let poseidon = Poseidon::new();
        let init = FGL::ZERO - FGL::ONE;
        let input = vec![init; 8];
        let state = vec![init; 4];
        let res = poseidon.hash(&input, &state, 4).unwrap();
        let expected = vec![
            FGL::from(0xbe0085cfc57a8357u64),
            FGL::from(0xd95af71847d05c09u64),
            FGL::from(0xcf55a13d33c1c953u64),
            FGL::from(0x95803a74f4530e82u64),
        ];
        assert_eq!(res, expected);
    }
}
