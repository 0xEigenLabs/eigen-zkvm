const REGISTERS_NUM = 8;
const BYTES_PER_REGISTER = 4;
const STEPS_PER_REGISTER = 2;
const STEPS = STEPS_PER_REGISTER * REGISTERS_NUM;

let LATCH_SIZE = REGISTERS_NUM * STEPS_PER_REGISTER;

let REG_SIZE = 2 ** 8
let CIN_SIZE = 2 ** 1
let P_LAST_SIZE = 2 ** 1
let OPCODE_SIZE = 2 ** 2

/*
    ==================
    Build Contants
    ==================
    FACTOR0_7, P_C, P_CIN, P_COUT, P_OPCODE
*/
module.exports.buildConstants = async function (pols) {

    const N = pols.P_C.length;
    buildFACTORS(pols.FACTOR, N);

    buildP_P_CIN(pols.P_CIN, CIN_SIZE, REG_SIZE * REG_SIZE, N);
    buildP_LAST(pols.P_LAST, P_LAST_SIZE, REG_SIZE * REG_SIZE * CIN_SIZE, N);
    buildP_OPCODE(pols.P_OPCODE, REG_SIZE * REG_SIZE * CIN_SIZE * P_LAST_SIZE, N);

    buildP_C_P_COUT_P_USE_CARRY(
        pols.P_CIN,
        pols.P_LAST,
        pols.P_OPCODE,
        pols.P_USE_CARRY,
        pols.P_C,
        pols.P_COUT,
        N);
}

/*  =========
    FACTORS
    =========
    FACTOR0 => [0x1,0x10000,0:14]  (cyclic)
    FACTOR1 => [0:2,0x1,0x10000,0:12] (cyclic)
    ...
    FACTOR7 => [0:14,0x1,0x10000] (cyclic)
*/
function buildFACTORS(FACTORS, N) {
    for (let index = 0; index < N; ++index) {
        const k = Math.floor(index / STEPS_PER_REGISTER) % REGISTERS_NUM;
        for (let j = 0; j < REGISTERS_NUM; ++j) {
            if (j == k) {
                FACTORS[j][index] = ((index % 2) == 0) ? 1n : 2n ** 16n;
             } else {
                FACTORS[j][index] = 0n;
             }
        }
    }
}

/*
    =========
    CIN
    =========
    0 0 0 ... {AccumulatedSize} ... 0 0 0 1 1 1 ... {AccumulatedSize} ... 1 1 1
    0 0 0 ... {AccumulatedSize} ... 0 0 0 1 1 1 ... {AccumulatedSize} ... 1 1 1
    ...
    0 0 0 ... {AccumulatedSize} ... 0 0 0 1 1 1 ... {AccumulatedSize} ... 1 1 1
 */
function buildP_P_CIN(pol, pol_size, accumulated_size, N) {
    let index = 0;
    for (let i = 0; i < N; i += (accumulated_size * pol_size)) {
        let value = 0;
        for (let j = 0; j < pol_size; j++) {
            for (let k = 0; k < accumulated_size; k++) {
                pol[index++] = BigInt(value);
            }
            value++;
        }
    }
}

/*
    =========
    OPCODE
    =========
    0 0 0 ... {current_size} ... 0 0 0
    1 1 1 ... {current_size} ... 1 1 1
    2 2 2 ... {current_size} ... 2 2 2
    ...
 */
function buildP_OPCODE(pol, current_size, N) {
    let index = 0;
    let value = 0;
    for (let i = 0; i < N; i = i + current_size) {
        for (let j = 0; j < current_size; j++) {
            pol[index++] = BigInt(value);
        }
        value++;
    }
}

function buildP_LAST(pol, pol_size, accumulated_size, N) {
    let index = 0;
    for (let i = 0; i < N; i += (accumulated_size * pol_size)) {
        let value = 0;
        for (let j = 0; j < pol_size; j++) {
            for (let k = 0; k < accumulated_size; k++) {
                pol[index++] = BigInt(value);
            }
            value++;
        }
    }
}

/*
    =========
    C & COUT
    =========
    1 => ADD
        * Extract less signative byte -> C
        * Get the carry out -> COUT
    0 => AND
        * A & B -> C
        * [C != 0] => COUT
    default
        * 0 -> C
        * 0 -> COUT
 */
function buildP_C_P_COUT_P_USE_CARRY(pol_cin, pol_last, pol_opc, pol_use_carry, pol_c, pol_cout, N) {
    // All opcodes
    let carry = 0;
    for (let i = 0; i < N; i++) {
        const pol_a = BigInt((i >> 8) & 0xFF);
        const pol_b = BigInt(i & 0xFF);
        switch (pol_opc[i]) {
            // ADD   (OPCODE = 0)
            case 0n:
                let sum = pol_cin[i] + pol_a + pol_b;
                pol_c[i] = sum & 255n;
                pol_cout[i] = sum >> 8n;
                pol_use_carry[i] = 0n;
                break;
            // SUB   (OPCODE = 1)
            case 1n:
                if (pol_a - pol_cin[i] >= pol_b) {
                    pol_c[i] = pol_a - pol_cin[i] - pol_b;
                    pol_cout[i] = 0n;
                } else {
                    pol_c[i] =  255n - pol_b + pol_a - pol_cin[i] + 1n;
                    pol_cout[i] = 1n;
                }
                pol_use_carry[i] = 0n;
                break;
            // LT    (OPCODE = 2)
            case 2n:
                if (pol_a < pol_b) {
                    pol_cout[i] = 1n;
                    pol_c[i] = pol_last[i] ? 1n : 0n;
                } else if (pol_a == pol_b) {
                    pol_cout[i] = pol_cin[i];
                    pol_c[i] = pol_last[i] ? pol_cin[i] : 0n;
                } else {
                    pol_cout[i] = 0n;
                    pol_c[i] = 0n;
                }
                pol_use_carry[i] = pol_last[i] ? 1n : 0n;
                break;
            // SLT   (OPCODE = 3)
            case 3n:
                if (!pol_last[i]) {
                    if (pol_a < pol_b) {
                        pol_cout[i] = 1n;
                        pol_c[i] = 0n;
                    } else if (pol_a == pol_b) {
                        pol_cout[i] = pol_cin[i];
                        pol_c[i] = 0n;
                    } else {
                        pol_cout[i] = 0n;
                        pol_c[i] = 0n;
                    }
                } else {
                    let sig_a = pol_a >> 7n;
                    let sig_b = pol_b >> 7n;
                    // A Negative ; B Positive
                    if (sig_a > sig_b) {
                        pol_cout[i] = 1n;
                        pol_c[i] = 1n;
                        // A Positive ; B Negative
                    } else if (sig_a < sig_b) {
                        pol_cout[i] = 0n;
                        pol_c[i] = 0n;
                        // A and B equals
                    } else {
                        if (pol_a < pol_b) {
                            pol_cout[i] = 1n;
                            pol_c[i] = 1n;
                        } else if (pol_a == pol_b) {
                            pol_cout[i] = pol_cin[i];
                            pol_c[i] = pol_cin[i];
                        } else {
                            pol_cout[i] = 0n;
                            pol_c[i] = 0n;
                        }
                    }
                }
                pol_use_carry[i] = pol_last[i] ? 1n : 0n;
                break;
            // EQ    (OPCODE = 4)
            case 4n:
                if (pol_a == pol_b && pol_cin[i] == 0n) {
                    pol_cout[i] = 0n;
                    pol_c[i] = pol_last[i] ? 1n : 0n;
                } else {
                    pol_cout[i] = 1n;
                    pol_c[i] = 0n
                }
                if (pol_last[i]) pol_cout[i] = (1n - pol_cout[i]);
                pol_use_carry[i] = pol_last[i] ? 1n : 0n;

                break;
            // AND   (OPCODE = 5)
            case 5n:
                pol_c[i] = pol_a & pol_b;
                if (pol_cin[i] == 0n && pol_c[i] == 0n) {
                    pol_cout[i] = 0n;
                } else {
                    pol_cout[i] = 1n;
                }
                pol_use_carry[i] = 0n;
                break;
            // OR    (OPCODE = 6)
            case 6n:
                pol_c[i] = pol_a | pol_b;
                pol_cout[i] = 0n;
                pol_use_carry[i] = 0n;
                break;
            // XOR   (OPCODE = 7)
            case 7n:
                pol_c[i] = pol_a ^ pol_b;
                pol_cout[i] = 0n;
                pol_use_carry[i] = 0n;
                break;
            // NOP   (OPCODE = 0)
            default:
                pol_c[i] = 0n;
                pol_cout[i] = 0n;
                pol_use_carry[i] = 0n;
        }
    }
}


module.exports.execute = async function (pols, input) {
    // Get N from definitions
    const N = pols.freeInA[0].length;

    // Split the input in little-endian bytes
    prepareInput256bits(input, N);

    var c0Temp = new Array();
    c0Temp.push(0n);
    // Initialization
    for (var i = 0; i < N; i++) {
        for (let j = 0; j < REGISTERS_NUM; j++) {
            pols.a[j][i] = 0n;
            pols.b[j][i] = 0n;
            pols.c[j][i] = 0n;
        }
        pols.opcode[i] = 0n;
        pols.freeInA[0][i] = 0n;
        pols.freeInA[1][i] = 0n;
        pols.freeInB[0][i] = 0n;
        pols.freeInB[1][i] = 0n;
        pols.freeInC[0][i] = 0n;
        pols.freeInC[1][i] = 0n;
        pols.cIn[i] = 0n;
        pols.cOut[i] = 0n;
        pols.cMiddle[i] = 0n;
        pols.lCout[i] = 0n;
        pols.lOpcode[i] = 0n;
        pols.useCarry[i] = 0n;
        pols.resultBinOp[i] = 0n;
        pols.resultValidRange[i] = 0n;
    }
    let FACTOR = [[], [], [], [], [], [], [], []];
    buildFACTORS(FACTOR, N);

    // Process all the inputs
    for (var i = 0; i < input.length; i++) {
        if (i % 10000 === 0) console.log(`Computing binary pols ${i}/${input.length}`);
        for (var j = 0; j < STEPS; j++) {
            const last = (j == (STEPS - 1)) ? 1n : 0n;
            const index = i * STEPS + j;
            pols.opcode[index] = BigInt(input[i].opcode);

            let cIn = 0n;
            let cOut = 0n;
            const reset = (j == 0) ? 1n:0n;
            let useCarry = 0n;
            for (let k = 0; k < 2; ++k) {
                cIn = (k == 0) ? pols.cIn[index] : cOut;
                const byteA = BigInt(input[i]["a_bytes"][j * 2 + k]);
                const byteB = BigInt(input[i]["b_bytes"][j * 2 + k]);
                const byteC = BigInt(input[i]["c_bytes"][j * 2 + k]);
                const resetByte = reset && k == 0;
                const lastByte = last && k == 1;
                pols.freeInA[k][index] = byteA;
                pols.freeInB[k][index] = byteB;
                pols.freeInC[k][index] = byteC;

                // carry management

                switch (BigInt(input[i].opcode)) {

                    // ADD   (OPCODE = 0)
                    case 0n:
                        let sum = byteA + byteB + cIn;
                        cOut = BigInt(sum >> 8n);
                        break;

                    // SUB   (OPCODE = 1)
                    case 1n:
                        if (byteA - cIn >= byteB) {
                            cOut = 0n;
                        } else {
                            cOut = 1n;
                        }
                        break;

                    // LT    (OPCODE = 2)
                    case 2n:
                        if (resetByte) {
                            pols.freeInC[0][index] = BigInt(input[i]["c_bytes"][STEPS-1]); // Only change the freeInC when reset or Last
                        }
                        if ((byteA < byteB)) {
                            cOut = 1n;
                        } else if (byteA == byteB) {
                            cOut = cIn;
                        } else {
                            cOut = 0n;
                        }
                        if (lastByte) {
                            useCarry = 1n;
                            pols.freeInC[1][index] = BigInt(input[i]["c_bytes"][0]);
                        }
                        break;

                    // SLT    (OPCODE = 3)
                    case 3n:
                        useCarry = last ? 1n : 0n;
                        if (resetByte) {
                            pols.freeInC[0][index] = BigInt(input[i]["c_bytes"][STEPS-1]);  // Only change the freeInC when reset or Last
                        }
                        if (lastByte) {
                            let sig_a = byteA >> 7n;
                            let sig_b = byteB >> 7n;
                            // A Negative ; B Positive
                            if (sig_a > sig_b) {
                                cOut = 1n;
                                // A Positive ; B Negative
                            } else if (sig_a < sig_b) {
                                cOut = 0n;
                                // A and B equals
                            } else {
                                if ((byteA < byteB)) {
                                    cOut = 1n;
                                } else if (byteA == byteB) {
                                    cOut = cIn;
                                } else {
                                    cOut = 0n;
                                }
                            }
                            pols.freeInC[k][index] = BigInt(input[i]["c_bytes"][0]); // Only change the freeInC when reset or Last
                        } else {
                            if ((byteA < byteB)) {
                                cOut = 1n;
                            } else if (byteA == byteB) {
                                cOut = cIn;
                            } else {
                                cOut = 0n;
                            }
                        }
                        break;

                    // EQ    (OPCODE = 4)
                    case 4n:
                        if (resetByte) {
                            // cIn = 1n
                            // pols.cIn[index] = 1n;
                            pols.freeInC[k][index] = BigInt(input[i]["c_bytes"][STEPS-1]);
                        }

                        if (byteA == byteB && cIn == 0n) {
                            cOut = 0n;
                        } else {
                            cOut = 1n;
                        }

                        if (lastByte) {
                            useCarry = 1n;
                            cOut = cOut ? 0n:1n;
                            pols.freeInC[k][index] = BigInt(input[i]["c_bytes"][0]); // Only change the freeInC when reset or Last
                        }
                        break;
                    // AND    (OPCODE = 5)
                    case 5n:
                        // setting carry if result of AND was non zero
                        if (byteC == 0n && cIn == 0n) {
                            cOut = 0n;
                        } else {
                            cOut = 1n;
                        }

                        break;
                    default:
                        cIn = 0n;
                        cOut = 0n;
                        break;
                }

                // setting carries
                if (k == 0) {
                    pols.cMiddle[index] = cOut;
                } else {
                    pols.cOut[index] = cOut;
                }
            }
            pols.useCarry[index] = useCarry;

            const nextIndex = (index + 1) % N;
            const nextReset = (nextIndex % STEPS) == 0 ? 1n:0n;
            // We can set the cIn and the LCin when RESET =1
            if (nextReset) {
                pols.cIn[nextIndex] = 0n;
            } else {
                pols.cIn[nextIndex] = pols.cOut[index];
            }
            pols.lCout[nextIndex] = pols.cOut[index]
            pols.lOpcode[nextIndex] = pols.opcode[index]

            pols.a[0][nextIndex] = pols.a[0][index] * (1n - reset) + pols.freeInA[0][index] * FACTOR[0][index] + 256n * pols.freeInA[1][index] * FACTOR[0][index];
            pols.b[0][nextIndex] = pols.b[0][index] * (1n - reset) + pols.freeInB[0][index] * FACTOR[0][index] + 256n * pols.freeInB[1][index] * FACTOR[0][index];

            c0Temp[index] = pols.c[0][index] * (1n - reset) + pols.freeInC[0][index] * FACTOR[0][index] + 256n * pols.freeInC[1][index] * FACTOR[0][index];
            pols.c[0][nextIndex] = pols.useCarry[index] ? pols.cOut[index] : c0Temp[index];

            if (index % 10000 === 0) console.log(`Computing final binary pols ${index}/${N}`);

            for (let k = 1; k < REGISTERS_NUM; k++) {
                pols.a[k][nextIndex] = pols.a[k][index] * (1n - reset) + pols.freeInA[0][index] * FACTOR[k][index] + 256n * pols.freeInA[1][index] * FACTOR[k][index];
                pols.b[k][nextIndex] = pols.b[k][index] * (1n - reset) + pols.freeInB[0][index] * FACTOR[k][index] + 256n * pols.freeInB[1][index] * FACTOR[k][index];
                if (last && useCarry) {
                    pols.c[k][nextIndex] = 0n;
                } else {
                    pols.c[k][nextIndex] = pols.c[k][index] * (1n - reset) + pols.freeInC[0][index] * FACTOR[k][index] + 256n * pols.freeInC[1][index] * FACTOR[k][index];
                }
            }
        }
        if (input[i].type == 1) {
            pols.resultBinOp[((i+1) * STEPS)%N] = 1n;
        }
        if (input[i].type == 2) {
            pols.resultValidRange [((i+1) * STEPS)%N] = 1n;
        }
    }
    for (let index = input.length * STEPS; index < N; index++) {
        if (index % 10000 === 0) console.log(`Computing final binary pols ${index}/${N}`);
        const nextIndex = (index + 1) % N;
        const reset = (index % STEPS) == 0 ? 1n : 0n;
        pols.a[0][nextIndex] = pols.a[0][index] * (1n - reset) + pols.freeInA[0][index] * FACTOR[0][index] + 256n * pols.freeInA[1][index] * FACTOR[0][index];
        pols.b[0][nextIndex] = pols.b[0][index] * (1n - reset) + pols.freeInB[0][index] * FACTOR[0][index] + 256n * pols.freeInB[1][index] * FACTOR[0][index];

        c0Temp[index] = pols.c[0][index] * (1n - reset) + pols.freeInC[0][index] * FACTOR[0][index] + 256n * pols.freeInC[1][index] * FACTOR[0][index];
        pols.c[0][nextIndex] = pols.useCarry[index] * (pols.cOut[index] - c0Temp[index]) + c0Temp[index];

        for (let j = 1; j < REGISTERS_NUM; j++) {
            pols.a[j][nextIndex] = pols.a[j][index] * (1n - reset) + pols.freeInA[0][index] * FACTOR[j][index] + 256n * pols.freeInA[1][index] * FACTOR[j][index];
            pols.b[j][nextIndex] = pols.b[j][index] * (1n - reset) + pols.freeInB[0][index] * FACTOR[j][index] + 256n * pols.freeInB[1][index] * FACTOR[j][index];
            pols.c[j][nextIndex] = pols.c[j][index] * (1n - reset) + pols.freeInC[0][index] * FACTOR[j][index] + 256n * pols.freeInC[1][index] * FACTOR[j][index];
        }
    }
}

function prepareInput256bits(input, N) {
    // Process all the inputs
    for (let i = 0; i < input.length; i++) {
        // Get all the keys and split them with padding
        for (var key of Object.keys(input[i])) {
            input[i][`${key}_bytes`] = hexToBytes(input[i][key].toString(16).padStart(64, "0"));
        }
    }
    function hexToBytes(hex) {
        for (var bytes = [], c = 64 - 2; c >= 0; c -= 2)
            bytes.push(BigInt(parseInt(hex.substr(c, 2), 16) || 0n));
        return bytes;
    }
}