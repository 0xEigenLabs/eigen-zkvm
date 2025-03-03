#![allow(non_snake_case)]
use algebraic::circom_circuit::{Constraint, R1CS};
use array_tool::vec::Shift;
use fields::field_gl::Fr as FGL;
use fields::field_gl::{Fr, GL};
use std::collections::BTreeMap;
use std::ops::Neg;

#[derive(Debug)]
pub struct PlonkGate(pub usize, pub usize, pub usize, pub FGL, pub FGL, pub FGL, pub FGL, pub FGL);

impl std::fmt::Display for PlonkGate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}, {}, {}, {}, {}, {}, {}, {})",
            self.0,
            self.1,
            self.2,
            self.3.as_int(),
            self.4.as_int(),
            self.5.as_int(),
            self.6.as_int(),
            self.7.as_int()
        )
    }
}

impl PlonkGate {
    pub fn str_key(&self) -> String {
        format!(
            "{:x},{:x},{:x},{:x},{:x}",
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
        write!(f, "({}, {}, {}, {})", self.0, self.1, self.2.as_int(), self.3.as_int())
    }
}

pub fn r1cs2plonk(r1cs: &R1CS<GL>) -> (Vec<PlonkGate>, Vec<PlonkAdd>) {
    let mut plonk_n_var = r1cs.num_variables;
    let mut plonk_constraints: Vec<PlonkGate> = vec![];
    let mut plonk_additions: Vec<PlonkAdd> = vec![];

    let normalize = |lc: &mut BTreeMap<usize, FGL>| {
        lc.retain(|_, v| *v != FGL::ZERO);
    };

    let join =
        |lc1: &BTreeMap<usize, FGL>, k: &FGL, lc2: &BTreeMap<usize, FGL>| -> BTreeMap<usize, FGL> {
            let mut res: BTreeMap<usize, FGL> = BTreeMap::new();
            for (key, val) in lc1.iter() {
                if !res.contains_key(key) {
                    res.insert(*key, *k * (*val));
                } else {
                    let tmp = res[key];
                    res.insert(*key, *k * (*val) + tmp);
                }
            }

            for (key, val) in lc2.iter() {
                if !res.contains_key(key) {
                    res.insert(*key, *val);
                } else {
                    let tmp = res[key];
                    res.insert(*key, *val + tmp);
                }
            }
            normalize(&mut res);
            res
        };

    let reduce_coefs = |lc: &BTreeMap<usize, FGL>,
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
            let c1 = cs.shift().unwrap();
            let c2 = cs.shift().unwrap();

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

    let add_constraint_mul = |la: &BTreeMap<usize, FGL>,
                              lb: &BTreeMap<usize, FGL>,
                              lc: &BTreeMap<usize, FGL>,
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

    let add_constraint_sum = |lc: &BTreeMap<usize, FGL>,
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

    let to_be_map = |lc: &Vec<(usize, Fr)>| -> BTreeMap<usize, FGL> {
        let mut res: BTreeMap<usize, FGL> = BTreeMap::new();
        for c in lc.iter() {
            assert!(!res.contains_key(&c.0));
            res.insert(c.0, c.1);
        }
        res
    };

    let get_lc_type = |lc: &mut BTreeMap<usize, FGL>| -> String {
        let mut k = FGL::ZERO;
        let mut n = 0;
        let keys: Vec<usize> = lc.keys().copied().collect();
        for key in keys.iter() {
            let val = lc[key];
            if val == FGL::ZERO {
                lc.remove(key).unwrap();
            } else if *key == 0 {
                k = k + val;
            } else {
                n += 1;
            }
        }
        if n > 0 {
            return n.to_string();
        }
        if k != FGL::ZERO {
            return String::from("k");
        }
        String::from("0")
    };

    let process =
        |c: &Constraint<GL>, pc: &mut Vec<PlonkGate>, pa: &mut Vec<PlonkAdd>, n_var: &mut usize| {
            let mut lc_a = to_be_map(&c.0);
            let mut lc_b = to_be_map(&c.1);
            let mut lc_c = to_be_map(&c.2);
            let lca = get_lc_type(&mut lc_a);
            let lcb = get_lc_type(&mut lc_b);
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
        };

    for (i, c) in r1cs.constraints.iter().enumerate() {
        if i % 100000 == 0 {
            log::trace!("processing constraints: {}/{}", i, r1cs.constraints.len());
        }
        process(c, &mut plonk_constraints, &mut plonk_additions, &mut plonk_n_var);
    }
    (plonk_constraints, plonk_additions)
}

#[cfg(test)]
mod test {
    use super::*;
    use algebraic::reader::load_r1cs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    /// The js dump code as below:
    /// ```js
    /// fs.writeFileSync("plonk_constrains_js.json", JSON.stringify(plonkConstraints, (key, value) =>
    ///     typeof value === 'bigint' ? value.toString() :value
    /// ));
    /// fs.writeFileSync("plonk_additions_js.json", JSON.stringify(plonkAdditions, (key, value) =>
    ///     typeof value === 'bigint' ? value.toString() :value
    /// ));
    /// ```
    #[test]
    #[ignore]
    fn test_r1cs2plonk() {
        let CIRCUIT = "fib.verifier";

        let r1cs_file = format!("/tmp/{CIRCUIT}.r1cs");
        let r1cs = load_r1cs::<GL>(&r1cs_file);

        let (plonk_constrains, plonk_additions) = r1cs2plonk(&r1cs);

        // test the r1cs2plonk data by dump its data.
        let mut file = File::create(Path::new("plonk_constrains_rs.json")).unwrap();
        let input = plonk_constrains.iter().map(|pa| pa.to_string()).collect::<Vec<String>>();
        let input = serde_json::to_string(&input).unwrap();
        write!(file, "{}", input).unwrap();

        let mut file = File::create(Path::new("/tmp/plonk_additions_rs.json")).unwrap();
        let input = plonk_additions.iter().map(|pa| pa.to_string()).collect::<Vec<String>>();
        let input = serde_json::to_string(&input).unwrap();
        write!(file, "{}", input).unwrap();
    }
}
