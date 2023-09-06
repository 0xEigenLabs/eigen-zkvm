import * as test from "./test";

// const { assert } = require("chai");
const { buildBabyjub } = require("circomlibjs");

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

function bn1togl5(in1) {
    const out = new Array(5);
    const n2b = num2Bits(5 * 64, in1);
    for (let i = 0; i < 5; i ++) {
        out[i] = bits2Num(64, n2b.slice(64 * i, 64 * (i+1)));
    }
    return out;
}

/* globals describe, before, it */
describe("Test BN to GL5", function() {
  let circuit;
  let babyJub;

  before(async () => {
    babyJub = await buildBabyjub();
    circuit = await test.genMain("circuits/bn1togl3.circom",
      "BN1toGL5", "", [], {"include": "node_modules/circomlib/circuits", "prime": "bn128"});
  })

  it("BN1toGL5", async () => {
    console.log(babyJub.F.p)
    const in1 = babyJub.F.p - 1n;
    const wtns = await test.executeCircuit(circuit, {
          in: babyJub.F.p - 1n
        });
    const out = bn1togl5(in1);
    console.log("expected", out);
    await circuit.assertOut(wtns, { out: out });
  })
})
