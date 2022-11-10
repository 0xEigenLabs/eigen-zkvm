use std::collections::HashMap;
use crate::types::Expression;
use crate::errors::EigenError;
use winter_math::fields::f64::BaseElement;

pub struct Calculated {
    exps: Vec<bool>,
    exps_prime: Vec<bool>
}

pub struct Context<'a> {
    pil: &'a PIL,
    exp_id: i32,
    tmp_used: u32,
    code: Vec<Code>,
    clculated: Calculated,
}

pub struct Node {
    pub type_: String,
    id: Option<i32>,
    value: Option<String>,
    dim: Option<i32>,
    prime: Option<()>,
    tree_pos: Option<i32>,
}

impl Node {
    pub fn new(type_: String, id: Option<i32>, value: Option<String>, dim: Option<i32>, prime: Option<()>, tree_pos: Option<i32>) -> Self {
        Node {
            type_: type_,
            id: id,
            value: value,
            dim: dim,
            prime: prime,
            tree_pos: tree_pos,
        }
    }
}

pub struct Code {
    pub op: String,
    pub dest: Node,
    pub src: Vec<Node>,
}

pub struct StepCode {
    pub first: Vec<Code>,
    pub i: Vec<Code>,
    pub last: Vec<Code>,
    tmp_used: u32,
}


struct MapSections {
    cm1_n: i32,
    cm1_2ns: i32,
    cm2_n: i32,
    cm2_2ns: i32,
    cm3_n: i32,
    cm3_2ns: i32,
    q_2ns: i32,
    exps_withq_n: i32,
    exps_withq_2ns: i32,
    exps_withoutq_n: i32,
    exps_withoutq_2ns: i32,
}

//
// prime: false by default
// mode: "" by default
pub fn pil_code_gen(ctx: &mut Context, exp_id: i32, bool: prime, mode: String) {
    if mode == "evalQ" && prime {
        pil_code_gen(ctx, exp_id, false, mode);
        if ctx.pil.expressions[exp_id].idQ.is_some() &&
            !ctx.pil.expressions[exp_id].is_none() {
                pil_code_gen(ctx, exp_id, true, "".to_string());
        }
        return;
    }

    let prime_idx = if prime { "expsPrime"} else {"exps"};
    //FIXME
    if ctx.calculated[prime_idx][exp_id] {
        return;
    }

    calculate_deps(ctx, ctx.pil.expressions[exp_id], prime, exp_id, mode);

    let mut code_ctx = Context {
        pil: pil,
        exp_id: exp_id,
        tmp_used: ctx.tmp_used,
        code: Vec::new(),
        calculated: Calculated {
            exps: vec![false; ctx.pil.expressions.len()],
            exps_prime: vec![false; ctx.pil.expressions.len()],
        },
    };

    let ret_ref = eval_exp(&mut code_ctx, &ctx.pil.expressions[exp_id], prime)?;

}

pub fn eval_exp(code_ctx: &mut Context, exp: &Expression, prime: bool) -> Result<Node> {

    match exp.op.as_str() {
    "add" => {
        let a = eval_exp(code_ctx, exp.values.unwrap()[0], prime);
        let b = eval_exp(code_ctx, exp.values.unwrap()[1], prime);
        let r = Node::new("tmp".to_string(), Some(code_ctx.tmp_used), None, None, None, None);
        code_ctx.tmp_used += 1;
        let c = Code {
            op: "add".to_string(),
            dest: r,
            src: vec![a, b],
        };
        code_ctx.code.push(c);
        Ok(r)
    },
    "sub" => {
        let a = eval_exp(code_ctx, exp.values.unwrap()[0], prime);
        let b = eval_exp(code_ctx, exp.values.unwrap()[1], prime);
        let r = Node::new("tmp".to_string(), Some(code_ctx.tmp_used), None, None, None, None);
        code_ctx.tmp_used += 1;
        let c = Code {
            op: "sub".to_string(),
            dest: r,
            src: vec![a, b],
        };
        code_ctx.code.push(c);
        Ok(r)
    },
    "mul" => {
        let a = eval_exp(code_ctx, exp.values.unwrap()[0], prime);
        let b = eval_exp(code_ctx, exp.values.unwrap()[1], prime);
        let r = Node::new("tmp".to_string(), Some(code_ctx.tmp_used), None, None, None, None);
        code_ctx.tmp_used += 1;
        let c = Code {
            op: "mul".to_string(),
            dest: r,
            src: vec![a, b],
        };
        code_ctx.code.push(c);
        Ok(r)
    },
    "addc" => {
        let a = eval_exp(code_ctx, exp.values.unwrap()[0], prime);
        let b = Node::new("number".to_string(), None, Some(exp.const_.unwrap().to_string()), None, None, None);
        let r = Node::new("tmp".to_string(), Some(code_ctx.tmp_used), None, None, None, None);
        code_ctx.tmp_used += 1;
        let c = Code {
            op: "add".to_string(),
            dest: r,
            src: vec![a, b],
        };
        code_ctx.code.push(c);
        Ok(r)
    },
    "mulc" => {
        let a = eval_exp(code_ctx, exp.values.unwrap()[0], prime);
        let b = Node::new("number".to_string(), None, Some(exp.const_.unwrap().to_string()), None, None, None);
        let r = Node::new("tmp".to_string(), Some(code_ctx.tmp_used), None, None, None, None);
        code_ctx.tmp_used += 1;

        let c = Code {
            op: "mul".to_string(),
            dest: r,
            src: vec![a, b],
        };
        code_ctx.code.push(c);
        Ok(r)
    },
    "neg" => {
        let a = Node::new("number".to_string(), None, Some("0".to_string()), None, None, None);
        let b = eval_exp(code_ctx, exp.values.unwrap()[0], prime);

        let r = Node::new("tmp".to_string(), Some(code_ctx.tmp_used), None, None, None, None);
        code_ctx.tmp_used += 1;

        codeCtx.code.push({
            op: "sub",
            dest: r,
            src: [a, b]
        });
        return r;
    },
    "cm" => {
        if (exp.next.is_some() && prime) {
            expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id)?;
        }
        Node::new("cm".to_string(), Some(exp.id), None, None, Some(exp.next.is_some() || prime), None)
    },
    "const" => {
        if (exp.next.is_some() && prime) {
            expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id)?;
        }
        Node::new("const".to_string(), Some(exp.id), None, None, Some(exp.next.is_some() || prime), None)
    },
    "exp" => {
        if (exp.next.is_some() && prime) {
            expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id)?;
        }
        Node::new("exp".to_string(), Some(exp.id), None, None, Some(exp.next.is_some() || prime), None)
    },
    "q" => {
        if (exp.next.is_some() && prime) {
            expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id)?;
        }
        Node::new("q".to_string(), Some(exp.id), None, None, Some(exp.next.is_some() || prime), None)
    },
    "number" => {
        Node::new("number".to_string(), None, Some(exp.value), None, None, None)
    },
    "public" => {
        Node::new("public".to_string(), Some(exp.id), None, None, None, None)
    },
    "challenge" => {
        Node::new("challenge".to_string(), Some(exp.id), None, None, None, None)
    },
    "eval" => {
        Node::new("eval".to_string(), Some(exp.id), None, None, None, None)
    },
    "xDivXSubXi" => {
        Node::new("xDivXSubXi".to_string(), None, None, None, None, None)
    },
    "xDivXSubWXi" => {
        Node::new("xDivXSubWXi".to_string(), None, None, None, None, None)
    },
    "x" => {
        Node::new("x".to_string(), None, None, None, None, None)
    },
    _ => {
        Err(EigenError::InvalidOperator(exp.op.to_string())
    }
}

pub fn calculate_deps(ctx: &mut Context, expr: &Expression, prime: bool, exp_id: i32, mode: String) -> Result<()> {
    if expr.op == "exp" {
        if prime && exp.next {
            expression_error(ctx.pil, "Double prime".to_string(), exp_id, exp.id)?;
        }
        pil_code_gen(ctx, expr.id, prime || expr.next, mode);
    }
    if expr.values.is_some() {
        for e in expr.values.unwrap().iter() {
            calculate_deps(ctx, e, prime, exp_id, mode);
        }
    }
    Ok(())
}

pub fn expression_error(pil: &PIL, strerr: String, e1: i32, e2: i32) -> Result<()>{
    //TODO
    Err(EigenError::ExpressionError(strerr))
}

pub fn build_code(ctx: &mut Context) -> StepCode {
    let step_code = StepCode {
        first: build_linear_code(ctx, "first".to_string()),
        i: build_linear_code(ctx, "i".to_string()),
        last: build_linear_code(ctx, "last".to_string()),
        tmp_used: ctx.tmp_used,
    };

    for (i, e) in ctx.pil.expressions.iter().enumerate() {
        if (!e.keep.is_some()) && e.idQ.is_none() {
            ctx.calculated.exps[i] = false;
            ctx.calculated.exps_prime[i] = false;
        }
    }
    ctx.code = vec![];
    step_code
}

pub fn build_linear_code(ctx: &mut Context, loop_pos: String) -> Vec<Code> {

    let exp_and_expprimes = match loop_pos.as_str() {
        "i", "last" =>  {
            get_exp_and_expprimes(ctx)
        },
        _ => {
            HashMap::<String, bool>::new()
    };

    let res: Vec<Code> = [];
    for (i, c) in ctx.code.iter().enumerate() {
        if exp_and_expprimes[i] {
            if ((loop_pos.as_str() == "i") && (!ctx.code[i].prime)) ||
                (loop_pos.as_str() == "last") {
                    continue;
            }
        }
        for cc in ctx.code[i].code.iter() {
            res.push(cc);
        }
    }
    res
}

//FIXME where is the exp_id from
fn get_exp_and_expprimes(ctx: &mut Context) -> HashMap<String, bool> {

    let mut calc_exps = HashMap::<String, i32>::new();
    for (i, c) in ctx.code.iter().enumerate() {
        if (ctx.pil.expressions[ctx.code[i].exp_id].idQ.is_some()) ||
            ctx.pil.expressions[ctx.code[i].exp_id].keep.is_some() ||
                ctx.pil.expressions[ctx.code[i].exp_id].keep2ns.is_some() {
                    let mask = if ctx.code[i].prime.is_some() {2} else {1};
                    calc_exps[ctx.code[i].exp_id] = calc_exps[ctx.code[i].exp_id || 0] | mask;
        }
    }

    let res = HashMap::<String, bool>::new();
    for (k, v) in calc_exps.iter() {
        res[k] = if v == 3 {true}  else {false};
    }
    res
}


pub fn iterate_code(code: &StepCode, f: fn(&Node, &mut Context), ctx: &mut Context) {
    iterate(&code.first, f, ctx);
    iterate(&code.i, f, ctx);
    iterate(&code.last, f, ctx);
}

fn iterate(code: &Vec<Code>, f: fn(&Node, &mut Context), ctx: &mut Context) {
    for c in code.iter() {
        for s in c.src.iter() {
            f(s, ctx);
        }
        f(&c.dest, ctx);
    }
}
