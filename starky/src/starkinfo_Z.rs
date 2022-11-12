use crate::expressionops::ExpressionOps as E;
use crate::helper::get_ks;
use crate::starkinfo::StarkInfo;
use crate::starkinfo::PECTX;
use crate::starkinfo_codegen::{build_code, pil_code_gen, Context};

impl StarkInfo {
    pub fn generate_permutation_LC(&mut self, ctx: &mut Context) {
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

            pil_code_gen(ctx, f_exp_id.clone(), false, &"".to_string());
            pil_code_gen(ctx, t_exp_id.clone(), false, &"".to_string());

            self.pe_ctx.push(PECTX { f_exp_id, t_exp_id });
        }
    }

    pub fn generate_plonk_Z(&mut self, &mut ctx: Context) {
        let pui = ctx.pil.plookupIdentities;
        for (i,pu) in pui.iter().enumerate() {
           self.pu_ctx[i].z_id = ctx.pil.nCommitments;
           ctx.pil.nCommitments += 1;

           let h1 = E::cm(self.pu_ctx[i].h1_id, None);
           let h2 = E::cm(self.pu_ctx[i].h2_id, None);

           let h1p = E::cm(self.pu_ctx[i].h1_id, Some(true));
           let f = E::cm(self.pu_ctx[i].f_exp_id, None);
           let t = E::cm(self.pu_ctx[i].t_exp_id, None);
           let tp = E::cm(self.pu_ctx[i].t_exp_id, Some(true));

           let zp = E::cm(self.pu_ctx[i].z_id, Some(true));

           if ctx.pil.references["Global.L1".to_string()].get().is_none() {
               panic!("Global.L1 must be defined");
           }


        }

    }

    /*
function generatePlookupZ(res, pil, ctx) {
    const E = new ExpressionOps();

    for (let i=0; i<pil.plookupIdentities.length; i++) {
        const puCtx = res.puCtx[i];
        puCtx.zId = pil.nCommitments++;


        const h1 = E.cm(puCtx.h1Id);
        const h2 =  E.cm(puCtx.h2Id);
        const h1p = E.cm(puCtx.h1Id, true);
        const f = E.exp(puCtx.fExpId);
        const t = E.exp(puCtx.tExpId);
        const tp = E.exp(puCtx.tExpId, true);
        const z = E.cm(puCtx.zId);
        const zp = E.cm(puCtx.zId, true);

        if ( typeof pil.references["Global.L1"] === "undefined") throw new Error("Global.L1 must be defined");

        const l1 = E.const(pil.references["Global.L1"].id);

        const c1 = E.mul(l1,  E.sub(z, E.number(1)));
        c1.deg=2;
        puCtx.c1Id = pil.expressions.length;
        pil.expressions.push(c1);
        pil.polIdentities.push({e: puCtx.c1Id});

        const gamma = E.challenge("gamma");
        const beta = E.challenge("beta");

        const numExp = E.mul(
            E.mul(
                E.add(f, gamma),
                E.add(
                    E.add(
                        t,
                        E.mul(
                            tp,
                            beta
                        )
                    ),
                    E.mul(gamma,E.add(E.number(1), beta))
                )
            ),
            E.add(E.number(1), beta)
        );
        numExp.idQ = pil.nQ++;
        puCtx.numId = pil.expressions.length;
        numExp.keep = true;
        pil.expressions.push(numExp);

        const denExp = E.mul(
            E.add(
                E.add(
                    h1,
                    E.mul(
                        h2,
                        beta
                    )
                ),
                E.mul(gamma,E.add(E.number(1), beta))
            ),
            E.add(
                E.add(
                    h2,
                    E.mul(
                        h1p,
                        beta
                    )
                ),
                E.mul(gamma,E.add(E.number(1), beta))
            )
        );
        denExp.idQ = pil.nQ++;
        puCtx.denId = pil.expressions.length;
        denExp.keep = true;
        pil.expressions.push(denExp);

        const num = E.exp(puCtx.numId);
        const den = E.exp(puCtx.denId);

        const c2 = E.sub(  E.mul(zp, den), E.mul(z, num)  );
        c2.deg=2;
        puCtx.c2Id = pil.expressions.length;
        pil.expressions.push(c2);
        pil.polIdentities.push({e: puCtx.c2Id});

        pilCodeGen(ctx, puCtx.numId, false);
        pilCodeGen(ctx, puCtx.denId, false);
    }
}


function generatePermutationZ(res, pil, ctx) {
    const E = new ExpressionOps();

    for (let i=0; i<pil.permutationIdentities.length; i++) {
        peCtx = res.peCtx[i];

        peCtx.zId = pil.nCommitments++;

        const f = E.exp(peCtx.fExpId);
        const t = E.exp(peCtx.tExpId);
        const z = E.cm(peCtx.zId);
        const zp = E.cm(peCtx.zId, true);

        if ( typeof pil.references["Global.L1"] === "undefined") throw new Error("Global.L1 must be defined");

        const l1 = E.const(pil.references["Global.L1"].id);

        const c1 = E.mul(l1,  E.sub(z, E.number(1)));
        c1.deg=2;
        peCtx.c1Id = pil.expressions.length;
        pil.expressions.push(c1);
        pil.polIdentities.push({e: peCtx.c1Id});

        const beta = E.challenge("beta");

        const numExp = E.add( f, beta);
        peCtx.numId = pil.expressions.length;
        numExp.keep = true;
        pil.expressions.push(numExp);

        const denExp = E.add( t, beta);
        peCtx.denId = pil.expressions.length;
        denExp.keep = true;
        pil.expressions.push(denExp);

        const c2 = E.sub(  E.mul(zp,  E.exp( peCtx.denId )), E.mul(z, E.exp( peCtx.numId )));
        c2.deg=2;
        peCtx.c2Id = pil.expressions.length;
        pil.expressions.push(c2);
        pil.polIdentities.push({e: peCtx.c2Id});

        pilCodeGen(ctx, peCtx.numId, false);
        pilCodeGen(ctx, peCtx.denId, false);
    }
}

function generateConnectionsZ(res, pil, ctx) {
    const E = new ExpressionOps();
    const F = new F1Field();

    for (let i=0; i<pil.connectionIdentities.length; i++) {
        const ci = pil.connectionIdentities[i];
        const ciCtx = {};

        ciCtx.zId = pil.nCommitments++;

        const beta = E.challenge("beta");
        const gamma = E.challenge("gamma");

        let numExp = E.add(
            E.add(
                E.exp(ci.pols[0]),
                E.mul(beta, E.x())
            ), gamma);

        let denExp = E.add(
            E.add(
                E.exp(ci.pols[0]),
                E.mul(beta, E.exp(ci.connections[0]))
            ), gamma);

        ciCtx.numId = pil.expressions.length;
        numExp.keep = true;
        pil.expressions.push(numExp);

        ciCtx.denId = pil.expressions.length;
        denExp.keep = true;
        pil.expressions.push(denExp);

        let ks = getKs(F, ci.pols.length-1);
        for (let i=1; i<ci.pols.length; i++) {
            const numExp =
                E.mul(
                    E.exp(ciCtx.numId),
                    E.add(
                        E.add(
                            E.exp(ci.pols[i]),
                            E.mul(E.mul(beta, E.number(ks[i-1])), E.x())
                        ),
                        gamma
                    )
                );
            numExp.idQ = pil.nQ++;

            const denExp =
                E.mul(
                    E.exp(ciCtx.denId),
                    E.add(
                        E.add(
                            E.exp(ci.pols[i]),
                            E.mul(beta, E.exp(ci.connections[i]))
                        ),
                        gamma
                    )
                );
            denExp.idQ = pil.nQ++;

            ciCtx.numId = pil.expressions.length;
            pil.expressions.push(numExp);

            ciCtx.denId = pil.expressions.length;
            pil.expressions.push(denExp);
        }

        const z = E.cm(ciCtx.zId);
        const zp = E.cm(ciCtx.zId, true);

        if ( typeof pil.references["Global.L1"] === "undefined") throw new Error("Global.L1 must be defined");

        const l1 = E.const(pil.references["Global.L1"].id);

        const c1 = E.mul(l1,  E.sub(z, E.number(1)));
        c1.deg=2;
        ciCtx.c1Id = pil.expressions.length;
        pil.expressions.push(c1);
        pil.polIdentities.push({e: ciCtx.c1Id});


        const c2 = E.sub(  E.mul(zp,  E.exp( ciCtx.denId )), E.mul(z, E.exp( ciCtx.numId )));
        c2.deg=2;
        ciCtx.c2Id = pil.expressions.length;
        pil.expressions.push(c2);
        pil.polIdentities.push({e: ciCtx.c2Id});

        pilCodeGen(ctx, ciCtx.numId, false);
        pilCodeGen(ctx, ciCtx.denId, false);

        res.ciCtx.push(ciCtx);
    }
}
*/
}
