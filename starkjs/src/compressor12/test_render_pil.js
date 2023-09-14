const fs = require("fs");
const path = require("path");
const ejs = require("ejs");

// test render_pil_js.
// node run test_render_pil.js
async function run() {

    const template = await fs.promises.readFile(path.join(__dirname, "compressor12.pil.ejs"), "utf8");
    const obj = {
        nBits: 5,
        nPublics: 5,
    };

    const pilStr = ejs.render(template ,  obj);
    const pilFile = "./render_pil_js.pil";
    await fs.promises.writeFile(pilFile, pilStr, "utf8");

}


// node test_render_pil.js
run().then(()=> {
    process.exit(0);
}, (err) => {
    console.log(err.message);
    console.log(err.stack);
    process.exit(1);
});
