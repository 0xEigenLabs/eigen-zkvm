let REGISTERS_NUM = 8;
let BYTES_PER_REGISTER = 4;
let LATCH_SIZE = REGISTERS_NUM * BYTES_PER_REGISTER;

let REG_SIZE = 2 ** 8
let CIN_SIZE = 2 ** 1
let P_LAST_SIZE = 2 ** 1
let OPCODE_SIZE = 2 ** 2

/*
    ==================
    Build Contants
    ==================
    FACTOR0_7, P_A, P_B, P_C, P_CIN, P_COUT, P_OPCODE, RESET
*/
module.exports.buildConstants = async function (pols) {

    const N = pols.RESET.length;
    buildFACTORS(pols.FACTOR, N);
    buildRESET(pols.RESET, N);

    buildP_A(pols.P_A, REG_SIZE, N);
    buildP_B(pols.P_B, REG_SIZE, N);
    buildP_P_CIN(pols.P_CIN, CIN_SIZE, REG_SIZE * REG_SIZE, N);
    buildP_LAST(pols.P_LAST, P_LAST_SIZE, REG_SIZE * REG_SIZE * CIN_SIZE, N);
    buildP_OPCODE(pols.P_OPCODE, REG_SIZE * REG_SIZE * CIN_SIZE * P_LAST_SIZE, N);

    buildP_C_P_COUT_P_USE_CARRY(
        pols.P_A,
        pols.P_B,
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
    FACTOR0 => 0x1  0x100   0x10000 0x01000000  0x0  0x0    0x0     0x0         ... 0x0  0x0    0x0     0x0         0x1 0x100   0x10000 0x01000000  0x0  ...
    FACTOR1 => 0x0  0x0     0x0     0x0         0x1  0x100  0x10000 0x01000000  ... 0x0  0x0    0x0     0x0         0x0 0x0     0x0     0x0         0x0  ...
    ...
    FACTOR7 => 0x0  0x0     0x0     0x0         0x0  0x0     0x0     0x0        ... 0x1  0x100  0x10000 0x01000000  0x0 0x0     0x0     0x0         0x0  ...
*/
function buildFACTORS(FACTORS, N) {
    // The REGISTERS_NUM is equal to the number of factors
    for (let i = 0; i < REGISTERS_NUM; i++) {
        let index = 0;
        for (let j = 0; j < N; j += BYTES_PER_REGISTER) {
            for (let k = 0; k < BYTES_PER_REGISTER; k++) {
                let factor = BigInt((2 ** 8) ** k) * BigInt((j % (REGISTERS_NUM * BYTES_PER_REGISTER)) / BYTES_PER_REGISTER == i);
                FACTORS[i][index++] = factor;
            }
        }
    }
}

/*  =========
    RESET
    =========
    1 0 0 ... { REGISTERS_NUM * BYTES_PER_REGISTER } ... 0 1 0 ... { REGISTERS_NUM * BYTES_PER_REGISTER } 0
    1 0 0 ... { REGISTERS_NUM * BYTES_PER_REGISTER } ... 0 1 0 ... { REGISTERS_NUM * BYTES_PER_REGISTER } 0
    ...
    1 0 0 ... { REGISTERS_NUM * BYTES_PER_REGISTER } ... 0 1 0 ... { REGISTERS_NUM * BYTES_PER_REGISTER } 0
*/
function buildRESET(pol, N) {
    for (let i = 0; i < N; i++) {
        pol[i] = BigInt(i % (REGISTERS_NUM * BYTES_PER_REGISTER) == 0);
    }
}

/*  ============
    A
    =========
    0 .. {size} .. 0 1 .. {size} .. 1 ... {size} ... 15 ... {size} ... 15 (size * size)
    0 .. {size} .. 0 1 .. {size} .. 1 ... {size} ... 15 ... {size} ... 15
    ...
    0 .. {size} .. 0 1 .. {size} .. 1 ... {size} ... 15 ... {size} ... 15
*/
function buildP_A(pol, size, N) {
    let index = 0;
    for (let i = 0; i < N; i += (size * size)) {
        let value = 0;
        for (let j = 0; j < size; j++) {
            for (let k = 0; k < size; k++) {
                pol[index++] = BigInt(value);
            }
            value++;
        }
    }
}

/*  =========
    B
    =========
    0 1 2 .. {size} .. 15 0 1 2 .. {size} .. 15 0 1 2 .. {size} .. 15 0 1 2 ... {size} ... 15 (size * size)
    0 1 2 .. {size} .. 15 0 1 2 .. {size} .. 15 0 1 2 .. {size} .. 15 0 1 2 ... {size} ... 15 (size * size)
    ...
    0 1 2 .. {size} .. 15 0 1 2 .. {size} .. 15 0 1 2 .. {size} .. 15 0 1 2 ... {size} ... 15 (size * size)
 */
function buildP_B(pol, size, N) {
    let index = 0;
    for (let i = 0; i < N; i = i + (size * size)) {
        for (let j = 0; j < size; j++) {
            let value = 0;
            for (let k = 0; k < size; k++) {
                pol[index++] = BigInt(value);
                value++;
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
        * 0 -> COUT (AND doesn't have carry)
    default
        * 0 -> C
        * 0 -> COUT
 */
function buildP_C_P_COUT_P_USE_CARRY(pol_a, pol_b, pol_cin, pol_last, pol_opc, pol_use_carry, pol_c, pol_cout, N) {
    // All opcodes
    let carry = 0;
    for (let i = 0; i < N; i++) {
        switch (pol_opc[i]) {
            // ADD   (OPCODE = 0)
            case 0n:
                let sum = pol_cin[i] + pol_a[i] + pol_b[i];
                pol_c[i] = sum & 255n;
                pol_cout[i] = sum >> 8n;
                pol_use_carry[i] = 0n;
                break;
            // SUB   (OPCODE = 1)
            case 1n:
                if (pol_a[i] - pol_cin[i] >= pol_b[i]) {
                    pol_c[i] = pol_a[i] - pol_cin[i] - pol_b[i];
                    pol_cout[i] = 0n;
                } else {
                    pol_c[i] =  255n - pol_b[i] + pol_a[i] - pol_cin[i] + 1n;
                    pol_cout[i] = 1n;
                }
                pol_use_carry[i] = 0n;
                break;
            // LT    (OPCODE = 2)
            case 2n:
                if (pol_a[i] < pol_b[i]) {
                    pol_cout[i] = 1n;
                    pol_c[i] = pol_last[i] ? 1n : 0n;
                } else if (pol_a[i] == pol_b[i]) {
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
                    if (pol_a[i] < pol_b[i]) {
                        pol_cout[i] = 1n;
                        pol_c[i] = 0n;
                    } else if (pol_a[i] == pol_b[i]) {
                        pol_cout[i] = pol_cin[i];
                        pol_c[i] = 0n;
                    } else {
                        pol_cout[i] = 0n;
                        pol_c[i] = 0n;
                    }
                } else {
                    let sig_a = pol_a[i] >> 7n;
                    let sig_b = pol_b[i] >> 7n;
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
                        if (pol_a[i] < pol_b[i]) {
                            pol_cout[i] = 1n;
                            pol_c[i] = 1n;
                        } else if (pol_a[i] == pol_b[i]) {
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
                if (pol_a[i] == pol_b[i] && pol_cin[i] == 1n) {
                    pol_cout[i] = 1n;
                    pol_c[i] = pol_last[i] ? 1n : 0n;
                } else {
                    pol_cout[i] = 0n;
                    pol_c[i] = 0n
                }
                pol_use_carry[i] = pol_last[i] ? 1n : 0n;

                break;
            // AND   (OPCODE = 5)
            case 5n:
                pol_c[i] = pol_a[i] & pol_b[i];
                pol_cout[i] = 0n;
                pol_use_carry[i] = 0n;
                break;
            // OR    (OPCODE = 6)
            case 6n:
                pol_c[i] = pol_a[i] | pol_b[i];
                pol_cout[i] = 0n;
                pol_use_carry[i] = 0n;
                break;
            // XOR   (OPCODE = 7)
            case 7n:
                pol_c[i] = pol_a[i] ^ pol_b[i];
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
    const N = pols.freeInA.length;

    // Split the input in little-endian bytes
    prepareInput256bits(input, N);

    var c0Temp = new Array();
    c0Temp.push(0n);
    // Initialization
    for (var i = 0; i < N; i++) {
        for (let j = 0; j < REGISTERS_NUM; j++) {
            pols[`a${j}`][i] = 0n;
            pols[`b${j}`][i] = 0n;
            pols[`c${j}`][i] = 0n;
        }
        pols.last[i] = 0n;
        pols.opcode[i] = 0n;
        pols.freeInA[i] = 0n;
        pols.freeInB[i] = 0n;
        pols.freeInC[i] = 0n;
        pols.cIn[i] = 0n;
        pols.cOut[i] = 0n;
        pols.lCout[i] = 0n;
        pols.lOpcode[i] = 0n;
        pols.useCarry[i] = 0n;
    }
    let FACTOR = [[], [], [], [], [], [], [], []];
    let RESET = [];
    buildFACTORS(FACTOR, N);
    buildRESET(RESET, N);

    // Porcess all the inputs
    for (var i = 0; i < input.length; i++) {
        if (i % 10000 === 0) console.log(`Computing binary pols ${i}/${input.length}`);
        for (var j = 0; j < LATCH_SIZE; j++) {
            pols.opcode[i * LATCH_SIZE + j] = BigInt("0x" + input[i].opcode)
            pols.freeInA[i * LATCH_SIZE + j] = BigInt(input[i]["a_bytes"][j])
            pols.freeInB[i * LATCH_SIZE + j] = BigInt(input[i]["b_bytes"][j])
            pols.freeInC[i * LATCH_SIZE + j] = BigInt(input[i]["c_bytes"][j])

            if (j == LATCH_SIZE - 1) {
                pols.last[i * LATCH_SIZE + j] = BigInt(1n)
            } else {
                pols.last[i * LATCH_SIZE + j] = BigInt(0n)
            }

            let cout;
            switch (BigInt("0x" + input[i].opcode)) {
                // ADD   (OPCODE = 0)
                case 0n:
                    let sum = input[i]["a_bytes"][j] + input[i]["b_bytes"][j] + pols.cIn[i * LATCH_SIZE + j]
                    pols.cOut[i * LATCH_SIZE + j] = BigInt(sum >> 8n);
                    break;
                // SUB   (OPCODE = 1)
                case 1n:
                    if (input[i]["a_bytes"][j] - pols.cIn[i * LATCH_SIZE + j] >= input[i]["b_bytes"][j]) {
                        pols.cOut[i * LATCH_SIZE + j] = 0n;
                    } else {
                        pols.cOut[i * LATCH_SIZE + j] = 1n;
                    }
                    break;
                // LT    (OPCODE = 2)
                case 2n:
                    if (RESET[i * LATCH_SIZE + j]) {
                        pols.freeInC[i * LATCH_SIZE + j] = BigInt(input[i]["c_bytes"][LATCH_SIZE - 1]); // Only change the freeInC when reset or Last
                    }
                    if ((input[i]["a_bytes"][j] < input[i]["b_bytes"][j])) {
                        cout = 1n;
                    } else if (input[i]["a_bytes"][j] == input[i]["b_bytes"][j]) {
                        cout = pols.cIn[i * LATCH_SIZE + j];
                    } else {
                        cout = 0n;
                    }
                    pols.cOut[i * LATCH_SIZE + j] = cout;
                    if (pols.last[i * LATCH_SIZE + j] == 1n) {
                        pols.useCarry[i * LATCH_SIZE + j] = 1n
                        pols.freeInC[i * LATCH_SIZE + j] = BigInt(input[i]["c_bytes"][0])
                    } else {
                        pols.useCarry[i * LATCH_SIZE + j] = 0n;
                    }
                    break;
                // SLT    (OPCODE = 3)
                case 3n:
                    pols.last[i * LATCH_SIZE + j] ? pols.useCarry[i * LATCH_SIZE + j] = 1n : pols.useCarry[i * LATCH_SIZE + j] = 0n;
                    if (RESET[i * LATCH_SIZE + j]) {
                        pols.freeInC[i * LATCH_SIZE + j] = BigInt(input[i]["c_bytes"][LATCH_SIZE - 1]);  // Only change the freeInC when reset or Last
                    }
                    if (pols.last[i * LATCH_SIZE + j]) {
                        let sig_a = input[i]["a_bytes"][j] >> 7n;
                        let sig_b = input[i]["b_bytes"][j] >> 7n;
                        // A Negative ; B Positive
                        if (sig_a > sig_b) {
                            cout = 1n;
                            // A Positive ; B Negative
                        } else if (sig_a < sig_b) {
                            cout = 0n;
                            // A and B equals
                        } else {
                            if ((input[i]["a_bytes"][j] < input[i]["b_bytes"][j])) {
                                cout = 1n;
                            } else if (input[i]["a_bytes"][j] == input[i]["b_bytes"][j]) {
                                cout = pols.cIn[i * LATCH_SIZE + j];
                            } else {
                                cout = 0n;
                            }
                        }
                        pols.freeInC[i * LATCH_SIZE + j] = BigInt(input[i]["c_bytes"][0]) // Only change the freeInC when reset or Last
                    } else {
                        if ((input[i]["a_bytes"][j] < input[i]["b_bytes"][j])) {
                            cout = 1n;
                        } else if (input[i]["a_bytes"][j] == input[i]["b_bytes"][j]) {
                            cout = pols.cIn[i * LATCH_SIZE + j];
                        } else {
                            cout = 0n;
                        }
                    }
                    pols.cOut[i * LATCH_SIZE + j] = cout;
                    break;
                // EQ    (OPCODE = 4)
                case 4n:
                    if (RESET[i * LATCH_SIZE + j]) {
                        pols.cIn[i * LATCH_SIZE + j] = 1n
                        pols.freeInC[i * LATCH_SIZE + j] = BigInt(input[i]["c_bytes"][LATCH_SIZE - 1]);
                    }

                    if (input[i]["a_bytes"][j] == input[i]["b_bytes"][j] && pols.cIn[i * LATCH_SIZE + j] == 1) {
                        cout = 1n;
                    } else {
                        cout = 0n;
                    }
                    pols.cOut[i * LATCH_SIZE + j] = cout;

                    if (pols.last[i * LATCH_SIZE + j] == 1n) {
                        pols.useCarry[i * LATCH_SIZE + j] = 1n
                        pols.freeInC[i * LATCH_SIZE + j] = BigInt(input[i]["c_bytes"][0]) // Only change the freeInC when reset or Last
                    } else {
                        pols.useCarry[i * LATCH_SIZE + j] = 0n;
                    }
                    break;
                default:
                    pols.cIn[i * LATCH_SIZE + j] = 0n;
                    pols.cOut[i * LATCH_SIZE + j] = 0n;
                    break;
            }
            // We can set the cIn and the LCin when RESET =1
            if (RESET[(i * LATCH_SIZE + j + 1) % N]) {
                pols.cIn[(i * LATCH_SIZE + j + 1) % N] = 0n;
            } else {
                pols.cIn[(i * LATCH_SIZE + j + 1) % N] = pols.cOut[i * LATCH_SIZE + j]
            }
            pols.lCout[(i * LATCH_SIZE + j + 1) % N] = pols.cOut[i * LATCH_SIZE + j]
            pols.lOpcode[(i * LATCH_SIZE + j + 1) % N] = pols.opcode[i * LATCH_SIZE + j]

            pols[`a0`][(i * LATCH_SIZE + j + 1) % N] = pols[`a0`][(i * LATCH_SIZE + j) % N] * (1n - RESET[(i * LATCH_SIZE + j) % N]) + pols.freeInA[(i * LATCH_SIZE + j) % N] * FACTOR[0][(i * LATCH_SIZE + j) % N];
            pols[`b0`][(i * LATCH_SIZE + j + 1) % N] = pols[`b0`][(i * LATCH_SIZE + j) % N] * (1n - RESET[(i * LATCH_SIZE + j) % N]) + pols.freeInB[(i * LATCH_SIZE + j) % N] * FACTOR[0][(i * LATCH_SIZE + j) % N];

            c0Temp[(i * LATCH_SIZE + j) % N] = pols[`c0`][(i * LATCH_SIZE + j) % N] * (1n - RESET[(i * LATCH_SIZE + j) % N]) + pols.freeInC[(i * LATCH_SIZE + j) % N] * FACTOR[0][(i * LATCH_SIZE + j) % N];
            pols[`c0`][(i * LATCH_SIZE + j + 1) % N] = pols.useCarry[(i * LATCH_SIZE + j) % N] * (pols.cOut[(i * LATCH_SIZE + j) % N] - c0Temp[(i * LATCH_SIZE + j) % N]) + c0Temp[(i * LATCH_SIZE + j) % N];

            if ((i * LATCH_SIZE + j) % 10000 === 0) console.log(`Computing final binary pols ${(i * LATCH_SIZE + j)}/${N}`);

            for (let k = 1; k < REGISTERS_NUM; k++) {
                pols[`a${k}`][(i * LATCH_SIZE + j + 1) % N] = pols[`a${k}`][(i * LATCH_SIZE + j) % N] * (1n - RESET[(i * LATCH_SIZE + j) % N]) + pols.freeInA[(i * LATCH_SIZE + j) % N] * FACTOR[k][(i * LATCH_SIZE + j) % N];
                pols[`b${k}`][(i * LATCH_SIZE + j + 1) % N] = pols[`b${k}`][(i * LATCH_SIZE + j) % N] * (1n - RESET[(i * LATCH_SIZE + j) % N]) + pols.freeInB[(i * LATCH_SIZE + j) % N] * FACTOR[k][(i * LATCH_SIZE + j) % N];
                if (pols.last[i * LATCH_SIZE + j] && pols.useCarry[i * LATCH_SIZE + j]) {
                    pols[`c${k}`][(i * LATCH_SIZE + j + 1) % N] = 0n
                } else {
                    pols[`c${k}`][(i * LATCH_SIZE + j + 1) % N] = pols[`c${k}`][(i * LATCH_SIZE + j) % N] * (1n - RESET[(i * LATCH_SIZE + j) % N]) + pols.freeInC[(i * LATCH_SIZE + j) % N] * FACTOR[k][(i * LATCH_SIZE + j) % N];
                }
            }
        }
    }
    for (var i = input.length * LATCH_SIZE; i < N; i++) {
        if (i % 10000 === 0) console.log(`Computing final binary pols ${i}/${N}`);
        pols[`a0`][(i + 1) % N] = pols[`a0`][i] * (1n - RESET[i]) + pols.freeInA[i] * FACTOR[0][i];
        pols[`b0`][(i + 1) % N] = pols[`b0`][i] * (1n - RESET[i]) + pols.freeInB[i] * FACTOR[0][i];

        c0Temp[i] = pols[`c0`][i] * (1n - RESET[i]) + pols.freeInC[i] * FACTOR[0][i];
        pols[`c0`][(i + 1) % N] = pols.useCarry[i] * (pols.cOut[i] - c0Temp[i]) + c0Temp[i];

        for (let j = 1; j < REGISTERS_NUM; j++) {
            pols[`a${j}`][(i + 1) % N] = pols[`a${j}`][i] * (1n - RESET[i]) + pols.freeInA[i] * FACTOR[j][i];
            pols[`b${j}`][(i + 1) % N] = pols[`b${j}`][i] * (1n - RESET[i]) + pols.freeInB[i] * FACTOR[j][i];
            pols[`c${j}`][(i + 1) % N] = pols[`c${j}`][i] * (1n - RESET[i]) + pols.freeInC[i] * FACTOR[j][i];
        }
    }
}

function prepareInput256bits(input, N) {
    // Porcess all the inputs
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