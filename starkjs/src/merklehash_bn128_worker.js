// copied and modified from pil-stark
const workerpool = require('workerpool');
const maxPages = 20000;

function alloc(wasmMem, length) {
    const wasmMem32 = new Uint32Array(wasmMem.buffer);
    length = (((length-1)>>3) +1)<<3;       // Align to 64 bits.

    const res = wasmMem32[0];
    wasmMem32[0] += length;
    return res;
}

async function linearHash(wasmModule, buffIn, width, st_i, st_n) {
    console.log(`linear hash bn128 start.... ${st_i}/${st_n}`);

    const heigth = buffIn.length/width;

    const bytesRequired = 16*32 + 32 + 32;
    // const bytesRequired = heigth*32*16 + 32 + heigth*32;
    const pagesRequired = Math.floor((bytesRequired - 1)/(1<<16)) +10000;

    const wasmMem = new WebAssembly.Memory({initial:pagesRequired});

    const instance = await WebAssembly.instantiate(wasmModule, {
        env: {
            "memory": wasmMem
        }
    });


    const pIn = alloc(wasmMem, 16*32);
    const pSt = alloc(wasmMem, 32);

    const in64 =new BigUint64Array(wasmMem.buffer, pIn, 4*16);
    const st64 =new BigUint64Array(wasmMem.buffer, pSt, 4);

    const prime = 0xFFFFFFFF00000001;

    const buffOut64 = new BigUint64Array(heigth*4);

    for (let i=0; i<heigth; i++) {
        const bb = new BigUint64Array(buffIn.buffer, buffIn.byteOffset + i*width*8, width);
        hash(bb);
        buffOut64.set(st64,  i*4);
    }

    console.log(`linear hash bn128 end.... ${st_i}/${st_n}`);
    return buffOut64;

    function hash(vals) {
        for (let i=0; i<4; i++) st64[i] = 0n;

        if (vals.length <=4) {
            for (let i=0; i<vals.length; i++) {
                st64[i] = vals[i];
            }
            instance.exports.frm_toMontgomery(pSt, pSt);
            return;
        }

        let p=0;

        for(let i=0; i<vals.length; i++) {
            if (vals[i] > prime) vals[i] -= prime;
            in64[p] = vals[i];
            p++;
            if (p==16*4) {
                instance.exports.poseidon(pSt, pIn, 16, pSt, 1);
                p=0;
            }
            if (i%3 == 2) {
                in64[p] = 0n;
                p++;
                instance.exports.frm_toMontgomery(pIn + p*8 - 32,pIn + p*8 - 32);
                if (p==16*4) {
                    instance.exports.poseidon(pSt, pIn, 16, pSt, 1);
                    p=0;
                }
            }
        }
        if (p>0) {
            const nLast = Math.floor((p-1)/4)+1;
            while (p<nLast*4) {
                in64[p] = 0n;
                p++;
                if (p%4 == 0) {
                    instance.exports.frm_toMontgomery(pIn + p*8 - 32,pIn + p*8 - 32);
                }
            }
            instance.exports.poseidon(pSt, pIn, nLast, pSt, 1);
            p=0;
        }
    }
}


// a deliberately inefficient implementation of the fibonacci sequence
async function merkelizeLevel(wasmModule, buffIn, st_i, st_n) {
    console.log(`merkelizing bn128 hash start.... ${st_i}/${st_n}`);
    const nOps = buffIn.byteLength / (32*16);

    // const bytesRequired = nOps*32*16 + 32 + nOps*32;
    const bytesRequired = 16*32 + 32 + 32;
    const pagesRequired = Math.floor((bytesRequired - 1)/(1<<16)) +10000;

    const wasmMem = new WebAssembly.Memory({initial:pagesRequired});
    const instance = await WebAssembly.instantiate(wasmModule, {
        env: {
            "memory": wasmMem
        }
    });

    const pIn = alloc(wasmMem, 16*32);
    const pSt = alloc(wasmMem, 32);
    const pOut = alloc(wasmMem, 32);

    const in64 =new BigUint64Array(wasmMem.buffer, pIn, 4*16);
    const st64 =new BigUint64Array(wasmMem.buffer, pSt, 4);
    const out64 =new BigUint64Array(wasmMem.buffer, pOut, 4);


    const buffOut64 = new BigUint64Array(nOps*4);

    for (let i=0; i<nOps; i++) {
        st64[0] = 0n;
        st64[1] = 0n;
        st64[2] = 0n;
        st64[3] = 0n;

        const sBuff = new BigUint64Array(buffIn.buffer, buffIn.byteOffset + i*(16*32), 16*4);
        in64.set(sBuff);
        instance.exports.poseidon(pSt, pIn, 16, pOut, 1);
        buffOut64.set(out64, i*4);
    }

    console.log(`merkelizing bn128 hash end.... ${st_i}/${st_n}`);
    return buffOut64;
}

if (!workerpool.isMainThread) {
    workerpool.worker({
        linearHash: linearHash,
        merkelizeLevel: merkelizeLevel
    });
}

module.exports.linearHash = linearHash;
module.exports.merkelizeLevel = merkelizeLevel;