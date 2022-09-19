const path = require("path");
const { assert } = require("chai");
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

function elapse(phase, res) {
  var end = new Date().getTime()
  var cost = 0;
  var total = 0;
  if (res.length > 0) {
    cost = end - res[res.length - 1][3];
    total = end - res[0][3];
  }
  res.push([phase, cost/1000, total/1000, end]);
}

module.exports = {
  async generate(workspace, pilFile, builder, starkStruct) {
    let timer = []
    elapse("begin", timer);
    // create and compile the trace polynomial
    let pil = await compile(FGL, pilFile);

    let constPols = newConstantPolsArray(pil);
    await builder.buildConstants(constPols.Fibonacci);
    let cmPols = newCommitPolsArray(pil);
    await builder.execute(cmPols.Fibonacci, [1, 2]);

    // verify the input and trace constraints
    const res = await verifyPil(FGL, pil, cmPols, constPols);
    assert(res.length == 0);

    elapse("arithmetization", timer);

    // prove and verify the stark proof
    const proof = await this.proveAndVerify(pil, constPols, cmPols, starkStruct);
    let zkIn = proof2zkin(proof.proof);
    zkIn.publics = proof.publics;
    elapse("proving", timer);

    // generate vk
    const vk = await this.buildConsttree(pil, constPols, cmPols, starkStruct);

    const circomFile = path.join(__dirname, "../node_modules/pil-stark/circuits.gl/fibonacci.verifier.circom");
    const verifier = await this.pil2circom(pil, vk.constRoot, starkStruct)
    await fs.promises.writeFile(circomFile, verifier, "utf8");

    elapse("pil2circom", timer);

    //TODO: this is for debug
    let circuit = await wasm_tester(circomFile, {O:1, prime: "goldilocks", include: "../node_modules/pil-stark/circuits.gl", output: workspace});
    console.log("End comliling..., circuits: ", circuit);

    elapse("snark_compile", timer);
    // setup key
    const F = FGL;
    const r1csFile = path.join(circuit.dir, "fibonacci.verifier.r1cs")
    const r1cs = await readR1cs(r1csFile, {F: F, logger:console });
    const setupRes = await plonkSetup(r1cs);

    let c12PilFile = path.join(workspace, "c12.pil");
    await fs.promises.writeFile(c12PilFile, setupRes.pilStr, "utf8");

    const c12Pil = await compile(F, c12PilFile, null, {}/*pilConfig*/);
    elapse("snark_setup", timer);

    // gen stark info
    const c12StarkStruct = {
      nBits: 16,
      nBitsExt: 17,
      nQueries: 9,
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
    c12ConstPols.$$array = setupRes.constPols.$$array;

    const wasmFile = path.join(workspace, "fibonacci.verifier_js/fibonacci.verifier.wasm");
    const fd =await fs.promises.open(wasmFile, "r");
    const st =await fd.stat();
    const wasm = new Uint8Array(st.size);
    await fd.read(wasm, 0, st.size);
    await fd.close();

    const wc = await WitnessCalculatorBuilder(wasm);

    // read input
    const nAdds = setupRes.plonkAdditions.length;
    const nSMap = setupRes.sMap[0].length;
    const addsBuff = setupRes.plonkAdditions;
    const sMapBuff = setupRes.sMap;

    const w = await wc.calculateWitness(zkIn);

    for (let i=0; i<nAdds; i++) {
      w.push( F.add( F.mul( w[addsBuff[i][0]], addsBuff[i][2]), F.mul( w[addsBuff[i][1]],  addsBuff[i][3])));
    }

    const Nbits = log2(nSMap -1) +1;
    const N = 1 << Nbits

    for (let i=0; i<nSMap; i++) {
      for (let j=0; j<12; j++) {
        if (sMapBuff[j][i] != 0) {
          c12CmPols.Compressor.a[j][i] = w[sMapBuff[j][i]];
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

    elapse("snark_wtns", timer);
    const c12Vk = await this.buildConsttree(c12Pil, c12ConstPols, c12CmPols, c12StarkStruct);
    const c12Verifier = await this.pil2circom(c12Pil, c12Vk.constRoot, c12StarkStruct)
    let c12CircomFile = path.join(workspace, "c12.verifier.circom");
    await fs.promises.writeFile(c12CircomFile, c12Verifier, "utf8");
    // verify
    elapse("pil2circom_2", timer);

    const c12Proof = await this.proveAndVerify(c12Pil, c12ConstPols, c12CmPols, c12StarkStruct);

    elapse("snark_prove", timer);
    const c12ZkIn = proof2zkin(c12Proof.proof);
    c12ZkIn.proverAddr = BigInt("0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4");
    c12ZkIn.publics = c12Proof.publics;

    // ----debug begin----
    let publicFile = path.join(workspace, "c12.public.info.json")
    await fs.promises.writeFile(publicFile, JSONbig.stringify(c12Proof.publics, null, 1), "utf8");
    // ----debug end----

    let zkinFile = path.join(workspace, "c12.zkin.json")
    await fs.promises.writeFile(zkinFile, JSONbig.stringify(c12ZkIn, (k, v) => {
      if (typeof(v) === "bigint") {
        return v.toString();
      } else {
        return v;
      }
    }, 1), "utf8");

    // ----debug begin----
    let proofFile = path.join(workspace, "c12.proof.json")
    await fs.promises.writeFile(proofFile, JSONbig.stringify(c12Proof.proof, null, 1), "utf8");
    elapse("snark_generate_input", timer);
    console.log("cost: ", timer);
    // ----debug end----
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
    assert(verified == true);
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
