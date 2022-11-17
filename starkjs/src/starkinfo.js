// copied and modified from pil-stark
const F1Field = require("./f3g.js");
const getKs = require("pilcom").getKs;

const generatePublicCalculators = require("./starkinfo_publics");
const generateStep2 = require("./starkinfo_step2");
const generateStep3 = require("./starkinfo_step3");
const generateConstraintPolynomial = require("./starkinfo_cp_prover");
const generateFRIPolynomial = require("./starkinfo_fri_prover");

const generateConstraintPolynomialVerifier = require("./starkinfo_cp_ver");
const generateVerifierQuery = require("./starkinfo_fri_ver");
const map = require("./starkinfo_map");
const { elapse } = require("./utils");

module.exports = function starkInfoGen(_pil, starkStruct) {
    const pil = JSON.parse(JSON.stringify(_pil));    // Make a copy as we are going to destroy pil
    const F = new F1Field();
    const pilDeg = Object.values(pil.references)[0].polDeg;
    const starkDeg = 2 ** starkStruct.nBits;

    if ( starkDeg != pilDeg) {
        throw new Error(`Starkpil and pil have degree mismatch (starkpil:${starkDeg} pil:${pilDeg})`);
    }

    if ( starkStruct.nBitsExt != starkStruct.steps[0].nBits) {
        throw new Error(`Starkpil.nBitsExt and first step of Starkpil have a mismatch (nBitsExt:${starkStruct.nBitsExt} pil:${starkStruct.steps[0].nBits})`);
    }

    const res = {
        varPolMap: [],
        puCtx: [],
        peCtx: [],
        ciCtx: []
    };

    res.starkStruct = starkStruct;
    res.nConstants = pil.nConstants;
    res.nPublics = pil.publics.length;

    generatePublicCalculators(res, pil);
    res.nCm1 = pil.nCommitments;

    const ctx = {
        pil: pil,
        calculated: {
            exps: {},
            expsPrime: {}
        },
        tmpUsed: 0,
        code: []
    };

    const ctx2ns = {
        pil: pil,
        calculated: {
            exps: {},
            expsPrime: {}
        },
        tmpUsed: 0,
        code: []
    };

    let timer = [];
    elapse("start", timer);
    generateStep2(res, pil, ctx);                        // H1, H2
    res.nCm2 = pil.nCommitments - res.nCm1;

    elapse("starkInfoGen/generateStep2", timer);

    generateStep3(res, pil, ctx);                        // Z Polynomials and LC of permutation chcks.
    res.nCm3 = pil.nCommitments - res.nCm1 - res.nCm2;
    elapse("starkInfoGen/generateStep3", timer);

    generateConstraintPolynomial(res, pil, ctx, ctx2ns);            // Step4
    res.nCm4 = pil.nCommitments - res.nCm3 -res.nCm2-res.nCm1;
    res.nQ = pil.nQ;
    elapse("starkInfoGen/generateConstraintPolynomial", timer);

    generateConstraintPolynomialVerifier(res, pil);
    elapse("starkInfoGen/generateConstraintPolynomialVerifier", timer);

    generateFRIPolynomial(res, pil, ctx2ns);
    elapse("starkInfoGen/generateFRIPolynomial", timer);

    generateVerifierQuery(res, pil);
    elapse("starkInfoGen/generateVerifierQuery", timer);

    map(res, pil);
    elapse("starkInfoGen/map", timer);

//    cPolBuilder(pil, res.cExp);

    res.publics = pil.publics;
    return res;

}




