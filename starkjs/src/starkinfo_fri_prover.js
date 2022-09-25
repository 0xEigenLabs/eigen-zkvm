// copied and modified from pil-stark
const {pilCodeGen, buildCode, fixCode} = require("./starkinfo_codegen.js");
const ExpressionOps = require("./expressionops.js");


module.exports = function generateFRIPolynomial(res, pil, ctx2ns) {
    const E = new ExpressionOps();

    const vf1 = E.challenge("vf1");
    const vf2 = E.challenge("vf2");

    let friExp = null;
    for (let i=0; i<pil.nCommitments; i++) {
        if (friExp) {
            friExp = E.add(E.mul(vf1, friExp), E.cm(i));
        } else {
            friExp = E.cm(i);
        }
    }
    for (let i=0; i<pil.nQ; i++) {
        if (friExp) {
            friExp = E.add(E.mul(vf1, friExp), E.q(i));
        } else {
            friExp = E.q(i);
        }
    }

    let fri1exp = null;
    let fri2exp = null;
    const xi = E.challenge("xi");
    for (let i=0; i<res.evMap.length; i++) {
        const ev = res.evMap[i];
        let friExp = ev.prime ? fri2exp : fri1exp;
        const e = E[ev.type](ev.id);
        if (friExp) {
            friExp = E.add(E.mul(friExp, vf2), E.sub(e,  E.eval(i)));
        } else {
            friExp = E.sub(e,  E.eval(i));
        }
        if (ev.prime) {
            fri2exp = friExp;
        } else {
            fri1exp = friExp;
        }
    }


    fri1exp = E.mul(fri1exp, E.xDivXSubXi() );
    if (friExp) {
        friExp = E.add(E.mul(vf1, friExp),  fri1exp );
    } else {
        friExp = fri1exp;
    }

    fri2exp =  E.mul(fri2exp, E.xDivXSubWXi() );
    if (friExp) {
        friExp = E.add(E.mul(vf1, friExp),  fri2exp );
    } else {
        friExp = fri2exp;
    }

    res.friExpId = pil.expressions.length;
    friExp.keep2ns = true;
    pil.expressions.push(friExp);

    pilCodeGen(ctx2ns, res.friExpId);
    res.step52ns = buildCode(ctx2ns);

}
