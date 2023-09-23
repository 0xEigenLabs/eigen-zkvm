import * as test from "./test";
const {evalPol} = require("../../starkjs/node_modules/pil-stark/src/polutils");
const F3g = require("../../starkjs/node_modules/pil-stark/src/f3g")

describe("EvalPol Circuit Test", function () {
    let circuit;

    this.timeout(1000000);

    before( async () => {
        circuit = await test.genMain("circuits/evalpol.circom","EvalPol", "", [32], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
    });

    it("Should calculate polynomial evaluation selector", async () => {
        const F = new F3g();

        const nBits = 5;
        const N = 1 << nBits;

        const pol = [];
        for (let j=0; j<N; j++) {
            pol[j] = [];
            for (let k=0; k<3; k++) {
                pol[j][k] = BigInt(k*100+j);
            }
        }
        const x = [555n, 666n, 777n];

        const input={
            pol: pol,
            x: x
        };

        const res = evalPol(F, pol, x);

        const w1 = await circuit.calculateWitness(input, true);

        await circuit.assertOut(w1, {out: res});
    });
});
