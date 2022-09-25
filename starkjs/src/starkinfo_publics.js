// copied and modified from pil-stark
const {pilCodeGen, buildCode, iterateCode} = require("./starkinfo_codegen.js");

module.exports = function generatePublicCalculators(res, pil) {
    res.publicsCode = [];
    for (let i=0; i<pil.publics.length; i++) {
        if (pil.publics[i].polType == "imP") {
            const ctx = {
                pil: pil,
                calculated: {
                    exps: {},
                    expsPrime: {}
                },
                tmpUsed: 0,
                code: []
            };
            pilCodeGen(ctx, pil.publics[i].polId, false);
            res.publicsCode[i] = buildCode(ctx);
            const ctxF = {};
            ctxF.expMap = [{}, {}];
            ctxF.code = res.publicsCode[i];
            iterateCode(res.publicsCode[i], function fixRef(r, ctx) {
                const p = r.prime ? 1 : 0;
                if (r.type === "exp") {
                    if (typeof ctx.expMap[p][r.id] === "undefined") {
                        ctx.expMap[p][r.id] = ctx.code.tmpUsed ++;
                    }
                    delete r.prime;
                    r.type= "tmp";
                    r.id= ctx.expMap[p][r.id];
                }
            }, ctxF);
            ctx.calculated =  { exps: {}, expsPrime: {} }  // Public inputs expressions are caculated at a single point, so they cannot be considered as calculated
        }
    }
}
