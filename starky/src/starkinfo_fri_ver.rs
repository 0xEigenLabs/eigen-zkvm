use crate::errors::Result;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Context, ContextF, EVIdx, Node,
};
use crate::types::PIL;
use std::collections::HashMap;

impl StarkInfo {
    pub fn generate_fri_verifier(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        pil_code_gen(ctx, pil, self.fri_exp_id, false, "")?;

        let mut code = build_code(ctx, pil);
        self.n_exps = pil.expressions.len() as i32;

        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: code.tmp_used,
            ev_idx: EVIdx::new(),
            dom: "".to_string(),
            starkinfo: self,
        };

        let fix_ref = |r: &mut Node, ctx: &mut ContextF, pil: &mut PIL| match r.type_.as_str() {
            "cm" | "q" | "const" => {}
            "exp" => {
                let p = if r.prime { 1 } else { 0 };
                let id = r.id;
                if ctx.exp_map.get(&(p, id)).is_none() {
                    ctx.exp_map.insert((p, id), ctx.tmp_used);
                    ctx.tmp_used += 1;
                }
                r.prime = false;
                r.type_ = "tmp".to_string();
                r.id = *ctx.exp_map.get(&(p, id)).unwrap();
            }

            "number" | "challenge" | "public" | "tmp" | "xDivXSubXi" | "xDivXSubWXi" | "Z"
            | "x" | "eval" | "tree1" | "tree2" | "tree3" | "tree3" => {}

            _ => panic!("{}", format!("Invalid reference type: {}", r.type_)),
        };
        iterate_code(&mut code, fix_ref, &mut ctx_f, pil);
        code.tmp_used = ctx.tmp_used;
        program.verifier_query_code = code;

        Ok(())
    }
}
