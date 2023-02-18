
const buildPoseidon = require("@0xpolygonhermez/zkevm-commonjs").getPoseidon;
const Scalar = require("ffjavascript");
const { existsSync } = require("fs");
const { exit } = require("process");

const LOG_STORAGE_EXECUTOR = false;

function logger (m) {
    if (LOG_STORAGE_EXECUTOR) console.log(m);
}

module.exports.buildConstants = async function (pols) {
}


module.exports.execute = async function (pols, access) {
    const poseidon = await buildPoseidon();
    const fr = poseidon.F;

    let a = 0; // access number, so current access is access[a]

    // We use variables to store the previous values of addr and step. We need this
    // to complete the "empty" evaluations of the polynomials addr and step. We cannot
    // do it with i-1 because we have to "protect" the case that the access list is empty
    let lastAddr = 0n;
    let prevStep = 0n;

    access.sort((a,b) => {
        if (a.address == b.address) {
            return a.pc - b.pc;
        } else {
            return a.address - b.address;
        }
    });

    const degree = pols.addr.length;

    for (let i=0; i<degree; i++) {
        if (a<access.length) {
            pols.addr[i] = BigInt(access[a].address);
            pols.step[i] = BigInt(access[a].pc);
            pols.mOp[i] = 1n;
            pols.mWr[i] = (access[a].bIsWrite) ? 1n : 0n;
            pols.val[0][i] = BigInt(access[a].fe0);
            pols.val[1][i] = BigInt(access[a].fe1);
            pols.val[2][i] = BigInt(access[a].fe2);
            pols.val[3][i] = BigInt(access[a].fe3);
            pols.val[4][i] = BigInt(access[a].fe4);
            pols.val[5][i] = BigInt(access[a].fe5);
            pols.val[6][i] = BigInt(access[a].fe6);
            pols.val[7][i] = BigInt(access[a].fe7);
            pols.lastAccess[i] = ((a < access.length-1) && (access[a].address == access[a+1].address)) ? 0n : 1n;

            logger("Memory executor i="+i+
            " addr="+pols.addr[i].toString(16)+
            " step="+pols.step[i]+
            " mOp="+pols.mOp[i]+
            " mWr="+pols.mWr[i]+
            " val="+fr.toString(pols.val[7][i],16)+
                ":"+fr.toString(pols.val[6][i],16)+
                ":"+fr.toString(pols.val[5][i],16)+
                ":"+fr.toString(pols.val[4][i],16)+
                ":"+fr.toString(pols.val[3][i],16)+
                ":"+fr.toString(pols.val[2][i],16)+
                ":"+fr.toString(pols.val[1][i],16)+
                ":"+fr.toString(pols.val[0][i],16)+
            " lastAccess="+pols.lastAccess[i]);


            lastAddr = pols.addr[i];
            prevStep = pols.step[i];

            // Increment memory access counter
            a++;
        }
        // If access list has been completely consumed
        else
        {
            // We complete the remaining polynomial evaluations. To validate the pil correctly
            // keep last addr incremented +1 and increment the step respect to the previous value
            pols.addr[i] = lastAddr+1n;
            prevStep++;
            pols.step[i] = BigInt(prevStep);
            pols.mOp[i] = 0n;
            pols.mWr[i] = 0n;
            pols.val[0][i] = 0n;
            pols.val[1][i] = 0n;
            pols.val[2][i] = 0n;
            pols.val[3][i] = 0n;
            pols.val[4][i] = 0n;
            pols.val[5][i] = 0n;
            pols.val[6][i] = 0n;
            pols.val[7][i] = 0n;
            // lastAccess = 1 in the last evaluation to ensure ciclical validation
            pols.lastAccess[i] = (i==degree-1) ? 1n : 0n;
        }
    }

    logger("MemoryExecutor successfully processed");
}
