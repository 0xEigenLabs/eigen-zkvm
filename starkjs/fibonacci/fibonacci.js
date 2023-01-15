const { FGL } = require("pil-stark");
const {pil_verifier, utils} = require("../index.js");
const path = require("path");

class FibonacciJS {
  async buildConstants(pols_) {
    const pols = pols_.Fibonacci;
    const N = pols.ISLAST.length;
    for (let i = 0; i < N-1; i++) {
      pols.ISLAST[i] = 0n;
    }
    pols.ISLAST[N-1] = 1n;
  }

  async execute(pols_, input) {
    const pols = pols_.Fibonacci;
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
  .alias("w", "workspace") //workspace to stash temp and output files
  .demand('workspace')
  .argv;

// construct the stark parameters
const starkStruct = {
  nBits: 10,
  nBitsExt: 11,
  nQueries: 8,
  verificationHashType: "GL",
  steps: [
    {nBits: 11},
    {nBits: 3}
  ]
}
console.log("security level(bits)", utils.security_test(starkStruct, 1024))

const pilFile = path.join(__dirname, "./fibonacci.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()
const pilConfig = {};
const pilCache = "/tmp/fib"
pil_verifier.generate(argv.workspace, pilFile, pilConfig, pilCache, new FibonacciJS(), starkStruct, proverAddr, [1, 2]).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})
