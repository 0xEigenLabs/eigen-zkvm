import { expect } from "chai";
import { ethers } from "hardhat";

const proof_agg = require("/tmp/aggregation/aggregation_proof.json");

describe("Plonk verifier test", function() {
  it("Should return true when proof is correct", async function() {
    const verifierFactory = await ethers.getContractFactory("KeysWithPlonkVerifier");
    const verifier = await verifierFactory.deploy();

    await verifier.deployed();

    expect(await verifier.verifyAggregatedProof(
        proof_agg[0],
        proof_agg[1],
        proof_agg[2],
        proof_agg[3],
        proof_agg[4],
    )).to.equal(true);
  });
});
