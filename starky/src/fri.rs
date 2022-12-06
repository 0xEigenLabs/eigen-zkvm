use crate::constant::{MG, SHIFT, SHIFT_INV};
use crate::digest_bn128::ElementDigest;
use crate::errors::{EigenError::FRIVerifierFailed, Result};
use crate::f3g::F3G;
use crate::fft::FFT;
use crate::helper::log2_any;
use crate::merklehash_bn128::MerkleTree;
use crate::poseidon_bn128::Fr;
use crate::poseidon_bn128::FrRepr;
use crate::transcript_bn128::TranscriptBN128;
use crate::types::{StarkStruct, Step};
use ff::*;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;
use winter_math::StarkField;

pub struct FRI {
    pub in_nbits: usize,
    pub max_deg_nbits: usize,
    pub n_queries: usize,
    pub steps: Vec<Step>,
}

#[derive(Default, Clone)]
pub struct ProofOne {
    pub pol_queries: Vec<Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>>,
    pub root: ElementDigest,
}

impl fmt::Display for ProofOne {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "root {}\n", self.root)?;
        write!(f, "pol_queries size {}\n", self.pol_queries.len())?;
        for (i, pq) in self.pol_queries.iter().enumerate() {
            write!(f, "\t pq {}\n", i)?;
            for (j, qq) in pq.iter().enumerate() {
                write!(f, "\t\tleaf: ")?;
                for qqq in qq.0.iter() {
                    write!(f, "{},", qqq)?;
                }
                write!(f, "\n\t\tnode:")?;
                for qqq in qq.1.iter() {
                    write!(f, "\t\t[\n")?;
                    for t in qqq.iter() {
                        write!(f, "\t\t\t Fr {}\n", crate::helper::fr_to_biguint(t))?;
                    }
                    write!(f, "\t\t]\n")?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct FRIProof {
    pub queries: Vec<ProofOne>,
    pub last: Vec<F3G>,
}

impl FRIProof {
    pub fn new(qs: usize) -> Self {
        FRIProof {
            queries: vec![ProofOne::default(); qs],
            last: Vec::new(),
        }
    }
}

impl FRI {
    pub fn new(stark_struct: &StarkStruct) -> Self {
        Self {
            in_nbits: stark_struct.nBitsExt,
            max_deg_nbits: stark_struct.nBits,
            n_queries: stark_struct.nQueries,
            steps: stark_struct.steps.clone(),
        }
    }

    pub fn prove(
        &mut self,
        transcript: &mut TranscriptBN128,
        pol: &Vec<F3G>,
        mut query_pol: impl FnMut(usize) -> Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>,
    ) -> Result<FRIProof> {
        let mut pol = pol.clone();
        let mut standard_fft = FFT::new();
        let mut pol_bits = log2_any(pol.len()) as usize;
        //println!("fri prove {} {}", pol.len(), 1 << pol_bits);
        assert_eq!(1 << pol_bits, pol.len());
        assert_eq!(pol_bits, self.in_nbits);

        let mut shift_inv = SHIFT_INV.clone();
        let mut shift = SHIFT.clone();
        let mut tree: Vec<MerkleTree> = vec![];

        let mut proof: FRIProof = FRIProof::new(self.steps.len());
        for si in 0..self.steps.len() {
            let reduction_bits = pol_bits - self.steps[si].nBits;
            let pol2N = 1 << (pol_bits - reduction_bits);
            let nX = pol.len() / pol2N;

            let mut pol2_e = vec![F3G::ZERO; pol2N];
            let special_x = transcript.get_field();

            let mut sinv = shift_inv;
            let wi = F3G::inv(MG.0[pol_bits]);
            //println!("nX {} {} {}", nX, pol2N, special_x);
            crate::helper::pretty_print_array(&pol);

            for g in 0..(pol.len() / nX) {
                if si == 0 {
                    pol2_e[g] = pol[g];
                } else {
                    let mut ppar = vec![F3G::ZERO; nX];
                    for i in 0..nX {
                        ppar[i] = pol[i * pol2N + g];
                    }

                    let mut ppar_c = standard_fft.ifft(&ppar);
                    pol_mul_axi(&mut ppar_c, F3G::ONE, &sinv);
                    pol2_e[g] = eval_pol(&ppar_c, &special_x);
                    sinv = sinv * wi;
                }
            }
            //println!("pol2_e 0={}, 1={}", pol2_e[0], pol2_e[1]);

            if si < self.steps.len() - 1 {
                let n_groups = 1 << self.steps[si + 1].nBits;
                let group_size = (1 << self.steps[si].nBits) / n_groups;
                let pol2_etb = getTransposedBuffer(&pol2_e, self.steps[si + 1].nBits);
                tree.push(MerkleTree::merkelize(pol2_etb, 3 * group_size, n_groups)?);
                proof.queries[si + 1].root = tree[si].root();
                let rrr: Fr = proof.queries[si + 1].root.into();
                //println!(
                //    "proof.queries {}={}",
                //    si + 1,
                //    crate::helper::fr_to_biguint(&rrr)
                //);
                transcript.put(&vec![tree[si].root().into()])?;
            } else {
                //println!("last {}", pol2_e.len());
                for e in pol2_e.iter() {
                    let elems = e.as_elements();
                    let v = vec![
                        Fr::from_repr(FrRepr::from(elems[0].as_int())).unwrap(),
                        Fr::from_repr(FrRepr::from(elems[1].as_int())).unwrap(),
                        Fr::from_repr(FrRepr::from(elems[2].as_int())).unwrap(),
                    ];
                    transcript.put(&v)?;
                }
            }

            pol = pol2_e;
            pol_bits -= reduction_bits;

            for j in 0..reduction_bits {
                shift_inv = shift_inv * shift_inv;
                shift = shift * shift;
            }
        }
        let mut last_pol: Vec<F3G> = vec![];
        for p in pol.iter() {
            last_pol.push(*p);
        }

        proof.last = last_pol;

        let mut ys = transcript.get_permutations(self.n_queries, self.steps[0].nBits)?;

        let query_pol_fn = |si: usize, idx: usize| -> Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)> {
            //println!("query_pol_fn: si:{}, idx:{}", si, idx);
            vec![tree[si].get_group_proof(idx).unwrap()]
        };

        for si in 0..self.steps.len() {
            for ys_ in ys.iter() {
                if si == 0 {
                    proof.queries[si].pol_queries.push(query_pol(*ys_));
                } else {
                    proof.queries[si]
                        .pol_queries
                        .push(query_pol_fn(si - 1, *ys_));
                }
            }
            //println!("prove_query_pol: {} {}", si, proof.queries[si]);

            if si < self.steps.len() - 1 {
                for i in 0..ys.len() {
                    ys[i] = ys[i] % (1 << self.steps[si + 1].nBits);
                }
            }
        }
        Ok(proof)
    }

    pub fn verify(
        &self,
        transcript: &mut TranscriptBN128,
        proof: &FRIProof,
        mut check_query: impl FnMut(&Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>, usize) -> Result<Vec<F3G>>,
    ) -> Result<bool> {
        let tree = MerkleTree::new();
        let mut standard_fft = FFT::new();
        assert_eq!(proof.queries.len(), self.steps.len()); // the last +1 is ommited
        let mut special_x: Vec<F3G> = vec![];
        for si in 0..self.steps.len() {
            special_x.push(transcript.get_field());
            if si < self.steps.len() - 1 {
                let n_groups = 1 << self.steps[si + 1].nBits;
                let group_size = (1 << self.steps[si].nBits) / n_groups;
                transcript.put(&vec![proof.queries[si + 1].root.into()])?;
            } else {
                let mut pp: Vec<Fr> = vec![];
                for e in proof.last.iter() {
                    let elems = e.as_elements();
                    pp.push(Fr::from_str(&elems[0].as_int().to_string()).unwrap());
                    pp.push(Fr::from_str(&elems[1].as_int().to_string()).unwrap());
                    pp.push(Fr::from_str(&elems[2].as_int().to_string()).unwrap());
                }
                transcript.put(&pp)?;
            }
        }

        let n_queries = self.n_queries;
        let mut ys = transcript.get_permutations(self.n_queries, self.steps[0].nBits)?;
        let mut pol_bits = self.in_nbits;
        //println!("ys: {:?}, pol_bits {}", ys, self.in_nbits);
        let mut shift = SHIFT.clone();

        let check_query_fn = |si: usize,
                              query: &Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>,
                              idx: usize|
         -> Result<Vec<F3G>> {
            let res =
                tree.verify_group_proof(&proof.queries[si].root, &query[0].1, idx, &query[0].0)?;
            if !res {
                return Err(FRIVerifierFailed);
            }
            Ok(split3(&query[0].0))
        };

        for si in 0..self.steps.len() {
            let proof_item = &proof.queries[si];
            //println!("si: {}, queries: {}", si, proof_item);
            let reduction_bits = pol_bits - self.steps[si].nBits;
            //println!("si {} reduction_bits {}", si, reduction_bits);
            for i in 0..n_queries {
                let pgroup_e: Vec<F3G> = match si {
                    0 => {
                        let pgroup_e = check_query(&proof_item.pol_queries[i], ys[i])?;
                        if pgroup_e.len() == 0 {
                            return Ok(false);
                        }
                        pgroup_e
                    }
                    _ => check_query_fn(si, &proof_item.pol_queries[i], ys[i])?,
                };

                //println!("pgroup_e");
                crate::helper::pretty_print_array(&pgroup_e);

                let pgroup_c = standard_fft.ifft(&pgroup_e);

                //println!("pgroup_c");
                crate::helper::pretty_print_array(&pgroup_c);

                let sinv = F3G::inv(shift * (MG.0[pol_bits].exp(ys[i])));

                //println!("sinv {}, special_x[{}]={}", sinv, si, special_x[si]);

                let ev = eval_pol(&pgroup_c, &(special_x[si] * sinv));
                //println!("ev {}", ev);

                if si < self.steps.len() - 1 {
                    let next_n_groups = 1 << self.steps[si + 1].nBits;
                    let group_idx = ys[i] / next_n_groups;
                    if !ev.eq(&get3(&proof.queries[si + 1].pol_queries[i][0].0, group_idx)) {
                        return Ok(false);
                    }
                } else {
                    if !ev.eq(&proof.last[ys[i]]) {
                        return Ok(false);
                    }
                }
            }

            let pol_bits = self.steps[si].nBits;
            for j in 0..reduction_bits {
                shift = shift * shift;
            }

            if si < self.steps.len() - 1 {
                for i in 0..ys.len() {
                    ys[i] = ys[i] % (1 << self.steps[si + 1].nBits);
                }
            }
        }

        let mut last_pol_e = proof.last.clone();

        let mut maxDeg = 0usize;
        if (pol_bits - (self.in_nbits - self.max_deg_nbits)) < 0 {
            maxDeg = 0;
        } else {
            maxDeg = 1 << (pol_bits - (self.in_nbits - self.max_deg_nbits));
        }

        let lastPol_c = standard_fft.ifft(&last_pol_e);

        for i in (maxDeg + 1)..lastPol_c.len() {
            if !lastPol_c[i].is_zero() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

fn getTransposedBuffer(pol: &Vec<F3G>, trasposeBits: usize) -> Vec<F3G> {
    let n = pol.len();
    let w = 1 << trasposeBits;
    let h = n / w;
    let mut res: Vec<F3G> = vec![F3G::ZERO; n * 3];
    for i in 0..w {
        for j in 0..h {
            let di = i * h * 3 + j * 3;
            let fi = j * w + i;
            let pb = pol[fi].as_elements();
            assert_eq!(pol[fi].dim, 3);
            res[di] = F3G::from(pb[0]);
            res[di + 1] = F3G::from(pb[1]);
            res[di + 2] = F3G::from(pb[2]);
        }
    }
    res
}

fn get3(arr: &Vec<BaseElement>, idx: usize) -> F3G {
    F3G::new(arr[idx * 3], arr[idx * 3 + 1], arr[idx * 3 + 2])
}

fn split3(arr: &Vec<BaseElement>) -> Vec<F3G> {
    let mut res: Vec<F3G> = Vec::new();
    for i in (0..arr.len()).step_by(3) {
        res.push(F3G::new(arr[i], arr[i + 1], arr[i + 2]));
    }
    return res;
}

fn pol_mul_axi(p: &mut Vec<F3G>, init: F3G, acc: &F3G) {
    let mut r = init;
    for i in 0..p.len() {
        p[i] *= r;
        r *= *acc;
    }
}

// TODO: use winter_math
fn eval_pol(p: &Vec<F3G>, x: &F3G) -> F3G {
    if p.len() == 0 {
        return F3G::ZERO;
    }
    let mut res = p[p.len() - 1];
    for i in (0..(p.len() - 1)).rev() {
        res = res * *x + p[i];
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::f3g::F3G;
    use crate::fri::eval_pol;
    use winter_math::polynom;
    use winter_math::StarkField;

    #[test]
    fn test_eval_pol() {
        let p = vec![1, 2, 3, 4]
            .iter()
            .map(|e| F3G::from(e))
            .collect::<Vec<F3G>>();
        let xi = F3G::from(10);

        let expected = polynom::eval(&p, xi);
        assert_eq!(eval_pol(&p, &xi), expected);
    }
}
