// copied and modified from pil-stark
const {pilCodeGen, buildCode, fixCode} = require("./starkinfo_codegen.js");
const ExpressionOps = require("./expressionops.js");
const getKs = require("pilcom").getKs;
const F1Field = require("./f3g.js");

module.exports = function generateStep3(res, pil, ctx) {

    generatePermutationLC(res, pil, ctx);
    generatePlookupZ(res, pil, ctx);
    generatePermutationZ(res, pil, ctx);
    generateConnectionsZ(res, pil, ctx);


    res.step3prev = buildCode(ctx);
}



function generatePermutationLC(res, pil, ctx) {

    const E = new ExpressionOps();

    for (let i=0; i<pil.permutationIdentities.length; i++) {
        const peCtx = {};
        const pi = pil.permutationIdentities[i];

        let tExp = null;
        const u = E.challenge("u");
        const defVal = E.challenge("defVal");
        for (let j=0; j<pi.t.length; j++) {
            const e = E.exp(pi.t[j]);
            if (tExp) {
                tExp = E.add(E.mul(u, tExp), e);
            } else {
                tExp = e;
            }
        }
        if (pi.selT !== null) {
            tExp = E.sub(tExp, defVal);
            tExp = E.mul(tExp, E.exp(pi.selT));
            tExp = E.add(tExp, defVal);

            tExp.idQ = pil.nQ;
            pil.nQ++;
        }

        peCtx.tExpId = pil.expressions.length;
        pil.expressions.push(tExp);


        fExp = null;
        for (let j=0; j<pi.f.length; j++) {
            const e = E.exp(pi.f[j]);
            if (fExp) {
                fExp = E.add(E.mul(fExp, u), e);
            } else {
                fExp = e;
            }
        }
        if (pi.selF !== null) {
            fExp = E.sub(fExp, defVal);
            fExp = E.mul(fExp, E.exp(pi.selF));
            fExp = E.add(fExp, defVal);

            fExp.idQ = pil.nQ;
            pil.nQ++;
        }

        peCtx.fExpId = pil.expressions.length;
        pil.expressions.push(fExp);

        pilCodeGen(ctx, peCtx.fExpId, false);
        pilCodeGen(ctx, peCtx.tExpId, false);

        res.peCtx.push(peCtx);
    }
}


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
