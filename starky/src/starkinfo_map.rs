use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::f3g::F3G;
use crate::helper::get_ks;
use crate::starkinfo::{CICTX, PECTX, StarkInfo};
use crate::starkinfo_codegen::{
    build_code, iterate_code, pil_code_gen, Calculated, Context, ContextF, EVIdx, Node, PolType, MapOffsetOrDeg, Segment, Section
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

        let fix_ref = |r: &mut Node, ctx: &mut ContextF| {
            if r.type_.as_str() == "cm" {
                let p1 = self.var_pol_map[self.cm_2ns[r.id.unwrap() as usize]];
                match p1.section.as_str() {
                    "cm1_2ns" => { r.type_ = "tree1".to_string(); },
                    "cm2_2ns" => { r.type_ = "tree2".to_string(); },
                    "cm3_2ns" => { r.type_ = "tree3".to_string(); },
                    _ => { panic!("Invalid cm section"); },
                }
                r.tree_pos = p1.section_pos;
                r.dim = p1.dim;
            } else if r.type_.as_str() == "q" {
                let p2 = self.var_pol_map[self.qs[r.id.unwrap() as usize]];
                r.type_ = "tree4".to_string();
                r.tree_pos = p2.section_pos;
                r.dim = p2.dim;
            }
        };

        iterate_code(&mut self.verifier_query_code, fix_ref, ctx);

        for i in 0..self.n_publics {
            if self.publics_code[i].tmp_used >= 0 { //FIXME
                set_code_demensions(self.publics_code[i], stark_struct, 1);
            }
        }

        /*
    for (let i=0; i<res.nPublics; i++) {
        if (res.publicsCode[i]) {
            setCodeDimensions(res.publicsCode[i], res, 1);
        }
    }

    setCodeDimensions(res.step2prev, res, 1);
    setCodeDimensions(res.step3prev,res, 1);
    setCodeDimensions(res.step4, res, 1);
    setCodeDimensions(res.step42ns, res, 1);
    setCodeDimensions(res.step52ns, res, 1);
    setCodeDimensions(res.verifierCode, res, 3);
    setCodeDimensions(res.verifierQueryCode, res, 1);
        */


        Ok(())
    }

    fn set_dim(r: &mut Node, dim: i32, tmp_dim: &mut Vec<i32>) {
        match r.type_.as_str() {
            "tmp" => { tmp_dim[r.id.unwrap()] = dim; r.dim = Some(dim); },
            "exp" | "cm" | "q" => {r.dim = Some(dim); },
            _ => { panic!("Invalid referenece type set {}", r.type_); }
        }
    }

    fn get_dim(r: &mut Node, tmp_dim: &mut Vec<i32>) {
        let mut d = 0;
        match r.type_.as_str() {
            "tmp" => {d = tmp_dim[r.id.unwrap()];},
            "tree1" | "tree2" | "tree3" | "tree4" => { d = r.dim.unwrap(); },

            "exp" => {
                d = if self.var_pol_map[self.exps_2ns[r.id.unwrap()]].tmp_used >= 0? {self.var_pol_map[self.exps_2ns[r.id.unwrap()]].dim} else {self.var_pol_map[self.exps_n[r.id.unwrap()].dim};
            },
            ""
        }
    }

    fn set_code_demensions (segment: &mut Segment, stark_struct: &StarkStruct, dim_x: i32) -> Result<()> {

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

    //FIXME: use it as Section's method
    fn set_section_field(name: &str, sec: &mut Section, value: i32) {
        match name {
            "cm1_n" => { sec.cm1_n = value; },
            "cm1_2ns" => { sec.cm1_2ns = value; },
            "cm2_n" => {sec.cm2_n = value; },
            "cm2_2ns" => { sec.cm2_2ns = value; },
            "cm3_n" => { sec.cm3_n = value; },
            "cm3_2ns" => { sec.cm3_2ns = value;  },
            "q_2ns" => {sec.q_2ns = value; },
            "exps_withq_n" => { sec.exps_withq_2ns = value; },
            "exps_withq_2ns" => { sec.exps_withq_2ns = value; },
            "exps_withoutq_n" => { sec.exps_withoutq_n = value; },
            "exps_withoutq_2ns" => {sec.exps_withoutq_2ns = value; },
            _ => { panic!("invalid domain {}", name); },
        }
    }

    fn map_section(&mut self) -> Result<()> {
        let names: [&str] = [
            "cm1_n",
            "cm1_2ns",
            "cm2_n",
            "cm2_2ns",
            "cm3_n",
            "cm3_2ns",
            "q_2ns",
            "exps_withq_n",
            "exps_withq_2ns",
            "exps_withoutq_n",
            "exps_withoutq_2ns",
        ];

        for s in names.iter() {
            let mut p = 0;
            for e in 1..=3 {
                for pp in self.var_pol_map.iter_mut() {
                    if pp.section == s && pp.dim == e {
                        pp.section_pos = p;
                        p += e;
                    }
                }
                if e == 1 {
                    Self::set_section_field(s, &mut self.map_sectionsN1, p);
                }
                if e == 3 {
                    Self::set_section_field(s, &mut self.map_sectionsN, p);
                }
            }
            let t = (self.map_sectionsN[s] - self.map_sectionsN1[s] ) / 3;
            Self::set_section_field(s, &mut self.map_sectionsN3, t);
        }
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