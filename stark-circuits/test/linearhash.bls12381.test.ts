import * as test from "./test";

describe("Linear Hash Circuit Test", function () {
    let circuit;
    let circuit100;

    this.timeout(1000000);

    before( async () => {
        circuit = await test.genMain("circuits/linearhash_bls12381.circom","LinearHash", "", [9,3], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuit100 = await test.genMain("circuits/linearhash_bls12381.circom","LinearHash", "", [100,3], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
    });

    it("Should calculate linear hash of 9 complex elements", async () => {

        const input={
            in: [
                [0n,0n,0n],
                [1n,1n,1n],
                [2n,2n,2n],
                [3n,3n,3n],
                [4n,4n,4n],
                [5n,5n,5n],
                [6n,6n,6n],
                [7n,7n,7n],
                [8n,8n,8n],
            ]
        };
        console.log(input)

        const w1 = await circuit.calculateWitness(input, true);
        await circuit.assertOut(w1, {out: "47151923872170312558486671489594063022534199585560147550196414719559738047675"});
    });

    it("Should calculate linear hash of 100 complex elements", async () => {
        const input={
            in: []
        };

        for (let i=0; i<100; i++) {
            input.in.push([i, i*1000, i*1000000])
        }
        console.log(input)

        const w1 = await circuit100.calculateWitness(input, true);
        await circuit100.assertOut(w1, {out : "12173687307340502514807899805788742433388743486605722425856884343695310570174"});
    });
});
