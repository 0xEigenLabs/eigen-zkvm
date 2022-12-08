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
        //println!("cp ver begin ctx {:?}, c_exp: {}", ctx, self.c_exp);
        pil_code_gen(ctx, pil, self.c_exp, false, "", 0)?;

        //println!("cp ver buildcode ctx begin {:?}", ctx);
        let mut code = build_code(ctx, pil);
        //println!("cp ver buildcode {}", code);

        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: code.tmp_used,
            dom: "".to_string(),
            starkinfo: self,
        };
        //println!("cp ver code.tmp_used begin {}", code.tmp_used);

        let fix_ref = |r: &mut Node, ctx: &mut ContextF, _pil: &mut PIL| {
            let p = if r.prime { 1 } else { 0 };
            let id = r.id;
            match r.type_.as_str() {
                "exp" => {
                    let idx = ctx.starkinfo.im_exps_list.iter().position(|&s| s == r.id);
                    if idx.is_some() {
                        let idx = idx.unwrap();
                        r.type_ = "cm".to_string();
                        r.id = ctx.starkinfo.im_exp2cm[&ctx.starkinfo.im_exps_list[idx]];

                        // go to cm branch, TODO
                        if ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, id).is_none() {
                            ctx.starkinfo.ev_idx.set(
                                r.type_.as_str(),
                                p,
                                id,
                                ctx.starkinfo.ev_map.len(),
                            );
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
                        r.id = *ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, id).unwrap();
                        r.type_ = "eval".to_string();
                    } else {
                        let p = if r.prime { 1 } else { 0 };
                        if ctx.exp_map.get(&(p, id)).is_none() {
                            ctx.exp_map.insert((p, id), ctx.tmp_used);
                            ctx.tmp_used += 1;
                        }
                        r.type_ = "tmp".to_string();
                        r.exp_id = r.id;
                        r.id = *ctx.exp_map.get(&(p, id)).unwrap();
                    }
                }
                "cm" | "const" => {
                    if ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, id).is_none() {
                        ctx.starkinfo.ev_idx.set(
                            r.type_.as_str(),
                            p,
                            id,
                            ctx.starkinfo.ev_map.len(),
                        );
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
                    r.id = *ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, id).unwrap();
                    r.type_ = "eval".to_string();
                }
                "number" | "challenge" | "public" | "tmp" | "Z" | "x" | "eval" => {}
                _ => panic!("Invalid reference type: {:?}", r),
            };
            //println!("ev_map: {:?}", ctx.starkinfo.ev_map);
        };

        iterate_code(&mut code, fix_ref, &mut ctx_f, pil);

        //println!("q_deg: {}", ctx_f.starkinfo.q_deg);
        for i in 0..ctx_f.starkinfo.q_deg {
            ctx_f.starkinfo.ev_idx.set(
                "cm",
                0,
                ctx_f.starkinfo.qs[i],
                ctx_f.starkinfo.ev_map.len(),
            );
            let rf = Node::new("cm".to_string(), ctx_f.starkinfo.qs[i], None, 0, false, 0);
            ctx_f.starkinfo.ev_map.push(rf);
        }

        code.tmp_used = ctx_f.tmp_used;
        //println!("ev_idx: {:?}", ctx_f.starkinfo.ev_idx);
        //println!("ev_map: {:?}", ctx_f.starkinfo.ev_map);
        //println!("cp ver code.tmp_used {}", code.tmp_used);
        println!("cp ver code {}", code);
        program.verifier_code = code;
        Ok(())
    }
}
