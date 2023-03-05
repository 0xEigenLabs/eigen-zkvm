
const buildPoseidon = require("@0xpolygonhermez/zkevm-commonjs").getPoseidon;

const BYTESPERELEMENT = 7;
const NELEMENTS = 8;
const BYTESPERBLOCK = BYTESPERELEMENT*NELEMENTS;

module.exports.buildConstants = async function (pols) {
    const poseidon = await buildPoseidon();
    const F = poseidon.F;

    const N = pols.lastBlock.length;


    const nBlocks = Math.floor((N - 1)/BYTESPERBLOCK)+1;

    let p =0;

    for (let i=0; i<nBlocks; i++) {
        const bytesBlock = N-p > BYTESPERBLOCK ? BYTESPERBLOCK : N-p;
        for (let j=0; j<bytesBlock; j++) {

            let acci = Math.floor(j / BYTESPERELEMENT);
            let sh = BigInt((j % BYTESPERELEMENT)*8);

            if (j == bytesBlock-1) {
                acci = 7;
                sh = BigInt(6*8);
            }

            for (let k=0; k<8; k++) {
                pols.F[k][p] =(k == acci) ? (1n << sh) : 0n;
            }
            pols.lastBlock[p] = (j == bytesBlock-1) ? 1n : 0n;

            p += 1;
        }
    }
}


module.exports.execute = async function (pols, input) {

    prepareInput(input);
    const poseidon = await buildPoseidon();
    const F = poseidon.F;

    const required = {
        PoseidonG: [],
    };

    const N = pols.acc[0].length;
    const POSEIDONG_PERMUTATION4_ID = 4;

    pols.crF = [];
    pols.crV = [];

    let p = 0;

    for (let i=0; i<NELEMENTS; i++) {
        pols.acc[i][p] = 0n;
    }

    for (let i=0; i<8; i++) {
        pols.crF[i] = pols[`crF${i}`];
        pols.crV[i] = pols[`crV${i}`];
    }

    pols.prevHash0[p] = 0n;
    pols.prevHash1[p] = 0n;
    pols.prevHash2[p] = 0n;
    pols.prevHash3[p] = 0n;
    pols.incCounter[p] = 1n;

    for (let k=0; k<8; k++) {
        pols.crV[k][p] = 0n;
    }

    let addr = 0n;

    for (let i=0; i<input.length; i++) {

        let curRead = -1;
        let lastOffset = 0n;

        for (let j=0; j<input[i].dataBytes.length; j++) {

            pols.freeIn[p] = BigInt(input[i].dataBytes[j]);

            const acci = Math.floor((j % BYTESPERBLOCK) / BYTESPERELEMENT);
            const sh = BigInt((j % BYTESPERELEMENT)*8);
            for (let k=0; k<NELEMENTS; k++) {
                if (k == acci) {
                    pols.acc[k][p+1] = pols.acc[k][p] | (pols.freeIn[p] << sh);
                } else {
                    pols.acc[k][p+1] = pols.acc[k][p];
                }
            }

            pols.prevHash0[p+1] = pols.prevHash0[p];
            pols.prevHash1[p+1] = pols.prevHash1[p];
            pols.prevHash2[p+1] = pols.prevHash2[p];
            pols.prevHash3[p+1] = pols.prevHash3[p];
            pols.incCounter[p+1] = pols.incCounter[p];

            pols.len[p] = input[i].realLen;
            pols.addr[p] = addr;
            pols.rem[p] = F.e(input[i].realLen - BigInt(j));
            pols.remInv[p] = pols.rem[p] == 0n ? 0n : F.inv(pols.rem[p]);
            pols.spare[p] = pols.rem[p] > 0xFFFFn ? 1n : 0n;
            pols.firstHash[p] = j==0 ? 1n : 0n;
            const lastBlock = (p % BYTESPERBLOCK) == (BYTESPERBLOCK - 1);
            const lastHash = lastBlock && (pols.spare[p] || !pols.rem[p]);

            // at least must be done a len before a digest
            pols.lastHashLen[p] = (lastHash && input[i].lenCalled) ? 1n: 0n
            pols.lastHashDigest[p] = (lastHash && input[i].digestCalled) ? 1n: 0n;

            if (lastOffset == 0n) {
                curRead += 1;
                pols.crLen[p] = curRead<input[i].reads.length ? BigInt(input[i].reads[curRead]) : 1n;
                pols.crOffset[p] = pols.crLen[p] - 1n;
            } else {
                pols.crLen[p] = pols.crLen[p-1];
                pols.crOffset[p] = pols.crOffset[p-1] - 1n;
            }
            pols.crOffsetInv[p] = pols.crOffset[p] == 0n ? 0n : F.inv(pols.crOffset[p]);

            const crAccI = Math.floor(Number(pols.crOffset[p]) / 4);
            const crSh = BigInt((Number(pols.crOffset[p]) % 4) * 8);

            for (let k=0; k<8; k++) {
                pols.crF[k][p] = (k==crAccI) ? 1n << crSh : 0n;
                if (pols.crOffset[p] == 0n) {
                    pols.crV[k][p+1] = 0n;
                } else {
                    pols.crV[k][p+1] = (k==crAccI) ? pols.crV[k][p] + (pols.freeIn[p] << crSh) : pols.crV[k][p];
                }
            }

            lastOffset = pols.crOffset[p];

            if (j % BYTESPERBLOCK == (BYTESPERBLOCK -1) ) {
                [
                    pols.curHash0[p],
                    pols.curHash1[p],
                    pols.curHash2[p],
                    pols.curHash3[p]
                ] = poseidon([
                    pols.acc[0][p+1],
                    pols.acc[1][p+1],
                    pols.acc[2][p+1],
                    pols.acc[3][p+1],
                    pols.acc[4][p+1],
                    pols.acc[5][p+1],
                    pols.acc[6][p+1],
                    pols.acc[7][p+1],
                ],  [
                    pols.prevHash0[p],
                    pols.prevHash1[p],
                    pols.prevHash2[p],
                    pols.prevHash3[p],
                ]);
                required.PoseidonG.push([
                    pols.acc[0][p+1],
                    pols.acc[1][p+1],
                    pols.acc[2][p+1],
                    pols.acc[3][p+1],
                    pols.acc[4][p+1],
                    pols.acc[5][p+1],
                    pols.acc[6][p+1],
                    pols.acc[7][p+1],
                    pols.prevHash0[p],
                    pols.prevHash1[p],
                    pols.prevHash2[p],
                    pols.prevHash3[p],
                    pols.curHash0[p],
                    pols.curHash1[p],
                    pols.curHash2[p],
                    pols.curHash3[p],
                    POSEIDONG_PERMUTATION4_ID
                ]);
                pols.acc[0][p+1] = 0n;
                pols.acc[1][p+1] = 0n;
                pols.acc[2][p+1] = 0n;
                pols.acc[3][p+1] = 0n;
                pols.acc[4][p+1] = 0n;
                pols.acc[5][p+1] = 0n;
                pols.acc[6][p+1] = 0n;
                pols.acc[7][p+1] = 0n;
                for (k=1; k<BYTESPERBLOCK; k++) {
                    pols.curHash0[p-k] = pols.curHash0[p];
                    pols.curHash1[p-k] = pols.curHash1[p];
                    pols.curHash2[p-k] = pols.curHash2[p];
                    pols.curHash3[p-k] = pols.curHash3[p];
                }
                pols.prevHash0[p+1] = pols.curHash0[p];
                pols.prevHash1[p+1] = pols.curHash1[p];
                pols.prevHash2[p+1] = pols.curHash2[p];
                pols.prevHash3[p+1] = pols.curHash3[p];
                pols.incCounter[p+1] = pols.incCounter[p] + 1n;



                if (j == input[i].dataBytes.length - 1) {
                    pols.prevHash0[p+1] = 0n;
                    pols.prevHash1[p+1] = 0n;
                    pols.prevHash2[p+1] = 0n;
                    pols.prevHash3[p+1] = 0n;
                    pols.incCounter[p+1] = 1n;
                }

            }

            p += 1;
        }
        addr += 1n;
    }

    const nFullUnused = Math.floor((N -p - 1)/BYTESPERBLOCK)+1;

    const h0 = poseidon([ 0x1n, 0n, 0n, 0n, 0n, 0n, 0n, 0x80n << 48n ], [0n, 0n, 0n, 0n]);
    required.PoseidonG.push([ 0x1n, 0n, 0n, 0n, 0n, 0n, 0n, 0x80n << 48n, 0n, 0n, 0n, 0n, ...h0, POSEIDONG_PERMUTATION4_ID ]);


    for (let i=0; i<nFullUnused; i++) {
        const bytesBlock = N-p > BYTESPERBLOCK ? BYTESPERBLOCK : N-p;
        if (bytesBlock < 2) {
            throw new Error("Alignment is not possible");
        }
        for (let j=0; j<bytesBlock; j++) {
            if (j==0) {
                pols.freeIn[p] = 1n
            } else if (j==bytesBlock-1) {
                pols.freeIn[p] = 0x80n;
            } else {
                pols.freeIn[p] = 0n;
            }
            pols.acc[0][p] = (j==0) ? 0n : 0x1n;
            pols.acc[1][p] = 0n;
            pols.acc[2][p] = 0n;
            pols.acc[3][p] = 0n;
            pols.acc[4][p] = 0n;
            pols.acc[5][p] = 0n;
            pols.acc[6][p] = 0n;
            pols.acc[7][p] = 0n;
            pols.len[p] = 0n;
            pols.addr[p] = addr;
            pols.rem[p] = F.e(-j);
            pols.remInv[p] = pols.rem[p] == 0n ? 0n : F.inv(pols.rem[p]);
            pols.spare[p] = j>0 ? 1n : 0n;
            pols.firstHash[p] = j==0 ? 1n : 0n;
            pols.lastHashLen[p] = 0n;
            pols.lastHashDigest[p] = 0n;

            pols.prevHash0[p] = 0n;
            pols.prevHash1[p] = 0n;
            pols.prevHash2[p] = 0n;
            pols.prevHash3[p] = 0n;
            pols.incCounter[p] = 1n;
            pols.curHash0[p] = h0[0];
            pols.curHash1[p] = h0[1];
            pols.curHash2[p] = h0[2];
            pols.curHash3[p] = h0[3];

            pols.crOffset[p] = 0n;
            pols.crLen[p] = 1n;
            pols.crOffsetInv[p] = 0n;
            pols.crF0[p] = 1n;
            pols.crF1[p] = 0n;
            pols.crF1[p] = 0n;
            pols.crF2[p] = 0n;
            pols.crF3[p] = 0n;
            pols.crF4[p] = 0n;
            pols.crF5[p] = 0n;
            pols.crF6[p] = 0n;
            pols.crF7[p] = 0n;

            pols.crV0[p] = 0n;
            pols.crV1[p] = 0n;
            pols.crV2[p] = 0n;
            pols.crV3[p] = 0n;
            pols.crV4[p] = 0n;
            pols.crV5[p] = 0n;
            pols.crV6[p] = 0n;
            pols.crV7[p] = 0n;

            p += 1;
        }
        addr += 1n;
    }

    return required;
}

function prepareInput(input) {
    function hexToBytes(hex) {
        for (var bytes = [], c = 0; c < hex.length; c += 2)
            bytes.push(parseInt(hex.substr(c, 2), 16));
        return bytes;
    }

    for (let i=0; i<input.length; i++) {
        // TODO: check if test send information as string and order of bytes on data
        if (typeof input[i].data === 'string') {
            input[i].dataBytes = hexToBytes(input[i].data);
        } else {
            input[i].dataBytes = input[i].data;
        }
        input[i].realLen = BigInt(input[i].dataBytes.length);

        input[i].dataBytes.push(0x1);

        while (input[i].dataBytes.length % BYTESPERBLOCK) input[i].dataBytes.push(0);

        input[i].dataBytes[ input[i].dataBytes.length - 1] |= 0x80;
    }
}
