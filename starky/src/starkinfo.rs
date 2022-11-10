use crate::f3g as field;
use std::collections::HashMap;
use crate::types::{StarkStruct, PIL};
use crate::errors::{EigenError, Result};
use crate::starkinfo_codegen::{Context, Calculated, pil_code_gen, build_code, iterate_code, Node, ContextF, Segment};

pub struct StarkInfo {
    var_pol_map: usize,
    pu_ctx: usize,
    pe_ctx: usize,
    ci_ctx: usize,
    n_constants: usize,
    n_publics: usize,
    publics_code: Vec<Segment>,
}

impl StarkInfo {

    fn new(pil: &PIL, stark_struct: &StarkStruct) -> Result<()> {
        let pil_deg = pil.references.values().nth(0).unwrap().polDeg as i32;

        let stark_deg = 2i32.pow(stark_struct.nBits as u32);

        if stark_deg != pil_deg {
            return Err(EigenError::MustEqualDegreeError(stark_deg, pil_deg));
        }

        if stark_struct.nBitsExt != stark_struct.steps[0]["nBits"] {
            return Err(EigenError::MustEqualDegreeError(stark_struct.nBitsExt, stark_struct.steps[0]["nBits"]));
        }

        Ok(())

        //Ok(StarkInfo::new())
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
                    code: &mut segment,
                    tmp_used: 0,
                };

                let f = |r: &mut Node, ctx: &mut ContextF| {
                    let p = if r.prime.is_some()  {1} else {0};
                    if r.type_.as_str() == "exp" {
                        if ctx.exp_map.get(&(p, r.id.unwrap())).is_none() {
                            ctx.exp_map.insert((p, r.id.unwrap()), ctx.code.tmp_used);
                            ctx.code.tmp_used += 1;
                        }
                        r.prime = None;
                        r.type_ = "tmp".to_string();
                        r.id = Some(*ctx.exp_map.get(&(p, r.id.unwrap())).unwrap());
                    }
                };
                iterate_code(f ,&mut ctx_f);

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
