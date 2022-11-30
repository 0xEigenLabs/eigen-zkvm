#![allow(non_snake_case, dead_code)]
use crate::f3g::F3G;
use crate::stark_gen::StarkContext;
use crate::starkinfo::StarkInfo;
use crate::starkinfo_codegen::Node;
use crate::starkinfo_codegen::Section;
use std::fmt;
use winter_math::{FieldElement, StarkField};

#[derive(Clone, Debug)]
pub enum Ops {
    Vari(F3G), // instant value
    Add,       // add and push the result into stack
    Sub,       // sub and push the result into stack
    Mul,       // mul and push the result into stack
    Copy_,     // push instant value into stack
    Assign,    // assign value from mem into an address. *op = val
    Refer, // refer to a variable in memory, the index must be of format: offset + ((i+next)%N) * size.
    Ret,   // must return
}

/// example: `ctx.const_n[${r.id} + ((i+1)%${N})*${ctx.starkInfo.nConstants} ]`;
/// where the r.id, N, ctx.starkInfo.nConstants modified by `${}` are the instant value, ctx.const_n and i are the symble.
/// the symbol should the fields of the global context, have same name as Index.
/// so the example would be Expr { op: Refer, syms: [ctx.const_n, i], defs: [Vari, Vari...] }
#[derive(Clone, Debug)]
pub struct Expr {
    pub op: Ops,
    pub syms: Vec<String>,
    pub defs: Vec<Expr>,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.op {
            Ops::Add | Ops::Mul | Ops::Sub => {
                write!(f, "{:?} {} {}", self.op, self.defs[0], self.defs[1])
            }
            Ops::Copy_ => {
                write!(f, "copy ({})", self.defs[0])
            }
            Ops::Ret => {
                write!(f, "ret")
            }
            Ops::Refer => {
                write!(
                    f,
                    "addr ({}) ({} + ((i + {})%{}) * {})",
                    self.syms[0], self.defs[0], self.defs[1], self.defs[2], self.defs[3]
                )
            }
            Ops::Vari(x) => {
                write!(f, "{}", x)
            }
            Ops::Assign => {
                write!(f, "write ({})", self.defs[0])
            }
            _ => {
                panic!("Invalid op");
            }
        }
    }
}

impl Expr {
    pub fn new(op: Ops, syms: Vec<String>, defs: Vec<Expr>) -> Self {
        Self { op, syms, defs }
    }
}

impl From<F3G> for Expr {
    fn from(v: F3G) -> Self {
        Expr::new(Ops::Vari(v), vec![], vec![])
    }
}

#[derive(Debug)]
pub struct Block {
    pub namespace: String,
    pub exprs: Vec<Expr>,
}

impl Block {
    /// parameters: ctx, i
    /// example:
    /// let block = compile_code();
    /// block.eval(&mut ctx, i);
    pub fn eval(&self, ctx: &mut StarkContext, arg_i: usize) -> F3G {
        let mut val_stack: Vec<F3G> = Vec::new();
        let length = self.exprs.len();
        //println!("length: {}", length);

        let mut i = 0usize;
        while i < length {
            let expr = &self.exprs[i];
            //println!("op@{} is {}", i, expr);
            i += 1;
            match expr.op {
                Ops::Ret => {
                    return val_stack.pop().unwrap();
                }
                Ops::Vari(x) => {
                    val_stack.push(x);
                }
                Ops::Add => {
                    let lhs = match expr.defs[0].op {
                        Ops::Vari(x) => x,
                        _ => get_value(ctx, &expr.defs[0], arg_i),
                    };
                    let rhs = match expr.defs[1].op {
                        Ops::Vari(x) => x,
                        _ => get_value(ctx, &expr.defs[1], arg_i),
                    };
                    val_stack.push(lhs + rhs);
                }
                Ops::Mul => {
                    let lhs = match expr.defs[0].op {
                        Ops::Vari(x) => x,
                        _ => get_value(ctx, &expr.defs[0], arg_i),
                    };
                    let rhs = match expr.defs[1].op {
                        Ops::Vari(x) => x,
                        _ => get_value(ctx, &expr.defs[1], arg_i),
                    };
                    val_stack.push(lhs - rhs);
                }
                Ops::Sub => {
                    let lhs = match expr.defs[0].op {
                        Ops::Vari(x) => x,
                        _ => get_value(ctx, &expr.defs[0], arg_i),
                    };
                    let rhs = match expr.defs[1].op {
                        Ops::Vari(x) => x,
                        _ => get_value(ctx, &expr.defs[1], arg_i),
                    };
                    val_stack.push(lhs * rhs);
                }
                Ops::Copy_ => {
                    let x = if let Ops::Vari(x) = expr.defs[0].op {
                        x
                    } else {
                        // get value from address
                        get_value(ctx, &expr.defs[0], arg_i)
                    };
                    val_stack.push(x);
                }
                Ops::Assign => {
                    let next_expr = &expr.defs[0];
                    let id = get_i(next_expr, arg_i);
                    let addr = &next_expr.syms[0];
                    let val = val_stack.pop().unwrap(); // get the value from stack

                    let mut val_addr = ctx.get_mut(addr.as_str());
                    if val.dim == 1 || addr.as_str() == "tmp" {
                        val_addr[id] = val;
                    } else {
                        let vals = val.as_base_elements();
                        //println!("11111111111111111111113d {:?} ", vals);
                        val_addr[id] = F3G::from(vals[0]);
                        val_addr[id + 1] = F3G::from(vals[1]);
                        val_addr[id + 2] = F3G::from(vals[2]);
                    }
                }
                Ops::Refer => {
                    // push value into stack
                    let addr = &expr.syms[0];
                    let x = get_value(ctx, expr, arg_i);
                    val_stack.push(x);
                }
                _ => {
                    panic!("Invalid op in block");
                }
            }
        }
        F3G::ZERO
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ns: {}\n", self.namespace)?;
        for i in 0..self.exprs.len() {
            write!(f, "\t {}\n", self.exprs[i]);
        }
        Ok(())
    }
}

pub fn compile_code(
    ctx: &StarkContext,
    starkinfo: &StarkInfo,
    code: &Vec<Section>,
    dom: &str,
    ret: bool,
) -> Block {
    let next = if dom == "n" {
        1
    } else {
        1 << (ctx.nbits_ext - ctx.nbits)
    };
    let next = next;

    let N = if dom == "n" {
        1 << ctx.nbits
    } else {
        1 << ctx.nbits_ext
    };
    let N = N;

    let mut body: Block = Block {
        namespace: "ctx".to_string(),
        exprs: Vec::new(),
    };

    for i in 0..code.len() {
        //println!("compile: {:?}", code[i]);
        let mut src: Vec<Expr> = Vec::new();
        for j in 0..code[i].src.len() {
            src.push(get_ref(ctx, starkinfo, &code[i].src[j], &dom, next, N));
        }

        let exp = match (&code[i].op).as_str() {
            "add" => Expr::new(Ops::Add, Vec::new(), (&src[0..2]).to_vec()),
            "sub" => Expr::new(Ops::Sub, Vec::new(), (&src[0..2]).to_vec()),
            "mul" => Expr::new(Ops::Mul, Vec::new(), (&src[0..2]).to_vec()),
            "copy" => Expr::new(Ops::Copy_, Vec::new(), (&src[0..1]).to_vec()),
            _ => {
                panic!("Invalid op {:?}", code[i])
            }
        };
        set_ref(ctx, starkinfo, &code[i].dest, exp, dom, next, N, &mut body);
    }
    if ret {
        let sz = code.len() - 1;
        body.exprs
            .push(get_ref(ctx, starkinfo, &code[sz].dest, &dom, next, N));
        body.exprs.push(Expr::new(Ops::Ret, vec![], vec![]));
    }
    body
}

fn get_index(offset: usize, next: usize, N: usize, size: usize) -> Vec<Expr> {
    let offset = Expr::from(F3G::from(offset));
    let size = Expr::from(F3G::from(size));
    let next = Expr::from(F3G::from(next));
    let NB = Expr::from(F3G::from(N));
    vec![offset, next, NB, size]
}

fn get_i(expr: &Expr, arg_i: usize) -> usize {
    let get_val = |i: usize| -> usize {
        match expr.defs[i].op {
            // reference to instant value
            Ops::Vari(x) => x.to_be().as_int() as usize, //FIXME out of range
            _ => {
                // reference to address
                panic!("Invalid Vari: {}", expr);
            }
        }
    };
    let offset = get_val(0);
    let next = get_val(1);
    let N = get_val(2);
    let size = get_val(3);
    offset + ((arg_i + next) % N) * size
}

fn get_value(ctx: &mut StarkContext, expr: &Expr, arg_i: usize) -> F3G {
    let addr = &expr.syms[0];

    match addr.as_str() {
        "tmp" => {
            let id = get_i(expr, arg_i);
            ctx.tmp[id]
        }
        "cm1_n" => {
            let id = get_i(expr, arg_i);
            ctx.cm1_n[id]
        }
        "cm1_2ns" => {
            let id = get_i(expr, arg_i);
            ctx.cm1_2ns[id]
        }
        "cm2_n" => {
            let id = get_i(expr, arg_i);
            ctx.cm2_n[id]
        }
        "cm2_2ns" => {
            let id = get_i(expr, arg_i);
            ctx.cm2_2ns[id]
        }
        "cm3_n" => {
            let id = get_i(expr, arg_i);
            ctx.cm3_n[id]
        }
        "cm3_2ns" => {
            let id = get_i(expr, arg_i);
            ctx.cm3_2ns[id]
        }
        "q_2ns" => {
            let id = get_i(expr, arg_i);
            ctx.q_2ns[id]
        }
        "exps_n" => {
            let id = get_i(expr, arg_i);
            ctx.exps_n[id]
        }
        "exps_2ns" => {
            let id = get_i(expr, arg_i);
            ctx.exps_2ns[id]
        }
        "const_n" => {
            let id = get_i(expr, arg_i);
            ctx.const_n[id]
        }
        "const_2ns" => {
            let id = get_i(expr, arg_i);
            ctx.const_2ns[id]
        }
        "exps_withq_n" => {
            let id = get_i(expr, arg_i);
            ctx.exps_withq_n[id]
        }
        "exps_withq_2ns" => {
            let id = get_i(expr, arg_i);
            ctx.exps_withq_2ns[id]
        }
        "publics" => {
            let id = get_i(expr, arg_i);
            ctx.publics[id]
        }
        "challenge" => {
            let id = get_i(expr, arg_i);
            ctx.challenges[id]
        }
        "evals" => {
            let id = get_i(expr, arg_i);
            ctx.evals[id]
        }
        "xDivXSubXi" => {
            let id = get_i(expr, arg_i);
            F3G::new(
                ctx.xDivXSubXi[id],
                ctx.xDivXSubXi[id + 1],
                ctx.xDivXSubXi[id + 2],
            )
        }
        "xDivXSubWXi" => {
            let id = get_i(expr, arg_i);
            F3G::new(
                ctx.xDivXSubWXi[id],
                ctx.xDivXSubWXi[id + 1],
                ctx.xDivXSubWXi[id + 2],
            )
        }
        "x_n" => ctx.x_n[arg_i],
        "x_2ns" => ctx.x_2ns[arg_i],
        "Zi" => (ctx.Zi)(arg_i),
        _ => {
            panic!("invalid symbol {:?}", addr);
        }
    }
}

fn set_ref(
    ctx: &StarkContext,
    starkinfo: &StarkInfo,
    r: &Node,
    val: Expr,
    dom: &str,
    next: usize,
    N: usize,
    body: &mut Block,
) {
    let e_dst = match r.type_.as_str() {
        "tmp" => Expr::new(
            Ops::Refer,
            vec!["tmp".to_string()],
            get_index(r.id, 0, N, 0),
        ),
        "exp" => {
            if dom == "n" {
                let pol_id = starkinfo.exps_n[r.id].clone();
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else if dom == "2ns" {
                let pol_id = starkinfo.exps_2ns[r.id].clone();
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else {
                panic!("Invalid dom");
            }
        }
        "q" => {
            if dom == "n" {
                panic!("Accesssing q in domain n");
            } else if dom == "2ns" {
                let pol_id = starkinfo.qs[r.id];
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else {
                panic!("Invalid dom");
            }
        }
        _ => {
            panic!("Invalid reference type set {}", r.type_)
        }
    };
    body.exprs.push(val);
    body.exprs.push(Expr::new(Ops::Assign, vec![], vec![e_dst]));
}

fn get_ref(
    ctx: &StarkContext,
    starkinfo: &StarkInfo,
    r: &Node,
    dom: &str,
    next: usize,
    N: usize,
) -> Expr {
    match r.type_.as_str() {
        "tmp" => Expr::new(
            Ops::Refer,
            vec!["tmp".to_string()],
            get_index(r.id, 0, N, 0),
        ),
        "const" => {
            if dom == "n" {
                if r.prime {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_n".to_string(), "i".to_string()],
                        get_index(r.id, 1, N, starkinfo.n_constants),
                    )
                } else {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_n".to_string(), "i".to_string()],
                        get_index(r.id, 0, N, starkinfo.n_constants),
                    )
                }
            } else if dom == "2ns" {
                if r.prime {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_2ns".to_string(), "i".to_string()],
                        get_index(r.id, next, N, starkinfo.n_constants),
                    )
                } else {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_2ns".to_string(), "i".to_string()],
                        get_index(r.id, 0, N, starkinfo.n_constants),
                    )
                }
            } else {
                panic!("Invalid dom");
            }
        }
        "cm" => {
            if dom == "n" {
                let pol_id = starkinfo.cm_n[r.id];
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else if dom == "2ns" {
                let pol_id = starkinfo.cm_2ns[r.id];
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else {
                panic!("Invalid dom");
            }
        }
        "q" => {
            if dom == "n" {
                panic!("Accesssing q in domain n");
            } else if dom == "2ns" {
                let pol_id = starkinfo.qs[r.id];
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else {
                panic!("Invalid dom");
            }
        }
        "exp" => {
            if dom == "n" {
                let pol_id = starkinfo.exps_n[r.id].clone();
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else if dom == "2ns" {
                let pol_id = starkinfo.exps_2ns[r.id].clone();
                eval_map(ctx, starkinfo, &pol_id, &r.prime, &next, &N)
            } else {
                panic!("Invalid dom");
            }
        }

        "number" => Expr::new(
            Ops::Vari(F3G::from(r.value.clone().unwrap().parse::<u64>().unwrap())),
            vec![],
            vec![],
        ),
        "public" => Expr::new(
            Ops::Refer,
            vec!["publics".to_string()],
            get_index(r.id, 0, N, 0),
        ),
        "challenge" => Expr::new(
            Ops::Refer,
            vec!["challenge".to_string()],
            get_index(r.id, 0, N, 0),
        ),
        "eval" => Expr::new(
            Ops::Refer,
            vec!["evals".to_string()],
            get_index(r.id, 0, N, 0),
        ),
        "xDivXSubXi" => Expr::new(
            Ops::Refer,
            vec!["xDivXSubXi".to_string()],
            get_index(0, 0, N, 3),
        ),
        "xDivXSubWXi" => Expr::new(
            Ops::Refer,
            vec!["xDivXSubWXi".to_string()],
            get_index(0, 0, N, 3),
        ),
        "x" => {
            if dom == "n" {
                Expr::new(Ops::Refer, vec!["x_n".to_string()], get_index(0, 0, N, 1))
            } else if dom == "2ns" {
                Expr::new(
                    Ops::Refer,
                    vec!["x_2ns".to_string()],
                    get_index(0, 0, N, 1), //i
                )
            } else {
                panic!("Invalid dom");
            }
        }
        "Zi" => Expr::new(Ops::Refer, vec!["Zi".to_string()], get_index(0, 0, N, 1)),
        _ => panic!("Invalid reference type get, {}", r.type_),
    }
}

fn eval_map(
    ctx: &StarkContext,
    starkinfo: &StarkInfo,
    pol_id: &usize,
    prime: &bool,
    next: &usize,
    N: &usize,
) -> Expr {
    let p = &starkinfo.var_pol_map[*pol_id];
    let offset = Expr::from(F3G::from(p.section_pos));
    let size = Expr::from(F3G::from(starkinfo.map_sectionsN.get(&p.section)));
    let next = Expr::from(F3G::from(*next));
    let NB = Expr::from(F3G::from(*N));
    let zero = Expr::from(F3G::from(0));
    if p.dim == 1 {
        if *prime {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone(), "i".to_string()],
                vec![offset, next, NB, size],
            )
        } else {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone(), "i".to_string()],
                vec![offset, zero, NB, size],
            )
        }
    } else if p.dim == 3 {
        if *prime {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone(), "i".to_string(), "3".to_string()],
                vec![offset, next, NB, size],
            )
        } else {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone(), "i".to_string(), "3".to_string()],
                vec![offset, zero, NB, size],
            )
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
}
