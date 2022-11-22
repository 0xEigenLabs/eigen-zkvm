use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context};
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

        println!("fri_exp 1 {:?}", fri_exp);
        for i in 0..pil.nQ {
            if E::is_nop(&fri_exp) {
                fri_exp = E::q(i, None);
            } else {
                fri_exp = E::add(&E::mul(&vf1, &fri_exp), &E::q(i, None));
            }
        }

        println!("fri_exp 2 {:?}", fri_exp);
        let mut fri1_exp = E::nop();
        let mut fri2_exp = E::nop();
        //let xi = E::challenge("xi".to_string());
        println!("ev_map {:?}", self.ev_map);

        for (i, ev) in self.ev_map.iter().enumerate() {
            let mut fri_exp = match ev.prime {
                true => fri2_exp.clone(),
                _ => fri1_exp.clone(),
            };
            let ev_id = ev.id;
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

            if ev.prime {
                fri2_exp = fri_exp;
            } else {
                fri1_exp = fri_exp;
            }
        }

        println!("fri_exp 3 {:?}", fri_exp);

        fri1_exp = E::mul(&fri1_exp, &E::xDivXSubXi());
        if !E::is_nop(&fri_exp) {
            fri_exp = E::add(&E::mul(&vf1, &fri_exp), &fri1_exp);
        } else {
            fri_exp = fri1_exp;
        }

        println!("fri_exp 4 {:?}", fri_exp);

        fri2_exp = E::mul(&fri2_exp, &E::xDivXSubWXi());
        if !E::is_nop(&fri_exp) {
            fri_exp = E::add(&E::mul(&vf1, &fri_exp), &fri2_exp);
        } else {
            fri_exp = fri2_exp;
        }

        println!("fri_exp {:?}", fri_exp);
        self.fri_exp_id = pil.expressions.len() as i32;
        fri_exp.keep2ns = Some(true);
        if E::is_nop(&fri_exp) {
            panic!("nop {:?}", format!("{:?}", fri_exp));
        }
        pil.expressions.push(fri_exp);

        println!(
            "pil_code_gen in fri prover {:?}",
            pil.expressions[self.fri_exp_id as usize]
        );

        pil_code_gen(ctx, pil, self.fri_exp_id, false, "")?;
        program.step52ns = build_code(ctx, pil);
        Ok(())
    }
}
