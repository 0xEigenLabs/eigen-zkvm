const fs= require("fs");
const version = require("../../package").version;

const argv = require("yargs")
    .version(version)
    .usage("node --starksetup <starksetup.json> --zkin1 <in1.zkin.json> --zkin2 <in2.zkin.json>  --zkinout <out.zkin.json>")
    .alias("s","starksetup")
    .alias("1","zkin1")
    .alias("2","zkin2")
    .alias("o","zkinout")
    .argv;

async function run() {

    const starkSetupFile = typeof(argv.starksetup) === "string" ?  argv.starksetup.trim() : "starksetup.json";
    const zkin1File = typeof(argv.zkin1) === "string" ?  argv.zkin1.trim() : "zkin1.json";
    const zkin2File = typeof(argv.zkin2) === "string" ?  argv.zkin2.trim() : "zkin2.json";
    const zkinOutFile = typeof(argv.zkinout) === "string" ?  argv.zkinout : "zkinOut.json";

    const starkSetup = JSON.parse(await fs.promises.readFile(starkSetupFile, "utf8"));
    const zkin1 = JSON.parse(await fs.promises.readFile(zkin1File, "utf8"));
    const zkin2 = JSON.parse(await fs.promises.readFile(zkin2File, "utf8"));

    const zkinOut = {};

    zkinOut.a_publics = zkin1.publics;
    zkinOut.a_rootC = zkin1.rootC;
    zkinOut.a_root1 = zkin1.root1;
    zkinOut.a_root2 = zkin1.root2;
    zkinOut.a_root3 = zkin1.root3;
    zkinOut.a_root4 = zkin1.root4;
    zkinOut.a_evals = zkin1.evals;
    zkinOut.a_s0_vals1 = zkin1.s0_vals1;
    zkinOut.a_s0_vals3 = zkin1.s0_vals3;
    zkinOut.a_s0_vals4 = zkin1.s0_vals4;
    zkinOut.a_s0_valsC = zkin1.s0_valsC;
    zkinOut.a_s0_siblings1 = zkin1.s0_siblings1;
    zkinOut.a_s0_siblings3 = zkin1.s0_siblings3;
    zkinOut.a_s0_siblings4 = zkin1.s0_siblings4;
    zkinOut.a_s0_siblingsC = zkin1.s0_siblingsC;
    for (let i = 1; i < starkSetup["steps"].length; i++) {
        let keyRoot = `a_s${i}_root`;
        let keySiblings = `a_s${i}_siblings`;
        let keyVals = `a_s${i}_vals`;

        zkinOut[keyRoot] = zkin1[`s${i}_root`];
        zkinOut[keySiblings] = zkin1[`s${i}_siblings`];
        zkinOut[keyVals] = zkin1[`s${i}_vals`];
    }
    zkinOut.a_finalPol = zkin1.finalPol;

    zkinOut.b_publics = zkin2.publics;
    zkinOut.b_rootC = zkin2.rootC;
    zkinOut.b_root1 = zkin2.root1;
    zkinOut.b_root2 = zkin2.root2;
    zkinOut.b_root3 = zkin2.root3;
    zkinOut.b_root4 = zkin2.root4;
    zkinOut.b_evals = zkin2.evals;
    zkinOut.b_s0_vals1 = zkin2.s0_vals1;
    zkinOut.b_s0_vals3 = zkin2.s0_vals3;
    zkinOut.b_s0_vals4 = zkin2.s0_vals4;
    zkinOut.b_s0_valsC = zkin2.s0_valsC;
    zkinOut.b_s0_siblings1 = zkin2.s0_siblings1;
    zkinOut.b_s0_siblings3 = zkin2.s0_siblings3;
    zkinOut.b_s0_siblings4 = zkin2.s0_siblings4;
    zkinOut.b_s0_siblingsC = zkin2.s0_siblingsC;
    for (let i = 1; i < starkSetup["steps"].length; i++) {
        let keyRoot = `b_s${i}_root`;
        let keySiblings = `b_s${i}_siblings`;
        let keyVals = `b_s${i}_vals`;

        zkinOut[keyRoot] = zkin2[`s${i}_root`];
        zkinOut[keySiblings] = zkin2[`s${i}_siblings`];
        zkinOut[keyVals] = zkin2[`s${i}_vals`];
    }

    zkinOut.b_finalPol = zkin2.finalPol;
    
    fs.writeFileSync(zkinOutFile, JSON.stringify(zkinOut, null, 1), "utf8");

    console.log("file Generated Correctly");

}

run().then(()=> {
    process.exit(0);
}, (err) => {
    console.log(err.message);
    console.log(err.stack);
    process.exit(1);
});
