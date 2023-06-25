const { FGL } = require("pil-stark");
const {pil_verifier, utils} = require("../index.js");
const path = require("path");
const poseidonExecutor = require("./sm_poseidong.js");

class PoseidonJS {
  async buildConstants(pols_) {
    await poseidonExecutor.buildConstants(pols_.PoseidonG)
  }

  async execute(pols_, input) {
    return await poseidonExecutor.execute(pols_.PoseidonG,input)
  }
}

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node main_poseidon.js -w /path/to/workspace -i 0  --pc /tmp/poseidon")
  .alias("w", "workspace") //workspace to stash temp and output files
  .alias("i", "input")
  .alias("pc","pilCache")
  .demand('workspace')
  .demand("input")
  .demand("pilCache")
  .argv

// construct the stark parameters
const starkStruct = {
  nBits: 10,
  nBitsExt: 11,
  nQueries: 8,
  verificationHashType: "GL",
  steps: [
    {nBits: 11},
    {nBits: 7},
    {nBits: 3}
  ]
}
console.log("security level(bits)", utils.security_test(starkStruct, 1024))

const pilFile = path.join(__dirname, "./poseidong.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()
const pilConfig = {};
const pilCache = argv.pilCache
console.log("pilCache:",pilCache)

let _input;
if (argv.input=="0") {
  _input = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0,  0,  0, 0x3c18a9786cb0b359n, 0xc4055e3364a246c3n, 0x7953db0ab48808f4n, 0xc71603f33a1144can]
  ];
}else{
  _input = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0xd64e1e3efc5b8e9en, 0x53666633020aaa47n, 0xd40285597c6a8825n, 0x613a4f81e81231d2n]
  ]
}

pil_verifier.generate(argv.workspace, pilFile, pilConfig, pilCache, new PoseidonJS(), starkStruct, proverAddr, _input).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})
