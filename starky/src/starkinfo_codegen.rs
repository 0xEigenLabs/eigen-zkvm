#![allow(non_snake_case)]
use crate::errors::{EigenError, Result};
use crate::expressionops::ExpressionOps;
use crate::f3g::F3G;
use crate::starkinfo::StarkInfo;
use crate::types::Expression;
use crate::types::PIL;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Calculated {
    pub exps: Vec<bool>,
    pub exps_prime: Vec<bool>,
}

#[derive(Debug)]
pub struct Context {
    pub exp_id: usize,
    pub tmp_used: usize,
    pub code: Vec<Code>,
    pub calculated: Calculated,
    pub calculated_mark: HashMap<(&'static str, usize), bool>,
}

#[derive(Debug)]
pub struct ContextC {
    pub exp_id: usize,
    pub tmp_used: usize,
    pub code: Vec<Section>,
}

#[derive(Debug)]
pub struct ContextF<'a> {
    pub exp_map: HashMap<(usize, usize), usize>,
    pub tmp_used: usize,
    pub ev_idx: EVIdx,
    pub dom: String,

    pub starkinfo: &'a mut StarkInfo,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Node {
    pub type_: String,
    pub id: usize,
    pub value: Option<String>,
    pub dim: usize,
    pub prime: bool,
    pub tree_pos: usize,
    pub p: usize,
    pub exp_id: usize,
}

impl Node {
    pub fn new(
        type_: String,
        id: usize,
        value: Option<String>,
        dim: usize,
        prime: bool,
        tree_pos: usize,
    ) -> Self {
        Node {
            type_: type_,
            id: id,
            value: value,
            dim: dim,
            prime: prime,
            tree_pos: tree_pos,
            p: 0,
            exp_id: 0,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Section {
    pub op: String,
    pub dest: Node,
    pub src: Vec<Node>,
}

#[derive(Debug, Default, Serialize)]
pub struct Segment {
    pub first: Vec<Section>,
    pub i: Vec<Section>,
    pub last: Vec<Section>,
    pub tmp_used: usize,
}

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let obj = json!(self);
        write!(f, "{}", serde_json::to_string_pretty(&obj).unwrap())
    }
}

impl Segment {
    pub fn get_code_mut_by_idx(&mut self, i: usize) -> &mut Vec<Section> {
        match i {
            0 => &mut self.first,
            1 => &mut self.i,
            2 => &mut self.last,
            _ => panic!("invalid code idx: {}", i),
        }
    }

    pub fn is_some(&self) -> bool {
        self.first.len() > 0 || self.i.len() > 0 || self.last.len() > 0
    }
}

#[derive(Debug)]
pub struct Code {
    pub exp_id: usize,
    pub prime: Option<bool>,
    pub tmp_used: usize,
    pub code: Vec<Section>,
    pub idQ: Option<usize>,
}

#[derive(Debug, Default, Serialize)]
pub struct IndexVec {
    pub cm1_n: Vec<usize>,
    pub cm1_2ns: Vec<usize>,
    pub cm2_n: Vec<usize>,
    pub cm2_2ns: Vec<usize>,
    pub cm3_n: Vec<usize>,
    pub cm3_2ns: Vec<usize>,
    pub q_2ns: Vec<usize>,
    pub exps_withq_n: Vec<usize>,
    pub exps_withq_2ns: Vec<usize>,
    pub exps_withoutq_n: Vec<usize>,
    pub exps_withoutq_2ns: Vec<usize>,
}

#[derive(Debug, Default, Serialize)]
pub struct Index {
    pub cm1_n: usize,
    pub cm1_2ns: usize,
    pub cm2_n: usize,
    pub cm2_2ns: usize,
    pub cm3_n: usize,
    pub cm3_2ns: usize,
    pub q_2ns: usize,
    pub exps_withq_n: usize,
    pub exps_withq_2ns: usize,
    pub exps_withoutq_n: usize,
    pub exps_withoutq_2ns: usize,
    pub map_total_n: usize,
}

impl Index {
    pub fn get(&self, name: &str) -> usize {
        match name {
            "cm1_n" => self.cm1_n,
            "cm1_2ns" => self.cm1_2ns,
            "cm2_n" => self.cm2_n,
            "cm2_2ns" => self.cm2_2ns,
            "cm3_n" => self.cm3_n,
            "cm3_2ns" => self.cm3_2ns,
            "q_2ns" => self.q_2ns,
            "exps_withq_n" => self.exps_withq_n,
            "exps_withq_2ns" => self.exps_withq_2ns,
            "exps_withoutq_n" => self.exps_withoutq_n,
            "exps_withoutq_2ns" => self.exps_withoutq_2ns,
            "map_total_n" => self.map_total_n,
            _ => panic!("Invalid name={} in index", name),
        }
    }

    pub fn set(&mut self, name: &str, val: usize) {
        match name {
            "cm1_n" => {
                self.cm1_n = val;
            }
            "cm1_2ns" => {
                self.cm1_2ns = val;
            }
            "cm2_n" => {
                self.cm2_n = val;
            }
            "cm2_2ns" => {
                self.cm2_2ns = val;
            }
            "cm3_n" => {
                self.cm3_n = val;
            }
            "cm3_2ns" => {
                self.cm3_2ns = val;
            }
            "q_2ns" => {
                self.q_2ns = val;
            }
            "exps_withq_n" => {
                self.exps_withq_n = val;
            }
            "exps_withq_2ns" => {
                self.exps_withq_2ns = val;
            }
            "exps_withoutq_n" => {
                self.exps_withoutq_n = val;
            }
            "exps_withoutq_2ns" => {
                self.exps_withoutq_2ns = val;
            }
            "map_total_n" => {
                self.map_total_n = val;
            }
            _ => panic!("Invalid name={} in index", name),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct PolType {
    pub section: String,
    pub section_pos: usize,
    pub dim: usize,
    pub exp_id: usize,
}

#[derive(Debug)]
pub struct Polynom<'a> {
    pub buffer: &'a mut Vec<F3G>,
    pub deg: usize,
    pub offset: usize,
    pub size: usize,
    pub dim: usize,
}

#[derive(Debug, Clone)]
pub struct EVIdx {
    pub cm: HashMap<(usize, usize), usize>,
    pub q: HashMap<(usize, usize), usize>,
    pub const_: HashMap<(usize, usize), usize>,
}

impl EVIdx {
    pub fn new() -> Self {
        EVIdx {
            cm: HashMap::new(),
            q: HashMap::new(),
            const_: HashMap::new(),
        }
    }

    pub fn get(&self, type_: &str, p: usize, id: usize) -> Option<&usize> {
        if type_ == "cm" {
            self.cm.get(&(p, id))
        } else if type_ == "q" {
            self.q.get(&(p, id))
        } else {
            assert_eq!(type_, "const");
            self.const_.get(&(p, id))
        }
    }

    pub fn set(&mut self, type_: &str, p: usize, id: usize, idx: usize) {
        if type_ == "cm" {
            self.cm.insert((p, id), idx);
        } else if type_ == "q" {
            self.q.insert((p, id), idx);
        } else {
            assert_eq!(type_, "const");
            self.const_.insert((p, id), idx);
        }
    }
}

//
// prime: false by default
// mode: "" by default
pub fn pil_code_gen(
    ctx: &mut Context,
    pil: &mut PIL,
    exp_id: usize,
    prime: bool,
    mode: &str,
) -> Result<()> {
    //println!("pil_code_gen: {} {}, {:?}", exp_id, prime, mode);
    if mode == "evalQ" && prime {
        pil_code_gen(ctx, pil, exp_id, false, mode)?;
        let exp_in = &pil.expressions[exp_id];
        if exp_in.idQ.is_some() && !exp_in.keep2ns.is_none() {
            pil_code_gen(ctx, pil, exp_id, true, "")?;
        }
        return Ok(());
    }

    let prime_idx = if prime { "expsPrime" } else { "exps" };
    if ctx.calculated_mark.get(&(prime_idx, exp_id)).is_some() {
        return Ok(());
    }

    let exp = pil.expressions[exp_id].clone();
    calculate_deps(ctx, pil, &exp, prime, exp_id, mode)?;

    let mut code_ctx = ContextC {
        exp_id: exp_id,
        tmp_used: ctx.tmp_used,
        code: Vec::new(),
    };

    let exp = pil.expressions[exp_id].clone();
    let ret_ref = eval_exp(&mut code_ctx, pil, &exp, prime)?;
    if (mode == "evalQ") && (pil.expressions[exp_id].idQ.is_some()) {
        if prime {
            return Err(EigenError::InvalidOperator(
                "EvalQ cannot be prime".to_string(),
            ));
        }

        let rqz = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
        code_ctx.tmp_used += 1;

        let exp_node = Node::new("exp".to_string(), exp_id, None, 0, prime, 0);
        code_ctx.code.push(Section {
            op: "sub".to_string(),
            src: vec![ret_ref.clone(), exp_node],
            dest: rqz.clone(),
        });

        let Zi = Node::new("Zi".to_string(), 0, None, 0, false, 0);
        let q = Node::new(
            "q".to_string(),
            pil.expressions[exp_id].idQ.unwrap(),
            None,
            0,
            prime,
            0,
        );
        code_ctx.code.push(Section {
            op: "mul".to_string(),
            src: vec![Zi, rqz],
            dest: q,
        });
    } else if (mode == "correctQ") && (pil.expressions[exp_id].idQ.is_some()) {
        let rqz = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
        code_ctx.tmp_used += 1;

        let q = Node::new(
            "q".to_string(),
            pil.expressions[exp_id].idQ.unwrap(),
            None,
            0,
            prime,
            0,
        );
        let Z = Node::new("Z".to_string(), 0, None, 0, prime, 0);
        code_ctx.code.push(Section {
            op: "mul".to_string(),
            dest: rqz.clone(),
            src: vec![q, Z],
        });
        let exp_node = Node::new("exp".to_string(), exp_id, None, 0, prime, 0);
        code_ctx.code.push(Section {
            op: "sub".to_string(),
            dest: exp_node,
            src: vec![ret_ref.clone(), rqz],
        });
    } else {
        if ret_ref.type_.as_str() == "tmp" {
            let exp_node = Node::new("exp".to_string(), exp_id, None, 0, prime, 0);
            let size = code_ctx.code.len() - 1;
            code_ctx.code[size].dest = exp_node;
            code_ctx.tmp_used -= 1;
        } else {
            let exp_node = Node::new("exp".to_string(), exp_id, None, 0, prime, 0);
            code_ctx.code.push(Section {
                op: "copy".to_string(),
                dest: exp_node,
                src: vec![ret_ref],
            });
        }
    }

    ctx.code.push(Code {
        exp_id: exp_id,
        prime: Some(prime),
        code: code_ctx.code,
        idQ: if pil.expressions[exp_id].idQ.is_some() {
            pil.expressions[exp_id].idQ
        } else {
            None
        },
        tmp_used: 0,
    });

    ctx.calculated_mark.insert((prime_idx, exp_id), true);
    if code_ctx.tmp_used > ctx.tmp_used {
        ctx.tmp_used = code_ctx.tmp_used;
    }
    Ok(())
}

pub fn eval_exp(
    code_ctx: &mut ContextC,
    pil: &mut PIL,
    exp: &Expression,
    prime: bool,
) -> Result<Node> {
    //println!("eval, expression {:?}", exp);
    if ExpressionOps::is_nop(exp) {
        panic!("exp: {:?}", exp);
    }
    let def: Vec<Expression> = vec![];
    let values = match &exp.values {
        Some(x) => x,
        _ => &def,
    };
    match exp.op.as_str() {
        "add" => {
            let a = eval_exp(code_ctx, pil, &(values[0]), prime)?;
            let b = eval_exp(code_ctx, pil, &(values[1]), prime)?;
            let r = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
            code_ctx.tmp_used += 1;
            let c = Section {
                op: "add".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "sub" => {
            let a = eval_exp(code_ctx, pil, &(values[0]), prime)?;
            let b = eval_exp(code_ctx, pil, &(values[1]), prime)?;
            let r = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
            code_ctx.tmp_used += 1;
            let c = Section {
                op: "sub".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "mul" => {
            let a = eval_exp(code_ctx, pil, &values[0], prime)?;
            let b = eval_exp(code_ctx, pil, &values[1], prime)?;
            let r = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
            code_ctx.tmp_used += 1;
            let c = Section {
                op: "mul".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "addc" => {
            let a = eval_exp(code_ctx, pil, &values[0], prime)?;
            let b = Node::new(
                "number".to_string(),
                0,
                Some(exp.const_.unwrap().to_string()),
                0,
                false,
                0,
            );
            let r = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
            code_ctx.tmp_used += 1;
            let c = Section {
                op: "add".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "mulc" => {
            let a = eval_exp(code_ctx, pil, &values[0], prime)?;
            let b = Node::new(
                "number".to_string(),
                0,
                Some(exp.const_.unwrap().to_string()),
                0,
                false,
                0,
            );
            let r = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
            code_ctx.tmp_used += 1;

            let c = Section {
                op: "mul".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "neg" => {
            let a = Node::new("number".to_string(), 0, Some("0".to_string()), 0, false, 0);
            let b = eval_exp(code_ctx, pil, &values[0], prime)?;

            let r = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
            code_ctx.tmp_used += 1;

            let c = Section {
                op: "sub".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "cm" => {
            if exp.next() && prime {
                expression_error(pil, "Double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "cm".to_string(),
                exp.id.unwrap(),
                None,
                0,
                exp.next() || prime,
                0,
            ))
        }
        "const" => {
            if exp.next() && prime {
                expression_error(pil, "Double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "const".to_string(),
                exp.id.unwrap(),
                None,
                0,
                exp.next() || prime,
                0,
            ))
        }
        "exp" => {
            if exp.next() && prime {
                expression_error(pil, "Double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "exp".to_string(),
                exp.id.unwrap(),
                None,
                0,
                exp.next() || prime,
                0,
            ))
        }
        "q" => {
            if exp.next() && prime {
                expression_error(pil, "double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "q".to_string(),
                exp.id.unwrap(),
                None,
                0,
                exp.next() || prime,
                0,
            ))
        }
        "number" => Ok(Node::new(
            "number".to_string(),
            0,
            exp.value.clone(),
            0,
            false,
            0,
        )),
        "public" => Ok(Node::new(
            "public".to_string(),
            exp.id.unwrap(),
            None,
            0,
            false,
            0,
        )),
        "challenge" => Ok(Node::new(
            "challenge".to_string(),
            exp.id.unwrap(),
            None,
            0,
            false,
            0,
        )),
        "eval" => Ok(Node::new(
            "eval".to_string(),
            exp.id.unwrap(),
            None,
            0,
            false,
            0,
        )),
        "xDivXSubXi" => Ok(Node::new("xDivXSubXi".to_string(), 0, None, 0, false, 0)),
        "xDivXSubWXi" => Ok(Node::new("xDivXSubWXi".to_string(), 0, None, 0, false, 0)),
        "x" => Ok(Node::new("x".to_string(), 0, None, 0, false, 0)),
        _ => Err(EigenError::InvalidOperator(format!("eval_exp: {}", exp.op))),
    }
}

pub fn calculate_deps(
    ctx: &mut Context,
    pil: &mut PIL,
    expr: &Expression,
    prime: bool,
    exp_id: usize,
    mode: &str,
) -> Result<()> {
    if expr.op == "exp" {
        let id: usize = if expr.id.is_some() {
            expr.id.unwrap()
        } else {
            0
        };
        if prime && expr.next() {
            expression_error(pil, "Double prime".to_string(), exp_id, id)?;
        }
        pil_code_gen(ctx, pil, id, prime || expr.next(), mode)?;
    }
    match &expr.values {
        Some(x) => {
            for e in x.iter() {
                calculate_deps(ctx, pil, e, prime, exp_id, mode)?;
            }
        }
        &None => {}
    }
    Ok(())
}

pub fn expression_error(pil: &PIL, strerr: String, e1: usize, e2: usize) -> Result<()> {
    //TODO
    Err(EigenError::ExpressionError(strerr))
}

pub fn build_code(ctx: &mut Context, pil: &mut PIL) -> Segment {
    let step_code = Segment {
        first: build_linear_code(ctx, pil, "first".to_string()),
        i: build_linear_code(ctx, pil, "i".to_string()),
        last: build_linear_code(ctx, pil, "last".to_string()),
        tmp_used: ctx.tmp_used,
    };

    if ctx.calculated.exps.len() < pil.expressions.len() {
        ctx.calculated.exps.resize(pil.expressions.len(), false);
        ctx.calculated
            .exps_prime
            .resize(pil.expressions.len(), false);
    }
    for (i, e) in pil.expressions.iter().enumerate() {
        if (!e.keep.is_some()) && e.idQ.is_none() {
            ctx.calculated.exps[i] = false;
            ctx.calculated.exps_prime[i] = false;
        }
    }
    ctx.code = vec![];
    step_code
}

pub fn build_linear_code(ctx: &mut Context, pil: &mut PIL, loop_pos: String) -> Vec<Section> {
    let exp_and_expprimes = match loop_pos.as_str() {
        "i" | "last" => get_exp_and_expprimes(ctx, pil),
        _ => HashMap::<usize, bool>::new(),
    };

    let mut res: Vec<Section> = vec![];
    for (i, c) in ctx.code.iter().enumerate() {
        if exp_and_expprimes.get(&(i)).is_some() {
            if ((loop_pos.as_str() == "i") && (!ctx.code[i].prime.is_some()))
                || (loop_pos.as_str() == "last")
            {
                continue;
            }
        }
        for cc in ctx.code[i].code.iter() {
            res.push(cc.clone());
        }
    }
    res
}

//FIXME where is the exp_id from
fn get_exp_and_expprimes(ctx: &mut Context, pil: &mut PIL) -> HashMap<usize, bool> {
    let mut calc_exps = HashMap::<usize, usize>::new();
    for (i, c) in ctx.code.iter().enumerate() {
        if (pil.expressions[ctx.code[i].exp_id].idQ.is_some())
            || pil.expressions[ctx.code[i].exp_id].keep.is_some()
            || pil.expressions[ctx.code[i].exp_id].keep2ns.is_some()
        {
            let mask = if ctx.code[i].prime.is_some() { 2 } else { 1 };

            let val = match calc_exps.get(&ctx.code[i].exp_id) {
                Some(x) => *x,
                _ => 0,
            };
            calc_exps.insert(ctx.code[i].exp_id, val | mask);
        }
    }

    let mut res = HashMap::<usize, bool>::new();
    for (k, v) in calc_exps.iter() {
        res.insert(*k, if *v == 3 { true } else { false });
    }
    res
}

pub fn iterate_code(
    code: &mut Segment,
    f: fn(&mut Node, &mut ContextF, pil: &mut PIL),
    ctx: &mut ContextF,
    pil: &mut PIL,
) {
    iterate(&mut code.first, f, ctx, pil);
    iterate(&mut code.i, f, ctx, pil);
    iterate(&mut code.last, f, ctx, pil);
}

fn iterate(
    code: &mut Vec<Section>,
    f: fn(&mut Node, &mut ContextF, pil: &mut PIL),
    ctx: &mut ContextF,
    pil: &mut PIL,
) {
    for c in code.iter_mut() {
        for s in c.src.iter_mut() {
            f(s, ctx, pil);
        }
        f(&mut c.dest, ctx, pil);
    }
}
