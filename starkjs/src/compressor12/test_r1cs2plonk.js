const fs = require("fs");
const r1cs2plonk = require("../../node_modules/pil-stark/src/r1cs2plonk");
const {readR1cs} = require("r1csfile");
const F1Field = require("../../node_modules/pil-stark/src/f3g.js");

// test render_pil_js.
// node run test_render_pil.js
async function run() {
    const F = new F1Field();

    const r1cs = await readR1cs("/tmp/multiplier2.r1cs", {F: F, logger:console });

    const [plonkConstraints, plonkAdditions] = r1cs2plonk(F, r1cs);

    // test-dump data
    fs.writeFileSync("./r1cs2plonk/plonk_constrains_js.json", JSON.stringify(plonkConstraints, (key, value) =>
        typeof value === 'bigint' ? value.toString() :value
    ));
    fs.writeFileSync("./r1cs2plonk/plonk_additions_js.json", JSON.stringify(plonkAdditions, (key, value) =>
        typeof value === 'bigint' ? value.toString() :value
    ));

    for( var i =0; i<  plonkConstraints.length; i++ ){
            let pc= plonkConstraints[i];
            console.log("{}",  pc)
    }


}


// node test_render_pil.js
run().then(()=> {
    process.exit(0);
}, (err) => {
    console.log(err.message);
    console.log(err.stack);
    process.exit(1);
});
