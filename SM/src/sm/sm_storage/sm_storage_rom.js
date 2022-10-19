class StorageRomLine
{
    constructor ()
    {
        // Mandatory fields
        this.line = 0;
        this.fileName = "";

        // Instructions
        this.iJmpz = false;
        this.iJmp = false;
        this.iRotateLevel = false;
        this.iHash = false;
        this.iHashType = 0;
        this.iClimbRkey = false;
        this.iClimbSiblingRkey = false;
        this.iClimbSiblingRkeyN = false;
        this.iLatchGet = false;
        this.iLatchSet = false;

        // Selectors
        this.inFREE = false;
        this.inOLD_ROOT = false;
        this.inNEW_ROOT = false;
        this.inRKEY_BIT = false;
        this.inVALUE_LOW = false;
        this.inVALUE_HIGH = false;
        this.inRKEY = false;
        this.inSIBLING_RKEY = false;
        this.inSIBLING_VALUE_HASH = false;

        // Setters
        this.setRKEY = false;
        this.setRKEY_BIT = false;
        this.setVALUE_LOW = false;
        this.setVALUE_HIGH = false;
        this.setLEVEL = false;
        this.setOLD_ROOT = false;
        this.setNEW_ROOT = false;
        this.setHASH_LEFT = false;
        this.setHASH_RIGHT = false;
        this.setSIBLING_RKEY = false;
        this.setSIBLING_VALUE_HASH = false;

        // Jump parameters
        this.addressLabel = "";
        this.address = 0;

        // inFREE parameters
        this.op = "";
        this.funcName = "";
        this.params = []; //vector<uint64_t>

        // Constant
        this.CONST = "";
    }

    print(l)
    {
        let found = this.fileName.lastIndexOf("/");
        if (found==-1) found = this.fileName.lastIndexOf("\\");
        let path = this.fileName.substring(0,found);
        let file = this.fileName.substring(found+1);

        // Mandatory fields
        let logstr = "StorageRomLine l="+l+" line="+this.line+" file="+file+" ";

         // Selectors
        if (this.inFREE) logstr += "inFREE ";
        if (this.op.length>0) // inFREE parameters
        {
            logstr += "op=" + this.op;
            logstr += " funcName=" + this.funcName;
            logstr += " #params=" + this.params.length + " ";
            for (let i=0; i<this.params.length; i++)
            {
                logstr += "params[" + i + "]=" + this.params[i] + " ";
            }
        }
        if (this.CONST.length>0) logstr += "CONST=" + this.CONST + " "; // Constant
        if (this.inOLD_ROOT) logstr += "inOLD_ROOT ";
        if (this.inNEW_ROOT) logstr += "inNEW_ROOT ";
        if (this.inRKEY_BIT) logstr += "inRKEY_BIT ";
        if (this.inVALUE_LOW) logstr += "inVALUE_LOW ";
        if (this.inVALUE_HIGH) logstr += "inVALUE_HIGH ";
        if (this.inRKEY) logstr += "inRKEY ";
        if (this.inSIBLING_RKEY) logstr += "inSIBLING_RKEY ";
        if (this.inSIBLING_VALUE_HASH) logstr += "inSIBLING_VALUE_HASH ";
        if (this.inROTL_VH) logstr += "inROTL_VH ";

        // Instructions
        if (this.iJmpz) logstr += "iJmpz ";
        if (this.iJmp) logstr += "iJmp ";
        if (this.addressLabel.length>0) logstr += "addressLabel=" + this.addressLabel + " "; // Jump parameter
        if (this.address>0) logstr += "address=" + this.address + " "; // Jump parameter
        if (this.iRotateLevel) logstr += "iRotateLevel ";
        if (this.iHash) logstr += "iHash " + "iHashType=" + this.iHashType + " ";
        if (this.iClimbRkey) logstr += "iClimbRkey ";
        if (this.iClimbSiblingRkey) logstr += "iClimbSiblingRkey ";
        if (this.iClimbSiblingRkeyN) logstr += "iClimbSiblingRkeyN ";
        if (this.iLatchGet) logstr += "iLatchGet ";
        if (this.iLatchSet) logstr += "iLatchSet ";

        // Setters
        if (this.setRKEY) logstr += "setRKEY ";
        if (this.setRKEY_BIT) logstr += "setRKEY_BIT ";
        if (this.setVALUE_LOW) logstr += "setVALUE_LOW ";
        if (this.setVALUE_HIGH) logstr += "setVALUE_HIGH ";
        if (this.setLEVEL) logstr += "setLEVEL ";
        if (this.setOLD_ROOT) logstr += "setOLD_ROOT ";
        if (this.setNEW_ROOT) logstr += "setNEW_ROOT ";
        if (this.setHASH_LEFT) logstr += "setHASH_LEFT ";
        if (this.setHASH_RIGHT) logstr += "setHASH_RIGHT ";
        if (this.setSIBLING_RKEY) logstr += "setSIBLING_RKEY ";
        if (this.setSIBLING_VALUE_HASH) logstr += "setSIBLING_VALUE_HASH ";

        console.log(logstr);
    }
}

class StorageRom
{
    constructor ()
    {
        this.line = [];
    }

    load (j)
    {
        if (!j.hasOwnProperty("program") || !j.program.length>0) {
            console.error("Error: StorageRom::load() could not find a root program array");
            process.exit(-1);
        }

        for (let i=0; i<j.program.length; i++) {
            let romLine = new StorageRomLine;

            // Mandatory fields
            romLine.line = j.program[i].line;
            romLine.fileName = j.program[i].fileName;

            // Instructions
            if (j.program[i].hasOwnProperty("iJmpz")) romLine.iJmpz = true;
            if (j.program[i].hasOwnProperty("iJmp")) romLine.iJmp = true;
            if (j.program[i].hasOwnProperty("iRotateLevel")) romLine.iRotateLevel = true;
            if (j.program[i].hasOwnProperty("iHash")) romLine.iHash = true;
            if (j.program[i].hasOwnProperty("iHashType")) romLine.iHashType = j.program[i].iHashType;
            if (j.program[i].hasOwnProperty("iClimbRkey")) romLine.iClimbRkey = true;
            if (j.program[i].hasOwnProperty("iClimbSiblingRkey")) romLine.iClimbSiblingRkey = true;
            if (j.program[i].hasOwnProperty("iClimbSiblingRkeyN")) romLine.iClimbSiblingRkeyN = true;
            if (j.program[i].hasOwnProperty("iLatchGet")) romLine.iLatchGet = true;
            if (j.program[i].hasOwnProperty("iLatchSet")) romLine.iLatchSet = true;

            // Selectors
            if (j.program[i].hasOwnProperty("inFREE")) romLine.inFREE = true;
            if (j.program[i].hasOwnProperty("inOLD_ROOT")) romLine.inOLD_ROOT = true;
            if (j.program[i].hasOwnProperty("inNEW_ROOT")) romLine.inNEW_ROOT = true;
            if (j.program[i].hasOwnProperty("inVALUE_LOW")) romLine.inVALUE_LOW = true;
            if (j.program[i].hasOwnProperty("inVALUE_HIGH")) romLine.inVALUE_HIGH = true;
            if (j.program[i].hasOwnProperty("inRKEY")) romLine.inRKEY = true;
            if (j.program[i].hasOwnProperty("inRKEY_BIT")) romLine.inRKEY_BIT = true;
            if (j.program[i].hasOwnProperty("inSIBLING_RKEY")) romLine.inSIBLING_RKEY = true;
            if (j.program[i].hasOwnProperty("inSIBLING_VALUE_HASH")) romLine.inSIBLING_VALUE_HASH = true;
            if (j.program[i].hasOwnProperty("inROTL_VH")) romLine.inROTL_VH = true;

            // Setters
            if (j.program[i].hasOwnProperty("setRKEY")) romLine.setRKEY = true;
            if (j.program[i].hasOwnProperty("setRKEY_BIT")) romLine.setRKEY_BIT = true;
            if (j.program[i].hasOwnProperty("setVALUE_LOW")) romLine.setVALUE_LOW = true;
            if (j.program[i].hasOwnProperty("setVALUE_HIGH")) romLine.setVALUE_HIGH = true;
            if (j.program[i].hasOwnProperty("setLEVEL")) romLine.setLEVEL = true;
            if (j.program[i].hasOwnProperty("setOLD_ROOT")) romLine.setOLD_ROOT = true;
            if (j.program[i].hasOwnProperty("setNEW_ROOT")) romLine.setNEW_ROOT = true;
            if (j.program[i].hasOwnProperty("setHASH_LEFT")) romLine.setHASH_LEFT = true;
            if (j.program[i].hasOwnProperty("setHASH_RIGHT")) romLine.setHASH_RIGHT = true;
            if (j.program[i].hasOwnProperty("setSIBLING_RKEY")) romLine.setSIBLING_RKEY = true;
            if (j.program[i].hasOwnProperty("setSIBLING_VALUE_HASH")) romLine.setSIBLING_VALUE_HASH = true;

            // Jump parameters
            if (romLine.iJmp || romLine.iJmpz)
            {
                romLine.addressLabel = j.program[i].addressLabel;
                romLine.address = j.program[i].address;
            }

            // inFREE parameters
            if (romLine.inFREE)
            {
                romLine.op = j.program[i].freeInTag.op;
                if (romLine.op=="functionCall")
                {
                    romLine.funcName = j.program[i].freeInTag.funcName;
                    for (let p=0; p<j.program[i].freeInTag.params.length; p++)
                    {
                        romLine.params.push(j.program[i].freeInTag.params[p].num);
                    }
                }
            }

            // Constant
            if (j.program[i].hasOwnProperty("CONST"))
            {
                romLine.CONST = j.program[i].CONST;
            }

            this.line.push(romLine);
        }
    }
}

module.exports = {StorageRomLine, StorageRom};
