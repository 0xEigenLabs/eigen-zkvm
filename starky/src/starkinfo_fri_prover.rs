use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context, Node};
use crate::types::PIL;

impl StarkInfo {
    pub fn generate_fri_polynomial(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
    ) -> Result<()> {
        let vf1 = E::challenge("vf1".to_string());
        let vf2 = E::challenge("vf2".to_string());

        let mut fri_exp = E::nop();
        for i in 0..pil.nCommitments {
            if E::is_nop(&fri_exp) {
                fri_exp = E::cm(i, None);
            } else {
                fri_exp = E::add(&E::mul(&vf1, &fri_exp), &E::cm(i, None));
            }
        }

        //println!("fri_exp {}", fri_exp);
        let mut fri1_exp = E::nop();
        let mut fri2_exp = E::nop();
        //println!("ev_map: {}", serde_json::to_string_pretty(&self.ev_map).unwrap());
        for (i, ev) in self.ev_map.iter().enumerate() {
            let mut fri_exp = match ev.prime {
                true => fri2_exp.clone(),
                false => fri1_exp.clone(),
            };
            let ev_id = ev.id;
            let e = match ev.type_.as_str() {
                "cm" => E::cm(ev_id, None),
                "q" => E::q(ev_id, None),
                "const" => E::const_(ev_id, None),
                _ => panic!("Invalid exp op {}", ev.type_),
            };
            if !E::is_nop(&fri_exp) {
                fri_exp = E::add(&E::mul(&fri_exp, &vf2), &E::sub(&e, &E::eval(i)));
            } else {
                fri_exp = E::sub(&e, &E::eval(i));
            }

            if ev.prime {
                fri2_exp = fri_exp;
            } else {
                fri1_exp = fri_exp;
            }
        }

        //println!("fri1exp {}", fri1_exp);
        //println!("fri2exp {}", fri2_exp);

        if !E::is_nop(&fri_exp) {
            fri1_exp = E::mul(&fri1_exp, &E::xDivXSubXi());
            if !E::is_nop(&fri_exp) {
                fri_exp = E::add(&E::mul(&vf1, &fri_exp), &fri1_exp);
            } else {
                fri_exp = fri1_exp;
            }
        }

        if !E::is_nop(&fri2_exp) {
            fri2_exp = E::mul(&fri2_exp, &E::xDivXSubWXi());
            if !E::is_nop(&fri_exp) {
                fri_exp = E::add(&E::mul(&vf1, &fri_exp), &fri2_exp);
            } else {
                fri_exp = fri2_exp;
            }
        }

        //println!("fri_exp {}", fri_exp);
        self.fri_exp_id = pil.expressions.len();
        fri_exp.keep2ns = Some(true);
        if E::is_nop(&fri_exp) {
            panic!("nop {:?}", format!("{:?}", fri_exp));
        }
        pil.expressions.push(fri_exp);

        pil_code_gen(ctx, pil, self.fri_exp_id, false, "f", 0)?;
        let sz = ctx.code.len() - 1;
        let code = &mut ctx.code[sz].code;
        let sz = code.len() - 1;
        code[sz].dest = Node::new("f".to_string(), 0, None, 0, false, 0);

        program.step52ns = build_code(ctx, pil);
        //println!("step52ns:{}", program.step52ns);
        Ok(())
    }
}
