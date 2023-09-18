import * as test from "./test";

describe("Poseidon Circuit test", function () {
    let circuit2;

    this.timeout(1000000);

    before( async () => {
        circuit2 = await test.genMain("circuits/poseidon_bls12381.circom","Poseidon", "", [1], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
    });

    it("Should check constrain of hash([1]) t=2", async () => {
        const w = await circuit2.calculateWitness({inputs: [1]}, true);
        await circuit2.assertOut(w, {out : "10090463338479474364654416042385169859560025017303585988626920959727361545503"});
        await circuit2.checkConstraints(w);
    });

});
