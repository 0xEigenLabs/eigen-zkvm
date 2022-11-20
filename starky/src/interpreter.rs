use crate::stark_gen::StarkContext;
use crate::stark_codegen::Subcode;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;
use crate::starkinfo_codegen::Node;


pub fn compile_code(ctx: &StarkContext, dom: &str, code: &mut Vec<Subcode>, ret: bool) -> Box<dyn Fn(i32) -> BaseElement> {
    Box::new(| input_i: i32 | -> BaseElement {
        compile(ctx, dom, code, ret, input_i)
    })
}

fn compile(ctx: &StarkContext, dom: &str, code: &mut Vec<Subcode>, ret: bool, input_i: i32) -> BaseElement {
    let next = if dom == "n"  {1} else { 1 << (ctx.nbits_ext - ctx.nbits) };
    let next = next as usize;

    let input_i = input_i as usize;

    let N = if dom == "n"  { 1 << ctx.nbits } else { 1<< ctx.nbits_ext };
    let N = N as usize;

    for i in 0..code.len() {
        let src = vec![BaseElement::ZERO; code[i].src.len()];
        for j in 0..code[i].src.len() {
            src[j] = get_ref(ctx, &code[i].src[j], &dom, &next, &N, &input_i);
        }

        let exp =
            match code[i].op.as_str() {
                "add" => { src[0] + src[1] },
                "sub" => { src[0] - src[1] },
                "mul" => { src[0] * src[1] },
                "copy" => { src[0] },
            };
        set_ref(&mut code[i].dest, exp, dom, &next, &N, &input_i);
    }

    if ret {
        get_ref(ctx, code[code.length-1].dest, dom, next, N, input_i)
    } else {
        BaseElement::ZERO
    }
}

fn set_ref(ctx: &StarkContext, r: &mut Node, val: BaseElement, dom: &str, next: &usize, N: &usize, i: &usize) -> Expr {

    let e_dst = match r.type_.as_str() {
        "tmp" => { Expr{dom: "".to_string(), op: "assign".to_string(), oprands: [ctx.tmp[r.id], BaseElement::ZERO, BaseElement::ZERO]} },
        "exp" => {
            if dom == "n" {
                eval_map(&ctx.stark_setup.starkinfo.exps_n[r.id], &r.prime, &next, &N, &i)
            } else if dom == "2ns" {
                eval_map(&ctx.stark_setup.starkinfo.exps_2ns[r.id], &r.prime, &next, &N, &i)
            } else {
                panic!("Invalid dom");
            }
        },
        "q" => {
            if dom == "n" {
                panic!("Accesssing q in domain n");
            } else if dom == "2ns" {
                eval_map(&ctx.stark_setup.starkinfo.q_2ns[r.id], &r.prime, next, N, i)
            } else {
                panic!("Invalid dom");
            }
        },

        _ => { panic!("Invalid reference type set {}", r.type_) }
    };

    Expr{dom: "".to_string(), op: "assign".to_string(), oprands: [e_dst, val, BaseElement::ZERO]}
}

fn get_ref(ctx: &StarkContext, r: &Node, dom: &str, next: &usize, N: &usize, i: &usize) -> BaseElement {
    match r.type_.as_str() {
        "tmp" => {ctx.const_n[r.id]},
        "const" => {
            if dom == "n" {
                if r.prime {
                    return ctx.const_n[r.id + (i + 1)%N * ctx.stark_setup.starkinfo.n_contants as usize];
                } else {
                    return ctx.const_n[r.id + i * ctx.stark_setup.starkinfo.n_contants as usize];
                }
            } else if dom == "2ns" {
                if r.prime {
                    return ctx.const_2ns[r.id + (i + 1)%N * ctx.stark_setup.starkinfo.n_contants as usize];
                } else {
                    return ctx.const_2ns[r.id + i * ctx.stark_setup.starkinfo.n_contants];
                }
            } else {
                panic!("Invalid dom");
            }
        },
        "cm" => {
            if dom == "n" {
                return eval_map(ctx.stark_setup.starkinfo.cm_n[r.id], r.prime, next, N);
            } else if dom == "2ns" {
                return eval_map(ctx.stark_setup.starkinfo.cm_2ns[r.id], r.prime, next, N);
            } else {
                panic!("Invalid dom");
            }
        },
        "q" => {
            if dom == "n" {
                panic!("Accesssing q in domain n");
            } else if dom == "2ns" {
                return eval_map(ctx.stark_setup.starkinfo.cm_2ns[r.id], r.prime);
            } else {
                panic!("Invalid dom");
            }
        },
        "exp" => {
            if dom == "n" {
                return eval_map(ctx.stark_setup.starkinfo.exps_n[r.id], r.prime, next, N);
            } else if dom == "2ns" {
                return eval_map(ctx.stark_setup.starkinfo.exps_2ns[r.id], r.prime, next, N);
            } else {
                panic!("Invalid dom");
            }
        },

        "number" => { BaseElement::from(r.value) },

        "public" => { ctx.publics[r.id] },
        "challenge" => { ctx.challenge[r.id] },
        "eval" => {ctx.evals[r.id]},
        "xDivXSubXi" => { [ctx.xDivXSubXi[3*i], ctx.xDivXSubXi[3*i+1], ctx.xDivXSubXi[3*i+2]] },
        "xDivXSubWXi" => { [ctx.xDivXSubWXi[3*i], ctx.xDivXSubWXi[3*i+1], ctx.xDivXSubWXi[3*i+2]] },
        "x" =>  {
            if dom == "n" {
                return ctx.x_n[i];
            } else if dom == "2ns" {
                return ctx.x_2ns[i];
            } else {
                panic!("Invalid dom");
            }
        },
        "Zi" => { ctx.Zi(i) },  // FIXME,
        _ => panic!("Invalid reference type get, {}", r.type_),
    }
}

fn eval_map(ctx: &StarkContext, pol_id: i32, prime: bool, next: &usize, N: &usize, i: &usize) -> Vec<BaseElement> {
    let p = &ctx.stark_setup.starkinfo.var_pol_map[pol_id as usize];
    let offset = p.section_pos as usize;
    let size = ctx.stark_setup.starkinfo.map_sectionsN.get(&p.section) as usize;
    if p.dim == 1 {
        if prime {
            match p.section.as_str() {
                "cm1_n" => vec![ctx.cm1_n[offset + ((i + next) % N) * size]],
                "cm1_2ns" => vec![ctx.cm1_2ns[offset + ((i + next) % N) * size]],
                "cm2_n" => vec![ctx.cm2_n[offset + ((i + next) % N) * size]],
                "cm2_2ns" => vec![ctx.cm2_2ns[offset + ((i + next) % N) * size]],
                "cm3_n" => vec![ctx.cm3_n[offset + ((i + next) % N) * size]],
                "cm3_2ns" => vec![ctx.cm3_2ns[offset + ((i + next) % N) * size]],
                "q_2ns" => vec![ctx.q_2ns[offset + ((i + next) % N) * size]],
                "exps_withq_2ns" => vec![ctx.exps_withq_2ns[offset + ((i + next) % N) * size]],
                _ => { panic!("Invalid section {}", p.section); }
            }
        } else {
            match p.section.as_str() {
                "cm1_n" => vec![ctx.cm1_n[offset + i * size]],
                "cm1_2ns" => vec![ctx.cm1_2ns[offset + i * size]],
                "cm2_n" => vec![ctx.cm2_n[offset + i * size]],
                "cm2_2ns" => vec![ctx.cm2_2ns[offset + i * size]],
                "cm3_n" => vec![ctx.cm3_n[offset + i * size]],
                "cm3_2ns" => vec![ctx.cm3_2ns[offset + i * size]],
                "q_2ns" => vec![ctx.q_2ns[offset + i * size]],
                "exps_withq_2ns" => vec![ctx.exps_withq_2ns[offset + i * size]],
                _ => { panic!("Invalid section {}", p.section); }
            }
        }
    } else if p.dim == 3 {
        if prime {
            match p.section.as_str() {
                "cm1_n" => vec![
                    ctx.cm1_n[offset + ((i + next) % N) * size],
                    ctx.cm1_n[offset + ((i + next) % N) * size + 1],
                    ctx.cm1_n[offset + ((i + next) % N) * size + 2],
                ],
                "cm1_2ns" => vec![
                    ctx.cm1_2ns[offset + ((i + next) % N) * size],
                    ctx.cm1_2ns[offset + ((i + next) % N) * size + 1],
                    ctx.cm1_2ns[offset + ((i + next) % N) * size + 2],
                ],
                "cm2_n" => vec![
                    ctx.cm2_n[offset + ((i + next) % N) * size],
                    ctx.cm2_n[offset + ((i + next) % N) * size + 1],
                    ctx.cm2_n[offset + ((i + next) % N) * size + 2],
                ],
                "cm2_2ns" => vec![
                    ctx.cm2_2ns[offset + ((i + next) % N) * size],
                    ctx.cm2_2ns[offset + ((i + next) % N) * size + 1],
                    ctx.cm2_2ns[offset + ((i + next) % N) * size + 2],
                ],
                "cm3_n" => vec![
                    ctx.cm3_n[offset + ((i + next) % N) * size],
                    ctx.cm3_n[offset + ((i + next) % N) * size + 1],
                    ctx.cm3_n[offset + ((i + next) % N) * size + 2],
                ],
                "cm3_2ns" => vec![
                    ctx.cm3_2ns[offset + ((i + next) % N) * size],
                    ctx.cm3_2ns[offset + ((i + next) % N) * size + 1],
                    ctx.cm3_2ns[offset + ((i + next) % N) * size + 2],
                ],
                "q_2ns" => vec![
                    ctx.q_2ns[offset + ((i + next) % N) * size],
                    ctx.q_2ns[offset + ((i + next) % N) * size + 1],
                    ctx.q_2ns[offset + ((i + next) % N) * size + 2],
                ],
                "exps_withq_2ns" => vec![
                    ctx.exps_withq_2ns[offset + ((i + next) % N) * size],
                    ctx.exps_withq_2ns[offset + ((i + next) % N) * size + 1],
                    ctx.exps_withq_2ns[offset + ((i + next) % N) * size + 2],
                ],
                _ => { panic!("Invalid section {}", p.section); }
            }
        } else {
            match p.section.as_str() {
                "cm1_n" => vec![
                    ctx.cm1_n[offset + i * size],
                    ctx.cm1_n[offset + i * size + 1],
                    ctx.cm1_n[offset + i * size + 2],
                ],
                "cm1_2ns" => vec![
                    ctx.cm1_2ns[offset + i * size],
                    ctx.cm1_2ns[offset + i * size + 1],
                    ctx.cm1_2ns[offset + i * size + 2],
                ],
                "cm2_n" => vec![
                    ctx.cm2_n[offset + i * size],
                    ctx.cm2_n[offset + i * size + 1],
                    ctx.cm2_n[offset + i * size + 2],
                ],
                "cm2_2ns" => vec![
                    ctx.cm2_2ns[offset + i * size],
                    ctx.cm2_2ns[offset + i * size + 1],
                    ctx.cm2_2ns[offset + i * size + 2],
                ],
                "cm3_n" => vec![
                    ctx.cm3_n[offset + i * size],
                    ctx.cm3_n[offset + i * size + 1],
                    ctx.cm3_n[offset + i * size + 2],
                ],
                "cm3_2ns" => vec![
                    ctx.cm3_2ns[offset + i * size],
                    ctx.cm3_2ns[offset + i * size + 1],
                    ctx.cm3_2ns[offset + i * size + 2],
                ],
                "q_2ns" => vec![
                    ctx.q_2ns[offset + i * size],
                    ctx.q_2ns[offset + i * size + 1],
                    ctx.q_2ns[offset + i * size + 2],
                ],
                "exps_withq_2ns" => vec![
                    ctx.exps_withq_2ns[offset + i * size],
                    ctx.exps_withq_2ns[offset + i * size + 1],
                    ctx.exps_withq_2ns[offset + i * size + 2],
                ],
                _ => { panic!("Invalid section {}", p.section); }
            }
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
}

