#![allow(non_snake_case, dead_code)]
use crate::stark_gen::StarkContext;
use crate::starkinfo::StarkInfo;
use crate::starkinfo_codegen::Node;
use crate::starkinfo_codegen::Section;
use crate::traits::FieldExtension;
use crate::types::parse_pil_number;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Ops<F: FieldExtension> {
    Vari(F), // instant value
    Add,     // add and push the result into stack
    Sub,     // sub and push the result into stack
    Mul,     // mul and push the result into stack
    Copy_,   // push instant value into stack
    Write,   // assign value from mem into an address. *op = val
    Refer, // format := [addr, [dim]], refer to a variable in memory with dimension dim, the index must be of format: offset + ((i+next)%N) * size.
    Ret,   // must return
}

/// example: `ctx.const_n[${r.id} + ((i+1)%${N})*${ctx.starkInfo.nConstants} ]`;
/// where the r.id, N, ctx.starkInfo.nConstants modified by `${}` are the instant value, ctx.const_n and i are the symble.
/// the symbol should the fields of the global context, have same name as Index.
/// so the example would be Expr { op: Refer, syms: [ctx.const_n, i], defs: [Vari, Vari...] }
#[derive(Clone, Debug)]
pub struct Expr<T: FieldExtension> {
    pub op: Ops<T>,
    pub syms: Vec<String>,  // symbol: tmp, q_2ns etc.
    pub defs: Vec<Expr<T>>, // values bound to the symbol
    pub addr: Vec<usize>,   // address, format: (offset, next, modulas, size)
}

impl<T: FieldExtension> fmt::Display for Expr<T> {
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
                    "addr ({}) ({} + ((i + {})%{}) * {}) dim={}",
                    self.syms[0],
                    self.addr[0],
                    self.addr[1],
                    self.addr[2],
                    self.addr[3],
                    if self.syms.len() == 2 { 3 } else { 1 }
                )
            }
            Ops::Vari(x) => {
                write!(f, "{}", x)
            }
            Ops::Write => {
                write!(f, "write ({})", self.defs[0])
            }
        }
    }
}

impl<T: FieldExtension> Expr<T> {
    pub fn new(op: Ops<T>, syms: Vec<String>, defs: Vec<Expr<T>>, addr: Vec<usize>) -> Self {
        Self { op, syms, defs, addr }
    }
}

impl<T: FieldExtension> From<T> for Expr<T> {
    fn from(v: T) -> Self {
        Expr::new(Ops::<T>::Vari(v), vec![], vec![], vec![])
    }
}

#[derive(Debug)]
pub struct Block<T: FieldExtension> {
    pub namespace: String,
    pub exprs: Vec<Expr<T>>,
}

impl<T: FieldExtension> Block<T> {
    /// parameters: ctx, i
    /// example:
    /// let block = compile_code();
    /// block.eval(&mut ctx, i);
    pub fn eval(&self, ctx: &mut StarkContext<T>, arg_i: usize) -> T {
        let mut val_stack: Vec<T> = Vec::new();
        let length = self.exprs.len();

        let mut i = 0usize;
        while i < length {
            let expr = &self.exprs[i];
            // log::trace!("op@{} is {}", i, expr);
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
                    val_stack.push(lhs * rhs);
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
                    val_stack.push(lhs - rhs);
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
                Ops::Write => {
                    let next_expr = &expr.defs[0];
                    let id = get_i(next_expr, arg_i);
                    let addr = &next_expr.syms[0];
                    let val = val_stack.pop().unwrap(); // get the value from stack

                    let val_addr = ctx.get_mut(addr.as_str());
                    if val.dim() == 1 || addr.as_str() == "tmp" {
                        // TODO: need double confirm the condition
                        val_addr[id] = val;
                    } else {
                        // here we again unfold elements of GF(2^3) to 3-tuple(triple)
                        let vals = val.as_elements();
                        val_addr[id] = T::from(vals[0]);
                        val_addr[id + 1] = T::from(vals[1]);
                        val_addr[id + 2] = T::from(vals[2]);
                    }
                }
                Ops::Refer => {
                    // push value into stack
                    let x = get_value(ctx, expr, arg_i);
                    val_stack.push(x);
                }
            }
        }
        T::ZERO
    }
}

impl<T: FieldExtension> fmt::Display for Block<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.exprs.len() {
            writeln!(f, "  {}", self.exprs[i])?;
        }
        writeln!(f)
    }
}

pub fn compile_code<T: FieldExtension>(
    ctx: &StarkContext<T>,
    starkinfo: &StarkInfo,
    code: &[Section],
    dom: &str,
    ret: bool,
) -> Block<T> {
    let next = if dom == "n" { 1 } else { 1 << (ctx.nbits_ext - ctx.nbits) };

    let N = if dom == "n" { 1 << ctx.nbits } else { 1 << ctx.nbits_ext };
    let modulas = N;

    let mut body: Block<T> = Block { namespace: "ctx".to_string(), exprs: Vec::new() };

    for cj in code.iter() {
        let mut src: Vec<Expr<T>> = Vec::new();
        for k in 0..cj.src.len() {
            src.push(get_ref(ctx, starkinfo, &cj.src[k], dom, next, modulas));
            //log::trace!("get_ref_src: {}", src[src.len() - 1]);
        }

        let exp = match cj.op.as_str() {
            "add" => Expr::new(Ops::Add, Vec::new(), src[0..2].to_vec(), vec![]),
            "sub" => Expr::new(Ops::Sub, Vec::new(), src[0..2].to_vec(), vec![]),
            "mul" => Expr::new(Ops::Mul, Vec::new(), src[0..2].to_vec(), vec![]),
            "copy" => Expr::new(Ops::Copy_, Vec::new(), src[0..1].to_vec(), vec![]),
            _ => {
                panic!("Invalid op {:?}", cj)
            }
        };
        set_ref(ctx, starkinfo, &cj.dest, exp, dom, next, modulas, &mut body);
    }
    if ret {
        let sz = code.len() - 1;
        body.exprs.push(get_ref(ctx, starkinfo, &code[sz].dest, dom, next, modulas));
        body.exprs.push(Expr::new(Ops::Ret, vec![], vec![], vec![]));
    }
    body
}

#[inline(always)]
fn get_i<T: FieldExtension>(expr: &Expr<T>, arg_i: usize) -> usize {
    let offset = expr.addr[0];
    let next = expr.addr[1];
    let modulas = expr.addr[2];
    let size = expr.addr[3];
    offset + ((arg_i + next) % modulas) * size
}

fn get_value<T: FieldExtension>(ctx: &mut StarkContext<T>, expr: &Expr<T>, arg_i: usize) -> T {
    let addr = &expr.syms[0];
    match addr.as_str() {
        "tmp" | "cm1_n" | "cm1_2ns" | "cm2_n" | "cm2_2ns" | "cm3_n" | "cm3_2ns" | "cm4_n"
        | "cm4_2ns" | "q_2ns" | "f_2ns" | "publics" | "challenge" | "exps_n" | "exps_2ns"
        | "const_n" | "const_2ns" | "evals" | "x_n" | "x_2ns" | "tmpexp_n" => {
            let id = get_i(expr, arg_i);
            let ctx_section = ctx.get_mut(addr.as_str()); // OPT: readonly ctx
            let dim = match expr.syms.len() {
                2 => expr.syms[1].parse::<usize>().unwrap(),
                _ => 1,
            };

            // TODO: I just add the
            match dim {
                5 => T::from_vec(vec![
                    ctx_section[id].to_be(),
                    ctx_section[id + 1].to_be(),
                    ctx_section[id + 2].to_be(),
                    ctx_section[id + 3].to_be(),
                    ctx_section[id + 4].to_be(),
                ]),
                3 => T::from_vec(vec![
                    ctx_section[id].to_be(),
                    ctx_section[id + 1].to_be(),
                    ctx_section[id + 2].to_be(),
                ]),

                1 => ctx_section[id],
                _ => panic!("Invalid dim"),
            }
        }
        "xDivXSubXi" => {
            let id = get_i(expr, arg_i);
            // TODO: We need to Support F5G , FG
            T::from_vec(vec![ctx.xDivXSubXi[id], ctx.xDivXSubXi[id + 1], ctx.xDivXSubXi[id + 2]])
        }
        "xDivXSubWXi" => {
            let id = get_i(expr, arg_i);
            // TODO: We need to Support F5G , FG
            T::from_vec(vec![ctx.xDivXSubWXi[id], ctx.xDivXSubWXi[id + 1], ctx.xDivXSubWXi[id + 2]])
        }
        "Zi" => (ctx.Zi)(arg_i),
        _ => {
            panic!("invalid symbol {:?}", addr);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn set_ref<T: FieldExtension>(
    ctx: &StarkContext<T>,
    starkinfo: &StarkInfo,
    r: &Node,
    val: Expr<T>,
    dom: &str,
    next: usize,
    modulas: usize,
    body: &mut Block<T>,
) {
    //log::trace!("set_ref: r {:?}  dom {} val {}", r, dom, val);
    let e_dst = match r.type_.as_str() {
        "tmp" => Expr::new(Ops::Refer, vec!["tmp".to_string()], vec![], vec![r.id, 0, modulas, 0]),
        "q" => {
            if dom == "n" {
                panic!("Accesssing q in domain n");
            } else if dom == "2ns" {
                if starkinfo.q_dim == 3 {
                    Expr::new(
                        Ops::Refer,
                        vec!["q_2ns".to_string(), "3".to_string()],
                        vec![],
                        vec![r.id, 0, modulas, 3],
                    )
                } else if starkinfo.q_dim == 1 {
                    Expr::new(
                        Ops::Refer,
                        vec!["q_2ns".to_string()],
                        vec![],
                        vec![r.id, 0, modulas, 1],
                    )
                } else {
                    panic!("Invalid dom");
                }
            } else {
                panic!("Invalid dom");
            }
        }
        "f" => {
            if dom == "n" {
                panic!("Accesssing q in domain n");
            } else if dom == "2ns" {
                Expr::new(
                    Ops::Refer,
                    vec!["f_2ns".to_string(), "3".to_string()],
                    vec![],
                    vec![r.id, 0, modulas, 3],
                )
            } else {
                panic!("Invalid dom");
            }
        }
        "cm" => {
            if dom == "n" {
                let pol_id = starkinfo.cm_n[r.id];
                eval_map(ctx, starkinfo, pol_id, r.prime, next, modulas)
            } else if dom == "2ns" {
                let pol_id = starkinfo.cm_2ns[r.id];
                eval_map(ctx, starkinfo, pol_id, r.prime, next, modulas)
            } else {
                panic!("Invalid dom");
            }
        }
        "tmpExp" => {
            if dom == "n" {
                let pol_id = starkinfo.tmpexp_n[r.id];
                eval_map(ctx, starkinfo, pol_id, r.prime, next, modulas)
            } else {
                panic!("Invalid dom");
            }
        }
        _ => {
            panic!("Invalid reference type set {}", r.type_)
        }
    };
    body.exprs.push(val);
    body.exprs.push(Expr::new(Ops::Write, vec![], vec![e_dst], vec![]));
}

fn get_ref<F: FieldExtension>(
    ctx: &StarkContext<F>,
    starkinfo: &StarkInfo,
    r: &Node,
    dom: &str,
    next: usize,
    modulas: usize,
) -> Expr<F> {
    //log::trace!("get_ref: r {:?}  dom {} ", r, dom);
    match r.type_.as_str() {
        "tmp" => Expr::new(Ops::Refer, vec!["tmp".to_string()], vec![], vec![r.id, 0, modulas, 0]),
        "const" => {
            if dom == "n" {
                if r.prime {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_n".to_string()],
                        vec![],
                        vec![r.id, 1, modulas, starkinfo.n_constants],
                    )
                } else {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_n".to_string()],
                        vec![],
                        vec![r.id, 0, modulas, starkinfo.n_constants],
                    )
                }
            } else if dom == "2ns" {
                if r.prime {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_2ns".to_string()],
                        vec![],
                        vec![r.id, next, modulas, starkinfo.n_constants],
                    )
                } else {
                    Expr::new(
                        Ops::Refer,
                        vec!["const_2ns".to_string()],
                        vec![],
                        vec![r.id, 0, modulas, starkinfo.n_constants],
                    )
                }
            } else {
                panic!("Invalid dom");
            }
        }
        "cm" => {
            if dom == "n" {
                let pol_id = starkinfo.cm_n[r.id];
                eval_map(ctx, starkinfo, pol_id, r.prime, next, modulas)
            } else if dom == "2ns" {
                let pol_id = starkinfo.cm_2ns[r.id];
                eval_map(ctx, starkinfo, pol_id, r.prime, next, modulas)
            } else {
                panic!("Invalid dom");
            }
        }
        "tmpExp" => {
            if dom == "n" {
                let pol_id = starkinfo.tmpexp_n[r.id];
                eval_map(ctx, starkinfo, pol_id, r.prime, next, modulas)
            } else {
                panic!("Invalid dom");
            }
        }
        "number" => {
            let n_val = parse_pil_number(r.value.as_ref().unwrap());
            Expr::new(Ops::Vari(F::from(n_val)), vec![], vec![], vec![])
        }
        "public" => {
            Expr::new(Ops::Refer, vec!["publics".to_string()], vec![], vec![r.id, 0, modulas, 0])
        }
        "challenge" => {
            Expr::new(Ops::Refer, vec!["challenge".to_string()], vec![], vec![r.id, 0, modulas, 0])
        }
        "eval" => {
            Expr::new(Ops::Refer, vec!["evals".to_string()], vec![], vec![r.id, 0, modulas, 0])
        }
        "xDivXSubXi" => Expr::new(
            Ops::Refer,
            vec!["xDivXSubXi".to_string(), "3".to_string()],
            vec![],
            vec![0, 0, modulas, 3],
        ),
        "xDivXSubWXi" => Expr::new(
            Ops::Refer,
            vec!["xDivXSubWXi".to_string(), "3".to_string()],
            vec![],
            vec![0, 0, modulas, 3],
        ),
        "x" => {
            if dom == "n" {
                Expr::new(Ops::Refer, vec!["x_n".to_string()], vec![], vec![0, 0, modulas, 1])
            } else if dom == "2ns" {
                Expr::new(
                    Ops::Refer,
                    vec!["x_2ns".to_string()],
                    vec![],
                    vec![0, 0, modulas, 1], //i
                )
            } else {
                panic!("Invalid dom");
            }
        }
        "Zi" => Expr::new(Ops::Refer, vec!["Zi".to_string()], vec![], vec![0, 0, modulas, 1]),
        _ => panic!("Invalid reference type get, {}", r.type_),
    }
}

fn eval_map<F: FieldExtension>(
    _ctx: &StarkContext<F>,
    starkinfo: &StarkInfo,
    pol_id: usize,
    prime: bool,
    next: usize,
    modulas: usize,
) -> Expr<F> {
    let p = &starkinfo.var_pol_map[pol_id];
    //log::trace!("eval_map: {:?}", p);
    let offset = p.section_pos;
    let size = starkinfo.map_sectionsN.get(&p.section);
    let zero = 0;
    if p.dim == 1 {
        if prime {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone()],
                vec![],
                vec![offset, next, modulas, size],
            )
        } else {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone()],
                vec![],
                vec![offset, zero, modulas, size],
            )
        }
    } else if p.dim == 3 {
        if prime {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone(), "3".to_string()],
                vec![],
                vec![offset, next, modulas, size],
            )
        } else {
            Expr::new(
                Ops::Refer,
                vec![p.section.clone(), "3".to_string()],
                vec![],
                vec![offset, zero, modulas, size],
            )
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
}
