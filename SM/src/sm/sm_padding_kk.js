const { scalar2fea } = require("@0xpolygonhermez/zkevm-commonjs/src/smt-utils");
const { ethers } = require("ethers");
Scalar = require("ffjavascript").Scalar;

const buildPoseidon = require("@0xpolygonhermez/zkevm-commonjs").getPoseidon;

const BYTESPERBLOCK = 136;
const BlockSize = 155286;

module.exports.buildConstants = async function (pols) {
    const poseidon = await buildPoseidon();
    const F = poseidon.F;

    const N = pols.lastBlock.length;

    const nBlocks = 44*Math.floor((N-1)/BlockSize);

    let p =0;

    for (let i=0; i<nBlocks; i++) {
        const bytesBlock = 136;
        for (let j=0; j<bytesBlock; j++) {
            pols.lastBlock[p] = (j == bytesBlock-1) ? 1n : 0n;
            pols.lastBlockLatch[p] = (j == bytesBlock-1) ? 1n : 0n;
            pols.r8Id[p] = F.e(p);
            pols.sOutId[p] =  (j == bytesBlock-1) ? F.e(i) : F.zero;
            pols.forceLastHash[p] = ((j == bytesBlock-1)&&(i==nBlocks-1)) ? F.one : F.zero;
            pols.r8valid[p] = F.one;
            p += 1;
        }
    }

    for (let i=p; i<N; i++) {
        pols.r8Id[i] = F.zero;    // Must repeat the first byte
        pols.lastBlock[i] = i<N-1 ? F.zero : F.one;
        pols.lastBlockLatch[i] = F.zero;
        pols.sOutId[i] =  F.zero;
        pols.forceLastHash[i] = i==N-1 ? F.one : F.zero;
        pols.r8valid[i] = F.zero;
    }
}


module.exports.execute = async function (pols, input) {
    const poseidon = await buildPoseidon();
    const F = poseidon.F;

    prepareInput(input);

    const required = {
        paddingKKBit: []
    };

    const N = pols.freeIn.length;

    pols.crF = [];
    pols.crV = [];

    let p = 0;

    let addr = 0n;

    for (let i=0; i<8; i++) {
        pols.crF[i] = pols[`crF${i}`];
        pols.crV[i] = pols[`crV${i}`];
    }

    for (let k=0; k<8; k++) {
        pols.crV[k][p] = 0n;
    }

    for (let i=0; i<input.length; i++) {

        let curRead = -1;
        let lastOffset = 0n;

        for (let j=0; j<input[i].dataBytes.length; j++) {

            pols.freeIn[p] = BigInt(input[i].dataBytes[j]);

            pols.len[p] = input[i].realLen;
            pols.addr[p] = addr;
            pols.connected[p] = j<BYTESPERBLOCK ? 0n : 1n;
            pols.rem[p] = F.e(input[i].realLen - BigInt(j));
            pols.remInv[p] = pols.rem[p] == 0n ? 0n : F.inv(pols.rem[p]);

            // spare means we are in padding zone (realLen < j < dataBytes.length )
            // check if pols.rem[p] was "negative"
            pols.spare[p] = pols.rem[p] > 0x7FFFFFFF80000000n ? 1n : 0n;
            pols.firstHash[p] = j==0 ? 1n : 0n;
            pols.incCounter[p] = BigInt(Math.floor(j / BYTESPERBLOCK) +1);
            const lastBlockLatch = (p % BYTESPERBLOCK) == (BYTESPERBLOCK - 1);
            const lastHashLatch = lastBlockLatch && (pols.spare[p] || !pols.rem[p]);

            pols.lastHashLen[p] = lastHashLatch && input[i].lenCalled ? 1n: 0n;
            pols.lastHashDigest[p] = lastHashLatch && input[i].digestCalled ? 1n: 0n;

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
                required.paddingKKBit.push({
                    r: input[i].dataBytes.slice(j - BYTESPERBLOCK +1, j+1),
                    connected: (j<BYTESPERBLOCK) ? false : true
                });

                if (j == input[i].dataBytes.length - 1) {

                    [
                        pols.hash0[p],
                        pols.hash1[p],
                        pols.hash2[p],
                        pols.hash3[p],
                        pols.hash4[p],
                        pols.hash5[p],
                        pols.hash6[p],
                        pols.hash7[p]
                    ] =  scalar2fea(F, Scalar.e(input[i].hash));

                    for (k=1; k<input[i].dataBytes.length; k++) {
                        pols.hash0[p-k] = pols.hash0[p];
                        pols.hash1[p-k] = pols.hash1[p];
                        pols.hash2[p-k] = pols.hash2[p];
                        pols.hash3[p-k] = pols.hash3[p];
                        pols.hash4[p-k] = pols.hash4[p];
                        pols.hash5[p-k] = pols.hash5[p];
                        pols.hash6[p-k] = pols.hash6[p];
                        pols.hash7[p-k] = pols.hash7[p];
                    }
                }

            }

            p += 1;
        }
        addr += 1n;
    }

    const nTotalBlocks = 44*Math.floor(N/BlockSize);
    const nUsedBlocks = p/BYTESPERBLOCK;

    if (nUsedBlocks > nTotalBlocks) throw new Error(`Too many keccak blocks (${nUsedBlocks} vs ${nTotalBlocks} BS:${BlockSize})`);

    const nFullUnused = nTotalBlocks - nUsedBlocks;

    const bytes0 = [];
    for (let i=0; i<BYTESPERBLOCK; i++) {
        bytes0[i] = i==0 ? 1 : (  (i==BYTESPERBLOCK-1) ? 0x80 : 0);
    }
    const hash0 =  scalar2fea(F, Scalar.e(ethers.utils.keccak256([])));

    for (let i=0; i<nFullUnused; i++) {

        for (let j=0; j<BYTESPERBLOCK; j++) {

            pols.freeIn[p] = j==0 ? F.one : ((j == BYTESPERBLOCK-1) ? F.e(0x80) : F.zero);

            pols.len[p] = F.zero;
            pols.addr[p] = addr;
            pols.rem[p] = F.e(-j);
            pols.remInv[p] = pols.rem[p] == 0n ? 0n : F.inv(pols.rem[p]);

            // spare means we are in padding zone
            // check if pols.rem[p] was "negative"
            pols.spare[p] = pols.rem[p] > 0x7FFFFFFF80000000n ? 1n : 0n;
            pols.firstHash[p] = j==0 ? 1n : 0n;
            pols.connected[p] = 0n;
            pols.incCounter[p] = 1n;
            pols.lastHashLen[p] = 0n;
            pols.lastHashDigest[p] = 0n;


            pols.crLen[p] =  F.one;
            pols.crOffset[p] = F.zero;

            pols.crOffsetInv[p] = pols.crOffset[p] == 0n ? 0n : F.inv(pols.crOffset[p]);

            for (let k=0; k<8; k++) {
                pols.crF[k][p] = (k==0) ? 1n : 0n;
                pols.crV[k][p+1] = 0n;
            }

            if (j % BYTESPERBLOCK == (BYTESPERBLOCK -1) ) {
                    required.paddingKKBit.push({
                        r: bytes0,
                        connected: false
                    });

                    [
                        pols.hash0[p],
                        pols.hash1[p],
                        pols.hash2[p],
                        pols.hash3[p],
                        pols.hash4[p],
                        pols.hash5[p],
                        pols.hash6[p],
                        pols.hash7[p]
                    ] =  hash0;
                    for (k=1; k<BYTESPERBLOCK; k++) {
                        pols.hash0[p-k] = pols.hash0[p];
                        pols.hash1[p-k] = pols.hash1[p];
                        pols.hash2[p-k] = pols.hash2[p];
                        pols.hash3[p-k] = pols.hash3[p];
                        pols.hash4[p-k] = pols.hash4[p];
                        pols.hash5[p-k] = pols.hash5[p];
                        pols.hash6[p-k] = pols.hash6[p];
                        pols.hash7[p-k] = pols.hash7[p];
                    }
            }

            p += 1;
        }
        addr += 1n;
    }

    const fp = p;
    while (p<N) {
        pols.freeIn[p] = F.zero;

        pols.len[p] = F.zero;
        pols.addr[p] = addr;
        pols.connected[p] = 0n;
        pols.incCounter[p] = 1n;

        pols.rem[p] = p==fp ? 0n : F.sub(pols.rem[p-1], F.one) ;
        pols.remInv[p] = pols.rem[p] == 0n ? 0n : F.inv(pols.rem[p]);
        pols.spare[p] =  p==fp ? 0n : 1n;
        pols.firstHash[p] = p==fp ? 1n : 0n;
        pols.lastHashLen[p] = 0n;
        pols.lastHashDigest[p] = 0n;

        pols.crLen[p] =  F.one;
        pols.crOffset[p] = F.zero;

        pols.crOffsetInv[p] = pols.crOffset[p] == 0n ? 0n : F.inv(pols.crOffset[p]);

        for (let k=0; k<8; k++) {
            pols.crF[k][p] = (k==0) ? 1n : 0n;
            pols.crV[k][(p+1)%N] = 0n;
        }

        pols.hash0[p] = F.zero;
        pols.hash1[p] = F.zero;
        pols.hash2[p] = F.zero;
        pols.hash3[p] = F.zero;
        pols.hash4[p] = F.zero;
        pols.hash5[p] = F.zero;
        pols.hash6[p] = F.zero;
        pols.hash7[p] = F.zero;

        p += 1;
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
        if (typeof input[i].data === "string") {
            input[i].dataBytes = hexToBytes(input[i].data);
        } else if (Array.isArray(input[i].data) ) {
            input[i].dataBytes = input[i].data.slice();
        }

        input[i].hash = ethers.utils.keccak256(input[i].dataBytes);

        input[i].realLen = BigInt(input[i].dataBytes.length);

        input[i].dataBytes.push(0x1);


        while (input[i].dataBytes.length % BYTESPERBLOCK) input[i].dataBytes.push(0);

        input[i].dataBytes[ input[i].dataBytes.length - 1] |= 0x80;
    }
}
