const { expect } = require("chai");
const { ethers } = require("hardhat");
import { BigNumberish } from "ethers";
import * as fs from 'fs';

interface Proof {
    a: [BigNumberish, BigNumberish];
    b: [[BigNumberish, BigNumberish], [BigNumberish, BigNumberish]];
    c: [BigNumberish, BigNumberish];
}

function parseProof(proof: any): Proof {
    return {
        a: [proof.pi_a.x, proof.pi_a.y],
        b: [
            [proof.pi_b.x[0], proof.pi_b.x[1]],
            [proof.pi_b.y[0], proof.pi_b.y[1]],
        ],
        c: [proof.pi_c.x, proof.pi_c.y],
    };
}


describe("Plonk & Groth16 verifier test", function() {
  it("Test Plonk verifier", async function() {
    const input = require("../../public.json");
    const proof = require("../../proof.json");
    const verifierFactory = await ethers.getContractFactory("KeyedVerifier");
    const verifier = await verifierFactory.deploy();

    await verifier.deployed();

    expect(await verifier.verify_serialized_proof(input, proof)).to.equal(true);
  });

  it.only("Test Groth16 verifier", async () => {
    let F = await ethers.getContractFactory("Verifier");
    let contract = await F.deploy();
    await contract.deployed();
    let proof_json = JSON.parse(fs.readFileSync("../input/groth16_proof.json", "utf8"))
    let proof = parseProof(proof_json)
    const publicInput = require("../input/groth16_public_input.json");

    expect(await contract.verifyTx(
        proof,
        publicInput
    )).to.eq(true)
  })
});
