const snarkjs = require("snarkjs");
require("dotenv").config();

async function generateG16Proof(witness) {
  const result = await snarkjs.groth16.fullProve(witness, "/tmp/aggregation/circuits.wasm","/tmp/aggregation/g16.zkey");
  const inputs = result.publicSignals;
  const proof = result.proof;
  const solProof = [
    [proof.pi_a[0], proof.pi_a[1]],
    [
      [proof.pi_b[0][1], proof.pi_b[0][0]],
      [proof.pi_b[1][1], proof.pi_b[1][0]],
    ],
    [proof.pi_c[0], proof.pi_c[1]],
  ];

  console.log("inputs:",inputs)

  return [solProof, inputs];
}

exports.generateG16Proof = generateG16Proof;