import * as test from "./test";
const { buildBls12381 } = require("ffjavascript");

function bits2Num(n, in1) {
    let lc1=0n;
    let e2 = 1n;
    for (let i = 0; i < n; i++) {
        lc1 += BigInt(in1[i]) * e2;
        e2 = e2 + e2;
    }
    return lc1
}

function num2Bits(n, in1) {
    const out = new Array(n).fill(0);
    let e2=1n;
    for (let i = 0; i<n; i++) {
        out[i] = Number((in1 >> BigInt(i)) & 1n);
        e2 = e2+e2;
    }

    return out;
}

function bls1togl3(in1) {
    const out = new Array(3);
    const n2b = num2Bits(3 * 64, in1);
    for (let i = 0; i < 3; i ++) {
        out[i] = bits2Num(64, n2b.slice(64 * i, 64 * (i+1)));
    }
    return out;
}

/* globals describe, before, it */
describe("Test BLS to GL3", function() {
  let circuit;
  let bls12381;

  before(async () => {
    circuit = await test.genMain("circuits/bls1togl3.circom",
      "BLS1toGL3", "", [], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
    bls12381 = await buildBls12381();
  })

  it("BLS1toGL3", async () => {
    console.log(bls12381.r)
    const in1 = bls12381.r - 1n;
    const wtns = await test.executeCircuit(circuit, {
          in: bls12381.r - 1n
        });
    const out = bls1togl3(in1);
    console.log("expected", out);
    await circuit.assertOut(wtns, { out: out });
  })

  after( async() => {
    bls12381.terminate();
  });
})
