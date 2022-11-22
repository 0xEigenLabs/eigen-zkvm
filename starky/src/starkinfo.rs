#![allow(non_snake_case)]
use crate::errors::{EigenError, Result};
use crate::expressionops::ExpressionOps as E;
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Calculated, Context, ContextF, EVIdx, Node, PolType,
    Section, SectionVec, Segment,
};
use crate::types::{Expression, Public, StarkStruct, PIL};
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

#[derive(Debug, Default)]
pub struct Program {
    pub publics_code: Vec<Segment>,
    pub step2prev: Segment,
    pub step3prev: Segment,
    pub step4: Segment,
    pub step42ns: Segment,
    pub step52ns: Segment,
    pub verifier_code: Segment,
    pub verifier_query_code: Segment,
}

#[derive(Debug, Default)]
pub struct StarkInfo {
    pub var_pol_map: Vec<PolType>,
    pub n_cm1: i32,
    pub n_cm2: i32,
    pub n_cm3: i32,
    pub n_cm4: i32,
    pub n_q: i32,
    pub pu_ctx: Vec<PUCTX>,
    pub pe_ctx: Vec<PECTX>,
    pub ci_ctx: Vec<CICTX>,
    pub n_constants: i32,
    pub n_publics: i32,
    pub c_exp: i32,

    pub ev_map: Vec<Node>,
    pub fri_exp_id: i32,
    pub n_exps: i32,

    pub cm_n: Vec<i32>,
    pub cm_2ns: Vec<i32>,
    pub exps_n: Vec<i32>,
    pub exps_2ns: Vec<i32>,
    pub qs: Vec<i32>,

    pub map_sections: SectionVec,
    pub map_sectionsN1: Section,
    pub map_sectionsN3: Section,
    pub map_sectionsN: Section,
    pub map_offsets: Section,
    pub map_deg: Section,

    pub publics: Vec<Public>,
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
            var_pol_map: Vec::new(),
            pu_ctx: Vec::new(),
            pe_ctx: Vec::new(),
            ci_ctx: Vec::new(),
            n_constants: pil.nConstants,
            n_publics: pil.publics.len() as i32,
            n_cm1: pil.nCommitments,
            n_cm2: 0,
            n_cm3: 0,
            n_cm4: 0,
            n_q: 0,
            c_exp: 0,
            ev_map: Vec::new(),
            fri_exp_id: 0,
            n_exps: 0,

            cm_n: Vec::new(),
            cm_2ns: Vec::new(),
            exps_n: Vec::new(),
            exps_2ns: Vec::new(),
            qs: Vec::new(),
            map_sections: SectionVec::default(),
            map_sectionsN1: Section::default(),
            map_sectionsN3: Section::default(),
            map_sectionsN: Section::default(),
            map_offsets: Section::default(),
            map_deg: Section::default(),

            publics: Vec::new(),
        };

        let mut program = Program {
            publics_code: vec![],
            step2prev: Segment::default(),
            step3prev: Segment::default(),
            step4: Segment::default(),
            step42ns: Segment::default(),
            step52ns: Segment::default(),
            verifier_code: Segment::default(),
            verifier_query_code: Segment::default(),
        };

        let mut ctx = Context {
            calculated: Calculated {
                exps: vec![],
                exps_prime: vec![],
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };
        info.generate_pubulic_calculators(&mut ctx, pil, &mut program)?;
        println!("generate_step2");
        info.generate_step2(&mut ctx, pil, &mut program)?; // H1, H2
        info.n_cm2 = pil.nCommitments - info.n_cm1;

        println!("generate_step3");
        info.generate_step3(&mut ctx, pil, &mut program)?; // Z Polynonmial and LC of the permutation checks
        info.n_cm3 = pil.nCommitments - info.n_cm1 - info.n_cm2;

        let mut ctx2ns = Context {
            calculated: Calculated {
                exps: Vec::new(),
                exps_prime: Vec::new(),
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };

        let mut ctx = Context {
            calculated: Calculated {
                exps: vec![],
                exps_prime: vec![],
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };
        println!("generate_constraint_polynomial");

        info.generate_constraint_polynomial(&mut ctx, &mut ctx2ns, pil, &mut program)?;
        info.n_cm4 = pil.nCommitments - info.n_cm1 - info.n_cm2 - info.n_cm3;
        info.n_q = pil.nQ;

        let mut ctx = Context {
            calculated: Calculated {
                exps: vec![],
                exps_prime: vec![],
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };
        println!("generate_constraint_polynomial_verifier");
        info.generate_constraint_polynomial_verifier(&mut ctx, pil, &mut program)?;
        println!("generate_fri_polynomial");
        info.generate_fri_polynomial(&mut ctx2ns, pil, &mut program)?;

        let mut ctx = Context {
            calculated: Calculated {
                exps: vec![],
                exps_prime: vec![],
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };
        println!("generate_fri_verifier");
        info.generate_fri_verifier(&mut ctx, pil, &mut program)?;

        let mut ctx = Context {
            calculated: Calculated {
                exps: vec![],
                exps_prime: vec![],
            },
            tmp_used: 0,
            code: vec![],
            calculated_mark: HashMap::new(),
            exp_id: -1,
        };
        println!("map");
        info.map(&mut ctx, pil, &stark_struct, &mut program)?;

        info.publics = pil.publics.clone();

        Ok(info)
    }

    pub fn generate_pubulic_calculators(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        let publics = pil.publics.clone();
        for p in publics.iter() {
            if p.polType.as_str() == "imP" {
                pil_code_gen(ctx, pil, p.polId, false, "")?;
                let mut segment = build_code(ctx, pil);

                let mut ctx_f = ContextF {
                    exp_map: HashMap::new(),
                    tmp_used: segment.tmp_used,
                    ev_idx: EVIdx::new(),
                    dom: "".to_string(),
                    starkinfo: self,
                };

                let fix_ref = |r: &mut Node, ctx: &mut ContextF, pil: &mut PIL| {
                    let p = if r.prime { 1 } else { 0 };
                    if r.type_.as_str() == "exp" {
                        if ctx.exp_map.get(&(p, r.id)).is_none() {
                            ctx.exp_map.insert((p, r.id), ctx.tmp_used);
                            ctx.tmp_used += 1;
                        }
                        r.prime = false;
                        r.type_ = "tmp".to_string();
                        r.id = *ctx.exp_map.get(&(p, r.id)).unwrap();
                    }
                };
                iterate_code(&mut segment, fix_ref, &mut ctx_f, pil);

                segment.tmp_used = ctx_f.tmp_used;
                program.publics_code.push(segment);
                ctx.calculated = Calculated {
                    exps: vec![],
                    exps_prime: vec![],
                };
            }
        }
        Ok(())
    }

    pub fn generate_step2(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        let ppi = pil.plookupIdentities.clone();
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
                t_exp.idQ = Some(pil.nQ as i32);
                pil.nQ += 1;
            }

            let t_exp_id = pil.expressions.len() as i32;
            t_exp.keep = Some(true);

            if E::is_nop(&t_exp) {
                panic!("nop {}", format!("{:?}", t_exp));
            }
            pil.expressions.push(t_exp);

            let mut f_exp = E::nop();
            for j in pi.f.as_ref().unwrap().iter() {
                let e = E::exp(j.clone(), None);
                if f_exp == E::nop() {
                    f_exp = e;
                } else {
                    f_exp = E::add(&E::mul(&f_exp, &u), &e);
                }
            }

            let f_exp_id = pil.expressions.len() as i32;
            f_exp.keep = Some(true);
            if E::is_nop(&f_exp) {
                panic!("nop {}", format!("{:?}", f_exp));
            }

            pil.expressions.push(f_exp);

            pil_code_gen(ctx, pil, f_exp_id.clone(), false, "")?;
            pil_code_gen(ctx, pil, t_exp_id.clone(), false, "")?;

            let h1_id = pil.nCommitments;
            pil.nCommitments += 1;
            let h2_id = pil.nCommitments;
            pil.nCommitments += 1;

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

        program.step2prev = build_code(ctx, pil);
        Ok(())
    }
}
