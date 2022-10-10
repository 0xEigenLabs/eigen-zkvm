const { FGL } = require("pil-stark");
const {fri_verifier, utils} = require("../../starkjs/index.js");
const path = require("path");
const fs = require("fs");

const VM = require("./vm.js");

console.log(VM);

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node fibonacci.js -w /path/to/workspace")
  .alias("w", "workspace") //workspace to stash temp and output files
  .demand('workspace')
  .argv;

// construct the stark parameters
const starkStruct = JSON.parse(fs.readFileSync(path.join(__dirname, "../build/proof/starkstruct.json")))

console.log("security level(bits)", utils.security_test(starkStruct, 2**23))

const pilFile = path.join(__dirname, "../pil/main.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()

const input = {
  inputFile: path.join(__dirname, "../tools/build-genesis/input_executor.json"),
  romFile: path.join(__dirname, "../build/proof/rom.json"),
  debug: false,
  debugInfo: { inputName: 'input_executor' },
  unsigned: false,
  execute: false,
  tracer: false,
  outputFile: path.join(argv.workspace, "aaa.out")
}

fri_verifier.generate(argv.workspace, pilFile, new VM(), starkStruct, proverAddr, input).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})
