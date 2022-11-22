#![allow(non_snake_case)]
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::Segment;

use crate::constant::{SHIFT, TWIDDLES};
use crate::digest_bn128::ElementDigest;
use crate::errors::Result;
use crate::f3g::F3G;
use crate::interpreter::compile_code;
use crate::merklehash_bn128::MerkleTree;
use crate::polsarray::PolsArray;
use crate::stark_setup::StarkSetup;
use crate::types::{StarkStruct, PIL};
use winter_fri::FriProof;
use winter_math::fft;
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, StarkField};

pub struct StarkContext {
    pub nbits: usize,
    pub nbits_ext: usize,
    pub N: usize,
    pub Next: usize,
    pub challenge: Vec<F3G>,
    pub tmp: Vec<F3G>,
    pub cm1_n: Vec<F3G>,
    pub cm2_n: Vec<F3G>,
    pub cm3_n: Vec<F3G>,
    pub exps_withq_n: Vec<F3G>,
    pub exps_withoutq_n: Vec<F3G>,
    pub cm1_2ns: Vec<F3G>,
    pub cm2_2ns: Vec<F3G>,
    pub cm3_2ns: Vec<F3G>,
    pub q_2ns: Vec<F3G>,
    pub exps_withq_2ns: Vec<F3G>,
    pub exps_withoutq_2ns: Vec<F3G>,
    pub x_n: Vec<F3G>,
    pub x_2ns: Vec<F3G>,
    pub Zi: Box<dyn Fn(usize) -> F3G>,
    pub const_n: Vec<F3G>,
    pub const_2ns: Vec<F3G>,
    pub publics: Vec<F3G>,
    pub xDivXSubXi: Vec<BaseElement>,
    pub xDivXSubWXi: Vec<BaseElement>,
    pub evals: Vec<F3G>,

    pub exps_n: Vec<F3G>,
    pub exps_2ns: Vec<F3G>,
}

impl Default for StarkContext {
    fn default() -> Self {
        StarkContext {
            nbits: 0,
            nbits_ext: 0,
            N: 0,
            Next: 0,
            challenge: Vec::new(),
            tmp: Vec::new(),
            cm1_n: Vec::new(),
            cm2_n: Vec::new(),
            cm3_n: Vec::new(),
            exps_withq_n: Vec::new(),
            exps_withoutq_n: Vec::new(),
            cm1_2ns: Vec::new(),
            cm2_2ns: Vec::new(),
            cm3_2ns: Vec::new(),
            q_2ns: Vec::new(),
            exps_withq_2ns: Vec::new(),
            exps_withoutq_2ns: Vec::new(),
            x_n: Vec::new(),
            x_2ns: Vec::new(),
            Zi: Box::new(|i: usize| F3G::ZERO),
            const_n: Vec::new(),
            const_2ns: Vec::new(),
            publics: Vec::new(),
            xDivXSubXi: Vec::new(),
            xDivXSubWXi: Vec::new(),
            evals: Vec::new(),
            exps_n: Vec::new(),
            exps_2ns: Vec::new(),
        }
    }
}

pub struct StarkProof<'a> {
    stark_setup: &'a StarkSetup,
    fri_proof: FriProof,
    root: [ElementDigest; 4],
    publics: Vec<Segment>,
}

impl<'a> StarkProof<'a> {
    pub fn stark_gen(
        cm_pols: &PolsArray,
        const_pols: &PolsArray,
        const_tree: &MerkleTree,
        starkinfo: &'a StarkInfo,
        program: &Program,
        pil: &PIL,
        stark_struct: &StarkStruct,
    ) -> Result<StarkContext> {
        let mut ctx = StarkContext::default();

        ctx.nbits = stark_struct.nBits as usize;
        ctx.nbits_ext = stark_struct.nBitsExt as usize;
        ctx.N = 1 << stark_struct.nBits as usize;
        ctx.Next = 1 << stark_struct.nBitsExt as usize;
        assert_eq!(1 << ctx.nbits, ctx.N, "N must be a power of 2");

        let n_cm = starkinfo.n_cm1;

        ctx.cm1_n = cm_pols.write_buff();
        ctx.cm2_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm2_n as usize) * ctx.N];
        ctx.cm3_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm3_n as usize) * ctx.N];
        ctx.exps_withq_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.exps_withq_n as usize) * ctx.N];
        ctx.exps_withoutq_n =
            vec![F3G::ZERO; (starkinfo.map_sectionsN.exps_withoutq_n as usize) * ctx.N];

        ctx.cm1_2ns = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm1_n as usize) * ctx.Next];
        ctx.cm2_2ns = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm2_n as usize) * ctx.Next];
        ctx.cm3_2ns = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm3_n as usize) * ctx.Next];

        ctx.q_2ns = vec![F3G::ZERO; starkinfo.map_sectionsN.q_2ns as usize * ctx.Next];
        ctx.exps_withq_2ns =
            vec![F3G::ZERO; starkinfo.map_sectionsN.exps_withq_2ns as usize * ctx.Next];
        ctx.exps_withoutq_2ns =
            vec![F3G::ZERO; starkinfo.map_sectionsN.exps_withoutq_2ns as usize * ctx.Next];

        ctx.x_n = vec![F3G::ZERO; ctx.N];
        let mut xx = F3G::ONE;
        for i in 0..ctx.N {
            ctx.x_n[i] = xx;
            xx = xx * TWIDDLES[ctx.nbits];
        }

        let extendBits = ctx.nbits_ext - ctx.nbits;
        ctx.x_2ns = vec![F3G::ZERO; ctx.N];
        let mut xx = SHIFT.clone();
        for i in 0..(1 << (ctx.nbits_ext - ctx.nbits)) {
            ctx.x_2ns[i] = xx;
            xx = xx * TWIDDLES[ctx.nbits_ext];
        }

        ctx.Zi = Self::build_Zh_Inv(ctx.nbits, extendBits);

        ctx.const_n = const_pols.write_buff();
        ctx.const_2ns = const_tree.write_buff();

        ctx.publics = vec![F3G::ZERO; starkinfo.publics.len()];
        for (i, pe) in starkinfo.publics.iter().enumerate() {
            if pe.polType.as_str() == "cmP" {
                ctx.publics[i] =
                    ctx.cm1_n[(pe.idx * starkinfo.map_sectionsN.cm1_n + pe.polId) as usize];
            } else if pe.polType.as_str() == "imP" {
                ctx.publics[i] = Self::calculate_exp_at_point(
                    &mut ctx,
                    starkinfo,
                    &program.publics_code[i],
                    pe.idx,
                );
            } else {
                panic!("Invalid public type {}", pe.polType);
            }
        }

        Ok(ctx)
    }

    pub fn calculate_exp_at_point(
        ctx: &mut StarkContext,
        starkinfo: &StarkInfo,
        seg: &Segment,
        idx: i32,
    ) -> F3G {
        ctx.tmp = vec![F3G::ZERO; seg.tmp_used as usize];
        let t = compile_code(ctx, starkinfo, &seg.first, "n", true);
        t.eval(ctx, idx as usize)
    }

    pub fn build_Zh_Inv(nBits: usize, extendBits: usize) -> Box<dyn Fn(usize) -> F3G + 'static> {
        let mut w = F3G::ONE;
        let mut sn = SHIFT.clone();
        for i in 0..nBits {
            sn = sn * sn;
        }
        let mut ZHInv = vec![];
        for i in 0..(1 << extendBits) {
            ZHInv[i] = -(sn * w - F3G::ONE);
            w = w * TWIDDLES[extendBits];
        }
        Box::new(move |i: usize| ZHInv[i].clone())
    }
}
