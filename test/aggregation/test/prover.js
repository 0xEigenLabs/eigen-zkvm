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

  // the inputs format :
  // input[0] = encodingKey % modulusBn254;
  // input[1] = xBn254;
  // input[2] = mask;
  const signals = {
    signals: [inputs[0], inputs[1], inputs[2]],
  };

  return [solProof, signals];
}

exports.generateG16Proof = generateG16Proof;