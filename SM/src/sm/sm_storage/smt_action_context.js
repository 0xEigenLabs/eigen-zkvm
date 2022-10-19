const { isLogging, logger, fea42String, fea42String10 }  = require("./sm_storage_utils.js");
const { Scalar } = require("ffjavascript");

class SmtActionContext
{
    constructor () {
        // Deepest level and current level
        this.level = 0; // Level at which the proof starts
        this.currentLevel = 0; // Current level, from level to zero, as we climb up the tree

        // Remaining key and preceding bits
        this.rkey = [];
        this.siblingRkey = [];
        this.bits = []; // Key bits consumed in the tree nodes, i.e. preceding remaining key rKey
        this.siblingBits = []; // Sibling key bits consumed in the tree nodes, i.e. preceding sibling remaining key siblingRKey
    }

    completeAction (fr, a) {
        if (a.bIsSet) {
            a.setResult.oldRoot = a.setResult.oldRoot ?? new Array(4).fill(fr.zero); //Array of FieldElement
            a.setResult.newRoot = a.setResult.newRoot ?? new Array(4).fill(fr.zero); //Array of FieldElement
            a.setResult.key = a.setResult.key ?? new Array(4).fill(fr.zero); //Array of FieldElement
            a.setResult.siblings = a.setResult.siblings ?? [];
            a.setResult.insKey = a.setResult.insKey ?? new Array(4).fill(fr.zero); //Array of FieldElement
            a.setResult.insValue = a.setResult.insValue ?? Scalar.zero;
            a.setResult.isOld0 = a.setResult.isOld0 ?? false;
            a.setResult.oldValue = a.setResult.oldValue ?? Scalar.zero;
            a.setResult.newValue = a.setResult.newValue ?? Scalar.zero;
            a.setResult.mode = a.setResult.mode ?? "";
        } else {
            a.getResult.root = a.getResult.root ?? new Array(4).fill(fr.zero); //Array of FieldElement
            a.getResult.key = a.getResult.key ?? new Array(4).fill(fr.zero); //Array of FieldElement
            a.getResult.siblings = a.getResult.siblings ?? [];
            a.getResult.insKey = a.getResult.insKey ?? new Array(4).fill(fr.zero);
            a.getResult.insValue = a.getResult.insValue ?? Scalar.zero; // value found
            a.getResult.isOld0 = a.getResult.isOld0 ?? false; // is new insert or delete
            a.getResult.value = a.getResult.value ?? Scalar.zero; // value retrieved
        }
    }

    init (fr, action) {

        this.completeAction(fr, action);

        if (action.bIsSet) {
            // Deepest, initial level
            this.level = action.setResult.siblings.length;

            // Initial value of rKey is key
            this.rkey[0] = action.setResult.key[0];
            this.rkey[1] = action.setResult.key[1];
            this.rkey[2] = action.setResult.key[2];
            this.rkey[3] = action.setResult.key[3];

            this.siblingRkey[0] = action.setResult.insKey[0];
            this.siblingRkey[1] = action.setResult.insKey[1];
            this.siblingRkey[2] = action.setResult.insKey[2];
            this.siblingRkey[3] = action.setResult.insKey[3];

            logger ("SmtActionContext::init() mode=" + action.setResult.mode);

        } else {
            this.level = action.getResult.siblings.length;

            // Initial value of rKey is key
            this.rkey[0] = action.getResult.key[0];
            this.rkey[1] = action.getResult.key[1];
            this.rkey[2] = action.getResult.key[2];
            this.rkey[3] = action.getResult.key[3];

            this.siblingRkey[0] = action.getResult.insKey[0];
            this.siblingRkey[1] = action.getResult.insKey[1];
            this.siblingRkey[2] = action.getResult.insKey[2];
            this.siblingRkey[3] = action.getResult.insKey[3];
        }

        if (true) {
            logger("SmtActionContext::init() key=" + fea42String10(fr, (action.bIsSet) ? action.setResult.key : action.getResult.key));
            logger("SmtActionContext::init() insKey=" + fea42String10(fr, (action.bIsSet) ? action.setResult.insKey : action.getResult.insKey));
            logger("SmtActionContext::init() insValue=" + ((action.bIsSet) ? action.setResult.insValue.toString(16) : action.getResult.insValue.toString(16)));
            logger("SmtActionContext::init() level=" + this.level);
            let siblings = (action.bIsSet) ? action.setResult.siblings : action.getResult.siblings;
            for (let [level, svalues] of siblings.entries()) {
                let logstr = "";
                logstr = "siblings[" + level +"]= ";
                for (let i=0; i<svalues.length; i++) {
                    logstr += svalues[i].toString(16) + ":";
                }
                logger(logstr);
            }
        }

        // Reset bits vectors
        this.bits = [];
        this.siblingBits = [];

        if (!action.bIsSet ||
            ( action.bIsSet && (action.setResult.mode=="update") ) ||
            ( action.bIsSet && (action.setResult.mode=="deleteNotFound") ) ||
            ( action.bIsSet && (action.setResult.mode=="zeroToZero") ) ||
            ( action.bIsSet && (action.setResult.mode=="insertNotFound") ) )
        {
            for (let i=0; i<this.level; i++)
            {
                let keyNumber = i%4; // 0, 1, 2, 3, 0, 1, 2, 3...
                let bit = this.rkey[keyNumber]&1n;
                let siblingBit = this.siblingRkey[keyNumber]&1n;
                this.bits.push(bit);
                this.siblingBits.push(siblingBit);
                this.rkey[keyNumber] /= 2n;
                this.siblingRkey[keyNumber] /= 2n;
            }
            logger("SmtActionContext::init() rKey=" + fea42String10 (fr, this.rkey));
        }

        // Generate bits vectors when there is a found sibling
        if ( ( action.bIsSet && (action.setResult.mode=="insertFound") ) ||
            ( action.bIsSet && (action.setResult.mode=="deleteFound") ) )
        {
            for (let i=0; i<256; i++)
            {
                let keyNumber = i%4; // 0, 1, 2, 3, 0, 1, 2, 3...
                let bit = this.rkey[keyNumber]&1n;
                let siblingBit = this.siblingRkey[keyNumber]&1n;
                this.rkey[keyNumber] /= 2n;
                this.siblingRkey[keyNumber] /= 2n;
                this.bits.push(bit);
                this.siblingBits.push(siblingBit);
                if (bit!=siblingBit) break;
            }

            logger("SmtActionContext::init() rKey=" + fea42String10(fr, this.rkey));
            logger("SmtActionContext::init() siblingRKey=" + fea42String10(fr, this.siblingRkey));

            // Update level
            this.level = this.bits.length;
        }

        this.currentLevel = this.level;

        // Print bits vector content
        if (isLogging) {
            let logstr = "SmtActionContext::init() ";
            for (let i=0; i<this.bits.length; i++)
            {
                logstr += "bits[" + i + "]=" + this.bits[i] + " ";
            }
            logger(logstr);
            logger("SmtActionContext::init() currentLevel=" + this.currentLevel);
        }

    }
}

module.exports = SmtActionContext;