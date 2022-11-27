use crate::errors::Result;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Context, ContextF, EVIdx, Node,
};
use crate::types::PIL;
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
        //println!("cp ver buildcode {}", code);

        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: code.tmp_used,
            ev_idx: EVIdx::new(),
            dom: "".to_string(),
            starkinfo: self,
        };
        //println!("cp ver code.tmp_used begin {}", code.tmp_used);

        let fix_ref = |r: &mut Node, ctx: &mut ContextF, _pil: &mut PIL| {
            let p = if r.prime { 1 } else { 0 };
            let id = r.id;
            match r.type_.as_str() {
                "cm" | "q" | "const" => {
                    if ctx.ev_idx.get(r.type_.as_str(), p, id).is_none() {
                        ctx.ev_idx
                            .set(r.type_.as_str(), p, id, ctx.starkinfo.ev_map.len());
                        ctx.starkinfo.ev_map.push(Node::new(
                            r.type_.clone(),
                            r.id,
                            None,
                            0,
                            r.prime,
                            0,
                        ));
                    }
                    r.prime = false; // NOTE: js: delete r.prime
                    r.id = *ctx.ev_idx.get(r.type_.as_str(), p, id).unwrap();
                    r.type_ = "eval".to_string();
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
            };
            //println!("ev_map: {:?}", ctx.starkinfo.ev_map);
        };

        iterate_code(&mut code, fix_ref, &mut ctx_f, pil);
        code.tmp_used = ctx_f.tmp_used;
        //println!("ev_idx: {:?}", ctx_f.ev_idx);
        //println!("cp ver code.tmp_used {}", code.tmp_used);
        //println!("cp ver code {}", code);
        program.verifier_code = code;
        Ok(())
    }
}
