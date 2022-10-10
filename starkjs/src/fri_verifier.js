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

const { BigBuffer, newConstantPolsArray, newCommitPolsArray, compile, verifyPil } = require("pilcom");

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
  async generate(workspace, pilFile, builder, starkStruct, proverAddr, input) {
    let timer = []
    elapse("begin", timer);
    // create and compile the trace polynomial
    let pil = await compile(FGL, pilFile);

    let constPols = newConstantPolsArray(pil);
    await builder.buildConstants(constPols, input);
    let cmPols = newCommitPolsArray(pil);
    await builder.execute(cmPols, input);

    // verify the input and trace constraints
    const res = await verifyPil(FGL, pil, cmPols, constPols);
    assert(res.length == 0);

    elapse("arithmetization", timer);

    // prove and verify the stark proof
    const proof = await this.proveAndVerify(pil, constPols, cmPols, starkStruct);
    let zkIn = proof2zkin(proof.proof);
    zkIn.publics = proof.publics;
    zkIn.proverAddr = BigInt(proverAddr);
    elapse("proving", timer);

    // generate vk
    const vk = await this.buildConsttree(pil, constPols, cmPols, starkStruct);

    const  circomFile = path.join(workspace, "fibonacci.circom")
    const verifier = await this.pil2circom(pil, vk.constRoot, starkStruct)
    console.log(circomFile);
    await fs.promises.writeFile(circomFile, verifier, "utf8");

    // ----debug begin----
    let publicFile = path.join(workspace, "fibonacci.public.info.json")
    await fs.promises.writeFile(publicFile, JSONbig.stringify(proof.publics, null, 1), "utf8");
    // ----debug end----

    let zkinFile = path.join(workspace, "fibonacci.zkin.json")
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
    const setup = await starkSetup(constPols, pil, starkStruct);
    const proof = await starkGen(cmPols, constPols, setup.constTree, setup.starkInfo);
    const verified = await starkVerify(proof.proof, proof.publics, setup.constRoot, setup.starkInfo);
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
