use crate::constant::{MG, SHIFT};
use crate::digest_bn128::ElementDigest;
use crate::errors::{EigenError::FRIVerifierFailed, Result};
use crate::f3g::F3G;
use crate::fri::FRI;
use crate::merklehash_bn128::MerkleTree;
use crate::poseidon_bn128::Fr;
use crate::poseidon_bn128::FrRepr;
use crate::stark_gen::StarkContext;
use crate::stark_gen::StarkProof;
use crate::starkinfo::Program;
use crate::starkinfo::StarkInfo;
use crate::starkinfo_codegen::Segment;
use crate::starkinfo_codegen::{Node, Section};
use crate::transcript_bn128::TranscriptBN128;
use crate::types::StarkStruct;
use ff::PrimeField;
use winter_math::{fields::f64::BaseElement, FieldElement, StarkField};

pub fn stark_verify(
    proof: &StarkProof,
    const_root: &ElementDigest,
    starkinfo: &mut StarkInfo,
    stark_struct: &StarkStruct,
    program: &mut Program,
) -> Result<bool> {
    let mut transcript = TranscriptBN128::new();

    let mut ctx = StarkContext::default();

    let extendBits = stark_struct.nBitsExt - stark_struct.nBits;

    ctx.N = 1 << stark_struct.nBits;
    ctx.nbits = stark_struct.nBits;
    ctx.nbits_ext = stark_struct.nBitsExt;
    ctx.evals = proof.evals.clone();
    ctx.publics = proof.publics.clone();

    for i in 0..proof.publics.len() {
        let b = ctx.publics[i]
            .as_elements()
            .iter()
            .map(|e| Fr::from_repr(FrRepr::from(e.as_int())).unwrap())
            .collect::<Vec<Fr>>();
        transcript.put(&b);
    }

    transcript.put(&[proof.root1.into()])?;
    ctx.challenges[0] = transcript.get_field(); // u
    ctx.challenges[1] = transcript.get_field(); // defVal
    transcript.put(&[proof.root2.into()])?;
    ctx.challenges[2] = transcript.get_field(); // gamma
    ctx.challenges[3] = transcript.get_field(); // beta

    transcript.put(&[proof.root3.into()])?;
    ctx.challenges[4] = transcript.get_field(); // vc

    ctx.challenges[7] = transcript.get_field(); // xi
    for i in 0..ctx.evals.len() {
        let b = ctx.evals[i]
            .as_elements()
            .iter()
            .map(|e| Fr::from_repr(FrRepr::from(e.as_int())).unwrap())
            .collect::<Vec<Fr>>();
        transcript.put(&b);
    }

    transcript.put(&[proof.root4.into()])?;
    ctx.challenges[5] = transcript.get_field(); // v1
    ctx.challenges[6] = transcript.get_field(); // v2

    let xN = ctx.challenges[7].exp(7);
    ctx.Z = xN - F3G::ONE;
    ctx.Zp = (ctx.challenges[7] * MG.0[ctx.nbits]).pow(ctx.N) - F3G::ONE;

    let res = execute_code(&mut ctx, &mut program.verifier_code.first);

    let mut xAcc = F3G::ONE;
    let mut q = F3G::ZERO;
    for i in 0..starkinfo.q_deg {
        //q = F.add(q, F.mul(xAcc, ctx.evals[starkInfo.evIdx.cm[0][starkInfo.qs[i]]]));
        q = q + xAcc * ctx.evals[*starkinfo.ev_idx.get("cm", 0, starkinfo.qs[i]).unwrap()];
        //xAcc = F.mul(xAcc, xN);
        xAcc = xAcc * xN;
    }
    let qZ = q * ctx.Z;

    if !res.eq(&qZ) {
        return Ok(false);
    }

    let fri = FRI::new(stark_struct);
    let check_query =
        |query: &Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)>, idx: usize| -> Result<Vec<F3G>> {
            let tree = MerkleTree::new();
            let res = tree.verify_group_proof(&proof.root1, &query[0].1, idx, &query[0].0)?;
            if !res {
                return Err(FRIVerifierFailed);
            }

            let res = tree.verify_group_proof(&proof.root2, &query[1].1, idx, &query[1].0)?;
            if !res {
                return Err(FRIVerifierFailed);
            }
            let res = tree.verify_group_proof(&proof.root3, &query[2].1, idx, &query[2].0)?;
            if !res {
                return Err(FRIVerifierFailed);
            }
            let res = tree.verify_group_proof(&proof.root4, &query[3].1, idx, &query[3].0)?;
            if !res {
                return Err(FRIVerifierFailed);
            }
            let res = tree.verify_group_proof(&const_root, &query[4].1, idx, &query[4].0)?;
            if !res {
                return Err(FRIVerifierFailed);
            }
            let mut ctx_query = StarkContext::default();
            ctx_query.tree1 = query[0].0.clone();
            ctx_query.tree2 = query[1].0.clone();
            ctx_query.tree3 = query[2].0.clone();
            ctx_query.tree4 = query[3].0.clone();
            ctx_query.consts = query[4].0.clone();

            ctx_query.evals = ctx.evals.clone();
            ctx_query.publics = ctx.publics.clone();
            ctx_query.challenges = ctx.challenges.clone();

            let x = SHIFT.clone() * (MG.0[ctx.nbits + extendBits].exp(idx));
            ctx_query.xDivXSubXi = (x / (x - ctx_query.challenges[7])).as_elements();
            ctx_query.xDivXSubWXi =
                (x / (x - (ctx_query.challenges[7] * MG.0[ctx.nbits]))).as_elements();

            let vals = [execute_code(
                &mut ctx_query,
                &mut program.verifier_code.first,
            )];

            Ok(vals.to_vec())
        };

    fri.verify(&mut transcript, &proof.fri_proof, check_query)
}

fn execute_code(ctx: &mut StarkContext, code: &mut Vec<Section>) -> F3G {
    let mut tmp: Vec<F3G> = vec![];

    let extract_val = |arr: &Vec<BaseElement>, pos: usize, dim: usize| -> F3G {
        if dim == 1 {
            F3G::from(arr[pos])
        } else if dim == 3 {
            let r = &arr[pos..pos + 3];
            F3G::new(r[0], r[1], r[2])
        } else {
            panic!("Invalid dimension");
        }
    };

    let get_ref = |r: &Node, tmp: &Vec<F3G>| -> F3G {
        match r.type_.as_str() {
            "tmp" => tmp[r.id],
            "tree1" => extract_val(&ctx.tree1, r.tree_pos, r.dim),
            _ => panic!("Invalid reference type, get: {}", r.type_),
        }
    };

    let set_ref = |r: &mut Node, val: F3G, tmp: &mut Vec<F3G>| match r.type_.as_str() {
        "tmp" => {
            tmp[r.id] = val;
        }
        _ => {
            panic!("Invalid reference type set: {}", r.type_);
        }
    };

    for i in 0..code.len() {
        let mut src: Vec<F3G> = vec![];
        for s in code[i].src.iter() {
            src.push(get_ref(s, &tmp));
        }
        let res = match code[i].op.as_str() {
            "add" => src[0] + src[1],
            "sub" => src[0] - src[1],
            "mul" => src[0] * src[1],
            "copy" => src[0],
            _ => panic!("Invalid op: {}", code[i].op),
        };

        set_ref(&mut code[i].dest, res, &mut tmp);
    }
    let sz = code.len() - 1;
    get_ref(&code[sz].dest, &tmp)
}
