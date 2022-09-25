// copied and modified from pil-stark
const { assert } = require("chai");
const {polMulAxi, evalPol} = require("./polutils");
const {log2} = require("./utils");
const GL3 = require("./f3g");
const {BigBuffer} = require("pilcom");

class FRI {

    constructor(starkStruct, MH) {
        this.F = new GL3();
        this.inNBits = starkStruct.nBitsExt;
        this.maxDegNBits = starkStruct.nBits;
        this.nQueries = starkStruct.nQueries;
        this.MH = MH;
        if (starkStruct) {
            this.steps = starkStruct.steps;
        } else {
            throw new Error("stark struct not defined");
        }
    }

    async prove(transcript, pol, queryPol) {
        const self = this;
        const proof = [];
        const F = this.F;

        let polBits = log2(pol.length);
        assert(1<<polBits == pol.length, "Invalid poluynomial size");    // Check the input polynomial is a power of 2
        assert(polBits == this.inNBits, "Invalid polynomial size");

        let shiftInv = F.shiftInv;
        let shift = F.shift;
        let tree = [];

        for (let si = 0; si<this.steps.length; si++) proof[si] = {};
        for (let si = 0; si<this.steps.length; si++) {
            const reductionBits = polBits - this.steps[si].nBits;

            const pol2N = 1 << (polBits - reductionBits);
            const nX = pol.length / pol2N;

            const pol2_e = new Array(pol2N);

            let special_x = transcript.getField();

            let sinv = shiftInv;
            const wi = F.inv(F.w[polBits]);
            for (let g = 0; g<pol.length/nX; g++) {
                if (si==0) {
                    pol2_e[g] = pol[g];
                } else {
                    const ppar = new Array(nX);
                    for (let i=0; i<nX; i++) {
                        ppar[i] = pol[(i*pol2N)+g];
                    }
                    const ppar_c = F.ifft(ppar);
                    polMulAxi(F, ppar_c, F.one, sinv);    // Multiplies coefs by 1, shiftInv, shiftInv^2, shiftInv^3, ......

                    pol2_e[g] = evalPol(F, ppar_c, special_x);
                    sinv = F.mul(sinv, wi);
                }
            }


            if (si < this.steps.length-1) {
                const nGroups = 1<< this.steps[si+1].nBits;

                let groupSize = (1 << this.steps[si].nBits) / nGroups;


                const pol2_etb = getTransposedBuffer(pol2_e, this.steps[si+1].nBits);

                tree[si] = await this.MH.merkelize(pol2_etb, 3* groupSize, nGroups);

                proof[si+1].root= this.MH.root(tree[si]);
                transcript.put(this.MH.root(tree[si]));
            } else {
                for (let i=0; i<pol2_e.length; i++) {
                    transcript.put(pol2_e[i]);
                }
            }

            pol = pol2_e;
            polBits = polBits-reductionBits;

            for (let j=0; j<reductionBits; j++) {
                shiftInv = F.mul(shiftInv, shiftInv);
                shift = F.mul(shift, shift);
            }
        }
        const lastPol = [];
        for (let i=0; i<pol.length; i++) {
            lastPol.push(pol[i]);
        }
        proof.push(lastPol);



        const ys = transcript.getPermutations(this.nQueries, this.steps[0].nBits);

        for (let si = 0; si<this.steps.length; si++) {

            proof[si].polQueries = [];
            for (let i=0; i<ys.length; i++) {
                const gIdx =
                proof[si].polQueries.push(queryPol(ys[i]));
            }


            if (si < this.steps.length -1) {
                queryPol = (idx) => {
                    return self.MH.getGroupProof(tree[si], idx);
                }

                for (let i=0; i<ys.length; i++) {
                    ys[i] = ys[i] % (1 << this.steps[si+1].nBits);
                }
            }
        }

        return proof;
    }

    verify(transcript, proof, checkQuery) {
        const self = this;
        const F = this.F;
        const GMT = [];

        assert(proof.length == this.steps.length+1, "Invalid proof size");


        let special_x = [];

        for (let si=0; si<this.steps.length; si++) {
            special_x[si] = transcript.getField();

            if (si < this.steps.length-1) {
                const nGroups = 1<< this.steps[si+1].nBits;

                let groupSize = (1 << this.steps[si].nBits) / nGroups;
                transcript.put(proof[si+1].root);
            } else {
                for (let i=0; i<proof[proof.length-1].length; i++) {
                    transcript.put(proof[proof.length-1][i]);
                }
            }
        }


        const nQueries = this.nQueries;
        const ys = transcript.getPermutations(this.nQueries, this.steps[0].nBits);

        let polBits = this.inNBits;
        let shift = F.shift;
        for (let si=0; si<this.steps.length; si++) {

            const proofItem=proof[si];

            const reductionBits = polBits - this.steps[si].nBits;

            for (let i=0; i<nQueries; i++) {
                const pgroup_e = checkQuery(proofItem.polQueries[i], ys[i]);
                if (!pgroup_e) return false;

                const pgroup_c = F.ifft(pgroup_e);
                const sinv = F.inv(F.mul( shift, F.exp(  F.w[polBits], ys[i])));
//                polMulAxi(F, pgroup_c, F.one, sinv);    // Multiplies coefs by 1, shiftInv, shiftInv^2, shiftInv^3, ......
//                const ev = evalPol(F, pgroup_c, special_x[si]);
                const ev = evalPol(F, pgroup_c, F.mul(special_x[si], sinv));

                if (si < this.steps.length - 1) {
                    const nextNGroups = 1 << this.steps[si+1].nBits
                    const groupIdx  =Math.floor(ys[i] / nextNGroups);
                    if (!F.eq(get3(proof[si+1].polQueries[i][0], groupIdx), ev)) return false;
                } else {
                    if (!F.eq(proof[si+1][ys[i]], ev)) return false;
                }
            }

            checkQuery = (query, idx) => {
                const res = self.MH.verifyGroupProof(proof[si+1].root, query[1], idx, query[0]);
                if (!res) return false;
                return split3(query[0]);
            }

            polBits = this.steps[si].nBits;
            for (let j=0; j<reductionBits; j++) shift = F.mul(shift, shift);

            if (si < this.steps.length -1) {
                for (let i=0; i<ys.length; i++) {
                    ys[i] = ys[i] % (1 << this.steps[si+1].nBits);
                }
            }

        }

        const lastPol_e = proof[proof.length-1];

        let maxDeg;
        if (( polBits - (this.inNBits - this.maxDegNBits)) <0) {
            maxDeg =0;
        } else {
            maxDeg = 1 <<  ( polBits - (this.inNBits - this.maxDegNBits));
        }

        const lastPol_c = F.ifft(lastPol_e);
        // We don't need to divide by shift as we just need to check for zeros

        for (let i=maxDeg+1; i< lastPol_c.length; i++) {
            if (!F.isZero(lastPol_c[i])) return false;
        }

        return true;

    }
}

module.exports = FRI;

function createPol(n) {
    const buff = new BigUint64Array(n*3*64)
    return new Proxy({
        buffer: buff,
        deg: n
    }, {
        get( obj, prop) {
            if (!isNaN(prop)) {
                prop = Number(prop);
                assert(prop<obj.deg, "Out of range");
                return [
                    obj.buffer[3*prop],
                    obj.buffer[3*prop+1],
                    obj.buffer[3*prop+2]
                ];
            } else if (prop == "length") {
                return obj.deg;
            } else if (prop == "buffer") {
                return obj.buffer;
            }
        },
        set( obj, prop, v) {
            if (!isNaN(prop)) {
                prop = Number(prop);
                assert(prop<obj.deg, "Out of range");
                if (Array.isArray(v)) {
                    [
                        obj.buffer[3*prop],
                        obj.buffer[3*prop+1],
                        obj.buffer[3*prop+2]
                    ] = v;
                } else {
                    [
                        obj.buffer[3*prop],
                        obj.buffer[3*prop+1],
                        obj.buffer[3*prop+2]
                    ] = [ v, 0n, 0n];
                }
                return true;
            }
        }

    });
}

function split3(arr) {
    const res = [];
    for (let i=0; i<arr.length; i+=3) {
        res.push([arr[i], arr[i+1], arr[i+2]]);
    }
    return res;
}

function get3(arr, idx) {
    return [arr[idx*3], arr[idx*3+1], arr[idx*3+2]];
}

function getTransposedBuffer(pol, trasposeBits) {
    const res = new BigBuffer(pol.length*3);
    const n = pol.length;
    const w = 1 << trasposeBits;
    const h = n/w;
    for (let i=0; i<w; i++) {
        for (let j=0; j<h; j++) {
            const fi = j*w + i;
            const di = i*h*3 +j*3;
            res.setElement(di, pol[fi][0]);
            res.setElement(di+1, pol[fi][1]);
            res.setElement(di+2, pol[fi][2]);
        }
    }
    return res;
}



