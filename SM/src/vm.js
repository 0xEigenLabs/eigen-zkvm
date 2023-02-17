const path = require("path");
const fs = require("fs");
const { F1Field } = require("ffjavascript");
const { newConstantPolsArray, newCommitPolsArray } = require("pilcom");
const { fri_verifier } = require("../../starkjs");

const smArith = require("./sm/sm_arith/sm_arith.js");
const smBinary = require("./sm/sm_binary.js");
const smGlobal = require("./sm/sm_global.js");
const smKeccakF = require("./sm/sm_keccakf/sm_keccakf.js");
const smMain = require("./sm/sm_main/sm_main.js");
const smMemAlign = require("./sm/sm_mem_align.js");
const smMem = require("./sm/sm_mem.js");
const smNine2One = require("./sm/sm_nine2one.js");
const smPaddingKK = require("./sm/sm_padding_kk.js");
const smPaddingKKBit = require("./sm/sm_padding_kkbit/sm_padding_kkbit.js");
const smPaddingPG = require("./sm/sm_padding_pg.js");
const smPoseidonG = require("./sm/sm_poseidong.js");
const smRom = require("./sm/sm_rom.js");
const smStorage = require("./sm/sm_storage/sm_storage.js");

module.exports = class VM {
  async buildConstants(constPols, argv) {
    const N = constPols.Global.L1.length;
    console.log(`N = ${N}`);

    if (constPols.Arith) {
        console.log("Arith...");
        await smArith.buildConstants(constPols.Arith);
    }
    if (constPols.Binary) {
        console.log("Binary...");
        await smBinary.buildConstants(constPols.Binary);
    }
    if (constPols.Global) {
        console.log("Global...");
        await smGlobal.buildConstants(constPols.Global);
    }
    if (constPols.KeccakF) {
        console.log("KeccakF...");
        await smKeccakF.buildConstants(constPols.KeccakF);
    }
    if (constPols.Main) {
        console.log("Main...");
        await smMain.buildConstants(constPols.Main);
    }
    if (constPols.MemAlign) {
        console.log("MemAlign...");
        await smMemAlign.buildConstants(constPols.MemAlign);
    }
    if (constPols.Mem) {
        console.log("Mem...");
        await smMem.buildConstants(constPols.Mem);
    }
    if (constPols.Nine2One) {
        console.log("Nine2One...");
        await smNine2One.buildConstants(constPols.Nine2One);
    }
    if (constPols.PaddingKK) {
        console.log("PaddingKK...");
        await smPaddingKK.buildConstants(constPols.PaddingKK);
    }
    if (constPols.PaddingKKBit) {
        console.log("PaddingKKBit...");
        await smPaddingKKBit.buildConstants(constPols.PaddingKKBit);
    }
    if (constPols.PaddingPG) {
        console.log("PaddingPG...");
        await smPaddingPG.buildConstants(constPols.PaddingPG);
    }
    if (constPols.PoseidonG) {
        console.log("PoseidonG...");
        await smPoseidonG.buildConstants(constPols.PoseidonG);
    }
    if (constPols.Rom) {
        console.log("Rom...");
        const rom = JSON.parse(await fs.promises.readFile(argv.romFile, "utf8"));
        await smRom.buildConstants(constPols.Rom, rom);
    }
    if (constPols.Storage) {
        console.log("Storage...");
        await smStorage.buildConstants(constPols.Storage);
    }

    for (let i=0; i<constPols.$$array.length; i++) {
        for (let j=0; j<N; j++) {
            if (typeof constPols.$$array[i][j] === "undefined") {
                throw new Error(`Polinomial not fited ${constPols.$$defArray[i].name} at ${j}` )
            }
        }
    }

    console.log("Constants generated succefully!");
  }

  async execute(cmPols, argv) {
    const input = JSON.parse(await fs.promises.readFile(argv.inputFile, "utf8"));
    const rom = JSON.parse(await fs.promises.readFile(argv.romFile, "utf8"));
    const test = argv.testFile ? JSON.parse(await fs.promises.readFile(argv.testFile, "utf8")) : false;
    const config = {
        test: test,
        debug: (argv.debug === true),
        debugInfo: {
            inputName: path.basename(argv.inputFile, ".json")
        },
        unsigned: (argv.unsigned === true),
        execute: (argv.execute === true),
        tracer: (argv.tracer === true)
    }

    const N = cmPols.Main.PC.length;
	
    let metadata = {};
    console.log(`N = ${N}`);
    console.log("Main ...");
    const requiredMain = await smMain.execute(cmPols.Main, input, rom, config, metadata);
    if (cmPols.Storage) {
        console.log("Storage...");
    }
    const requiredStorage = cmPols.Storage ? await smStorage.execute(cmPols.Storage, requiredMain.Storage) : false;

    if (cmPols.Arith) {
        console.log("Arith...");
        await smArith.execute(cmPols.Arith, requiredMain.Arith || []);
    }
    if (cmPols.Binary) {
        console.log("Binary...");
        await smBinary.execute(cmPols.Binary, requiredMain.Binary || []);
    }
    if (cmPols.MemAlign) {
        console.log("MemAlign...");
        await smMemAlign.execute(cmPols.MemAlign, requiredMain.MemAlign || []);
    }
    if (cmPols.Mem) {
        console.log("Mem...");
        await smMem.execute(cmPols.Mem, requiredMain.Mem || []);
    }
    if (cmPols.PaddingKK) console.log("PaddingKK...");
    const requiredKK = cmPols.PaddingKK ? await smPaddingKK.execute(cmPols.PaddingKK, requiredMain.PaddingKK || []) : false;

    if (cmPols.PaddingKKBit) console.log("PaddingKKbit...");
    const requiredKKBit = cmPols.PaddingKKBit ? await smPaddingKKBit.execute(cmPols.PaddingKKBit, requiredKK.paddingKKBit || []): false;

    if (cmPols.Nine2One) console.log("Nine2One...");
    const requiredNine2One = cmPols.Nine2One ? await smNine2One.execute(cmPols.Nine2One, requiredKKBit.Nine2One || []) : false;

    if (cmPols.KeccakF) {
        console.log("KeccakF...");
        await smKeccakF.execute(cmPols.KeccakF, requiredNine2One.KeccakF || []);
    }

    if (cmPols.PaddingPG) console.log("PaddingPG...");
    const requiredPaddingPG = cmPols.PaddingPG ? await smPaddingPG.execute(cmPols.PaddingPG, requiredMain.PaddingPG || []) : false;

    if (cmPols.PoseidonG) {
        console.log("PoseidonG...");
        const allPoseidonG = [ ...(requiredMain.PoseidonG || []), ...(requiredPaddingPG.PoseidonG || []), ...(requiredStorage.PoseidonG || []) ];
        await smPoseidonG.execute(cmPols.PoseidonG, allPoseidonG);
    }

    for (let i=0; i<cmPols.$$array.length; i++) {
        for (let j=0; j<N; j++) {
            if (typeof cmPols.$$array[i][j] === "undefined") {
                throw new Error(`Polinomial not fited ${cmPols.$$defArray[i].name} at ${j}` )
            }
        }
    }

    console.log("Executor finished correctly");
  }
}
