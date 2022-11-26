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
    Refer,     // refer to a variable in memory
    Ret,       // must return
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

        let mut i = 0usize;
        while i < length {
            let expr = &self.exprs[i];
            i += 1;
            match expr.op {
                Ops::Ret => {
                    return val_stack.pop().unwrap();
                }
                Ops::Vari(x) => {
                    val_stack.push(x);
                }
                Ops::Add => {
                    let lhs = val_stack.pop().unwrap();
                    let rhs = val_stack.pop().unwrap();
                    val_stack.push(lhs + rhs);
                }
                Ops::Mul => {
                    let lhs = val_stack.pop().unwrap();
                    let rhs = val_stack.pop().unwrap();
                    val_stack.push(lhs - rhs);
                }
                Ops::Sub => {
                    let lhs = val_stack.pop().unwrap();
                    let rhs = val_stack.pop().unwrap();
                    val_stack.push(lhs * rhs);
                }
                Ops::Copy_ => {
                    let x = if let Ops::Vari(x) = expr.defs[0].op {
                        x
                    } else {
                        panic!("invalid oprand {:?}", expr)
                    };
                    val_stack.push(x);
                }
                Ops::Assign => {
                    let addr = &expr.syms[0];
                    let id = if let Ops::Vari(x) = expr.defs[1].op {
                        x.to_be().as_int() as usize // FIXME out of range
                    } else {
                        panic!("invalid oprand {:?}", expr)
                    };
                    let val = val_stack.pop().unwrap(); // get the value from stack
                                                        //*addr = value
                    match addr.as_str() {
                        "tmp" => {
                            ctx.tmp[id] = val;
                        }
                        "cm1_n" => {
                            ctx.cm1_n[id] = val;
                        }
                        "cm1_2ns" => {
                            ctx.cm1_2ns[id] = val;
                        }
                        "cm2_n" => {
                            ctx.cm2_n[id] = val;
                        }
                        "cm2_2ns" => {
                            ctx.cm2_2ns[id] = val;
                        }
                        "cm3_n" => {
                            ctx.cm3_n[id] = val;
                        }
                        "cm3_2ns" => {
                            ctx.cm3_2ns[id] = val;
                        }
                        "q_2ns" => {
                            ctx.q_2ns[id] = val;
                        }
                        "exps_n" => {
                            ctx.exps_n[id] = val;
                        }
                        "exps_2ns" => {
                            ctx.exps_2ns[id] = val;
                        }
                        "exps_withq_n" => {
                            ctx.exps_withq_n[id] = val;
                        }
                        "exps_withq_2ns" => {
                            ctx.exps_withq_2ns[id] = val;
                        }
                        _ => {
                            panic!("invalid symbol {:?}", addr);
                        }
                    }
                }
                Ops::Refer => {
                    // push value into stack
                    // syms: [addr, i, dim]
                    let addr = &expr.syms[0];
                    let i = if expr.syms.len() == 2 { true } else { false }; // i exists
                    let dim = if expr.syms.len() == 3 { 3 } else { 1 };

                    // defs: [offset, next, N, size] => index = offset + ((1+next)%N) * size
                    let get_val = |i: usize| -> usize {
                        match expr.defs[i].op {
                            Ops::Vari(x) => x.to_be().as_int() as usize, //FIXME out of range
                            _ => {
                                panic!("invalid oprand {:?}", expr);
                            }
                        }
                    };

                    let get_4 = || {
                        let offset = get_val(0);
                        let next = get_val(1);
                        let N = get_val(2);
                        let size = get_val(3);
                        offset + ((arg_i + next) % N) * size
                    };

                    let x = match addr.as_str() {
                        "tmp" => {
                            let id = get_val(0);
                            ctx.tmp[id]
                        }
                        "cm1_n" => {
                            let id = get_4();
                            ctx.cm1_n[id]
                        }
                        "cm1_2ns" => {
                            let id = get_4();
                            ctx.cm1_2ns[id]
                        }
                        "cm2_n" => {
                            let id = get_4();
                            ctx.cm2_n[id]
                        }
                        "cm2_2ns" => {
                            let id = get_4();
                            ctx.cm2_2ns[id]
                        }
                        "cm3_n" => {
                            let id = get_4();
                            ctx.cm3_n[id]
                        }
                        "cm3_2ns" => {
                            let id = get_4();
                            ctx.cm3_2ns[id]
                        }
                        "q_2ns" => {
                            let id = get_4();
                            ctx.q_2ns[id]
                        }
                        "exps_n" => {
                            let id = get_4();
                            ctx.exps_n[id]
                        }
                        "exps_2ns" => {
                            let id = get_4();
                            ctx.exps_2ns[id]
                        }
                        "const_n" => {
                            let id = get_4();
                            ctx.const_n[id]
                        }
                        "const_2ns" => {
                            let id = get_4();
                            ctx.const_2ns[id]
                        }
                        "exps_withq_n" => {
                            let id = get_4();
                            ctx.exps_withq_n[id]
                        }
                        "exps_withq_2ns" => {
                            let id = get_4();
                            ctx.exps_withq_2ns[id]
                        }
                        "publics" => {
                            let id = get_val(0);
                            ctx.publics[id]
                        }
                        "challenge" => {
                            let id = get_val(0);
                            ctx.challenges[id]
                        }
                        "evals" => {
                            let id = get_val(0);
                            ctx.evals[id]
                        }
                        "xDivXSubXi" => {
                            let id = arg_i;
                            F3G::new(
                                ctx.xDivXSubXi[3 * id],
                                ctx.xDivXSubXi[3 * id + 1],
                                ctx.xDivXSubXi[3 * id + 2],
                            )
                        }
                        "xDivXSubWXi" => {
                            let id = arg_i;
                            F3G::new(
                                ctx.xDivXSubWXi[3 * id],
                                ctx.xDivXSubWXi[3 * id + 1],
                                ctx.xDivXSubWXi[3 * id + 2],
                            )
                        }
                        "x_n" => ctx.x_n[arg_i],
                        "x_2ns" => ctx.x_2ns[arg_i],
                        "Zi" => (ctx.Zi)(arg_i),
                        _ => {
                            panic!("invalid symbol {:?}", addr);
                        }
                    };
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
        write!(f, "({}, {:?})", self.namespace, self.exprs)
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
        let mut src: Vec<Expr> = Vec::new();
        for j in 0..code[i].src.len() {
            src[j] = get_ref(ctx, starkinfo, &code[i].src[j], &dom, &next, &N);
        }

        let exp = match (&code[i].op).as_str() {
            "add" => Expr::new(Ops::Add, Vec::new(), (&src[0..2]).to_vec()),
            "sub" => Expr::new(Ops::Sub, Vec::new(), (&src[0..2]).to_vec()),
            "muk" => Expr::new(Ops::Mul, Vec::new(), (&src[0..2]).to_vec()),
            "copy" => Expr::new(Ops::Copy_, Vec::new(), (&src[0..1]).to_vec()),
            _ => {
                panic!("Invalid op")
            }
        };
        set_ref(
            ctx,
            starkinfo,
            &code[i].dest,
            exp,
            dom,
            &next,
            &N,
            &mut body,
        );
    }

    if ret {
        let sz = code.len() - 1;
        body.exprs
            .push(get_ref(ctx, starkinfo, &code[sz].dest, &dom, &next, &N));
        body.exprs.push(Expr::new(Ops::Ret, vec![], vec![]));
    }
    body
}

fn set_ref(
    ctx: &StarkContext,
    starkinfo: &StarkInfo,
    r: &Node,
    val: Expr,
    dom: &str,
    next: &usize,
    N: &usize,
    body: &mut Block,
) {
    let e_dst = match r.type_.as_str() {
        "tmp" => Expr::new(
            Ops::Refer,
            vec!["tmp".to_string()],
            vec![Expr::from(F3G::from(r.id))],
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
    next: &usize,
    N: &usize,
) -> Expr {
    match r.type_.as_str() {
        "tmp" => Expr::new(
            Ops::Refer,
            vec!["tmp".to_string()],
            vec![Expr::from(F3G::from(r.id))],
        ),
        "const" => {
            if dom == "n" {
                if r.prime {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_n".to_string(), "i".to_string()],
                        vec![
                            Expr::from(F3G::from(r.id)),
                            Expr::from(F3G::ONE),
                            Expr::from(F3G::from(N)),
                            Expr::from(F3G::from(starkinfo.n_constants)),
                        ],
                    )
                } else {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_n".to_string(), "i".to_string()],
                        vec![
                            Expr::from(F3G::from(r.id)),
                            Expr::from(F3G::ZERO),
                            Expr::from(F3G::from(N)),
                            Expr::from(F3G::from(starkinfo.n_constants)),
                        ],
                    )
                }
            } else if dom == "2ns" {
                if r.prime {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_2ns".to_string(), "i".to_string()],
                        vec![
                            Expr::from(F3G::from(r.id)),
                            Expr::from(F3G::from(next)),
                            Expr::from(F3G::from(N)),
                            Expr::from(F3G::from(starkinfo.n_constants)),
                        ],
                    )
                } else {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_2ns".to_string(), "i".to_string()],
                        vec![
                            Expr::from(F3G::from(r.id)),
                            Expr::from(F3G::ZERO),
                            Expr::from(F3G::from(N)),
                            Expr::from(F3G::from(starkinfo.n_constants)),
                        ],
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
            Ops::Refer,
            vec![],
            vec![Expr::from(F3G::from(
                r.value.clone().unwrap().parse::<u64>().unwrap(),
            ))],
        ),
        "public" => Expr::new(
            Ops::Refer,
            vec!["publics".to_string()],
            vec![Expr::from(F3G::from(r.id))],
        ),
        "challenge" => Expr::new(
            Ops::Refer,
            vec!["challenge".to_string()],
            vec![Expr::from(F3G::from(r.id))],
        ),
        "eval" => Expr::new(
            Ops::Refer,
            vec!["evals".to_string()],
            vec![Expr::from(F3G::from(r.id))],
        ),
        "xDivXSubXi" => Expr::new(
            Ops::Refer,
            vec!["xDivXSubXi".to_string(), "i".to_string()],
            vec![],
        ),
        "xDivXSubWXi" => Expr::new(
            Ops::Refer,
            vec!["xDivXSubWXi".to_string(), "i".to_string()],
            vec![],
        ),
        "x" => {
            if dom == "n" {
                Expr::new(Ops::Refer, vec!["x_n".to_string(), "i".to_string()], vec![])
            } else if dom == "2ns" {
                Expr::new(
                    Ops::Refer,
                    vec!["x_2ns".to_string(), "i".to_string()],
                    vec![],
                )
            } else {
                panic!("Invalid dom");
            }
        }
        "Zi" => Expr::new(Ops::Refer, vec!["Zi".to_string(), "i".to_string()], vec![]),
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
    let zero = Expr::from(F3G::ZERO);

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
