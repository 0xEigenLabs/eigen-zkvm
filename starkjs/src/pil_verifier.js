const path = require("path");
const { assert } = require("chai");
const fs = require("fs");
const ejs = require("ejs");
const { FGL, starkSetup, starkGen, starkVerify } = require("pil-stark");
const { interpolate } = require("../node_modules/pil-stark//src/fft_p.js");
const starkInfoGen = require("../node_modules/pil-stark/src/starkinfo.js");
const { proof2zkin } = require("../node_modules/pil-stark/src/proof2zkin.js");
const pil2circom = require("../node_modules/pil-stark/src/pil2circom.js");
const buildMerklehashBN128 = require("../node_modules/pil-stark/src/merklehash_bn128_p.js");
const buildMerkleHashGL = require("../node_modules/pil-stark/src/merklehash_p.js");
const JSONbig = require('json-bigint')({ useNativeBigInt: true, alwaysParseAsBig: true, storeAsString: true });

const {elapse} = require("./utils");

const { BigBuffer, newConstantPolsArray, newCommitPolsArray, compile, verifyPil } = require("pilcom");

module.exports = {
  async generate(workspace, pilFile, pilConfig, fileCachePil, builder, starkStruct, proverAddr, input) {
    let timer = []
    elapse("begin", timer);
    // create and compile the trace polynomial
    let pil;

    if (typeof fileCachePil !== 'undefined' && fs.existsSync(fileCachePil)) {
      pil = JSON.parse(await fs.promises.readFile(fileCachePil, "utf8"));
    } else {
      pil = await compile(FGL, pilFile, null, pilConfig);
      if (typeof fileCachePil !== "undefined") {
        await fs.promises.writeFile(fileCachePil + ".pil.json", JSON.stringify(pil, null, 1) + "\n", "utf8");
      }
    }

    let constPols = newConstantPolsArray(pil);
    await builder.buildConstants(constPols, input);
    elapse("buildConstants", timer);
    let cmPols = newCommitPolsArray(pil);
    await builder.execute(cmPols, input);
    elapse("execute", timer);
    if (typeof fileCachePil !== 'undefined') {
      constPols.saveToFile(fileCachePil + ".const")
      cmPols.saveToFile(fileCachePil + ".cm")
    }

    // verify the input and trace constraints
    const res = await verifyPil(FGL, pil, cmPols, constPols);
    assert(res.length == 0);

    elapse("arithmetization", timer);
/*
    // prove and verify the stark proof
    const proof = await this.proveAndVerify(pil, constPols, cmPols, starkStruct);
    elapse("proveAndVerify", timer);
    let zkIn = proof2zkin(proof.proof);
    elapse("proof2zkin", timer);
    zkIn.publics = proof.publics;
    if (starkStruct.verificationHashType == "BN128") {
      zkIn.proverAddr = BigInt(proverAddr);
    }
    elapse("proving", timer);

    // generate vk
    const vk = await this.buildConsttree(pil, constPols, cmPols, starkStruct);
    elapse("buildConsttree", timer);

    const  circomFile = path.join(workspace, "circuit.circom")
    const starkInfo = starkInfoGen(pil, starkStruct);
    const verifier = await pil2circom(pil, vk.constRoot, starkInfo)
    elapse("pil2circom", timer);
    console.log(circomFile);
    await fs.promises.writeFile(circomFile, verifier, "utf8");
    elapse("pil2circomToFile", timer);

    // ----debug begin----
    let publicFile = path.join(workspace, "circuit.public.info.json")
    await fs.promises.writeFile(publicFile, JSONbig.stringify(proof.publics, null, 1), "utf8");
    // ----debug end----

    let zkinFile = path.join(workspace, "circuit.zkin.json")
    await fs.promises.writeFile(zkinFile, JSONbig.stringify(zkIn, (k, v) => {
      if (typeof(v) === "bigint") {
        return v.toString();
      } else {
        return v;
      }
    }, 1), "utf8");
   */
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

  // TODO: call starky by RPC
  async proveAndVerify(pil, constPols, cmPols, starkStruct) {
    let timer = []
    elapse("proveAndVerify/start", timer);
    const setup = await starkSetup(constPols, pil, starkStruct);
    console.log("const root: ", setup.constRoot);
    elapse("proveAndVerify/starkSetup", timer);
    const proof = await starkGen(cmPols, constPols, setup.constTree, setup.starkInfo);
    elapse("proveAndVerify/starkGen", timer);
    const verified = await starkVerify(proof.proof, proof.publics, setup.constRoot, setup.starkInfo);
    elapse("proveAndVerify/starkVerify", timer);
    assert(verified == true);
    return proof;
  },
}
