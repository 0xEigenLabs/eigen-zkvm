const F1Field = require("pil-stark").FGL;
const {pil_verifier, utils} = require("../index.js");
const path = require("path");

class Plookup {
  async buildConstants_ (pols) {
    const N = pols.A.length;

    let p=0;
    for ( let i=0; i<16;- i++) {
      for (let j=0; j<16; j++) {
        pols.A[p] = BigInt(i);
        pols.B[p] = BigInt(j);
        pols.SEL[p] = BigInt(1);
        p += 1;
      }
    }

    while (p<N) {
      pols.A[p] = 0n;
      pols.B[p] = 0n;
      pols.SEL[p] = 0n;
      p += 1;
    }
  }

  async buildConstants (pols) {
    await utils.buildConstantsGlobal(pols.Global)
    await this.buildConstants_(pols.Plookup)
  }

  async execute(pols_, input) {
    let pols = pols_.Plookup;
    const N = pols.cc.length;
    let p=0;
    for ( let i=0; i<16; i++) {
      for (let j=0; j<16; j++) {
        pols.cc[p] = BigInt(i*j);
        p+= 1;
      }
    }
    while (p<N) {
      pols.cc[p] = BigInt(p);
      p+= 1;
    }


    p=0;
    for (let i=0; i<10; i++) {
      pols.sel[p] = 1n;
      pols.a[p] = BigInt(i);
      pols.b[p] = i== 0 ? 55n : BigInt(i+3);
      p += 1;
    }

    pols.sel[p] = 0n;
    pols.a[p] = 55n;
    pols.b[p] = 10n;
    p += 1;

    while (p<N) {
      pols.sel[p] = 0n;
      pols.a[p] = 55n;
      pols.b[p] = 55n;
      p+= 1;
    }
  }
}

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node plookup.js -w /path/to/workspace")
  .alias("w", "workspace") //workspace to stash temp and output files
  .demand('workspace')
  .argv;

// construct the stark parameters
const starkStruct = {
  nBits: 10,
  nBitsExt: 11,
  nQueries: 8,
  verificationHashType: "GL", //FIXME BN128 not work
  steps: [
    {nBits: 11},
    {nBits: 7},
    {nBits: 3}
  ]
}
console.log("security level(bits)", utils.security_test(starkStruct, 1024))

const pilFile = path.join(__dirname, "./plookup_main.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()
const pilConfig = {};
const pilCache = "/tmp/plookup"
pil_verifier.generate(argv.workspace, pilFile, pilConfig, pilCache, new Plookup(), starkStruct, proverAddr, []).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})
