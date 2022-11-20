use crate::starkinfo::StarkInfo;
use crate::starkinfo_codegen::Segment;

use crate::digest_bn128::ElementDigest;
use crate::errors::Result;
use crate::f3g::F3G;
use crate::merklehash_bn128::MerkleTree;
use crate::polsarray::PolsArray;
use crate::stark_setup::StarkSetup;
use crate::transcript_bn128::TranscriptBN128;
use crate::types::{StarkStruct, PIL};
use winter_fri::FriProof;

use crate::constant::{SHIFT, TWIDDLES};
use winter_math::{fft, fields::f64::BaseElement};
use winter_math::{FieldElement, StarkField};

pub struct StarkContext {
    pub nbits: i32,
    pub nbits_ext: i32,
    pub N: i32,
    pub Next: i32,
    pub challenge: Vec<BaseElement>,
    pub tmp: Vec<BaseElement>,
    pub cm1_n: Vec<BaseElement>,
    pub cm2_n: Vec<BaseElement>,
    pub cm3_n: Vec<BaseElement>,
    pub exps_withq_n: Vec<BaseElement>,
    pub exps_withoutq_n: Vec<BaseElement>,
    pub cm1_2ns: Vec<BaseElement>,
    pub cm2_2ns: Vec<BaseElement>,
    pub cm3_2ns: Vec<BaseElement>,
    pub q_2ns: Vec<BaseElement>,
    pub exps_withq_2ns: Vec<BaseElement>,
    pub exps_withoutq_2ns: Vec<BaseElement>,
    pub x_n: Vec<BaseElement>,
    pub x_2ns: Vec<BaseElement>,
    pub Zi: Box<dyn Fn(usize) -> BaseElement>,
    pub const_n: Vec<BaseElement>,
    pub const_2ns: Vec<BaseElement>,
    pub publics: Vec<BaseElement>,
    pub xDivXSubXi: Vec<BaseElement>,
    pub xDivXSubWXi: Vec<BaseElement>,
    pub evals: Vec<F3G>,
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
            Zi: Box::new(|i: usize| BaseElement::ZERO),
            const_n: Vec::new(),
            const_2ns: Vec::new(),
            publics: Vec::new(),
            xDivXSubXi: Vec::new(),
            xDivXSubWXi: Vec::new(),
            evals: Vec::new(),
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
        starkinfo: &'a StarkInfo,
        pil: &PIL,
        stark_struct: &StarkStruct,
    ) -> Result<StarkContext> {
        let N = 1 << stark_struct.nBits as usize;
        let extendBits = (stark_struct.nBitsExt - stark_struct.nBits) as usize;
        let nBitsExt = stark_struct.nBitsExt;
        let nBits = stark_struct.nBits as usize;
        assert_eq!(1 << nBits, N, "N must be a power of 2");

        let mut ctx = StarkContext::default();

        ctx.x_n = vec![BaseElement::ZERO; N];
        let mut xx = BaseElement::ONE;
        for i in 0..N {
            ctx.x_n[i] = xx;
            xx = xx * TWIDDLES[nBits];
        }

        ctx.x_2ns = vec![BaseElement::ZERO; N];
        let mut xx = SHIFT.clone();
        for i in 0..(1 << extendBits) {
            ctx.x_2ns[i] = xx;
            xx = xx * TWIDDLES[nBits + extendBits];
        }

        ctx.Zi = Self::build_Zh_Inv(nBits, extendBits);

        //ctx.const_n = const_pols.to_vec();

        Ok(ctx)
    }

    pub fn build_Zh_Inv(nBits: usize, extendBits: usize) -> Box<dyn Fn(usize) -> BaseElement> {
        let mut w = BaseElement::ONE;
        let mut sn = SHIFT.clone();
        for i in 0..nBits {
            sn = sn * sn;
        }
        let mut ZHInv = vec![];
        for i in 0..(1 << extendBits) {
            ZHInv[i] = -(sn * w - BaseElement::ONE);
            w = w * TWIDDLES[extendBits];
        }
        Box::new(|i: usize| ZHInv[i].clone())
    }
}
