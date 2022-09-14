const path = require("path");
const { expect } = require("chai");
const fs = require("fs");
const ejs = require("ejs");
const wasm_tester = require("circom_tester").wasm;
const { FGL, starkSetup, starkGen, starkVerify } = require("pil-stark");
const { interpolate } = require("../node_modules/pil-stark/src/fft_p.js");
const buildMerkleHashGL = require("../node_modules/pil-stark/src/merklehash_p.js");
const starkInfoGen = require("../node_modules/pil-stark/src/starkinfo.js");
const F1Field = require("../node_modules/pil-stark/src/f3g.js");
const { proof2zkin } = require("../node_modules/pil-stark/src/proof2zkin.js");
const { WitnessCalculatorBuilder } = require("circom_runtime");
const {log2} = require("../node_modules/pil-stark/src/utils.js");
const buildMerklehashBN128 = require("../node_modules/pil-stark/src/merklehash_bn128_p.js");
const JSONbig = require('json-bigint')({ useNativeBigInt: true, alwaysParseAsBig: true, storeAsString: true });

const {readR1cs} = require("r1csfile");
const plonkSetup = require("../node_modules/pil-stark/src/compressor12/compressor12_setup.js");

const {BigBuffer} = require("pilcom");
const { newConstantPolsArray, newCommitPolsArray, compile, verifyPil } = require("pilcom");

const util = require('util')

module.exports = {
  async generate(workspace, pilFile, builder, starkStruct) {
    // create and compile the trace polynomial
    let pil = await compile(FGL, pilFile);

    let constPols = newConstantPolsArray(pil);
    await builder.buildConstants(constPols.Fibonacci);
    let cmPols = newCommitPolsArray(pil);
    await builder.execute(cmPols.Fibonacci, [1, 2]);

    // verify the input and trace constraints
    const res = await verifyPil(FGL, pil, cmPols, constPols);
    expect(res.length).eq(0);


    // prove and verify the stark proof
    const proof = await this.proveAndVerify(pil, constPols, cmPols, starkStruct);
    let zkIn = proof2zkin(proof.proof);
    zkIn.publics = proof.publics;

    // generate vk
    const vk = await this.buildConsttree(pil, constPols, cmPols, starkStruct);

    const circomFile = path.join(__dirname, "../node_modules/pil-stark/circuits.gl/fibonacci.verifier.circom");
    const verifier = await this.pil2circom(pil, vk.constRoot, starkStruct)
    await fs.promises.writeFile(circomFile, verifier, "utf8");

    //TODO: this is for debug
    let circuit = await wasm_tester(circomFile, {O:1, prime: "goldilocks", include: "../node_modules/pil-stark/circuits.gl", output: workspace});
    console.log("End comliling..., circuits: ", circuit);

    // setup key
    const F = FGL;
    const r1csFile = path.join(circuit.dir, "fibonacci.verifier.r1cs")
    const r1cs = await readR1cs(r1csFile, {F: F, logger:console });
    const setupRes = await plonkSetup(r1cs);

    const c12ExecFile = path.join(workspace, "c12.exec");
    await this.writeExecFile(c12ExecFile, setupRes.plonkAdditions, setupRes.sMap);

    let c12PilFile = path.join(workspace, "c12.pil");
    await fs.promises.writeFile(c12PilFile, setupRes.pilStr, "utf8");
    let c12ConstFile = path.join(workspace, "c12.const");
    await setupRes.constPols.saveToFile(c12ConstFile)

    const c12Pil = await compile(F, c12PilFile, null, {}/*pilConfig*/);

    // gen stark info
    const c12StarkStruct = {
      nBits: 16,
      nBitsExt: 17,
      nQueries: 8,
      verificationHashType: "BN128",
      steps: [
        {nBits: 17},
        {nBits: 13},
        {nBits: 7}
      ]
    }

    // generate vk
    const starkInfo = starkInfoGen(c12Pil, c12StarkStruct);
    // prove

    const c12CmPols = newCommitPolsArray(c12Pil);
    const c12ConstPols = newConstantPolsArray(c12Pil);
    // load const pols
    await c12ConstPols.loadFromFile(c12ConstFile);

    const wasmFile = path.join(workspace, "fibonacci.verifier_js/fibonacci.verifier.wasm");
    const fd =await fs.promises.open(wasmFile, "r");
    const st =await fd.stat();
    const wasm = new Uint8Array(st.size);
    await fd.read(wasm, 0, st.size);
    await fd.close();

    const wc = await WitnessCalculatorBuilder(wasm);

    // read input
    const { nAdds, nSMap, addsBuff, sMapBuff } = await this.readExecFile(c12ExecFile);
    const w = await wc.calculateWitness(zkIn);

    for (let i=0; i<nAdds; i++) {
      w.push( F.add( F.mul( w[addsBuff[i*4]], addsBuff[i*4 + 2]), F.mul( w[addsBuff[i*4+1]],  addsBuff[i*4+3]  )));
    }

    const Nbits = log2(nSMap -1) +1;
    const N = 1 << Nbits

    for (let i=0; i<nSMap; i++) {
      for (let j=0; j<12; j++) {
        if (sMapBuff[12*i+j] != 0) {
          c12CmPols.Compressor.a[j][i] = w[sMapBuff[12*i+j]];
        } else {
          c12CmPols.Compressor.a[j][i] = 0n;
        }
      }
    }

    for (let i=nSMap; i<N; i++) {
      for (let j=0; j<12; j++) {
        c12CmPols.Compressor.a[j][i] = 0n;
      }
    }

    const c12Vk = await this.buildConsttree(c12Pil, c12ConstPols, c12CmPols, c12StarkStruct);
    const c12Verifier = await this.pil2circom(c12Pil, c12Vk.constRoot, c12StarkStruct)
    let c12CircomFile = path.join(workspace, "c12.verifier.circom");
    await fs.promises.writeFile(c12CircomFile, c12Verifier, "utf8");
    // verify

    const c12Proof = await this.proveAndVerify(c12Pil, c12ConstPols, c12CmPols, c12StarkStruct);

    const c12ZkIn = proof2zkin(c12Proof.proof);
    c12ZkIn.proverAddr = BigInt("0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4");
    c12ZkIn.publics = c12Proof.publics;

    let publicFile = path.join(workspace, "c12.public.info.json")
    await fs.promises.writeFile(publicFile, JSONbig.stringify(c12Proof.publics, null, 1), "utf8");

    let zkinFile = path.join(workspace, "c12.zkin.json")
    await fs.promises.writeFile(zkinFile, JSONbig.stringify(c12ZkIn, (k, v) => {
      if (typeof(v) === "bigint") {
        return v.toString();
      } else {
        return v;
      }
    }, 1), "utf8");

    let proofFile = path.join(workspace, "c12.proof.json")
    await fs.promises.writeFile(proofFile, JSONbig.stringify(c12Proof.proof, null, 1), "utf8");
  },

  async writeExecFile(execFile, adds, sMap) {

    const size = 2 + adds.length*4 + sMap.length*sMap[0].length;
    const buff = new BigUint64Array(size);

    buff[0] = BigInt(adds.length);
    buff[1] = BigInt(sMap[0].length);

    for (let i=0; i< adds.length; i++) {
      buff[2 + i*4     ] = BigInt(adds[i][0]);
      buff[2 + i*4 + 1 ] = BigInt(adds[i][1]);
      buff[2 + i*4 + 2 ] = adds[i][2];
      buff[2 + i*4 + 3 ] = adds[i][3];
    }

    for (let i=0; i<sMap[0].length; i++) {
      for (let c=0; c<12; c++) {
        buff[2 + adds.length*4 + 12*i + c] = BigInt(sMap[c][i]);
      }
    }

    const fd =await fs.promises.open(execFile, "w+");
    await fd.write(buff);
    await fd.close();

  },

  async readExecFile(execFile) {
    const fd =await fs.promises.open(execFile, "r");
    const buffH = new BigUint64Array(2);
    await fd.read(buffH, 0, 2*8);
    const nAdds= Number(buffH[0]);
    const nSMap= Number(buffH[1]);
    const addsBuff = new BigUint64Array(nAdds*4);
    await fd.read(addsBuff, 0, nAdds*4*8);

    const sMapBuff = new BigUint64Array(nSMap*12);
    await fd.read(sMapBuff, 0, nSMap*12*8);
    await fd.close();
    return { nAdds, nSMap, addsBuff, sMapBuff };
  },

  async buildConsttree(pil, constPols, cmPols, starkStruct) {
    const nBits = starkStruct.nBits;
    const nBitsExt = starkStruct.nBitsExt;
    const n = 1 << nBits;
    const nExt = 1 << nBitsExt;

    const constBuff  = constPols.writeToBuff();

    const constPolsArrayE = new BigBuffer(nExt*pil.nConstants);

    await interpolate(constBuff, pil.nConstants, nBits, constPolsArrayE, nBitsExt );

    let MH;
    if (starkStruct.verificationHashType == "BN128") {
      MH = await buildMerklehashBN128();
    } else if (starkStruct.verificationHashType == "GL"){
      MH = await buildMerkleHashGL();
    } else {
      throw new Error("Invalid hash type: " + starkStruct.verificationHashType)
    }

    console.log("Start merkelizing..");
    const constTree = await MH.merkelize(constPolsArrayE, pil.nConstants, nExt);

    const constRoot = MH.root(constTree);

    const verKey = {
      constRoot: constRoot
    };

    console.log("files Generated Correctly");
    return verKey
  },

  async proveAndVerify(pil, constPols, cmPols, starkStruct) {
    const setup = await starkSetup(constPols, pil, starkStruct);
    const proof = await starkGen(cmPols, constPols, setup.constTree, setup.starkInfo);
    const verified = await starkVerify(proof.proof, proof.publics, setup.constRoot, setup.starkInfo);
    expect(verified).eq(true);
    return proof;
  },

  async pil2circom(pil, constRoot, starkStruct) {
    //console.log(util.inspect(pil, {showHidden: false, depth: null, colors: true}))
    const starkInfo = starkInfoGen(pil, starkStruct);

    //console.log(util.inspect(starkInfo, {showHidden: false, depth: null, colors: true}))

    this.setDimensions(starkInfo.verifierCode.first);
    this.setDimensions(starkInfo.verifierQueryCode.first);
    let template;
    if (starkStruct.verificationHashType == "GL") {
      template = await fs.promises.readFile(path.join(__dirname, "../node_modules/pil-stark", "circuits.gl", "stark_verifier.circom.ejs"), "utf8");
    } else if (starkStruct.verificationHashType == "BN128") {
      template = await fs.promises.readFile(path.join(__dirname, "../node_modules/pil-stark", "circuits.bn128", "stark_verifier.circom.ejs"), "utf8");
    } else {
      throw new Error("Invalid Hash Type: "+ starkStruct.verificationHashType);
    }

    const obj = {
      F: FGL,
      starkInfo: starkInfo,
      starkStruct: starkStruct,
      constRoot: constRoot,
      pil: pil
    };

    return ejs.render(template ,  obj);
  },

  setDimensions(code) {
    const tmpDim = [];

    for (let i=0; i<code.length; i++) {
      let newDim;
      switch (code[i].op) {
        case 'add': newDim = Math.max(getDim(code[i].src[0]), getDim(code[i].src[1])); break;
        case 'sub': newDim = Math.max(getDim(code[i].src[0]), getDim(code[i].src[1])); break;
        case 'mul': newDim = Math.max(getDim(code[i].src[0]), getDim(code[i].src[1])); break;
        case 'copy': newDim = getDim(code[i].src[0]); break;
        default: throw new Error("Invalid op:"+ code[i].op);
      }
      setDim(code[i].dest, newDim);
    }

    function getDim(r) {
      let d;
      switch (r.type) {
        case "tmp": d=tmpDim[r.id]; break;
        case "tree1": d=r.dim; break;
        case "tree2": d=r.dim; break;
        case "tree3": d=r.dim; break;
        case "tree4": d=r.dim; break;
        case "const": d=1; break;
        case "eval": d=3; break;
        case "number": d=1; break;
        case "public": d=1; break;
        case "challenge": d=3; break;
        case "xDivXSubXi": d=3; break;
        case "xDivXSubWXi": d=3; break;
        case "x": d=3; break;
        case "Z": d=3; break;
        default: throw new Error("Invalid reference type get: " + r.type);
      }
      r.dim = d;
      return d;
    }

    function setDim(r, dim) {
      switch (r.type) {
        case "tmp": tmpDim[r.id] = dim; r.dim=dim; return;
        default: throw new Error("Invalid reference type set: " + r.type);
      }
    }
  },
}
