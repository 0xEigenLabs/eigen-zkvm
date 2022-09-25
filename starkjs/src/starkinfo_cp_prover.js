// copied and modified from pil-stark
const {pilCodeGen, buildCode, fixCode} = require("./starkinfo_codegen.js");
const ExpressionOps = require("./expressionops.js");

module.exports = function generateConstraintPolynomial(res, pil, ctx, ctx2ns) {

    const E = new ExpressionOps();

    const vc = E.challenge("vc");
    let cExp = null;
    for (let i=0; i<pil.polIdentities.length; i++) {
        const e = E.exp(pil.polIdentities[i].e);
        if (cExp) {
            cExp = E.add(E.mul(vc, cExp), e);
        } else {
            cExp = e;
        }
    }
    cExp.idQ = pil.nQ++;
    res.cExp = pil.expressions.length;
    pil.expressions.push(cExp);

    for (let i=0; i<pil.expressions.length; i++) {
        if (typeof pil.expressions[i].idQ != "undefined") {
            pilCodeGen(ctx, i);
            pilCodeGen(ctx2ns, i, false, "evalQ");
        }
    }

    res.step4 = buildCode(ctx);
    res.step42ns = buildCode(ctx2ns);
}