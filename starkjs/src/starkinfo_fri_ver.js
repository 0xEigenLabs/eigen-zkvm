// copied and modified from pil-stark
const {pilCodeGen, buildCode, fixCode, iterateCode} = require("./starkinfo_codegen.js");

module.exports = function generateVerifierQuery(res, pil) {

    const ctxFri = {
        pil: pil,
        calculated: {
            exps: {},
            expsPrime: {}
        },
        tmpUsed: 0,
        code: []
    };

    pilCodeGen(ctxFri, res.friExpId);
    res.verifierQueryCode = buildCode(ctxFri);
    res.nExps = pil.expressions.length;

    const ctxF = {};
    ctxF.expMap = [{}, {}];
    ctxF.code = res.verifierQueryCode;

    iterateCode(res.verifierQueryCode, fixRef2, ctxF);

    function fixRef2(r, ctx) {
        switch (r.type) {
            case "cm":
            case "q":
            case "const":
                break;
            case "exp":
                const p = r.prime ? 1 : 0;
                if (typeof ctx.expMap[p][r.id] === "undefined") {
                    ctx.expMap[p][r.id] = ctx.code.tmpUsed ++;
                }
                delete r.prime;
                r.type= "tmp";
                r.id= ctx.expMap[p][r.id];
                break;
            case "number":
            case "challenge":
            case "public":
            case "tmp":
            case "xDivXSubXi":
            case "xDivXSubWXi":
            case "x":
            case "eval":
            case "tree1":
            case "tree2":
            case "tree3":
            case "tree4":
                break;
            default:
                throw new Error("Invalid reference type: "+r.type);
        }
    }
}

