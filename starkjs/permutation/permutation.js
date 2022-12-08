const F1Field = require("pil-stark").FGL;
const { FGL } = require("pil-stark");
const {pil_verifier, utils} = require("../index.js");
const path = require("path");


class PE {
 async buildConstants(pols_) {

   let pols = pols_.Global;
   const N = pols.L1.length;

    for ( let i=0; i<N; i++) {
      pols.L1[i] = (i == 0)? 1n : 0n;
    }

 }

async execute(pols_) {
    let pols = pols_.Permutation;
    const N = pols.c.length;

    for (let i=0; i<N; i++) {
        pols.a[i] = BigInt(i*i+i+1);
        pols.b[N-i-1] = pols.a[i];
        if (i%2 == 0) {
            pols.selC[i] = 1n;
            pols.c[i] = pols.a[i];
            pols.selD[i/2] = 1n;
            pols.d[i/2] = pols.a[i];
        } else {
            pols.selC[i] = 0n;
            pols.c[i] = 44n;
            pols.selD[(N/2) + (i-1)/2] = 0n;
            pols.d[(N/2) + (i-1)/2] = 55n;
        }
    }

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
  verificationHashType: "BN128",
  steps: [
    {nBits: 11},
    {nBits: 3}
  ]
}
console.log("security level(bits)", utils.security_test(starkStruct, 1024))

const pilFile = path.join(__dirname, "./permutation_main.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()
const pilConfig = {};
const pilCache = "/tmp/pe.pil.json"
pil_verifier.generate(argv.workspace, pilFile, pilConfig, pilCache, new PE(), starkStruct, proverAddr, []).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})

