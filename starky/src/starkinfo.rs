use crate::errors::{EigenError, Result};
use crate::expressionops::ExpressionOps as E;
use crate::f3g as field;
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Calculated, Context, ContextF, EVIdx, Node, Segment,
};
use crate::types::{Expression, StarkStruct, PIL};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct PUCTX {
    pub f_exp_id: i32,
    pub t_exp_id: i32,
    pub h1_id: i32,
    pub h2_id: i32,

    pub z_id: i32,
    pub c1_id: i32,
    pub c2_id: i32,
    pub num_id: i32,
    pub den_id: i32,
}

#[derive(Default, Debug)]
pub struct PECTX {
    pub f_exp_id: i32,
    pub t_exp_id: i32,

    pub z_id: i32,
    pub c1_id: i32,
    pub c2_id: i32,
    pub num_id: i32,
    pub den_id: i32,
}

#[derive(Default, Debug)]
pub struct CICTX {
    pub z_id: i32,
    pub c1_id: i32,
    pub c2_id: i32,
    pub num_id: i32,
    pub den_id: i32,
}

#[derive(Debug)]
pub struct StarkInfo {
    pub var_pol_map: usize,
    pub n_cm1: i32,
    pub pu_ctx: Vec<PUCTX>,
    pub pe_ctx: Vec<PECTX>,
    pub ci_ctx: Vec<CICTX>,
    pub n_constants: i32,
    pub n_publics: i32,
    pub c_exp: i32,
    pub publics_code: Vec<Segment>,
    pub step4: Segment,
    pub step42ns: Segment,
    pub ev_map: Vec<Node>,
    pub verifier_code: Segment,
    pub fri_exp_id: i32,
    pub step52ns: Segment,
}

impl StarkInfo {
    pub fn new(pil: &mut PIL, stark_struct: &StarkStruct) -> Result<StarkInfo> {
        let pil_deg = pil.references.values().nth(0).unwrap().polDeg as i32;

        let stark_deg = 2i32.pow(stark_struct.nBits as u32);

        if stark_deg != pil_deg {
            return Err(EigenError::MustEqualDegreeError(stark_deg, pil_deg));
        }

        if stark_struct.nBitsExt != stark_struct.steps[0]["nBits"] {
            return Err(EigenError::MustEqualDegreeError(
                stark_struct.nBitsExt,
                stark_struct.steps[0]["nBits"],
            ));
        }

        let mut info = StarkInfo {
            var_pol_map: 0,
            pu_ctx: Vec::new(),
            pe_ctx: Vec::new(),
            ci_ctx: Vec::new(),
            n_constants: pil.nConstants,
            n_publics: pil.publics.len() as i32,
            publics_code: vec![],
            n_cm1: pil.nCommitments,
            c_exp: 0,
            step4: Segment::default(),
            step42ns: Segment::default(),
            ev_map: Vec::new(),
            verifier_code: Segment::default(),
            fri_exp_id: 0,
            step52ns: Segment::default(),
        };

        let mut ctx = Context {
            pil: pil,
            calculated: Calculated {
                exps: vec![],
                exps_prime: vec![],
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };
        info.generate_pubulic_calculators(&mut ctx);

        let mut ctx = Context {
            pil: pil,
            calculated: Calculated {
                exps: vec![],
                exps_prime: vec![],
            },
            tmp_used: 0,
            code: vec![],
            exp_id: -1,
            calculated_mark: HashMap::new(),
        };

        info.generate_step2(&mut ctx);
        println!("{:?}, {:?}", pil, info);

        let mut ctx2ns = Context {
            pil: pil,
            calculated: Calculated {
                exps: Vec::new(),
                exps_prime: Vec::new(),
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };

        Ok(info)
    }

    pub fn generate_pubulic_calculators(&mut self, ctx: &mut Context) -> Result<()> {
        let publics = ctx.pil.publics.clone();
        for p in publics.iter() {
            if p.polType.as_str() == "imP" {
                pil_code_gen(ctx, p.polId, false, &"".to_string());
                let mut segment = build_code(ctx);

                let mut ctx_f = ContextF {
                    exp_map: HashMap::new(),
                    tmp_used: segment.tmp_used,
                    ev_idx: EVIdx::new(),
                    ev_map: Vec::new(),
                };

                let fix_ref = |r: &mut Node, ctx: &mut ContextF| {
                    let p = if r.prime.is_some() { 1 } else { 0 };
                    if r.type_.as_str() == "exp" {
                        if ctx.exp_map.get(&(p, r.id.unwrap())).is_none() {
                            ctx.exp_map.insert((p, r.id.unwrap()), ctx.tmp_used);
                            ctx.tmp_used += 1;
                        }
                        r.prime = None;
                        r.type_ = "tmp".to_string();
                        r.id = Some(*ctx.exp_map.get(&(p, r.id.unwrap())).unwrap());
                    }
                };
                iterate_code(&mut segment, fix_ref, &mut ctx_f);

                segment.tmp_used = ctx_f.tmp_used;
                self.publics_code.push(segment);
                ctx.calculated = Calculated {
                    exps: vec![],
                    exps_prime: vec![],
                };
            }
        }
        Ok(())
    }

    pub fn generate_step2(&mut self, ctx: &mut Context) {
        let ppi = ctx.pil.plookupIdentities.clone();
        for pi in ppi.iter() {
            let u = E::challenge("u".to_string());
            let def_val = E::challenge("defVal".to_string());

            let mut t_exp: Expression = E::nop();
            for j in pi.t.as_ref().unwrap().iter() {
                let e = E::exp(*j, None);
                if E::is_nop(&t_exp) {
                    t_exp = e;
                } else {
                    t_exp = E::add(&E::mul(&u, &t_exp), &e);
                }
            }

            if pi.selT.is_some() {
                t_exp = E::sub(&t_exp, &def_val);
                t_exp = E::mul(&t_exp, &E::exp(pi.selT.unwrap(), None));
                t_exp = E::add(&t_exp, &def_val);
                t_exp.idQ = Some(ctx.pil.nQ as i32);
                ctx.pil.nQ += 1;
            }

            let t_exp_id = ctx.pil.expressions.len() as i32;
            t_exp.keep = Some(true);
            ctx.pil.expressions.push(t_exp);

            let mut f_exp = E::nop();
            for j in pi.f.as_ref().unwrap().iter() {
                let e = E::exp(j.clone(), None);
                if f_exp == E::nop() {
                    f_exp = e;
                } else {
                    f_exp = E::add(&E::mul(&f_exp, &u), &e);
                }
            }

            let f_exp_id = ctx.pil.expressions.len() as i32;
            f_exp.keep = Some(true);
            ctx.pil.expressions.push(f_exp);

            pil_code_gen(ctx, f_exp_id.clone(), false, &"".to_string());
            pil_code_gen(ctx, t_exp_id.clone(), false, &"".to_string());

            let h1_id = ctx.pil.nCommitments;
            ctx.pil.nCommitments += 1;
            let h2_id = ctx.pil.nCommitments;
            ctx.pil.nCommitments += 1;

            self.pu_ctx.push(PUCTX {
                f_exp_id,
                t_exp_id,
                h1_id,
                h2_id,
                z_id: 0,
                c1_id: 0,
                c2_id: 0,
                num_id: 0,
                den_id: 0,
            });
        }
    }
}
