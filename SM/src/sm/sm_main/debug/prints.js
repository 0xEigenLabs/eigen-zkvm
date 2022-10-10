const { smtUtils, stateUtils } = require("@0xpolygonhermez/zkevm-commonjs");

function sr8to4(F, SR) {
    const r=[];
    r[0] = F.add(SR[0], F.mul(SR[1], F.e("0x100000000")));
    r[1] = F.add(SR[2], F.mul(SR[3], F.e("0x100000000")));
    r[2] = F.add(SR[4], F.mul(SR[5], F.e("0x100000000")));
    r[3] = F.add(SR[6], F.mul(SR[7], F.e("0x100000000")));
    return r;
}

class Prints {

    constructor (ctx, smt){
        this.ctx = ctx;
        this.smt = smt;
    }

    async printAddress(address, stoKeys = [], options = {}) {
        const print = this._shouldPrint(options);

        if (print){
            const root = sr8to4(this.ctx.Fr, this.ctx.SR);
            const state = await stateUtils.getState(address, this.smt, root);

            const hashBytecode = await stateUtils.getContractHashBytecode(address, this.smt, root);
            const hashBytecodeLength = await stateUtils.getContractBytecodeLength(address, this.smt, root);
            const bytecode = this.ctx.input.contractsBytecode[hashBytecode];
            const sto = await stateUtils.getContractStorage(address, this.smt, root, stoKeys);

            const infoAddress = {};
            const storage = {}

            infoAddress.address = address;
            infoAddress.balance = state.balance.toString();
            infoAddress.nonce = state.nonce.toString();
            infoAddress.hashBytecode = hashBytecode;
            infoAddress.hashBytecodeLength = Number(hashBytecodeLength);
            // infoAddress.bytecode = typeof bytecode === "undefined" ? "0x0": bytecode;
            for (const key of Object.keys(sto)) {
                const keyS = "0x" + Scalar.e(key).toString(16).padStart(64, '0');
                storage[keyS] = sto[key].toString(16).length%2 === 0 ? "0x" + sto[key].toString(16) : "0x0" + sto[key].toString(16);
            }
            infoAddress.storage = storage;

            console.log(JSON.stringify(infoAddress, null, 2));
        }
    }

    async printStateRoot(options = {}){
        const print = this._shouldPrint(options);

        if (print){
            const h4 = sr8to4(this.ctx.Fr, this.ctx.SR);
            const root = smtUtils.h4toString(h4);
            console.log("State Root: ", root);
        }
    }

    _shouldPrint(options) {
        let print = false;

        if (Object.keys(options).length === 0){
            print = true;
        } else if (options.hasOwnProperty("step")){
            if (options.step === this.ctx.step){
                print = true;
            }
        } else if (options.line === this.ctx.line && this.ctx.fileName.includes(options.fileName)){
            print = true;
        }

        return print;
    }
}

module.exports = Prints;