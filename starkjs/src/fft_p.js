// copied and modified from pil-stark
const { assert } = require("chai");
const GL3 = require("./f3g.js");
const {log2} = require("./utils.js");
const {fft_block} = require("./fft_worker");
const workerpool = require("workerpool");
const {BigBuffer} = require("pilcom");

const F = new GL3();
const useThreads = true;

function BR(x, domainPow)
{
    x = (x >>> 16) | (x << 16);
    x = ((x & 0xFF00FF00) >>> 8) | ((x & 0x00FF00FF) << 8);
    x = ((x & 0xF0F0F0F0) >>> 4) | ((x & 0x0F0F0F0F) << 4);
    x = ((x & 0xCCCCCCCC) >>> 2) | ((x & 0x33333333) << 2);
    return (((x & 0xAAAAAAAA) >>> 1) | ((x & 0x55555555) << 1)) >>> (32 - domainPow);
}
/*
function bitReverse(buff, bits) {
    for (let i=0; i<buff.length; i++) {
        const ir = BR(i, bits);
        if (i<ir) {
            [buff[i], buff[ir]] = [buff[ir], buff[i]];
        }
    }
}
*/

function traspose(buffDst, buffSrc, nPols, nBits, trasposeBits) {
    const n = 1 << nBits;
    const w = 1 << trasposeBits;
    const h = n/w;
    for (let i=0; i<w; i++) {
        for (let j=0; j<h; j++) {
            const fi = j*w + i;
            const di = i*h +j;
            const src = buffSrc.slice(fi*nPols, fi*nPols + nPols);
            buffDst.set(src, di*nPols);
        }
    }
}


async function bitReverse(buffDst, buffSrc, nPols, nBits) {
    const n = 1 << nBits;
    for (let i=0; i<n; i++) {
        const ri = BR(i, nBits);
        const src = buffSrc.slice(  ri*nPols, ri*nPols + nPols);
        buffDst.set(src, i*nPols);
    }
}

async function interpolateBitReverse(buffDst, buffSrc, nPols, nBits) {
    const n = 1 << nBits;
    for (let i=0; i<n; i++) {
        const ri = BR(i, nBits);
        const rii = (n-ri)%n;
        const src = buffSrc.slice(  rii*nPols, rii*nPols + nPols);
        buffDst.set(src, i*nPols);
    }
}

async function invBitReverse(buffDst, buffSrc, nPols, nBits) {
    const n = 1 << nBits;
    const nInv = F.inv(BigInt(n));
    for (let i=0; i<n; i++) {
        const ri = BR(i, nBits);
        const rii = (n-ri)%n;
        for (let p=0; p<nPols; p++) {
            buffDst.setElement(i*nPols+p, F.mul(buffSrc.getElement(rii*nPols+p), nInv));
        }
    }
}


const maxNperThread = 1 << 18;
async function interpolatePrepare(pool, buff, nPols, nBits, nBitsExt ) {

    const n = 1 << nBits;
    const invN = F.inv(BigInt(n));
    const promisesLH = [];
    let res = [];


    const maxNPerThread = 1 << 18;
    const minNPerThread = 1 << 12;


    let nPerThreadF = Math.floor((n-1)/pool.maxWorkers)+1;

    const maxCorrected = Math.floor(maxNPerThread / nPols);
    const minCorrected = Math.floor(minNPerThread / nPols);

    if (nPerThreadF>maxCorrected) nPerThreadF = maxCorrected;
    if (nPerThreadF<minCorrected) nPerThreadF = minCorrected;
    for (let i=0; i< n; i+=nPerThreadF) {
        const curN = Math.min(nPerThreadF, n-i);
        const bb = buff.slice(i*nPols, (i+curN)*nPols);
        const start = F.mul(invN, F.exp(F.shift, i));
        const inc = F.shift;
        if (useThreads) {
            promisesLH.push(pool.exec("interpolatePrepareBlock", [bb, nPols, start, inc, i/nPerThreadF, Math.floor(n/nPerThreadF)]));
        } else {
            res.push(await interpolatePrepareBlock(bb, nPols, start, inc, i/nPerThreadF, Math.floor(n/nPerThreadF)));
        }
    }
    if (useThreads) {
        res = await Promise.all(promisesLH)
    }
    for (let i=0; i<res.length; i++) {
        buff.set(res[i], i*nPerThreadF*nPols);
    }
}


/*
async function interpolatePrepare(buff, nPols, nBits, nBitsExt ) {
    const n = 1 << nBits;
    const nExt = 1 << nBitsExt;
    let w = F.inv(BigInt(n));
    for (let i=0; i<n; i++) {
        for (let p=0; p<nPols; p++) {
            buff[i*nPols+p] = F.mul(buff[i*nPols+p], w);
        }
        w = F.mul(w, F.shift);
    }
    const buffz = new BigUint64Array(buff.buffer, buff.byteOffset + n*nPols*8, (nExt-n)*nPols);
    buffz.fill(0n);
}
*/

// Adjustable parametees
const maxBlockBits = 16;
const minBlockBits = 12;
//const maxBlockBits = 2;
//const minBlockBits = 2;
const blocksPerThread = 8;
async function _fft(buffSrc, nPols, nBits, buffDst, inverse) {
    const n = 1 << nBits;
    const tmpBuff = new BigBuffer(n*nPols);
    const outBuff = buffDst;

    let bIn, bOut;

    const pool = workerpool.pool(__dirname + '/fft_worker.js');

    const idealNBlocks = pool.maxWorkers*blocksPerThread;
    let blockBits = log2(n*nPols/idealNBlocks);
    if (blockBits < minBlockBits) blockBits = minBlockBits;
    if (blockBits > maxBlockBits) blockBits = maxBlockBits;
    blockBits = Math.min(nBits, blockBits);
    const blockSize = 1 << blockBits;
    const nBlocks = n / blockSize;

    let nTrasposes;
    if (nBits == blockBits) {
        nTrasposes = 0;
    } else {
        nTrasposes = Math.floor((nBits-1) / blockBits)+1;
    }

    if (nTrasposes & 1) {
        bOut = tmpBuff;
        bIn = outBuff;
    } else {
        bOut = outBuff;
        bIn = tmpBuff;
    }

    if (inverse) {
        await invBitReverse(bOut, buffSrc, nPols, nBits);
    } else {
        await bitReverse(bOut, buffSrc, nPols, nBits);
    }
    [bIn, bOut] = [bOut, bIn];

    for (let i=0; i<nBits; i+= blockBits) {
        const sInc = Math.min(blockBits, nBits-i);
        const promisesFFT = [];

        // let results = [];
        for (j=0; j<nBlocks; j++) {
            const bb = bIn.slice(j*blockSize*nPols, (j+1)*blockSize*nPols);
            promisesFFT.push(pool.exec("fft_block", [bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc]));

            // results[j] = await fft_block(bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc);
        }
        const results = await Promise.all(promisesFFT);
        for (let i=0; i<results.length; i++) {
            bIn.set(results[i], i*blockSize*nPols)
        }
        if (sInc < nBits) {                // Do not transpose if it's the same
            await traspose(bOut, bIn, nPols, nBits, sInc);
            [bIn, bOut] = [bOut, bIn];
        }
    }

    await pool.terminate();
}

async function fft(buffSrc, nPols, nBits, buffDst) {
    await _fft(buffSrc, nPols, nBits, buffDst, false)
}

async function ifft(buffSrc, nPols, nBits, buffDst) {
    await _fft(buffSrc, nPols, nBits, buffDst, true)
}


async function interpolate(buffSrc, nPols, nBits, buffDst, nBitsExt) {
    const n = 1 << nBits;
    const nExt = 1 << nBitsExt;
    const tmpBuff = new BigBuffer(nExt*nPols);
    const outBuff = buffDst;

    let bIn, bOut;

    const pool = workerpool.pool(__dirname + '/fft_worker.js');

    const idealNBlocks = pool.maxWorkers*blocksPerThread;
    let nTrasposes = 0;


    let blockBits = log2(n*nPols/idealNBlocks);
    if (blockBits < minBlockBits) blockBits = minBlockBits;
    if (blockBits > maxBlockBits) blockBits = maxBlockBits;
    blockBits = Math.min(nBits, blockBits);
    const blockSize = 1 << blockBits;
    const nBlocks = n / blockSize;

    if (blockBits < nBits) {
        nTrasposes += Math.floor((nBits-1) / blockBits)+1;
    }

    nTrasposes += 1 // The middle convertion

    let blockBitsExt = log2(nExt*nPols/idealNBlocks);
    if (blockBitsExt < minBlockBits) blockBitsExt = minBlockBits;
    if (blockBitsExt > maxBlockBits) blockBitsExt = maxBlockBits;
    blockBitsExt = Math.min(nBitsExt, blockBitsExt);
    const blockSizeExt = 1 << blockBitsExt;
    const nBlocksExt = nExt / blockSizeExt;

    if (blockBitsExt < nBitsExt) {
        nTrasposes += Math.floor((nBitsExt-1) / blockBitsExt)+1;
    }


    if (nTrasposes & 1) {
        bOut = tmpBuff;
        bIn = outBuff;
    } else {
        bOut = outBuff;
        bIn = tmpBuff;
    }

    console.log("Interpolating reverse....")
    await interpolateBitReverse(bOut, buffSrc, nPols, nBits);
    [bIn, bOut] = [bOut, bIn];

    for (let i=0; i<nBits; i+= blockBits) {
        console.log("Layer ifft"+i);
        const sInc = Math.min(blockBits, nBits-i);
        const promisesFFT = [];

        // let results = [];
        for (j=0; j<nBlocks; j++) {
            const bb = bIn.slice(j*blockSize*nPols, (j+1)*blockSize*nPols);
            promisesFFT.push(pool.exec("fft_block", [bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc]));

            // results[j] = await fft_block(bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc);
        }
        const results = await Promise.all(promisesFFT);
        for (let i=0; i<results.length; i++) {
            bIn.set(results[i], i*blockSize*nPols)
        }

        if (sInc < nBits) {                // Do not transpose if it's the same
            await traspose(bOut, bIn, nPols, nBits, sInc);
            [bIn, bOut] = [bOut, bIn];
        }
    }

    console.log("Interpolating prepare....")
    await interpolatePrepare(pool, bIn, nPols, nBits, nBitsExt);
    console.log("Bit reverse....")
    await bitReverse(bOut, bIn, nPols, nBitsExt);
    [bIn, bOut] = [bOut, bIn];

    for (let i=0; i<nBitsExt; i+= blockBitsExt) {
        console.log("Layer fft "+i);
        const sInc = Math.min(blockBitsExt, nBitsExt-i);
        const promisesFFT = [];

        // let results = [];
        for (j=0; j<nBlocksExt; j++) {
            const bb = bIn.slice(j*blockSizeExt*nPols, (j+1)*blockSizeExt*nPols);
            promisesFFT.push(pool.exec("fft_block", [bb, j*blockSizeExt, nPols, nBitsExt, i+sInc, blockBitsExt, sInc]));

            // results[j] = await fft_block(bb, j*blockSizeExt, nPols, nBitsExt, i+sInc, blockBitsExt, sInc);
        }
        const results = await Promise.all(promisesFFT);
        for (let i=0; i<results.length; i++) {
            bIn.set(results[i], i*blockSizeExt*nPols)
        }

        if (sInc < nBitsExt) {                // Do not transpose if it's the same
            await traspose(bOut, bIn, nPols, nBitsExt, sInc);
            [bIn, bOut] = [bOut, bIn];
        }
    }
    console.log("interpolation terminated");

    await pool.terminate();
    console.log("pool terminated");
}

module.exports.fft = fft;
module.exports.ifft = ifft;
module.exports.interpolate = interpolate;
module.exports.traspose = traspose;


