const fs = require("fs");
const path = require("path");
const ethers = require("ethers");
const VM = require('@polygon-hermez/vm').default;
const Common = require('@ethereumjs/common').default;
const { Hardfork } = require('@ethereumjs/common');

const {
  SMT, MemDB, ZkEVMDB, processorUtils, smtUtils, getPoseidon
} = require("@0xpolygonhermez/zkevm-commonjs");

const pathGenesisInput = path.join(__dirname, "../build-genesis/input_gen.json");
const prevExecutorInput = path.join(__dirname, "../build-genesis/input_executor.json");
const pathOutput = path.join(__dirname, "./input_executor.json");
const pathTxs = path.join(__dirname, "./txs.json");
const pathDB = path.join(__dirname, "./../build-genesis/smt.db");
const localdb = JSON.parse(
  fs.readFileSync(pathDB)
)

const dbhandler = require("../dbhandler.js");

const dbproxy = new Proxy(localdb, dbhandler)

async function main(sender) {
  if (sender === undefined) {
    sender = "0x617b3a3528F9cDd6630fd3301B9c8911F7Bf063D";
  }
  const poseidon = await getPoseidon();
  const F = poseidon.F;
  const genesis = require(pathGenesisInput).genesis;
  const prevExecutor = require(prevExecutorInput);

  const walletMap = new Map();
  for (let i = 0; i < genesis.length; i++) {
    const {
      address, pvtKey
    } = genesis[i];
    const newWallet = new ethers.Wallet(pvtKey);
    walletMap.set(address, newWallet);
  }

  if (walletMap.get(sender) == undefined) {
    throw new Error("${sender} is not in walletMap")
  }

  const txs = require(pathTxs);
  let customRawTxs = []
  let initNonce = 1; //FIXME
  for (let tx of txs) {
    initNonce += 1;
    tx.nonce = initNonce;
    console.log(tx)
    let rawTxEthers = await walletMap.get(sender).signTransaction(tx)
    customRawTxs.push(processorUtils.rawTxToCustomRawTx(rawTxEthers))
  }

  const db = new MemDB(F, dbproxy);
  const common = Common.custom({ chainId: prevExecutor.chainID }, { hardfork: Hardfork.Berlin });
  const newVm = new VM({ common });
  const newSmt = new SMT(db, poseidon, poseidon.F);

  const zkEVMDB = await ZkEVMDB.newZkEVM(
    db,
    poseidon,
    smtUtils.stringToH4(prevExecutor.newStateRoot),
    smtUtils.stringToH4(prevExecutor.oldLocalExitRoot),
    genesis,
    newVm,
    newSmt,
    prevExecutor.chainID
  );
  const batch = await zkEVMDB.buildBatch(
    prevExecutor.timestamp,
    prevExecutor.sequencerAddr,
    smtUtils.stringToH4(prevExecutor.globalExitRoot)
  );

  for (let tx of customRawTxs) {
    console.log(tx)
    batch.addRawTx(tx)
  }

  await batch.executeTxs();
  await zkEVMDB.consolidate(batch)

  const starkInput = await batch.getStarkInput();
  delete starkInput.inputHash;
  delete starkInput.batchHashData;

  const updatedAccounts = batch.getUpdatedAccountsBatch();
  const newLeafs = {};
  for (const item in updatedAccounts) {
    const address = item;
    const account = updatedAccounts[address];
    newLeafs[address] = {};

    newLeafs[address].balance = account.balance.toString();
    newLeafs[address].nonce = account.nonce.toString();

    const storage = await zkEVMDB.dumpStorage(address);
    const hashBytecode = await zkEVMDB.getHashBytecode(address);
    newLeafs[address].storage = storage;
    newLeafs[address].hashBytecode = hashBytecode;
  }
  //generateData.expectedLeafs = newLeafs;
  console.log("Done")
  fs.writeFileSync(pathOutput, JSON.stringify(starkInput, null, 2));
  // de-comment
  //fs.writeFileSync(pathInput, JSON.stringify(generateData, null, 2));
  fs.writeFileSync(pathDB, JSON.stringify(dbproxy, null, 2));
}

main().then(() => {
  console.log("gen tx done")
})
