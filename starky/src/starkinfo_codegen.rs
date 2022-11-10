use crate::errors::{EigenError, Result};
use crate::types::Expression;
use crate::types::PIL;
use std::collections::HashMap;
use winter_math::fields::f64::BaseElement;

pub struct Calculated {
    pub exps: Vec<bool>,
    pub exps_prime: Vec<bool>,
}

pub struct Context<'a> {
    pub pil: &'a PIL,
    pub exp_id: i32,
    pub tmp_used: i32,
    pub code: Vec<Code>,
    pub calculated: Calculated,
    pub calculated_mark: HashMap<(String, i32), bool>,
}

pub struct ContextC<'a> {
    pub pil: &'a PIL,
    pub exp_id: i32,
    pub tmp_used: i32,
    pub code: Vec<Subcode>,
}

pub struct ContextF {
    pub exp_map: HashMap<(i32, i32), i32>,
    pub tmp_used: i32,
}

#[derive(Clone)]
pub struct Node {
    pub type_: String,
    pub id: Option<i32>,
    pub value: Option<String>,
    pub dim: Option<i32>,
    pub prime: Option<bool>,
    pub tree_pos: Option<i32>,
}

impl Node {
    pub fn new(
        type_: String,
        id: Option<i32>,
        value: Option<String>,
        dim: Option<i32>,
        prime: Option<bool>,
        tree_pos: Option<i32>,
    ) -> Self {
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

#[derive(Clone)]
pub struct Subcode {
    pub op: String,
    pub dest: Node,
    pub src: Vec<Node>,
}

pub struct Segment {
    pub first: Vec<Subcode>,
    pub i: Vec<Subcode>,
    pub last: Vec<Subcode>,
    pub tmp_used: i32,
}

pub struct Code {
    pub exp_id: i32,
    pub prime: Option<bool>,
    pub tmp_used: i32,
    pub code: Vec<Subcode>,
    pub idQ: Option<i32>,
}

struct MapSections {
    pub cm1_n: i32,
    pub cm1_2ns: i32,
    pub cm2_n: i32,
    pub cm2_2ns: i32,
    pub cm3_n: i32,
    pub cm3_2ns: i32,
    pub q_2ns: i32,
    pub exps_withq_n: i32,
    pub exps_withq_2ns: i32,
    pub exps_withoutq_n: i32,
    pub exps_withoutq_2ns: i32,
}

//
// prime: false by default
// mode: "" by default
pub fn pil_code_gen(ctx: &mut Context, exp_id: i32, prime: bool, mode: &String) -> Result<()> {
    if mode.as_str() == "evalQ" && prime {
        pil_code_gen(ctx, exp_id, false, mode);
        if ctx.pil.expressions[exp_id as usize].idQ.is_some()
            && !ctx.pil.expressions[exp_id as usize].keep2ns.is_none()
        {
            pil_code_gen(ctx, exp_id, true, &"".to_string());
        }
        return Ok(());
    }

    let prime_idx = (if prime { "expsPrime" } else { "exps" }).to_string();
    if ctx
        .calculated_mark
        .get(&(prime_idx.clone(), exp_id))
        .is_some()
    {
        return Ok(());
    }

    calculate_deps(
        ctx,
        &ctx.pil.expressions[exp_id as usize],
        prime,
        exp_id,
        &mode,
    );

    let mut code_ctx = ContextC {
        pil: ctx.pil,
        exp_id: exp_id,
        tmp_used: ctx.tmp_used,
        code: Vec::new(),
    };

    let ret_ref = eval_exp(&mut code_ctx, &ctx.pil.expressions[exp_id as usize], prime)?;
    if (mode.as_str() == "evalQ") && (ctx.pil.expressions[exp_id as usize].idQ.is_some()) {
        if prime {
            return Err(EigenError::InvalidOperator(
                "EvalQ cannot be prime".to_string(),
            ));
        }

        let rqz = Node::new(
            "tmp".to_string(),
            Some(code_ctx.tmp_used),
            None,
            None,
            None,
            None,
        );
        code_ctx.tmp_used += 1;

        let exp_node = Node::new(
            "exp".to_string(),
            Some(exp_id),
            None,
            None,
            Some(prime),
            None,
        );
        code_ctx.code.push(Subcode {
            op: "sub".to_string(),
            src: vec![ret_ref.clone(), exp_node],
            dest: rqz.clone(),
        });

        let Zi = Node::new("Zi".to_string(), None, None, None, None, None);
        let q = Node::new(
            "q".to_string(),
            Some(ctx.pil.expressions[exp_id as usize].idQ.unwrap()),
            None,
            None,
            Some(prime),
            None,
        );
        code_ctx.code.push(Subcode {
            op: "mul".to_string(),
            src: vec![Zi, rqz],
            dest: q,
        });
    } else if (mode.as_str() == "correctQ") && (ctx.pil.expressions[exp_id as usize].idQ.is_some())
    {
        let rqz = Node::new(
            "tmp".to_string(),
            Some(code_ctx.tmp_used),
            None,
            None,
            None,
            None,
        );
        code_ctx.tmp_used += 1;

        let q = Node::new(
            "q".to_string(),
            Some(ctx.pil.expressions[exp_id as usize].idQ.unwrap()),
            None,
            None,
            Some(prime),
            None,
        );
        let Z = Node::new("Z".to_string(), None, None, None, Some(prime), None);
        code_ctx.code.push(Subcode {
            op: "mul".to_string(),
            dest: rqz.clone(),
            src: vec![q, Z],
        });
        let exp_node = Node::new(
            "exp".to_string(),
            Some(exp_id),
            None,
            None,
            Some(prime),
            None,
        );
        code_ctx.code.push(Subcode {
            op: "sub".to_string(),
            dest: exp_node,
            src: vec![ret_ref.clone(), rqz],
        });
    } else {
        if ret_ref.type_.as_str() == "tmp" {
            let exp_node = Node::new(
                "exp".to_string(),
                Some(exp_id),
                None,
                None,
                Some(prime),
                None,
            );
            let size = code_ctx.code.len() - 1;
            code_ctx.code[size].dest = exp_node;
            code_ctx.tmp_used -= 1;
        } else {
            let exp_node = Node::new(
                "exp".to_string(),
                Some(exp_id),
                None,
                None,
                Some(prime),
                None,
            );
            code_ctx.code.push(Subcode {
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
        idQ: if ctx.pil.expressions[exp_id as usize].idQ.is_some() {
            ctx.pil.expressions[exp_id as usize].idQ
        } else {
            None
        },
        tmp_used: -1,
    });

    ctx.calculated_mark.insert((prime_idx, exp_id), true);
    if code_ctx.tmp_used > ctx.tmp_used {
        ctx.tmp_used = code_ctx.tmp_used;
    }
    Ok(())
}

pub fn eval_exp(code_ctx: &mut ContextC, exp: &Expression, prime: bool) -> Result<Node> {
    let def: Vec<Expression> = vec![];
    let values = match &exp.values {
        Some(x) => x,
        _ => &def,
    };
    match exp.op.as_str() {
        "add" => {
            let a = eval_exp(code_ctx, &(values[0]), prime)?;
            let b = eval_exp(code_ctx, &(values[1]), prime)?;
            let r = Node::new(
                "tmp".to_string(),
                Some(code_ctx.tmp_used),
                None,
                None,
                None,
                None,
            );
            code_ctx.tmp_used += 1;
            let c = Subcode {
                op: "add".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "sub" => {
            let a = eval_exp(code_ctx, &(values[0]), prime)?;
            let b = eval_exp(code_ctx, &(values[1]), prime)?;
            let r = Node::new(
                "tmp".to_string(),
                Some(code_ctx.tmp_used),
                None,
                None,
                None,
                None,
            );
            code_ctx.tmp_used += 1;
            let c = Subcode {
                op: "sub".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "mul" => {
            let a = eval_exp(code_ctx, &values[0], prime)?;
            let b = eval_exp(code_ctx, &values[1], prime)?;
            let r = Node::new(
                "tmp".to_string(),
                Some(code_ctx.tmp_used),
                None,
                None,
                None,
                None,
            );
            code_ctx.tmp_used += 1;
            let c = Subcode {
                op: "mul".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "addc" => {
            let a = eval_exp(code_ctx, &values[0], prime)?;
            let b = Node::new(
                "number".to_string(),
                None,
                Some(exp.const_.unwrap().to_string()),
                None,
                None,
                None,
            );
            let r = Node::new(
                "tmp".to_string(),
                Some(code_ctx.tmp_used),
                None,
                None,
                None,
                None,
            );
            code_ctx.tmp_used += 1;
            let c = Subcode {
                op: "add".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "mulc" => {
            let a = eval_exp(code_ctx, &values[0], prime)?;
            let b = Node::new(
                "number".to_string(),
                None,
                Some(exp.const_.unwrap().to_string()),
                None,
                None,
                None,
            );
            let r = Node::new(
                "tmp".to_string(),
                Some(code_ctx.tmp_used),
                None,
                None,
                None,
                None,
            );
            code_ctx.tmp_used += 1;

            let c = Subcode {
                op: "mul".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "neg" => {
            let a = Node::new(
                "number".to_string(),
                None,
                Some("0".to_string()),
                None,
                None,
                None,
            );
            let b = eval_exp(code_ctx, &values[0], prime)?;

            let r = Node::new(
                "tmp".to_string(),
                Some(code_ctx.tmp_used),
                None,
                None,
                None,
                None,
            );
            code_ctx.tmp_used += 1;

            let c = Subcode {
                op: "sub".to_string(),
                dest: r.clone(),
                src: vec![a, b],
            };
            code_ctx.code.push(c);
            Ok(r)
        }
        "cm" => {
            if exp.next.is_some() && prime {
                expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "cm".to_string(),
                exp.id,
                None,
                None,
                Some(exp.next.is_some() || prime),
                None,
            ))
        }
        "const" => {
            if exp.next.is_some() && prime {
                expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "const".to_string(),
                exp.id,
                None,
                None,
                Some(exp.next.is_some() || prime),
                None,
            ))
        }
        "exp" => {
            if exp.next.is_some() && prime {
                expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "exp".to_string(),
                exp.id,
                None,
                None,
                Some(exp.next.is_some() || prime),
                None,
            ))
        }
        "q" => {
            if exp.next.is_some() && prime {
                expression_error(code_ctx.pil, "double Prime".to_string(), code_ctx.exp_id, 0)?;
            }
            Ok(Node::new(
                "q".to_string(),
                exp.id,
                None,
                None,
                Some(exp.next.is_some() || prime),
                None,
            ))
        }
        "number" => Ok(Node::new(
            "number".to_string(),
            None,
            exp.value.clone(),
            None,
            None,
            None,
        )),
        "public" => Ok(Node::new(
            "public".to_string(),
            exp.id,
            None,
            None,
            None,
            None,
        )),
        "challenge" => Ok(Node::new(
            "challenge".to_string(),
            exp.id,
            None,
            None,
            None,
            None,
        )),
        "eval" => Ok(Node::new(
            "eval".to_string(),
            exp.id,
            None,
            None,
            None,
            None,
        )),
        "xDivXSubXi" => Ok(Node::new(
            "xDivXSubXi".to_string(),
            None,
            None,
            None,
            None,
            None,
        )),
        "xDivXSubWXi" => Ok(Node::new(
            "xDivXSubWXi".to_string(),
            None,
            None,
            None,
            None,
            None,
        )),
        "x" => Ok(Node::new("x".to_string(), None, None, None, None, None)),
        _ => Err(EigenError::InvalidOperator(exp.op.to_string())),
    }
}

pub fn calculate_deps(
    ctx: &mut Context,
    expr: &Expression,
    prime: bool,
    exp_id: i32,
    mode: &String,
) -> Result<()> {
    if expr.op == "exp" {
        let id: i32 = if expr.id.is_some() {
            expr.id.unwrap()
        } else {
            0
        };
        if prime && expr.next.is_some() {
            expression_error(ctx.pil, "Double prime".to_string(), exp_id, id)?;
        }
        pil_code_gen(ctx, id, prime || expr.next.is_some(), mode);
    }
    match &expr.values {
        Some(x) => {
            for e in x.iter() {
                calculate_deps(ctx, e, prime, exp_id, mode);
            }
        }
        &None => {}
    }
    Ok(())
}

pub fn expression_error(pil: &PIL, strerr: String, e1: i32, e2: i32) -> Result<()> {
    //TODO
    Err(EigenError::ExpressionError(strerr))
}

pub fn build_code(ctx: &mut Context) -> Segment {
    let mut step_code = Segment {
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

pub fn build_linear_code(ctx: &mut Context, loop_pos: String) -> Vec<Subcode> {
    let exp_and_expprimes = match loop_pos.as_str() {
        "i" | "last" => get_exp_and_expprimes(ctx),
        _ => HashMap::<i32, bool>::new(),
    };

    let mut res: Vec<Subcode> = vec![];
    for (i, c) in ctx.code.iter().enumerate() {
        if exp_and_expprimes.get(&(i as i32)).is_some() {
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
fn get_exp_and_expprimes(ctx: &mut Context) -> HashMap<i32, bool> {
    let mut calc_exps = HashMap::<i32, i32>::new();
    for (i, c) in ctx.code.iter().enumerate() {
        if (ctx.pil.expressions[ctx.code[i].exp_id as usize]
            .idQ
            .is_some())
            || ctx.pil.expressions[ctx.code[i].exp_id as usize]
                .keep
                .is_some()
            || ctx.pil.expressions[ctx.code[i].exp_id as usize]
                .keep2ns
                .is_some()
        {
            let mask = if ctx.code[i].prime.is_some() { 2 } else { 1 };

            let val = match calc_exps.get(&ctx.code[i].exp_id) {
                Some(x) => *x,
                _ => 0,
            };
            calc_exps.insert(ctx.code[i].exp_id, val | mask);
        }
    }

    let mut res = HashMap::<i32, bool>::new();
    for (k, v) in calc_exps.iter() {
        res.insert(*k, if *v == 3 { true } else { false });
    }
    res
}

pub fn iterate_code(code: &mut Segment, f: fn(&mut Node, &mut ContextF), ctx: &mut ContextF) {
    iterate(&mut code.first, f, ctx);
    iterate(&mut code.i, f, ctx);
    iterate(&mut code.last, f, ctx);
}

fn iterate(code: &mut Vec<Subcode>, f: fn(&mut Node, &mut ContextF), ctx: &mut ContextF) {
    for c in code.iter_mut() {
        for s in c.src.iter_mut() {
            f(s, ctx);
        }
        f(&mut c.dest, ctx);
    }
}
