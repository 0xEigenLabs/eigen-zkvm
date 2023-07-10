#![allow(dead_code, non_snake_case)]
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
pub struct Context {
    pub exp_id: usize,
    pub tmp_used: usize,
    pub code: Vec<Code>,
    pub calculated: HashMap<(&'static str, usize), bool>,
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
    pub dom: String,
    pub tmpexps: &'a mut HashMap<usize, usize>,
    pub starkinfo: &'a mut StarkInfo,
}

#[derive(Debug)]
pub struct Code {
    pub exp_id: usize,
    pub prime: bool,
    pub tmp_used: usize,
    pub code: Vec<Section>,
    pub idQ: Option<usize>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Node {
    pub type_: String,
    pub id: usize,
    pub value: Option<String>,
    pub dim: usize,
    pub prime: bool,
    pub tree_pos: usize,
    pub p: usize, // position
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
        assert_eq!(type_.len() > 0, true);
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

/// Subcode
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

#[derive(Debug, Default, Serialize)]
pub struct IndexVec {
    pub cm1_n: Vec<usize>,
    pub cm1_2ns: Vec<usize>,
    pub cm2_n: Vec<usize>,
    pub cm2_2ns: Vec<usize>,
    pub cm3_n: Vec<usize>,
    pub cm3_2ns: Vec<usize>,
    pub cm4_n: Vec<usize>,
    pub cm4_2ns: Vec<usize>,
    pub tmpexp_n: Vec<usize>,
    pub q_2ns: Vec<usize>,
    pub f_2ns: Vec<usize>,
}

impl IndexVec {
    pub fn get(&self, name: &str) -> &Vec<usize> {
        match name {
            "cm1_n" => &self.cm1_n,
            "cm1_2ns" => &self.cm1_2ns,
            "cm2_n" => &self.cm2_n,
            "cm2_2ns" => &self.cm2_2ns,
            "cm3_n" => &self.cm3_n,
            "cm3_2ns" => &self.cm3_2ns,
            "cm4_n" => &self.cm4_n,
            "cm4_2ns" => &self.cm4_2ns,
            "tmpexp_n" => &self.tmpexp_n,
            "q_2ns" => &self.q_2ns,
            "f_2ns" => &self.q_2ns,
            _ => panic!("Invalid name={} in index", name),
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Index {
    pub cm1_n: usize,
    pub cm1_2ns: usize,
    pub cm2_n: usize,
    pub cm2_2ns: usize,
    pub cm3_n: usize,
    pub cm3_2ns: usize,
    pub cm4_n: usize,
    pub cm4_2ns: usize,
    pub tmpexp_n: usize,
    pub q_2ns: usize,
    pub f_2ns: usize,
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
            "cm4_n" => self.cm4_n,
            "cm4_2ns" => self.cm4_2ns,
            "tmpexp_n" => self.tmpexp_n,
            "q_2ns" => self.q_2ns,
            "f_2ns" => self.q_2ns,
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
            "cm4_n" => {
                self.cm4_n = val;
            }
            "cm4_2ns" => {
                self.cm4_2ns = val;
            }
            "q_2ns" => {
                self.q_2ns = val;
            }
            "f_2ns" => {
                self.f_2ns = val;
            }
            "tmpexp_n" => {
                self.tmpexp_n = val;
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct EVIdx {
    pub cm: HashMap<(usize, usize), usize>,
    pub const_: HashMap<(usize, usize), usize>,
}

impl EVIdx {
    pub fn new() -> Self {
        EVIdx {
            cm: HashMap::new(),
            const_: HashMap::new(),
        }
    }

    pub fn get(&self, type_: &str, p: usize, id: usize) -> Option<&usize> {
        if type_ == "cm" {
            self.cm.get(&(p, id))
        } else {
            assert_eq!(type_, "const");
            self.const_.get(&(p, id))
        }
    }

    pub fn set(&mut self, type_: &str, p: usize, id: usize, idx: usize) {
        if type_ == "cm" {
            self.cm.insert((p, id), idx);
        } else {
            assert_eq!(type_, "const");
            self.const_.insert((p, id), idx);
        }
    }
}

// prime: false by default
// res_type: "" by default
// res_id: 0 by default
pub fn pil_code_gen(
    ctx: &mut Context,
    pil: &mut PIL,
    exp_id: usize,
    prime: bool,
    res_type: &str,
    res_id: usize,
    muladd: bool,
) -> Result<()> {
    log::debug!(
        "pil_code_gen: {} {}, {} {}",
        exp_id,
        prime,
        res_type,
        res_id
    );
    let prime_idx = if prime { "expsPrime" } else { "exps" };
    if ctx.calculated.get(&(prime_idx, exp_id)).is_some() {
        if res_type.len() > 0 {
            let idx = ctx
                .code
                .iter()
                .position(|x| (x.exp_id == exp_id) && (x.prime == prime))
                .unwrap();
            let c = &mut ctx.code[idx];
            let dest = Node::new(res_type.to_string(), res_id, None, 0, prime, 0);
            c.code.push(Section {
                op: "copy".to_string(),
                dest: dest,
                src: vec![c.code[c.code.len() - 1].dest.clone()],
            });
        }
        return Ok(());
    }

    let exp = pil.expressions[exp_id].clone();
    calculate_deps(ctx, pil, &exp, prime, exp_id, false)?;

    let mut code_ctx = ContextC {
        exp_id: exp_id,
        tmp_used: ctx.tmp_used,
        code: Vec::new(),
    };
    let _exp = pil.expressions[exp_id].clone();
    let exp = match muladd {
        true => find_muladd(&_exp),
        _ => _exp,
    };
    let ret_ref = eval_exp(&mut code_ctx, pil, &exp, prime)?;
    if ret_ref.type_.as_str() == "tmp" {
        let sz = code_ctx.code.len() - 1;
        code_ctx.code[sz].dest = Node::new("exp".to_string(), exp_id, None, 0, prime, 0);
        code_ctx.tmp_used -= 1;
    } else {
        let exp_node = Node::new("exp".to_string(), exp_id, None, 0, prime, 0);
        code_ctx.code.push(Section {
            op: "copy".to_string(),
            dest: exp_node,
            src: vec![ret_ref],
        });
    }
    if res_type.len() > 0 {
        if prime {
            panic!("Prime in retType");
        }

        let dest = Node::new(res_type.to_string(), res_id, None, 0, prime, 0);
        let src = Node::new("exp".to_string(), exp_id, None, 0, prime, 0);
        code_ctx.code.push(Section {
            op: "copy".to_string(),
            dest: dest,
            src: vec![src],
        });
    }

    ctx.code.push(Code {
        exp_id: exp_id,
        prime: prime,
        code: code_ctx.code,
        tmp_used: 0,
        idQ: None,
    });

    ctx.calculated.insert((prime_idx, exp_id), true);
    if code_ctx.tmp_used > ctx.tmp_used {
        ctx.tmp_used = code_ctx.tmp_used;
    }
    log::debug!("ctx.calculated: {:?}", ctx.calculated);
    Ok(())
}

fn find_muladd(exp: &Expression) -> Expression {
    if exp.values.is_some() {
        let values = exp.values.as_ref().unwrap();
        if exp.op.as_str() == "add" && values[0].op.as_str() == "mul" {
            log::debug!("exp: {:?}", exp);
            let value_of_values = values[0].values.as_ref().unwrap();
            let a = find_muladd(&value_of_values[0]);
            let b = find_muladd(&value_of_values[1]);
            let c = find_muladd(&values[1]);
            return Expression::new("muladd".to_string(), 0, None, None, Some(vec![a, b, c]));
        } else if exp.op.as_str() == "add" && values[1].op.as_str() == "mul" {
            log::debug!("exp: 1 {:?}", exp);
            let value_of_values = values[1].values.as_ref().unwrap();
            let a = find_muladd(&value_of_values[0]);
            let b = find_muladd(&value_of_values[1]);
            let c = find_muladd(&values[0]);
            return Expression::new("muladd".to_string(), 0, None, None, Some(vec![a, b, c]));
        } else {
            let mut r = exp.clone();
            let mut mut_values: Vec<Expression> = Vec::new();
            for i in 0..values.len() {
                mut_values.push(find_muladd(&values[i]))
            }
            if mut_values.len() > 0 {
                r.values = Some(mut_values);
            }
            return r;
        }
    }
    exp.clone()
}

pub fn eval_exp(
    code_ctx: &mut ContextC,
    pil: &mut PIL,
    exp: &Expression,
    prime: bool,
) -> Result<Node> {
    //log::debug!("eval, expression {}", exp);
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
        "muladd" => {
            let a = eval_exp(code_ctx, pil, &values[0], prime)?;
            let b = eval_exp(code_ctx, pil, &values[1], prime)?;
            let c = eval_exp(code_ctx, pil, &values[2], prime)?;
            let r = Node::new("tmp".to_string(), code_ctx.tmp_used, None, 0, false, 0);
            code_ctx.tmp_used += 1;
            let c = Section {
                op: "muladd".to_string(),
                dest: r.clone(),
                src: vec![a, b, c],
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
    muladd: bool,
) -> Result<()> {
    //log::debug!("calculate_deps: {}", expr);
    if expr.op == "exp" {
        let id = expr.id.unwrap();
        if prime && expr.next() {
            expression_error(pil, "Double prime".to_string(), exp_id, id)?;
        }
        pil_code_gen(ctx, pil, id, prime || expr.next(), "", 0, muladd)?;
    }
    if expr.values.is_some() {
        for e in expr.values.as_ref().unwrap().iter() {
            calculate_deps(ctx, pil, e, prime, exp_id, muladd)?;
        }
    }
    Ok(())
}

pub fn expression_error(_pil: &PIL, strerr: String, _e1: usize, _e2: usize) -> Result<()> {
    //TODO
    Err(EigenError::ExpressionError(strerr))
}

pub fn build_code(ctx: &mut Context, pil: &mut PIL) -> Segment {
    let seg = Segment {
        first: build_linear_code(ctx, pil, "first"),
        i: build_linear_code(ctx, pil, "i"),
        last: build_linear_code(ctx, pil, "last"),
        tmp_used: ctx.tmp_used,
    };

    // FIXME: deprecated
    for (i, e) in pil.expressions.iter().enumerate() {
        if (!e.keep.is_some()) && e.idQ.is_none() {
            ctx.calculated.insert(("exps", i), false);
            ctx.calculated.insert(("expsPrime", i), false);
        }
    }
    ctx.code = vec![];
    seg
}

pub fn build_linear_code(ctx: &mut Context, pil: &PIL, loop_pos: &str) -> Vec<Section> {
    let exp_and_expprimes = match loop_pos {
        "i" | "last" => get_exp_and_expprimes(ctx, pil),
        _ => HashMap::<usize, bool>::new(),
    };

    let mut res: Vec<Section> = vec![];
    for i in 0..ctx.code.len() {
        let ep = exp_and_expprimes.get(&i);
        if ep.is_some() && (*ep.unwrap()) {
            if ((loop_pos == "i") && (!ctx.code[i].prime)) || (loop_pos == "last") {
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
fn get_exp_and_expprimes(ctx: &mut Context, pil: &PIL) -> HashMap<usize, bool> {
    let mut calc_exps = HashMap::<usize, usize>::new();
    for i in 0..ctx.code.len() {
        if (pil.expressions[ctx.code[i].exp_id].idQ.is_some())
            || pil.expressions[ctx.code[i].exp_id].keep.is_some()
            || pil.expressions[ctx.code[i].exp_id].keep2ns.is_some()
        {
            let mask = if ctx.code[i].prime { 2 } else { 1 };
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
    let mut iterate = |sec: &mut Vec<Section>, f: fn(&mut Node, &mut ContextF, pil: &mut PIL)| {
        for c in sec.iter_mut() {
            for s in c.src.iter_mut() {
                f(s, ctx, pil);
            }
            f(&mut c.dest, ctx, pil);
        }
    };
    iterate(&mut code.first, f);
    iterate(&mut code.i, f);
    iterate(&mut code.last, f);
}
