const path = require("path");
const { assert } = require("chai");
const fs = require("fs");
const ejs = require("ejs");
const { FGL, starkSetup, starkGen, starkVerify } = require("pil-stark");
const { interpolate } = require("./fft_p.js");
//const buildMerkleHashGL = require("../node_modules/pil-stark/src/merklehash_p.js");
const starkInfoGen = require("./starkinfo.js");
const { proof2zkin } = require("./proof2zkin.js");
const buildMerklehashBN128 = require("./merklehash_bn128_p.js");
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
        await fs.promises.writeFile(fileCachePil, JSON.stringify(pil, null, 1) + "\n", "utf8");
      }
    }

    let constPols = newConstantPolsArray(pil);
    await builder.buildConstants(constPols, input);
    elapse("buildConstants", timer);
    let cmPols = newCommitPolsArray(pil);
    await builder.execute(cmPols, input);
    elapse("execute", timer);

    // verify the input and trace constraints
    const res = await verifyPil(FGL, pil, cmPols, constPols);
    assert(res.length == 0);

    elapse("arithmetization", timer);

    // prove and verify the stark proof
    const proof = await this.proveAndVerify(pil, constPols, cmPols, starkStruct);
    elapse("proveAndVerify", timer);
    let zkIn = proof2zkin(proof.proof);
    elapse("proof2zkin", timer);
    zkIn.publics = proof.publics;
    zkIn.proverAddr = BigInt(proverAddr);
    elapse("proving", timer);

    // generate vk
    const vk = await this.buildConsttree(pil, constPols, cmPols, starkStruct);
    elapse("buildConsttree", timer);

    const  circomFile = path.join(workspace, "circuit.circom")
    const verifier = await this.pil2circom(pil, vk.constRoot, starkStruct)
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
    //  MH = await buildMerkleHashGL();
    //} else {
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
    let timer = []
    elapse("proveAndVerify/start", timer);
    const setup = await starkSetup(constPols, pil, starkStruct);
    elapse("proveAndVerify/starkSetup", timer);
    const proof = await starkGen(cmPols, constPols, setup.constTree, setup.starkInfo);
    elapse("proveAndVerify/starkGen", timer);
    const verified = await starkVerify(proof.proof, proof.publics, setup.constRoot, setup.starkInfo);
    elapse("proveAndVerify/starkVerify", timer);
    assert(verified == true);
    return proof;
  },

  async pil2circom(pil, constRoot, starkStruct) {
    const starkInfo = starkInfoGen(pil, starkStruct);

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
