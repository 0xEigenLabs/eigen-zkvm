use crate::errors::Result;
use crate::expressionops::ExpressionOps as E;
use crate::f3g::F3G;
use crate::helper::get_ks;
use crate::starkinfo::StarkInfo;
use crate::starkinfo::{CICTX, PECTX};
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context};
use crate::types::PolIdentity;

impl StarkInfo {

    pub fn generate_step3(&mut self, ctx: &mut Context) -> Result<()>{
        self.generate_permutation_LC(ctx)?;
        self.generate_plonk_Z(ctx)?;
        self.generate_permutation_Z(ctx)?;
        self.generate_connections_Z(ctx)?;

        self.step3prev = build_code(ctx);
    }

    pub fn generate_permutation_LC(&mut self, ctx: &mut Context) -> Result<()> {
        let ppi = match &ctx.pil.permutationIdentities {
            Some(x) => x.clone(),
            _ => Vec::new(),
        };
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
                t_exp.idQ = Some(ctx.pil.nQ);
                ctx.pil.nQ += 1;
            }

            let t_exp_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(t_exp);

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
                f_exp.idQ = Some(ctx.pil.nQ);
                ctx.pil.nQ += 1;
            }

            let f_exp_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(f_exp);

            pil_code_gen(ctx, f_exp_id.clone(), false, &"".to_string())?;
            pil_code_gen(ctx, t_exp_id.clone(), false, &"".to_string())?;

            self.pe_ctx.push(PECTX {
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

    pub fn generate_plonk_Z(&mut self, ctx: &mut Context) -> Result<()> {
        let pui = ctx.pil.plookupIdentities.clone();
        for (i, _pu) in pui.iter().enumerate() {
            self.pu_ctx[i].z_id = ctx.pil.nCommitments;
            ctx.pil.nCommitments += 1;

            let h1 = E::cm(self.pu_ctx[i].h1_id, None);
            let h2 = E::cm(self.pu_ctx[i].h2_id, None);

            let h1p = E::cm(self.pu_ctx[i].h1_id, Some(true));
            let f = E::cm(self.pu_ctx[i].f_exp_id, None);
            let t = E::cm(self.pu_ctx[i].t_exp_id, None);
            let tp = E::cm(self.pu_ctx[i].t_exp_id, Some(true));

            let z = E::cm(self.pu_ctx[i].z_id, None);
            let zp = E::cm(self.pu_ctx[i].z_id, Some(true));

            if ctx.pil.references.get(&"Global.L1".to_string()).is_none() {
                panic!("Global.L1 must be defined");
            }

            let l1 = E::const_(ctx.pil.references[&"Global.L1".to_string()].id as i32, None);
            let mut c1 = E::mul(&l1, &E::sub(&z, &E::number("1".to_string())));
            c1.deg = 2;

            self.pu_ctx[i].c1_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(c1);
            ctx.pil.polIdentities.push(PolIdentity {
                e: self.pu_ctx[i].c1_id.clone(),
                line: 0,
                fileName: "".to_string(),
            });

            let gamma = E::challenge("gamma".to_string());
            let beta = E::challenge("beta".to_string());

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

            num_exp.idQ = Some(ctx.pil.nQ);

            ctx.pil.nQ += 1;
            self.pu_ctx[i].num_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(num_exp);

            let mut den_exp = E::mul(
                &E::mul(
                    &E::add(&f, &gamma),
                    &E::add(
                        &E::add(&h1, &E::mul(&h2, &beta)),
                        &E::mul(&gamma, &E::add(&E::number("1".to_string()), &beta)),
                    ),
                ),
                &E::add(
                    &E::add(&h2, &E::mul(&h1p, &beta)),
                    &E::mul(&gamma, &E::add(&E::number("1".to_string()), &beta)),
                ),
            );

            den_exp.idQ = Some(ctx.pil.nQ);
            ctx.pil.nQ += 1;

            self.pu_ctx[i].den_id = ctx.pil.expressions.len() as i32;
            den_exp.keep = Some(true);
            ctx.pil.expressions.push(den_exp);

            let num = E::exp(self.pu_ctx[i].num_id, None);
            let den = E::exp(self.pu_ctx[i].den_id, None);

            let mut c2 = E::sub(&E::mul(&zp, &den), &E::mul(&z, &num));
            c2.deg = 2;
            self.pu_ctx[i].c2_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(c2);

            ctx.pil.polIdentities.push(PolIdentity {
                e: self.pu_ctx[i].c2_id.clone(),
                line: 0,
                fileName: "".to_string(),
            });
            pil_code_gen(ctx, self.pu_ctx[i].num_id.clone(), false, &"".to_string())?;
            pil_code_gen(ctx, self.pu_ctx[i].den_id.clone(), false, &"".to_string())?;
        }
        Ok(())
    }

    pub fn generate_permutation_Z(&mut self, ctx: &mut Context) -> Result<()> {
        let ppi = match &ctx.pil.permutationIdentities {
            Some(x) => x.clone(),
            _ => Vec::new(),
        };

        for (i, pi) in ppi.iter().enumerate() {
            self.pe_ctx[i].z_id = ctx.pil.nCommitments;
            ctx.pil.nCommitments += 1;

            let f = E::exp(self.pe_ctx[i].f_exp_id, None);
            let t = E::exp(self.pe_ctx[i].t_exp_id, None);
            let z = E::cm(self.pe_ctx[i].z_id, None);
            let zp = E::cm(self.pe_ctx[i].z_id, Some(true));

            if ctx.pil.references.get(&"Global.L1".to_string()).is_none() {
                panic!("Global.L1 must be defined");
            }
            let l1 = E::const_(ctx.pil.references[&"Global.L1".to_string()].id as i32, None);
            let mut c1 = E::mul(&l1, &E::sub(&z, &E::number("1".to_string())));
            c1.deg = 2;

            self.pe_ctx[i].c1_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(c1);
            ctx.pil.polIdentities.push(PolIdentity {
                e: self.pe_ctx[i].c1_id.clone(),
                line: 0,
                fileName: "".to_string(),
            });

            let beta = E::challenge("beta".to_string());

            let mut num_exp = E::add(&f, &beta);
            self.pe_ctx[i].num_id = ctx.pil.expressions.len() as i32;
            num_exp.keep = Some(true);
            ctx.pil.expressions.push(num_exp);

            let mut den_exp = E::add(&t, &beta);
            self.pe_ctx[i].den_id = ctx.pil.expressions.len() as i32;
            den_exp.keep = Some(true);
            ctx.pil.expressions.push(den_exp);

            let mut c2 = E::sub(
                &E::mul(&zp, &E::exp(self.pe_ctx[i].den_id.clone(), None)),
                &E::mul(&z, &E::exp(self.pe_ctx[i].num_id.clone(), None)),
            );
            c2.deg = 2;
            self.pe_ctx[i].c2_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(c2);
            ctx.pil.polIdentities.push(PolIdentity {
                e: self.pe_ctx[i].c2_id.clone(),
                line: 0,
                fileName: "".to_string(),
            });

            pil_code_gen(ctx, self.pe_ctx[i].num_id.clone(), false, &"".to_string())?;
            pil_code_gen(ctx, self.pe_ctx[i].den_id.clone(), false, &"".to_string())?;
        }
        Ok(())
    }

    pub fn generate_connections_Z(&mut self, ctx: &mut Context) -> Result<()> {
        let cii = match &ctx.pil.connectionIdentities {
            Some(x) => x.clone(),
            _ => Vec::new(),
        };

        for ci in cii.iter() {
            let ci_pols = match &ci.pols {
                Some(x) => x.clone(),
                _ => panic!("ci.pols is empty"),
            };
            let ci_connections = match &ci.connections {
                Some(x) => x.clone(),
                _ => panic!("ci.connections is empty"),
            };

            let mut ci_ctx = CICTX::default();
            ci_ctx.z_id = ctx.pil.nCommitments as i32;
            ctx.pil.nCommitments += 1;

            let gamma = E::challenge("gamma".to_string());
            let beta = E::challenge("beta".to_string());

            let mut num_exp = E::add(
                &E::add(&E::exp(ci_pols[0], None), &E::mul(&beta, &E::x())),
                &gamma,
            );

            let mut den_exp = E::add(
                &E::add(
                    &E::exp(ci_pols[0], None),
                    &E::mul(&beta, &E::exp(ci_connections[0], None)),
                ),
                &gamma,
            );
            ci_ctx.num_id = ctx.pil.expressions.len() as i32;
            num_exp.keep = Some(true);
            ctx.pil.expressions.push(num_exp);

            ci_ctx.den_id = ctx.pil.expressions.len() as i32;
            den_exp.keep = Some(true);
            ctx.pil.expressions.push(den_exp);

            let ks = get_ks(ci_pols.len() - 1);
            for i in 1..ci_pols.len() {
                let mut num_exp = E::mul(
                    &E::exp(ci_ctx.num_id, None),
                    &E::add(
                        &E::add(
                            &E::exp(ci_pols[i], None),
                            &E::mul(&E::mul(&beta, &E::number(ks[i - 1].to_string())), &E::x()),
                        ),
                        &gamma,
                    ),
                );
                num_exp.idQ = Some(ctx.pil.nQ);
                ctx.pil.nQ += 1;

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
                den_exp.idQ = Some(ctx.pil.nQ);
                ctx.pil.nQ += 1;

                ci_ctx.num_id = ctx.pil.expressions.len() as i32;
                ctx.pil.expressions.push(num_exp);
                ci_ctx.den_id = ctx.pil.expressions.len() as i32;
                ctx.pil.expressions.push(den_exp);
            }

            let z = E::cm(ci_ctx.z_id, None);
            let zp = E::cm(ci_ctx.z_id, Some(true));

            if ctx.pil.references.get(&"Global.L1".to_string()).is_none() {
                panic!("Global.L1 must be defined");
            }
            let l1 = E::const_(ctx.pil.references[&"Global.L1".to_string()].id as i32, None);
            let mut c1 = E::mul(&l1, &E::sub(&z, &E::number("1".to_string())));
            c1.deg = 2;

            ci_ctx.c1_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(c1);

            ctx.pil.polIdentities.push(PolIdentity {
                e: ci_ctx.c1_id.clone(),
                line: 0,
                fileName: "".to_string(),
            });

            let mut c2 = E::sub(
                &E::mul(&zp, &E::exp(ci_ctx.den_id.clone(), None)),
                &E::mul(&z, &E::exp(ci_ctx.num_id.clone(), None)),
            );
            c2.deg = 2;
            ci_ctx.c2_id = ctx.pil.expressions.len() as i32;
            ctx.pil.expressions.push(c2);
            ctx.pil.polIdentities.push(PolIdentity {
                e: ci_ctx.c2_id.clone(),
                line: 0,
                fileName: "".to_string(),
            });

            pil_code_gen(ctx, ci_ctx.num_id.clone(), false, &"".to_string())?;
            pil_code_gen(ctx, ci_ctx.den_id.clone(), false, &"".to_string())?;
            self.ci_ctx.push(ci_ctx);
        }
        Ok(())
    }
}
