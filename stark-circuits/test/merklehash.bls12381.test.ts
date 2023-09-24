import * as test from "./test";
const chai = require("chai");
const assert = chai.assert;

function getBits(idx, nBits) {
    let res = [];
    for (let i=0; i<nBits; i++) {
        res[i] = (idx >> i)&1 ? 1n : 0n;
    }
    return res;
}

describe("Merkle Hash Circuit Test", function () {
    let circuit;

    this.timeout(1000000);

    before( async () => {
        circuit = await test.genMain("circuits/merklehash_bls12381.circom","MerkleHash", "", [3, 1, 4], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
    });

    it("Should calculate linear hash of 1 complex elements", async () => {
        const nBits = 2;
        const idx = 1;
        const values =  
        [
            [ 2n, 12n, 22n ]
        ];
        const siblings =[ 
        [
            7145929705339707732933780940877937508353n,
            7486212072260646196415602292383415271426n,
            7826494439181584659897423643888893034499n,
            8166776806102523123379244995394370797572n,
            0n,
            0n,
            0n,
            0n,
            0n,
            0n,
            0n,
            0n,
            0n,
            0n,
            0n,
            0n,
        ]];

        const input={
            values: values,
            siblings: siblings,
            key: getBits(idx, nBits)
        };
        console.log(input)
        const w1 = await circuit.calculateWitness(input, true);

        await circuit.assertOut(w1, {root: "32227206116237215740162377531481191838063909532381497804787245624658969614932"});
    });
});
