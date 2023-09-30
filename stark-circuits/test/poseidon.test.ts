import * as test from "./test";

describe("Poseidon Circuit test", function () {
    let circuit2;
    let circuit3;
    let circuit4;
    let circuit5;
    let circuit6;
    let circuit7;
    let circuit17;

    this.timeout(1000000);

    before( async () => {
        circuit2 = await test.genMain("circuits/poseidon.circom","Poseidon", "", [1], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuit3 = await test.genMain("circuits/poseidon.circom","Poseidon", "", [2], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuit4 = await test.genMain("circuits/poseidon.circom","Poseidon", "", [3], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuit5 = await test.genMain("circuits/poseidon.circom","Poseidon", "", [4], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuit6 = await test.genMain("circuits/poseidon.circom","Poseidon", "", [5], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuit7 = await test.genMain("circuits/poseidon.circom","Poseidon", "", [6], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
        circuit17 = await test.genMain("circuits/poseidon.circom","Poseidon", "", [16], {"include": "node_modules/circomlib/circuits", "prime": "bls12381"});
    });

    it("Should check constrain of hash([1]) t=2", async () => {
        const w = await circuit2.calculateWitness({inputs: [1]}, true);
        await circuit2.assertOut(w, {out : "10090463338479474364654416042385169859560025017303585988626920959727361545503"});
        await circuit2.checkConstraints(w);
    });

    it("Should check constrain of hash([2]) t=3", async () => {
        const w = await circuit3.calculateWitness({inputs: [1,0]}, true);
        await circuit3.assertOut(w, {out : "40315999570263005229566068098191840653718756303362127561954793579940120806360"});
        await circuit3.checkConstraints(w);
    });

    it("Should check constrain of hash([3]) t=4", async () => {
        const w = await circuit4.calculateWitness({inputs: [1,0,0]}, true);
        await circuit4.assertOut(w, {out : "52171919706604857662228147548523676303297329614804576829062159794914391577198"});
        await circuit4.checkConstraints(w);
    });

    it("Should check constrain of hash([5]) t=6", async () => {
        const w = await circuit6.calculateWitness({inputs: [1,2,0,0,0]}, true);
        await circuit6.assertOut(w, {out : "25489954628706771422434337159093356230875147553184381182493646336226215511862"});
        await circuit6.checkConstraints(w);
    });

    it("Should check constrain of hash([6]) t=7", async () => {
        const w = await circuit7.calculateWitness({inputs: [1,2,0,0,0,0]}, true);
        await circuit7.assertOut(w, {out : "1013898857847217674473086247177895055941699630695530588118970595082884522651"});
        await circuit7.checkConstraints(w);
    });

    it("Should check constrain of hash([5]) t=6", async () => {
        const w = await circuit6.calculateWitness({inputs: [3,4,0,0,0]}, true);
        await circuit6.assertOut(w, {out : "11663712849936763722275869035629160480859126086041635677673535448082509089528"});
        await circuit6.checkConstraints(w);
    });

    it("Should check constrain of hash([6]) t=7", async () => {
        const w = await circuit7.calculateWitness({inputs: [3,4,0,0,0,0]}, true);
        await circuit7.assertOut(w, {out : "5752311989137819405955540762621381412568871047030860382530977484434251590586"});
        await circuit7.checkConstraints(w);
    });

    it("Should check constrain of hash([4]) t=5", async () => {
        const w = await circuit5.calculateWitness({inputs: [1,2,3,4]}, true);
        await circuit5.assertOut(w, {out : "50374862952696036512232585533148559412665642735378685892656796916864806976141"});
        await circuit5.checkConstraints(w);
    });

    it("Should check constrain of hash([16]) t=17", async () => {
        const w = await circuit17.calculateWitness({inputs: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15]}, true);
        await circuit17.assertOut(w, {out : "8515241672374781049985699179100419324899359624275223371256009421843839607813"});
        await circuit17.checkConstraints(w);
    });
});
