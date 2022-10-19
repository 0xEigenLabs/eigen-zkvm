const fs = require("fs");
const path = require("path");
const ethers = require("ethers");
const { performance } = require("perf_hooks");

const {execute} = require("../../src/vm");

const zkcommonjs = require("@0xpolygonhermez/zkevm-commonjs");
const buildPoseidon = require("@0xpolygonhermez/zkevm-commonjs").getPoseidon;
const helpers = require("./helpers");

const argv = require("yargs")
    .usage("node generate-txs.js -t <nTx> -e <flagRun> -r <rom.json> -p <pil.json> -o <flagOnlyExecutor>")
    .help('h')
    .alias("t", "transactions")
    .alias("e", "executor")
    .alias("r", "rom")
    .alias("p", "pil")
    .alias("o", "onlyexecutor")
    .argv;

async function main(){
    // Input paraneters
    helpers.checkParam(argv.transactions, "Number of transactions");

    const nTxs = Number(argv.transactions);
    const flagExecutor = argv.executor === "true" ?  true : false;
    const flagOnlyExecutor = argv.onlyexecutor === "true" ?  true : false;

    let romFile;
    let pilJsonFile;

    if (flagExecutor){
        helpers.checkParam(argv.rom, "Rom file");
        helpers.checkParam(argv.pil, "Pil json file");

        romFile = argv.rom.trim();
        pilJsonFile = argv.pil.trim();
    }

    if (flagOnlyExecutor !== true){
        // Initialize Poseidon
        const poseidon = await buildPoseidon();
        const F = poseidon.F;

        // Define two wallets
        const wallets = {};

        const pvtKeyA = "0x28b2b0318721be8c8339199172cd7cc8f5e273800a35616ec893083a4b32c02e";
        const pvtKeyB = "0x4d27a600dce8c29b7bd080e29a26972377dbb04d7a27d919adbb602bf13cfd23";

        const walletA = new ethers.Wallet(pvtKeyA);
        const walletB = new ethers.Wallet(pvtKeyB);

        wallets[walletA.address] = walletA;
        wallets[walletB.address] = walletB;

        // Init SMT DB
        const arity = 4;
        const db = new zkcommonjs.MemDB(F);
        const smt = new zkcommonjs.SMT(db, arity, poseidon, poseidon.F);

        // Build genesis
        console.log("Building genesis..");

        let infoGenesis = await helpers.buildGenesis(smt,
            [walletA.address, walletB.address],
            ["100000000000000000000", "200000000000000000000"]
        );

        console.log("Finish building genesis\n");
        // Build input executor
        const inputExecutor = {};

        inputExecutor.sequencerAddr = walletA.address;
        inputExecutor.oldStateRoot = "0x" + F.toString(infoGenesis.root, 16).padStart(64, "0");

        // generate transactions
        const value = ethers.utils.parseEther("0.1");
        const gasLimit = 100000;
        const gasPrice = ethers.utils.parseUnits("1", "gwei");
        const chainId = 400;

        const rawTxs = [];
        let nonce = 0;

        console.log("Building txs...");

        for (let i = 0; i < nTxs; i++){
            const from = (i % 2) ? walletB.address : walletA.address;
            const to = (i % 2) ? walletA.address : walletB.address;

            const tx = {
                to,
                nonce,
                value,
                gasLimit,
                gasPrice,
                chainId,
            }

            nonce = (i % 2) ? nonce+1 : nonce;

            const rawTx = await wallets[from].signTransaction(tx);
            rawTxs.push(rawTx);
        }
        console.log("Finish building txs\n");

        let batchL2Data = "0x";
        for (let i = 0; i < rawTxs.length; i++) {
            const customRawTx = zkcommonjs.processorUtils.rawTxToCustomRawTx(rawTxs[i]);
            batchL2Data = batchL2Data.concat(customRawTx.slice(2));
        }
        inputExecutor.batchL2Data = batchL2Data;
        inputExecutor.chainId = chainId;

        const newRoot = await helpers.processTxs(infoGenesis.root, smt, rawTxs, inputExecutor.sequencerAddr);

        inputExecutor.newStateRoot = "0x" + F.toString(newRoot, 16).padStart(64, "0");
        inputExecutor.db = await helpers.getSMT(infoGenesis.root, db, F);
        inputExecutor.globalExitRoot = "0x090bcaf734c4f06c93954a827b45a6e8c67b8e0fd1e0a35a1c5982d6961828f9";
        inputExecutor.newLocalExitRoot = "0x17c04c3760510b48c6012742c540a81aba4bca2f78b9d14bfd2f123e2e53ea3e";
        inputExecutor.oldLocalExitRoot = "0x17c04c3760510b48c6012742c540a81aba4bca2f78b9d14bfd2f123e2e53ea3e";
        inputExecutor.numBatch = 1;
        inputExecutor.timestamp = 1944498032;
        if (!inputExecutor.batchL2Data)
                inputExecutor.batchL2Data = "0x";
        inputExecutor.batchHashData = zkcommonjs.contractUtils.calculateBatchHashData(inputExecutor.batchL2Data, inputExecutor.globalExitRoot);

        // Save executor input
        const fileName = `input-${nTxs}.json`;
        await fs.writeFileSync(path.join(__dirname, fileName), JSON.stringify(inputExecutor, null, 2));
    }


    // run executor js
    if (flagExecutor){
        const fileName = `input-${nTxs}.json`;

        const rom = JSON.parse(await fs.promises.readFile(romFile, "utf8"));
        const pil = JSON.parse(await fs.promises.readFile(pilJsonFile, "utf8"));
        console.log("Start executor JS...");
        const startTime = performance.now();

        const config = {
          inputFile: path.join(__dirname, fileName),
          romFile: rom,
          debug: false,
          debugInfo: { inputName: 'input_executor' },
          unsigned: false,
          execute: false,
          tracer: false
        }

        console.log(config);

        let cmPols = newCommitPolsArray(pil);
        //await execute(input, rom, pil, {N: 2**16});
        await execute(cmPols, config);
        const stopTime = performance.now();
        console.log(`Finish executor JS ==> ${(stopTime - startTime)/1000} s`);
    }
}

main().then(()=> {
    process.exit(0);
}, (err) => {
    console.log(err.message);
    console.log(err.stack);
    process.exit(1);
});
