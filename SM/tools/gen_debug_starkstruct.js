const fs = require("fs")
const { compile } = require("pilcom");
const { F1Field } = require("ffjavascript");

const argv = require("yargs")
    .usage("node gen_debug_starkstruct.js -p <pil> -P <pilconfig> -s <starkstruct.json> -t <GL|BN128> -q <queries> -e <extensionBits> -d <stepdelta>")
    .help('h')
    .alias("s", "starkstruct")
    .alias("p", "pil")
    .alias("P", "pilconfig")
    .alias("t", "type")
    .alias("q", "queries")
    .alias("e", "extbits")
    .alias("d", "stepdelta")
    .argv;

async function main(){
    let Fr = new F1Field("0xFFFFFFFF00000001");
    const pilFile = typeof(argv.pil) === "string" ?  argv.pil.trim() : "pil/main.pil";
    const pilConfig = typeof(argv.pilconfig) === "string" ? JSON.parse(fs.readFileSync(argv.pilconfig.trim())) : {};
    const starkstructFile = typeof(argv.starkstruct) === "string" ?  argv.starkstruct.trim() : "stark_struct.json";
    const starkType = typeof(argv.type) === "string" ?  argv.type.trim() : "GL";
    const nQueries = typeof(argv.queries) === "number" ?  argv.queries : 4;
    const extBits = typeof(argv.extbits) === "number" ?  argv.extbits : 1;
    const stepDelta = typeof(argv.stepdelta) === "number" ?  argv.stepdelta : 4;

    if (starkType !== "GL" && starkType !== "BN128") {
        throw new Error(`Invalid type ${starkType}, valid types are: GL,BN128`);
    }

    const pil = await compile(Fr, pilFile, null, pilConfig);
    const pilDeg = Object.values(pil.references)[0].polDeg;
    const pilBits = Math.log2(pilDeg);
    let starkStruct = {
        nBits: pilBits,
        nBitsExt: pilBits+extBits,
        nQueries: nQueries,
        verificationHashType: starkType,
        steps: []
    }

    let stepBits = pilBits+extBits;
    do {
        starkStruct.steps.push({nBits: stepBits});
        stepBits -= stepDelta;
    } while (stepBits > 4);
    fs.writeFileSync(starkstructFile, JSON.stringify(starkStruct, null, 1) + "\n", "utf8");
}

main().then(()=> {
    process.exit(0);
}, (err) => {
    console.log(err.message);
    console.log(err.stack);
    process.exit(1);
});