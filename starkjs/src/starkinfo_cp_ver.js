// copied and modified from pil-stark
const {pilCodeGen, buildCode, iterateCode} = require("./starkinfo_codegen.js");

module.exports  = function generateConstraintPolynomialVerifier(res, pil) {
    const ctxC = {
        pil: pil,
        calculated: {
            exps: {},
            expsPrime: {}
        },
        tmpUsed: 0,
        code: []
    };

    pilCodeGen(ctxC, res.cExp, false, "correctQ");

    res.verifierCode = buildCode(ctxC);


    res.evIdx = {
        cm: [{}, {}],
        q: [{}, {}],
        const: [{}, {}]
    }

    res.evMap = [];

    const ctxF = {};
    ctxF.expMap = [{}, {}];
    ctxF.code = res.verifierCode;

    iterateCode(res.verifierCode, fixRef, ctxF);

    function fixRef(r, ctx) {
        const p = r.prime ? 1 : 0;
        switch (r.type) {
            case "cm":
            case "q":
            case "const":
                if (typeof res.evIdx[r.type][p][r.id] === "undefined") {
                    res.evIdx[r.type][p][r.id] = res.evMap.length;
                    const rf = {
                        type: r.type,
                        id: r.id,
                        prime: r.prime ? true : false,
                    };
                    res.evMap.push(rf);
                }
                delete r.prime;
                r.id= res.evIdx[r.type][p][r.id];
                r.type= "eval";
                break;
            case "exp":
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
            case "Z":
            case "x":
            case "eval":
                    break;
            default:
                throw new Error("Invalid reference type: "+r.type);
        }
    }
}
