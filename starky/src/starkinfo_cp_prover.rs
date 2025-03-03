use crate::expressionops::ExpressionOps as E;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::Section;
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context, Node};
use crate::types::Expression;
use crate::types::{StarkStruct, PIL};
use anyhow::Result;
use std::collections::HashMap;

impl StarkInfo {
    #[allow(clippy::unnecessary_unwrap)]
    pub fn generate_constraint_polynomial(
        &mut self,
        ctx: &mut Context,
        ctx2ns: &mut Context,
        pil: &mut PIL,
        stark_struct: &StarkStruct,
        program: &mut Program,
    ) -> Result<()> {
        //log::trace!(
        //    "generate_constraint_polynomial ctx begin: {} {:?}",
        //    pil,
        //    ctx
        //);

        let vc = E::challenge("vc".to_string());
        let mut c_exp = E::nop();
        for pi in pil.polIdentities.iter() {
            let e = E::exp(pi.e, None);
            if !E::is_nop(&c_exp) {
                c_exp = E::add(&E::mul(&vc, &c_exp), &e);
            } else {
                c_exp = e;
            }
        }
        self.q_deg = 0;
        let max_deg = (1 << (stark_struct.nBitsExt - stark_struct.nBits)) + 1;
        for d in 2..=max_deg {
            let (im_exps, q_deg) = calculate_im_pols(pil, &c_exp, d)?;
            if im_exps.is_some()
                && (self.q_deg == 0
                    || (im_exps.as_ref().unwrap().len() + (q_deg as usize)
                        < self.im_exps.len() + self.q_deg))
            {
                self.q_deg = q_deg as usize;
                self.im_exps = im_exps.unwrap();
            }
        }

        //log::trace!("im_exps: {:?} q_deg {}", self.im_exps, self.q_deg);

        for k in self.im_exps.keys() {
            self.im_exps_list.push(*k);
        }
        // NOTE: sort im_exps_list due to map is not ordered
        self.im_exps_list.sort();

        self.im_exp2cm = HashMap::new();
        for i in 0..self.im_exps_list.len() {
            self.im_exp2cm.insert(self.im_exps_list[i], pil.nCommitments);
            pil.nCommitments += 1;

            let lhs = pil.expressions[self.im_exps_list[i]].clone();
            let rhs = Expression::new("cm".to_string(), 0, Some(pil.nCommitments - 1), None, None);
            let e = Expression::new("sub".to_string(), 0, None, None, Some(vec![lhs, rhs]));
            if !E::is_nop(&c_exp) {
                c_exp = E::add(&E::mul(&vc, &c_exp), &e);
            } else {
                c_exp = e;
            }
        }

        //log::trace!(
        //    "generate_constraint_polynomial: c_exp: {}, pil.nQ: {:?}, im_exp2cm: {:?}, im_exps_list :{:?}",
        //    c_exp, pil.nQ, self.im_exp2cm, self.im_exps_list
        //);
        self.c_exp = pil.expressions.len();
        pil.expressions.push(c_exp);

        self.n_cm3 = pil.nCommitments - self.n_cm1 - self.n_cm2;
        self.qs = vec![0usize; self.q_deg];

        for i in 0..self.q_deg {
            self.qs[i] = pil.nCommitments;
            pil.nCommitments += 1;
        }

        for i in 0..self.im_exps_list.len() {
            pil_code_gen(ctx, pil, self.im_exps_list[i], false, "", 0, false)?;
        }

        program.step3 = build_code(ctx, pil);
        //log::trace!("generate_constraint_polynomial: step3: {}", program.step3);

        for (k, v) in self.im_exps.iter() {
            ctx2ns.calculated.insert(("exps", *k), *v);
            ctx2ns.calculated.insert(("expsPrime", *k), *v);
        }
        //log::trace!("ctx2ns: {} {:?}", pil, ctx2ns);
        pil_code_gen(ctx2ns, pil, self.c_exp, false, "", 0, false)?;

        let sz = ctx2ns.code.len() - 1;
        let code = &mut ctx2ns.code[sz].code;

        let sz = code.len() - 1;
        code.push(Section {
            op: "mul".to_string(),
            dest: Node::new("q".to_string(), 0, None, 0, false, 0),
            src: vec![code[sz].dest.clone(), Node::new("Zi".to_string(), 0, None, 0, false, 0)],
        });

        program.step42ns = build_code(ctx2ns, pil);
        self.n_cm4 = self.q_deg;
        //log::trace!(
        //    "generate_constraint_polynomial: step42ns: {}",
        //    program.step42ns
        //);
        Ok(())
    }
}

#[allow(clippy::if_same_then_else)]
fn _calculate_im_pols(
    pil: &mut PIL,
    exp: &Expression,
    im_expressions: &Option<HashMap<usize, bool>>,
    max_deg: usize,
    abs_max: usize,
    abs_max_d: &mut i32,
) -> (Option<HashMap<usize, bool>>, i32) {
    //log::trace!(
    //    "im_expressions: {:?}, exp: {}, max_deg {}",
    //    im_expressions,
    //    exp,
    //    max_deg
    //);
    if im_expressions.is_none() {
        return (None, -1);
    }
    //log::trace!("_calculate_im_pols: {}", exp.op);
    if ["add", "sub", "addc", "mulc", "neg"].contains(&exp.op.as_str()) {
        let mut md = 0;
        #[allow(unused_assignments)]
        let mut d: i32 = 0;
        let mut im_e: Option<HashMap<usize, bool>> = im_expressions.clone();
        let values: &Vec<Expression> = exp.values.as_ref().unwrap();
        for v in values.iter() {
            (im_e, d) = _calculate_im_pols(pil, v, &im_e, max_deg, abs_max, abs_max_d);
            if d > md {
                md = d;
            }
        }
        (im_e, md)
    } else if ["number", "public", "challenge"].contains(&exp.op.as_str()) {
        return (im_expressions.clone(), 0);
    } else if ["x", "const", "cm"].contains(&exp.op.as_str()) {
        if max_deg < 1 {
            return (None, -1);
        }
        return (im_expressions.clone(), 1);
    } else if exp.op.as_str() == "mul" {
        let mut eb: Option<HashMap<usize, bool>> = None;
        let mut ed = -1;
        let values: &Vec<Expression> = exp.values.as_ref().unwrap();
        // TODO explain
        if ["number", "public", "challenge"].contains(&values[0].op.as_str()) {
            return _calculate_im_pols(
                pil,
                &(values[1]),
                im_expressions,
                max_deg,
                abs_max,
                abs_max_d,
            );
        }
        if ["number", "public", "challenge"].contains(&values[1].op.as_str()) {
            return _calculate_im_pols(
                pil,
                &(values[0]),
                im_expressions,
                max_deg,
                abs_max,
                abs_max_d,
            );
        }
        let max_deg_here = get_exp_dim(pil, exp);
        if max_deg_here <= (max_deg as i32) {
            return (im_expressions.clone(), max_deg_here);
        }
        for l in 0..=max_deg {
            let r = max_deg - l;
            let (e1, d1) =
                _calculate_im_pols(pil, &(values[0]), im_expressions, l, abs_max, abs_max_d);
            let (e2, d2) = _calculate_im_pols(pil, &(values[1]), &e1, r, abs_max, abs_max_d);
            if e2.is_some() {
                if eb.is_none() {
                    eb = e2;
                    ed = d1 + d2;
                } else if e2.as_ref().unwrap().len() < eb.as_ref().unwrap().len() {
                    eb = e2;
                    ed = d1 + d2;
                }
            }
            if eb.is_some()
                && im_expressions.is_some()
                && eb.as_ref().unwrap().len() == im_expressions.as_ref().unwrap().len()
            {
                return (eb, ed);
            }
        }
        return (eb, ed);
    } else if exp.op.as_str() == "exp" {
        if max_deg < 1 {
            return (None, -1);
        }

        if im_expressions.is_some()
            && im_expressions.as_ref().unwrap().get(&exp.id.unwrap()).is_some()
        {
            return (im_expressions.clone(), 1);
        }
        let exp_n = pil.expressions[exp.id.unwrap()].clone();
        let (e, d) = _calculate_im_pols(pil, &exp_n, im_expressions, abs_max, abs_max, abs_max_d);
        if e.is_none() {
            return (None, -1);
        }

        let mut e = e.unwrap();
        if d > (max_deg as i32) {
            e.insert(exp.id.unwrap(), true);
            if d > *abs_max_d {
                *abs_max_d = d;
            }
            return (Some(e), 1);
        } else {
            return (Some(e), d);
        }
    } else {
        panic!("Exp op not defined: {}", exp.op);
    }
}

pub fn get_exp_dim(pil: &PIL, exp: &Expression) -> i32 {
    let values: Vec<Expression> =
        if exp.values.is_none() { Vec::new() } else { exp.values.clone().unwrap() };
    match exp.op.as_str() {
        "add" | "sub" | "addc" | "mulc" | "neg" => {
            let mut md = 1;
            for vi in &values {
                let d = get_exp_dim(pil, vi);
                if d > md {
                    md = d;
                }
            }
            md
        }
        "mul" => get_exp_dim(pil, &values[0]) + get_exp_dim(pil, &values[1]),
        "muladd" => std::cmp::max(
            get_exp_dim(pil, &values[0]) + get_exp_dim(pil, &values[1]),
            get_exp_dim(pil, &values[2]),
        ),
        "cm" | "const" | "x" => 1,
        "exp" => get_exp_dim(pil, &pil.expressions[exp.id.unwrap()]),
        "number" | "public" | "challenge" | "eval" => 0,
        _ => panic!("Exp op not defined: {}", exp.op),
    }
}

pub fn calculate_im_pols(
    pil: &mut PIL,
    exp: &Expression,
    max_deg: usize,
) -> Result<(Option<HashMap<usize, bool>>, i32)> {
    //log::trace!("calculate_im_pols: {} {}", exp, max_deg);

    let mut abs_max_d = 0;
    let im_expressions: HashMap<usize, bool> = HashMap::new();
    let (re, rd) =
        _calculate_im_pols(pil, exp, &Some(im_expressions), max_deg, max_deg, &mut abs_max_d);

    //log::trace!(
    //    "maxDeg: {}, nIm: {}, d: {}",
    //    max_deg,
    //    re.as_ref().unwrap().len(),
    //    rd
    //);
    Ok((re, std::cmp::max(rd, abs_max_d) - 1))
}
