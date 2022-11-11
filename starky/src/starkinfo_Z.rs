use crate::expressionops::ExpressionOps as E;
use crate::helper::get_ks;
use crate::starkinfo::StarkInfo;
use crate::starkinfo::PECTX;
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context};

impl StarkInfo {
    pub fn generate_permutation_LC(&mut self, ctx: &mut Context) {
        let ppi = match &ctx.pil.permutationIdentities {
            Some(x) => x.clone(),
            _ => Vec::new(),
        };
        for pi in ppi.iter() {
            let mut t_exp = E::nop();
            let u = E::challenge("u".to_string());
            let def_val = E::challenge("defVal".to_string());
            for j in pi.t.as_ref().unwrap().iter() {
                let e = E::exp(*j, None);
                if E::is_nop(&t_exp) {
                    t_exp = e;
                } else {
                    t_exp = E::add(&E::mul(&u, &t_exp), &e)
                }
            }

            if pi.selT.is_some() {
                t_exp = E::sub(&t_exp, &def_val);
                t_exp = E::mul(&t_exp, &E::exp(pi.selT.unwrap(), None));
                t_exp = E::add(&t_exp, &def_val);
                t_exp.idQ = Some(ctx.pil.nQ);
                ctx.pil.nQ += 1;
            }

            let t_exp_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(t_exp);

            let mut f_exp = E::nop();
            for j in pi.f.as_ref().unwrap().iter() {
                let e = E::exp(*j, None);
                if E::is_nop(&f_exp) {
                    f_exp = e;
                } else {
                    f_exp = E::add(&E::mul(&f_exp, &u), &e);
                }
            }

            if pi.selF.is_some() {
                f_exp = E::sub(&f_exp, &def_val);
                f_exp = E::mul(&f_exp, &E::exp(pi.selF.unwrap(), None));
                f_exp = E::add(&f_exp, &def_val);
                f_exp.idQ = Some(ctx.pil.nQ);
                ctx.pil.nQ += 1;
            }

            let f_exp_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(f_exp);

            pil_code_gen(ctx, f_exp_id.clone(), false, &"".to_string());
            pil_code_gen(ctx, t_exp_id.clone(), false, &"".to_string());

            self.pe_ctx.push(PECTX { f_exp_id, t_exp_id });
        }
    }
}
