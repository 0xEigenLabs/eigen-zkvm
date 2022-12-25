use crate::constant::{MG, SHIFT, SHIFT_INV};
use crate::digest::ElementDigest;
use crate::errors::{EigenError::FRIVerifierFailed, Result};
use crate::f3g::F3G;
use crate::fft::FFT;
use crate::field_bn128::{Fr, FrRepr};
use crate::helper::log2_any;
use crate::traits::{MerkleTree, Transcript};
use crate::types::{StarkStruct, Step};
use ff::*;
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
pub struct Query<MB: Clone + std::default::Default> {
    pub pol_queries: Vec<Vec<(Vec<BaseElement>, Vec<Vec<MB>>)>>,
    pub root: ElementDigest,
}

/*
use std::fmt;
impl fmt::Display for Query<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "root {}\n", self.root)?;
        write!(f, "pol_queries size {}\n", self.pol_queries.len())?;
        for (i, pq) in self.pol_queries.iter().enumerate() {
            write!(f, "\t pq {}\n", i)?;
            for (_j, qq) in pq.iter().enumerate() {
                write!(f, "\t\tleaf: ")?;
                for qqq in qq.0.iter() {
                    write!(f, "{},", qqq)?;
                }
                write!(f, "\n\t\tnode:")?;
                for qqq in qq.1.iter() {
                    write!(f, "\t\t[\n")?;
                    for t in qqq.iter() {
                        //write!(f, "\t\t\t Fr {}\n", crate::helper::fr_to_biguint(t))?; //FIXME
                    }
                    write!(f, "\t\t]\n")?;
                }
            }
        }
        Ok(())
    }
}
*/

#[derive(Clone)]
pub struct FRIProof<M: MerkleTree> {
    pub queries: Vec<Query<M::BaseField>>,
    pub last: Vec<F3G>,
}

impl<M: MerkleTree> FRIProof<M> {
    pub fn new(qs: usize) -> Self {
        FRIProof {
            queries: vec![Query::<M::BaseField>::default(); qs],
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

    pub fn prove<M: MerkleTree, T: Transcript>(
        &mut self,
        transcript: &mut T,
        pol: &Vec<F3G>,
        mut query_pol: impl FnMut(usize) -> Vec<(Vec<BaseElement>, Vec<Vec<M::BaseField>>)>,
    ) -> Result<FRIProof<M>> {
        let mut pol = pol.clone();
        let mut standard_fft = FFT::new();
        let mut pol_bits = log2_any(pol.len()) as usize;
        log::debug!("fri prove {} {}", pol.len(), 1 << pol_bits);
        assert_eq!(1 << pol_bits, pol.len());
        assert_eq!(pol_bits, self.in_nbits);

        let mut shift_inv = SHIFT_INV.clone();
        let mut shift = SHIFT.clone();
        let mut tree: Vec<M> = vec![];

        let mut proof: FRIProof<M> = FRIProof::<M>::new(self.steps.len());
        for si in 0..self.steps.len() {
            let reduction_bits = pol_bits - self.steps[si].nBits;
            let pol2_n = 1 << (pol_bits - reduction_bits);
            let n_x = pol.len() / pol2_n;

            let mut pol2_e = vec![F3G::ZERO; pol2_n];
            let special_x = transcript.get_field();

            let mut sinv = shift_inv;
            let wi = F3G::inv(MG.0[pol_bits]);
            log::debug!("n_x {} {} {}", n_x, pol2_n, special_x);
            //crate::helper::pretty_print_array(&pol);

            for g in 0..(pol.len() / n_x) {
                if si == 0 {
                    pol2_e[g] = pol[g];
                } else {
                    let mut ppar = vec![F3G::ZERO; n_x];
                    for i in 0..n_x {
                        ppar[i] = pol[i * pol2_n + g];
                    }

                    let mut ppar_c = standard_fft.ifft(&ppar);
                    pol_mul_axi(&mut ppar_c, F3G::ONE, &sinv);
                    pol2_e[g] = eval_pol(&ppar_c, &special_x);
                    sinv = sinv * wi;
                }
            }
            log::debug!("pol2_e 0={}, 1={}", pol2_e[0], pol2_e[1]);

            if si < self.steps.len() - 1 {
                let n_groups = 1 << self.steps[si + 1].nBits;
                let group_size = (1 << self.steps[si].nBits) / n_groups;
                let pol2_etb = get_transposed_buffer(&pol2_e, self.steps[si + 1].nBits);
                let mut tmptree = M::new();
                tmptree.merkelize(pol2_etb, 3 * group_size, n_groups)?;
                tree.push(tmptree);
                proof.queries[si + 1].root = tree[si].root();
                transcript.put(&[tree[si].root()])?;
            } else {
                log::debug!("last {}", pol2_e.len());
                for e in pol2_e.iter() {
                    let elems = e.as_elements();
                    let v = [
                        ElementDigest::from(
                            &Fr::from_repr(FrRepr::from(elems[0].as_int())).unwrap(),
                        ),
                        ElementDigest::from(
                            &Fr::from_repr(FrRepr::from(elems[1].as_int())).unwrap(),
                        ),
                        ElementDigest::from(
                            &Fr::from_repr(FrRepr::from(elems[2].as_int())).unwrap(),
                        ),
                    ];
                    transcript.put(&v)?;
                }
            }

            pol = pol2_e;
            pol_bits -= reduction_bits;

            for _j in 0..reduction_bits {
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

        let query_pol_fn =
            |si: usize, idx: usize| -> Vec<(Vec<BaseElement>, Vec<Vec<M::BaseField>>)> {
                log::debug!("query_pol_fn: si:{}, idx:{}", si, idx);
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
            //log::debug!("prove_query_pol: {} {}", si, proof.queries[si]);

            if si < self.steps.len() - 1 {
                for i in 0..ys.len() {
                    ys[i] = ys[i] % (1 << self.steps[si + 1].nBits);
                }
            }
        }
        Ok(proof)
    }

    pub fn verify<M: MerkleTree, T: Transcript>(
        &self,
        transcript: &mut T,
        proof: &FRIProof<M>,
        mut check_query: impl FnMut(
            &Vec<(Vec<BaseElement>, Vec<Vec<M::BaseField>>)>,
            usize,
        ) -> Result<Vec<F3G>>,
    ) -> Result<bool> {
        let tree = M::new();
        let mut standard_fft = FFT::new();
        assert_eq!(proof.queries.len(), self.steps.len()); // the last +1 is ommited
        let mut special_x: Vec<F3G> = vec![];
        for si in 0..self.steps.len() {
            special_x.push(transcript.get_field());
            if si < self.steps.len() - 1 {
                //let n_groups = 1 << self.steps[si + 1].nBits;
                //let group_size = (1 << self.steps[si].nBits) / n_groups;
                transcript.put(&[proof.queries[si + 1].root])?;
            } else {
                let mut pp: Vec<ElementDigest> = vec![];
                for e in proof.last.iter() {
                    let elems = e.as_elements();
                    pp.push(ElementDigest::from(
                        &Fr::from_str(&elems[0].as_int().to_string()).unwrap(),
                    ));
                    pp.push(ElementDigest::from(
                        &Fr::from_str(&elems[1].as_int().to_string()).unwrap(),
                    ));
                    pp.push(ElementDigest::from(
                        &Fr::from_str(&elems[2].as_int().to_string()).unwrap(),
                    ));
                }
                transcript.put(&pp[..])?;
            }
        }

        let n_queries = self.n_queries;
        let mut ys = transcript.get_permutations(self.n_queries, self.steps[0].nBits)?;
        let pol_bits = self.in_nbits;
        log::debug!("ys: {:?}, pol_bits {}", ys, self.in_nbits);
        let mut shift = SHIFT.clone();

        let check_query_fn = |si: usize,
                              query: &Vec<(Vec<BaseElement>, Vec<Vec<M::BaseField>>)>,
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
            //log::debug!("si: {}, queries: {}", si, proof_item);
            let reduction_bits = pol_bits - self.steps[si].nBits;
            log::debug!("si {} reduction_bits {}", si, reduction_bits);
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

                let pgroup_c = standard_fft.ifft(&pgroup_e);
                let sinv = F3G::inv(shift * (MG.0[pol_bits].exp(ys[i])));

                log::debug!("sinv {}, special_x[{}]={}", sinv, si, special_x[si]);

                let ev = eval_pol(&pgroup_c, &(special_x[si] * sinv));
                log::debug!("ev {}", ev);

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

            //let pol_bits = self.steps[si].nBits;
            for _j in 0..reduction_bits {
                shift = shift * shift;
            }

            if si < self.steps.len() - 1 {
                for i in 0..ys.len() {
                    ys[i] = ys[i] % (1 << self.steps[si + 1].nBits);
                }
            }
        }

        let last_pol_e = proof.last.clone();

        #[allow(unused_assignments)]
        let mut max_deg = 0usize;
        if pol_bits < (self.in_nbits - self.max_deg_nbits) {
            max_deg = 0;
        } else {
            max_deg = 1 << (pol_bits - (self.in_nbits - self.max_deg_nbits));
        }

        let last_pol_c = standard_fft.ifft(&last_pol_e);

        for i in (max_deg + 1)..last_pol_c.len() {
            if !last_pol_c[i].is_zero() {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn get_transposed_buffer(pol: &Vec<F3G>, transpose_bits: usize) -> Vec<BaseElement> {
    let n = pol.len();
    let w = 1 << transpose_bits;
    let h = n / w;
    let mut res: Vec<BaseElement> = vec![BaseElement::ZERO; n * 3];
    for i in 0..w {
        for j in 0..h {
            let di = i * h * 3 + j * 3;
            let fi = j * w + i;
            let pb = pol[fi].as_elements();
            assert_eq!(pol[fi].dim, 3);
            res[di] = pb[0];
            res[di + 1] = pb[1];
            res[di + 2] = pb[2];
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
