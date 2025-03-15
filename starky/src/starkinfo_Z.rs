#![allow(non_snake_case, dead_code)]
use crate::constant::GLOBAL_L1;
use crate::expressionops::ExpressionOps as E;
use crate::helper::get_ks;
use crate::starkinfo::PCCTX;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context};
use crate::types::{PolIdentity, PIL};
use anyhow::Result;

impl StarkInfo {
    pub fn generate_step3(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        program: &mut Program,
        global_l1: Option<String>,
    ) -> Result<()> {
        let global_l1_value = global_l1.unwrap_or((&GLOBAL_L1).to_string());

        self.generate_permutation_LC(ctx, pil)?;
        self.generate_plookup_Z(ctx, pil, &global_l1_value)?;
        self.generate_permutation_Z(ctx, pil, &global_l1_value)?;
        self.generate_connections_Z(ctx, pil, &global_l1_value)?;

        program.step3prev = build_code(ctx, pil);
        //log::trace!("step3prev {}", program.step3prev);
        ctx.calculated.clear();
        Ok(())
    }

    pub fn generate_permutation_LC(&mut self, _ctx: &mut Context, pil: &mut PIL) -> Result<()> {
        let ppi = match &pil.permutationIdentities {
            Some(x) => x.clone(),
            _ => Vec::new(),
        };
        log::trace!("generate_permutation_LC size: {}", ppi.len());
        for pi in ppi.iter() {
            let mut t_exp = E::nop();
            let u = E::challenge("u".to_string());
            let def_val = E::challenge("defVal".to_string());
            for j in pi.t.as_ref().unwrap().iter() {
                let e = E::exp(*j, None);
                if E::is_nop(&t_exp) {
                    t_exp = e;
                } else {
                    t_exp = E::add(&E::mul(&u, &t_exp), &e)
                }
            }

            if pi.selT.is_some() {
                t_exp = E::sub(&t_exp, &def_val);
                t_exp = E::mul(&t_exp, &E::exp(pi.selT.unwrap(), None));
                t_exp = E::add(&t_exp, &def_val);
                t_exp.idQ = Some(pil.nQ);
                pil.nQ += 1;
            }

            if E::is_nop(&t_exp) {
                panic!("nop {:?}", t_exp);
            }

            let t_exp_id = pil.expressions.len();
            pil.expressions.push(t_exp);

            let mut f_exp = E::nop();
            for j in pi.f.as_ref().unwrap().iter() {
                let e = E::exp(*j, None);
                if E::is_nop(&f_exp) {
                    f_exp = e;
                } else {
                    f_exp = E::add(&E::mul(&f_exp, &u), &e);
                }
            }

            if pi.selF.is_some() {
                f_exp = E::sub(&f_exp, &def_val);
                f_exp = E::mul(&f_exp, &E::exp(pi.selF.unwrap(), None));
                f_exp = E::add(&f_exp, &def_val);
                f_exp.idQ = Some(pil.nQ);
                pil.nQ += 1;
            }

            let f_exp_id = pil.expressions.len();
            if E::is_nop(&f_exp) {
                panic!("nop {:?}", f_exp);
            }

            pil.expressions.push(f_exp);

            self.pe_ctx.push(PCCTX {
                h1_id: 0,
                h2_id: 0,
                f_exp_id,
                t_exp_id,
                c1_id: 0,
                c2_id: 0,
                den_id: 0,
                num_id: 0,
                z_id: 0,
            });
        }
        Ok(())
    }

    // paper: https://eprint.iacr.org/2020/315.pdf
    pub fn generate_plookup_Z(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        global_l1: &str,
    ) -> Result<()> {
        for i in 0..pil.plookupIdentities.len() {
            let pu_ctx = &mut self.pu_ctx[i];
            pu_ctx.z_id = pil.nCommitments;
            pil.nCommitments += 1;

            let h1 = E::cm(pu_ctx.h1_id, None);
            let h2 = E::cm(pu_ctx.h2_id, None);
            let h1p = E::cm(pu_ctx.h1_id, Some(true));
            let f = E::exp(pu_ctx.f_exp_id, None);
            let t = E::exp(pu_ctx.t_exp_id, None);
            let tp = E::exp(pu_ctx.t_exp_id, Some(true));
            let z = E::cm(pu_ctx.z_id, None);
            let zp = E::cm(pu_ctx.z_id, Some(true));

            if !pil.references.contains_key(global_l1) {
                panic!("{} must be defined: {:?}", global_l1, pil.references);
            }

            let l1 = E::const_(pil.references[global_l1].id, None);
            let mut c1 = E::mul(&l1, &E::sub(&z, &E::number("1".to_string())));
            c1.deg = 2;

            pu_ctx.c1_id = pil.expressions.len();
            pil.expressions.push(c1);
            pil.polIdentities.push(PolIdentity {
                e: pu_ctx.c1_id,
                line: 0,
                fileName: "".to_string(),
            });

            let gamma = E::challenge("gamma".to_string());
            let beta = E::challenge("beta".to_string());

            // F(\beta, \gamma)
            let mut num_exp = E::mul(
                &E::mul(
                    &E::add(&f, &gamma),
                    &E::add(
                        &E::add(&t, &E::mul(&tp, &beta)),
                        &E::mul(&gamma, &E::add(&E::number("1".to_string()), &beta)),
                    ),
                ),
                &E::add(&E::number("1".to_string()), &beta),
            );

            num_exp.idQ = Some(pil.nQ);
            pil.nQ += 1;
            num_exp.keep = Some(true);
            pu_ctx.num_id = pil.expressions.len();
            pil.expressions.push(num_exp);

            // G(\beta, \gamma)
            let mut den_exp = E::mul(
                &E::add(
                    &E::add(&h1, &E::mul(&h2, &beta)),
                    &E::mul(&gamma, &E::add(&E::number("1".to_string()), &beta)),
                ),
                &E::add(
                    &E::add(&h2, &E::mul(&h1p, &beta)),
                    &E::mul(&gamma, &E::add(&E::number("1".to_string()), &beta)),
                ),
            );

            den_exp.idQ = Some(pil.nQ);
            pil.nQ += 1;
            pu_ctx.den_id = pil.expressions.len();
            den_exp.keep = Some(true);
            pil.expressions.push(den_exp);

            let num = E::exp(pu_ctx.num_id, None);
            let den = E::exp(pu_ctx.den_id, None);

            let mut c2 = E::sub(&E::mul(&zp, &den), &E::mul(&z, &num));
            c2.deg = 2;
            pu_ctx.c2_id = pil.expressions.len();
            pil.expressions.push(c2);

            pil.polIdentities.push(PolIdentity {
                e: pu_ctx.c2_id,
                line: 0,
                fileName: "".to_string(),
            });
            pil_code_gen(ctx, pil, pu_ctx.num_id, false, "", 0, false)?;
            pil_code_gen(ctx, pil, pu_ctx.den_id, false, "", 0, false)?;
        }
        Ok(())
    }

    pub fn generate_permutation_Z(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        global_l1: &str,
    ) -> Result<()> {
        let ppi = match &pil.permutationIdentities {
            Some(x) => x.clone(),
            _ => Vec::new(),
        };
        log::trace!("generate_permutation_Z size: {}", ppi.len());

        for (i, _pi) in ppi.iter().enumerate() {
            self.pe_ctx[i].z_id = pil.nCommitments;
            pil.nCommitments += 1;

            let f = E::exp(self.pe_ctx[i].f_exp_id, None);
            let t = E::exp(self.pe_ctx[i].t_exp_id, None);
            let z = E::cm(self.pe_ctx[i].z_id, None);
            let zp = E::cm(self.pe_ctx[i].z_id, Some(true));

            if !pil.references.contains_key(global_l1) {
                panic!("{} must be defined", global_l1);
            }
            let l1 = E::const_(pil.references[global_l1].id, None);
            let mut c1 = E::mul(&l1, &E::sub(&z, &E::number("1".to_string())));
            c1.deg = 2;

            self.pe_ctx[i].c1_id = pil.expressions.len();
            if E::is_nop(&c1) {
                panic!("nop {:?}", format!("{:?}", c1));
            }

            pil.expressions.push(c1);
            pil.polIdentities.push(PolIdentity {
                e: self.pe_ctx[i].c1_id,
                line: 0,
                fileName: "".to_string(),
            });

            let beta = E::challenge("beta".to_string());

            let mut num_exp = E::add(&f, &beta);
            self.pe_ctx[i].num_id = pil.expressions.len();
            num_exp.keep = Some(true);
            if E::is_nop(&num_exp) {
                panic!("nop {:?}", format!("{:?}", num_exp));
            }

            pil.expressions.push(num_exp);

            let mut den_exp = E::add(&t, &beta);
            self.pe_ctx[i].den_id = pil.expressions.len();
            den_exp.keep = Some(true);
            if E::is_nop(&den_exp) {
                panic!("nop {:?}", format!("{:?}", den_exp));
            }

            pil.expressions.push(den_exp);

            let mut c2 = E::sub(
                &E::mul(&zp, &E::exp(self.pe_ctx[i].den_id, None)),
                &E::mul(&z, &E::exp(self.pe_ctx[i].num_id, None)),
            );
            c2.deg = 2;
            self.pe_ctx[i].c2_id = pil.expressions.len();
            if E::is_nop(&c2) {
                panic!("nop {:?}", format!("{:?}", c2));
            }
            pil.expressions.push(c2);
            pil.polIdentities.push(PolIdentity {
                e: self.pe_ctx[i].c2_id,
                line: 0,
                fileName: "".to_string(),
            });

            pil_code_gen(ctx, pil, self.pe_ctx[i].num_id, false, "", 0, false)?;
            pil_code_gen(ctx, pil, self.pe_ctx[i].den_id, false, "", 0, false)?;
        }
        Ok(())
    }

    pub fn generate_connections_Z(
        &mut self,
        ctx: &mut Context,
        pil: &mut PIL,
        global_l1: &str,
    ) -> Result<()> {
        let cii = match &pil.connectionIdentities {
            Some(x) => x.clone(),
            _ => Vec::new(),
        };
        log::trace!("generate_connections_Z size: {}", cii.len());

        for ci in cii.iter() {
            let ci_pols = match &ci.pols {
                Some(x) => x.clone(),
                _ => panic!("ci.pols is empty"),
            };
            let ci_connections = match &ci.connections {
                Some(x) => x.clone(),
                _ => panic!("ci.connections is empty"),
            };

            let mut ci_ctx = PCCTX { z_id: pil.nCommitments, ..Default::default() };
            pil.nCommitments += 1;

            let gamma = E::challenge("gamma".to_string());
            let beta = E::challenge("beta".to_string());

            let mut num_exp =
                E::add(&E::add(&E::exp(ci_pols[0], None), &E::mul(&beta, &E::x())), &gamma);

            let mut den_exp = E::add(
                &E::add(
                    &E::exp(ci_pols[0], None),
                    &E::mul(&beta, &E::exp(ci_connections[0], None)),
                ),
                &gamma,
            );
            ci_ctx.num_id = pil.expressions.len();
            num_exp.keep = Some(true);
            if E::is_nop(&num_exp) {
                panic!("nop {:?}", format!("{:?}", num_exp));
            }
            pil.expressions.push(num_exp);

            ci_ctx.den_id = pil.expressions.len();
            den_exp.keep = Some(true);
            if E::is_nop(&den_exp) {
                panic!("nop {:?}", format!("{:?}", den_exp));
            }
            pil.expressions.push(den_exp);

            let ks = get_ks(ci_pols.len() - 1);
            for i in 1..ci_pols.len() {
                let mut num_exp = E::mul(
                    &E::exp(ci_ctx.num_id, None),
                    &E::add(
                        &E::add(
                            &E::exp(ci_pols[i], None),
                            &E::mul(
                                &E::mul(&beta, &E::number(ks[i - 1].as_int().to_string())),
                                &E::x(),
                            ),
                        ),
                        &gamma,
                    ),
                );
                num_exp.idQ = Some(pil.nQ);
                pil.nQ += 1;

                let mut den_exp = E::mul(
                    &E::exp(ci_ctx.den_id, None),
                    &E::add(
                        &E::add(
                            &E::exp(ci_pols[i], None),
                            &E::mul(&beta, &E::exp(ci_connections[i], None)),
                        ),
                        &gamma,
                    ),
                );
                den_exp.idQ = Some(pil.nQ);
                pil.nQ += 1;

                if E::is_nop(&num_exp) {
                    panic!("nop {:?}", format!("{:?}", num_exp));
                }
                ci_ctx.num_id = pil.expressions.len();
                pil.expressions.push(num_exp);
                ci_ctx.den_id = pil.expressions.len();
                if E::is_nop(&den_exp) {
                    panic!("nop {:?}", format!("{:?}", den_exp));
                }
                pil.expressions.push(den_exp);
            }

            let z = E::cm(ci_ctx.z_id, None);
            let zp = E::cm(ci_ctx.z_id, Some(true));

            if !pil.references.contains_key(global_l1) {
                panic!("{} must be defined", global_l1);
            }
            let l1 = E::const_(pil.references[global_l1].id, None);
            let mut c1 = E::mul(&l1, &E::sub(&z, &E::number("1".to_string())));
            c1.deg = 2;

            ci_ctx.c1_id = pil.expressions.len();
            if E::is_nop(&c1) {
                panic!("nop {:?}", format!("{:?}", c1));
            }
            pil.expressions.push(c1);

            pil.polIdentities.push(PolIdentity {
                e: ci_ctx.c1_id,
                line: 0,
                fileName: "".to_string(),
            });

            let mut c2 = E::sub(
                &E::mul(&zp, &E::exp(ci_ctx.den_id, None)),
                &E::mul(&z, &E::exp(ci_ctx.num_id, None)),
            );
            c2.deg = 2;
            ci_ctx.c2_id = pil.expressions.len();

            if E::is_nop(&c2) {
                panic!("nop {:?}", format!("{:?}", c2));
            }

            pil.expressions.push(c2);
            pil.polIdentities.push(PolIdentity {
                e: ci_ctx.c2_id,
                line: 0,
                fileName: "".to_string(),
            });

            pil_code_gen(ctx, pil, ci_ctx.num_id, false, "", 0, false)?;
            pil_code_gen(ctx, pil, ci_ctx.den_id, false, "", 0, false)?;
            self.ci_ctx.push(ci_ctx);
        }
        Ok(())
    }
}
