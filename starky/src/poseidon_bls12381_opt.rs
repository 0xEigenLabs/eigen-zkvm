#![allow(non_snake_case)]
use crate::constant::POSEIDON_BLS12381_CONSTANTS_OPT;
use crate::field_bls12381::Fr;
use crate::poseidon_bls12381::Constants;
use crate::poseidon_bls12381_constants_opt as constants;
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
    // The n_rounds_p values are chosen based on the recommendations from the Neptune project.
    // For more details, please refer to: https://github.com/lurk-lab/neptune/blob/main/src/round_numbers.rs
    Constants {
        c,
        m,
        p,
        s,
        n_rounds_f: 8,
        n_rounds_p: vec![55, 55, 56, 56, 56, 56, 57, 57, 57, 57, 57, 57, 57, 57, 59, 59],
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
        let result = self.hash_inner(inp, init_state, 2)?;
        // Return the second element of the result based on Neptune project's specifications.
        // -------------------------------------------------------
        // In accordance with Neptune project guidelines, the second element of the result vector
        // is chosen as the final hash output.
        // For more details, refer to https://github.com/lurk-lab/neptune/blob/main/src/poseidon_alt.rs.
        Ok(result[1])
    }

    pub fn hash_ex(&self, inp: &[Fr], init_state: &Fr, out: usize) -> Result<Vec<Fr>> {
        self.hash_inner(inp, init_state, out)
    }

    fn hash_inner(&self, inp: &[Fr], init_state: &Fr, out: usize) -> Result<Vec<Fr>> {
        if inp.is_empty() || inp.len() > POSEIDON_BLS12381_CONSTANTS_OPT.n_rounds_p.len() {
            bail!(format!(
                "Wrong inputs length {} > {}",
                inp.len(),
                POSEIDON_BLS12381_CONSTANTS_OPT.n_rounds_p.len()
            ));
        }

        let t = inp.len() + 1;
        let n_rounds_f = POSEIDON_BLS12381_CONSTANTS_OPT.n_rounds_f;
        let n_rounds_p = POSEIDON_BLS12381_CONSTANTS_OPT.n_rounds_p[t - 2];
        let C = &POSEIDON_BLS12381_CONSTANTS_OPT.c[t - 2];
        let M = &POSEIDON_BLS12381_CONSTANTS_OPT.m[t - 2];
        let P = &POSEIDON_BLS12381_CONSTANTS_OPT.p[t - 2];
        let S = &POSEIDON_BLS12381_CONSTANTS_OPT.s[t - 2];
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
    use crate::poseidon_bls12381_opt::*;
    use ff::PrimeField;

    #[test]
    fn test_poseidon_opt_hash() {
        let poseidon = Poseidon::new();

        let b0: Fr = Fr::from_str("0").unwrap();
        let b1: Fr = Fr::from_str("1").unwrap();
        let b2: Fr = Fr::from_str("2").unwrap();
        let b3: Fr = Fr::from_str("3").unwrap();
        let b4: Fr = Fr::from_str("4").unwrap();

        let is = Fr::zero();
        let h = poseidon.hash(&[b1], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x164efff6c8a32ef98836c868f8c8dedcbe3068d16ba6098f282a6d185edb551f)" //10090463338479474364654416042385169859560025017303585988626920959727361545503
        );

        let h = poseidon.hash(&[b1, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x59220c0fc5748e83c141c7bb8dae0a2bd5bbb227c778ede87296ba07960ec3d8)" //40315999570263005229566068098191840653718756303362127561954793579940120806360
        );

        let h = poseidon.hash(&[b1, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x73584296b068384db6028b55d995108518d4483ab177197274effe979b91526e)" //52171919706604857662228147548523676303297329614804576829062159794914391577198
        );

        let h = poseidon.hash(&[b1, b2, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x385acd94e53a8c6f981809c2201582beceaec12250200f1e75ba93e6cf5ec736)" //25489954628706771422434337159093356230875147553184381182493646336226215511862
        );

        let h = poseidon.hash(&[b1, b2, b0, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x023dd8aecc0967c0588754eebd39af39bdae2bbf4195fee1208613c909aaa29b)" //1013898857847217674473086247177895055941699630695530588118970595082884522651
        );

        let h = poseidon.hash(&[b3, b4, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x19c96d726da9e3df4e5d0da19f324f7bf376dc7bf97efbf37082473f7fa24af8)" //11663712849936763722275869035629160480859126086041635677673535448082509089528
        );

        let h = poseidon.hash(&[b3, b4, b0, b0, b0, b0], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x0cb7b1761b9abe661847a10701c6eae7c631ff580c5b7f3ac2f8be1088d22bba)" //5752311989137819405955540762621381412568871047030860382530977484434251590586
        );

        let h = poseidon.hash(&[b1, b2, b3, b4], &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x6f5f297b0ab0d1e7400501b9bdd4c3be2fe676b6a05deb845143b87355167a8d)" //50374862952696036512232585533148559412665642735378685892656796916864806976141
        );
    }

    #[test]
    fn test_batch_opt_hash() {
        let poseidon = Poseidon::new();

        let inputs: Vec<_> = (0..16).collect::<Vec<u64>>();
        let inp: Vec<Fr> = inputs.iter().map(|e| Fr::from_str(&e.to_string()).unwrap()).collect();

        let is = Fr::zero();
        let h = poseidon.hash(&inp, &is).unwrap();
        assert_eq!(
            h.to_string(),
            "Fr(0x12d374bbdb8d3c1c0230b20b8fe1572f1e652a616d16e834718a982574106405)", //8515241672374781049985699179100419324899359624275223371256009421843839607813
        );
    }
}
