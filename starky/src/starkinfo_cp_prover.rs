use crate::expressionops::ExpressionOps as E;
use crate::f3g::F3G;
use crate::helper::get_ks;
use crate::starkinfo::StarkInfo;
use crate::starkinfo::{CICTX, PECTX};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Calculated, Context};
use crate::types::PolIdentity;
use std::collections::HashMap;

impl StarkInfo {
    pub fn generate_constraint_polynomial(&mut self, ctx: &mut Context, ctx2ns: &mut Context) {
        let vc = E::challenge("vc".to_string());
        let mut c_exp = E::nop();
        for pi in ctx.pil.polIdentities.iter() {
            let e = E::exp(pi.e, None);

            if E::is_nop(&c_exp) {
                c_exp = e;
            } else {
                c_exp = E::add(&E::mul(&vc, &c_exp), &e);
            }
        }

        c_exp.idQ = Some(ctx.pil.nQ);
        ctx.pil.nQ += 1;

        self.c_exp = ctx.pil.expressions.len() as i32;
        ctx.pil.expressions.push(c_exp);

        let pe = &(ctx.pil.expressions).clone();
        for (i, p) in pe.iter().enumerate() {
            if p.idQ.is_some() {
                pil_code_gen(ctx, i as i32, false, &"".to_string()); // FIXME: prime should be undefined
                pil_code_gen(ctx2ns, i as i32, false, &"evalQ".to_string());
            }
        }

        self.step4 = build_code(ctx);
        self.step42ns = build_code(ctx2ns);
    }
}
