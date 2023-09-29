import * as test from "./test";
const F3g = require("../../starkjs/node_modules/pil-stark/src/f3g")

describe("FFT Circuit Test", function () {
    let circuitFFT;
    let circuitIFFT;

    this.timeout(1000000);

    before( async () => {
        circuitFFT = await test.genMain("circuits/fft.circom","FFT", "", [3,0], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuitIFFT = await test.genMain("circuits/fft.circom","FFT", "", [3,1], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
    });

    it("Should calculate shifted fft and shifted ifft size 8", async () => {
        const F = new F3g();

        const v = [
            [1n,2n,3n],
            [4n,5n,6n],
            [7n,8n,9n],
            [10n,11n,12n],
            [13n,14n,15n],
            [16n,17n,18n],
            [19n,20n,21n],
            [22n,23n,24n]
        ];

        const input={
            in: v
        };

        const inFFT = F.fft(v);

        const w1 = await circuitFFT.calculateWitness(input, true);

        await circuitFFT.assertOut(w1, {out: inFFT});

        const input2 = {
            in: inFFT
        };

        const w2 = await circuitIFFT.calculateWitness(input2, true);

        await circuitIFFT.assertOut(w2, {out: v});

    });
});
