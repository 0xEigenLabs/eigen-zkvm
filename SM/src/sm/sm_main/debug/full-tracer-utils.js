const { ethers } = require("ethers");
const { toHexStringRlp } = require("@0xpolygonhermez/zkevm-commonjs").processorUtils;
const { scalar2fea, fea2scalar } = require("@0xpolygonhermez/zkevm-commonjs").smtUtils;

/**
 * Compute transaction hash from a transaction RLP enconding and hashing with keccak
 * @param {String} to - hex string
 * @param {Number} value - int number
 * @param {Number} nonce - int number
 * @param {String} gasLimit - hex string
 * @param {String} gasPrice - hex string
 * @param {String} data - hex string of the data
 * @param {String} r - hex string of r signature
 * @param {String} s - hex string of s signature
 * @param {String} v - hex string of v signature with EIP-155 applied (https://github.com/ethereum/EIPs/blob/master/EIPS/eip-155.md)
 * @returns {String} - Hex string with the transaction hash
 */
function getTransactionHash(to, value, nonce, gasLimit, gasPrice, data, r, s, v) {
    const txu = {
        value: toHexStringRlp(value),
        nonce: toHexStringRlp(nonce),
        gasLimit: toHexStringRlp(gasLimit),
        gasPrice: toHexStringRlp(gasPrice),
        data: toHexStringRlp(data),
        to: toHexStringRlp(to)
    }

    const sig = {
        r: toHexStringRlp(r),
        s: toHexStringRlp(s),
        v: toHexStringRlp(v)
    }

    const fields = [txu.nonce, txu.gasPrice, txu.gasLimit, txu.to, txu.value, txu.data, sig.v, sig.r, sig.s];
    const rlp = ethers.utils.RLP.encode(fields);
    const kecc = ethers.utils.keccak256(rlp);
    return {
        tx_hash: kecc,
        rlp_tx: rlp
    }
}

/**
 * Returns the value of a rom label
 * @param {Object} program of the rom
 * @param {String} label name of the label
 * @returns {String} label value or null if not found
 */
function findOffsetLabel(program, label) {
    for (let i = 0; i < program.length; i++) {
        if (program[i].offsetLabel === label) {
            return program[i].offset;
        }
    }
    return null;
}

/**
 * Get a global or context variable
 * @param {Object} ctx current context object
 * @param {Boolean} global true if label is global, false if is ctx label
 * @param {String} varLabel name of the label
 * @returns {Scalar} value of the label
 */
function getVarFromCtx(ctx, global, varLabel) {
    const offsetCtx = global ? 0 : Number(ctx.CTX) * 0x40000;
    const offsetRelative = findOffsetLabel(ctx.rom.program, varLabel);
    const addressMem = offsetCtx + offsetRelative;
    const value = ctx.mem[addressMem];
    const finalValue = typeof value === "undefined" ? 0 : value;
    if (!finalValue) return 0n;
    return fea2scalar(ctx.Fr, finalValue);
}

/**
 * Get the stored calldata in the stack
 * @param {Object} ctx current context object
 * @param {Number} offset to start read from calldata
 * @param {Number} length size of the bytes to read from offset
 * @returns {Scalar} value of the label
 */
function getCalldataFromStack(ctx, offset = 0, length) {
    const addr = 0x20000 + 1024 + Number(ctx.CTX) * 0x40000;
    let value = "0x";
    for (let i = addr + Number(offset); i < 0x30000 + Number(ctx.CTX) * 0x40000; i++) {
        const memVal = ctx.mem[i];
        if (!memVal) break;
        value += ethers.utils.hexlify(fea2scalar(ctx.Fr, memVal)).slice(2);
    }
    if (length) {
        value = value.slice(0, 2 + length * 2);
    }
    return value;
}

/**
 * Get the value of a reg (A, B, C, D, E...)
 * @param {Object} ctx current context object
 * @param {String} reg label string of the reg to retrieve
 * @returns {Scalar} value of the reg
 */
function getRegFromCtx(ctx, reg) {
    return fea2scalar(ctx.Fr, ctx[reg]);
}

/**
 * Get range from memory
 * @param {Object} ctx current context object
 * @param {Number} offset to start read from calldata
 * @param {Number} length size of the bytes to read from offset
 * @returns {Array} string array with 32 bytes hexa values
 */
function getFromMemory(offset, length, ctx) {
    const offsetCtx = Number(ctx.CTX) * 0x40000;
    let addrMem = 0;
    addrMem += offsetCtx;
    addrMem += 0x30000;

    let finalMemory = "";
    
    const init = addrMem + (Number(offset) / 32);
    const end = addrMem + ((Number(offset) + Number(length)) / 32);
    const initCeil = Math.ceil(init);
    const endFloor = Math.floor(end);

    if (init != initCeil) {
        let memValueStart = ctx.mem[Math.floor(init)];
        if (typeof memValueStart === "undefined")
            memValueStart = scalar2fea(ctx.Fr, 0);;
        let memScalarStart = fea2scalar(ctx.Fr, memValueStart);
        let hexStringStart = memScalarStart.toString(16);
        hexStringStart = hexStringStart.padStart(64, "0");
        const bytesToRetrieve = (init - Math.floor(init)) * 32; 
        hexStringStart = hexStringStart.slice(bytesToRetrieve * 2);
        finalMemory = finalMemory.concat(hexStringStart);
    }

    for (let i = initCeil; i < endFloor; i++) {
        let memValue = ctx.mem[i];
        if (typeof memValue === "undefined")
            memValue = scalar2fea(ctx.Fr, 0);;
        let memScalar = fea2scalar(ctx.Fr, memValue);
        let hexString = memScalar.toString(16);
        hexString = hexString.padStart(64, "0");
        finalMemory = finalMemory.concat(hexString);
    }

    if (end != endFloor) {
        memValueEnd = ctx.mem[endFloor];
        if (typeof memValueEnd === "undefined")
            memValueEnd = scalar2fea(ctx.Fr, 0);;
        memScalarEnd = fea2scalar(ctx.Fr, memValueEnd);
        hexStringEnd = memScalarEnd.toString(16);
        hexStringEnd = hexStringEnd.padStart(64, "0");
        const bytesToKeep = (end - endFloor) * 32; 
        hexStringEnd = hexStringEnd.slice(0, bytesToKeep * 2);
        finalMemory = finalMemory.concat(hexStringEnd);
    }
    return finalMemory
}

module.exports = {
    getTransactionHash,
    findOffsetLabel,
    getVarFromCtx,
    getCalldataFromStack,
    getRegFromCtx,
    getFromMemory
}

