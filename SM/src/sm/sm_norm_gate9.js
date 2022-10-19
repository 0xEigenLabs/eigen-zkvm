const assert = require("assert");

module.exports.buildConstants = async function (pols) {
    const N = pols.Value3.length;

    if (N < (1<<21)) throw new Error("GateNorm9 Minimum deg = 2**21");

    const nBlocks = Math.floor(N/3);

    for (let i=0; i<N; i++) {
        const v = BigInt(i) & 0b111111111111111111111n;
        pols.Value3[i] = v;
        pols.Value3Norm[i] = v & 0b000000100000010000001n;

        a0 = BigInt((i >> 0) & 1);
        a1 = BigInt((i >> 1) & 1);
        a2 = BigInt((i >> 2) & 1);
        b0 = BigInt((i >> 3) & 1);
        b1 = BigInt((i >> 4) & 1);
        b2 = BigInt((i >> 5) & 1);
        op = BigInt((i >> 6) & 1);

        pols.Gate9Type[i] = op;
        pols.Gate9A[i] = a0 + (a1 << 7n) + (a2 << 14n);
        pols.Gate9B[i] = b0 + (b1 << 7n) + (b2 << 14n);
        pols.Gate9C[i] = op ? ((pols.Gate9A[i] ^ 0b000000100000010000001n) & pols.Gate9B[i]) : (pols.Gate9A[i] ^ pols.Gate9B[i]);

        const b = Math.floor(i/3);
        let k = i%3;
        if (b<nBlocks) {
            pols.Factor[i] = 1n << BigInt(k * 21);
            pols.Latch[i] = k==2 ? 1n : 0n;
        } else {
            pols.Factor[i] = 0n;
            pols.Latch[i] = 0n;
        }
    }

}

module.exports.execute = async function (pols, input) {
    const N = pols.a.length;

    const nBlocks = Math.floor(N/3);

    if (input.length > nBlocks) throw new Error("NormGate9 not big enougth");

    let acca = 0n;
    let accb = 0n;
    let accc = 0n;
    let p=0;
    for (let i=0; i<input.length; i++) {
        for (let j=0; j<3; j++) {
            pols.freeA[p] = (input[i][1]  >> (21n * BigInt(j))) & 0b111111111111111111111n
            pols.freeB[p] = (input[i][2]  >> (21n * BigInt(j))) & 0b111111111111111111111n

            pols.freeANorm[p] = pols.freeA[p] & 0b000000100000010000001n
            pols.freeBNorm[p] = pols.freeB[p] & 0b000000100000010000001n

            if (input[i][0] == "XORN") {
                pols.gateType[p] = 0n;
                pols.freeCNorm[p] = pols.freeANorm[p] ^ pols.freeBNorm[p];
            } else if (input[i][0] == "ANDP") {
                pols.gateType[p] = 1n;
                pols.freeCNorm[p] = (pols.freeANorm[p] ^ 0b000000100000010000001n) & pols.freeBNorm[p];
            } else {
                assert(false, "Invalid door " + input[i][0])
            }

            pols.a[p] = acca;
            pols.b[p] = accb;
            pols.c[p] = accc;

            acca = acca + (pols.freeA[p] << (21n * BigInt(j)));
            accb = accb + (pols.freeB[p] << (21n * BigInt(j)));
            accc = accc + (pols.freeCNorm[p] << (21n * BigInt(j)));

            if (j==2) {
                acca = 0n;
                accb = 0n;
                accc = 0n;
            }

            p+=1;
        }
    }

    while (p<N) {

        pols.freeA[p] = 0n
        pols.freeB[p] = 0n
        pols.freeANorm[p] = 0n
        pols.freeBNorm[p] = 0n
        pols.freeCNorm[p] = 0n
        pols.gateType[p] = 0n;

        pols.a[p] = acca;
        pols.b[p] = accb;
        pols.c[p] = accc;

        acca = 0n;
        accb = 0n;
        accc = 0n;

        p+=1;
    }

    pols.a[0] = acca;
    pols.b[0] = accb;
    pols.c[0] = accc;
}
