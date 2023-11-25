const StateDB = require("./db");
const { getPoseidon } = require('@0xpolygonhermez/zkevm-commonjs');

let feaKey = {
  fe0: 0,
  fe1: 0,
  fe2: 0,
  fe3: 0,
}

async function main() {
  const poseidon = await getPoseidon()
  F = poseidon.F
  let db = new StateDB(F)

  let key = feaKey
  let value
  // value = await db.getProgram(key)
  // console.log("value: ", value)
  value = "123"
  await db.setProgram(key, value)
  console.log("setProgram finished")
  res = await db.getProgram(key)
  console.log("getProgram finished, res: ", res)
}

main().then(() => {

})