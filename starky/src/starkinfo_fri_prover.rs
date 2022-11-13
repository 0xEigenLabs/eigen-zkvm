use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::f3g::F3G;
use crate::helper::get_ks;
use crate::starkinfo::StarkInfo;
use crate::starkinfo::{CICTX, PECTX};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Calculated, Context};
use crate::types::PolIdentity;
use std::collections::HashMap;

impl StarkInfo {
    pub fn generate_fri_polynomial(&mut self, ctx: &mut Context) -> Result<()> {
        let vf1 = E::challenge("vf1".to_string());
        let vf2 = E::challenge("vf2".to_string());

        let mut fri_exp = E::nop();
        for i in 0..ctx.pil.nCommitments {
            if E::is_nop(&fri_exp) {
                fri_exp = E::cm(i, None);
            } else {
                fri_exp = E::add(&E::mul(&vf1, &fri_exp), &E::cm(i, None));
            }
        }

        for i in 0..ctx.pil.nQ {
            if E::is_nop(&fri_exp) {
                fri_exp = E::q(i, None);
            } else {
                fri_exp = E::add(&E::mul(&vf1, &fri_exp), &E::q(i, None));
            }
        }

        let mut fri1_exp = E::nop();
        let mut fri2_exp = E::nop();
        let x1 = E::challenge("xi".to_string());
        for (i, ev) in self.ev_map.iter().enumerate() {
            let mut fri_exp = match ev.prime {
                Some(_) => fri2_exp.clone(),
                None => fri1_exp.clone(),
            };
            let ev_id = ev.id.unwrap();
            let e = match ev.type_.as_str() {
                "cm" => E::cm(ev_id, None),
                "q" => E::q(ev_id, None),
                "const" => E::const_(ev_id, None),
                _ => panic!("Invalid exp op {}", ev.type_),
            };
            if E::is_nop(&fri_exp) {
                fri_exp = E::sub(&e, &E::eval(i as i32));
            } else {
                fri_exp = E::add(&E::mul(&fri_exp, &vf2), &E::sub(&e, &E::eval(i as i32)));
            }

            if ev.prime.is_some() {
                fri2_exp = fri_exp;
            } else {
                fri1_exp = fri_exp;
            }
        }

        fri1_exp = E::mul(&fri1_exp, &E::xDivXSubXi());
        if !E::is_nop(&fri_exp) {
            fri_exp = E::add(&E::mul(&vf1, &fri_exp), &fri1_exp);
        } else {
            fri_exp = fri1_exp;
        }

        fri2_exp = E::mul(&fri2_exp, &E::xDivXSubWXi());
        if !E::is_nop(&fri_exp) {
            fri_exp = E::add(&E::mul(&vf1, &fri_exp), &fri2_exp);
        } else {
            fri_exp = fri2_exp;
        }

        self.fri_exp_id = ctx.pil.expressions.len() as i32;
        fri_exp.keep2ns = Some(true);
        ctx.pil.expressions.push(fri_exp);

        pil_code_gen(ctx, self.fri_exp_id, false, &"".to_string())?;
        self.step52ns = build_code(ctx);
        Ok(())
    }
}
