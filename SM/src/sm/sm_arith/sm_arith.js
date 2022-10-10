// all arith sources and tools on https://github.com/hermeznetwork/sm_arith.git

const arithEq0 = require('./sm_arith_eq0');
const arithEq1 = require('./sm_arith_eq1');
const arithEq2 = require('./sm_arith_eq2');
const arithEq3 = require('./sm_arith_eq3');
const arithEq4 = require('./sm_arith_eq4');

const F1Field = require("ffjavascript").F1Field;

module.exports.buildConstants = async function (pols) {
    const N = pols.CLK[0].length;

    buildClocks(pols, N, 32);
    buildByte2Bits16(pols, N);
    buildRange(pols, N, 'GL_SIGNED_4BITS_C0', -16n, 16n);
    buildRange(pols, N, 'GL_SIGNED_4BITS_C1', -16n, 16n, 33);
    buildRange(pols, N, 'GL_SIGNED_4BITS_C2', -16n, 16n, 33*33);
    buildRange(pols, N, 'GL_SIGNED_18BITS', -(2n**18n), (2n**18n));
}

function buildByte2Bits16(pols, N) {
    const modB1 = (2 ** 16);
    const modB2 = (2 ** 19);
    const modBase = modB1 + modB2
    for (let i = 0; i < N; i++) {
        const value = i % modBase;
        pols.SEL_BYTE2_BIT19[i] = (i < modB1 ? 0n:1n);
        pols.BYTE2_BIT19[i] = BigInt(value);
    }
}

function buildClocks(pols, N, clocksByCycle) {
    for (let i = 0; i < clocksByCycle; i++) {
        for (let j = 0; j < N; ++j) {
            pols.CLK[i][j] = ((j + (clocksByCycle - i)) % clocksByCycle) == 0 ? 1n : 0n;
        }
    }
}

function buildBitsRange(pols, N, name, bits) {
    let moduleBase = (2 ** bits);
    for (let i = 0; i < N; i++) {
        pols[name][i] = BigInt(i % moduleBase);
    }
}

function buildRange(pols, N, name, fromValue, toValue, steps = 1) {
    let value = fromValue;
    let csteps = steps;
    for (let i = 0; i < N; i++) {
        pols[name][i] = value;
        csteps -= 1;
        if (csteps <= 0) {
            csteps = steps;
            if (value === toValue) value = fromValue;
            else value += 1n;
        }
    }
}

module.exports.execute = async function (pols, input) {
    // Get N from definitions
    const N = pols.x1[0].length;

    // Field Elliptic Curve
    let pFec = 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2fn;
    const Fec = new F1Field(pFec);
    const Fr = new F1Field(0xffffffff00000001n);

    // Split the input in little-endian bytes
    prepareInput256bits(input, N);
    let eqCalculates = [arithEq0.calculate, arithEq1.calculate, arithEq2.calculate, arithEq3.calculate, arithEq4.calculate];

    // Initialization
    for (let i = 0; i < N; i++) {
        for (let j = 0; j < 16; j++) {
            pols.x1[j][i] = 0n;
            pols.y1[j][i] = 0n;
            pols.x2[j][i] = 0n;
            pols.y2[j][i] = 0n;
            pols.x3[j][i] = 0n;
            pols.y3[j][i] = 0n;
            pols.q0[j][i] = 0n;
            pols.q1[j][i] = 0n;
            pols.q2[j][i] = 0n;
            pols.s[j][i] = 0n;
            if (j < pols.carryL.length) pols.carryL[j][i] = 0n;
            if (j < pols.carryH.length) pols.carryH[j][i] = 0n;
            if (j < pols.selEq.length) pols.selEq[j][i] = 0n;
        }
    }
    let s, q0, q1, q2;
    for (let i = 0; i < input.length; i++) {
        // TODO: if not have x1, need to componse it
        let x1 = BigInt(input[i]["x1"]);
        let y1 = BigInt(input[i]["y1"]);
        let x2 = BigInt(input[i]["x2"]);
        let y2 = BigInt(input[i]["y2"]);
        let x3 = BigInt(input[i]["x3"]);
        let y3 = BigInt(input[i]["y3"]);

        if (input[i].selEq1) {
            s = Fec.div(Fec.sub(y2, y1), Fec.sub(x2, x1));
            let pq0 = s * x2 - s * x1 - y2 + y1;
            q0 = -(pq0/pFec);
            if ((pq0 + pFec*q0) != 0n) {
                throw new Error(`For input ${i}, with the calculated q0 the residual is not zero (diff point)`);
            }
            q0 += 2n ** 258n;
        }
        else if (input[i].selEq2) {
            s = Fec.div(Fec.mul(3n, Fec.mul(x1, x1)), Fec.add(y1, y1));
            let pq0 = s * 2n * y1 - 3n * x1 * x1;
            q0 = -(pq0/pFec);
            if ((pq0 + pFec*q0) != 0n) {
                throw new Error(`For input ${i}, with the calculated q0 the residual is not zero (same point)`);
            }
            q0 += 2n ** 258n;
        }
        else {
            s = 0n;
            q0 = 0n;
        }

        if (input[i].selEq3) {
            let pq1 = s * s - x1 - x2 - x3;
            q1 = -(pq1/pFec);
            if ((pq1 + pFec*q1) != 0n) {
                throw new Error(`For input ${i}, with the calculated q1 the residual is not zero`);
            }
            q1 += 2n ** 258n;

            let pq2 = s * x1 - s * x3 - y1 - y3;
            q2 = -(pq2/pFec);
            if ((pq2 + pFec*q2) != 0n) {
                throw new Error(`For input ${i}, with the calculated q2 the residual is not zero`);
            }
            q2 += 2n ** 258n;
        }
        else {
            q1 = 0n;
            q2 = 0n;
        }
        input[i]['_s'] = to16bitsRegisters(s);
        input[i]['_q0'] = to16bitsRegisters(q0);
        input[i]['_q1'] = to16bitsRegisters(q1);
        input[i]['_q2'] = to16bitsRegisters(q2);
    }

    for (let i = 0; i < input.length; i++) {
        let offset = i * 32;
        for (let step = 0; step < 32; ++step) {
            for (let j = 0; j < 16; j++) {
                pols.x1[j][offset + step] = BigInt(input[i]["_x1"][j])
                pols.y1[j][offset + step] = BigInt(input[i]["_y1"][j])
                pols.x2[j][offset + step] = BigInt(input[i]["_x2"][j])
                pols.y2[j][offset + step] = BigInt(input[i]["_y2"][j])
                pols.x3[j][offset + step] = BigInt(input[i]["_x3"][j])
                pols.y3[j][offset + step] = BigInt(input[i]["_y3"][j])
                pols.s[j][offset + step]  = BigInt(input[i]["_s"][j])
                pols.q0[j][offset + step] = BigInt(input[i]["_q0"][j])
                pols.q1[j][offset + step] = BigInt(input[i]["_q1"][j])
                pols.q2[j][offset + step] = BigInt(input[i]["_q2"][j])
            }
            pols.selEq[0][offset + step] = BigInt(input[i].selEq0);
            pols.selEq[1][offset + step] = BigInt(input[i].selEq1);
            pols.selEq[2][offset + step] = BigInt(input[i].selEq2);
            pols.selEq[3][offset + step] = BigInt(input[i].selEq3);
        }
        let carry = [0n, 0n, 0n];
        const eqIndexToCarryIndex = [0, 0, 0, 1, 2];
        let eq = [0n, 0n , 0n, 0n, 0n]

        let eqIndexes = [];
        if (pols.selEq[0][offset]) eqIndexes.push(0);
        if (pols.selEq[1][offset]) eqIndexes.push(1);
        if (pols.selEq[2][offset]) eqIndexes.push(2);
        if (pols.selEq[3][offset]) eqIndexes = eqIndexes.concat([3, 4]);

        for (let step = 0; step < 32; ++step) {
            eqIndexes.forEach((eqIndex) => {
                let carryIndex = eqIndexToCarryIndex[eqIndex];
                eq[eqIndex] = eqCalculates[eqIndex](pols, step, offset);
                pols.carryL[carryIndex][offset + step] = Fr.e((carry[carryIndex]) % (2n**18n));
                pols.carryH[carryIndex][offset + step] = Fr.e((carry[carryIndex]) / (2n**18n));
                carry[carryIndex] = (eq[eqIndex] + carry[carryIndex]) / (2n ** 16n);
            });
        }
    }
}

function prepareInput256bits(input, N) {
    for (let i = 0; i < input.length; i++) {
        for (var key of Object.keys(input[i])) {
            input[i][`_${key}`] = to16bitsRegisters(input[i][key]);
        }
    }
}

function to16bitsRegisters(value) {
    if (typeof value !== 'bigint') {
        value = BigInt(value);
    }

    let parts = [];
    for (let part = 0; part < 16; ++part) {
        parts.push(value & (part < 15 ? 0xFFFFn:0xFFFFFn));
        value = value >> 16n;
    }
    return parts;
}
