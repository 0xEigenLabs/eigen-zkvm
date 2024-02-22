const { FGL } = require("pil-stark");
const {pil_verifier, utils} = require("../index.js");
const path = require("path");


class FibonacciJS {
  async buildConstants(pols_) {
    const pols = pols_.Fibonacci;
    const N = pols.L1.length;
    for (let i = 0; i < N; i++) {
      pols.L1[i] = (i == 0) ? 1n : 0n;
      pols.LLAST[i] = (i == N-1) ? 1n : 0n;
    }
  }

  async execute(pols_, input) {
    const pols = pols_.Fibonacci;
    const N = pols.l1.length;
    pols.l2[0] = BigInt(input[0]);
    pols.l1[0] = BigInt(input[1]);

    for (let i = 1; i < N; i ++) {
      pols.l2[i] =pols.l1[i-1];
      pols.l1[i] =FGL.add(FGL.square(pols.l2[i-1]), FGL.square(pols.l1[i-1]));
    }
    return pols.l1[N - 1];
  }
}

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node fibonacci.js -w /path/to/workspace --i 0 --pc /tmp/fib")
  .alias("w", "workspace") //workspace to stash temp and output files
  .alias("i", "input")
  .alias("pc","pilCache")
  .demand('workspace')
  .demand("input")
  .demand("pilCache")
  .argv;

// construct the stark parameters
const starkStruct = {
  nBits: 10,
  nBitsExt: 11,
  nQueries: 8,
  verificationHashType: "BN128",
  steps: [
    {nBits: 11},
    {nBits: 7},
    {nBits: 3}
  ]
}
console.log("security level(bits)", utils.security_test(starkStruct, 1024))

const pilFile = path.join(__dirname, "./fibonacci.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()
const pilConfig = {};
const pilCache = argv.pilCache
console.log("pilCache:", pilCache)
let input;
if (argv.input == "0") {
  input = [1, 2]
} else if (argv.input == "1") {
  input = [3, 4]
} else if (argv.input == "2") {
  input = [5, 6]
} else {
  input = [7, 8]
}

pil_verifier.generate(argv.workspace, pilFile, pilConfig, pilCache, new FibonacciJS(), starkStruct, proverAddr, input).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})
