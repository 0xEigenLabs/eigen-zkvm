const { assert } = require("chai");

const keccakF = require("./keccak.js").keccakF;

const { log2 } = require("@0xpolygonhermez/zkevm-commonjs").utils;

const { F1Field } = require("ffjavascript");
const getKs = require("pilcom").getKs;


const SlotSize = 158418;

module.exports.buildConstants = async function (pols) {


    const F = new F1Field("0xFFFFFFFF00000001");

    const N = pols.r8Id.length;

    const nSlots = 9*Math.floor((N-1) / SlotSize);

    const pow = log2(N);
    assert(1<<pow == N);

    const ks = getKs(F, 2);

    let w = F.one;
    for (let i=0; i<N; i++) {
        pols.ConnSOutBit[i] = w;
        pols.ConnSInBit[i] = F.mul(w, ks[0]);
        pols.ConnNine2OneBit[i] = F.mul(w, ks[1]);
        w = F.mul(w, F.FFT.w[pow]);
    }

    function connect(p1, i1, p2, i2) {
        [p1[i1], p2[i2]] = [p2[i2], p1[i1]];
    }


    let p = 0;
    for (let i=0; i<nSlots; i++) {
        let lasti = i-1;
        if (lasti==-1) lasti = nSlots-1;
        for (let j=0; j<136; j++) {
            for (let k=0; k<8; k++) {
                pols.r8Id[p] = F.e(-1);
                pols.sOutId[p] = F.e(-1);
                pols.latchR8[p] = F.zero;
                pols.Fr8[p] = F.e(1 << k);
                pols.rBitValid[p] = F.one;
                pols.latchSOut[p] = F.zero;
                pols.FSOut0[p] = F.zero;
                pols.FSOut1[p] = F.zero;
                pols.FSOut2[p] = F.zero;
                pols.FSOut3[p] = F.zero;
                pols.FSOut4[p] = F.zero;
                pols.FSOut5[p] = F.zero;
                pols.FSOut6[p] = F.zero;
                pols.FSOut7[p] = F.zero;

                connect(pols.ConnSOutBit, p, pols.ConnNine2OneBit, nine2onebit(lasti, true, j*8+k) );
                connect(pols.ConnSInBit, p, pols.ConnNine2OneBit, nine2onebit(i, false, j*8+k) );

                p += 1;
            }

            pols.r8Id[p] = F.e(i*136+j);
            pols.sOutId[p] = F.e(-1);
            pols.latchR8[p] = F.one;
            pols.Fr8[p] = F.zero;
            pols.rBitValid[p] = F.zero;
            pols.latchSOut[p] = F.zero;
            pols.FSOut0[p] = F.zero;
            pols.FSOut1[p] = F.zero;
            pols.FSOut2[p] = F.zero;
            pols.FSOut3[p] = F.zero;
            pols.FSOut4[p] = F.zero;
            pols.FSOut5[p] = F.zero;
            pols.FSOut6[p] = F.zero;
            pols.FSOut7[p] = F.zero;
            p+=1;
        }

        for (let k=0; k<512; k++) {
            pols.sOutId[p] = F.e(-1);
            pols.r8Id[p] = F.e(-1);
            pols.latchR8[p] = F.zero;
            pols.Fr8[p] = F.zero;
            pols.rBitValid[p] = F.zero;
            pols.latchSOut[p] = F.zero;
            pols.FSOut0[p] = F.zero;
            pols.FSOut1[p] = F.zero;
            pols.FSOut2[p] = F.zero;
            pols.FSOut3[p] = F.zero;
            pols.FSOut4[p] = F.zero;
            pols.FSOut5[p] = F.zero;
            pols.FSOut6[p] = F.zero;
            pols.FSOut7[p] = F.zero;

            connect(pols.ConnSOutBit, p, pols.ConnNine2OneBit, nine2onebit(lasti, true, 1088 +k) );
            connect(pols.ConnSInBit, p, pols.ConnNine2OneBit, nine2onebit(i, false, 1088 +k) )

            p += 1;
        }

        for (let k=0; k<256; k++) {
            pols.sOutId[p] = F.e(-1);
            pols.r8Id[p] = F.e(-1);
            pols.latchR8[p] = F.zero;
            pols.Fr8[p] = F.zero;
            pols.rBitValid[p] = F.zero;
            pols.latchSOut[p] = F.zero;

            const bit = k%8;
            const byte = Math.floor(k/8);
            const chunk = 7 - Math.floor(byte/4);
            const byteInChunk = 3 - byte%4;

            pols.FSOut0[p] = (chunk == 0) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;
            pols.FSOut1[p] = (chunk == 1) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;
            pols.FSOut2[p] = (chunk == 2) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;
            pols.FSOut3[p] = (chunk == 3) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;
            pols.FSOut4[p] = (chunk == 4) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;
            pols.FSOut5[p] = (chunk == 5) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;
            pols.FSOut6[p] = (chunk == 6) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;
            pols.FSOut7[p] = (chunk == 7) ? F.e(1n << BigInt( byteInChunk*8 +bit)) : F.zero;

            connect(pols.ConnSOutBit, p, pols.ConnNine2OneBit, nine2onebit(i, true, k) );

            p += 1;
        }

        pols.sOutId[p] = F.e(i);
        pols.r8Id[p] = F.e(-1);
        pols.latchR8[p] = F.zero;
        pols.Fr8[p] = F.zero;
        pols.rBitValid[p] = F.zero;
        pols.latchSOut[p] = F.one;
        pols.FSOut0[p] = F.zero;
        pols.FSOut1[p] = F.zero;
        pols.FSOut2[p] = F.zero;
        pols.FSOut3[p] = F.zero;
        pols.FSOut4[p] = F.zero;
        pols.FSOut5[p] = F.zero;
        pols.FSOut6[p] = F.zero;
        pols.FSOut7[p] = F.zero;
        p += 1;
    }

    while (p<N) {

        pols.sOutId[p] = F.e(-1);
        pols.r8Id[p] = F.e(-1);
        pols.latchR8[p] = F.zero;
        pols.Fr8[p] = F.zero;
        pols.rBitValid[p] = F.zero;
        pols.latchSOut[p] = F.zero;
        pols.FSOut0[p] = F.zero;
        pols.FSOut1[p] = F.zero;
        pols.FSOut2[p] = F.zero;
        pols.FSOut3[p] = F.zero;
        pols.FSOut4[p] = F.zero;
        pols.FSOut5[p] = F.zero;
        pols.FSOut6[p] = F.zero;
        pols.FSOut7[p] = F.zero;
        p += 1;
    }


    function nine2onebit(slot, out, bit) {
        let o = 1;
        o += Math.floor(slot / 9 ) * SlotSize;
        if (out) o += 1600*9;
        o += bit*9;
        o += slot % 9;
        return o;
    }

}



module.exports.execute = async function (pols, input) {

    const required = {
        Nine2One: []
    }

    const N = pols.r8.length;

    const nSlots = 9*Math.floor((N-1) / SlotSize);

    let curInput =0;
    let p=0;

    let  v;

    pols.sOut = [];
    for (let k=0; k<8; k++) {
        pols.sOut[k] = pols["sOut"+k];
    }

    let curState;


    for (let i = 0; i<nSlots; i++) {
        let connected = true;
        let stateWithR;
        if ((curInput>=input.length) || (input[curInput].connected == false)) {
            connected = false;
            stateWithR = [
                [[0,0],[0,0],[0,0],[0,0],[0,0]],
                [[0,0],[0,0],[0,0],[0,0],[0,0]],
                [[0,0],[0,0],[0,0],[0,0],[0,0]],
                [[0,0],[0,0],[0,0],[0,0],[0,0]],
                [[0,0],[0,0],[0,0],[0,0],[0,0]]
            ];
        } else {
            stateWithR = JSON.parse(JSON.stringify(curState));
        }

        for (let j=0; j<136; j++) {
            const byte =  (curInput < input.length) ? input[curInput].r[j] : 0;
            pols.r8[p] = 0n;
            for (k=0; k<8; k++) {
                const bit = (byte >> k) & 1;
                setStateBit(stateWithR, j*8+k, bit);
                pols.rBit[p] = BigInt(bit);
                pols.r8[p+1] = pols.r8[p] | BigInt((bit << k));
                if ( curState) pols.sOutBit[p] = bitFromState(curState, j*8 + k);
                for (r=0; r<8; r++) pols.sOut[r][p] = 0n;
                pols.connected[p] = connected ? 1n : 0n;

                p += 1;
            }

            pols.rBit[p] = 0n;
            if ( curState) pols.sOutBit[p] = 0n;
            for (k=0; k<8; k++) pols.sOut[k][p] = 0n;
            pols.connected[p] = connected ? 1n : 0n;

            p += 1;
        }

        for (let j=0; j<512; j++) {
            pols.rBit[p] = 0n;
            pols.r8[p] = 0n;
            if ( curState) pols.sOutBit[p] = bitFromState(curState, 136*8 + j);
            for (r=0; r<8; r++) pols.sOut[r][p] = 0n;
            pols.connected[p] = connected ? 1n : 0n;

            p += 1;
        }
        curState = keccakF(stateWithR);
        required.Nine2One.push([stateWithR, curState]);

        for (let k=0; k<8; k++) pols.sOut[k][p] = 0n;
        for (let j=0; j<256; j++) {
            pols.rBit[p] = 0n;
            pols.r8[p] = 0n;
            pols.sOutBit[p] = bitFromState(curState, j);
            pols.connected[p] = connected ? 1n : 0n;

            const bit = j%8;
            const byte = Math.floor(j/8);
            const chunk = 7 - Math.floor(byte/4);
            const byteInChunk = 3 - byte%4;

            for (k=0; k<8; k++) {
                if ( k == chunk) {
                    pols.sOut[k][p+1] = pols.sOut[k][p] | (pols.sOutBit[p] << BigInt( byteInChunk*8 + bit));
                } else {
                    pols.sOut[k][p+1] = pols.sOut[k][p];
                }
            }
            p += 1;
        }

        // 0x52b3f53ff196a28e7d2d01283ef9427070bda64128fb5630b97b6ab17a8ff0a8

        pols.rBit[p] = 0n;
        pols.r8[p] = 0n;
        pols.sOutBit[p] = 0n;
        pols.connected[p] = connected ? 1n : 0n;
        p += 1;

        curInput += 1;
    }

    let pp = 0;
    // Connect the last state with the first
    for (let j=0; j<136; j++) {
        for (k=0; k<8; k++) {
            pols.sOutBit[pp] = bitFromState(curState, j*8 + k);
            pp += 1;
        }
        pols.sOutBit[pp] = 0n;
        pp += 1;
    }

    for (let j=0; j<512; j++) {
        pols.sOutBit[pp] = bitFromState(curState, 136*8 + j);

        pp += 1;
    }

    while (p<N) {
        pols.rBit[p] = 0n;
        pols.r8[p] = 0n;
        pols.sOutBit[p] = 0n;
        for (r=0; r<8; r++) pols.sOut[r][p] = 0n;
        pols.connected[p] = 0n;

        p += 1;
    }

    return required;

}


function bitFromState(st, i) {

    const y = Math.floor(i / 320);
    const x = Math.floor((i % 320) / 64);
    const z = i % 64
    const z1 = Math.floor(z / 32);
    const z2 = z%32;

    return BigInt((st[x][y][z1] >> z2) & 1);

}

function setStateBit(st, i, b) {

    const y = Math.floor(i / 320);
    const x = Math.floor((i % 320) / 64);
    const z = i % 64
    const z1 = Math.floor(z / 32);
    const z2 = z%32;



    st[x][y][z1] ^=  (b << z2);
}
