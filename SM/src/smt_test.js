const StateDB = require("./db");
const SMT = require("./smt");
const { getPoseidon } = require('@0xpolygonhermez/zkevm-commonjs');

let root = {
  fe0: 0,
  fe1: 0,
  fe2: 0,
  fe3: 0,
}

let feaKey = {
  fe0: 1,
  fe1: 1,
  fe2: 1,
  fe3: 1,
}

const convertKey = (key) => {
  if (key.startsWith('0x')) {
    return key;
  } else {
    return '0x' + key;
  }
}

async function main() {
  const poseidon = await getPoseidon()
  F = poseidon.F
  let db = new StateDB(F)
  let smt = new SMT(db, poseidon, F)
  let key = feaKey
  // await smt.get(root, key)

  let value = "0x123"
  let setResp = await smt.set(root, key, value)
  console.log("setResp: ", setResp)

  let getResp = await smt.get(setResp.new_root, key)
  console.log("getResp: ", getResp)
}

main().then(() => {

})