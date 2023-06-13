const { assert } = require("chai");
const fs = require("fs");
const path = require("path");
const F3G = require("../../node_modules/pil-stark/src/f3g.js");
const {log2} = require("../utils");
const {tmpName} = require("tmp-promise");
const { newConstantPolsArray, compile, getKs } = require("pilcom");
const ejs = require("ejs");
const r1cs2plonk = require("../../node_modules/pil-stark/src/r1cs2plonk");

module.exports = async function plonkSetup(r1cs, options) {
    const F = new F3G();

    // seems to use plonkup to convert r1cs circuits to plonkish circuits.
    const [plonkConstraints, plonkAdditions] = r1cs2plonk(F, r1cs);

    const plonkInfo = getNormalPlonkInfo();

    console.log(`nConstraints: ${plonkInfo.nConstraints}`);
    console.log(`PLONK nConstraints: ${plonkInfo.nPlonkConstraints}`);
    console.log(`plonkAdditions: ${plonkInfo.nPlonkAdditions}`);

    const customGatesInfo = getCustomGatesInfo();

    // the number of public parameters is equal to the sum of r1cs-inputs and r1cs-outputs  
    let nPublics = r1cs.nOutputs + r1cs.nPubInputs;
    const nPublicRows = Math.floor((nPublics - 1)/12) +1;
    
    const NUsed = nPublicRows + plonkInfo.N + customGatesInfo.nCMul + customGatesInfo.nMDS*2;
    let nBits = log2(NUsed - 1) + 1;

    if (options.forceNBits) {
        if (options.forceNBits < nBits) {
            throw new Error("ForceNBits is less than required");
        }
        nBits = options.forceNBits;
    }
    const N = 1 << nBits;

    const template = await fs.promises.readFile(path.join(__dirname, "compressor12.pil.ejs"), "utf8");
    const obj = {
        N: N,
        NUsed: NUsed,
        nBits: nBits,
        nPublics: nPublics
    };

    const pilStr = ejs.render(template ,  obj);
    const pilFile = await tmpName();
    await fs.promises.writeFile(pilFile, pilStr, "utf8");

    const pil = await compile(F, pilFile);
    const constPols =  newConstantPolsArray(pil);

    fs.promises.unlink(pilFile);

    const sMap = [];
    for (let i=0;i<12; i++) {
        sMap[i] = new Uint32Array(NUsed);
    }

    let r=0;

    // Paste public inputs.
    for (let i=0; i<nPublicRows; i++) {
        constPols.Compressor.Qm[r+i] = 0n;
        constPols.Compressor.Ql[r+i] = 0n;
        constPols.Compressor.Qr[r+i] = 0n;
        constPols.Compressor.Qo[r+i] = 0n;
        constPols.Compressor.Qk[r+i] = 0n;
        constPols.Compressor.QCMul[r+i] = 0n;
        constPols.Compressor.QMDS[r+i] = 0n;
    }

    for (let i=0; i<nPublics; i++) {
        sMap[i%12][r+Math.floor(i/12)] = 1+i;
    }

    for (let i=nPublics; i<nPublicRows*12; i++) {
        sMap[i%12][r+Math.floor(i/12)] = 0;
    }
    r += nPublicRows;

    // Paste plonk constraints.
    const partialRows = {};
    for (let i=0; i<plonkConstraints.length; i++) {
        if ((i%10000) == 0) console.log(`Processing constraint... ${i}/${plonkConstraints.length}`);
        const c = plonkConstraints[i];
        const k= c.slice(3, 8).map( a=> a.toString(16)).join(",");
        if (partialRows[k]) {
            const pr = partialRows[k];
            sMap[pr.nUsed*3][pr.row] = c[0];
            sMap[pr.nUsed*3+1][pr.row] = c[1];
            sMap[pr.nUsed*3+2][pr.row] = c[2];
            pr.nUsed ++;
            if (pr.nUsed == 4) {
                delete partialRows[k];
            }
        } else {
            constPols.Compressor.Qm[r] = c[3];
            constPols.Compressor.Ql[r] = c[4];
            constPols.Compressor.Qr[r] = c[5];
            constPols.Compressor.Qo[r] = c[6];
            constPols.Compressor.Qk[r] = c[7];
            constPols.Compressor.QCMul[r] = 0n;
            constPols.Compressor.QMDS[r] = 0n;
            sMap[0][r] = c[0];
            sMap[1][r] = c[1];
            sMap[2][r] = c[2];
            partialRows[k] = {
                row: r,
                nUsed: 1
            };
            r ++;
        }
    }

    // Terminate the empty rows (Copyn the same constraint)
    const openRows = Object.keys(partialRows);
    for (let i=0; i<openRows.length; i++) {
        const pr = partialRows[openRows[i]];
        for (let j=pr.nUsed; j<4; j++) {
            sMap[j*3][pr.row] = sMap[0][pr.row];
            sMap[j*3+1][pr.row] = sMap[1][pr.row];;
            sMap[j*3+2][pr.row] = sMap[2][pr.row];;
        }
    }

    // Generate Custom Gates
    for (let i=0; i<r1cs.customGatesUses.length; i++) {
        if ((i%10000) == 0) console.log(`Processing custom gates... ${i}/${r1cs.customGatesUses.length}`);
        const cgu = r1cs.customGatesUses[i];
        if (cgu.id == customGatesInfo.CMDSId) {
            assert(cgu.signals.length == 24);
            for (let i=0; i<12; i++) {
                sMap[i][r] = cgu.signals[i];
                sMap[i][r+1] = cgu.signals[i+12];
            }
            constPols.Compressor.Qm[r] = 0n;
            constPols.Compressor.Ql[r] = 0n;
            constPols.Compressor.Qr[r] = 0n;
            constPols.Compressor.Qo[r] = 0n;
            constPols.Compressor.Qk[r] = 0n;
            constPols.Compressor.QCMul[r] = 0n;
            constPols.Compressor.QMDS[r] = 1n;
            constPols.Compressor.Qm[r+1] = 0n;
            constPols.Compressor.Ql[r+1] = 0n;
            constPols.Compressor.Qr[r+1] = 0n;
            constPols.Compressor.Qo[r+1] = 0n;
            constPols.Compressor.Qk[r+1] = 0n;
            constPols.Compressor.QCMul[r+1] = 0n;
            constPols.Compressor.QMDS[r+1] = 0n;

            r+=2;
        } else if (cgu.id == customGatesInfo.CMulId) {
            for (let i=0; i<9; i++) {
                sMap[i][r] = cgu.signals[i];
            }
            for (let i=9; i<12; i++) {
                sMap[i][r] = 0;
            }
            constPols.Compressor.Qm[r] = 0n;
            constPols.Compressor.Ql[r] = 0n;
            constPols.Compressor.Qr[r] = 0n;
            constPols.Compressor.Qo[r] = 0n;
            constPols.Compressor.Qk[r] = 0n;
            constPols.Compressor.QCMul[r] = 1n;
            constPols.Compressor.QMDS[r] = 0n;

            r+= 1;
        }
    }

    // Calculate S Polynomials
    const ks = getKs(F, 11);
    let w = F.one;
    for (let i=0; i<N; i++) {
        if ((i%10000) == 0) console.log(`Preparing S... ${i}/${N}`);
        constPols.Compressor.S[0][i] = w;
        for (let j=1; j<12; j++) {
            constPols.Compressor.S[j][i] = F.mul(w, ks[j-1]);
        }
        w = F.mul(w, F.w[nBits]);
    }

    const lastSignal = {}
    for (let i=0; i<r; i++) {
        if ((i%10000) == 0) console.log(`Connection S... ${i}/${r}`);
        for (let j=0; j<12; j++) {
            if (sMap[j][i]) {
                if (typeof lastSignal[sMap[j][i]] !== "undefined") {
                    const ls = lastSignal[sMap[j][i]];
                    connect(constPols.Compressor.S[ls.col], ls.row, constPols.Compressor.S[j], i);
                } else {
                    lastSignal[sMap[j][i]] = {
                        col: j,
                        row: i
                    };
                }
            }
        }
    }

    // Fill unused rows
    while (r<N) {
        if ((r%100000) == 0) console.log(`Empty gates... ${r}/${N}`);
        constPols.Compressor.Qm[r] = 0n;
        constPols.Compressor.Ql[r] = 0n;
        constPols.Compressor.Qr[r] = 0n;
        constPols.Compressor.Qo[r] = 0n;
        constPols.Compressor.Qk[r] = 0n;
        constPols.Compressor.QCMul[r] = 0n;
        constPols.Compressor.QMDS[r] = 0n;
        r +=1;
    }

    for (let i=0; i<nPublicRows; i++) {
        const L = constPols.Global["L" + (i+1)];
        for (let i=0; i<N; i++) {
            L[i] = 0n;
        }
        L[i] = 1n;
    }

    return {
        pilStr: pilStr,
        constPols: constPols,
        sMap: sMap,
        plonkAdditions: plonkAdditions
    };

    function connect(p1, i1, p2, i2) {
        [p1[i1], p2[i2]] = [p2[i2], p1[i1]];
    }



    function getNormalPlonkInfo() {

        const uses = {};
        for (let i=0; i<plonkConstraints.length; i++) {
            if ((i%10000) == 0) console.log(`Plonk info constraint processing... ${i}/${plonkConstraints.length}`);
            const c = plonkConstraints[i];
            const k= c.slice(3, 8).map( a=> a.toString(16)).join(",");
            uses[k] ||=  0;
            uses[k]++;
        };
        const result = Object.keys(uses).map((key) => [key, uses[key]]);
        result.sort((a,b) => b[1] - a[1] );

        let N = 0;
        result.forEach((r) => {
            console.log(`${r[0]} => ${r[1]}`);
            N += Math.floor((r[1] - 1) / 4) +1;
        });


        return {
            N: N,
            nConstraints: r1cs.nConstraints,
            nPlonkConstraints: plonkConstraints.length,
            nPlonkAdditions: plonkAdditions.length
        };

    }

    function getCustomGatesInfo() {
        let CMulId;
        let CMDSId;
        assert(r1cs.customGates.length == 2);
        for (let i=0; i<r1cs.customGates.length; i++) {
            switch (r1cs.customGates[i].templateName) {
                case "CMul":
                    CMulId =i;
                    assert(r1cs.customGates[0].parameters.length == 0);
                    break;
                case "MDS":
                    CMDSId =i;
                    assert(r1cs.customGates[0].parameters.length == 0);
                    break;
                default:
                    throw new Error("Invalid custom gate: " , r1cs.customGates[0].name);
            }
        }
        if (typeof CMulId === "undefined") throw new Error("CMul custom gate not defined");
        if (typeof CMDSId === "undefined") throw new Error("CMDSId custom gate not defined");

        const res = {
            CMulId: CMulId,
            CMDSId: CMDSId,
            nCMul: 0,
            nMDS: 0
        }

        for (let i=0; i< r1cs.customGatesUses.length; i++) {
            if (r1cs.customGatesUses[i].id == CMulId) {
                res.nCMul ++;
            } else if (r1cs.customGatesUses[i].id == CMDSId) {
                res.nMDS ++;
            } else {
                throw new Error("Custom gate not defined" + r1cs.customGatesUses[i].id);
            }
        }

        return res;
    }

}
