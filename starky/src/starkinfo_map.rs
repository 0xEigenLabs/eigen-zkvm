use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::f3g::F3G;
use crate::helper::get_ks;
use crate::starkinfo::{CICTX, PECTX, StarkInfo};
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Calculated, Context, ContextF, EVIdx, Node, PolType, MapOffsetOrDeg, Segment
};
use crate::types::{PIL, Expression, StarkStruct};
use std::collections::HashMap;

impl StarkInfo {

    pub fn map(&mut self, ctx: &mut Context, stark_struct: &StarkStruct) -> Result<()> {
        let mut add_pol = |pol_type: PolType| -> i32 {
            self.var_pol_map.push(pol_type);
            (self.var_pol_map.len() - 1) as i32
        };

        ctx.pil.cm_dims = vec![0i32; (self.n_cm1 + self.n_cm2 + self.n_cm3 + self.n_cm4) as usize]; //FIXME
        for i in 0..self.n_cm1 {
            let pp_n = add_pol(PolType{ section: "cm1_n".to_string(), dim: 1, exp_id: -1 });
            let pp_2ns = add_pol(PolType{ section: "cm1_2ns".to_string(), dim: 1, exp_id: -1 });
            self.cm_n.push(pp_n.clone());
            self.cm_2ns.push(pp_2ns.clone());
            self.map_sections.cm1_n.push(pp_n);
            self.map_sections.cm1_2ns.push(pp_2ns);
            ctx.pil.cm_dims[i as usize] = 1
        }

        for (i, pu) in self.pu_ctx.iter().enumerate() {
            let dim = std::cmp::max(
                Self::get_exp_dim(ctx.pil, &ctx.pil.expressions[pu.f_exp_id as usize]),
                Self::get_exp_dim(ctx.pil, &ctx.pil.expressions[pu.t_exp_id as usize]),
            );

            let pph1_n = add_pol(PolType{section: "cm2_n".to_string(), dim: dim, exp_id: -1});
            let pph2_2ns = add_pol(PolType{section: "cm2_2ns".to_string(), dim: dim, exp_id: -1});

            self.cm_n.push(pph1_n.clone());
            self.cm_n.push(pph2_2ns.clone());

            self.map_sections.cm2_n.push(pph1_n);
            self.map_sections.cm2_2ns.push(pph2_2ns);

            ctx.pil.cm_dims[self.n_cm1 as usize + i * 2] = dim;

            let pph2_n = add_pol(PolType{section: "cm2_n".to_string(), dim: dim, exp_id: -1});
            let pph2_2ns = add_pol(PolType{section: "cm2_2ns".to_string(), dim: dim, exp_id: -1});

            self.cm_n.push(pph2_n.clone());
            self.cm_2ns.push(pph2_2ns.clone());
            self.map_sections.cm2_n.push(pph2_n);
            self.map_sections.cm2_2ns.push(pph2_2ns);
            ctx.pil.cm_dims[self.n_cm1 as usize + i*2+1] = dim;
        }

        for i in 0..self.n_cm3 {
            let ppz_n = add_pol(PolType{section: "cm3_n".to_string(), dim: 3, exp_id: -1});
            let ppz_2ns = add_pol(PolType{section: "cm3_2ns".to_string(), dim: 3, exp_id: -1});
            self.cm_n.push(ppz_n.clone());
            self.cm_2ns.push(ppz_2ns.clone());
            self.map_sections.cm3_n.push(ppz_n);
            self.map_sections.cm3_2ns.push(ppz_2ns);
            ctx.pil.cm_dims[(self.n_cm1 + self.n_cm2 + i) as usize] = 3;
        }

        let mut q_dims: Vec<i32> = vec![]; // FIXME: useless??
        ctx.pil.q2exp = vec![];

        for (i, e) in ctx.pil.expressions.iter().enumerate() {
            if e.idQ.is_some() {
                q_dims[e.idQ.unwrap() as usize] = Self::get_exp_dim(ctx.pil, &ctx.pil.expressions[i]);
                ctx.pil.q2exp[e.idQ.unwrap() as usize] = i as i32;
            }
        }

        let mut used_qs = HashMap::<i32, bool>::new();

        for (i, ev) in self.ev_map.iter().enumerate() {
            if ev.type_.as_str() == "q" {
                used_qs.insert(ev.id.unwrap(), true);
            }
        }

        for i in 0..ctx.pil.nQ {
            let mut dim = 0;
            if used_qs[&i] {
                dim = Self::get_exp_dim(ctx.pil, &ctx.pil.expressions[ctx.pil.q2exp[i as usize] as usize]);
            } else {
                dim = 0;
                //expression_warning(); FIXME
            }
            let ppq = add_pol(PolType{section: "q_2ns".to_string(), dim: dim, exp_id: ctx.pil.q2exp[i as usize]});
            self.qs.push(ppq.clone() as i32);
            if dim > 0 {
                self.map_sections.q_2ns.push(ppq as i32);
            }
        }

        for (i, e) in ctx.pil.expressions.iter().enumerate() {
            if e.idQ.is_some() {
                let dim = Self::get_exp_dim(&ctx.pil, &ctx.pil.expressions[i]);
                let pp_n = add_pol(PolType{section: "exps_withq_n".to_string(), dim: dim, exp_id: i as i32});
                let pp_2ns = add_pol(PolType{section: "exps_withq_2ns".to_string(), dim: dim, exp_id: i as i32});
                self.map_sections.exps_withq_n.push(pp_n.clone());
                self.map_sections.exps_withq_2ns.push(pp_2ns.clone());
                self.exps_n.push(pp_n);
                self.exps_2ns.push(pp_2ns);
            } else if e.keep.is_some() {
                let dim = Self::get_exp_dim(&ctx.pil, &ctx.pil.expressions[i]);
                let pp_n = add_pol(PolType{section: "exps_withq_n".to_string(), dim: dim, exp_id: i as i32});
                self.map_sections.exps_withq_n.push(pp_n.clone());
                self.exps_n.push(pp_n);
            } else if e.keep2ns.is_some() {
                let dim = Self::get_exp_dim(&ctx.pil, &ctx.pil.expressions[i]);
                let pp_2ns = add_pol(PolType{section: "exps_withq_2ns".to_string(), dim: dim, exp_id: i as i32});
                self.map_sections.exps_withq_2ns.push(pp_2ns.clone());
                self.exps_2ns.push(pp_2ns);
            } else {
                self.exps_n[i] = -1;  //null
                self.exps_2ns[i] = -1;
            }
        }

        self.map_section()?;
        let N = 1 << stark_struct.nBits;
        let Next = 1 << stark_struct.nBitsExt;

        self.map_offsets = MapOffsetOrDeg {
             cm1_n: 0,
             cm2_n: self.map_offsets.cm1_n + N * self.map_sectionsN.cm1_n,
             cm3_n: self.map_offsets.cm2_n + N * self.map_sectionsN.cm2_n,
             exps_withq_n: self.map_offsets.cm3_n + N * self.map_sectionsN.cm3_n,
             exps_withoutq_n: self.map_offsets.exps_withq_n + N * self.map_sectionsN.exps_withq_n,
             cm1_2ns: self.map_offsets.exps_withoutq_n +  N * self.map_sectionsN.exps_withoutq_n,
             cm2_2ns: self.map_offsets.cm1_2ns +  N * self.map_sectionsN.cm1_2ns,
             cm3_2ns: self.map_offsets.cm2_2ns +  N * self.map_sectionsN.cm2_2ns,
             q_2ns: self.map_offsets.cm3_2ns +  Next * self.map_sectionsN.cm3_2ns,
             exps_withq_2ns: self.map_offsets.q_2ns + Next * self.map_sectionsN.q_2ns,
             exps_withoutq_2ns: self.map_offsets.exps_withq_2ns + Next * self.map_sectionsN.exps_withq_2ns,
             map_total_n: self.map_offsets.exps_withoutq_2ns +  Next * self.map_sectionsN.exps_withoutq_2ns,
        };

        self.map_deg = MapOffsetOrDeg {
           cm1_n: N,
           cm2_n: N,
           cm3_n: N,
           exps_withq_n: N,
           exps_withoutq_n: N,
           cm1_2ns: Next,
           cm2_2ns: Next,
           cm3_2ns: Next,
           q_2ns: Next,
           exps_withq_2ns: Next,
           exps_withoutq_2ns: Next,
           map_total_n: -1,
        };

        for p in self.publics_code.iter_mut() {
            self.fix_prover_code(p, "n");
        }

        self.fix_prover_code(&mut self.step2prev, "n");
        self.fix_prover_code(&mut self.step3prev, "n");
        self.fix_prover_code(&mut self.step4, "n");
        self.fix_prover_code(&mut self.step42ns, "2ns");
        self.fix_prover_code(&mut self.step52ns, "2ns");

        Ok(())
    }

    fn fix_prover_code(&mut self, segment: &mut Segment, dom: &str) {
        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: segment.tmp_used,
            ev_idx: EVIdx::new(),
            ev_map: Vec::new(),
            dom: dom.to_string(),
        };


        let fix_ref = |r: &mut Node, ctx: &mut ContextF| {

            match r.type_.as_str() {
                "cm" =>  {
                    if ctx.dom.as_str() == "n" {
                        r.p = self.cm_n[r.id.unwrap() as usize];
                    } else if ctx.dom.as_str() == "2ns" {
                        r.p = self.cm_2ns[r.id.unwrap() as usize];
                    } else {
                        panic!("Invalid domain {}", ctx.dom);
                    }
                },

                "q" => {
                    if ctx.dom.as_str() == "n" {
                        panic!("Accession q in domain n");
                    } else if ctx.dom.as_str() == "2ns" {
                        r.p = self.qs[r.id.unwrap() as usize];
                    } else {
                        panic!("Invalid domain {}", ctx.dom);
                    }
                },

                "exp" => {
                    if ctx.pil.expressions[r.id.unwrap() as usize].idQ.is_some() { //FIXME ctx has no pil
                        if ctx.dom.as_str() == "n" {
                            r.p = self.exps_n[r.id.unwrap() as usize];
                        } else if ctx.dom.as_str() == "2ns" {
                            r.p = self.exps_2ns[r.id.unwrap() as usize];
                        } else {
                            panic!("Invalid domain {}", ctx.dom);
                        }
                    } else if ctx.pil.expressions[r.id.unwrap() as usize].keep.is_some() && ctx.dom.as_str() == "n" {
                        r.p = self.exps_n[r.id.unwrap() as usize];
                    } else if ctx.pil.expressions[r.id.unwrap() as usize].keep2ns.is_some() {
                        if ctx.dom.as_str() == "n" {
                            panic!("Accession q in domain n");
                        }
                    } else {
                        let p = if r.prime.is_some() {1} else {0};
                        if ctx.exp_map.get(&(p, r.id.unwrap())).is_none() {
                            ctx.exp_map.insert((p, r.id.unwrap()), ctx.tmp_used);
                            ctx.tmp_used += 1;
                        }
                        r.type_ = "tmp".to_string();
                        r.exp_id = r.id.unwrap();
                        r.id = ctx.exp_map.get(&(p, r.id.unwrap()));
                    }
                },
                "const" | "number" | "challenge" | "public" | "tmp" | "Zi" | "xDivXSubXi" | "xDivXSubWXi" | "eval" | "x" => {},
                _ => {panic!("Invalid reference type {}", r.type_);}

            }

        };


        iterate_code(segment, fix_ref, &mut ctx_f);
        segment.tmp_used = ctx_f.tmp_used;
    }

    fn map_section(&mut self) -> Result<()> {

        //map sections  cm1_n
        //map sections  cm1_2ns
        //map sections  cm2_n
        //map sections  cm2_2ns
        //map sections  cm3_n
        //map sections  cm3_2ns
        //map sections  q_2ns
        //map sections  exps_withq_n
        //map sections  exps_withq_2ns
        //map sections  exps_withoutq_n
        //map sections  exps_withoutq_2ns

        let mut p = 0;
        for e in 1..=3 {


            for pp in self.var_pol_map.iter_mut() {
                if pp.section == s && pp.dim == e {
                    pp.section_pos = p;
                    p += e;
                }
            }
            if e == 1 {
                self.map_sectionsN1[s] == p;
            }
            if e == 3 {
                self.map_sectionsN[s] = p;
            }
        }

        self.map_sectionsN3[s] = (self.map_sectionsN[s] - self.map_sectionsN1[s] ) / 3;

        Ok(())
    }

    fn get_exp_dim(pil: &PIL, exp: &Expression) -> i32 {
        let exp_id = exp.id.unwrap();
        match exp.op.as_str() {
             "add" | "sub" | "mul" | "addc" | "mulc" | "neg" => {
                 let mut md = 1;
                 let values = exp.values.as_ref().unwrap();
                 for v in values.iter() {
                     let d = Self::get_exp_dim(pil, v);
                     if d>md {md=d}
                 }
                 md
            },
             "cm" => pil.cm_dims[exp_id as usize],
             "const" => 1,
             "exp" => Self::get_exp_dim(pil, &pil.expressions[exp_id as usize]),
             "q" => Self::get_exp_dim(pil, &pil.expressions[pil.q2exp[exp_id as usize] as usize]),
             "number" | "public" => 1,
             "challenge" | "eval" | "xDivXSubXi" | "xDivXSubWXi" => 3,
             "x" => {1},
             _ => panic!("Exp op not defined: {}", exp.op),
        }
    }

}
