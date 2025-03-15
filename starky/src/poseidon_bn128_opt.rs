#![allow(non_snake_case)]
use crate::constant::POSEIDON_BN128_CONSTANTS_OPT;
use crate::field_bn128::Fr;
use crate::poseidon_bn128::Constants;
use crate::poseidon_bn128_constants_opt as constants;
use anyhow::bail;
use anyhow::Result;
use ff::{from_hex, Field};
use serde::{Deserialize, Serialize};

pub fn load_constants() -> Constants {
    let (c_str, m_str, p_str, s_str) = constants::constants();
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

    let mut p: Vec<Vec<Vec<Fr>>> = Vec::new();
    for v1 in p_str {
        let mut mi: Vec<Vec<Fr>> = Vec::new();
        for v2 in v1 {
            let mut mij: Vec<Fr> = Vec::new();
            for s in v2 {
                let b: Fr = from_hex(s).unwrap();
                mij.push(b);
            }
            mi.push(mij);
        }
        p.push(mi);
    }

    let mut s: Vec<Vec<Fr>> = Vec::new();
    for v1 in s_str {
        let mut cci: Vec<Fr> = Vec::new();
        for v2 in v1 {
            let b: Fr = from_hex(v2).unwrap();
            cci.push(b);
        }
        s.push(cci);
    }

    Constants {
        c,
        m,
        p,
        s,
        n_rounds_f: 8,
        n_rounds_p: vec![56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68],
    }
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
    fn pow5(x: &mut Fr) {
        let aux = *x;
        x.square();
        x.square();
        x.mul_assign(&aux);
    }

    /// Hash function
    /// init_state would be Fr::zero() initially
    pub fn hash(&self, inp: &[Fr], init_state: &Fr) -> Result<Fr> {
        let result = self.hash_inner(inp, init_state, 1)?;
        Ok(result[0])
    }

    pub fn hash_ex(&self, inp: &[Fr], init_state: &Fr, out: usize) -> Result<Vec<Fr>> {
        self.hash_inner(inp, init_state, out)
    }

    fn hash_inner(&self, inp: &[Fr], init_state: &Fr, out: usize) -> Result<Vec<Fr>> {
        if inp.is_empty() || inp.len() > POSEIDON_BN128_CONSTANTS_OPT.n_rounds_p.len() {
            bail!(format!(
                "Wrong inputs length {} > {}",
                inp.len(),
                POSEIDON_BN128_CONSTANTS_OPT.n_rounds_p.len()
            ));
        }

        let t = inp.len() + 1;
        let n_rounds_f = POSEIDON_BN128_CONSTANTS_OPT.n_rounds_f;
        let n_rounds_p = POSEIDON_BN128_CONSTANTS_OPT.n_rounds_p[t - 2];
        let C = &POSEIDON_BN128_CONSTANTS_OPT.c[t - 2];
        let S = &POSEIDON_BN128_CONSTANTS_OPT.s[t - 2];
        let M = &POSEIDON_BN128_CONSTANTS_OPT.m[t - 2];
        let P = &POSEIDON_BN128_CONSTANTS_OPT.p[t - 2];
        let mut tmp_state = vec![Fr::zero(); t];

        let mut state = vec![*init_state; t];
        state[1..].clone_from_slice(inp);
        state.iter_mut().enumerate().for_each(|(i, a)| a.add_assign(&C[i]));

        for r in 0..(n_rounds_f / 2 - 1) {
            state.iter_mut().for_each(Self::pow5);
            state.iter_mut().enumerate().for_each(|(i, a)| {
                a.add_assign(&C[(r + 1) * t + i]);
            });

            //state = state.map((_, i) =>
            //    state.reduce((acc, a, j) => F.add(acc, F.mul(M[j][i], a)), F.zero)
            //);
            let sz = state.len();
            tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
                let mut acc = Fr::zero();
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

        state.iter_mut().for_each(Self::pow5);
        state.iter_mut().enumerate().for_each(|(i, a)| {
            a.add_assign(&C[(n_rounds_f / 2 - 1 + 1) * t + i]);
        }); //opt

        let sz = state.len();
        tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
            let mut acc = Fr::zero();
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
            Self::pow5(&mut state[0]);
            state[0].add_assign(&C[(n_rounds_f / 2 + 1) * t + r]);

            let sz = state.len();
            let mut s0 = Fr::zero();
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
            state.iter_mut().for_each(Self::pow5);
            state.iter_mut().enumerate().for_each(|(i, a)| {
                a.add_assign(&C[(n_rounds_f / 2 + 1) * t + n_rounds_p + r * t + i]);
            });

            let sz = state.len();
            tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
                let mut acc = Fr::zero();
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

        state.iter_mut().for_each(Self::pow5);
        let sz = state.len();
        tmp_state.iter_mut().enumerate().for_each(|(i, out)| {
            let mut acc = Fr::zero();
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
    use crate::poseidon_bn128_opt::*;
    use ff::PrimeField;

    #[test]
    fn test_poseidon_bn128_opt_hash() {
        let poseidon = Poseidon::new();
        let b0: Fr = Fr::from_str("0").unwrap();
        let b1: Fr = Fr::from_str("1").unwrap();
        let b2: Fr = Fr::from_str("2").unwrap();
        let b3: Fr = Fr::from_str("3").unwrap();
        let b4: Fr = Fr::from_str("4").unwrap();
        let b5: Fr = Fr::from_str("5").unwrap();
        let b6: Fr = Fr::from_str("6").unwrap();

        let is = Fr::zero();
        let h = poseidon.hash(&[b1], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x29176100eaa962bdc1fe6c654d6a3c130e96a4d1168b33848b897dc502820133)" // "18586133768512220936620570745912940619677854269274689475585506675881198879027"
        );

        let h = poseidon.hash(&[b1, b2], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x115cc0f5e7d690413df64c6b9662e9cf2a3617f2743245519e19607a4417189a)" // "7853200120776062878684798364095072458815029376092732009249414926327459813530"
        );

        let h = poseidon.hash(&[b1, b2, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x024058dd1e168f34bac462b6fffe58fd69982807e9884c1c6148182319cee427)" // "1018317224307729531995786483840663576608797660851238720571059489595066344487"
        );

        let h = poseidon.hash(&[b1, b2, b0, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x21e82f465e00a15965e97a44fe3c30f3bf5279d8bf37d4e65765b6c2550f42a1)" // "15336558801450556532856248569924170992202208561737609669134139141992924267169"
        );

        let h = poseidon.hash(&[b3, b4, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x0cd93f1bab9e8c9166ef00f2a1b0e1d66d6a4145e596abe0526247747cc71214)" // "5811595552068139067952687508729883632420015185677766880877743348592482390548"
        );

        let h = poseidon.hash(&[b3, b4, b0, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x1b1caddfc5ea47e09bb445a7447eb9694b8d1b75a97fff58e884398c6b22825a)" // "12263118664590987767234828103155242843640892839966517009184493198782366909018"
        );

        let h = poseidon.hash(&[b1, b2, b3, b4, b5, b6], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x2d1a03850084442813c8ebf094dea47538490a68b05f2239134a4cca2f6302e1)" // "20400040500897583745843009878988256314335038853985262692600694741116813247201"
        );
    }

    #[test]
    fn test_batch_hash_opt() {
        let poseidon = Poseidon::new();

        let inputs: Vec<_> = (0..16).collect::<Vec<u64>>();
        let inp: Vec<Fr> = inputs.iter().map(|e| Fr::from_str(&e.to_string()).unwrap()).collect();

        let is = Fr::zero();
        let h = poseidon.hash(&inp, &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x1b733f2ff41971b23819a16bc8c16bbe13d98173358429fcc12f6f0826407a56)",
        );
    }
}
