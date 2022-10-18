const ethers = require("ethers")
const fs = require("fs");
var path = require("path");

function getParamRLP(param){
    let hexParam = Number(param).toString(16);
    if(hexParam.startsWith("0x")) {
        hexParam = (hexParam.length%2 === 0) ? hexParam : "0x0"+hexParam.slice(2);
    } else {
        if(hexParam === "0") hexParam = "0x";
        else hexParam = (hexParam.length%2 === 0) ? "0x"+hexParam : "0x0"+hexParam;
    }
    return hexParam;
}

function generateTx(txJson_) {
  var txJson = path.resolve(txJson_);
  let nonce, gasPrice, gasLimit, to, value, data, v, r, s;
  if (!fs.existsSync(txJson)) {
    console.log("${txJson} not exist")
    return;
  }

  console.log(txJson)
  var txData = require(txJson);

  nonce = txData.nonce;
  gasPrice = txData.gasPrice;
  gasLimit = txData.gasLimit;
  to = txData.to;
  value = txData.value;
  data = txData.data;
  v = txData.v;
  r = txData.r;
  s = txData.s;

  const chainId = (Number(v) - 35) >> 1;
  const messageToHash = [getParamRLP(nonce), getParamRLP(gasPrice), getParamRLP(gasLimit), getParamRLP(to), getParamRLP(value), getParamRLP(data), getParamRLP(chainId), "0x", "0x"]
  const signData = ethers.utils.RLP.encode(messageToHash).slice(2);
  r = r.slice(2).padStart(32*2, 0);
  s = s.slice(2).padStart(32*2, 0);
  const sign = !(Number(v) & 1);
  v = (sign + 27).toString(16).padStart(1*2, '0');
  const calldata = `0x${signData.concat(r).concat(s).concat(v)}`;
  console.log("calldata----------------------------")
  console.log(calldata)
  const signDataDecode = ethers.utils.RLP.decode("0x"+signData);
  console.log("tx----------------------------------")
  console.log("nonce: ", signDataDecode[0])
  console.log("gasPrice: ", signDataDecode[1])
  console.log("gasLimit: ", signDataDecode[2])
  console.log("to: ", signDataDecode[3])
  console.log("value: ", signDataDecode[4])
  console.log("data: ", signDataDecode[5])
  console.log("chainId: ", signDataDecode[6])
  return calldata;
}


const version = require("../package").version;
const argv = require("yargs")
  .version(version)
  .usage("node generate_tx.js -f ../tools/tx-example.json")
  .alias("f", "txJson") //workspace to stash temp and output files
  .demand('txJson')
  .argv;

generateTx(argv.txJson)
