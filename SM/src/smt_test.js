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

async function main() {
  const poseidon = await getPoseidon()
  F = poseidon.F
  let db = new StateDB(F)
  let smt = new SMT(db, poseidon, F)
  // let key = feaKey
  let root = [ 0n, 0n, 0n, 0n ]
  let key = [
    14833827758303204589n,
    15154033943678652181n,
    5489675274157668397n,
    7250342125880245156n
  ]
  // await smt.get(root, key)
  
  let value = 1000000000000000000000n
  let setResp = await smt.set(root, key, value)
  console.log("setResp: ", setResp)

  let getResp = await smt.get(setResp.new_root, key)
  console.log("getResp: ", getResp)
}

main().then(() => {

})