use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context};
use crate::types::PIL;

impl StarkInfo {
    pub fn generate_constraint_polynomial(
        &mut self,
        ctx: &mut Context,
        ctx2ns: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        let vc = E::challenge("vc".to_string());
        let mut c_exp = E::nop();
        for pi in pil.polIdentities.iter() {
            let e = E::exp(pi.e, None);
            if E::is_nop(&c_exp) {
                c_exp = e;
            } else {
                c_exp = E::add(&E::mul(&vc, &c_exp), &e);
            }
        }

        c_exp.idQ = Some(pil.nQ);
        pil.nQ += 1;
        //println!(
        //    "generate_constraint_polynomial: c_exp: {}, pil.nQ: {:?}",
        //    c_exp, pil.nQ
        //);
        self.c_exp = pil.expressions.len();

        if E::is_nop(&c_exp) {
            panic!("nop {:?}", format!("{:?}", c_exp));
        }
        pil.expressions.push(c_exp);

        for i in 0..pil.expressions.len() {
            if pil.expressions[i].idQ.is_some() {
                pil_code_gen(ctx, pil, i, false, "")?;
                pil_code_gen(ctx2ns, pil, i, false, "evalQ")?;
            }
        }

        program.step4 = build_code(ctx, pil);
        program.step42ns = build_code(ctx2ns, pil);
        //println!(
        //    "generate_constraint_polynomial: step4: {:?}, step42ns: {:?}",
        //    program.step4, program.step42ns
        //);
        Ok(())
    }
}
