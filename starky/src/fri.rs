use crate::f3g::F3G;
use ff::*;
use std::collections::HashMap;
use std::rc::Rc;
use winter_math::FieldElement;
use winter_math::StarkField;
use winter_math::{fft, fields::f64::BaseElement, log2, polynom};

use crate::constant::{SHIFT, SHIFT_INV, W};
use crate::digest_bn128::ElementDigest;
use crate::errors::{EigenError::FRIVerifierFailed, Result};
use crate::merklehash_bn128::MerkleTree;
use crate::poseidon_bn128::Fr;
use crate::transcript_bn128::TranscriptBN128;
use crate::types::{StarkStruct, Step};

pub struct FRI {
    pub in_nbits: usize,
    pub max_deg_nbits: usize,
    pub n_queries: usize,
    pub steps: Vec<Step>,
}

#[derive(Default, Clone)]
pub struct ProofOne {
    pub polQueries: Vec<Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>>,
    pub root: ElementDigest,
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

        let mut pol_bits = log2(pol.len()) as usize;
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
            let inv_twiddles = fft::get_inv_twiddles(nX);

            let mut pol2_e = vec![F3G::ZERO; pol2N];
            let special_x = transcript.get_field();

            let mut sinv = shift_inv;
            let wi = F3G::inv(W.0[pol_bits]);

            for g in 0..(pol.len() / nX) {
                if si == 0 {
                    pol2_e[g] = pol[g];
                } else {
                    let mut ppar = vec![F3G::ZERO; nX];
                    for i in 0..nX {
                        ppar[i] = pol[i * pol2N + g];
                    }

                    fft::interpolate_poly(&mut ppar, &inv_twiddles);

                    pol_mul_axi(&mut ppar, &F3G::ONE, &sinv);
                    pol2_e[g] = eval_pol(&ppar, &special_x);
                    sinv = sinv * wi;
                }
            }

            if si < self.steps.len() - 1 {
                let n_groups = 1 << self.steps[si + 1].nBits;
                let group_size = (1 << self.steps[si].nBits) / n_groups;

                let pol2_etb = getTransposedBuffer(&pol2_e, self.steps[si + 1].nBits);
                tree.push(MerkleTree::merkelize(pol2_etb, 3 * group_size, n_groups)?);
                proof.queries[si + 1].root = tree[si].root();
                transcript.put(&vec![tree[si].root().into()]);
            } else {
                let mut pp: Vec<Fr> = vec![];
                for e in pol2_e.iter() {
                    let elems = e.as_base_elements();
                    pp.push(Fr::from_str(&elems[0].as_int().to_string()).unwrap());
                    pp.push(Fr::from_str(&elems[1].as_int().to_string()).unwrap());
                    pp.push(Fr::from_str(&elems[2].as_int().to_string()).unwrap());
                }
                transcript.put(&pp);
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

        for ys_ in ys.iter() {
            proof.queries[0].polQueries.push(query_pol(*ys_));
        }

        for si in 1..self.steps.len() {
            let query_pol = |idx: usize| -> Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)> {
                vec![tree[si].get_group_proof(idx).unwrap()]
            };

            for i in 0..ys.len() {
                ys[i] = ys[i] % (1 << self.steps[si].nBits);
            }

            // calculate the next query immediately.
            for ys_ in ys.iter() {
                proof.queries[si].polQueries.push(query_pol(*ys_));
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
        assert_eq!(proof.queries.len(), self.steps.len()); // the last +1 is ommited
        let mut special_x: Vec<F3G> = vec![];
        for si in 0..self.steps.len() {
            special_x.push(transcript.get_field());
            if si < self.steps.len() - 1 {
                let n_groups = 1 << self.steps[si + 1].nBits;
                let group_size = (1 << self.steps[si].nBits) / n_groups;
                transcript.put(&vec![proof.queries[si + 1].root.into()]);
            } else {
                let mut pp: Vec<Fr> = vec![];
                for e in proof.last.iter() {
                    let elems = e.as_base_elements();
                    pp.push(Fr::from_str(&elems[0].as_int().to_string()).unwrap());
                    pp.push(Fr::from_str(&elems[1].as_int().to_string()).unwrap());
                    pp.push(Fr::from_str(&elems[2].as_int().to_string()).unwrap());
                }
                transcript.put(&pp);
            }
        }

        let n_queries = self.n_queries;
        let mut ys = transcript.get_permutations(self.n_queries, self.steps[0].nBits)?;
        let mut pol_bits = self.in_nbits;
        let mut shift = SHIFT.clone();

        let check_query_fn = |si: usize,
                              query: &Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>,
                              idx: usize|
         -> Result<Vec<F3G>> {
            let res = tree.verify_group_proof(
                &proof.queries[si + 1].root,
                &query[0].1,
                idx,
                &query[0].0,
            )?;
            if !res {
                return Err(FRIVerifierFailed);
            }
            Ok(split3(&query[0].0))
        };

        for si in 0..self.steps.len() {
            let proof_item = &proof.queries[si];
            let reduction_bits = pol_bits - self.steps[si].nBits;
            for i in 0..n_queries {
                let mut pgroup_e: Vec<F3G> = vec![];
                if si == 0 {
                    pgroup_e = check_query(&proof_item.polQueries[i], ys[i])?;
                    if pgroup_e.len() == 0 {
                        return Ok(false);
                    }
                } else {
                    pgroup_e = check_query_fn(si, &proof_item.polQueries[i], ys[i])?;
                }

                // ifft
                let inv_twiddles = fft::get_inv_twiddles(pgroup_e.len());
                fft::interpolate_poly(&mut pgroup_e, &inv_twiddles);

                let sinv = F3G::inv(shift * (W.0[pol_bits].exp(ys[i])));
                let ev = eval_pol(&pgroup_e, &(special_x[si] * sinv));

                if si < self.steps.len() - 1 {
                    let next_n_groups = 1 << self.steps[si + 1].nBits;
                    let group_idx = ys[i] / next_n_groups;
                    if !ev.eq(&get3(&proof.queries[si + 1].polQueries[i][0].0, group_idx)) {
                        return Ok(false);
                    }
                } else {
                    if !ev.eq(&proof.last[ys[i]]) {
                        return Ok(false);
                    }
                }
            }
            /*
            check_query =
                &|query: &Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>, idx: usize| -> Result<Vec<F3G>> {
                    let res = tree.verify_group_proof(
                        &proof.queries[si + 1].root,
                        &query[0].1,
                        idx,
                        &query[0].0,
                    )?;
                    if !res {
                        return Err(FRIVerifierFailed);
                    }
                    Ok(split3(&query[0].0))
                };
            */

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

        let inv_twiddles = fft::get_inv_twiddles(last_pol_e.len());
        fft::interpolate_poly(&mut last_pol_e, &inv_twiddles);

        for i in (maxDeg + 1)..last_pol_e.len() {
            if !last_pol_e[i].is_zero() {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

fn getTransposedBuffer(pol: &Vec<F3G>, trasposeBits: usize) -> Vec<Vec<BaseElement>> {
    let n = pol.len();
    let w = 1 << trasposeBits;
    let h = n / w;
    let mut res: Vec<Vec<BaseElement>> = vec![Vec::new(); h];
    for j in 0..h {
        res[j] = vec![BaseElement::ZERO; w * 3];
        for i in 0..w {
            let di = i * 3;
            let fi = j * h + i;
            let pb = pol[fi].as_base_elements();
            res[j][di] = pb[0];
            res[j][di + 1] = pb[1];
            res[j][di + 2] = pb[2];
        }
    }

    /*
    for i in 0..n_pols {
        columns[i] = vec![BaseElement::ZERO; n];
        for j in 0..n {
            columns[i][j] = p[i * n_pols + j].to_be();
        }
    }
    */
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

fn pol_mul_axi(p: &mut Vec<F3G>, init: &F3G, acc: &F3G) {
    let mut r = *init;
    for i in 0..p.len() {
        p[i] = p[i] * r;
        r = r * *acc;
    }
}

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
