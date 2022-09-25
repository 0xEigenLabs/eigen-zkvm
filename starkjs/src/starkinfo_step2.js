// copied and modified from pil-stark
const {pilCodeGen, buildCode, fixCode} = require("./starkinfo_codegen.js");
const ExpressionOps = require("./expressionops.js");


module.exports = function generateStep2(res, pil, ctx) {

    const E = new ExpressionOps();

    for (let i=0; i<pil.plookupIdentities.length; i++) {
        const puCtx = {};
        const pi = pil.plookupIdentities[i];

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

        puCtx.tExpId = pil.expressions.length;
        tExp.keep = true;
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
            fExp = E.sub(fExp, E.exp(puCtx.tExpId));
            fExp = E.mul(fExp, E.exp(pi.selF));
            fExp = E.add(fExp, E.exp(puCtx.tExpId));

            fExp.idQ = pil.nQ;
            pil.nQ++;
        }

        puCtx.fExpId = pil.expressions.length;
        fExp.keep = true;
        pil.expressions.push(fExp);

        pilCodeGen(ctx, puCtx.fExpId, false);
        pilCodeGen(ctx, puCtx.tExpId, false);

        puCtx.h1Id = pil.nCommitments++;
        puCtx.h2Id = pil.nCommitments++;

        res.puCtx.push(puCtx);
    }

    res.step2prev = buildCode(ctx);
}
