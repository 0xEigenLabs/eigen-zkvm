const fs = require('fs');

const { scalar2fea } = require("@0xpolygonhermez/zkevm-commonjs").smtUtils;
const buildPoseidon = require("@0xpolygonhermez/zkevm-commonjs").getPoseidon;

const { isLogging, logger, fea42String, scalar2fea4, fea4IsEq }  = require("./sm_storage_utils.js");
const SmtActionContext = require("./smt_action_context.js");
const { StorageRomLine } = require("./sm_storage_rom.js");
const StorageRom = require("./sm_storage_rom.js").StorageRom;

module.exports.buildConstants = async function (pols) {
    const poseidon = await buildPoseidon();
    const fr = poseidon.F;

    // Init rom from file
    const rawdata = fs.readFileSync("testvectors/storage_sm_rom.json");
    const j = JSON.parse(rawdata);
    rom = new StorageRom;
    rom.load(j);

    const polSize = pols.rLine.length;

    for (let i=0; i<polSize; i++) {
        // TODO: REVIEW Jordi
        const romLine = i % rom.line.length;
        const l = rom.line[romLine];

        pols.rHash[i] = l.iHash ? BigInt(l.iHash) : 0n;
        pols.rHashType[i] = l.iHashType ? BigInt(l.iHashType) : 0n;
        pols.rLatchGet[i] = l.iLatchGet ? BigInt(l.iLatchGet) : 0n;
        pols.rLatchSet[i] = l.iLatchSet ? BigInt(l.iLatchSet) : 0n;
        pols.rClimbRkey[i] = l.iClimbRkey ? BigInt(l.iClimbRkey) : 0n;
        pols.rClimbSiblingRkey[i] = l.iClimbSiblingRkey ? BigInt(l.iClimbSiblingRkey) : 0n;
        pols.rClimbSiblingRkeyN[i] = l.iClimbSiblingRkeyN ? BigInt(l.iClimbSiblingRkeyN) : 0n;
        pols.rRotateLevel[i] = l.iRotateLevel ? BigInt(l.iRotateLevel) : 0n;
        pols.rJmpz[i] = l.iJmpz ? BigInt(l.iJmpz) : 0n;
        pols.rJmp[i] = l.iJmp ? BigInt(l.iJmp) : 0n;
        let consFea4;
        if (l.CONST) {
            consFea4 = scalar2fea4(fr,BigInt(l.CONST));
        } else {
            consFea4 = [fr.zero, fr.zero, fr.zero, fr.zero];
        }
        pols.rConst0[i] = consFea4[0];
        pols.rConst1[i] = consFea4[1];
        pols.rConst2[i] = consFea4[2];
        pols.rConst3[i] = consFea4[3];

        pols.rAddress[i] = l.address ? BigInt(l.address) : 0n;
        pols.rLine[i] = BigInt(romLine);

        pols.rInFree[i] = l.inFREE ? BigInt(l.inFREE) : 0n;
        pols.rInNewRoot[i] = l.inNEW_ROOT ? BigInt(l.inNEW_ROOT):0n;
        pols.rInOldRoot[i] = l.inOLD_ROOT ? BigInt(l.inOLD_ROOT):0n;
        pols.rInRkey[i] = l.inRKEY ? BigInt(l.inRKEY):0n;
        pols.rInRkeyBit[i] = l.inRKEY_BIT ? BigInt(l.inRKEY_BIT):0n;
        pols.rInSiblingRkey[i] = l.inSIBLING_RKEY ? BigInt(l.inSIBLING_RKEY):0n;
        pols.rInSiblingValueHash[i] = l.inSIBLING_VALUE_HASH ? BigInt(l.inSIBLING_VALUE_HASH):0n;

        pols.rSetHashLeft[i] = l.setHASH_LEFT ? BigInt(l.setHASH_LEFT):0n;
        pols.rSetHashRight[i] = l.setHASH_RIGHT ? BigInt(l.setHASH_RIGHT):0n;
        pols.rSetLevel[i] = l.setLEVEL ? BigInt(l.setLEVEL):0n;
        pols.rSetNewRoot[i] = l.setNEW_ROOT ? BigInt(l.setNEW_ROOT):0n;
        pols.rSetOldRoot[i] = l.setOLD_ROOT ? BigInt(l.setOLD_ROOT):0n;
        pols.rSetRkey[i] = l.setRKEY ? BigInt(l.setRKEY):0n;
        pols.rSetRkeyBit[i] = l.setRKEY_BIT ? BigInt(l.setRKEY_BIT):0n;
        pols.rSetSiblingRkey[i] = l.setSIBLING_RKEY ? BigInt(l.setSIBLING_RKEY):0n;
        pols.rSetSiblingValueHash[i] = l.setSIBLING_VALUE_HASH ? BigInt(l.setSIBLING_VALUE_HASH):0n;
        pols.rSetValueHigh[i] = l.setVALUE_HIGH ? BigInt(l.setVALUE_HIGH):0n;
        pols.rSetValueLow[i] = l.setVALUE_LOW ? BigInt(l.setVALUE_LOW):0n;
    }
}

module.exports.execute = async function (pols, action) {
    const polSize = pols.pc.length;

    const poseidon = await buildPoseidon();
    const fr = poseidon.F;
    const POSEIDONG_PERMUTATION3_ID = 3;

    // Init rom from file
    const rawdata = fs.readFileSync("testvectors/storage_sm_rom.json");
    const j = JSON.parse(rawdata);
    rom = new StorageRom;
    rom.load(j);

    const required = {PoseidonG: []};

    initPols (pols, polSize);

    let l=0; // rom line number, so current line is rom.line[l]
    let a=0; // action number, so current action is action[a]
    let actionListEmpty = (action.length==0); // becomes true when we run out of actions

    logger("actionListEmpty="+actionListEmpty);

    const ctx = new SmtActionContext(isLogging);

    if (!actionListEmpty) {
        ctx.init (fr, action[a]);
    }
    let prevlineId
    for (let i=0; i<polSize; i++) {

        // op is the internal register, reset to 0 at every evaluation
        let op = [fr.zero, fr.zero, fr.zero, fr.zero];

        // Current rom line is set by the program counter of this evaluation
        l = pols.pc[i];

        // Set the next evaluation index, which will be 0 when we reach the last evaluation
        let nexti = (i+1)%polSize;

        if (isLogging) {
            if (rom.line[l].funcName!="isAlmostEndPolynomial") {
                rom.line[l].print(l);
            }
        }

        /*************/
        /* Selectors */
        /*************/

        // When the rom assembler code calls inFREE, it specifies the requested input data
        // using an operation + function name string couple

        if (rom.line[l].inFREE)
        {
            if (rom.line[l].op == "functionCall")
            {
                /* Possible values of mode when action is SMT Set:
                    - update -> update existing value
                    - insertFound -> insert with found key; found a leaf node with a common set of key bits
                    - insertNotFound -> insert with no found key
                    - deleteFound -> delete with found key
                    - deleteNotFound -> delete with no found key
                    - deleteLast -> delete the last node, so root becomes 0
                    - zeroToZero -> value was zero and remains zero
                */
                if (rom.line[l].funcName == "isSetUpdate")
                {
                    if (!actionListEmpty && action[a].bIsSet &&
                        action[a].setResult.mode == "update")
                    {
                        op[0] = fr.one;
                        logger ("StorageExecutor isUpdate returns " + fea42String(fr, op));
                    }
                }
                else if (rom.line[l].funcName == "isSetInsertFound")
                {
                    if (!actionListEmpty && action[a].bIsSet &&
                        action[a].setResult.mode == "insertFound")
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isInsertFound returns " + fea42String(fr, op));
                    }
                }
                else if (rom.line[l].funcName == "isSetInsertNotFound")
                {
                    if (!actionListEmpty && action[a].bIsSet &&
                        action[a].setResult.mode == "insertNotFound")
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isInsertNotFound returns " + fea42String(fr, op));
                    }
                }
                else if (rom.line[l].funcName == "isSetDeleteLast")
                {
                    if (!actionListEmpty && action[a].bIsSet &&
                        action[a].setResult.mode == "deleteLast")
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isDeleteLast returns " + fea42String(fr, op));
                    }
                }
                else if (rom.line[l].funcName == "isSetDeleteFound")
                {
                    if (!actionListEmpty && action[a].bIsSet &&
                        action[a].setResult.mode == "deleteFound")
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isSetDeleteFound returns " + fea42String(fr, op));
                    }
                }
                else if (rom.line[l].funcName == "isSetDeleteNotFound")
                {
                    if (!actionListEmpty && action[a].bIsSet &&
                        action[a].setResult.mode == "deleteNotFound")
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isSetDeleteNotFound returns " + fea42String(fr, op));
                    }
                }
                else if (rom.line[l].funcName == "isSetZeroToZero")
                {
                    if (!actionListEmpty && action[a].bIsSet &&
                        action[a].setResult.mode == "zeroToZero")
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isZeroToZero returns " + fea42String(fr, op));
                    }
                }

                // The SMT action can be a final leaf (isOld0 = true)
                else if (rom.line[l].funcName=="GetIsOld0")
                {
                    if (!actionListEmpty && (action[a].bIsSet ? action[a].setResult.isOld0 : action[a].getResult.isOld0))
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isOld0 returns " + fea42String(fr, op));
                    }
                }
                // The SMT action can be a get, which can return a zero value (key not found) or a non-zero value
                else if (rom.line[l].funcName=="isGet")
                {
                    if (!actionListEmpty && !action[a].bIsSet)
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isGet returns " + fea42String(fr, op));
                    }
                }

                // Get the remaining key, i.e. the key after removing the bits used in the tree node navigation
                else if (rom.line[l].funcName=="GetRkey")
                {
                    op[0] = ctx.rkey[0];
                    op[1] = ctx.rkey[1];
                    op[2] = ctx.rkey[2];
                    op[3] = ctx.rkey[3];

                    logger("StorageExecutor GetRkey returns " + fea42String(fr, op));
                }

                // Get the sibling remaining key, i.e. the part that is not common to the value key
                else if (rom.line[l].funcName=="GetSiblingRkey")
                {
                    op[0] = ctx.siblingRkey[0];
                    op[1] = ctx.siblingRkey[1];
                    op[2] = ctx.siblingRkey[2];
                    op[3] = ctx.siblingRkey[3];

                    logger("StorageExecutor GetSiblingRkey returns " + fea42String(fr, op));
                }

                // Get the sibling hash, obtained from the siblings array of the current level,
                // taking into account that the sibling bit is the opposite (1-x) of the value bit
                else if (rom.line[l].funcName=="GetSiblingHash")
                {
                    if (action[a].bIsSet)
                    {
                        op[0] = action[a].setResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n];
                        op[1] = action[a].setResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n+1n];
                        op[2] = action[a].setResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n+2n];
                        op[3] = action[a].setResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n+3n];
                    }
                    else
                    {
                        op[0] = action[a].getResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n];
                        op[1] = action[a].getResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n+1n];
                        op[2] = action[a].getResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n+2n];
                        op[3] = action[a].getResult.siblings[ctx.currentLevel][(1n-ctx.bits[ctx.currentLevel])*4n+3n];
                    }

                    logger("StorageExecutor GetSiblingHash returns " + fea42String(fr, op));
                }

                // Value is an u256 split in 8 u32 chuncks, each one stored in the lower 32 bits of an u63 field element
                // u63 means that it is not an u64, since some of the possible values are lost due to the prime effect

                // Get the lower 4 field elements of the value
                else if (rom.line[l].funcName=="GetValueLow")
                {
                    let fea = scalar2fea(fr, action[a].bIsSet ? action[a].setResult.newValue : action[a].getResult.value);
                    op[0] = fea[0];
                    op[1] = fea[1];
                    op[2] = fea[2];
                    op[3] = fea[3];

                    logger("StorageExecutor GetValueLow returns " + fea42String(fr, op));
                }

                // Get the higher 4 field elements of the value
                else if (rom.line[l].funcName=="GetValueHigh")
                {
                    let fea = scalar2fea(fr, action[a].bIsSet ? action[a].setResult.newValue : action[a].getResult.value);
                    op[0] = fea[4];
                    op[1] = fea[5];
                    op[2] = fea[6];
                    op[3] = fea[7];

                    logger("StorageExecutor GetValueHigh returns " + fea42String(fr, op));
                }

                // Get the lower 4 field elements of the sibling value
                else if (rom.line[l].funcName=="GetSiblingValueLow")
                {
                    let fea = scalar2fea(fr, action[a].bIsSet ? action[a].setResult.insValue : action[a].getResult.insValue);
                    op[0] = fea[0];
                    op[1] = fea[1];
                    op[2] = fea[2];
                    op[3] = fea[3];

                    logger("StorageExecutor GetSiblingValueLow returns " + fea42String(fr, op));
                }

                // Get the higher 4 field elements of the sibling value
                else if (rom.line[l].funcName=="GetSiblingValueHigh")
                {
                    let fea = scalar2fea(fr, action[a].bIsSet ? action[a].setResult.insValue : action[a].getResult.insValue);
                    op[0] = fea[4];
                    op[1] = fea[5];
                    op[2] = fea[6];
                    op[3] = fea[7];

                    logger("StorageExecutor GetSiblingValueHigh returns " + fea42String(fr, op));
                }

                // Get the lower 4 field elements of the old value
                else if (rom.line[l].funcName=="GetOldValueLow")
                {
                    // This call only makes sense then this is an SMT set
                    if (!action[a].bIsSet)
                    {
                        console.error("Error: StorageExecutor() GetOldValueLow called in an SMT get action");
                        process.exit(-1);
                    }

                    logger("StorageExecutor action[a].setResult.oldValue = "+action[a].setResult.oldValue);
                    // Convert the oldValue scalar to an 8 field elements array
                    fea = scalar2fea(fr, action[a].setResult.oldValue);

                    // Take the lower 4 field elements
                    op[0] = fea[0];
                    op[1] = fea[1];
                    op[2] = fea[2];
                    op[3] = fea[3];

                    logger("StorageExecutor GetOldValueLow returns " + fea42String(fr, op));
                }

                // Get the higher 4 field elements of the old value
                else if (rom.line[l].funcName=="GetOldValueHigh")
                {
                    // This call only makes sense then this is an SMT set
                    if (!action[a].bIsSet)
                    {
                        console.error("Error: StorageExecutor() GetOldValueLow called in an SMT get action");
                        process.exit(-1);
                    }

                    // Convert the oldValue scalar to an 8 field elements array
                    fea=scalar2fea(fr, action[a].setResult.oldValue);

                    // Take the higher 4 field elements
                    op[0] = fea[4];
                    op[1] = fea[5];
                    op[2] = fea[6];
                    op[3] = fea[7];

                    logger("StorageExecutor GetOldValueHigh returns " + fea42String(fr, op));
                }

                // Get the level bit, i.e. the bit x (specified by the parameter) of the level number
                else if (rom.line[l].funcName=="GetLevelBit")
                {
                    // Check that we have the one single parameter: the bit number
                    if (rom.line[l].params.length!=1)
                    {
                        console.error("Error: StorageExecutor() called with GetLevelBit but wrong number of parameters=" + rom.line[l].params.length);
                        process.exit(-1);
                    }

                    // Get the bit parameter
                    let bit = rom.line[l].params[0];

                    // Check that the bit is either 0 or 1
                    if (bit!=0 && bit!=1)
                    {
                        console.error("Error: StorageExecutor() called with GetLevelBit but wrong bit=" + bit );
                        process.exit(-1);
                    }

                    // Set the bit in op[0]
                    if ( ( ctx.level & (1<<bit) ) != 0)
                    {
                        op[0] = fr.one;
                    }

                    logger("StorageExecutor GetLevelBit(" + bit + ") returns " + fea42String(fr, op));
                }

                // Returns 0 if we reached the top of the tree, i.e. if the current level is 0
                else if (rom.line[l].funcName=="GetTopTree")
                {
                    // Return 0 only if we reached the end of the tree, i.e. if the current level is 0
                    if (ctx.currentLevel > 0)
                    {
                        op[0] = fr.one;
                    }

                    logger("StorageExecutor GetTopTree returns " + fea42String(fr, op));
                }

                // Returns 0 if we reached the top of the branch, i.e. if the level matches the siblings size
                else if (rom.line[l].funcName=="GetTopOfBranch")
                {
                    // If we have consumed enough key bits to reach the deepest level of the siblings array, then we are at the top of the branch and we can start climing the tree
                    let siblingsSize = action[a].bIsSet ? action[a].setResult.siblings.length : action[a].getResult.siblings.length;
                    if (ctx.currentLevel > siblingsSize )
                    {
                        op[0] = fr.one;
                    }

                    logger("StorageExecutor GetTopOfBranch returns " + fea42String(fr, op));
                }

                // Get the next key bit
                // This call decrements automatically the current level
                else if (rom.line[l].funcName=="GetNextKeyBit")
                {
                    // Decrease current level
                    ctx.currentLevel--;
                    if (ctx.currentLevel<0)
                    {
                        console.error("Error: StorageExecutor.execute() GetNextKeyBit() found ctx.currentLevel<0");
                        process.exit(-1);
                    }

                    // Get the key bit corresponding to the current level
                    op[0] = ctx.bits[ctx.currentLevel];

                    logger("StorageExecutor GetNextKeyBit returns " + fea42String(fr, op));
                }

                // Return 1 if we completed all evaluations, except one
                else if (rom.line[l].funcName=="isAlmostEndPolynomial")
                {
                    // Return one if this is the one before the last evaluation of the polynomials
                    if (i == (polSize-2))
                    {
                        op[0] = fr.one;
                        logger("StorageExecutor isEndPolynomial returns " + fea42String(fr,op));
                    }
                }
                else
                {
                    logger("Error: StorageExecutor() unknown funcName:" + rom.line[l].funcName);
                    console.log(rom.line[l].funcName);
                    process.exit(-1);
                }
            }

            else if (rom.line[l].op=="")
            {
                // Ignore; this is just to report a list of setters
            }
            else
            {
                // Any other value is an unexpected value
                console.error("Error: StorageExecutor() unknown op:" + rom.line[l].op);
                process.exit(-1);
            }

            if (!fr.isZero(op[0])) pols.free0[i] = op[0];
            if (!fr.isZero(op[1])) pols.free1[i] = op[1];
            if (!fr.isZero(op[2])) pols.free2[i] = op[2];
            if (!fr.isZero(op[3])) pols.free3[i] = op[3];

            // Mark the inFREE register as 1
            pols.inFree[i] = 1n;
        }

        // If a constant is provided, set op to the constant
        if (rom.line[l].CONST!="")
        {
            let constScalar = BigInt(rom.line[l].CONST);

            op = scalar2fea4 (fr, constScalar);

            pols.iConst0[i] = op[0];
            pols.iConst1[i] = op[1];
            pols.iConst2[i] = op[2];
            pols.iConst3[i] = op[3];
        }

        // If inOLD_ROOT then op=OLD_ROOT
        if (rom.line[l].inOLD_ROOT)
        {
            op[0] = pols.oldRoot0[i];
            op[1] = pols.oldRoot1[i];
            op[2] = pols.oldRoot2[i];
            op[3] = pols.oldRoot3[i];
            pols.inOldRoot[i] = 1n;
        }

        // If inNEW_ROOT then op=NEW_ROOT
        if (rom.line[l].inNEW_ROOT)
        {
            op[0] = pols.newRoot0[i];
            op[1] = pols.newRoot1[i];
            op[2] = pols.newRoot2[i];
            op[3] = pols.newRoot3[i];
            pols.inNewRoot[i] = 1n;
        }

        // If inRKEY_BIT then op=RKEY_BIT
        if (rom.line[l].inRKEY_BIT)
        {
            op[0] = pols.rkeyBit[i];
            op[1] = fr.zero;
            op[2] = fr.zero;
            op[3] = fr.zero;
            pols.inRkeyBit[i] = 1n;
        }

        // If inVALUE_LOW then op=VALUE_LOW
        if (rom.line[l].inVALUE_LOW)
        {
            op[0] = pols.valueLow0[i];
            op[1] = pols.valueLow1[i];
            op[2] = pols.valueLow2[i];
            op[3] = pols.valueLow3[i];
            pols.inValueLow[i] = 1n;
        }

        // If inVALUE_HIGH then op=VALUE_HIGH
        if (rom.line[l].inVALUE_HIGH)
        {
            op[0] = pols.valueHigh0[i];
            op[1] = pols.valueHigh1[i];
            op[2] = pols.valueHigh2[i];
            op[3] = pols.valueHigh3[i];
            pols.inValueHigh[i] = 1n;
        }

        // If inRKEY then op=RKEY
        if (rom.line[l].inRKEY)
        {
            op[0] = pols.rkey0[i];
            op[1] = pols.rkey1[i];
            op[2] = pols.rkey2[i];
            op[3] = pols.rkey3[i];
            pols.inRkey[i] = 1n;
        }

        // If inSIBLING_RKEY then op=SIBLING_RKEY
        if (rom.line[l].inSIBLING_RKEY)
        {
            op[0] = pols.siblingRkey0[i];
            op[1] = pols.siblingRkey1[i];
            op[2] = pols.siblingRkey2[i];
            op[3] = pols.siblingRkey3[i];
            pols.inSiblingRkey[i] = 1n;
        }

        // If inSIBLING_VALUE_HASH then op=SIBLING_VALUE_HASH
        if (rom.line[l].inSIBLING_VALUE_HASH)
        {
            op[0] = pols.siblingValueHash0[i];
            op[1] = pols.siblingValueHash1[i];
            op[2] = pols.siblingValueHash2[i];
            op[3] = pols.siblingValueHash3[i];
            pols.inSiblingValueHash[i] = 1n;
        }

        // If inROTL_VH then op=rotate_left(VALUE_HIGH)
        if (rom.line[l].inROTL_VH)
        {
            op[0] = pols.valueHigh3[i];
            op[1] = pols.valueHigh0[i];
            op[2] = pols.valueHigh1[i];
            op[3] = pols.valueHigh2[i];
            pols.inRotlVh[i] = 1n;
        }

        /****************/
        /* Instructions */
        /****************/

        // JMPZ: Jump if OP==0
        if (rom.line[l].iJmpz)
        {
            if (fr.isZero(op[0]))
            {
                pols.pc[nexti] = BigInt(rom.line[l].address);
            }
            else
            {
                pols.pc[nexti] = pols.pc[i] + 1n;
            }
            pols.iAddress[i] = BigInt(rom.line[l].address);
            pols.iJmpz[i] = 1n;
        }
        // JMP: Jump always
        else if (rom.line[l].iJmp)
        {
            pols.pc[nexti] = BigInt(rom.line[l].address);
            pols.iAddress[i] = BigInt(rom.line[l].address);
            pols.iJmp[i] = 1n;
        }
        // If not any jump, then simply increment program counter
        else
        {
            pols.pc[nexti] = pols.pc[i] + 1n;
        }

        // Rotate level registers values, from higher to lower
        if (rom.line[l].iRotateLevel)
        {
            pols.level0[nexti] = pols.level1[i];
            pols.level1[nexti] = pols.level2[i];
            pols.level2[nexti] = pols.level3[i];
            pols.level3[nexti] = pols.level0[i];
            pols.iRotateLevel[i] = 1n;

            logger("StorageExecutor iRotateLevel level[3:2:1:0]=" + pols.level3[nexti] + ":" + pols.level2[nexti] + ":" + pols.level1[nexti] + ":" + pols.level0[nexti]);
        }

        // Hash: op = poseidon.hash(HASH_LEFT + HASH_RIGHT + (0 or 1, depending on iHashType))
        if (rom.line[l].iHash)
        {
            // Prepare the data to hash: HASH_LEFT + HASH_RIGHT + 0 or 1, depending on iHashType
            let fea = [];
            fea[0] = pols.hashLeft0[i];
            fea[1] = pols.hashLeft1[i];
            fea[2] = pols.hashLeft2[i];
            fea[3] = pols.hashLeft3[i];
            fea[4] = pols.hashRight0[i];
            fea[5] = pols.hashRight1[i];
            fea[6] = pols.hashRight2[i];
            fea[7] = pols.hashRight3[i];
            let cap = [];
            if (rom.line[l].iHashType==0)
            {
                cap[0] = fr.zero;
            }
            else if (rom.line[l].iHashType==1)
            {
                cap[0] = fr.one;
                pols.iHashType[i] = 1n;
            }
            else
            {
                console.error("Error: StorageExecutor:execute() found invalid iHashType=" + rom.line[l].iHashType);
                process.exit(-1);
            }
            cap[1] = fr.zero;
            cap[2] = fr.zero;
            cap[3] = fr.zero;

            // Call poseidon
            let rp = poseidon(fea, cap);

            // Get the calculated hash from the first 4 elements
            pols.free0[i] = rp[0];
            pols.free1[i] = rp[1];
            pols.free2[i] = rp[2];
            pols.free3[i] = rp[3];

            op[0] = fr.add(op[0],fr.mul(BigInt(rom.line[l].inFREE), pols.free0[i]));
            op[1] = fr.add(op[1],fr.mul(BigInt(rom.line[l].inFREE), pols.free1[i]));
            op[2] = fr.add(op[2],fr.mul(BigInt(rom.line[l].inFREE), pols.free2[i]));
            op[3] = fr.add(op[3],fr.mul(BigInt(rom.line[l].inFREE), pols.free3[i]));

            pols.iHash[i] = 1n;

            required.PoseidonG.push([fea[0],fea[1],fea[2],fea[3],fea[4],fea[5],fea[6],fea[7],cap[0],cap[1],cap[2],cap[3],rp[0],rp[1],rp[2],rp[3], POSEIDONG_PERMUTATION3_ID]);

            if (isLogging) {
                let mlog = "StorageExecutor iHash" + rom.line[l].iHashType + " hash=" + fea42String(fr, op) + " value=";
                for (let i=0; i<8; i++) mlog += fr.toString(fea[i],16) + ":";
                for (let i=0; i<4; i++) mlog += fr.toString(cap[i],16) + ":";
                logger(mlog);
            }
        }

        // Climb the remaining key, by injecting the RKEY_BIT in the register specified by LEVEL
        if (rom.line[l].iClimbRkey)
        {
            let bit = pols.rkeyBit[i];
            pols.rkey0[nexti] = pols.rkey0[i];
            pols.rkey1[nexti] = pols.rkey1[i];
            pols.rkey2[nexti] = pols.rkey2[i];
            pols.rkey3[nexti] = pols.rkey3[i];
            if (pols.level0[i] == fr.one)
            {
                pols.rkey0[nexti] = (pols.rkey0[i]<<1n) + bit;
            }
            if (pols.level1[i] == fr.one)
            {
                pols.rkey1[nexti] = (pols.rkey1[i]<<1n) + bit;
            }
            if (pols.level2[i] == fr.one)
            {
                pols.rkey2[nexti] = (pols.rkey2[i]<<1n) + bit;
            }
            if (pols.level3[i] == fr.one)
            {
                pols.rkey3[nexti] = (pols.rkey3[i]<<1n) + bit;
            }
            pols.iClimbRkey[i] = 1n;

            if (isLogging) {
                let fea = [pols.rkey0[nexti], pols.rkey1[nexti], pols.rkey2[nexti], pols.rkey3[nexti]];
                logger("StorageExecutor iClimbRkey sibling bit=" + bit + " rkey=" + fea42String(fr,fea));
            }
        }

        // Climb the sibling remaining key, by injecting the sibling bit in the register specified by LEVEL
        if (rom.line[l].iClimbSiblingRkey)
        {
            if (isLogging) {
                let fea1 = [pols.siblingRkey0[i], pols.siblingRkey1[i], pols.siblingRkey2[i], pols.siblingRkey3[i]];
                logger("StorageExecutor iClimbSiblingRkey before rkey=" + fea42String(fr,fea1));
            }
            let bit = pols.rkeyBit[i];
            pols.siblingRkey0[nexti] = pols.siblingRkey0[i];
            pols.siblingRkey1[nexti] = pols.siblingRkey1[i];
            pols.siblingRkey2[nexti] = pols.siblingRkey2[i];
            pols.siblingRkey3[nexti] = pols.siblingRkey3[i];
            if (pols.level0[i] == fr.one)
            {
                pols.siblingRkey0[nexti] = (pols.siblingRkey0[i]<<1n) + bit;
            }
            if (pols.level1[i] == fr.one)
            {
                pols.siblingRkey1[nexti] = (pols.siblingRkey1[i]<<1n) + bit;
            }
            if (pols.level2[i] == fr.one)
            {
                pols.siblingRkey2[nexti] = (pols.siblingRkey2[i]<<1n) + bit;
            }
            if (pols.level3[i] == fr.one)
            {
                pols.siblingRkey3[nexti] = (pols.siblingRkey3[i]<<1n) + bit;
            }
            pols.iClimbSiblingRkey[i] = 1n;

            let fea = [pols.siblingRkey0[nexti], pols.siblingRkey1[nexti], pols.siblingRkey2[nexti], pols.siblingRkey3[nexti]];
            logger("StorageExecutor iClimbSiblingRkey after sibling bit=" + bit + " rkey=" + fea42String(fr,fea));
        }

        // Climb the sibling remaining key, by injecting the sibling bit in the register specified by LEVEL
        if (rom.line[l].iClimbSiblingRkeyN)
        {
            if (isLogging) {
                let fea1 = [pols.siblingRkey0[i], pols.siblingRkey1[i], pols.siblingRkey2[i], pols.siblingRkey3[i]];
                logger("StorageExecutor iClimbSiblingRkeyN before rkey=" + fea42String(fr,fea1));
            }
            let bit = 1n-pols.rkeyBit[i];
            pols.siblingRkey0[nexti] = pols.siblingRkey0[i];
            pols.siblingRkey1[nexti] = pols.siblingRkey1[i];
            pols.siblingRkey2[nexti] = pols.siblingRkey2[i];
            pols.siblingRkey3[nexti] = pols.siblingRkey3[i];
            if (pols.level0[i] == fr.one)
            {
                pols.siblingRkey0[nexti] = (pols.siblingRkey0[i]<<1n) + bit;
            }
            if (pols.level1[i] == fr.one)
            {
                pols.siblingRkey1[nexti] = (pols.siblingRkey1[i]<<1n) + bit;
            }
            if (pols.level2[i] == fr.one)
            {
                pols.siblingRkey2[nexti] = (pols.siblingRkey2[i]<<1n) + bit;
            }
            if (pols.level3[i] == fr.one)
            {
                pols.siblingRkey3[nexti] = (pols.siblingRkey3[i]<<1n) + bit;
            }
            pols.iClimbSiblingRkeyN[i] = 1n;

            let fea = [pols.siblingRkey0[nexti], pols.siblingRkey1[nexti], pols.siblingRkey2[nexti], pols.siblingRkey3[nexti]];
            logger("StorageExecutor iClimbSiblingRkeyN after sibling bit=" + bit + " rkey=" + fea42String(fr,fea));
        }

        // Latch get: at this point consistency is granted: OLD_ROOT, RKEY (complete key), VALUE_LOW, VALUE_HIGH, LEVEL
        if (rom.line[l].iLatchGet)
        {
            // Check that the current action is an SMT get
            if (action[a].bIsSet)
            {
                console.error("Error: StorageExecutor() LATCH GET found action " + a + " bIsSet=true");
                process.exit(-1);
            }

            // Check that the calculated old root is the same as the provided action root
            let oldRoot = [pols.oldRoot0[i], pols.oldRoot1[i], pols.oldRoot2[i], pols.oldRoot3[i]];
            if ( !fea4IsEq(fr, oldRoot, action[a].getResult.root) )
            {
                console.error("Error: StorageExecutor() LATCH GET found action " + a + " pols.oldRoot=" + fea42String(fr,oldRoot) + " different from action.getResult.root=" + fea42String(fr,action[a].getResult.root));
                process.exit(-1);
            }

            // Check that the calculated complete key is the same as the provided action key
            if ( pols.rkey0[i] != action[a].getResult.key[0] ||
                    pols.rkey1[i] != action[a].getResult.key[1] ||
                    pols.rkey2[i] != action[a].getResult.key[2] ||
                    pols.rkey3[i] != action[a].getResult.key[3] )
            {
                console.error("Error: StorageExecutor() LATCH GET found action " + a + " pols.rkey!=action.getResult.key");
                process.exit(-1);
            }

            // Check that final level state is consistent
            if ( pols.level0[i] != fr.one ||
                    pols.level1[i] != fr.zero ||
                    pols.level2[i] != fr.zero ||
                    pols.level3[i] != fr.zero )
            {
                console.error("Error: StorageExecutor() LATCH GET found action " + a + " wrong level=" + pols.level3[i] + ":" + pols.level2[i] + ":" + pols.level1[i] + ":" + pols.level0[i]);
                process.exit(-1);
            }

            logger("StorageExecutor LATCH GET");

            // Increase action
            a++;

            // In case we run out of actions, report the empty list to consume the rest of evaluations
            if (a>=action.length)
            {
                actionListEmpty = true;
                logger("StorageExecutor LATCH GET detected the end of the action list a=" + a + " i=" + i);
            }
            // Initialize the context for the new action
            else
            {
                ctx.init(fr, action[a]);
            }

            pols.iLatchGet[i] = 1n;
        }

        // Latch set: at this point consistency is granted: OLD_ROOT, NEW_ROOT, RKEY (complete key), VALUE_LOW, VALUE_HIGH, LEVEL
        if (rom.line[l].iLatchSet)
        {
            // Check that the current action is an SMT set
            if (!action[a].bIsSet)
            {
                console.error("Error: StorageExecutor() LATCH SET found action " + a + " bIsSet=false");
                process.exit(-1); //@exit(-1);;
            }

            // Check that the calculated old root is the same as the provided action root
            let oldRoot = [pols.oldRoot0[i], pols.oldRoot1[i], pols.oldRoot2[i], pols.oldRoot3[i]];
            if ( !fea4IsEq(fr, oldRoot, action[a].setResult.oldRoot) )
            {
                let newRoot = [pols.newRoot0[i], pols.newRoot1[i], pols.newRoot2[i], pols.newRoot3[i]];
                console.error("Error: StorageExecutor() LATCH SET found action " + a + " pols.oldRoot=" + fea42String(fr,oldRoot) + " different from action.setResult.oldRoot=" + fea42String(fr,action[a].setResult.oldRoot));
                console.error("Error: StorageExecutor() LATCH SET found action " + a + " pols.newRoot=" + fea42String(fr,newRoot) + " different from action.setResult.newRoot=" + fea42String(fr,action[a].setResult.newRoot));
                process.exit(-1); //@exit(-1);;
            }

            // Check that the calculated old root is the same as the provided action root
            let newRoot = [pols.newRoot0[i], pols.newRoot1[i], pols.newRoot2[i], pols.newRoot3[i]];
            if ( !fea4IsEq(fr, newRoot, action[a].setResult.newRoot) )
            {
                console.error("Error: StorageExecutor() LATCH SET found action " + a + " pols.newRoot=" + fea42String(fr,newRoot) + " different from action.setResult.newRoot=" + fea42String(fr,action[a].setResult.newRoot));
                process.exit(-1);
            }

            // Check that the calculated complete key is the same as the provided action key
            if ( pols.rkey0[i] != action[a].setResult.key[0] ||
                 pols.rkey1[i] != action[a].setResult.key[1] ||
                 pols.rkey2[i] != action[a].setResult.key[2] ||
                 pols.rkey3[i] != action[a].setResult.key[3] )
            {
                console.error("Error: StorageExecutor() LATCH SET found action " + a + " pols.rkey!=action.setResult.key");
                process.exit(-1);
            }

            // Check that final level state is consistent
            if ( pols.level0[i] != fr.one ||
                 pols.level1[i] != fr.zero ||
                 pols.level2[i] != fr.zero ||
                 pols.level3[i] != fr.zero )
            {
                console.error("Error: StorageExecutor() LATCH SET found action " + a + " wrong level=" + pols.level3[i] + ":" + pols.level2[i] + ":" + pols.level1[i] + ":" + pols.level0[i]);
                process.exit(-1);
            }

            logger("StorageExecutor LATCH SET");

            // Increase action
            a++;

            // In case we run out of actions, report the empty list to consume the rest of evaluations
            if (a>=action.length)
            {
                actionListEmpty = true;

                logger("StorageExecutor() LATCH SET detected the end of the action list a=" + a + " i=" + i);
            }
            // Initialize the context for the new action
            else
            {
                ctx.init(fr, action[a]);
            }

            pols.iLatchSet[i] = 1n;
        }

        /***********/
        /* Setters */
        /***********/

        // If setRKEY then RKEY=op
        if (rom.line[l].setRKEY)
        {
            pols.rkey0[nexti] = op[0];
            pols.rkey1[nexti] = op[1];
            pols.rkey2[nexti] = op[2];
            pols.rkey3[nexti] = op[3];
            pols.setRkey[i] = 1n;
        }
        else if (pols.iClimbRkey[i]==0)
        {
            pols.rkey0[nexti] = pols.rkey0[i];
            pols.rkey1[nexti] = pols.rkey1[i];
            pols.rkey2[nexti] = pols.rkey2[i];
            pols.rkey3[nexti] = pols.rkey3[i];

        }

        // If setRKEY_BIT then RKEY_BIT=op
        if (rom.line[l].setRKEY_BIT)
        {
            pols.rkeyBit[nexti] = op[0];
            pols.setRkeyBit[i] = 1n;
        }
        else
        {
            pols.rkeyBit[nexti] = pols.rkeyBit[i];
        }

        // If setVALUE_LOW then VALUE_LOW=op
        if (rom.line[l].setVALUE_LOW)
        {
            pols.valueLow0[nexti] = op[0];
            pols.valueLow1[nexti] = op[1];
            pols.valueLow2[nexti] = op[2];
            pols.valueLow3[nexti] = op[3];
            pols.setValueLow[i] = 1n;
        }
        else
        {
            pols.valueLow0[nexti] = pols.valueLow0[i];
            pols.valueLow1[nexti] = pols.valueLow1[i];
            pols.valueLow2[nexti] = pols.valueLow2[i];
            pols.valueLow3[nexti] = pols.valueLow3[i];
        }

        // If setVALUE_HIGH then VALUE_HIGH=op
        if (rom.line[l].setVALUE_HIGH)
        {
            pols.valueHigh0[nexti] = op[0];
            pols.valueHigh1[nexti] = op[1];
            pols.valueHigh2[nexti] = op[2];
            pols.valueHigh3[nexti] = op[3];
            pols.setValueHigh[i] = 1n;
        }
        else
        {
            pols.valueHigh0[nexti] = pols.valueHigh0[i];
            pols.valueHigh1[nexti] = pols.valueHigh1[i];
            pols.valueHigh2[nexti] = pols.valueHigh2[i];
            pols.valueHigh3[nexti] = pols.valueHigh3[i];
        }

        // If setLEVEL then LEVEL=op
        if (rom.line[l].setLEVEL)
        {
            pols.level0[nexti] = op[0];
            pols.level1[nexti] = op[1];
            pols.level2[nexti] = op[2];
            pols.level3[nexti] = op[3];
            pols.setLevel[i] = 1n;
        }
        else if (pols.iRotateLevel[i]==0)
        {
            pols.level0[nexti] = pols.level0[i];
            pols.level1[nexti] = pols.level1[i];
            pols.level2[nexti] = pols.level2[i];
            pols.level3[nexti] = pols.level3[i];
        }

        // If setOLD_ROOT then OLD_ROOT=op
        if (rom.line[l].setOLD_ROOT)
        {
            pols.oldRoot0[nexti] = op[0];
            pols.oldRoot1[nexti] = op[1];
            pols.oldRoot2[nexti] = op[2];
            pols.oldRoot3[nexti] = op[3];
            pols.setOldRoot[i] = 1n;
        }
        else
        {
            pols.oldRoot0[nexti] = pols.oldRoot0[i];
            pols.oldRoot1[nexti] = pols.oldRoot1[i];
            pols.oldRoot2[nexti] = pols.oldRoot2[i];
            pols.oldRoot3[nexti] = pols.oldRoot3[i];
        }

        // If setNEW_ROOT then NEW_ROOT=op
        if (rom.line[l].setNEW_ROOT)
        {
            pols.newRoot0[nexti] = op[0];
            pols.newRoot1[nexti] = op[1];
            pols.newRoot2[nexti] = op[2];
            pols.newRoot3[nexti] = op[3];
            pols.setNewRoot[i] = 1n;
        }
        else
        {
            pols.newRoot0[nexti] = pols.newRoot0[i];
            pols.newRoot1[nexti] = pols.newRoot1[i];
            pols.newRoot2[nexti] = pols.newRoot2[i];
            pols.newRoot3[nexti] = pols.newRoot3[i];
        }

        // If setHASH_LEFT then HASH_LEFT=op
        if (rom.line[l].setHASH_LEFT)
        {
            pols.hashLeft0[nexti] = op[0];
            pols.hashLeft1[nexti] = op[1];
            pols.hashLeft2[nexti] = op[2];
            pols.hashLeft3[nexti] = op[3];
            pols.setHashLeft[i] = 1n;
        }
        else
        {
            pols.hashLeft0[nexti] = pols.hashLeft0[i];
            pols.hashLeft1[nexti] = pols.hashLeft1[i];
            pols.hashLeft2[nexti] = pols.hashLeft2[i];
            pols.hashLeft3[nexti] = pols.hashLeft3[i];
        }

        // If setHASH_RIGHT then HASH_RIGHT=op
        if (rom.line[l].setHASH_RIGHT)
        {
            pols.hashRight0[nexti] = op[0];
            pols.hashRight1[nexti] = op[1];
            pols.hashRight2[nexti] = op[2];
            pols.hashRight3[nexti] = op[3];
            pols.setHashRight[i] = 1n;
        }
        else
        {
            pols.hashRight0[nexti] = pols.hashRight0[i];
            pols.hashRight1[nexti] = pols.hashRight1[i];
            pols.hashRight2[nexti] = pols.hashRight2[i];
            pols.hashRight3[nexti] = pols.hashRight3[i];
        }

        // If setSIBLING_RKEY then SIBLING_RKEY=op
        if (rom.line[l].setSIBLING_RKEY)
        {
            pols.siblingRkey0[nexti] = op[0];
            pols.siblingRkey1[nexti] = op[1];
            pols.siblingRkey2[nexti] = op[2];
            pols.siblingRkey3[nexti] = op[3];
            pols.setSiblingRkey[i] = 1n;
        }
        else if ((pols.iClimbSiblingRkey[i]==0) && (pols.iClimbSiblingRkeyN[i]==0))
        {
            pols.siblingRkey0[nexti] = pols.siblingRkey0[i];
            pols.siblingRkey1[nexti] = pols.siblingRkey1[i];
            pols.siblingRkey2[nexti] = pols.siblingRkey2[i];
            pols.siblingRkey3[nexti] = pols.siblingRkey3[i];
        }

        // If setSIBLING_VALUE_HASH then SIBLING_VALUE_HASH=op
        if (rom.line[l].setSIBLING_VALUE_HASH)
        {
            pols.siblingValueHash0[nexti] = op[0];
            pols.siblingValueHash1[nexti] = op[1];
            pols.siblingValueHash2[nexti] = op[2];
            pols.siblingValueHash3[nexti] = op[3];
            pols.setSiblingValueHash[i] = 1n;
        }
        else
        {
            pols.siblingValueHash0[nexti] = pols.siblingValueHash0[i];
            pols.siblingValueHash1[nexti] = pols.siblingValueHash1[i];
            pols.siblingValueHash2[nexti] = pols.siblingValueHash2[i];
            pols.siblingValueHash3[nexti] = pols.siblingValueHash3[i];
        }

        if (!fr.isZero(op[0]))
        {
            pols.op0inv[i] = fr.inv(op[0]);
        }


        // Increment counter at every hash, and reset it at every latch
        if (rom.line[l].iHash)
        {
            pols.incCounter[nexti] = pols.incCounter[i] + 1n;
        }
        else if (rom.line[l].iLatchGet || rom.line[l].iLatchSet)
        {
            pols.incCounter[nexti] = 0n;
        }
        else
        {
            pols.incCounter[nexti] = pols.incCounter[i];
        }

        if ((i%1000)==0) logger("StorageExecutor step "+ i +" done");

    }

    logger("StorageExecutor successfully processed " + action.length + " SMT actions");

    return required;
}

function initPols (pols, polSize) {
    for (let i=0; i<polSize; i++) {
        pols.free0[i] = 0n;
        pols.free1[i] = 0n;
        pols.free2[i] = 0n;
        pols.free3[i] = 0n;

        pols.hashLeft0[i] = 0n;
        pols.hashLeft1[i] = 0n;
        pols.hashLeft2[i] = 0n;
        pols.hashLeft3[i] = 0n;

        pols.hashRight0[i] = 0n;
        pols.hashRight1[i] = 0n;
        pols.hashRight2[i] = 0n;
        pols.hashRight3[i] = 0n;

        pols.oldRoot0[i] = 0n;
        pols.oldRoot1[i] = 0n;
        pols.oldRoot2[i] = 0n;
        pols.oldRoot3[i] = 0n;

        pols.newRoot0[i] = 0n;
        pols.newRoot1[i] = 0n;
        pols.newRoot2[i] = 0n;
        pols.newRoot3[i] = 0n;

        pols.valueLow0[i] = 0n;
        pols.valueLow1[i] = 0n;
        pols.valueLow2[i] = 0n;
        pols.valueLow3[i] = 0n;

        pols.valueHigh0[i] = 0n;
        pols.valueHigh1[i] = 0n;
        pols.valueHigh2[i] = 0n;
        pols.valueHigh3[i] = 0n;

        pols.siblingValueHash0[i] = 0n;
        pols.siblingValueHash1[i] = 0n;
        pols.siblingValueHash2[i] = 0n;
        pols.siblingValueHash3[i] = 0n;

        pols.rkey0[i] = 0n;
        pols.rkey1[i] = 0n;
        pols.rkey2[i] = 0n;
        pols.rkey3[i] = 0n;

        pols.siblingRkey0[i] = 0n;
        pols.siblingRkey1[i] = 0n;
        pols.siblingRkey2[i] = 0n;
        pols.siblingRkey3[i] = 0n;

        pols.rkeyBit[i] = 0n;

        pols.level0[i] = 0n;
        pols.level1[i] = 0n;
        pols.level2[i] = 0n;
        pols.level3[i] = 0n;

        pols.pc[i] = 0n;

        pols.inOldRoot[i] = 0n;
        pols.inNewRoot[i] = 0n;
        pols.inValueLow[i] = 0n;
        pols.inValueHigh[i] = 0n;
        pols.inSiblingValueHash[i] = 0n;
        pols.inRkey[i] = 0n;
        pols.inRkeyBit[i] = 0n;
        pols.inSiblingRkey[i] = 0n;
        pols.inFree[i] = 0n;
        pols.inRotlVh[i] = 0n;

        pols.setHashLeft[i] = 0n;
        pols.setHashRight[i] = 0n;
        pols.setOldRoot[i] = 0n;
        pols.setNewRoot[i] = 0n;
        pols.setValueLow[i] = 0n;
        pols.setValueHigh[i] = 0n;
        pols.setSiblingValueHash[i] = 0n;
        pols.setRkey[i] = 0n;
        pols.setSiblingRkey[i] = 0n;
        pols.setRkeyBit[i] = 0n;
        pols.setLevel[i] = 0n;

        pols.iHash[i] = 0n;
        pols.iHashType[i] = 0n;
        pols.iLatchGet[i] = 0n;
        pols.iLatchSet[i] = 0n;
        pols.iClimbRkey[i] = 0n;
        pols.iClimbSiblingRkey[i] = 0n;
        pols.iClimbSiblingRkeyN[i] = 0n;
        pols.iRotateLevel[i] = 0n;
        pols.iJmpz[i] = 0n;
        pols.iJmp[i] = 0n;
        pols.iConst0[i] = 0n;
        pols.iConst1[i] = 0n;
        pols.iConst2[i] = 0n;
        pols.iConst3[i] = 0n;
        pols.iAddress[i] = 0n;

        pols.op0inv[i] = 0n;
        pols.incCounter[i] = 0n;
    }
}
