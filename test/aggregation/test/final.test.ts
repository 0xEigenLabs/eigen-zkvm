const { expect } = require("chai");
const { ethers } = require("hardhat");

const proof = require("../fibonacci.final/proof.json");
const publics = require("../fibonacci.final/public.json");

describe("Plonk verifier test", function() {
  it("Groth16 Verify", async function() {
    const verifierFactory = await ethers.getContractFactory("Groth16Verifier");
    const verifier = await verifierFactory.deploy();

    await verifier.deployed();

    const solProof = [
        [proof.pi_a[0], proof.pi_a[1]],
        [
          [proof.pi_b[0][1], proof.pi_b[0][0]],
          [proof.pi_b[1][1], proof.pi_b[1][0]],
        ],
        [proof.pi_c[0], proof.pi_c[1]],
      ];
    

    expect(await verifier.verifyProof(
        solProof[0],
        solProof[1],
        solProof[2],
        publics,
    )).to.equal(true);
  });
});
