// copied and modified from pil-stark
const { assert } = require("chai");
const LinearHash = require("./linearhash.bn128");
const workerpool = require("workerpool");
const fs = require("fs");
const { buildPoseidon, buildPoseidonWasm } = require("circomlibjs");
const { buildF1m } = require("wasmcurves");
const {linearHash, merkelizeLevel} = require("./merklehash_bn128_worker");
const { ModuleBuilder } = require("wasmbuilder")
const { BigBuffer } = require("pilcom");

module.exports = async function buildMerkleHash() {
    const wasmModule = await getWasmModule();
    const poseidon = await buildPoseidon();
    const MH = new MerkleHash(poseidon, wasmModule);
    return MH;
}

class MerkleHash {

    constructor(poseidon, wasmModule) {
        this.poseidon = poseidon;
        this.F = poseidon.F;
        this.lh = new LinearHash(poseidon);
        this.minOpsPerThread = 1<<12;
        this.maxOpsPerThread = 1<<16;
        this.wasmModule = wasmModule;
        this.useThreads = true;
    }

    _getNNodes(n) {
        let nextN = (Math.floor((n-1)/16)+1);
        let acc = nextN*16;
        while (n>1) {
            // FIll with zeros if n nodes in the leve is not even
            n = nextN;
            nextN = (Math.floor((n-1)/16)+1);
            if (n>1) {
                acc += nextN*16;
            } else {
                acc +=1;
            }
        }
        return acc;
    }

    async merkelize(buff, width, height) {
        const self = this;
        const tree = {
            elements: buff,
            nodes: new BigUint64Array(this._getNNodes(height)*4),
            width: width,
            height: height
        };

        const pool = workerpool.pool(__dirname + '/merklehash_bn128_worker.js');
//const pool = {maxWorkers: 15};

        const promisesLH = [];
        let res = [];
        let nPerThreadF = Math.floor((height-1)/pool.maxWorkers)+1;
        const minPT = Math.floor(this.minOpsPerThread / (Math.floor((width -1) / (3*16)) + 1));
        if (nPerThreadF < minPT) nPerThreadF = minPT;
        if (nPerThreadF > this.maxOpsPerThread) nPerThreadF = this.maxOpsPerThread;
        for (let i=0; i< height; i+=nPerThreadF) {
            const curN = Math.min(nPerThreadF, height-i);
            const bb = tree.elements.slice(i*width, (i+ curN)*width);
            if (self.useThreads) {
                promisesLH.push(pool.exec("linearHash", [self.wasmModule, bb, width, i, height]));
            } else {
                res.push(await linearHash(self.wasmModule, bb, width, i, height));
            }

            let st = pool.stats();
            while (st.pendingTasks > pool.maxWorkers) {
                console.log("active waiting");
                await new Promise(r => setTimeout(r, 100));
                st = pool.stats();
            }
        }
        if (self.useThreads) {
            res = await Promise.all(promisesLH)
        }
        for (let i=0; i<res.length; i++) {
            tree.nodes.set(res[i], i*nPerThreadF*4);
        }

        let pIn = 0;
        let n256 = height;
        let nextN256 = (Math.floor((n256-1)/16)+1);
        let pOut = pIn + nextN256*16*32;
        while (n256>1) {
            // FIll with zeros if n nodes in the leve is not even
            await _merkelizeLevel(pIn, nextN256, pOut);

            n256 = nextN256;
            nextN256 = (Math.floor((n256-1)/16)+1);
            pIn = pOut;
            pOut = pIn + nextN256*16*32;
        }

        pool.terminate();

        return tree;

        async function _merkelizeLevel(pIn, nOps, pOut) {
            let res = [];
            const promises = [];
            let nOpsPerThread = Math.floor((nOps-1)/pool.maxWorkers)+1;
            if (nOpsPerThread < self.minOpsPerThread) nOpsPerThread = self.minOpsPerThread;

            for (let i=0; i< nOps; i+=nOpsPerThread) {
                const curNOps = Math.min(nOpsPerThread, nOps-i);
                const bb = tree.nodes.slice(pIn/8 + i*64, pIn/8 + (i+curNOps)*64);
                if (self.useThreads) {
                    promises.push(pool.exec("merkelizeLevel", [self.wasmModule, bb, i, nOps]));
                } else {
                    res.push(await merkelizeLevel(self.wasmModule, bb, i, nOps));
                }
            }
            if (self.useThreads) {
                res = await Promise.all(promises);
            }
            for (let i=0; i<res.length; i++) {
                tree.nodes.set(res[i], pOut/8 + i*nOpsPerThread*4 );
            }
        }
    }

    // idx is the root of unity
    getElement(tree, idx, subIdx) {

        return tree.elements.getElement(tree.width*idx + subIdx);
    }


    getGroupProof(tree, idx) {
        const self = this;
        if ((idx<0)||(idx>=tree.height)) throw new Error("Out of range");

        const v = new Array(tree.width);
        for (let i=0; i<tree.width; i++) {
            v[i] = this.getElement(tree, idx, i);
        }

        const mp = merkle_genMerkleProof(tree, idx, 0, tree.height);

        return [v, mp];

        function merkle_genMerkleProof(tree, idx, offset, n) {
            if (n<=1) return [];
            const nextIdx = idx >> 4;

            const si =  idx & 0xFFFFFFF0;

            const sibs = [];

            for (let i=0; i<16; i++) {
                const buff8 = new Uint8Array(tree.nodes.buffer, offset + (si+i)*32, 32 );
                sibs.push(self.F.toObject(buff8));
            }

            const nextN = Math.floor((n-1)/16)+1;

            return [sibs, ...merkle_genMerkleProof(tree, nextIdx, offset+ nextN*16*32, nextN )];
        }
    }

    calculateRootFromGroupProof(mp, idx, vals) {

        const self = this;
        const lh = this.lh;


        const a = [];
        for (let i=0; i<vals.length; i++) {
            if (Array.isArray(a[i])) {
                for (j=0; j<vals[i].length; j++) {
                    a.push(vals[i][j]);
                }
            } else {
                a.push(vals[i]);
            }
        }

        const h = lh.hash(a);

        return this.F.toObject(merkle_calculateRootFromProof(mp, idx, h));

        function merkle_calculateRootFromProof(mp, idx, value, offset) {
            offset = offset || 0;
            if (mp.length == offset) {
                return value;
            }

            const curIdx = idx & 0xF;
            const nextIdx = idx >> 4;

            const buff = new Uint8Array(32*16);
            for (let i=0; i<16; i++) {
                buff.set(self.F.e(mp[offset][i]), i*32);
            }
            buff.set(value, curIdx*32);

            const nextValue = self.poseidon(buff);

            return merkle_calculateRootFromProof(mp, nextIdx, nextValue, offset+1);
        }

    }

    eqRoot(r1, r2) {
        return r1 === r2;
    }

    verifyGroupProof(root, mp, idx, groupElements) {
        const cRoot = this.calculateRootFromGroupProof(mp, idx, groupElements);
        return this.eqRoot(cRoot, root);
    }

    root(tree) {
        const buff8 = new Uint8Array(tree.nodes.buffer, tree.nodes.byteLength-32, 32);
        return this.F.toObject(buff8);
    }

    async writeToFile(tree, fileName) {
        const fd =await fs.promises.open(fileName, "w+");
        const header = new BigUint64Array(2);
        header[0]= BigInt(tree.width);
        header[1]= BigInt(tree.height);
        await fd.write(header);
        await writeBigBuffer(fd, tree.elements);
        await writeBigBuffer(fd, tree.nodes);
        await fd.close();

        async function writeBigBuffer(fd, buff) {
            const MaxBuffSize = 1024*1024*32;  //  256Mb
            for (let i=0; i<buff.length; i+= MaxBuffSize) {
                console.log(`writting tree.. ${i} / ${buff.length}`);
                const n = Math.min(buff.length -i, MaxBuffSize);
                const sb = buff.slice(i, n+i);
                await fd.write(sb);
            }
        }
    }

    async readFromFile(fileName) {
        const fd =await fs.promises.open(fileName, "r");
        const header = new BigUint64Array(2);
        await fd.read(header, {offset:0, length: 16, position:0});
        const tree = {
            width: Number(header[0]),
            height: Number(header[1])
        }
        tree.elements = new BigBuffer(tree.width*tree.height);
        tree.nodes = new BigUint64Array(this._getNNodes(tree.height)*4);
        await readBigBuffer(fd, tree.elements, 16);
        await readBigBuffer(fd, tree.nodes, 16+ tree.elements.length*8);
        await fd.close();

        async function  readBigBuffer(fd, buff, pos) {
            const MaxBuffSize = 1024*1024*32;  //  256Mb
            let o =0;
            for (let i=0; i<buff.length; i+= MaxBuffSize) {
                const n = Math.min(buff.length -i, MaxBuffSize);
                const buff8 = new Uint8Array(n*8);
                await fd.read(buff8, {offset: 0, length:n*8, position:pos + i*8});
                const buff64 = new BigUint64Array(buff8.buffer);
                buff.set(buff64, o);
                o += n;
            }
        }

        return tree;
    }
}

async function getWasmModule() {

    const moduleBuilder = new ModuleBuilder();
    buildF1m(moduleBuilder, "21888242871839275222246405745257275088548364400416034343698204186575808495617", "frm");

    buildPoseidonWasm(moduleBuilder);

    const code = moduleBuilder.build();

    const wasmModule = await WebAssembly.compile(code);

    return wasmModule;
}




