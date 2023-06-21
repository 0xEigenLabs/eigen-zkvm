const JSONbig = require('json-bigint')({ useNativeBigInt: true, alwaysParseAsBig: true });
const fs = require("fs");
const path = require("path");
const ejs = require("ejs");
const version = require("../package").version;

const argv = require("yargs")
    .version(version)
    .usage("node main_genrecursive.js -v <verification_key.json>  -o <recursive.circom> ")
    .alias("v", "verkey")
    .alias("o", "output")
    .argv;

async function run() {

    const verKeyFile = typeof(argv.verkey) === "string" ?  argv.verkey.trim() : "mycircuit.verkey.json";
    const outputFile = typeof(argv.output) === "string" ?  argv.output.trim() : "mycircuit.verifier.circom";

    const verKey = JSONbig.parse(await fs.promises.readFile(verKeyFile, "utf8"));
    const constRoot = verKey.constRoot;

    const template = await fs.promises.readFile(path.join(__dirname, "..", "recursive", "recursive2.circom.ejs"), "utf8");

    const obj = {
        constRoot: constRoot,
    };

    const verifier = ejs.render(template ,  obj);

    await fs.promises.writeFile(outputFile, verifier, "utf8");

    console.log("file Generated Correctly");

}

run().then(()=> {
    process.exit(0);
}, (err) => {
    console.log(err.message);
    console.log(err.stack);
    process.exit(1);
});
