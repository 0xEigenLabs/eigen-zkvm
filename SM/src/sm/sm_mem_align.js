// function to map a value to array, if value == fromValue return first position of array mappings, if not return elseValue

mapRange = (value, fromValue, mappings, elseValue = 0) =>
          (value >= fromValue && value <= (fromValue + mappings.length - 1)) ? mappings[value - fromValue] : elseValue;

const CONST_F = {
    // 0 (x256), 1 (x256), ..., 255 (x256), 0 (x256), ...
    BYTE2A: (i) => (i & 0xFFFF) >> 8,

    // 0, 1, 2, 3, ..., 254, 255, 0, 1, 2, ...
    BYTE2B: (i) => (i & 0xFF),

    // 0 (x3072), 1 (x3072), ..., 255 (x3072), 0 (x3072), ...
    BYTE_C3072: (i) => (i / 3072) | 0,

    //    0 - 1023  WR256 = 0 WR8 = 0
    // 1024 - 2047  WR256 = 1 WR8 = 0
    // 2048 - 3071  WR256 = 0 WR8 = 1
    WR256: (i) => ((i % 3072) >= 1024 && (i % 3072) < 2048) ? true: false,
    WR8: (i) => (i % 3072) >= 2048 ?  true: false,

    // 0, 1, 2, ..., 30, 31, 0, 1, ...
    STEP: (i) => i % 32,

    // 1, 0 (x31), 1, 0 (x31), 1, ...
    RESET: (i) => (i % 32) == 0 ? true : false,

    // 0 (x32), 1 (x32), ...., 31 (x32), 0 (x32), ...
    OFFSET: (i) => (i >> 5) % 32,

    // For internal use
    V_BYTE: (i) => (31 + (CONST_F.OFFSET(i) + CONST_F.WR8(i)) - CONST_F.STEP(i)) % 32,

    SELM1: (i) => (CONST_F.WR8(i) ? (CONST_F.STEP(i) == CONST_F.OFFSET(i)) :
                                    (CONST_F.OFFSET(i) > CONST_F.STEP(i))) ? 1:0,

    FACTOR: (index, i) => mapRange(i % 32, 28 - 4 * index, [0x1000000, 0x10000, 0x100, 1], 0),
    FACTORV: (index, i) => (CONST_F.V_BYTE(i) >> 2) == index ? [1, 0x100, 0x10000, 0x1000000][CONST_F.V_BYTE(i) % 4] : 0,
}

module.exports.buildConstants = async function (pols) {
    const N = pols.STEP.length;
    Object.entries(CONST_F).forEach(([name, func]) => {
        if (typeof pols[name] === 'undefined') return;

        if (func.length == 1) {
            for (i = 0; i < N; ++i) pols[name][i] = BigInt(func(i));
        }
        else {
            const indexCount = name.startsWith('SEL') ? 2 : 8;
            for (let index = 0; index < indexCount; ++index) {
                for (i = 0; i < N; ++i) pols[name][index][i] = BigInt(func(index,i));
            }
        }
    });
}


module.exports.execute = async function (pols, input) {
    // Get N from definitions
    const N = pols.offset.length;

    // Initialization
    for (let i = 0; i < N; i++) {
        for (let j = 0; j < 8; j++) {
            pols.m0[j][i] = 0n;
            pols.m1[j][i] = 0n;
            pols.w0[j][i] = 0n;
            pols.w1[j][i] = 0n;
            pols.v[j][i] = 0n;
            pols.factorV[j][i] = 0n;
        }
        pols.inV[i]= 0n;
        pols.inM[0][i]= 0n;
        pols.inM[1][i]= 0n;
        pols.wr8[i]= 0n;
        pols.wr256[i]= 0n;
        pols.offset[i]= 0n;
        pols.selM1[i]= 0n;
    }
    const factors = [ 1, 2 ** 8, 2 ** 16, 2 ** 24];
    for (let i = 0; i < input.length; i++) {
        let m0v = BigInt(input[i]["m0"]);
        let m1v = BigInt(input[i]["m1"]);
        const _v = BigInt(input[i]["v"]);
        const offset = Number(input[i]["offset"]);
        const reverseOffset = 32 - offset;
        const wr8 = Number(input[i]["wr8"]);
        const wr256 = Number(input[i]["wr256"]);
        const polIndex = i * 32;
        let vv = _v;
        for (let j = 0; j < 32; ++j) {
            const _vByte = ((31 + (offset + wr8) - j) % 32);
            const _inM0 = getByte(m0v, 31-j);
            const _inM1 = getByte(m1v, 31-j);
            const _inV = getByte(vv, _vByte);
            const _selM1 = (wr8 ? (j == offset) :(offset > j)) ? 1:0;

            pols.wr8[polIndex + j + 1] = BigInt(wr8);
            pols.wr256[polIndex + j + 1] = BigInt(wr256);
            pols.offset[polIndex + j + 1] = BigInt(offset);
            pols.inM[0][polIndex + j] = BigInt(_inM0);
            pols.inM[1][polIndex + j] = BigInt(_inM1);
            pols.inV[polIndex + j] = BigInt(_inV);
            pols.selM1[polIndex + j] = BigInt(_selM1);
            pols.factorV[_vByte >> 2][polIndex + j] = BigInt(factors[(_vByte % 4)]);

            const mIndex = 7 - (j >> 2);

            const _inW0 = ((wr256 * (1 - _selM1)) || (wr8 * _selM1))? _inV : ((wr256 + wr8) * _inM0);
            const _inW1 = (wr256 * _selM1) ? _inV : ((wr256 + wr8) * _inM1);

            const factor = BigInt(factors[3 - (j % 4)]);

            pols.m0[mIndex][polIndex + 1 + j] = (( j === 0 ) ? 0n : pols.m0[mIndex][polIndex + j]) + BigInt(_inM0) * factor;
            pols.m1[mIndex][polIndex + 1 + j] = (( j === 0 ) ? 0n : pols.m1[mIndex][polIndex + j]) + BigInt(_inM1) * factor;

            pols.w0[mIndex][polIndex + 1 + j] = (( j === 0 ) ? 0n : pols.w0[mIndex][polIndex + j]) + BigInt(_inW0) * factor;
            pols.w1[mIndex][polIndex + 1 + j] = (( j === 0 ) ? 0n : pols.w1[mIndex][polIndex + j]) + BigInt(_inW1) * factor;
        }
        for (let j = 0; j < 32; ++j) {
            for (let index = 0; index < 8; ++index) {
                pols.v[index][polIndex + 1 + j] = (( j === 0 ) ? 0n : pols.v[index][polIndex + j]) + pols.inV[polIndex + j] * pols.factorV[index][polIndex + j];
            }
        }

        for (let index = 0; index < 8; ++index) {
            for (j = 32 - (index  * 4); j < 32; ++j) {
                pols.m0[index][polIndex + j + 1] = pols.m0[index][polIndex + j];
                pols.m1[index][polIndex + j + 1] = pols.m1[index][polIndex + j];
                pols.w0[index][polIndex + j + 1] = pols.w0[index][polIndex + j];
                pols.w1[index][polIndex + j + 1] = pols.w1[index][polIndex + j];
            }
        }
    }
    for (let i = (input.length * 32); i < N; i++) {
        for (let index = 0; index < 8; ++index) {
            pols.factorV[index][i] = BigInt(CONST_F.FACTORV(index, i % 32));
        }
    }
}

function getByte (value, index) {
    return Number((value >> (8n * BigInt(index))) & 0xFFn);
}