const getKs = require("pilcom").getKs;
const F = require("pil-stark").FGL;
const {pil_verifier, utils} = require("../index.js");
const path = require("path");

class Connection {
  async buildConstants(pols_) {
    utils.buildConstantsGlobal(pols_.Global)
    let pols = pols_.Connection;
    const N = pols.S1.length;
    const pow = utils.log2(N);

    let w = F.one;
    const ks = getKs(F, 2);
    for (let i=0; i<N; i++) {
      pols.S1[i] = w;
      pols.S2[i] = F.mul(w, ks[0]);
      pols.S3[i] = F.mul(w, ks[1]);
      w = F.mul(w, F.w[pow]);
    }

    function connect(p1, i1, p2, i2) {
      [p1[i1], p2[i2]] = [p2[i2], p1[i1]];
    }

    for (let i=0; i<N; i++) {
      if (i%2 == 0) {
        connect(pols.S1, i, pols.S2, i/2);
        connect(pols.S2, i, pols.S3, i/2);
      } else {
        connect(pols.S1, i, pols.S2, N/2 +  (i-1)/2);
        connect(pols.S2, i, pols.S3, N/2 +  (i-1)/2);
      }
    }
  }
  async execute (pols_) {
    let pols = pols_.Connection;
    const N = pols.a.length;
    for (let i=0; i<N; i++) {
      pols.a[i] = BigInt(i);
    }

    for (let i=0; i<N; i++) {
      if (i<N/2) {
        pols.b[i] = pols.a[i*2];
      } else {
        pols.b[i] = pols.a[(i-N/2)*2+1 ];
      }
    }

    for (let i=0; i<N; i++) {
      if (i<N/2) {
        pols.c[i] = pols.b[i*2];
      } else {
        pols.c[i] = pols.b[(i-N/2)*2+1 ];
      }
    }
  }
}

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node connection.js -w /path/to/workspace")
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

const pilFile = path.join(__dirname, "./connection_main.pil");
const proverAddr = "0x2FD31EB1BB3f0Ac8C4feBaF1114F42431c1F29E4";
var start = new Date().getTime()
const pilConfig = {};
const pilCache = "/tmp/connection"
pil_verifier.generate(argv.workspace, pilFile, pilConfig, pilCache, new Connection(), starkStruct, proverAddr, []).then(() => {
  var end = new Date().getTime()
  console.log('cost is', `${end - start}ms`)
})
