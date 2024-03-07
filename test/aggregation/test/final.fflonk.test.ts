import { expect } from "chai";
import { ethers } from "hardhat";

const proof_fflonk = require("../fibonacci.final/proof.fflonk.json");
const publics_fflonk = require("../fibonacci.final/public.fflonk.json");

// https://github.com/0xPolygonHermez/zkevm-contracts/blob/main/test/contracts/real-prover/real-prover-test-inputs.test.js#L5
function generateSolidityInputs(
    proofJson,
) {
    const { evaluations, polynomials } = proofJson;
    return [
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.C1[0]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.C1[1]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.C2[0]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.C2[1]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.W1[0]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.W1[1]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.W2[0]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(polynomials.W2[1]).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.ql).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.qr).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.qm).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.qo).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.qc).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.s1).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.s2).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.s3).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.a).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.b).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.c).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.z).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.zw).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.t1w).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.t2w).toHexString(), 32),
            ethers.utils.hexZeroPad(ethers.BigNumber.from(evaluations.inv).toHexString(), 32),
        ];
}

describe("Plonk verifier test", function() {
  it("Fflonk Verify", async function() {
    const verifierFactory = await ethers.getContractFactory("FflonkVerifier");
    const verifier = await verifierFactory.deploy();
    await verifier.deployed();

    expect(await verifier.verifyProof(generateSolidityInputs(proof_fflonk), publics_fflonk)).to.be.equal(true);
  });
});
