const { FGL } = require("pil-stark");
//const { newConstantPolsArray, newCommitPolsArray, compile, verifyPil } = require("pilcom");
const {fri_verifier} = require("../index.js");
const path = require("path");

class FibonacciJS {
  async buildConstants(pols) {
    const N = pols.ISLAST.length;
    for (let i = 0; i < N-1; i++) {
      pols.ISLAST[i] = 0n;
    }
    pols.ISLAST[N-1] = 1n;
  }

  async execute(pols, input) {
    const N = pols.aLast.length;
    pols.aBeforeLast[0] = BigInt(input[0]);
    pols.aLast[0] = BigInt(input[1]);

    for (let i = 1; i < N; i ++) {
      pols.aBeforeLast[i] = pols.aLast[i-1];
      pols.aLast[i] = FGL.add(pols.aBeforeLast[i-1], pols.aLast[i-1]);
    }
    return pols.aLast[N - 1];
  }
}

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node fibonacci.js -w /path/to/workspace")
  .alias("w", "workspace")   // workspace to stash temp and output files
  .argv;

// construct the stark parameters
const starkStruct = {
  nBits: 4,
  nBitsExt: 5,
  nQueries: 7,
  verificationHashType: "GL",
  steps: [
    {nBits: 5},
    {nBits: 3}
  ]
}
const pilFile = path.join(__dirname, "./fibonacci.pil");
fri_verifier.generate(argv.workspace, pilFile, new FibonacciJS(), starkStruct).then(() => {
  console.log("Done")
})
