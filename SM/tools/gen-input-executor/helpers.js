const { Scalar } = require("ffjavascript");
const ethers = require("ethers");

const smtKeyUtils = require("@0xpolygonhermez/zkevm-commonjs").smtUtils;
const stateUtils = require("@0xpolygonhermez/zkevm-commonjs").stateUtils;

function checkParam(param, paramStr){
    if (typeof param === "undefined"){
        console.error(`option "${paramStr}" not set`);
        process.exit(1);
    }
}

async function buildGenesis(smt, addresses, balances){
    if (addresses.length !== balances.length){
        throw new Error("Addresses and balances does not match length");
    }

    const F = smt.F;
    // keys object
    const keys = {};

    // initial root
    let root = F.zero;

    for (let i = 0; i < addresses.length; i++){
        const address = addresses[i];
        const balance = balances[i];

        const keyBalance = await smtKeyUtils.keyEthAddrBalance(address, smt.arity);
        const keyNonce = await smtKeyUtils.keyEthAddrNonce(address, smt.arity);
        let res = await smt.set(root, keyBalance, Scalar.e(balance));
        root = res.newRoot;

        // info keys
        keys[F.toString(keyBalance, 16).padStart(64, '0')] = Scalar.e(balance).toString(16).padStart(64, '0');
        keys[F.toString(keyNonce, 16).padStart(64, '0')] = Scalar.e("0").toString(16).padStart(64, '0');
    }

    return { root, keys };
}

async function processTxs(oldRoot, smt, txs, seqAddr){
    let root = oldRoot;

    console.log("Processing txs...");

    for (let i = 0; i < txs.length; i++){
        console.log(`       ${i} out of ${txs.length - 1}`);
        const rawTx = txs[i];

        const txFields = ethers.utils.RLP.decode(rawTx);

        const txDecoded = {
            nonce: txFields[0],
            gasPrice: txFields[1],
            gasLimit: txFields[2],
            to: txFields[3],
            value: txFields[4],
            data: txFields[5],
            v: txFields[6],
            r: txFields[7],
            s: txFields[8],
        };

        const sign = Number(!(txDecoded.v & 1));
        const chainId = Math.floor((Number(txDecoded.v) - 35) / 2);

        const e = [
            txDecoded.nonce,
            txDecoded.gasPrice,
            txDecoded.gasLimit,
            txDecoded.to,
            txDecoded.value,
            txDecoded.data,
            ethers.utils.hexlify(chainId),
            "0x",
            "0x"
        ];

        const signData = ethers.utils.RLP.encode(e);
        const digest = ethers.utils.keccak256(signData);

        const fromAddr = ethers.utils.recoverAddress(digest, {
            r: txDecoded.r,
            s: txDecoded.s,
            v: sign + 27
        });

        // gas costs
        const totalGasCost = Scalar.mul(Scalar.e(txDecoded.gasLimit), Scalar.e(txDecoded.gasPrice));
        const totalCost = Scalar.add(totalGasCost, Scalar.e(txDecoded.value));

        // get states from and to
        const oldStateFrom = await stateUtils.getState(fromAddr, smt, root);
        const oldStateTo = await stateUtils.getState(txDecoded.to, smt, root);

        const newStateFrom = Object.assign({}, oldStateFrom);
        const newStateTo = Object.assign({}, oldStateTo);

        newStateFrom.nonce = Scalar.add(newStateFrom.nonce, 1);
        newStateFrom.balance = Scalar.sub(newStateFrom.balance, totalCost);

        // hardcoded gas used for an ethereum tx: 21000
        const gasUsed = Scalar.e(21000);
        const refundGas = Scalar.sub(totalGasCost, Scalar.mul(gasUsed, txDecoded.gasPrice));
        newStateFrom.balance = Scalar.add(newStateFrom.balance, refundGas);

        newStateTo.balance = Scalar.add(newStateTo.balance, txDecoded.value);

        let tmpRoot = await stateUtils.setAccountState(fromAddr, smt, root, newStateFrom.balance, newStateFrom.nonce);
        tmpRoot = await stateUtils.setAccountState(txDecoded.to, smt, tmpRoot, newStateTo.balance, newStateTo.nonce);

        // pay fees
        const oldStateSeq = await stateUtils.getState(seqAddr, smt, tmpRoot);
        const newStateSeq = Object.assign({}, oldStateSeq);
        const feesCollected = Scalar.mul(gasUsed, txDecoded.gasPrice);
        newStateSeq.balance = Scalar.add(oldStateSeq.balance, feesCollected);

        tmpRoot = await stateUtils.setAccountState(seqAddr, smt, tmpRoot, newStateSeq.balance, newStateSeq.nonce);
        root = tmpRoot;
    }
    console.log("Finish processing txs\n");
    return root;
}

async function getSMT(root, db, F) {
    const smt = await _getSMT(root, db, F, {});
    //Reverse json object to have root at the top
    const arr = Object.keys(smt).map((key) => [key, smt[key]]);
    return arr.reverse().reduce((acc, curr) => {
        acc[curr[0]] = curr[1];
        return acc;
      }, {})
}

async function _getSMT(root, db, F, res = {}) {

    const sibilings = await db.getSmtNode(root);
    const value = [];
    //Reversed to have the root as the first key
    for(const  val of sibilings) {
        value.push(F.toString(val, 16).padStart(64, "0"));
        if(F.eq(sibilings[0], F.one) || F.isZero(val)) {
            continue;
        }
        await _getSMT(val, db, F, res);
    }

    res[F.toString(root, 16).padStart(64, "0")] = value;
    return res;
}

module.exports = {
    checkParam,
    buildGenesis,
    processTxs,
    getSMT
};
