const { FGL, starkSetup, starkGen, starkVerify } = require("pil-stark");
const { newConstantPolsArray, newCommitPolsArray, compile, verifyPil  } = require("pilcom");
const path = require("path");
const F1Field = require("ffjavascript").F1Field;
// Files
const pilFile = path.join(__dirname, "./poseidong.pil");
// const input = require("../mfib.input.json");
const poseidonExecutor = require("./sm_poseidong");
const starkStruct = require("./starkstruct.json");

async function generateAndVerifyPilStark(inputs) {
    // Generate constants (preprocessed)
    console.log(pilFile)
    const pil = await compile(FGL, pilFile, null, { defines: { N: 2 ** 8}});
    const constPols = newConstantPolsArray(pil);
    const cmPols = newCommitPolsArray(pil);
    // 
    await poseidonExecutor.buildConstants(constPols.PoseidonG);
    const executionResult = await poseidonExecutor.execute(cmPols.PoseidonG, inputs); 
    console.log(executionResult);

    // Generate trace
    const evaluationPilResult = await verifyPil(FGL, pil, cmPols , constPols); 
    if (evaluationPilResult.length != 0) {
        console.log("Abort: the execution trace generated does not satisfy the PIL description!"); 
        for (let i=0; i < evaluationPilResult.length; i++) {
          console.log(pilVerificationResult[i]); } return;
        } else { 
          console.log("Continue: execution trace matches the PIL!"); }

    // Setup for the stark
    const setup = await starkSetup(constPols, pil, starkStruct);

    // Generate the stark
    const proverResult = await starkGen(cmPols,constPols,setup.constTree,setup.starkInfo);

    console.log("STARK Proof:",proverResult)
    // Verify the stark
    const verifierResult= await starkVerify(proverResult.proof, proverResult.publics, setup.constRoot,setup.starkInfo);
    if (verifierResult === true) { console.log("VALID proof!");
     } else { console.log("INVALID proof!"); }
    
    return proverResult;
}

const F = new F1Field("0xFFFFFFFF00000001");
const n1 = F.e(-1);
const myinput = [
  [0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  0,  0, 0x3c18a9786cb0b359n, 0xc4055e3364a246c3n, 0x7953db0ab48808f4n, 0xc71603f33a1144can],
  [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0xd64e1e3efc5b8e9en, 0x53666633020aaa47n, 0xd40285597c6a8825n, 0x613a4f81e81231d2n],
  [n1, n1, n1, n1, n1, n1, n1, n1, n1, n1, n1, n1, 0xbe0085cfc57a8357n, 0xd95af71847d05c09n, 0xcf55a13d33c1c953n, 0x95803a74f4530e82n]
];
generateAndVerifyPilStark(myinput);