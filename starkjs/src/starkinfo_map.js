// copied and modified from pil-stark
const {iterateCode, expressionWarning} = require("./starkinfo_codegen.js");

module.exports = function map(res, pil) {
    res.varPolMap = [];
    function addPol(polType) {
        res.varPolMap.push(polType);
        return res.varPolMap.length-1;
    }

    res.cm_n  = [];
    res.cm_2ns  = [];
    res.exps_n = [];
    res.exps_2ns = [];
    res.qs = [];
    res.mapSections = {
        cm1_n: [],
        cm1_2ns: [],
        cm2_n:[],
        cm2_2ns:[],
        cm3_n:[],
        cm3_2ns:[],
        q_2ns:[],
        exps_withq_n:[],
        exps_withq_2ns:[],
        exps_withoutq_n:[],
        exps_withoutq_2ns:[]
    }
    res.mapSectionsN1 = {}    // Number of pols of base field i section
    res.mapSectionsN3 = {}    // Number of pols of base field i section
    res.mapSectionsN = {}    // Number of pols of base field i section


    pil.cmDims = [];
    for (let i=0; i<res.nCm1; i++) {
        const pp_n = addPol({
            section: "cm1_n",
            dim:1
        });
        const pp_2ns = addPol({
            section: "cm1_2ns",
            dim:1
        });
        res.cm_n.push(pp_n);
        res.cm_2ns.push(pp_2ns);
        res.mapSections.cm1_n.push(pp_n);
        res.mapSections.cm1_2ns.push(pp_2ns);
        pil.cmDims[i] = 1;
    }

    for (let i=0; i<res.puCtx.length; i++) {
        const dim = Math.max(getExpDim(pil, res.puCtx[i].fExpId), getExpDim(pil, res.puCtx[i].tExpId));
        const pph1_n = addPol({
            section: "cm2_n",
            dim:dim
        });
        const pph1_2ns = addPol({
            section: "cm2_2ns",
            dim:dim
        });
        res.cm_n.push(pph1_n);
        res.cm_2ns.push(pph1_2ns);
        res.mapSections.cm2_n.push(pph1_n);
        res.mapSections.cm2_2ns.push(pph1_2ns);
        pil.cmDims[res.nCm1 + i*2] = dim;
        const pph2_n = addPol({
            section: "cm2_n",
            dim:dim
        });
        const pph2_2ns = addPol({
            section: "cm2_2ns",
            dim:dim
        });
        res.cm_n.push(pph2_n);
        res.cm_2ns.push(pph2_2ns);
        res.mapSections.cm2_n.push(pph2_n);
        res.mapSections.cm2_2ns.push(pph2_2ns);
        pil.cmDims[res.nCm1 + i*2+1] = dim;
    }

    for (let i=0; i<res.nCm3; i++) {
        const ppz_n = addPol({
            section: "cm3_n",
            dim:3
        });
        const ppz_2ns = addPol({
            section: "cm3_2ns",
            dim:3
        });
        res.cm_n.push(ppz_n);
        res.cm_2ns.push(ppz_2ns);
        res.mapSections.cm3_n.push(ppz_n);
        res.mapSections.cm3_2ns.push(ppz_2ns);
        pil.cmDims[res.nCm1 + res.nCm2 + i] = 3;
    }

    const qDims = [];
    pil.q2exp = [];
    for (let i=0; i<pil.expressions.length; i++) {
        const e = pil.expressions[i];
        if (typeof e.idQ !== "undefined") {
            qDims[e.idQ] = getExpDim(pil, i);
            pil.q2exp[e.idQ] = i;
        }
    }

    const usedQs = {};
    for (let i=0; i<res.evMap.length; i++) {
        const ev = res.evMap[i];
        if (ev.type === "q") {
            usedQs[ev.id] = true;
        }
    }

    for (let i=0; i<pil.nQ; i++) {
        let dim;
        if (usedQs[i]) {
            dim = getExpDim(pil, pil.q2exp[i]);
        } else {
            dim = 0;
            expressionWarning(pil, "Expression with Q not used", pil.q2exp[i]);
        }
        const ppq = addPol({
            section: "q_2ns",
            dim:dim,
            expId: pil.q2exp[i]
        });
        res.qs.push(ppq);
        if (dim>0) {
            res.mapSections.q_2ns.push(ppq);
        }
    }

    for (let i=0; i<pil.expressions.length; i++) {
        const e = pil.expressions[i];
        if (typeof e.idQ !== "undefined") {
            const dim = getExpDim(pil, i);
            const pp_n = addPol({
                section: "exps_withq_n",
                dim:dim,
                expId: i
            });
            const pp_2ns = addPol({
                section: "exps_withq_2ns",
                dim:dim,
                expId: i
            });
            res.mapSections.exps_withq_n.push(pp_n);
            res.mapSections.exps_withq_2ns.push(pp_2ns);
            res.exps_n.push(pp_n);
            res.exps_2ns.push(pp_2ns);
        } else if (e.keep) {
            const dim = getExpDim(pil, i);
            const pp_n = addPol({
                section: "exps_withoutq_n",
                dim:dim,
                expId: i
            });
            res.mapSections.exps_withoutq_n.push(pp_n);
            res.exps_n.push(pp_n);
            res.exps_2ns.push(null);
        } else if (e.keep2ns) {
            const dim = getExpDim(pil, i);
            const pp_2ns = addPol({
                section: "exps_withoutq_2ns",
                dim:dim,
                expId: i
            });
            res.mapSections.exps_withoutq_2ns.push(pp_2ns);
            res.exps_n.push(null);
            res.exps_2ns.push(pp_2ns);
        } else {
            res.exps_n[i] = null;
            res.exps_2ns[i] = null;
        }
    }

    mapSections(res);
    let N = 1 << res.starkStruct.nBits;
    let Next = 1 << res.starkStruct.nBitsExt;
    res.mapOffsets = {};
    res.mapOffsets.cm1_n = 0;
    res.mapOffsets.cm2_n = res.mapOffsets.cm1_n +  N * res.mapSectionsN.cm1_n;
    res.mapOffsets.cm3_n = res.mapOffsets.cm2_n +  N * res.mapSectionsN.cm2_n;
    res.mapOffsets.exps_withq_n = res.mapOffsets.cm3_n +  N * res.mapSectionsN.cm3_n;
    res.mapOffsets.exps_withoutq_n = res.mapOffsets.exps_withq_n +  N * res.mapSectionsN.exps_withq_n;
    res.mapOffsets.cm1_2ns = res.mapOffsets.exps_withoutq_n +  N * res.mapSectionsN.exps_withoutq_n;
    res.mapOffsets.cm2_2ns = res.mapOffsets.cm1_2ns +  Next * res.mapSectionsN.cm1_2ns;
    res.mapOffsets.cm3_2ns = res.mapOffsets.cm2_2ns +  Next * res.mapSectionsN.cm2_2ns;
    res.mapOffsets.q_2ns = res.mapOffsets.cm3_2ns +  Next * res.mapSectionsN.cm3_2ns;
    res.mapOffsets.exps_withq_2ns = res.mapOffsets.q_2ns +  Next * res.mapSectionsN.q_2ns;
    res.mapOffsets.exps_withoutq_2ns = res.mapOffsets.exps_withq_2ns +  Next * res.mapSectionsN.exps_withq_2ns;
    res.mapTotalN = res.mapOffsets.exps_withoutq_2ns +  Next * res.mapSectionsN.exps_withoutq_2ns;

    res.mapDeg = {};
    res.mapDeg.cm1_n = N;
    res.mapDeg.cm2_n = N;
    res.mapDeg.cm3_n = N;
    res.mapDeg.exps_withq_n = N;
    res.mapDeg.exps_withoutq_n = N;
    res.mapDeg.cm1_2ns = Next;
    res.mapDeg.cm2_2ns = Next;
    res.mapDeg.cm3_2ns = Next;
    res.mapDeg.q_2ns = Next;
    res.mapDeg.exps_withq_2ns = Next;
    res.mapDeg.exps_withoutq_2ns = Next;



    for (let i=0; i< res.publicsCode.length; i++) {
        fixProverCode(res.publicsCode[i], "n");
    }
    fixProverCode(res.step2prev, "n");
    fixProverCode(res.step3prev, "n");
    fixProverCode(res.step4, "n");
    fixProverCode(res.step42ns, "2ns");
    fixProverCode(res.step52ns, "2ns");

    iterateCode(res.verifierQueryCode, function fixRef(r, ctx) {
        if (r.type == "cm") {
            const p1 = res.varPolMap[res.cm_2ns[r.id]];
            switch(p1.section) {
                case "cm1_2ns": r.type = "tree1"; break;
                case "cm2_2ns": r.type = "tree2"; break;
                case "cm3_2ns": r.type = "tree3"; break;
                default: throw new Error("Invalid cm section");
            }
            r.treePos = p1.sectionPos;
            r.dim = p1.dim;
        } else if (r.type == "q") {
            const p2 = res.varPolMap[res.qs[r.id]];
            r.type = "tree4";
            r.treePos = p2.sectionPos;
            r.dim = p2.dim;
        }
    });

    for (let i=0; i<res.nPublics; i++) {
        if (res.publicsCode[i]) {
            setCodeDimensions(res.publicsCode[i], res, 1);
        }
    }

    setCodeDimensions(res.step2prev, res, 1);
    setCodeDimensions(res.step3prev,res, 1);
    setCodeDimensions(res.step4, res, 1);
    setCodeDimensions(res.step42ns, res, 1);
    setCodeDimensions(res.step52ns, res, 1);
    setCodeDimensions(res.verifierCode, res, 3);
    setCodeDimensions(res.verifierQueryCode, res, 1);

    function fixProverCode(code, dom) {
        const ctx = {};
        ctx.expMap = [{}, {}];
        ctx.code = code;
        ctx.dom = dom;

        iterateCode(code, fixRef, ctx)

        function fixRef(r, ctx) {
            switch (r.type) {
                case "cm":
                    if (ctx.dom == "n") {
                        r.p = res.cm_n[r.id];
                    } else if (ctx.dom == "2ns") {
                        r.p = res.cm_2ns[r.id];
                    } else {
                        throw ("Invalid domain", ctx.dom);
                    }
                    break;
                case "q":
                    if (ctx.dom == "n") {
                        throw new Error("Accession q in domain n");
                    } else if (ctx.dom == "2ns") {
                        r.p = res.qs[r.id];
                    } else {
                        throw ("Invalid domain", ctx.dom);
                    }
                    break;
                case "exp":
                    if (typeof pil.expressions[r.id].idQ !== "undefined") {
                        if (ctx.dom == "n") {
                            r.p = res.exps_n[r.id];
                        } else if (ctx.dom == "2ns") {
                            r.p = res.exps_2ns[r.id];
                        } else {
                            throw ("Invalid domain", ctx.dom);
                        }
                    } else if ((pil.expressions[r.id].keep)&&(ctx.dom=="n")) {
                        r.p = res.exps_n[r.id];
                    } else if (pil.expressions[r.id].keep2ns) {
                        if (ctx.dom == "n") {
                            throw new Error("Accession keep2ns expresion in n domain");
                        } else if (ctx.dom == "2ns") {
                            r.p = res.exps_2ns[r.id];
                        } else {
                            throw ("Invalid domain", ctx.dom);
                        }
                    } else {
                        const p = r.prime ? 1 : 0;
                        if (typeof ctx.expMap[p][r.id] === "undefined") {
                            ctx.expMap[p][r.id] = ctx.code.tmpUsed ++;
                        }
                        r.type= "tmp";
                        r.expId = r.id;
                        r.id= ctx.expMap[p][r.id];
                    }
                    break;
                case "const":
                case "number":
                case "challenge":
                case "public":
                case "tmp":
                case "Zi":
                case "xDivXSubXi":
                case "xDivXSubWXi":
                case "eval":
                case "x":
                    break;
                default:
                    throw new Error("Invalid reference type " + r.type);
            }
        }
    }

}

/*
    Set the positions of all the secitions puting
*/
function mapSections(res) {
    Object.keys(res.mapSections).forEach((s) => {
        let p = 0;
        for (let e of [1,3]) {
            for (let i=0; i<res.varPolMap.length; i++) {
                const pp = res.varPolMap[i];
                if ((pp.section == s) && (pp.dim==e)) {
                    pp.sectionPos = p;
                    p += e;
                }
            }
            if (e==1) res.mapSectionsN1[s] = p;
            if (e==3) res.mapSectionsN[s] = p;
        }
        res.mapSectionsN3[s] = (res.mapSectionsN[s] - res.mapSectionsN1[s] ) / 3;
    });
}

function getExpDim(pil, expId) {

    return _getExpDim(pil.expressions[expId]);

    function _getExpDim(exp) {
        switch (exp.op) {
            case "add":
            case "sub":
            case "mul":
            case "addc":
            case "mulc":
            case "neg":
                let md = 1;
                for (let i=0; i<exp.values.length; i++) {
                    const d = _getExpDim(exp.values[i]);
                    if (d>md) md=d;
                }
                return md;
            case "cm": return pil.cmDims[exp.id];
            case "const": return 1;
            case "exp": return _getExpDim(pil.expressions[exp.id]);
            case "q": return _getExpDim(pil.expressions[pil.q2exp[exp.id]]);
            case "number": return 1;
            case "public": return 1;
            case "challenge": return 3;
            case "eval": return 3;
            case "xDivXSubXi":  return 3;
            case "xDivXSubWXi": return 3;
            case "x": return 1;
            default: throw new Error("Exp op not defined: " + exp.op);
        }
    }
}

function setCodeDimensions(code, starkInfo, dimX) {
    const tmpDim = [];

    _setCodeDimensions(code.first);
    _setCodeDimensions(code.i);
    _setCodeDimensions(code.last);


    function _setCodeDimensions(code) {

        for (let i=0; i<code.length; i++) {
            if (i==11759) {
                console.log(i);
            }
            let newDim;
            switch (code[i].op) {
                case 'add': newDim = Math.max(getDim(code[i].src[0]), getDim(code[i].src[1])); break;
                case 'sub': newDim = Math.max(getDim(code[i].src[0]), getDim(code[i].src[1])); break;
                case 'mul': newDim = Math.max(getDim(code[i].src[0]), getDim(code[i].src[1])); break;
                case 'copy': newDim = getDim(code[i].src[0]); break;
                default: throw new Error("Invalid op:"+ code[i].op);
            }
            setDim(code[i].dest, newDim);
        }


        function getDim(r) {
            let d;
            switch (r.type) {
                case "tmp": d=tmpDim[r.id]; break;
                case "tree1": d=r.dim; break;
                case "tree2": d=r.dim; break;
                case "tree3": d=r.dim; break;
                case "tree4": d=r.dim; break;
                case "exp": d= starkInfo.varPolMap[starkInfo.exps_2ns[r.id]] ?
                               starkInfo.varPolMap[starkInfo.exps_2ns[r.id]].dim:
                               starkInfo.varPolMap[starkInfo.exps_n[r.id]].dim; break;
                case "cm": d=starkInfo.varPolMap[starkInfo.cm_2ns[r.id]].dim; break;
                case "q": d=starkInfo.varPolMap[starkInfo.qs[r.id]].dim; break;
                case "const": d=1; break;
                case "eval": d=3; break;
                case "number": d=1; break;
                case "public": d=1; break;
                case "challenge": d=3; break;
                case "xDivXSubXi": d=dimX; break;
                case "xDivXSubWXi": d=dimX; break;
                case "x": d=dimX; break;
                case "Z": d=3; break;
                case "Zi": d=1; break;
                default: throw new Error("Invalid reference type get: " + r.type);
            }
            if (!d) {
                throw new Error("Invalid dim");
            }
            r.dim = d;
            return d;
        }

        function setDim(r, dim) {
            switch (r.type) {
                case "tmp": tmpDim[r.id] = dim; r.dim=dim; return;
                case "exp":
                case "cm":
                case "q": r.dim=dim; return;
                default: throw new Error("Invalid reference type set: " + r.type);
            }
        }
    }

}


