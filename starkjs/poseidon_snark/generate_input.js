const {buildPoseidonOpt} = require("circomlibjs");
const fs = require('fs');
const {
    BigNumber,
} = require("@ethersproject/bignumber");

const {
    hexlify,
} = require("@ethersproject/bytes");

const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node generate_input.js --i1 1 --i2 2 -o input.json")
  .alias("i1", "input1")
  .alias("i2", "input2")
  .alias("o","output")
  .demand("input1")
  .demand("input2")
  .demand("output")
  .argv


let zero32 = "0x0000000000000000000000000000000000000000000000000000000000000000"
function fmt_b32_input(n) {
    let b = BigNumber.from(n).toHexString()
    console.log(b)
    let pad0_len = (66 - b.length) + 2
    let b32 = zero32.slice(0,pad0_len).concat(b.slice(2))
    return b32 
}

async function gen_input_json()  {
    let poseidon =  await buildPoseidonOpt()

    let input1 = argv.input1
    let input2 = argv.input2

    let inputs = [input1,input2]

    let initialState = 0;
    let out = poseidon(inputs,initialState,1)
    out = hexlify(out)
    out = BigNumber.from(out).toBigInt()
    console.log(out)

    let jsonContext = {
        "a" : input1,
        "b" : input2,
        "c":out
    }
    let jsonStr = JSON.stringify(jsonContext, (key, value) => typeof value === 'bigint' ? value.toString() : value);
    
    fs.writeFile(argv.output, jsonStr, (error) => {
        if (error) {
          console.error(error);
        } else {
          console.log('Data written to file');
        }
      });
}

gen_input_json();