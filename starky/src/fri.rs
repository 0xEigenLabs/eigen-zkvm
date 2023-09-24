use crate::constant::{MG, SHIFT, SHIFT_INV};
use crate::errors::{EigenError::FRIVerifierFailed, Result};
use crate::fft::FFT;
use crate::helper::log2_any;
use crate::polutils::{eval_pol, pol_mul_axi};
use crate::traits::{FieldExtension, MTNodeType, MerkleTree, Transcript};
use crate::types::{StarkStruct, Step};
use plonky::field_gl::Fr as FGL;

#[derive(Debug)]
pub struct FRI {
    pub in_nbits: usize,
    pub max_deg_nbits: usize,
    pub n_queries: usize,
    pub steps: Vec<Step>,
}

#[derive(Debug, Default, Clone)]
pub struct Query<MB: Clone + std::default::Default, MN: MTNodeType> {
    pub pol_queries: Vec<Vec<(Vec<FGL>, Vec<Vec<MB>>)>>,
    pub root: MN,
}

#[derive(Debug, Clone)]
pub struct FRIProof<F: FieldExtension, M: MerkleTree<ExtendField = F>> {
    pub queries: Vec<Query<M::BaseField, M::MTNode>>,
    pub last: Vec<F>,
}

impl<F: FieldExtension, M: MerkleTree<ExtendField = F>> FRIProof<F, M> {
    pub fn new(qs: usize) -> Self {
        FRIProof {
            queries: vec![Query::<M::BaseField, M::MTNode>::default(); qs],
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

    pub fn prove<F: FieldExtension, M: MerkleTree<ExtendField = F>, T: Transcript>(
        &mut self,
        transcript: &mut T,
        pol: &Vec<M::ExtendField>,
        mut query_pol: impl FnMut(usize) -> Vec<(Vec<FGL>, Vec<Vec<M::BaseField>>)>,
    ) -> Result<FRIProof<F, M>> {
        let mut pol = pol.clone();
        let mut standard_fft = FFT::new();
        let mut pol_bits = log2_any(pol.len()) as usize;
        log::debug!("fri prove {} {}", pol.len(), 1 << pol_bits);
        assert_eq!(1 << pol_bits, pol.len());
        assert_eq!(pol_bits, self.in_nbits);

        let mut shift_inv = F::from(SHIFT_INV.clone());
        let mut shift = F::from(SHIFT.clone());
        let mut tree: Vec<M> = vec![];

        let mut proof: FRIProof<F, M> = FRIProof::<F, M>::new(self.steps.len());
        for si in 0..self.steps.len() {
            let reduction_bits = pol_bits - self.steps[si].nBits;
            let pol2_n = 1 << (pol_bits - reduction_bits);
            let n_x = pol.len() / pol2_n;

            let mut pol2_e = vec![F::ZERO; pol2_n];
            let special_x = transcript.get_field();

            let mut sinv = shift_inv;
            let wi = F::inv(F::from(MG.0[pol_bits]));

            for g in 0..(pol.len() / n_x) {
                if si == 0 {
                    pol2_e[g] = pol[g];
                } else {
                    let mut ppar = vec![F::ZERO; n_x];
                    for i in 0..n_x {
                        ppar[i] = pol[i * pol2_n + g];
                    }

                    let mut ppar_c = standard_fft.ifft(&ppar);
                    pol_mul_axi(&mut ppar_c, F::ONE, &sinv);
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
                transcript.put(&[tree[si].root().as_elements().to_vec()])?;
            } else {
                for e in pol2_e.iter() {
                    let elems = e.as_elements();
                    let v = [vec![elems[0]], vec![elems[1]], vec![elems[2]]];
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
        let mut last_pol: Vec<F> = vec![];
        for p in pol.iter() {
            last_pol.push(*p);
        }

        proof.last = last_pol;
        let mut ys = transcript.get_permutations(self.n_queries, self.steps[0].nBits)?;
        /*
        let query_pol_fn =
            |si: usize, idx: usize| -> Vec<(Vec<FGL>, Vec<Vec<M::BaseField>>)> {
                log::debug!("query_pol_fn: si:{}, idx:{}", si, idx);
                vec![tree[si].get_group_proof(idx).unwrap()]
            };
        */

        for si in 0..self.steps.len() {
            for ys_ in ys.iter() {
                if si == 0 {
                    proof.queries[si].pol_queries.push(query_pol(*ys_));
                } else {
                    proof.queries[si]
                        .pol_queries
                        .push(vec![tree[si - 1].get_group_proof(*ys_).unwrap()]);
                }
            }
            if si < self.steps.len() - 1 {
                for i in 0..ys.len() {
                    ys[i] = ys[i] % (1 << self.steps[si + 1].nBits);
                }
            }
        }
        Ok(proof)
    }

    pub fn verify<F: FieldExtension, M: MerkleTree<ExtendField = F>, T: Transcript>(
        &self,
        transcript: &mut T,
        proof: &FRIProof<F, M>,
        mut check_query: impl FnMut(&Vec<(Vec<FGL>, Vec<Vec<M::BaseField>>)>, usize) -> Result<Vec<F>>,
    ) -> Result<bool> {
        let tree = M::new();
        let mut standard_fft = FFT::new();
        assert_eq!(proof.queries.len(), self.steps.len()); // the last +1 is ommited
        let mut special_x: Vec<F> = vec![];
        for si in 0..self.steps.len() {
            special_x.push(transcript.get_field());
            if si < self.steps.len() - 1 {
                //let n_groups = 1 << self.steps[si + 1].nBits;
                //let group_size = (1 << self.steps[si].nBits) / n_groups;
                transcript.put(&[proof.queries[si + 1].root.as_elements().to_vec()])?;
            } else {
                let mut pp: Vec<Vec<FGL>> = vec![];
                for e in proof.last.iter() {
                    let elems = e.as_elements();
                    pp.push(vec![elems[0]]);
                    pp.push(vec![elems[1]]);
                    pp.push(vec![elems[2]]);
                }
                transcript.put(&pp[..])?;
            }
        }

        let n_queries = self.n_queries;
        let mut ys = transcript.get_permutations(self.n_queries, self.steps[0].nBits)?;
        let mut pol_bits = self.in_nbits;
        log::debug!("ys: {:?}, pol_bits {}", ys, self.in_nbits);
        let mut shift = F::from(SHIFT.clone());

        let check_query_fn = |si: usize,
                              query: &Vec<(Vec<FGL>, Vec<Vec<M::BaseField>>)>,
                              idx: usize|
         -> Result<Vec<F>> {
            let res =
                tree.verify_group_proof(&proof.queries[si].root, &query[0].1, idx, &query[0].0)?;
            if !res {
                log::error!("check_query_fn failed si:{},idx:{}", si, idx);
                return Err(FRIVerifierFailed);
            }
            Ok(split3(&query[0].0))
        };

        for si in 0..self.steps.len() {
            let proof_item = &proof.queries[si];
            let reduction_bits = pol_bits - self.steps[si].nBits;
            for i in 0..n_queries {
                let pgroup_e: Vec<F> = match si {
                    0 => {
                        let pgroup_e = check_query(&proof_item.pol_queries[i], ys[i])?;
                        if pgroup_e.len() == 0 {
                            log::error!("check_query failed si:{}", si);
                            return Ok(false);
                        }
                        pgroup_e
                    }
                    _ => check_query_fn(si, &proof_item.pol_queries[i], ys[i])?,
                };

                let pgroup_c = standard_fft.ifft(&pgroup_e);
                let sinv = F::inv(shift * (F::from(MG.0[pol_bits]).exp(ys[i])));
                let ev = eval_pol(&pgroup_c, &(special_x[si] * sinv));

                if si < self.steps.len() - 1 {
                    let next_n_groups = 1 << self.steps[si + 1].nBits;
                    let group_idx = ys[i] / next_n_groups;
                    if !ev.eq(&get3(&proof.queries[si + 1].pol_queries[i][0].0, group_idx)) {
                        log::error!("eq query failed si:{}", si + 1);
                        return Ok(false);
                    }
                } else {
                    if !ev.eq(&proof.last[ys[i]]) {
                        log::error!("eq last failed si:{}, {}!={}", si, ev, &proof.last[ys[i]]);
                        return Ok(false);
                    }
                }
            }

            pol_bits = self.steps[si].nBits;
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
                log::error!("check last pol c failed");
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn get_transposed_buffer<F: FieldExtension>(pol: &Vec<F>, transpose_bits: usize) -> Vec<FGL> {
    let n = pol.len();
    let w = 1 << transpose_bits;
    let h = n / w;
    let mut res: Vec<FGL> = vec![FGL::ZERO; n * 3];
    for i in 0..w {
        for j in 0..h {
            let di = i * h * 3 + j * 3;
            let fi = j * w + i;
            let pb = pol[fi].as_elements();
            // TODO: Support F5G
            assert_eq!(pol[fi].dim(), 3);
            res[di] = pb[0];
            res[di + 1] = pb[1];
            res[di + 2] = pb[2];
        }
    }
    res
}

// TODO: Support F5G
fn get3<F: FieldExtension>(arr: &Vec<FGL>, idx: usize) -> F {
    F::from_vec(vec![arr[idx * 3], arr[idx * 3 + 1], arr[idx * 3 + 2]])
}

// TODO: Support F5G
fn split3<F: FieldExtension>(arr: &Vec<FGL>) -> Vec<F> {
    let mut res: Vec<F> = Vec::new();
    for i in (0..arr.len()).step_by(3) {
        res.push(F::from_vec(vec![arr[i], arr[i + 1], arr[i + 2]]));
    }
    return res;
}

/*
#[cfg(test)]
mod tests {
    use crate::f3g::F3G;
    use crate::polutils::eval_pol;
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
*/
