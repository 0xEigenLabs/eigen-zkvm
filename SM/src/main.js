const { FGL } = require("pil-stark");
const {pil_verifier, utils} = require("../../starkjs/index.js");
const path = require("path");
const fs = require("fs");

const VM = require("./vm.js");

console.log(VM);

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node main.js -w /path/to/workspace")
  .alias("w", "workspace") //workspace to stash temp and output files
  .demand('workspace')
  .argv;

// construct the stark parameters
const starkStruct = JSON.parse(fs.readFileSync(path.join(__dirname, "../build/proof/starkstruct.json")))

console.log("security level(bits)", utils.security_test(starkStruct, 2**23))

const pilFile = path.join(__dirname, "../pil/main.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()

const config = {
  inputFile: path.join(__dirname, "../tools/build-genesis/input_executor.json"),
  romFile: path.join(__dirname, "../build/proof/rom.json"),
  debug: false,
  debugInfo: { inputName: 'input_executor' },
  unsigned: false,
  execute: false,
  tracer: false,
  outputFile: path.join(argv.workspace, "zkevm.commit")
}

const pilConfig = { defines: {N: 2 ** 23},
  namespaces: ['Global', 'Main', 'Rom', 'Byte4', 'MemAlign'],
  verbose: true,
  color: true
//  disableUnusedError: true
}

const fileCachePil = path.join(argv.workspace, "vm.pil.json");

pil_verifier.generate(argv.workspace, pilFile, pilConfig, fileCachePil, new VM(), starkStruct, proverAddr, config).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})
