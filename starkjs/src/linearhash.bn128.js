// copied and modified from pil-stark
module.exports = class LinearHashBN {

    constructor(poseidon) {
        this.H = poseidon;
        this.F = poseidon.F;
    }

    hash(vals) {
        const F = this.F;

        let st = F.zero;

        const vals3 = [];

        let acc = F.zero;
        let accN = 0;
        for(let i=0; i<vals.length; i++) {
            if (Array.isArray(vals[i])) {
                for (let k=0; k<vals[i].length; k++) {
                    acc = F.add(acc, F.e(BigInt(vals[i][k]) << BigInt(64*accN)) );
                    accN++;
                    if (accN == 3) {
                        vals3.push(acc);
                        acc =F.zero;
                        accN = 0;
                    }
                }
            } else {
                acc = F.add(acc, F.e(BigInt(vals[i]) << BigInt(64*accN)) );
                accN++;
                if (accN == 3) {
                    vals3.push(acc);
                    acc =F.zero;
                    accN = 0;
                }
            }
        }
        if (accN) {
            vals3.push(acc);
        }

        if (vals3.length == 0) return st;
        if (vals3.length == 1) return vals3[0];
        let inHash = [];
        for (let i=0; i<vals3.length;i++) {
            inHash.push(vals3[i]);
            if (inHash.length == 16) {
                st = this.H(inHash, st);
                inHash.length = 0;
            }
        }
        if (inHash.length>0) {
//            while (inHash.length<16) inHash.push(this.F.zero);
            st = this.H(inHash, st);
        }
        return st;
    }
}