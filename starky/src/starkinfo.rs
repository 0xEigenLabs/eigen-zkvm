use crate::errors::{EigenError, Result};
use crate::f3g as field;
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Calculated, Context, ContextF, Node, Segment,
};
use crate::types::{StarkStruct, PIL};
use std::collections::HashMap;

#[derive(Debug)]
pub struct StarkInfo {
    var_pol_map: usize,
    pu_ctx: usize,
    pe_ctx: usize,
    ci_ctx: usize,
    n_constants: i32,
    n_publics: i32,
    publics_code: Vec<Segment>,
}

impl StarkInfo {
    pub fn new(pil: &PIL, stark_struct: &StarkStruct) -> Result<StarkInfo> {
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
            pu_ctx: 0,
            pe_ctx: 0,
            ci_ctx: 0,
            n_constants: pil.nConstants,
            n_publics: pil.publics.len() as i32,
            publics_code: vec![],
        };
        info.generate_pubulic_calculators(pil);
        println!("{:?}", info);
        Ok(info)
    }

    pub fn generate_pubulic_calculators(&mut self, pil: &PIL) -> Result<()> {
        for p in pil.publics.iter() {
            if p.polType.as_str() == "imP" {
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
                pil_code_gen(&mut ctx, p.polId, false, &"".to_string());
                let mut segment = build_code(&mut ctx);

                let mut ctx_f = ContextF {
                    exp_map: HashMap::new(),
                    tmp_used: segment.tmp_used,
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
}
