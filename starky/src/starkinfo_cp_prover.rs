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

        self.c_exp = pil.expressions.len() as i32;

        if E::is_nop(&c_exp) {
            panic!("nop {:?}", format!("{:?}", c_exp));
        }
        pil.expressions.push(c_exp);
        println!(
            "expressions[3] {:?}",
            serde_json::to_string(&pil.expressions[3])
        );

        for i in 0..pil.expressions.len() {
            println!("expressions {:?} {:?}", i, pil.expressions.len());
            if pil.expressions[i].idQ.is_some() {
                pil_code_gen(ctx, pil, i as i32, false, "")?;
                pil_code_gen(ctx2ns, pil, i as i32, false, "evalQ")?;
            }
        }

        program.step4 = build_code(ctx, pil);
        program.step42ns = build_code(ctx2ns, pil);
        Ok(())
    }
}
