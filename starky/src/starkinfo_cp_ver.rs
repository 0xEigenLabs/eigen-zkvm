use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{build_code, iterate_code, pil_code_gen, Context, ContextF, Node};
use crate::types::PIL;
use anyhow::Result;
use std::collections::HashMap;

impl StarkInfo {
    pub fn generate_constraint_polynomial_verifier(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        //log::trace!("cp ver begin ctx {:?}, c_exp: {}", ctx, self.c_exp);
        pil_code_gen(ctx, pil, self.c_exp, false, "", 0, true)?;

        //log::trace!("cp ver buildcode ctx begin {:?}", ctx);
        let mut code = build_code(ctx, pil);
        //log::trace!("cp ver buildcode {}", code);

        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: code.tmp_used,
            dom: "".to_string(),
            tmpexps: &mut HashMap::new(),
            starkinfo: self,
        };
        //log::trace!("cp ver code.tmp_used begin {}", code.tmp_used);

        let fix_ref = |r: &mut Node, ctx: &mut ContextF, _pil: &mut PIL| {
            let p = if r.prime { 1 } else { 0 };
            match r.type_.as_str() {
                "exp" => {
                    let idx = ctx.starkinfo.im_exps_list.iter().position(|&s| s == r.id);
                    if let Some(idx) = idx {
                        r.type_ = "cm".to_string();
                        r.id = ctx.starkinfo.im_exp2cm[&ctx.starkinfo.im_exps_list[idx]];

                        // go to cm branch, TODO
                        if ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, r.id).is_none() {
                            ctx.starkinfo.ev_idx.set(
                                r.type_.as_str(),
                                p,
                                r.id,
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
                        r.id = *ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, r.id).unwrap();
                        r.type_ = "eval".to_string();
                    } else {
                        let p = if r.prime { 1 } else { 0 };
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            ctx.exp_map.entry((p, r.id))
                        {
                            e.insert(ctx.tmp_used);
                            ctx.tmp_used += 1;
                        }

                        r.type_ = "tmp".to_string();
                        r.exp_id = r.id;
                        r.id = *ctx.exp_map.get(&(p, r.id)).unwrap();
                    }
                }
                "cm" | "const" => {
                    if ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, r.id).is_none() {
                        ctx.starkinfo.ev_idx.set(
                            r.type_.as_str(),
                            p,
                            r.id,
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
                    r.id = *ctx.starkinfo.ev_idx.get(r.type_.as_str(), p, r.id).unwrap();
                    r.type_ = "eval".to_string();
                }
                "number" | "challenge" | "public" | "tmp" | "Z" | "x" | "eval" => {}
                _ => panic!("Invalid reference type: {:?}", r),
            };
        };

        iterate_code(&mut code, fix_ref, &mut ctx_f, pil);

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
        //log::trace!("ev_idx: {:?}", ctx_f.starkinfo.ev_idx);
        //log::trace!("ev_map: {:?}", ctx_f.starkinfo.ev_map);
        //log::trace!("cp ver code.tmp_used {}", code.tmp_used);
        //log::trace!("cp ver code {}", code);
        program.verifier_code = code;
        Ok(())
    }
}
