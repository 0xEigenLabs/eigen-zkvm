#![allow(non_snake_case)]

use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{iterate_code, ContextF, Index, Node, PolType, Section, Segment};
use crate::types::{Expression, StarkStruct, PIL};
use anyhow::Result;
use std::collections::HashMap;

impl StarkInfo {
    pub fn map(
        &mut self,
        pil: &mut PIL,
        stark_struct: &StarkStruct,
        program: &mut Program,
    ) -> Result<()> {
        let mut add_pol = |pol_type: PolType| -> usize {
            self.var_pol_map.push(pol_type);
            self.var_pol_map.len() - 1
        };

        let mut tmpexps: HashMap<usize, usize> = HashMap::new();
        let im_exps_none =
            |id: &usize| -> bool { !self.im_exps.contains_key(id) || !self.im_exps[id] };

        pil.cm_dims = vec![0usize; self.n_cm1 + self.n_cm2 + self.n_cm3 + self.n_cm4]; //FIXME
        for i in 0..self.n_cm1 {
            let pp_n = add_pol(PolType {
                section: "cm1_n".to_string(),
                dim: 1,
                exp_id: 0,
                section_pos: 0,
            });
            let pp_2ns = add_pol(PolType {
                section: "cm1_2ns".to_string(),
                dim: 1,
                exp_id: 0,
                section_pos: 0,
            });
            self.cm_n.push(pp_n);
            self.cm_2ns.push(pp_2ns);
            self.map_sections.cm1_n.push(pp_n);
            self.map_sections.cm1_2ns.push(pp_2ns);
            pil.cm_dims[i] = 1
        }

        //log::trace!("pu: {:?}", self.pu_ctx);
        for (i, pu) in self.pu_ctx.iter().enumerate() {
            let dim = std::cmp::max(
                Self::get_exp_dim(pil, &pil.expressions[pu.f_exp_id]),
                Self::get_exp_dim(pil, &pil.expressions[pu.t_exp_id]),
            );

            let pph1_n =
                add_pol(PolType { section: "cm2_n".to_string(), dim, exp_id: 0, section_pos: 0 });
            let pph1_2ns =
                add_pol(PolType { section: "cm2_2ns".to_string(), dim, exp_id: 0, section_pos: 0 });

            self.cm_n.push(pph1_n);
            self.cm_2ns.push(pph1_2ns);
            self.map_sections.cm2_n.push(pph1_n);
            self.map_sections.cm2_2ns.push(pph1_2ns);
            pil.cm_dims[self.n_cm1 + i * 2] = dim;

            let pph2_n =
                add_pol(PolType { section: "cm2_n".to_string(), dim, exp_id: 0, section_pos: 0 });
            let pph2_2ns =
                add_pol(PolType { section: "cm2_2ns".to_string(), dim, exp_id: 0, section_pos: 0 });

            self.cm_n.push(pph2_n);
            self.cm_2ns.push(pph2_2ns);
            self.map_sections.cm2_n.push(pph2_n);
            self.map_sections.cm2_2ns.push(pph2_2ns);
            pil.cm_dims[self.n_cm1 + i * 2 + 1] = dim;

            if im_exps_none(&pu.f_exp_id) && !tmpexps.contains_key(&pu.f_exp_id) {
                tmpexps.insert(pu.f_exp_id, self.tmpexp_n.len());
                let ppf_n = add_pol(PolType {
                    section: "tmpexp_n".to_string(),
                    dim,
                    exp_id: 0,
                    section_pos: 0,
                });
                self.tmpexp_n.push(ppf_n);
                self.map_sections.tmpexp_n.push(ppf_n);
                self.exp2pol.insert(pu.f_exp_id, ppf_n);
            }

            if im_exps_none(&pu.t_exp_id) && !tmpexps.contains_key(&pu.t_exp_id) {
                tmpexps.insert(pu.t_exp_id, self.tmpexp_n.len());
                let ppt_n = add_pol(PolType {
                    section: "tmpexp_n".to_string(),
                    dim,
                    exp_id: 0,
                    section_pos: 0,
                });
                self.tmpexp_n.push(ppt_n);
                self.map_sections.tmpexp_n.push(ppt_n);
                self.exp2pol.insert(pu.t_exp_id, ppt_n);
            }
        }

        for i in 0..(self.pu_ctx.len() + self.pe_ctx.len() + self.ci_ctx.len()) {
            let o;
            if i < self.pu_ctx.len() {
                o = &self.pu_ctx[i];
            } else if i < (self.pu_ctx.len() + self.pe_ctx.len()) {
                o = &self.pe_ctx[i - self.pu_ctx.len()];
            } else {
                o = &self.ci_ctx[i - self.pu_ctx.len() - self.pe_ctx.len()];
            }

            let ppz_n = add_pol(PolType {
                section: "cm3_n".to_string(),
                dim: 3,
                exp_id: 0,
                section_pos: 0,
            });
            let ppz_2ns = add_pol(PolType {
                section: "cm3_2ns".to_string(),
                dim: 3,
                exp_id: 0,
                section_pos: 0,
            });
            self.cm_n.push(ppz_n);
            self.cm_2ns.push(ppz_2ns);
            self.map_sections.cm3_n.push(ppz_n);
            self.map_sections.cm3_2ns.push(ppz_2ns);
            pil.cm_dims[self.n_cm1 + self.n_cm2 + i] = 3;

            if im_exps_none(&o.num_id) && !tmpexps.contains_key(&o.num_id) {
                tmpexps.insert(o.num_id, self.tmpexp_n.len());
                let pp_num_n = add_pol(PolType {
                    section: "tmpexp_n".to_string(),
                    dim: 3,
                    exp_id: 0,
                    section_pos: 0,
                });

                self.tmpexp_n.push(pp_num_n);
                self.map_sections.tmpexp_n.push(pp_num_n);
                self.exp2pol.insert(o.num_id, pp_num_n);
            }

            if im_exps_none(&o.den_id) && !tmpexps.contains_key(&o.den_id) {
                tmpexps.insert(o.den_id, self.tmpexp_n.len());
                let pp_den_n = add_pol(PolType {
                    section: "tmpexp_n".to_string(),
                    dim: 3,
                    exp_id: 0,
                    section_pos: 0,
                });

                self.tmpexp_n.push(pp_den_n);
                self.map_sections.tmpexp_n.push(pp_den_n);
                self.exp2pol.insert(o.den_id, pp_den_n);
            }
        }

        for i in 0..self.im_exps_list.len() {
            let dim = Self::get_exp_dim(pil, &pil.expressions[self.im_exps_list[i]]);

            let ppz_n =
                add_pol(PolType { section: "cm3_n".to_string(), dim, exp_id: 0, section_pos: 0 });

            let ppz_2ns =
                add_pol(PolType { section: "cm3_2ns".to_string(), dim, exp_id: 0, section_pos: 0 });

            self.cm_n.push(ppz_n);
            self.cm_2ns.push(ppz_2ns);
            self.map_sections.cm3_n.push(ppz_n);
            self.map_sections.cm3_2ns.push(ppz_2ns);
            pil.cm_dims[self.n_cm1 + self.n_cm2 + i] = dim;
            self.exp2pol.insert(self.im_exps_list[i], ppz_n);
        }

        self.q_dim = Self::get_exp_dim(pil, &pil.expressions[self.c_exp]);

        for i in 0..self.q_deg {
            let ppz_n = add_pol(PolType {
                section: "cm4_n".to_string(),
                dim: self.q_dim,
                exp_id: 0,
                section_pos: 0,
            });

            let ppz_2ns = add_pol(PolType {
                section: "cm4_2ns".to_string(),
                dim: self.q_dim,
                exp_id: 0,
                section_pos: 0,
            });

            self.cm_n.push(ppz_n);
            self.cm_2ns.push(ppz_2ns);
            self.map_sections.cm4_n.push(ppz_n);
            self.map_sections.cm4_2ns.push(ppz_2ns);
            pil.cm_dims[self.n_cm1 + self.n_cm2 + self.n_cm3 + i] = self.q_dim;
        }

        let ppq_2ns = add_pol(PolType {
            section: "q_2ns".to_string(),
            dim: self.q_dim,
            exp_id: 0,
            section_pos: 0,
        });
        self.q_2ns.push(ppq_2ns);

        let ppf_2ns =
            add_pol(PolType { section: "f_2ns".to_string(), dim: 3, exp_id: 0, section_pos: 0 });
        self.f_2ns.push(ppf_2ns);

        //log::trace!("cm_dims: {:?}", pil.cm_dims);
        self.map_section()?;
        let N = 1 << stark_struct.nBits;
        let Next = 1 << stark_struct.nBitsExt;

        self.map_offsets = Index::default();
        self.map_offsets.cm1_n = 0;
        self.map_offsets.cm2_n = self.map_offsets.cm1_n + N * self.map_sectionsN.cm1_n;
        self.map_offsets.cm3_n = self.map_offsets.cm2_n + N * self.map_sectionsN.cm2_n;
        self.map_offsets.cm4_n = self.map_offsets.cm3_n + N * self.map_sectionsN.cm3_n;
        self.map_offsets.tmpexp_n = self.map_offsets.cm4_n + N * self.map_sectionsN.cm4_n;
        self.map_offsets.cm1_2ns = self.map_offsets.tmpexp_n + N * self.map_sectionsN.tmpexp_n;
        self.map_offsets.cm2_2ns = self.map_offsets.cm1_2ns + Next * self.map_sectionsN.cm1_2ns;
        self.map_offsets.cm3_2ns = self.map_offsets.cm2_2ns + Next * self.map_sectionsN.cm2_2ns;
        self.map_offsets.cm4_2ns = self.map_offsets.cm3_2ns + Next * self.map_sectionsN.cm3_2ns;
        self.map_offsets.q_2ns = self.map_offsets.cm4_2ns + Next * self.map_sectionsN.cm4_2ns;
        self.map_offsets.f_2ns = self.map_offsets.q_2ns + Next * self.map_sectionsN.q_2ns;
        self.map_total_n = self.map_offsets.f_2ns + Next * self.map_sectionsN.f_2ns;
        self.map_deg = Index {
            cm1_n: N,
            cm2_n: N,
            cm3_n: N,
            cm4_n: N,
            tmpexp_n: N,
            cm1_2ns: Next,
            cm2_2ns: Next,
            cm3_2ns: Next,
            cm4_2ns: Next,
            q_2ns: Next,
            f_2ns: Next,
        };

        for i in 0..program.publics_code.len() {
            self.fix_prover_code(&mut program.publics_code[i], "n", pil, &mut tmpexps);
        }

        self.fix_prover_code(&mut program.step2prev, "n", pil, &mut tmpexps);
        self.fix_prover_code(&mut program.step3prev, "n", pil, &mut tmpexps);
        self.fix_prover_code(&mut program.step3, "n", pil, &mut tmpexps);
        self.fix_prover_code(&mut program.step42ns, "2ns", pil, &mut tmpexps);
        self.fix_prover_code(&mut program.step52ns, "2ns", pil, &mut tmpexps);
        self.fix_prover_code(&mut program.verifier_query_code, "2ns", pil, &mut tmpexps);

        let fix_ref = |r: &mut Node, ctx: &mut ContextF, _pil: &mut PIL| {
            if r.type_.as_str() == "cm" {
                let p1 = &ctx.starkinfo.var_pol_map[ctx.starkinfo.cm_2ns[r.id]];
                match p1.section.as_str() {
                    "cm1_2ns" => {
                        r.type_ = "tree1".to_string();
                    }
                    "cm2_2ns" => {
                        r.type_ = "tree2".to_string();
                    }
                    "cm3_2ns" => {
                        r.type_ = "tree3".to_string();
                    }
                    "cm4_2ns" => {
                        r.type_ = "tree4".to_string();
                    }
                    _ => {
                        panic!("Invalid cm section");
                    }
                }
                r.tree_pos = p1.section_pos;
                r.dim = p1.dim;
            }
        };

        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: 0,
            dom: "".to_string(),
            tmpexps: &mut HashMap::new(),
            starkinfo: self,
        };
        iterate_code(&mut program.verifier_query_code, fix_ref, &mut ctx_f, pil);

        for i in 0..(self.n_publics) {
            if i < program.publics_code.len() && program.publics_code[i].is_some() {
                self.set_code_dimensions(&mut program.publics_code[i], 1);
            }
        }

        self.set_code_dimensions(&mut program.step2prev, 1);
        self.set_code_dimensions(&mut program.step3prev, 1);
        self.set_code_dimensions(&mut program.step3, 1);
        self.set_code_dimensions(&mut program.step42ns, 1);
        self.set_code_dimensions(&mut program.step52ns, 1);
        self.set_code_dimensions(&mut program.verifier_code, 3);
        self.set_code_dimensions(&mut program.verifier_query_code, 1);

        Ok(())
    }

    fn set_dim(&self, r: &mut Node, dim: usize, tmp_dim: &mut HashMap<usize, usize>) {
        match r.type_.as_str() {
            "tmp" => {
                tmp_dim.insert(r.id, dim);
                r.dim = dim;
            }
            "exp" | "cm" | "q" | "tmpExp" | "f" => {
                r.dim = dim;
            }
            _ => {
                panic!("Invalid reference type set {}", r.type_);
            }
        }
    }

    fn get_dim(&mut self, r: &mut Node, tmp_dim: &HashMap<usize, usize>, dim_x: usize) -> usize {
        #[allow(unused_assignments)]
        let mut d = 0;
        match r.type_.as_str() {
            "tmp" => {
                d = *tmp_dim.get(&r.id).unwrap();
            }
            "tree1" | "tree2" | "tree3" | "tree4" | "tmpExp" => {
                d = r.dim;
            }
            /*
                        "exp" => {
                            d = if self.var_pol_map[self.exps_2ns[r.id]].section.len() > 0 {
                                self.var_pol_map[self.exps_2ns[r.id]].dim
                            } else {
                                self.var_pol_map[self.exps_n[r.id]].dim
                            };
                        }
            */
            "cm" => {
                d = self.var_pol_map[self.cm_2ns[r.id]].dim;
            }
            "q" => {
                d = self.var_pol_map[self.qs[r.id]].dim;
            }
            "const" | "number" | "public" | "Zi" => {
                d = 1;
            }
            "eval" | "challenge" | "Z" => {
                d = 3;
            }
            "xDivXSubXi" | "xDivXSubWXi" | "x" => {
                d = dim_x;
            }
            _ => {
                panic!("Invalid reference type get {}", r.type_);
            }
        }
        if d == 0 {
            panic!("Invalid dim");
        }
        r.dim = d;
        d
    }

    fn _set_code_dimensions(
        &mut self,
        codes: &mut [Section],
        tmp_dim: &mut HashMap<usize, usize>,
        dim_x: usize,
    ) {
        for c in codes.iter_mut() {
            #[allow(unused_assignments)]
            let mut new_dim = 0;
            match c.op.as_str() {
                "add" => {
                    new_dim = std::cmp::max(
                        self.get_dim(&mut c.src[0], tmp_dim, dim_x),
                        self.get_dim(&mut c.src[1], tmp_dim, dim_x),
                    );
                }
                "sub" => {
                    new_dim = std::cmp::max(
                        self.get_dim(&mut c.src[0], tmp_dim, dim_x),
                        self.get_dim(&mut c.src[1], tmp_dim, dim_x),
                    );
                }
                "mul" => {
                    new_dim = std::cmp::max(
                        self.get_dim(&mut c.src[0], tmp_dim, dim_x),
                        self.get_dim(&mut c.src[1], tmp_dim, dim_x),
                    );
                }
                "muladd" => {
                    let tmp = std::cmp::max(
                        self.get_dim(&mut c.src[0], tmp_dim, dim_x),
                        self.get_dim(&mut c.src[1], tmp_dim, dim_x),
                    );
                    new_dim = std::cmp::max(tmp, self.get_dim(&mut c.src[2], tmp_dim, dim_x));
                }
                "copy" => {
                    new_dim = self.get_dim(&mut c.src[0], tmp_dim, dim_x);
                }
                _ => {
                    panic!("Invalid op: {}", c.op);
                }
            };
            self.set_dim(&mut c.dest, new_dim, tmp_dim);
        }
    }

    fn set_code_dimensions(&mut self, segment: &mut Segment, dim_x: usize) {
        let mut tmp_dim: HashMap<usize, usize> = HashMap::new();

        self._set_code_dimensions(&mut segment.first, &mut tmp_dim, dim_x);
        self._set_code_dimensions(&mut segment.i, &mut tmp_dim, dim_x);
        self._set_code_dimensions(&mut segment.last, &mut tmp_dim, dim_x);
    }

    pub fn set_code_dimensions_first(&mut self, segment: &mut Segment) -> Result<()> {
        let mut tmp_dim: HashMap<usize, usize> = HashMap::new();
        let dim_x = 3;
        self._set_code_dimensions(&mut segment.first, &mut tmp_dim, dim_x);
        Ok(())
    }

    fn fix_prover_code(
        &mut self,
        segment: &mut Segment,
        dom: &str,
        pil: &mut PIL,
        tmpexps: &mut HashMap<usize, usize>,
    ) {
        //log::trace!("fix_prover_code: {} {} {:?}", segment, dom, self.tmpexp_n);
        let mut ctx_f = ContextF {
            exp_map: HashMap::new(),
            tmp_used: segment.tmp_used,
            dom: dom.to_string(),
            tmpexps,
            starkinfo: self,
        };

        let fix_ref = |r: &mut Node, ctx: &mut ContextF, pil: &mut PIL| {
            match r.type_.as_str() {
                "cm" => {
                    if ctx.dom.as_str() == "n" {
                        r.p = ctx.starkinfo.cm_n[r.id];
                    } else if ctx.dom.as_str() == "2ns" {
                        r.p = ctx.starkinfo.cm_2ns[r.id];
                    } else {
                        panic!("Invalid domain {}", ctx.dom);
                    }
                }

                "exp" => {
                    let idx = ctx.starkinfo.im_exps_list.iter().position(|&x| x == r.id);
                    if let Some(idx) = idx {
                        r.type_ = "cm".to_string();
                        r.id = ctx.starkinfo.im_exp2cm[&ctx.starkinfo.im_exps_list[idx]];
                    } else if ctx.tmpexps.get(&r.id).is_some() && ctx.dom == "n" {
                        r.type_ = "tmpExp".to_string();
                        r.dim = Self::get_exp_dim(pil, &pil.expressions[r.id]);
                        r.id = ctx.tmpexps[&r.id];
                    } else {
                        let p = if r.prime { 1 } else { 0 };
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            ctx.exp_map.entry((p, r.id))
                        {
                            e.insert(ctx.tmp_used);
                            ctx.tmp_used += 1;
                        }

                        r.type_ = "tmp".to_string();
                        r.exp_id = r.id;
                        r.id = *ctx.exp_map.get(&(p, r.id)).unwrap();
                    }
                }
                "const" | "number" | "challenge" | "public" | "tmp" | "Zi" | "xDivXSubXi"
                | "xDivXSubWXi" | "eval" | "x" | "q" | "f" | "tmpExp" => {}
                _ => {
                    panic!("Invalid reference type {}", r.type_);
                }
            };
        };

        iterate_code(segment, fix_ref, &mut ctx_f, pil);
        segment.tmp_used = ctx_f.tmp_used;
    }

    fn map_section(&mut self) -> Result<()> {
        let names: [&'static str; 11] = [
            "cm1_n", "cm1_2ns", "cm2_n", "cm2_2ns", "cm3_n", "cm3_2ns", "cm4_n", "cm4_2ns",
            "q_2ns", "f_2ns", "tmpexp_n",
        ];

        for s in names.iter() {
            let mut p = 0;
            for e in 1..=3 {
                for pp in self.var_pol_map.iter_mut() {
                    if pp.section.as_str() == *s && pp.dim == e {
                        pp.section_pos = p;
                        p += e;
                    }
                }
                if e == 1 {
                    self.map_sectionsN1.set(s, p);
                }
                if e == 3 {
                    self.map_sectionsN.set(s, p);
                }
            }
            let t = (self.map_sectionsN.get(s) - self.map_sectionsN1.get(s)) / 3;
            //log::trace!("map_sectionN3 set {} = {}", s, t);
            self.map_sectionsN3.set(s, t);
        }
        Ok(())
    }

    pub fn get_exp_dim(pil: &PIL, exp: &Expression) -> usize {
        match exp.op.as_str() {
            "add" | "sub" | "mul" | "muladd" | "addc" | "mulc" | "neg" => {
                let mut md = 1;
                let values = exp.values.as_ref().unwrap();
                for v in values.iter() {
                    let d = Self::get_exp_dim(pil, v);
                    if d > md {
                        md = d
                    }
                }
                md
            }
            "cm" => pil.cm_dims[exp.id.unwrap()],
            "const" => 1,
            "exp" => Self::get_exp_dim(pil, &pil.expressions[exp.id.unwrap()]),
            "q" => Self::get_exp_dim(pil, &pil.expressions[pil.q2exp[exp.id.unwrap()]]),
            "number" | "public" => 1,
            "challenge" | "eval" | "xDivXSubXi" | "xDivXSubWXi" => 3,
            "x" => 1,
            _ => panic!("Exp op not defined: {}", exp.op),
        }
    }
}
