#![allow(non_snake_case)]
use plonky::circom_circuit::Constraint;
use plonky::circom_circuit::R1CS;
use plonky::field_gl::Fr as FGL;
use plonky::field_gl::{Fr, GL};
use std::collections::HashMap;
use std::ops::Neg;

#[derive(Debug)]
pub struct PlonkGate(
    pub usize,
    pub usize,
    pub usize,
    pub FGL,
    pub FGL,
    pub FGL,
    pub FGL,
    pub FGL,
);

impl std::fmt::Display for PlonkGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}, {}, {}, {}, {}, {})",
            self.0, self.1, self.2, self.3, self.4, self.5, self.6, self.7
        )
    }
}

impl PlonkGate {
    pub fn str_key(&self) -> String {
        format!(
            "{:X},{:X},{:X},{:X},{:X}",
            self.3.as_int(),
            self.4.as_int(),
            self.5.as_int(),
            self.6.as_int(),
            self.7.as_int()
        )
    }
}

#[derive(Debug)]
pub struct PlonkAdd(pub usize, pub usize, pub FGL, pub FGL);
impl std::fmt::Display for PlonkAdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.0, self.1, self.2, self.3)
    }
}

pub fn r1cs2plonk(r1cs: &R1CS<GL>) -> (Vec<PlonkGate>, Vec<PlonkAdd>) {
    let mut plonk_n_var = r1cs.num_variables;
    let mut plonk_constraints: Vec<PlonkGate> = vec![];
    let mut plonk_additions: Vec<PlonkAdd> = vec![];

    let normalize = |lc: &mut HashMap<usize, FGL>| {
        lc.retain(|_, v| *v != FGL::ZERO);
    };

    let join =
        |lc1: &HashMap<usize, FGL>, k: &FGL, lc2: &HashMap<usize, FGL>| -> HashMap<usize, FGL> {
            let mut res: HashMap<usize, FGL> = HashMap::new();
            for (key, val) in lc1.iter() {
                if res.get(&key).is_none() {
                    res.insert(*key, *k * (*val));
                } else {
                    let tmp = res[key];
                    res.insert(*key, *k * (*val) + tmp);
                }
            }

            for (key, val) in lc2.iter() {
                if res.get(&key).is_none() {
                    res.insert(*key, *val);
                } else {
                    let tmp = res[key];
                    res.insert(*key, *val + tmp);
                }
            }
            normalize(&mut res);
            res
        };

    let reduce_coefs = |lc: &HashMap<usize, FGL>,
                        max_c: usize,
                        pc: &mut Vec<PlonkGate>,
                        pa: &mut Vec<PlonkAdd>,
                        n_var: &mut usize|
     -> (FGL, Vec<usize>, Vec<FGL>) {
        // (k, s, coefs)
        let mut res: (FGL, Vec<usize>, Vec<FGL>) = (FGL::ZERO, vec![], vec![]);
        let mut cs: Vec<(usize, FGL)> = vec![];
        for (key, val) in lc.iter() {
            if *key == 0 {
                res.0 = res.0 + *val;
            } else if *val != FGL::ZERO {
                cs.push((*key, *val));
            }
        }

        while cs.len() > max_c {
            let c1 = cs[0];
            let c2 = cs[1];

            let sl = c1.0;
            let sr = c2.0;
            let so = *n_var;
            *n_var += 1;

            let qm = FGL::ZERO;
            let ql = c1.1.neg();
            let qr = c2.1.neg();
            let qo = FGL::ONE;
            let qc = FGL::ZERO;

            pc.push(PlonkGate(sl, sr, so, qm, ql, qr, qo, qc));
            pa.push(PlonkAdd(sl, sr, c1.1, c2.1));
            cs.remove(0);
            cs.remove(0);
            cs.push((so, FGL::ONE));
        }
        for c in cs.iter() {
            res.1.push(c.0);
            res.2.push(c.1);
        }
        while res.2.len() < max_c {
            res.1.push(0);
            res.2.push(FGL::ZERO);
        }
        res
    };

    let add_constraint_mul = |la: &HashMap<usize, FGL>,
                              lb: &HashMap<usize, FGL>,
                              lc: &HashMap<usize, FGL>,
                              pc: &mut Vec<PlonkGate>,
                              pa: &mut Vec<PlonkAdd>,
                              n_var: &mut usize| {
        let A = reduce_coefs(la, 1, pc, pa, n_var);
        let B = reduce_coefs(lb, 1, pc, pa, n_var);
        let C = reduce_coefs(lc, 1, pc, pa, n_var);

        let sl = A.1[0];
        let sr = B.1[0];
        let so = C.1[0];
        let qm = A.2[0] * B.2[0];
        let ql = A.2[0] * B.0;
        let qr = A.0 * B.2[0];
        let qo = C.2[0].neg();
        let qc = A.0 * B.0 - C.0;
        pc.push(PlonkGate(sl, sr, so, qm, ql, qr, qo, qc));
    };

    let add_constraint_sum = |lc: &HashMap<usize, FGL>,
                              pc: &mut Vec<PlonkGate>,
                              pa: &mut Vec<PlonkAdd>,
                              n_var: &mut usize| {
        let C = reduce_coefs(lc, 3, pc, pa, n_var);
        let sl = C.1[0];
        let sr = C.1[1];
        let so = C.1[2];
        let qm = FGL::ZERO;
        let ql = C.2[0];
        let qr = C.2[1];
        let qo = C.2[2];
        let qc = C.0;
        pc.push(PlonkGate(sl, sr, so, qm, ql, qr, qo, qc));
    };

    let to_be_map = |lc: &Vec<(usize, Fr)>| -> HashMap<usize, FGL> {
        let mut res: HashMap<usize, FGL> = HashMap::new();
        for c in lc.iter() {
            assert!(res.get(&c.0).is_none());
            res.insert(c.0, FGL::from(c.1 .0 .0[0]));
        }
        res
    };

    let get_lc_type = |lc: &mut HashMap<usize, FGL>| -> String {
        let mut k = FGL::ZERO;
        let mut n = 0;
        for (key, val) in lc.iter() {
            if *val == FGL::ZERO {
                //delete
            } else if *key == 0 {
                k = k + *val;
            } else {
                n += 1;
            }
        }
        lc.retain(|_, v| *v != FGL::ZERO);
        //println!("get_lc_type lc.size {}", lc.len());

        if n > 0 {
            return format!("{}", n);
        }
        if k != FGL::ZERO {
            return format!("k");
        }
        format!("0")
    };

    let process =
        |c: &Constraint<GL>, pc: &mut Vec<PlonkGate>, pa: &mut Vec<PlonkAdd>, n_var: &mut usize| {
            let mut lc_a = to_be_map(&c.0);
            let mut lc_b = to_be_map(&c.1);
            let mut lc_c = to_be_map(&c.2);

            let lca = get_lc_type(&mut lc_a);
            let lcb = get_lc_type(&mut lc_b);
            //println!("process {} {}", lca, lcb);
            if lca.as_str() == "0" || lcb.as_str() == "0" {
                normalize(&mut lc_c);
                add_constraint_sum(&lc_c, pc, pa, n_var);
            } else if lca.as_str() == "k" {
                let lc_cc = join(&lc_b, &lc_a[&0], &lc_c);
                add_constraint_sum(&lc_cc, pc, pa, n_var);
            } else if lcb.as_str() == "k" {
                let lc_cc = join(&lc_a, &lc_b[&0], &lc_c);
                add_constraint_sum(&lc_cc, pc, pa, n_var);
            } else {
                add_constraint_mul(&lc_a, &lc_b, &lc_c, pc, pa, n_var);
            }
            //pc.iter().for_each(|c|println!("{}", c));
        };

    for (i, c) in r1cs.constraints.iter().enumerate() {
        if i % 100000 == 0 {
            println!("processing constraints: {}/{}", i, r1cs.constraints.len());
        }
        process(
            c,
            &mut plonk_constraints,
            &mut plonk_additions,
            &mut plonk_n_var,
        );
    }
    (plonk_constraints, plonk_additions)
}

#[cfg(test)]
pub mod tests {
    use crate::compressor12::compressor12_setup::{plonk_setup_render, Options};
    use crate::r1cs2plonk::r1cs2plonk;
    //use plonky::bellman_ce::bn256::Bn256;
    use plonky::field_gl::GL;
    use plonky::reader::load_r1cs;

    #[test]
    #[ignore]
    fn test_r1cs2plonk() {
        let r1cs = load_r1cs::<GL>("/tmp/circuit.gl.r1cs");
        let (pc, pa) = r1cs2plonk(&r1cs);
        println!("pc {}, pa {}", pc.len(), pa.len());
        let opts = Options { force_bits: 0 };
        let plonksetupinfo = plonk_setup_render(&r1cs, &opts, "/tmp/c12.pil");
    }
}
