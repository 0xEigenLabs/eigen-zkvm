use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::f3g::F3G;
use crate::helper::get_ks;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo::{CICTX, PECTX};
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Calculated, Context, ContextF, EVIdx, Node,
};
use crate::types::{PolIdentity, PIL};
use std::collections::HashMap;

impl StarkInfo {
    pub fn generate_constraint_polynomial_verifier(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        pil_code_gen(ctx, pil, self.c_exp, false, "correctQ")?;

        let mut code = build_code(ctx, pil);

        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: code.tmp_used,
            ev_idx: EVIdx::new(),
            dom: "".to_string(),
            starkinfo: self,
        };

        let fix_ref = |r: &mut Node, ctx: &mut ContextF, pil: &mut PIL| {
            let p = if r.prime { 1 } else { 0 };
            let id = r.id;
            match r.type_.as_str() {
                "cm" | "q" | "const" => {
                    if ctx.ev_idx.get(r.type_.as_str(), p, id).is_none() {
                        ctx.ev_idx
                            .set(r.type_.as_str(), p, id, ctx.starkinfo.ev_map.len() as i32);
                        ctx.starkinfo.ev_map.push(Node::new(
                            r.type_.clone(),
                            r.id,
                            None,
                            -1,
                            r.prime,
                            -1,
                        ));
                        r.prime = false;
                        r.id = *ctx.ev_idx.get(r.type_.as_str(), p, id).unwrap();
                        r.type_ = "eval".to_string();
                    }
                }
                "exp" => {
                    if ctx.exp_map.get(&(p, id)).is_none() {
                        ctx.exp_map.insert((p, id), ctx.tmp_used);
                        ctx.tmp_used += 1;
                    }
                    r.prime = false;
                    r.type_ = "tmp".to_string();
                    r.id = *ctx.exp_map.get(&(p, id)).unwrap();
                }

                "number" | "challenge" | "public" | "tmp" | "Z" | "x" | "eval" => {}
                _ => panic!("{}", format!("Invalid reference type: {}", r.type_)),
            }
        };
        iterate_code(&mut code, fix_ref, &mut ctx_f, pil);
        code.tmp_used = ctx.tmp_used;
        program.verifier_code = code;
        Ok(())
    }
}
