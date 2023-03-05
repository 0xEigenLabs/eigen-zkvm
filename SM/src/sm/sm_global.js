const { F1Field } = require("ffjavascript");

const F = require("ffjavascript").F1Field;

const CONST_F = {
    // 0, 1, 2 .... 65535 (cyclic)
    BYTE2: (i,N) => BigInt(i & 0xFFFF), // [0..65535]

    // 0, 1, 2, 3, ..., 254, 255, 0, 1, 2, ...
    BYTE: (i,N) => BigInt(i & 0xFF), // [0..255]

    // 0 (x256), 1 (x256), ..., 255 (x256), 0 (x256), ...
    BYTE_2A: (i,N) => BigInt((i & 0xFFFF) >> 8), // [0:256..255:256]

    // 0, 1, 2, ... N-1
    STEP: (i,N) => BigInt(i), // [0..N-1]

    // 0, 1, 2, ..., 30, 31, 0, 1, ...
    STEP32: (i,N) => BigInt(i % 32), // [0..32]

    // 1, 0, .... 0, 0
    L1: (i,N) => (i == 0) ? 1n : 0n,

    // 0, 0, .... 0, 1
    LLAST: (i,N) => (i == (N-1)) ? 1n:0n,

    // [0] = 1, 0:31  (cyclic)
    // [1] = 0, 1, 0:30 (cyclic)
    // [7] = 0:31, 1 (cyclic)
    CLK32: (index, i, N) => (i % 32) == index ? 1n : 0n,

    // [0] = 1,256,256**2,256**3, 0:28 (cyclic)
    // [1] = 0:4, 1,256,256**2,256**3, 0:24 (cyclic)
    // [7] = 0:28, 1,256,256**2,256**3 (cyclic)
    BYTE_FACTOR: (index, i, N) => ((i >> 2) & 0x07) == index ? [1n, 256n, 256n**2n, 256n**3n][i % 4]:0n
}

module.exports.buildConstants = async function (pols) {

    const F = new F1Field("0xFFFFFFFF00000001");

    const N = pols.BYTE.length;

    Object.entries(CONST_F).forEach(([name, func]) => {
        // if (typeof pols[name] === 'undefined') return;
        const isArrayPol = Array.isArray(pols[name][0]);
        if (isArrayPol) {
            const indexCount = pols[name].length;
            for (let index = 0; index < indexCount; ++index) {
                for (i = 0; i < N; ++i) pols[name][index][i] = func(index,i,N);
            }
        }
        else {
            for (i = 0; i < N; ++i) pols[name][i] = func(i,N);
        }
    });
}

