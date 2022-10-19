const path = require("path");
const { ethers } = require("ethers");
const { Scalar, F1Field } = require("ffjavascript");

const { calculateStarkInput, calculateBatchHashData } = require("@0xpolygonhermez/zkevm-commonjs").contractUtils;
const { scalar2fea, fea2scalar, fe2n, scalar2h4, h4toString,
    stringToH4, nodeIsEq, hashContractBytecode, fea2String } = require("@0xpolygonhermez/zkevm-commonjs").smtUtils;
const SMT = require("@0xpolygonhermez/zkevm-commonjs").SMT;
const MemDB = require("@0xpolygonhermez/zkevm-commonjs").MemDB;
const buildPoseidon = require("@0xpolygonhermez/zkevm-commonjs").getPoseidon;
const { byteArray2HexString } = require("@0xpolygonhermez/zkevm-commonjs").utils;

const testTools = require("./test_tools");

const FullTracer = require("./debug/full-tracer");
const Prints = require("./debug/prints");

const twoTo255 = Scalar.shl(Scalar.one, 255);
const twoTo256 = Scalar.shl(Scalar.one, 256);

const Mask256 = Scalar.sub(Scalar.shl(Scalar.e(1), 256), 1);
const byteMaskOn256 = Scalar.bor(Scalar.shl(Mask256, 256), Scalar.shr(Mask256, 8n));

let fullTracer;

module.exports = async function execute(pols, input, rom, config = {}) {

    const required = {
        Byte4: {},
        Arith: [],
        Binary: [],
        PaddingKK: [],
        PaddingPG: [],
        PoseidonG: [],
        Mem: [],
        MemAlign: [],
        Storage: []
    };

    if (config && config.test) {
        testTools.setup(config.test, evalCommand);
    }

    const debug = config && config.debug;
    const flagTracer = config && config.tracer;
    const N = pols.zkPC.length;
    const stepsN = (debug && config.stepsN) ? config.stepsN : N;

    if (config && config.unsigned){
        if (typeof input.from === 'undefined'){
            throw new Error('Unsigned flag requires a `from` in the input');
        }
    }

    const skipAsserts = config.unsigned || config.execute;

    const poseidon = await buildPoseidon();
    const Fr = poseidon.F;
    const Fec = new F1Field(0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2fn);
    const Fnec = new F1Field(0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141n);

    const db = new MemDB(Fr, input.db);
    const smt = new SMT(db, poseidon, Fr);
    initState(Fr, pols);

    let op7, op6, op5, op4, op3, op2, op1, op0;

    const ctx = {
        mem: [],
        hashK: [],
        hashP: [],
        pols: pols,
        input: input ,
        vars:[],
        Fr: Fr,
        Fec: Fec,
        Fnec: Fnec,
        sto: input.keys,
        rom: rom,
        outLogs: {},
        N,
        stepsN
    }

    preprocessTxs(ctx);

    if (debug && flagTracer) {
        fullTracer = new FullTracer(config.debugInfo.inputName)
    }

    const iPrint = new Prints(ctx, smt);
    let fastDebugExit = false;

    for (step=0; step < stepsN; step++) {
        const i = step % N;
        ctx.ln = Fr.toObject(pols.zkPC[i]);
        ctx.step = step;
        ctx.A = [pols.A0[i], pols.A1[i], pols.A2[i], pols.A3[i], pols.A4[i], pols.A5[i], pols.A6[i], pols.A7[i]];
        ctx.B = [pols.B0[i], pols.B1[i], pols.B2[i], pols.B3[i], pols.B4[i], pols.B5[i], pols.B6[i], pols.B7[i]];
        ctx.C = [pols.C0[i], pols.C1[i], pols.C2[i], pols.C3[i], pols.C4[i], pols.C5[i], pols.C6[i], pols.C7[i]];
        ctx.D = [pols.D0[i], pols.D1[i], pols.D2[i], pols.D3[i], pols.D4[i], pols.D5[i], pols.D6[i], pols.D7[i]];
        ctx.E = [pols.E0[i], pols.E1[i], pols.E2[i], pols.E3[i], pols.E4[i], pols.E5[i], pols.E6[i], pols.E7[i]];
        ctx.SR = [ pols.SR0[i], pols.SR1[i], pols.SR2[i], pols.SR3[i], pols.SR4[i], pols.SR5[i], pols.SR6[i], pols.SR7[i]];
        ctx.CTX = pols.CTX[i];
        ctx.SP = pols.SP[i];
        ctx.PC = pols.PC[i];
        ctx.RR = pols.RR[i];
        ctx.HASHPOS = pols.HASHPOS[i];
        ctx.MAXMEM = pols.MAXMEM[i];
        ctx.GAS = pols.GAS[i];
        ctx.zkPC = pols.zkPC[i];
        ctx.cntArith = pols.cntArith[i];
        ctx.cntBinary = pols.cntBinary[i];
        ctx.cntKeccakF = pols.cntKeccakF[i];
        ctx.cntMemAlign = pols.cntMemAlign[i];
        ctx.cntPoseidonG = pols.cntPoseidonG[i];
        ctx.cntPaddingPG = pols.cntPaddingPG[i];

        const l = rom.program[ ctx.zkPC ];

        ctx.fileName = l.fileName;
        ctx.line = l.line;

        // breaks the loop in debug mode in order to test and debug faster
        if (debug && Number(ctx.zkPC) === rom.labels.finalizeExecution) {
            fastDebugExit = true;
            break;
        }

        let incHashPos = 0;
        let incCounter = 0;

        // if (step%1000==0) console.log(`Step: ${step}`);

        if (step==330) {
             // console.log("### > "+l.fileName + ':' + l.line);
        }

        if (l.cmdBefore) {
            for (let j=0; j< l.cmdBefore.length; j++) {
                evalCommand(ctx, l.cmdBefore[j]);
            }
        }

//////////
// LOAD INPUTS
//////////

        [op0, op1, op2, op3, op4, op5, op6, op7] = [Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero];

        if (l.inA) {
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add(op0 , Fr.mul( Fr.e(l.inA), ctx.A[0])),
                 Fr.add(op1 , Fr.mul( Fr.e(l.inA), ctx.A[1])),
                 Fr.add(op2 , Fr.mul( Fr.e(l.inA), ctx.A[2])),
                 Fr.add(op3 , Fr.mul( Fr.e(l.inA), ctx.A[3])),
                 Fr.add(op4 , Fr.mul( Fr.e(l.inA), ctx.A[4])),
                 Fr.add(op5 , Fr.mul( Fr.e(l.inA), ctx.A[5])),
                 Fr.add(op6 , Fr.mul( Fr.e(l.inA), ctx.A[6])),
                 Fr.add(op7 , Fr.mul( Fr.e(l.inA), ctx.A[7]))
                ];
            pols.inA[i] = Fr.e(l.inA);
        } else {
            pols.inA[i] = Fr.zero;
        }

        if (l.inB) {
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add(op0 , Fr.mul( Fr.e(l.inB), ctx.B[0])),
                 Fr.add(op1 , Fr.mul( Fr.e(l.inB), ctx.B[1])),
                 Fr.add(op2 , Fr.mul( Fr.e(l.inB), ctx.B[2])),
                 Fr.add(op3 , Fr.mul( Fr.e(l.inB), ctx.B[3])),
                 Fr.add(op4 , Fr.mul( Fr.e(l.inB), ctx.B[4])),
                 Fr.add(op5 , Fr.mul( Fr.e(l.inB), ctx.B[5])),
                 Fr.add(op6 , Fr.mul( Fr.e(l.inB), ctx.B[6])),
                 Fr.add(op7 , Fr.mul( Fr.e(l.inB), ctx.B[7]))
                ];
            pols.inB[i] = Fr.e(l.inB);
        } else {
            pols.inB[i] = Fr.zero;
        }

        if (l.inC) {
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add(op0 , Fr.mul( Fr.e(l.inC), ctx.C[0])),
                 Fr.add(op1 , Fr.mul( Fr.e(l.inC), ctx.C[1])),
                 Fr.add(op2 , Fr.mul( Fr.e(l.inC), ctx.C[2])),
                 Fr.add(op3 , Fr.mul( Fr.e(l.inC), ctx.C[3])),
                 Fr.add(op4 , Fr.mul( Fr.e(l.inC), ctx.C[4])),
                 Fr.add(op5 , Fr.mul( Fr.e(l.inC), ctx.C[5])),
                 Fr.add(op6 , Fr.mul( Fr.e(l.inC), ctx.C[6])),
                 Fr.add(op7 , Fr.mul( Fr.e(l.inC), ctx.C[7]))
                ];
            pols.inC[i] = Fr.e(l.inC);
        } else {
            pols.inC[i] = Fr.zero;
        }

        if (l.inD) {
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add(op0 , Fr.mul( Fr.e(l.inD), ctx.D[0])),
                 Fr.add(op1 , Fr.mul( Fr.e(l.inD), ctx.D[1])),
                 Fr.add(op2 , Fr.mul( Fr.e(l.inD), ctx.D[2])),
                 Fr.add(op3 , Fr.mul( Fr.e(l.inD), ctx.D[3])),
                 Fr.add(op4 , Fr.mul( Fr.e(l.inD), ctx.D[4])),
                 Fr.add(op5 , Fr.mul( Fr.e(l.inD), ctx.D[5])),
                 Fr.add(op6 , Fr.mul( Fr.e(l.inD), ctx.D[6])),
                 Fr.add(op7 , Fr.mul( Fr.e(l.inD), ctx.D[7]))
                ];
            pols.inD[i] = Fr.e(l.inD);
        } else {
            pols.inD[i] = Fr.zero;
        }

        if (l.inE) {
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add(op0 , Fr.mul( Fr.e(l.inE), ctx.E[0])),
                 Fr.add(op1 , Fr.mul( Fr.e(l.inE), ctx.E[1])),
                 Fr.add(op2 , Fr.mul( Fr.e(l.inE), ctx.E[2])),
                 Fr.add(op3 , Fr.mul( Fr.e(l.inE), ctx.E[3])),
                 Fr.add(op4 , Fr.mul( Fr.e(l.inE), ctx.E[4])),
                 Fr.add(op5 , Fr.mul( Fr.e(l.inE), ctx.E[5])),
                 Fr.add(op6 , Fr.mul( Fr.e(l.inE), ctx.E[6])),
                 Fr.add(op7 , Fr.mul( Fr.e(l.inE), ctx.E[7]))
                ];
            pols.inE[i] = Fr.e(l.inE);
        } else {
            pols.inE[i] = Fr.zero;
        }

        if (l.inSR) {
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add(op0 , Fr.mul( Fr.e(l.inSR), ctx.SR[0])),
                 Fr.add(op1 , Fr.mul( Fr.e(l.inSR), ctx.SR[1])),
                 Fr.add(op2 , Fr.mul( Fr.e(l.inSR), ctx.SR[2])),
                 Fr.add(op3 , Fr.mul( Fr.e(l.inSR), ctx.SR[3])),
                 Fr.add(op4 , Fr.mul( Fr.e(l.inSR), ctx.SR[4])),
                 Fr.add(op5 , Fr.mul( Fr.e(l.inSR), ctx.SR[5])),
                 Fr.add(op6 , Fr.mul( Fr.e(l.inSR), ctx.SR[6])),
                 Fr.add(op7 , Fr.mul( Fr.e(l.inSR), ctx.SR[7]))
                ];
            pols.inSR[i] = Fr.e(l.inSR);
        } else {
            pols.inSR[i] = Fr.zero;
        }

        if (l.inCTX) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inCTX), Fr.e(ctx.CTX)));
            pols.inCTX[i] = Fr.e(l.inCTX);
        } else {
            pols.inCTX[i] = Fr.zero;
        }

        if (l.inSP) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inSP), Fr.e(ctx.SP)));
            pols.inSP[i] = Fr.e(l.inSP);
        } else {
            pols.inSP[i] = Fr.zero;
        }

        if (l.inPC) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inPC), Fr.e(ctx.PC)));
            pols.inPC[i] = Fr.e(l.inPC);
        } else {
            pols.inPC[i] = Fr.zero;
        }

        if (l.inGAS) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inGAS), Fr.e(ctx.GAS)));
            pols.inGAS[i] = Fr.e(l.inGAS);
        } else {
            pols.inGAS[i] = Fr.zero;
        }

        if (l.inMAXMEM) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inMAXMEM), Fr.e(ctx.MAXMEM)));
            pols.inMAXMEM[i] = Fr.e(l.inMAXMEM);
        } else {
            pols.inMAXMEM[i] = Fr.zero;
        }

        if (l.inSTEP) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inSTEP), Fr.e(i)));
            pols.inSTEP[i] = Fr.e(l.inSTEP);
        } else {
            pols.inSTEP[i] = Fr.zero;
        }

        if (l.inRR) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inRR), Fr.e(ctx.RR)));
            pols.inRR[i] = Fr.e(l.inRR);
        } else {
            pols.inRR[i] = Fr.zero;
        }

        if (l.inHASHPOS) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inHASHPOS), Fr.e(ctx.HASHPOS)));
            pols.inHASHPOS[i] = Fr.e(l.inHASHPOS);
        } else {
            pols.inHASHPOS[i] = Fr.zero;
        }

        // COUNTERS
        if (l.inCntArith) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inCntArith), Fr.e(ctx.cntArith)));
            pols.inCntArith[i] = Fr.e(l.inCntArith);
        } else {
            pols.inCntArith[i] = Fr.zero;
        }

        if (l.inCntBinary) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inCntBinary), Fr.e(ctx.cntBinary)));
            pols.inCntBinary[i] = Fr.e(l.inCntBinary);
        } else {
            pols.inCntBinary[i] = Fr.zero;
        }

        if (l.inCntMemAlign) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inCntMemAlign), Fr.e(ctx.cntMemAlign)));
            pols.inCntMemAlign[i] = Fr.e(l.inCntMemAlign);
        } else {
            pols.inCntMemAlign[i] = Fr.zero;
        }

        if (l.inCntKeccakF) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inCntKeccakF), Fr.e(ctx.cntKeccakF)));
            pols.inCntKeccakF[i] = Fr.e(l.inCntKeccakF);
        } else {
            pols.inCntKeccakF[i] = Fr.zero;
        }

        if (l.inCntPoseidonG) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inCntPoseidonG), Fr.e(ctx.cntPoseidonG)));
            pols.inCntPoseidonG[i] = Fr.e(l.inCntPoseidonG);
        } else {
            pols.inCntPoseidonG[i] = Fr.zero;
        }

        if (l.inCntPaddingPG) {
            op0 = Fr.add(op0, Fr.mul( Fr.e(l.inCntPaddingPG), Fr.e(ctx.cntPaddingPG)));
            pols.inCntPaddingPG[i] = Fr.e(l.inCntPaddingPG);
        } else {
            pols.inCntPaddingPG[i] = Fr.zero;
        }

        if (l.inROTL_C) {
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add(op0 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[7])),
                 Fr.add(op1 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[0])),
                 Fr.add(op2 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[1])),
                 Fr.add(op3 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[2])),
                 Fr.add(op4 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[3])),
                 Fr.add(op5 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[4])),
                 Fr.add(op6 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[5])),
                 Fr.add(op7 , Fr.mul( Fr.e(l.inROTL_C), ctx.C[6]))
                ];
            pols.inROTL_C[i] = Fr.e(l.inROTL_C);
        } else {
            pols.inROTL_C[i] = Fr.zero;
        }

        if ((!isNaN(l.CONSTL))&&(l.CONSTL)) {
            [
                pols.CONST0[i],
                pols.CONST1[i],
                pols.CONST2[i],
                pols.CONST3[i],
                pols.CONST4[i],
                pols.CONST5[i],
                pols.CONST6[i],
                pols.CONST7[i]
            ] = scalar2fea(Fr, l.CONSTL);
            [op0, op1, op2, op3, op4, op5, op6, op7] = [
                Fr.add(op0 , pols.CONST0[i]),
                Fr.add(op1 , pols.CONST1[i]),
                Fr.add(op2 , pols.CONST2[i]),
                Fr.add(op3 , pols.CONST3[i]),
                Fr.add(op4 , pols.CONST4[i]),
                Fr.add(op5 , pols.CONST5[i]),
                Fr.add(op6 , pols.CONST6[i]),
                Fr.add(op7 , pols.CONST7[i])
            ];
        } else if ((!isNaN(l.CONST))&&(l.CONST)) {
            pols.CONST0[i] = Fr.e(l.CONST);
            op0 = Fr.add(op0, pols.CONST0[i] );
            pols.CONST1[i] = Fr.zero;
            pols.CONST2[i] = Fr.zero;
            pols.CONST3[i] = Fr.zero;
            pols.CONST4[i] = Fr.zero;
            pols.CONST5[i] = Fr.zero;
            pols.CONST6[i] = Fr.zero;
            pols.CONST7[i] = Fr.zero;
        } else {
            pols.CONST0[i] = Fr.zero;
            pols.CONST1[i] = Fr.zero;
            pols.CONST2[i] = Fr.zero;
            pols.CONST3[i] = Fr.zero;
            pols.CONST4[i] = Fr.zero;
            pols.CONST5[i] = Fr.zero;
            pols.CONST6[i] = Fr.zero;
            pols.CONST7[i] = Fr.zero;
        }

////////////
// PREPARE AUXILIARY VARS
////////////

        let addrRel = 0;
        let addr = 0;
        if (l.mOp || l.JMP || l.JMPN || l.JMPC ||  l.hashP || l.hashPLen || l.hashPDigest ||  l.hashK || l.hashKLen || l.hashKDigest || l.JMP || l.JMPC) {
            if (l.ind) {
                addrRel = fe2n(Fr, ctx.E[0], ctx);
            }
            if (l.indRR) {
                addrRel += fe2n(Fr, ctx.RR, ctx);
            }
            if (l.offset) addrRel += l.offset;
            if (addrRel >= 0x10000) throw new Error(`Address too big: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            if (addrRel <0 ) throw new Error(`Address can not be negative: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            addr = addrRel;
        }
        if (l.useCTX==1) {
            addr += Number(ctx.CTX)*0x40000;
            pols.useCTX[i] = 1n;
        } else {
            pols.useCTX[i] = 0n;
        }
        if (l.isCode==1) {
            addr += 0x10000;
            pols.isCode[i] = 1n;
        } else {
            pols.isCode[i] = 0n;
        }
        if (l.isStack==1) {
            addr += 0x20000;
            addr += Number(ctx.SP);
            pols.isStack[i] = 1n;
        } else {
            pols.isStack[i] = 0n;
        }
        if (l.isMem==1) {
            addr += 0x30000;
            pols.isMem[i] = 1n;
        } else {
            pols.isMem[i] = 0n;
        }
        if (l.incCode) {
            pols.incCode[i] = BigInt(l.incCode);
        } else {
            pols.incCode[i] = 0n;
        }
        if (l.incStack) {
            pols.incStack[i] = BigInt(l.incStack);
        } else {
            pols.incStack[i] = 0n;
        }
        if (l.ind) {
            pols.ind[i] = 1n;
        } else {
            pols.ind[i] = 0n;
        }
        if (l.indRR) {
            pols.indRR[i] = 1n;
        } else {
            pols.indRR[i] = 0n;
        }
        if (l.offset) {
            pols.offset[i] = BigInt(l.offset);
        } else {
            pols.offset[i] = 0n;
        }

//////
// CALCULATE AND LOAD FREE INPUT
//////

        if (l.inFREE) {

            if (!l.freeInTag) {
                throw new Error(`Instruction with freeIn without freeInTag: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            }

            let fi;
            if (l.freeInTag.op=="") {
                let nHits = 0;
                if (l.mOp == 1 && l.mWR == 0) {
                    if (typeof ctx.mem[addr] != "undefined") {
                        fi = ctx.mem[addr];
                    } else {
                        fi = [Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero];
                    }
                    nHits++;
                }
                if (l.sRD == 1) {
                    const Kin0 = [
                        ctx.C[0],
                        ctx.C[1],
                        ctx.C[2],
                        ctx.C[3],
                        ctx.C[4],
                        ctx.C[5],
                        ctx.C[6],
                        ctx.C[7],
                    ];

                    const Kin1 = [
                        ctx.A[0],
                        ctx.A[1],
                        ctx.A[2],
                        ctx.A[3],
                        ctx.A[4],
                        ctx.A[5],
                        ctx.B[0],
                        ctx.B[1]
                    ];

                    const keyI = poseidon(Kin0);
                    required.PoseidonG.push([...Kin0, 0n, 0n, 0n, 0n, ...keyI]);
                    const key = poseidon(Kin1, keyI);
                    required.PoseidonG.push([...Kin1, ...keyI,  ...key]);

                    // commented since readings are done directly in the smt
                    // const keyS = Fr.toString(key, 16).padStart(64, "0");
                    // if (typeof ctx.sto[keyS] === "undefined" ) throw new Error(`Storage not initialized: ${ctx.ln}`);

                    // fi = scalar2fea(Fr, Scalar.e("0x" + ctx.sto[ keyS ]));
                    const res = await smt.get(sr8to4(ctx.Fr, ctx.SR), key);
                    incCounter = res.proofHashCounter + 2;
                    fi = scalar2fea(Fr, Scalar.e(res.value));
                    nHits++;
                }
                if (l.sWR == 1) {
                    ctx.lastSWrite = {};

                    const Kin0 = [
                        ctx.C[0],
                        ctx.C[1],
                        ctx.C[2],
                        ctx.C[3],
                        ctx.C[4],
                        ctx.C[5],
                        ctx.C[6],
                        ctx.C[7],
                    ];

                    const Kin1 = [
                        ctx.A[0],
                        ctx.A[1],
                        ctx.A[2],
                        ctx.A[3],
                        ctx.A[4],
                        ctx.A[5],
                        ctx.B[0],
                        ctx.B[1]
                    ];

                    const keyI = poseidon(Kin0);
                    required.PoseidonG.push([...Kin0, 0n, 0n, 0n, 0n, ...keyI]);
                    const key = poseidon(Kin1, keyI);
                    required.PoseidonG.push([...Kin1, ...keyI,  ...key]);

                    ctx.lastSWrite.keyI = keyI;
                    ctx.lastSWrite.key = key;

                    // commented since readings are also done directly in the s
                    // ctx.lastSWrite.keyS = ctx.lastSWrite.key.toString(16);
                    // if (typeof ctx.sto[ctx.lastSWrite.keyS ] === "undefined" ) throw new Error(`Storage not initialized: ${ctx.ln}`);

                    const res = await smt.set(sr8to4(ctx.Fr, ctx.SR), ctx.lastSWrite.key, fea2scalar(Fr, ctx.D));
                    incCounter = res.proofHashCounter + 2;

                    ctx.lastSWrite.newRoot = res.newRoot;
                    ctx.lastSWrite.res = res;
                    ctx.lastSWrite.step = step;

                    fi = sr4to8(ctx.Fr, ctx.lastSWrite.newRoot);
                    nHits++;
                }

                if (l.hashK == 1) {
                    if (typeof ctx.hashK[addr] === "undefined") ctx.hashK[addr] = { data: [], reads: {} };
                    const size = fe2n(Fr, ctx.D[0], ctx);
                    const pos = fe2n(Fr, ctx.HASHPOS, ctx);
                    if ((size<0) || (size>32)) throw new Error(`Invalid size for hash: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    if (pos+size > ctx.hashK[addr].data.length) throw new Error(`Accessing hashK out of bounds ${addr}, ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    let s = Scalar.zero;
                    for (let k=0; k<size; k++) {
                        if (typeof ctx.hashK[addr].data[pos + k] === "undefined") throw new Error(`Accessing hashK not defined place ${addr}, ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                        s = Scalar.add(Scalar.mul(s, 256), Scalar.e(ctx.hashK[addr].data[pos + k]));
                    }
                    fi = scalar2fea(Fr, s);
                    nHits++;
                }
                if (l.hashKDigest == 1) {
                    if (typeof ctx.hashK[addr] === "undefined") {
                        throw new Error(`digest not defined.  ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    }
                    if (typeof ctx.hashK[addr].digest === "undefined") {
                        throw new Error(`digest not calculated.  Call hashKlen to finish digest: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    }
                    fi = scalar2fea(Fr, ctx.hashK[addr].digest);
                    nHits++;
                }
                if (l.hashP == 1) {
                    if (typeof ctx.hashP[addr] === "undefined") ctx.hashP[addr] = { data: [], reads: {} };
                    const size = fe2n(Fr, ctx.D[0], ctx);
                    const pos = fe2n(Fr, ctx.HASHPOS, ctx);

                    if ((size<0) || (size>32)) throw new Error(`Invalid size for hash: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    if (pos+size > ctx.hashP[addr].data.length) throw new Error(`Accessing hashP out of bounds ${addr}, ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    let s = Scalar.zero;
                    for (let k=0; k<size; k++) {
                        if (typeof ctx.hashP[addr].data[pos + k] === "undefined") throw new Error(`Accessing hashP not defined place ${addr}, ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                        s = Scalar.add(Scalar.mul(s, 256), Scalar.e(ctx.hashP[addr].data[pos + k]));
                    }
                    fi = scalar2fea(Fr, s);
                    nHits++;
                }
                if (l.hashPDigest == 1) {
                    if (typeof ctx.hashP[addr] === "undefined") {
                        throw new Error(`digest not defined.  ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    }
                    if (typeof ctx.hashP[addr].digest === "undefined") {
                        throw new Error(`digest not calculated.  Call hashPlen to finish digest: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    }
                    fi = scalar2fea(Fr, ctx.hashP[addr].digest);
                    nHits++;
                }
                if (l.bin) {
                    if (l.binOpcode == 0) { // ADD
                        const a = Scalar.e(fea2scalar(Fr, ctx.A));
                        const b = Scalar.e(fea2scalar(Fr, ctx.B));
                        const c = Scalar.band(Scalar.add(a, b), Mask256);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else if (l.binOpcode == 1) { // SUB
                        const a = Scalar.e(fea2scalar(Fr, ctx.A));
                        const b = Scalar.e(fea2scalar(Fr, ctx.B));
                        const c = Scalar.band(Scalar.add(Scalar.sub(a, b), twoTo256), Mask256);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else if (l.binOpcode == 2) { // LT
                        const a = Scalar.e(fea2scalar(Fr, ctx.A));
                        const b = Scalar.e(fea2scalar(Fr, ctx.B));
                        const c = Scalar.lt(a, b);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else if (l.binOpcode == 3) { // SLT
                        let a = Scalar.e(fea2scalar(Fr, ctx.A));
                        if (Scalar.geq(a, twoTo255)) a = Scalar.sub(a, twoTo256);
                        let b = Scalar.e(fea2scalar(Fr, ctx.B));
                        if (Scalar.geq(b, twoTo255)) b = Scalar.sub(b, twoTo256);
                        const c = Scalar.lt(a, b);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else if (l.binOpcode == 4) { // EQ
                        const a = Scalar.e(fea2scalar(Fr, ctx.A));
                        const b = Scalar.e(fea2scalar(Fr, ctx.B));
                        const c = Scalar.eq(a, b);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else if (l.binOpcode == 5) { // AND
                        const a = Scalar.e(fea2scalar(Fr, ctx.A));
                        const b = Scalar.e(fea2scalar(Fr, ctx.B));
                        const c = Scalar.band(a, b);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else if (l.binOpcode == 6) { // OR
                        const a = Scalar.e(fea2scalar(Fr, ctx.A));
                        const b = Scalar.e(fea2scalar(Fr, ctx.B));
                        const c = Scalar.bor(a, b);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else if (l.binOpcode == 7) { // XOR
                        const a = Scalar.e(fea2scalar(Fr, ctx.A));
                        const b = Scalar.e(fea2scalar(Fr, ctx.B));
                        const c = Scalar.bxor(a, b);
                        fi = scalar2fea(Fr, c);
                        nHits ++;
                    } else {
                        throw new Error("Invalid Binary operation");
                    }
                }

                if (l.memAlign && !l.memAlignWR) {
                    const m0 = fea2scalar(Fr, ctx.A);
                    const m1 = fea2scalar(Fr, ctx.B);
                    const P2_256 = 2n ** 256n;
                    const MASK_256 = P2_256 - 1n;
                    const offset = fea2scalar(Fr, ctx.C);
                    if (offset < 0 || offset > 32) {
                        throw new Error(`MemAlign out of range (${offset}): ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                    }
                    const leftV = Scalar.band(Scalar.shl(m0, offset * 8n), MASK_256);
                    const rightV = Scalar.band(Scalar.shr(m1, 256n - (offset * 8n)), MASK_256 >> (256n - (offset * 8n)));
                    const _V = Scalar.bor(leftV, rightV);
                    fi = scalar2fea(Fr, _V);
                    nHits ++;
                }

                if (nHits==0) {
                    throw new Error(`Empty freeIn without a valid instruction: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }
                if (nHits>1) {
                    throw new Error(`Only one instruction that requires freeIn is allowed: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }
            } else {
                fi = evalCommand(ctx, l.freeInTag);
                if (!Array.isArray(fi)) fi = scalar2fea(Fr, fi);
            }
            [pols.FREE0[i], pols.FREE1[i], pols.FREE2[i], pols.FREE3[i], pols.FREE4[i], pols.FREE5[i], pols.FREE6[i], pols.FREE7[i]] = fi;
            [op0, op1, op2, op3, op4, op5, op6, op7] =
                [Fr.add( Fr.mul(Fr.e(l.inFREE), fi[0]), op0 ),
                 Fr.add( Fr.mul(Fr.e(l.inFREE), fi[1]), op1 ),
                 Fr.add( Fr.mul(Fr.e(l.inFREE), fi[2]), op2 ),
                 Fr.add( Fr.mul(Fr.e(l.inFREE), fi[3]), op3 ),
                 Fr.add( Fr.mul(Fr.e(l.inFREE), fi[4]), op4 ),
                 Fr.add( Fr.mul(Fr.e(l.inFREE), fi[5]), op5 ),
                 Fr.add( Fr.mul(Fr.e(l.inFREE), fi[6]), op6 ),
                 Fr.add( Fr.mul(Fr.e(l.inFREE), fi[7]), op7 )
                ];
            pols.inFREE[i] = Fr.e(l.inFREE);
        } else {
            [pols.FREE0[i], pols.FREE1[i], pols.FREE2[i], pols.FREE3[i], pols.FREE4[i], pols.FREE5[i], pols.FREE6[i], pols.FREE7[i]] = [Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero, Fr.zero];
            pols.inFREE[i] = Fr.zero;
        }

//////////
// PROCESS INSTRUCTIONS
//////////

        if (l.assert) {
            if ((Number(ctx.zkPC) === rom.labels.assertNewStateRoot) && skipAsserts){
                console.log("Skip assert newStateRoot");
            } else if ((Number(ctx.zkPC) === rom.labels.assertNewLocalExitRoot) && skipAsserts){
                console.log("Skip assert newLocalExitRoot");
            } else if (
                    (!Fr.eq(ctx.A[0], op0)) ||
                    (!Fr.eq(ctx.A[1], op1)) ||
                    (!Fr.eq(ctx.A[2], op2)) ||
                    (!Fr.eq(ctx.A[3], op3)) ||
                    (!Fr.eq(ctx.A[4], op4)) ||
                    (!Fr.eq(ctx.A[5], op5)) ||
                    (!Fr.eq(ctx.A[6], op6)) ||
                    (!Fr.eq(ctx.A[7], op7))
            ) {
                throw new Error(`Assert does not match: ${ctx.ln} at ${ctx.fileName}:${ctx.line} (op:${fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7])} A:${fea2scalar(Fr, ctx.A)})`);
            }
            pols.assert[i] = 1n;
        } else {
            pols.assert[i] = 0n;
        }


        if (l.mOp) {
            pols.mOp[i] = 1n;

            if (l.mWR) {
                pols.mWR[i] = 1n;
                ctx.mem[addr] = [op0, op1, op2, op3, op4, op5, op6, op7];
                required.Mem.push({
                    bIsWrite: true,
                    address: addr,
                    pc: step,
                    fe0:op0, fe1:op1, fe2:op2, fe3:op3, fe4:op4, fe5:op5, fe6:op6, fe7:op7
                });
            } else {
                pols.mWR[i] = 0n;
                required.Mem.push({
                    bIsWrite: false,
                    address: addr,
                    pc: step,
                    fe0:op0, fe1:op1, fe2:op2, fe3:op3, fe4:op4, fe5:op5, fe6:op6, fe7:op7
                });
                if (ctx.mem[addr]) {
                    if ((!Fr.eq(ctx.mem[addr][0],  op0)) ||
                        (!Fr.eq(ctx.mem[addr][1],  op1)) ||
                        (!Fr.eq(ctx.mem[addr][2],  op2)) ||
                        (!Fr.eq(ctx.mem[addr][3],  op3)) ||
                        (!Fr.eq(ctx.mem[addr][4],  op4)) ||
                        (!Fr.eq(ctx.mem[addr][5],  op5)) ||
                        (!Fr.eq(ctx.mem[addr][6],  op6)) ||
                        (!Fr.eq(ctx.mem[addr][7],  op7)))
                    {
                        throw new Error("Memory Read does not match");
                    }
                } else {
                    if ((!Fr.isZero(op0)) ||
                        (!Fr.isZero(op1)) ||
                        (!Fr.isZero(op2)) ||
                        (!Fr.isZero(op3)) ||
                        (!Fr.isZero(op4)) ||
                        (!Fr.isZero(op5)) ||
                        (!Fr.isZero(op6)) ||
                        (!Fr.isZero(op7)))
                    {
                        throw new Error("Memory Read does not match");
                    }
                }

            }

        } else {
            pols.mOp[i] = 0n;
            pols.mWR[i] = 0n;
        }

        if (l.sRD) {
            pols.sRD[i] = 1n;

            const Kin0 = [
                ctx.C[0],
                ctx.C[1],
                ctx.C[2],
                ctx.C[3],
                ctx.C[4],
                ctx.C[5],
                ctx.C[6],
                ctx.C[7],
            ];

            const Kin1 = [
                ctx.A[0],
                ctx.A[1],
                ctx.A[2],
                ctx.A[3],
                ctx.A[4],
                ctx.A[5],
                ctx.B[0],
                ctx.B[1]
            ];

            const keyI = poseidon(Kin0);
            const key = poseidon(Kin1, keyI);

            const res = await smt.get(sr8to4(ctx.Fr, ctx.SR), key);
            incCounter = res.proofHashCounter + 2;

            required.Storage.push({
                bIsSet: false,
                getResult: {
                    root: [...res.root],
                    key: [...res.key],
                    siblings: [...res.siblings],
                    insKey: res.insKey ? [...res.insKey] : new Array(4).fill(Scalar.zero),
                    insValue: res.insValue,
                    isOld0: res.isOld0,
                    value: res.value
                }});

            if (!Scalar.eq(res.value,fea2scalar(Fr,[op0, op1, op2, op3, op4, op5, op6, op7]))) {
                throw new Error(`Storage read does not match: ${ctx.ln}`);
            }

            for (let k=0; k<4; k++) {
                pols.sKeyI[k][i] =  keyI[k];
                pols.sKey[k][i] = key[k];
            }

        } else {
            pols.sRD[i] = 0n;
        }

        if (l.sWR) {
            pols.sWR[i] = 1n;

            if ((!ctx.lastSWrite)||(ctx.lastSWrite.step != step)) {
                ctx.lastSWrite = {};

                const Kin0 = [
                    ctx.C[0],
                    ctx.C[1],
                    ctx.C[2],
                    ctx.C[3],
                    ctx.C[4],
                    ctx.C[5],
                    ctx.C[6],
                    ctx.C[7],
                ];

                const Kin1 = [
                    ctx.A[0],
                    ctx.A[1],
                    ctx.A[2],
                    ctx.A[3],
                    ctx.A[4],
                    ctx.A[5],
                    ctx.B[0],
                    ctx.B[1]
                ];

                ctx.lastSWrite.keyI = poseidon(Kin0);
                ctx.lastSWrite.key = poseidon(Kin1, ctx.lastSWrite.keyI);

                // commented since readings are also done directly in the smt
                // ctx.lastSWrite.keyS = Fr.toString(ctx.lastSWrite.key, 16).padStart(64, "0");
                // if (typeof ctx.sto[ctx.lastSWrite.keyS ] === "undefined" ) throw new Error(`Storage not initialized: ${ctx.ln}`);

                const res = await smt.set(sr8to4(ctx.Fr, ctx.SR), ctx.lastSWrite.key, fea2scalar(Fr, ctx.D));
                incCounter = res.proofHashCounter + 2;

                ctx.lastSWrite.res = res;
                ctx.lastSWrite.newRoot = res.newRoot;
                ctx.lastSWrite.step = step;
            }

            required.Storage.push({
                bIsSet: true,
                setResult: {
                    oldRoot: [...ctx.lastSWrite.res.oldRoot],
                    newRoot: [...ctx.lastSWrite.res.newRoot],
                    key: [...ctx.lastSWrite.res.key],
                    siblings: [...ctx.lastSWrite.res.siblings],
                    insKey: ctx.lastSWrite.res.insKey ? [...ctx.lastSWrite.res.insKey] : new Array(4).fill(Scalar.zero),
                    insValue: ctx.lastSWrite.res.insValue,
                    isOld0: ctx.lastSWrite.res.isOld0,
                    oldValue: ctx.lastSWrite.res.oldValue,
                    newValue: ctx.lastSWrite.res.newValue,
                    mode: ctx.lastSWrite.res.mode
                }});

            if (!nodeIsEq(ctx.lastSWrite.newRoot, sr8to4(ctx.Fr, [op0, op1, op2, op3, op4, op5, op6, op7 ]), ctx.Fr)) {
                throw new Error(`Storage write does not match: ${ctx.ln}`);
            }

            // commented since readings are also done directly in the smt
            // ctx.sto[ ctx.lastSWrite.keyS ] = fea2scalar(Fr, ctx.D).toString(16).padStart(64, "0");
            for (let k=0; k<4; k++) {
                pols.sKeyI[k][i] =  ctx.lastSWrite.keyI[k];
                pols.sKey[k][i] = ctx.lastSWrite.key[k];
            }
        } else {
            pols.sWR[i] = 0n;
        }

        if ((!l.sRD) && (!l.sWR)) {
            for (let k=0; k<4; k++) {
                pols.sKeyI[k][i] =  Fr.zero;
                pols.sKey[k][i] = Fr.zero;
            }
        }


        if (l.hashK) {
            if (typeof ctx.hashK[addr] === "undefined") ctx.hashK[addr] = { data: [], reads: {} };
            pols.hashK[i] = 1n;
            const size = fe2n(Fr, ctx.D[0], ctx);
            const pos = fe2n(Fr, ctx.HASHPOS, ctx);
            if ((size<0) || (size>32)) throw new Error(`Invalid size for hashK: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            const a = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
            const maskByte = Scalar.e("0xFF");
            for (let k=0; k<size; k++) {
                const bm = Scalar.toNumber(Scalar.band( Scalar.shr( a, (size-k -1)*8 ) , maskByte));
                const bh = ctx.hashK[addr].data[pos + k];
                if (typeof bh === "undefined") {
                    ctx.hashK[addr].data[pos + k] = bm;
                } else if (bm != bh) {
                    throw new Error(`HashK do not match ${addr}:${pos+k} is ${bm} and should be ${bh}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`)
                }
            }

            const paddingA = Scalar.shr(a, size * 8);
            if (!Scalar.isZero(paddingA)) {
                throw new Error(`Incoherent size (${size}) and data (0x${a.toString(16)}) padding (0x${paddingA.toString(16)}) for hashK (w=${step}): ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            }

            if ((typeof ctx.hashK[addr].reads[pos] !== "undefined") &&
                (ctx.hashK[addr].reads[pos] != size))
            {
                throw new Error(`HashK diferent read sizes in the same position ${addr}:${pos}`)
            }
            ctx.hashK[addr].reads[pos] = size;
            incHashPos = size;
        } else {
            pols.hashK[i] = 0n;
        }

        if (l.hashKLen) {
            pols.hashKLen[i] = 1n;
            const lm = fe2n(Fr, op0, ctx);
            // If it's undefined compute hash 0f 0 bytes
            if(typeof ctx.hashK[addr] === "undefined") {
                // len must be 0
                if (lm != 0) throw new Error(`HashK length does not match ${addr}  is ${lm} and should be ${0}`);
                ctx.hashK[addr] = { data: [], reads: {} };
                ctx.hashK[addr].digest = ethers.utils.keccak256("0x");
            }
            const lh = ctx.hashK[addr].data.length;
            if (lm != lh) throw new Error(`HashK length does not match ${addr}  is ${lm} and should be ${lh}`);
            if (typeof ctx.hashK[addr].digest === "undefined") {
                ctx.hashK[addr].digest = ethers.utils.keccak256(ethers.utils.hexlify(ctx.hashK[addr].data));
            }
        } else {
            pols.hashKLen[i] = 0n;
        }

        if (l.hashKDigest) {
            pols.hashKDigest[i] = 1n;
            const dg = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
            if (typeof ctx.hashK[addr].digest === "undefined") {
                throw new Error(`Cannnot load keccak from DB`);
            }
            if (!Scalar.eq(Scalar.e(dg), Scalar.e(ctx.hashK[addr].digest))) {
                throw new Error(`Digest doesn't match`);
            }
            incCounter = Math.ceil((ctx.hashK[addr].data.length + 1) / 136)
        } else {
            pols.hashKDigest[i] = 0n;
        }

        if (l.hashP) {
            if (typeof ctx.hashP[addr] === "undefined") ctx.hashP[addr] = { data: [], reads: {} };
            pols.hashP[i] = 1n;
            const size = fe2n(Fr, ctx.D[0], ctx);
            const pos = fe2n(Fr, ctx.HASHPOS, ctx);
            if ((size<0) || (size>32)) throw new Error(`Invalid size for hash: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            const a = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
            const maskByte = Scalar.e("0xFF");
            for (let k=0; k<size; k++) {
                const bm = Scalar.toNumber(Scalar.band( Scalar.shr( a, (size-k -1)*8 ) , maskByte));
                const bh = ctx.hashP[addr].data[pos + k];
                if (typeof bh === "undefined") {
                    ctx.hashP[addr].data[pos + k] = bm;
                } else if (bm != bh) {
                    throw new Error(`HashP do not match ${addr}:${pos+k} is ${bm} and should be ${bh}`)
                }
            }
            const paddingA = Scalar.shr(a, size * 8);
            if (!Scalar.isZero(paddingA)) {
                throw new Error(`Incoherent size (${size}) and data (0x${a.toString(16)}) padding (0x${paddingA.toString(16)}) for hashP (w=${step}): ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            }

            if ((typeof ctx.hashP[addr].reads[pos] !== "undefined") &&
                (ctx.hashP[addr].reads[pos] != size))
            {
                throw new Error(`HashP diferent read sizes in the same position ${addr}:${pos}`)
            }
            ctx.hashP[addr].reads[pos] = size;
            incHashPos = size;
        } else {
            pols.hashP[i] = 0n;
        }

        if (l.hashPLen) {
            pols.hashPLen[i] = 1n;
            const lm = fe2n(Fr, op0, ctx);
            const lh = ctx.hashP[addr].data.length;
            if (lm != lh) throw new Error(`HashP length does not match ${addr}  is ${lm} and should be ${lh}`);
            if (typeof ctx.hashP[addr].digest === "undefined") {
                // ctx.hashP[addr].digest = poseidonLinear(ctx.hash[addr].data);
                ctx.hashP[addr].digest = await hashContractBytecode(byteArray2HexString(ctx.hashP[addr].data));
                await db.setProgram(stringToH4(ctx.hashP[addr].digest), ctx.hashP[addr].data)
            }
        } else {
            pols.hashPLen[i] = 0n;
        }

        if (l.hashPDigest) {
            pols.hashPDigest[i] = 1n;
            const dg = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
            if (typeof ctx.hashP[addr] === "undefined") {
                const k = scalar2h4(dg);
                const data = await smt.db.getProgram(k);

                ctx.hashP[addr] = {
                    data: data,
                    digest: dg
                }
            }
            incCounter = Math.ceil((ctx.hashP[addr].data.length + 1) / 56);
            if (!Scalar.eq(Scalar.e(dg), Scalar.e(ctx.hashP[addr].digest))) {
                throw new Error(`Digest doesn't match`);
            }
        } else {
            pols.hashPDigest[i] = 0n;
        }

        if (l.hashPDigest || l.sWR) {
            const op = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
            required.Binary.push({a: op, b: 0n, c: op, opcode: 1});
        }

        if (l.arith) {
            if (l.arithEq0 && (!l.arithEq1) && (!l.arithEq2) && (!l.arithEq3)) {
                const A = fea2scalar(Fr, ctx.A);
                const B = fea2scalar(Fr, ctx.B);
                const C = fea2scalar(Fr, ctx.C);
                const D = fea2scalar(Fr, ctx.D);
                const op = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                if (! Scalar.eq(Scalar.add(Scalar.mul(A, B), C),  Scalar.add(Scalar.shl(D, 256), op))   ) {
                    console.log('A: '+A.toString()+' (0x'+A.toString(16)+')');
                    console.log('B: '+B.toString()+' (0x'+B.toString(16)+')');
                    console.log('C: '+C.toString()+' (0x'+C.toString(16)+')');
                    console.log('D: '+D.toString()+' (0x'+D.toString(16)+')');
                    console.log('op: '+op.toString()+' (0x'+op.toString(16)+')');
                    let left = Scalar.add(Scalar.mul(A, B), C);
                    let right = Scalar.add(Scalar.shl(D, 256), op);
                    console.log(left.toString() + ' (0x'+left.toString(16)+') != '+ right.toString()
                                                + ' (0x' + right.toString(16)+')');
                    throw new Error(`Arithmetic does not match: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }
                pols.arith[i] = 1n;
                pols.arithEq0[i] = 1n;
                pols.arithEq1[i] = 0n;
                pols.arithEq2[i] = 0n;
                pols.arithEq3[i] = 0n;
                required.Arith.push({x1: A, y1: B, x2: C, y2: D, x3: Fr.zero, y3: op, selEq0: 1, selEq1: 0, selEq2: 0, selEq3: 0});
            }
            else {
                const x1 = fea2scalar(Fr, ctx.A);
                const y1 = fea2scalar(Fr, ctx.B);
                const x2 = fea2scalar(Fr, ctx.C);
                const y2 = fea2scalar(Fr, ctx.D);
                const x3 = fea2scalar(Fr, ctx.E);
                const y3 = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                let dbl = false;
                if ((!l.arithEq0) && l.arithEq1 && (!l.arithEq2) && l.arithEq3) {
                    dbl = false;
                } else if ((!l.arithEq0) && (!l.arithEq1) && l.arithEq2 && l.arithEq3) {
                    dbl = true;
                } else {
                    throw new Error(`Invalid arithmetic op: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }

                let s;
                if (dbl) {
                    // TODO: y1 == 0 => division by zero ==> how manage?
                    s = Fec.div(Fec.mul(3n, Fec.mul(x1, x1)), Fec.add(y1, y1));
                }
                else {
                    let deltaX = Fec.sub(x2, x1)
                    // TODO: deltaX == 0 => division by zero ==> how manage?
                    s = Fec.div(Fec.sub(y2, y1), deltaX );
                }

                const _x3 = Fec.sub(Fec.mul(s, s), Fec.add(x1, dbl ? x1 : x2));
                const _y3 = Fec.sub(Fec.mul(s, Fec.sub(x1,x3)), y1);
                const x3eq = Scalar.eq(x3, _x3);
                const y3eq = Scalar.eq(y3, _y3);

                if (!x3eq || !x3eq) {
                    console.log('x1,y1: ('+x1.toString()+', '+y1.toString()+')');
                    if (!dbl) {
                        console.log('x2,y2: ('+x2.toString()+', '+y2.toString()+')');
                    }

                    console.log('x3: '+x3.toString()+(x3eq ? ' == ' : ' != ')+_x3.toString());
                    console.log('y3: '+y3.toString()+(y3eq ? ' == ' : ' != ')+_y3.toString());

                    throw new Error('Arithmetic curve '+(dbl?'dbl':'add')+` point does not match: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }

                pols.arith[i] = 1n;
                pols.arithEq0[i] = 0n;
                pols.arithEq1[i] = dbl ? 0n : 1n;
                pols.arithEq2[i] = dbl ? 1n : 0n;
                pols.arithEq3[i] = 1n;
                required.Arith.push({x1: x1, y1: y1, x2: dbl ? x1:x2, y2: dbl? y1:y2, x3: x3, y3: y3, selEq0: 0, selEq1: dbl ? 0 : 1, selEq2: dbl ? 1 : 0, selEq3: 1});
            }
        } else {
            pols.arith[i] = 0n;
            pols.arithEq0[i] = 0n;
            pols.arithEq1[i] = 0n;
            pols.arithEq2[i] = 0n;
            pols.arithEq3[i] = 0n;
        }

        if (l.bin) {
            if (l.binOpcode == 0) { // ADD
                const a = fea2scalar(Fr, ctx.A);
                const b = fea2scalar(Fr, ctx.B);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.band(Scalar.add(a, b), Mask256);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("ADD does not match");
                }
                pols.binOpcode[i] = 0n;
                pols.carry[i] = (((a + b) >> 256n) > 0n) ? 1n : 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 0});
            } else if (l.binOpcode == 1) { // SUB
                const a = fea2scalar(Fr, ctx.A);
                const b = fea2scalar(Fr, ctx.B);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.band(Scalar.add(Scalar.sub(a, b), twoTo256), Mask256);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("SUB does not match");
                }
                pols.binOpcode[i] = 1n;
                pols.carry[i] = ((a - b) < 0n) ? 1n : 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 1});
            } else if (l.binOpcode == 2) { // LT
                const a = fea2scalar(Fr, ctx.A);
                const b = fea2scalar(Fr, ctx.B);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.lt(a, b);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("LT does not match");
                }
                pols.binOpcode[i] = 2n;
                pols.carry[i] = (a < b) ? 1n: 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 2});
            } else if (l.binOpcode == 3) { // SLT
                let a = Scalar.e(fea2scalar(Fr, ctx.A));
                if (Scalar.geq(a, twoTo255)) a = Scalar.sub(a, twoTo256);
                let b = Scalar.e(fea2scalar(Fr, ctx.B));
                if (Scalar.geq(b, twoTo255)) b = Scalar.sub(b, twoTo256);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.lt(a, b);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("SLT does not match");
                }
                pols.binOpcode[i] = 3n;
                pols.carry[i] = (a < b) ? 1n : 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 3});
            } else if (l.binOpcode == 4) { // EQ
                const a = fea2scalar(Fr, ctx.A);
                const b = fea2scalar(Fr, ctx.B);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.eq(a, b);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("EQ does not match");
                }
                pols.binOpcode[i] = 4n;
                pols.carry[i] = (a ==  b) ? 1n : 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 4});
            } else if (l.binOpcode == 5) { // AND
                const a = fea2scalar(Fr, ctx.A);
                const b = fea2scalar(Fr, ctx.B);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.band(a, b);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("AND does not match");
                }
                pols.binOpcode[i] = 5n;
                pols.carry[i] = 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 5});
            } else if (l.binOpcode == 6) { // OR
                const a = fea2scalar(Fr, ctx.A);
                const b = fea2scalar(Fr, ctx.B);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.bor(a, b);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("OR does not match");
                }
                pols.binOpcode[i] = 6n;
                pols.carry[i] = 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 6});
            } else if (l.binOpcode == 7) { // XOR
                const a = fea2scalar(Fr, ctx.A);
                const b = fea2scalar(Fr, ctx.B);
                const c = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
                const expectedC = Scalar.bxor(a, b);
                if (!Scalar.eq(c, expectedC)) {
                    throw new Error("XOR does not match");
                }
                pols.binOpcode[i] = 7n;
                pols.carry[i] = 0n;
                required.Binary.push({a: a, b: b, c: c, opcode: 7});
            } else {
                throw new Error("Invalid bin opcode");
            }
            pols.bin[i] = 1n;
        } else {
            pols.bin[i] = 0n;
            pols.binOpcode[i] = 0n;
            pols.carry[i] = 0n;
        }

        if (l.memAlign == 1) {
            const m0 = fea2scalar(Fr, ctx.A);
            const v = fea2scalar(Fr, [op0, op1, op2, op3, op4, op5, op6, op7]);
            const P2_256 = 2n ** 256n;
            const MASK_256 = P2_256 - 1n;
            const offset = fea2scalar(Fr, ctx.C);

            if (offset < 0 || offset >= 32) {
                throw new Error(`MemAlign out of range (${offset}): ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            }

            if (l.memAlignWR && !l.memAlignWR8) {
                const m1 = fea2scalar(Fr, ctx.B);
                const w0 = fea2scalar(Fr, ctx.D);
                const w1 = fea2scalar(Fr, ctx.E);
                const _W0 = Scalar.bor(Scalar.band(m0, P2_256 - (2n ** (256n - (8n * offset)))), Scalar.shr(v, 8n * offset));
                const _W1 = Scalar.bor(Scalar.band(m1, MASK_256 >> (offset * 8n)),
                                       Scalar.band(Scalar.shl(v, (256n - (offset * 8n))), MASK_256));
                if (!Scalar.eq(w0, _W0) || !Scalar.eq(w1, _W1) ) {
                    throw new Error(`MemAlign w0,w1 invalid (0x${w0.toString(16)},0x${w1.toString(16)}) vs (0x${_W0.toString(16)},0x${_W1.toString(16)})`+
                                    `[m0:${m0.toString(16)}, m1:${m1.toString(16)}, v:${v.toString(16)}, offset:${offset}]: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }
                pols.memAlign[i] = 1n;
                pols.memAlignWR[i] = 1n;
                pols.memAlignWR8[i] = 0n;
                required.MemAlign.push({m0: m0, m1: m1, v: v, w0: w0, w1: w1, offset: offset, wr256: 1n, wr8: 0n});
            }
            else if (!l.memAlignWR && l.memAlignWR8) {
                const w0 = fea2scalar(Fr, ctx.D);
                const _W0 = Scalar.bor(Scalar.band(m0, Scalar.shr(byteMaskOn256, 8n * offset)), Scalar.shl(Scalar.band(v, 0xFF), 8n * (31n - offset)));
                if (!Scalar.eq(w0, _W0)) {
                    throw new Error(`MemAlign w0 invalid (0x${w0.toString(16)}) vs (0x${_W0.toString(16)})`+
                                    `[m0:${m0.toString(16)}, v:${v.toString(16)}, offset:${offset}]: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }
                pols.memAlign[i] = 1n;
                pols.memAlignWR[i] = 0n;
                pols.memAlignWR8[i] = 1n;
                required.MemAlign.push({m0: m0, m1: 0n, v: v, w0: w0, w1: 0n, offset: offset, wr256: 0n, wr8: 1n});
            } else if (!l.memAlignWR && !l.memAlignWR8) {
                const m1 = fea2scalar(Fr, ctx.B);
                const leftV = Scalar.band(Scalar.shl(m0, offset * 8n), MASK_256);
                const rightV = Scalar.band(Scalar.shr(m1, 256n - (offset * 8n)), MASK_256 >> (256n - (offset * 8n)));
                const _V = Scalar.bor(leftV, rightV);
                if (!Scalar.eq(v, _V)) {
                    throw new Error(`MemAlign v invalid ${v.toString(16)} vs ${_V.toString(16)}:`+
                                    `[m0:${m0.toString(16)}, m1:${m1.toString(16)}, offset:${offset}]: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
                }
                pols.memAlign[i] = 1n;
                pols.memAlignWR[i] = 0n;
                pols.memAlignWR8[i] = 0n;
                required.MemAlign.push({m0: m0, m1: m1, v: v, w0: Fr.zero, w1: Fr.zero, offset: offset, wr256: 0n, wr8: 0n});
            } else {
                throw new Error(`Invalid memAlign operation (wr: ${l.memAlignWR}, wr8: ${l.memAlignWR8}): ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
            }
        } else {
            pols.memAlign[i] = 0n;
            pols.memAlignWR[i] = 0n;
            pols.memAlignWR8[i] = 0n;
        }

    //////////
    // SET NEXT REGISTERS
    //////////

        const nexti = (i+1) % N;

        if (l.setA == 1) {
            pols.setA[i]=1n;
            [pols.A0[nexti],
             pols.A1[nexti],
             pols.A2[nexti],
             pols.A3[nexti],
             pols.A4[nexti],
             pols.A5[nexti],
             pols.A6[nexti],
             pols.A7[nexti]
            ] = [op0, op1, op2, op3, op4, op5, op6, op7];
        } else {
            pols.setA[i]=0n;
            [pols.A0[nexti],
             pols.A1[nexti],
             pols.A2[nexti],
             pols.A3[nexti],
             pols.A4[nexti],
             pols.A5[nexti],
             pols.A6[nexti],
             pols.A7[nexti]
            ] = [
             pols.A0[i],
             pols.A1[i],
             pols.A2[i],
             pols.A3[i],
             pols.A4[i],
             pols.A5[i],
             pols.A6[i],
             pols.A7[i]
            ];

            // Set A register with input.from to process unsigned transactions
            if ((Number(ctx.zkPC) === rom.labels.checkAndSaveFrom) && config.unsigned){
                const feaFrom = scalar2fea(Fr, input.from);
                [pols.A0[nexti],
                 pols.A1[nexti],
                 pols.A2[nexti],
                 pols.A3[nexti],
                 pols.A4[nexti],
                 pols.A5[nexti],
                 pols.A6[nexti],
                 pols.A7[nexti]
                ] = [feaFrom[0], feaFrom[1], feaFrom[2], feaFrom[3], feaFrom[4], feaFrom[5], feaFrom[6], feaFrom[7]];
            }
        }

        if (l.setB == 1) {
            pols.setB[i]=1n;
            [pols.B0[nexti],
             pols.B1[nexti],
             pols.B2[nexti],
             pols.B3[nexti],
             pols.B4[nexti],
             pols.B5[nexti],
             pols.B6[nexti],
             pols.B7[nexti]
            ] = [op0, op1, op2, op3, op4, op5, op6, op7];
        } else {
            pols.setB[i]=0n;
            [pols.B0[nexti],
             pols.B1[nexti],
             pols.B2[nexti],
             pols.B3[nexti],
             pols.B4[nexti],
             pols.B5[nexti],
             pols.B6[nexti],
             pols.B7[nexti]
            ] = [
             pols.B0[i],
             pols.B1[i],
             pols.B2[i],
             pols.B3[i],
             pols.B4[i],
             pols.B5[i],
             pols.B6[i],
             pols.B7[i]
            ];
        }

        if (l.setC == 1) {
            pols.setC[i]=1n;
            [pols.C0[nexti],
             pols.C1[nexti],
             pols.C2[nexti],
             pols.C3[nexti],
             pols.C4[nexti],
             pols.C5[nexti],
             pols.C6[nexti],
             pols.C7[nexti]
            ] = [op0, op1, op2, op3, op4, op5, op6, op7];
        } else {
            pols.setC[i]=0n;
            [pols.C0[nexti],
             pols.C1[nexti],
             pols.C2[nexti],
             pols.C3[nexti],
             pols.C4[nexti],
             pols.C5[nexti],
             pols.C6[nexti],
             pols.C7[nexti]
            ] = [
             pols.C0[i],
             pols.C1[i],
             pols.C2[i],
             pols.C3[i],
             pols.C4[i],
             pols.C5[i],
             pols.C6[i],
             pols.C7[i]
            ];
        }

        if (l.setD == 1) {
            pols.setD[i]=1n;
            [pols.D0[nexti],
             pols.D1[nexti],
             pols.D2[nexti],
             pols.D3[nexti],
             pols.D4[nexti],
             pols.D5[nexti],
             pols.D6[nexti],
             pols.D7[nexti]
            ] = [op0, op1, op2, op3, op4, op5, op6, op7];
        } else {
            pols.setD[i]=0n;
            [pols.D0[nexti],
             pols.D1[nexti],
             pols.D2[nexti],
             pols.D3[nexti],
             pols.D4[nexti],
             pols.D5[nexti],
             pols.D6[nexti],
             pols.D7[nexti]
            ] = [
             pols.D0[i],
             pols.D1[i],
             pols.D2[i],
             pols.D3[i],
             pols.D4[i],
             pols.D5[i],
             pols.D6[i],
             pols.D7[i]
            ];
        }

        if (l.setE == 1) {
            pols.setE[i]=1n;
            [pols.E0[nexti],
             pols.E1[nexti],
             pols.E2[nexti],
             pols.E3[nexti],
             pols.E4[nexti],
             pols.E5[nexti],
             pols.E6[nexti],
             pols.E7[nexti]
            ] = [op0, op1, op2, op3, op4, op5, op6, op7];
        } else {
            pols.setE[i]=0n;
            [pols.E0[nexti],
             pols.E1[nexti],
             pols.E2[nexti],
             pols.E3[nexti],
             pols.E4[nexti],
             pols.E5[nexti],
             pols.E6[nexti],
             pols.E7[nexti]
            ] = [
             pols.E0[i],
             pols.E1[i],
             pols.E2[i],
             pols.E3[i],
             pols.E4[i],
             pols.E5[i],
             pols.E6[i],
             pols.E7[i]
            ];
        }


        if (l.setSR == 1) {
            pols.setSR[i]=1n;
            [pols.SR0[nexti],
             pols.SR1[nexti],
             pols.SR2[nexti],
             pols.SR3[nexti],
             pols.SR4[nexti],
             pols.SR5[nexti],
             pols.SR6[nexti],
             pols.SR7[nexti]
            ] = [op0, op1, op2, op3, op4, op5, op6, op7];
        } else {
            pols.setSR[i]=0n;
            [pols.SR0[nexti],
             pols.SR1[nexti],
             pols.SR2[nexti],
             pols.SR3[nexti],
             pols.SR4[nexti],
             pols.SR5[nexti],
             pols.SR6[nexti],
             pols.SR7[nexti]
            ] = [
             pols.SR0[i],
             pols.SR1[i],
             pols.SR2[i],
             pols.SR3[i],
             pols.SR4[i],
             pols.SR5[i],
             pols.SR6[i],
             pols.SR7[i]
            ];
        }

        if (l.setCTX == 1) {
            pols.setCTX[i]=1n;
            pols.CTX[nexti] = BigInt(fe2n(Fr, op0, ctx));
        } else {
            pols.setCTX[i]=0n;
            pols.CTX[nexti] = pols.CTX[i];
        }

        if (l.setSP == 1) {
            pols.setSP[i]=1n;
            pols.SP[nexti] = BigInt(fe2n(Fr, op0, ctx));
        } else {
            pols.setSP[i]=0n;
            pols.SP[nexti] = pols.SP[i] + BigInt((l.incStack || 0));
        }

        if (l.setPC == 1) {
            pols.setPC[i]=1n;
            pols.PC[nexti] = BigInt(fe2n(Fr, op0, ctx));
        } else {
            pols.setPC[i]=0n;
            pols.PC[nexti] = pols.PC[i] + BigInt((l.incCode || 0));
        }

        if (l.setRR == 1) {
            pols.setRR[i]=1n;
            pols.RR[nexti] = BigInt(fe2n(Fr, op0, ctx));
        } else {
            pols.setRR[i]=0n;
            pols.RR[nexti] = pols.RR[i];
        }

        if (l.arith == 1) {
            pols.cntArith[nexti] = pols.cntArith[i] + 1n;
        } else {
            pols.cntArith[nexti] = pols.cntArith[i];
        }

        if (l.bin == 1) {
            pols.cntBinary[nexti] = pols.cntBinary[i] + 1n;
        } else {
            pols.cntBinary[nexti] = pols.cntBinary[i];
        }

        if (l.memAlign == 1) {
            pols.cntMemAlign[nexti] = pols.cntMemAlign[i] + 1n;
        } else {
            pols.cntMemAlign[nexti] = pols.cntMemAlign[i];
        }


        if (l.JMPN) {
            const o = fe2n(Fr, op0, ctx);
            if (o<0) {
                pols.isNeg[i]=1n;
                pols.zkPC[nexti] = BigInt(addr);
                required.Byte4[0x100000000 + o] = true;
            } else {
                pols.isNeg[i]=0n;
                pols.zkPC[nexti] = pols.zkPC[i] + 1n;
                required.Byte4[o] = true;
            }
            pols.JMP[i] = 0n;
            pols.JMPN[i] = 1n;
            pols.JMPC[i] = 0n;
        } else if (l.JMPC) {
            if (pols.carry[i]) {
                pols.zkPC[nexti] = BigInt(addr);
            } else {
                pols.zkPC[nexti] = pols.zkPC[i] + 1n;
            }
            pols.isNeg[i]=0n;
            pols.JMP[i] = 0n;
            pols.JMPN[i] = 0n;
            pols.JMPC[i] = 1n;
        } else if (l.JMP) {
            pols.isNeg[i]=0n;
            pols.zkPC[nexti] = BigInt(addr);
            pols.JMP[i] = 1n;
            pols.JMPN[i] = 0n;
            pols.JMPC[i] = 0n;
        } else {
            pols.isNeg[i]=0n;
            pols.zkPC[nexti] = pols.zkPC[i] + 1n;
            pols.JMP[i] = 0n;
            pols.JMPN[i] = 0n;
            pols.JMPC[i] = 0n;
        }

        let maxMemCalculated;
        const mm = pols.MAXMEM[i];
        if (l.isMem) {
            if (addrRel>mm) {
                pols.isMaxMem[i] = 1n;
                maxMemCalculated = addrRel;
            } else {
                pols.isMaxMem[i] = 0n;
                maxMemCalculated = mm;
            }
        } else {
            pols.isMaxMem[i] = 0n;
            maxMemCalculated = mm;
        }

        if (l.setMAXMEM) {
            pols.setMAXMEM[i] = 1n;
            pols.MAXMEM[nexti] = BigInt(fe2n(Fr, op0, ctx));
        } else {
            pols.setMAXMEM[i] = 0n;
            pols.MAXMEM[nexti] = BigInt(maxMemCalculated);
        }

        if (l.setGAS == 1) {
            pols.setGAS[i]=1n;
            pols.GAS[nexti] = BigInt(fe2n(Fr, op0, ctx));
        } else {
            pols.setGAS[i]=0n;
            pols.GAS[nexti] = pols.GAS[i];
        }

        if (l.setHASHPOS == 1) {
            pols.setHASHPOS[i]=1n;
            pols.HASHPOS[nexti] = BigInt(fe2n(Fr, op0, ctx) + incHashPos);
        } else {
            pols.setHASHPOS[i]=0n;
            pols.HASHPOS[nexti] = pols.HASHPOS[i] + BigInt( incHashPos);
        }

        if (l.sRD || l.sWR || l.hashKDigest || l.hashPDigest) {
            pols.incCounter[i] = Fr.e(incCounter);
        } else {
            pols.incCounter[i] = Fr.zero;
        }
        // Setting current value of counters to next step


        if (l.hashKDigest) {
            pols.cntKeccakF[nexti] = pols.cntKeccakF[i] + BigInt(incCounter);
        } else {
            pols.cntKeccakF[nexti] = pols.cntKeccakF[i];
        }

        if (l.hashPDigest) {
            pols.cntPaddingPG[nexti] = pols.cntPaddingPG[i] + BigInt(incCounter);
        } else {
            pols.cntPaddingPG[nexti] = pols.cntPaddingPG[i];
        }

        if (l.sRD || l.sWR || l.hashPDigest) {
            pols.cntPoseidonG[nexti] = pols.cntPoseidonG[i] + BigInt(incCounter);
        } else {
            pols.cntPoseidonG[nexti] = pols.cntPoseidonG[i];
        }

        if (l.cmdAfter) {
            for (let j=0; j< l.cmdAfter.length; j++) {
                evalCommand(ctx, l.cmdAfter[j]);
            }
        }
    }

    if (!debug || !config.stepsN || !fastDebugExit) {
        checkFinalState(Fr, pols);
    }

    for (let i=0; i<ctx.hashK.length; i++) {
        const h = {
            data: ctx.hashK[i].data,
            reads: []
        }
        let p= 0;
        while (p<ctx.hashK[i].data.length) {
            if (ctx.hashK[i].reads[p]) {
                h.reads.push(ctx.hashK[i].reads[p]);
                p += ctx.hashK[i].reads[p];
            } else {
                h.reads.push(1);
                p += 1;
            }
        }
        if (p!= ctx.hashK[i].data.length) {
            throw new Error(`Reading hashK out of limits: ${step}`);
        }
        required.PaddingKK.push(h);
    }

    for (let i=0; i<ctx.hashP.length; i++) {
        const h = {
            data: ctx.hashP[i].data,
            reads: []
        }
        let p= 0;
        while (p<ctx.hashP[i].data.length) {
            if (ctx.hashP[i].reads[p]) {
                h.reads.push(ctx.hashP[i].reads[p]);
                p += ctx.hashP[i].reads[p];
            } else {
                h.reads.push(1);
                p += 1;
            }
        }
        if (p!= ctx.hashP[i].data.length) {
            throw new Error(`Reading hashP out of limits: ${step}`);
        }
        required.PaddingPG.push(h);
    }

    required.logs = ctx.outLogs;

    return required;
}


/*
    This function creates an array of polynomials and a mapping that maps the reference name in pil to the polynomial
*/

function checkFinalState(Fr, pols) {
    if (
        (!Fr.isZero(pols.A0[0])) ||
        (!Fr.isZero(pols.A1[0])) ||
        (!Fr.isZero(pols.A2[0])) ||
        (!Fr.isZero(pols.A3[0])) ||
        (!Fr.isZero(pols.A4[0])) ||
        (!Fr.isZero(pols.A5[0])) ||
        (!Fr.isZero(pols.A6[0])) ||
        (!Fr.isZero(pols.A7[0])) ||
        (!Fr.isZero(pols.B0[0])) ||
        (!Fr.isZero(pols.B1[0])) ||
        (!Fr.isZero(pols.B2[0])) ||
        (!Fr.isZero(pols.B3[0])) ||
        (!Fr.isZero(pols.B4[0])) ||
        (!Fr.isZero(pols.B5[0])) ||
        (!Fr.isZero(pols.B6[0])) ||
        (!Fr.isZero(pols.B7[0])) ||
        (!Fr.isZero(pols.C0[0])) ||
        (!Fr.isZero(pols.C1[0])) ||
        (!Fr.isZero(pols.C2[0])) ||
        (!Fr.isZero(pols.C3[0])) ||
        (!Fr.isZero(pols.C4[0])) ||
        (!Fr.isZero(pols.C5[0])) ||
        (!Fr.isZero(pols.C6[0])) ||
        (!Fr.isZero(pols.C7[0])) ||
        (!Fr.isZero(pols.D0[0])) ||
        (!Fr.isZero(pols.D1[0])) ||
        (!Fr.isZero(pols.D2[0])) ||
        (!Fr.isZero(pols.D3[0])) ||
        (!Fr.isZero(pols.D4[0])) ||
        (!Fr.isZero(pols.D5[0])) ||
        (!Fr.isZero(pols.D6[0])) ||
        (!Fr.isZero(pols.D7[0])) ||
        (!Fr.isZero(pols.E0[0])) ||
        (!Fr.isZero(pols.E1[0])) ||
        (!Fr.isZero(pols.E2[0])) ||
        (!Fr.isZero(pols.E3[0])) ||
        (!Fr.isZero(pols.E4[0])) ||
        (!Fr.isZero(pols.E5[0])) ||
        (!Fr.isZero(pols.E6[0])) ||
        (!Fr.isZero(pols.E7[0])) ||
        (!Fr.isZero(pols.SR0[0])) ||
        (!Fr.isZero(pols.SR1[0])) ||
        (!Fr.isZero(pols.SR2[0])) ||
        (!Fr.isZero(pols.SR3[0])) ||
        (!Fr.isZero(pols.SR4[0])) ||
        (!Fr.isZero(pols.SR5[0])) ||
        (!Fr.isZero(pols.SR6[0])) ||
        (!Fr.isZero(pols.SR7[0])) ||
        (pols.CTX[0]) ||
        (pols.SP[0]) ||
        (pols.PC[0]) ||
        (pols.MAXMEM[0]) ||
        (pols.GAS[0]) ||
        (pols.zkPC[0])
    ) {
        throw new Error("Program terminated with registers not set to zero");
    }

}


function initState(Fr, pols) {
    // Register value initial parameters
    pols.A0[0] = Fr.zero;
    pols.A1[0] = Fr.zero;
    pols.A2[0] = Fr.zero;
    pols.A3[0] = Fr.zero;
    pols.A4[0] = Fr.zero;
    pols.A5[0] = Fr.zero;
    pols.A6[0] = Fr.zero;
    pols.A7[0] = Fr.zero;
    pols.B0[0] = Fr.zero;
    pols.B1[0] = Fr.zero;
    pols.B2[0] = Fr.zero;
    pols.B3[0] = Fr.zero;
    pols.B4[0] = Fr.zero;
    pols.B5[0] = Fr.zero;
    pols.B6[0] = Fr.zero;
    pols.B7[0] = Fr.zero;
    pols.C0[0] = Fr.zero;
    pols.C1[0] = Fr.zero;
    pols.C2[0] = Fr.zero;
    pols.C3[0] = Fr.zero;
    pols.C4[0] = Fr.zero;
    pols.C5[0] = Fr.zero;
    pols.C6[0] = Fr.zero;
    pols.C7[0] = Fr.zero;
    pols.D0[0] = Fr.zero;
    pols.D1[0] = Fr.zero;
    pols.D2[0] = Fr.zero;
    pols.D3[0] = Fr.zero;
    pols.D4[0] = Fr.zero;
    pols.D5[0] = Fr.zero;
    pols.D6[0] = Fr.zero;
    pols.D7[0] = Fr.zero;
    pols.E0[0] = Fr.zero;
    pols.E1[0] = Fr.zero;
    pols.E2[0] = Fr.zero;
    pols.E3[0] = Fr.zero;
    pols.E4[0] = Fr.zero;
    pols.E5[0] = Fr.zero;
    pols.E6[0] = Fr.zero;
    pols.E7[0] = Fr.zero;
    pols.SR0[0] = Fr.zero;
    pols.SR1[0] = Fr.zero;
    pols.SR2[0] = Fr.zero;
    pols.SR3[0] = Fr.zero;
    pols.SR4[0] = Fr.zero;
    pols.SR5[0] = Fr.zero;
    pols.SR6[0] = Fr.zero;
    pols.SR7[0] = Fr.zero;
    pols.CTX[0] = 0n;
    pols.SP[0] = 0n;
    pols.PC[0] = 0n;
    pols.MAXMEM[0] = 0n;
    pols.HASHPOS[0] = 0n;
    pols.GAS[0] = 0n;
    pols.RR[0] = 0n;
    pols.zkPC[0] = 0n;
    pols.cntArith[0] = 0n;
    pols.cntBinary[0] = 0n;
    pols.cntKeccakF[0] = 0n;
    pols.cntMemAlign[0] = 0n;
    pols.cntPaddingPG[0] = 0n;
    pols.cntPoseidonG[0] = 0n;
}

function evalCommand(ctx, tag) {
    if (tag.op == "number") {
        return eval_number(ctx, tag);
    } else if (tag.op == "declareVar") {
        return eval_declareVar(ctx, tag);
    } else if (tag.op == "setVar") {
        return eval_setVar(ctx, tag);
    } else if (tag.op == "getVar") {
        return eval_getVar(ctx, tag);
    } else if (tag.op == "getReg") {
        return eval_getReg(ctx, tag);
    } else if (tag.op == "functionCall") {
        return eval_functionCall(ctx, tag);
    } else if (tag.op == "add") {
        return eval_add(ctx, tag);
    } else if (tag.op == "sub") {
        return eval_sub(ctx, tag);
    } else if (tag.op == "neg") {
        return eval_neg(ctx, tag);
    } else if (tag.op == "mul") {
        return eval_mul(ctx, tag);
    } else if (tag.op == "div") {
        return eval_div(ctx, tag);
    } else if (tag.op == "mod") {
        return eval_mod(ctx, tag);
    } else if (tag.op == "or" || tag.op == "and" || tag.op == "gt" || tag.op == "ge" || tag.op == "lt" || tag.op == "le" ||
               tag.op == "eq" || tag.op == "ne" || tag.op == "not" ) {
        return eval_logical_operation(ctx, tag);
    } else if (tag.op == "bitand" || tag.op == "bitor" || tag.op == "bitxor" || tag.op == "bitnot"|| tag.op == "shl" || tag.op == "shr") {
        return eval_bit_operation(ctx, tag);
    } else if (tag.op == "if") {
        return eval_if(ctx, tag);
    } else if (tag.op == "getMemValue") {
        return eval_getMemValue(ctx, tag);
    } else {
        throw new Error(`Invalid operation ${tag.op}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    }

}

function eval_number(ctx, tag) {
    return Scalar.e(tag.num);
}


function eval_setVar(ctx, tag) {

    const varName = eval_left(ctx, tag.values[0]);

    if (typeof ctx.vars[varName] == "undefined") throw new Error(`Variable not defined ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);

    ctx.vars[varName] = evalCommand(ctx, tag.values[1]);
    return ctx.vars[varName];
}

function eval_left(ctx, tag) {
    if (tag.op == "declareVar") {
        eval_declareVar(ctx, tag);
        return tag.varName;
    } else if (tag.op == "getVar") {
        return tag.varName;
    } else {
        throw new Error(`Invalid left expression: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    }
}

function eval_declareVar(ctx, tag) {
    // local variables, redeclared must start with _
    if (tag.varName[0] !== '_' && typeof ctx.vars[tag.varName] != "undefined") {
        throw new Error(`Variable already declared: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    }
    ctx.vars[tag.varName] = Scalar.e(0);
    return ctx.vars[tag.varName];
}

function eval_getVar(ctx, tag) {
    if (typeof ctx.vars[tag.varName] == "undefined") throw new Error(`Variable not defined ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return ctx.vars[tag.varName];
}

function eval_getReg(ctx, tag) {
    if (tag.regName == "A") {
        return fea2scalar(ctx.Fr, ctx.A);
    } else if (tag.regName == "B") {
        return fea2scalar(ctx.Fr, ctx.B);
    } else if (tag.regName == "C") {
        return fea2scalar(ctx.Fr, ctx.C);
    } else if (tag.regName == "D") {
        return fea2scalar(ctx.Fr, ctx.D);
    } else if (tag.regName == "E") {
        return fea2scalar(ctx.Fr, ctx.E);
    } else if (tag.regName == "SR") {
        return fea2scalar(ctx.Fr, ctx.SR);
    } else if (tag.regName == "CTX") {
        return Scalar.e(ctx.CTX);
    } else if (tag.regName == "SP") {
        return Scalar.e(ctx.SP);
    } else if (tag.regName == "PC") {
        return Scalar.e(ctx.PC);
    } else if (tag.regName == "MAXMEM") {
        return Scalar.e(ctx.MAXMEM);
    } else if (tag.regName == "GAS") {
        return Scalar.e(ctx.GAS);
    } else if (tag.regName == "zkPC") {
        return Scalar.e(ctx.zkPC);
    } else if (tag.regName == "RR") {
        return Scalar.e(ctx.RR);
    } else if (tag.regName == "CNT_ARITH") {
        return Scalar.e(ctx.cntArith);
    } else if (tag.regName == "CNT_BINARY") {
        return Scalar.e(ctx.cntBinary);
    } else if (tag.regName == "CNT_KECCAK_F") {
        return Scalar.e(ctx.cntKeccakF);
    } else if (tag.regName == "CNT_MEM_ALIGN") {
        return Scalar.e(ctx.cntMemAlign);
    } else if (tag.regName == "CNT_PADDING_PG") {
        return Scalar.e(ctx.cntPaddingPG);
    } else if (tag.regName == "CNT_POSEIDON_G") {
        return Scalar.e(ctx.cntPoseidonG);
    } else if (tag.regName == "STEP") {
        return Scalar.e(ctx.STEP);
    } else if (tag.regName == "HASHPOS") {
        return Scalar.e(ctx.HASHPOS);
    } else {
        throw new Error(`Invalid register ${tag.regName}:  ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    }
}

function eval_add(ctx, tag) {
    const a = evalCommand(ctx, tag.values[0]);
    const b = evalCommand(ctx, tag.values[1]);
    return Scalar.add(a,b);
}

function eval_sub(ctx, tag) {
    const a = evalCommand(ctx, tag.values[0]);
    const b = evalCommand(ctx, tag.values[1]);
    return Scalar.sub(a,b);
}

function eval_neg(ctx, tag) {
    const a = evalCommand(ctx, tag.values[0]);
    return Scalar.neg(a);
}

function eval_mul(ctx, tag) {
    const a = evalCommand(ctx, tag.values[0]);
    const b = evalCommand(ctx, tag.values[1]);
    return Scalar.mul(a,b);
}

function eval_div(ctx, tag) {
    const a = evalCommand(ctx, tag.values[0]);
    const b = evalCommand(ctx, tag.values[1]);
    return Scalar.div(a,b);
}

function eval_mod(ctx, tag) {
    const a = evalCommand(ctx, tag.values[0]);
    const b = evalCommand(ctx, tag.values[1]);
    return Scalar.mod(a,b);
}

function eval_bit_operation(ctx, tag)
{
    const a = evalCommand(ctx, tag.values[0]);
    if (tag.op == "bitnot") {
        return ~a;
    }
    const b = evalCommand(ctx, tag.values[1]);
    switch(tag.op) {
        case 'bitor':    return Scalar.bor(a,b);
        case 'bitand':   return Scalar.band(a,b);
        case 'bitxor':   return Scalar.bxor(a,b);
        case 'shl':      return Scalar.shl(a,b);
        case 'shr':      return Scalar.shr(a,b);
    }
    throw new Error(`bit operation ${tag.op} not defined: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
}

function eval_if(ctx, tag)
{
    const a = evalCommand(ctx, tag.values[0]);
    return evalCommand(ctx, tag.values[ a ? 1:2]);
}

function eval_logical_operation(ctx, tag)
{
    const a = evalCommand(ctx, tag.values[0]);
    if (tag.op === "not") {
        return (a)  ? 0 : 1;
    }
    const b = evalCommand(ctx, tag.values[1]);
    switch(tag.op) {
        case 'or':      return (a || b) ? 1 : 0;
        case 'and':     return (a && b) ? 1 : 0;
        case 'eq':      return (a == b) ? 1 : 0;
        case 'ne':      return (a != b) ? 1 : 0;
        case 'gt':      return (a > b)  ? 1 : 0;
        case 'ge':      return (a >= b) ? 1 : 0;
        case 'lt':      return (a < b)  ? 1 : 0;
        case 'le':      return (a > b)  ? 1 : 0;
    }
    throw new Error(`logical operation ${tag.op} not defined: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
}

function eval_getMemValue(ctx, tag) {
    return fea2scalar(ctx.Fr, ctx.mem[tag.offset]);
}

function eval_functionCall(ctx, tag) {
    if (tag.funcName == "getGlobalHash") {
        return eval_getGlobalHash(ctx, tag);
    } else if (tag.funcName == "getOldStateRoot") {
        return eval_getOldStateRoot(ctx, tag);
    } else if (tag.funcName == "getNewStateRoot") {
        return eval_getNewStateRoot(ctx, tag);
    } else if (tag.funcName == "getSequencerAddr") {
        return eval_getSequencerAddr(ctx, tag);
    } else if (tag.funcName == "getOldLocalExitRoot") {
        return eval_getOldLocalExitRoot(ctx, tag);
    } else if (tag.funcName == "getNewLocalExitRoot") {
        return eval_getNewLocalExitRoot(ctx, tag);
    } else if (tag.funcName == "getNumBatch") {
        return eval_getNumBatch(ctx, tag);
    } else if (tag.funcName == "getTimestamp") {
        return eval_getTimestamp(ctx, tag);
    } else if (tag.funcName == "getChainId") {
        return eval_getChainId(ctx, tag);
    } else if (tag.funcName == "getBatchHashData") {
        return eval_getBatchHashData(ctx, tag);
    } else if (tag.funcName == "getGlobalExitRoot") {
        return eval_getGlobalExitRoot(ctx, tag);
    } else if (tag.funcName == "getTxs") {
        return eval_getTxs(ctx, tag);
    } else if (tag.funcName == "getTxsLen") {
        return eval_getTxsLen(ctx, tag);
    } else if (tag.funcName == "eventLog") {
        return eval_eventLog(ctx, tag);
    } else if (tag.funcName == "cond") {
        return eval_cond(ctx, tag);
    } else if (tag.funcName == "inverseFpEc") {
        return eval_inverseFpEc(ctx, tag);
    } else if (tag.funcName == "inverseFnEc") {
        return eval_inverseFnEc(ctx, tag);
    } else if (tag.funcName == "sqrtFpEc") {
        return eval_sqrtFpEc(ctx, tag);
    } else if (tag.funcName == "dumpRegs") {
        return eval_dumpRegs(ctx, tag);
    } else if (tag.funcName == "dump") {
        return eval_dump(ctx, tag);
    } else if (tag.funcName == "dumphex") {
        return eval_dumphex(ctx, tag);
    } else if (tag.funcName == "xAddPointEc") {
        return eval_xAddPointEc(ctx, tag);
    } else if (tag.funcName == "yAddPointEc") {
        return eval_yAddPointEc(ctx, tag);
    } else if (tag.funcName == "xDblPointEc") {
        return eval_xDblPointEc(ctx, tag);
    } else if (tag.funcName == "yDblPointEc") {
        return eval_yDblPointEc(ctx, tag);
    } else if (tag.funcName.startsWith("test")) {
        let method = tag.funcName.charAt(4).toLowerCase() + tag.funcName.slice(5);
        if (typeof testTools[method] === 'function') {
            return testTools[method](ctx, tag);
        }
    } else if (tag.funcName == "getBytecode") { // Added by opcodes
        return eval_getBytecode(ctx, tag);
    } else if (tag.funcName == "beforeLast") {
        return eval_beforeLast(ctx, tag)
    } else if (tag.funcName == "isWarmedAddress") {
        return eval_isWarmedAddress(ctx, tag)
    } else if (tag.funcName == "checkpoint") {
        return eval_checkpoint(ctx, tag)
    } else if (tag.funcName == "revert") {
        return eval_revert(ctx, tag)
    } else if (tag.funcName == "commit") {
        return eval_commit(ctx, tag)
    } else if (tag.funcName == "clearWarmedStorage") {
        return eval_clearWarmedStorage(ctx, tag)
    } else if (tag.funcName == "isWarmedStorage") {
        return eval_isWarmedStorage(ctx, tag)
    } else if (tag.funcName.includes("bitwise")) {
        return eval_bitwise(ctx, tag);
    } else if (tag.funcName.includes("comp") && tag.funcName.split('_')[0] === "comp") {
        return eval_comp(ctx, tag);
    } else if (tag.funcName == "loadScalar") {
        return eval_loadScalar(ctx, tag);
    } else if (tag.funcName == "log") {
        return eval_log(ctx, tag);
    } else if (tag.funcName == "exp") {
        return eval_exp(ctx, tag)
    } else if (tag.funcName == "storeLog") {
        return eval_storeLog(ctx, tag)
    } else if (tag.funcName.includes("precompiled") && tag.funcName.split('_')[0] === "precompiled") {
        return eval_precompiled(ctx, tag);
    } else if (tag.funcName == "break") {
        return eval_breakPoint(ctx, tag);
    } else if (tag.funcName == "memAlignWR_W0") {
        return eval_memAlignWR_W0(ctx, tag);
    } else if (tag.funcName == "memAlignWR_W1") {
        return eval_memAlignWR_W1(ctx, tag);
    } else if (tag.funcName == "memAlignWR8_W0") {
        return eval_memAlignWR8_W0(ctx, tag);
    } else if (tag.funcName == "saveContractBytecode") { // Added by opcodes
        return eval_saveContractBytecode(ctx, tag);
    }  else {
        throw new Error(`function not defined ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    }
    throw new Error(`function not defined ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
}

function eval_getGlobalHash(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`)
    return scalar2fea(ctx.Fr, ctx.globalHash);
}

function eval_getSequencerAddr(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`)
    return scalar2fea(ctx.Fr, Scalar.e(ctx.input.sequencerAddr));
}

function eval_getBatchHashData(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`)
    return scalar2fea(ctx.Fr, Scalar.e(ctx.input.batchHashData));
}

function eval_getOldStateRoot(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return  scalar2fea(ctx.Fr, Scalar.e(ctx.input.oldStateRoot));
}

function eval_getNewStateRoot(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return  scalar2fea(ctx.Fr, Scalar.e(ctx.input.newStateRoot));
}

function eval_getTxs(ctx, tag) {
    if (tag.params.length != 2) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    const txs = ctx.input.batchL2Data;
    const offset = Number(evalCommand(ctx,tag.params[0]));
    const len = Number(evalCommand(ctx,tag.params[1]));
    let d = "0x" + txs.slice(2+offset*2, 2+offset*2 + len*2);
    if (d.length == 2) d = d+'0';
    return scalar2fea(ctx.Fr, Scalar.e(d));
}

function eval_getTxsLen(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return [ctx.Fr.e((ctx.input.batchL2Data.length-2) / 2), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_getOldLocalExitRoot(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return scalar2fea(ctx.Fr, Scalar.e(ctx.input.oldLocalExitRoot));
}

function eval_getGlobalExitRoot(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return scalar2fea(ctx.Fr, Scalar.e(ctx.input.globalExitRoot));
}

function eval_getNewLocalExitRoot(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return scalar2fea(ctx.Fr, Scalar.e(ctx.input.newLocalExitRoot));
}

function eval_getNumBatch(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return [ctx.Fr.e(ctx.input.numBatch), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_getTimestamp(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return [ctx.Fr.e(ctx.input.timestamp), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_getChainId(ctx, tag) {
    if (tag.params.length != 0) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return [ctx.Fr.e(ctx.input.chainID), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_eventLog(ctx, tag) {
    if (tag.params.length < 1) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    if(fullTracer) fullTracer.handleEvent(ctx, tag)
    return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_cond(ctx, tag) {
    if (tag.params.length != 1) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    const result = Number(evalCommand(ctx,tag.params[0]));
    if (result) {
        return [ctx.Fr.e(-1), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
    }
    return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_getBytecode(ctx, tag) {
    if (tag.params.length != 2 && tag.params.length != 3) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln}`)
    let hashcontract = evalCommand(ctx, tag.params[0]);
    hashcontract = "0x" + hashcontract.toString(16).padStart(64, '0');
    const bytecode = ctx.input.contractsBytecode[hashcontract] || ctx.input.contractsBytecode[hashcontract.slice(2)];
    const offset = Number(evalCommand(ctx, tag.params[1]));
    let len;
    if (tag.params[2])
        len = Number(evalCommand(ctx, tag.params[2]));
    else
        len = 1;
    if (bytecode === undefined) return scalar2fea(ctx.Fr, Scalar.e(0));
    // TODO: handle "0x"
    const offset0x = bytecode.startsWith('0x') ? 2 : 0;
    let d = "0x" + bytecode.slice(offset0x + offset * 2, offset0x + offset * 2 + len * 2);
    if (d.length == 2) d = d + '0';
    const ret = scalar2fea(ctx.Fr, Scalar.e(d));
    return scalar2fea(ctx.Fr, Scalar.e(d));
}
/**
 * Creates new storage checkpoint for warm slots and addresses
 */
function eval_checkpoint(ctx) {
    ctx.input.accessedStorage.push(new Map())
    return scalar2fea(ctx.Fr, Scalar.e(0));
}

/**
 * Consolidates checkpoint, merge last access storage with beforeLast access storage
 * @param {Object} ctx current rom context object
 */
function eval_commit(ctx) {
    const storageMap = ctx.input.accessedStorage.pop()
    if (storageMap) {
        const mapTarget = ctx.input.accessedStorage[ctx.input.accessedStorage.length - 1]
        if (mapTarget) {
            storageMap?.forEach((slotSet, addressString) => {
                const addressExists = mapTarget.get(addressString)
                if (!addressExists) {
                    mapTarget.set(addressString, new Set())
                }
                const storageSet = mapTarget.get(addressString)
                slotSet.forEach((value) => {
                    storageSet.add(value)
                })
            })
        }
    }
    return scalar2fea(ctx.Fr, Scalar.e(0));
}

/**
 * Revert accessedStorage to last checkpoint
 * @param {Object} ctx current rom context object
 */
function eval_revert(ctx) {
    ctx.input.accessedStorage.pop()
    return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

/**
 * Checks if the address is warm or cold. In case of cold, the address is added as warm
 * @param {Object} ctx current rom context object
 * @param {Object} tag tag inputs in rom function
 * @returns {FEA} returns 0 if address is warm, 1 if cold
 */
function eval_isWarmedAddress(ctx, tag) {
    if (tag.params.length != 1) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln}`)
    const address = evalCommand(ctx, tag.params[0]);
    const addr = address.toString(16)
    // if address is precompiled smart contract considered warm access
    if (Scalar.gt(address, 0) && Scalar.lt(address, 10)) {
        return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
    }

    // if address is warm return 0
    for (let i = ctx.input.accessedStorage.length - 1; i >= 0; i--) {
        const currentMap = ctx.input.accessedStorage[i]
        if (currentMap.has(addr)) {
            return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
    }
    //if address is not warm, return 1 and add it as warm. We add an emtpy set because is a warmed address (not warmed slot)
    const storageSet = ctx.input.accessedStorage[ctx.input.accessedStorage.length - 1].get(addr)
    if (!storageSet) {
        const emptyStorage = new Set()
        ctx.input.accessedStorage[ctx.input.accessedStorage.length - 1].set(addr, emptyStorage)
    }
    return [ctx.Fr.e(1), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

/**
 * Checks if the storage slot of the account is warm or cold. In case of cold, the slot is added as warm
 * @param {Object} ctx current rom context object
 * @param {Object} tag tag inputs in rom function
 * @returns {FEA} returns 0 if storage solt is warm, 1 if cold
 */
function eval_isWarmedStorage(ctx, tag) {
    if (tag.params.length != 2) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln}`)
    let addr = evalCommand(ctx, tag.params[0]).toString(16);
    let key = evalCommand(ctx, tag.params[1])
    // if address in acessStorage return 0
    for (let i = ctx.input.accessedStorage.length - 1; i >= 0; i--) {
        const currentMap = ctx.input.accessedStorage[i]
        if (currentMap.has(addr) && currentMap.get(addr).has(key)) {
            return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
        }
    }
    // if address in acessStorage return 1 and add it as warm
    let storageSet = ctx.input.accessedStorage[ctx.input.accessedStorage.length - 1].get(addr)
    if (!storageSet) {
        storageSet = new Set()
        ctx.input.accessedStorage[ctx.input.accessedStorage.length - 1].set(addr, storageSet)
    }
    storageSet.add(key)
    return [ctx.Fr.e(1), ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

/**
 * Clears wamred storage array, ready to process a new tx
 */
function eval_clearWarmedStorage(ctx) {
    ctx.input.accessedStorage = [new Map()]
    return scalar2fea(ctx.Fr, Scalar.e(0));
}

function eval_exp(ctx, tag) {
    if (tag.params.length != 2) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`)
    const a = evalCommand(ctx, tag.params[0]);
    const b = evalCommand(ctx, tag.params[1])
    return scalar2fea(ctx.Fr, Scalar.exp(a, b));;
}

function eval_bitwise(ctx, tag) {
    const func = tag.funcName.split('_')[1];
    const a = evalCommand(ctx, tag.params[0]);
    let b;

    switch (func) {
        case 'and':
            checkParams(ctx, tag, 2);
            b = evalCommand(ctx, tag.params[1]);
            return Scalar.band(a, b);
        case 'or':
            checkParams(ctx, tag, 2);
            b = evalCommand(ctx, tag.params[1]);
            return Scalar.bor(a, b);
        case 'xor':
            checkParams(ctx, tag, 2);
            b = evalCommand(ctx, tag.params[1]);
            return Scalar.bxor(a, b);
        case 'not':
            checkParams(ctx, tag, 1);
            return Scalar.bxor(a, Mask256);
        default:
            throw new Error(`Invalid bitwise operation ${func}. ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`)
    }
}

function eval_beforeLast(ctx) {
    if (ctx.step >= ctx.stepsN-2) {
        return [0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n];
    } else {
        return [ctx.Fr.negone, 0n, 0n, 0n, 0n, 0n, 0n, 0n];
    }
}

function eval_comp(ctx, tag){
    checkParams(ctx, tag, 2);

    const func = tag.funcName.split('_')[1];
    const a = evalCommand(ctx,tag.params[0]);
    const b = evalCommand(ctx,tag.params[1]);

    switch (func){
        case 'lt':
            return Scalar.lt(a, b) ? 1 : 0;
        case 'gt':
            return Scalar.gt(a, b) ? 1 : 0;
        case 'eq':
            return Scalar.eq(a, b) ? 1 : 0;
        default:
            throw new Error(`Invalid bitwise operation ${func}. ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`)
    }
}

function eval_loadScalar(ctx, tag){
    checkParams(ctx, tag, 1);
    return evalCommand(ctx,tag.params[0]);
}

function eval_storeLog(ctx, tag){
    checkParams(ctx, tag, 3);

    const indexLog = evalCommand(ctx, tag.params[0]);
    const isTopic = evalCommand(ctx, tag.params[1]);
    const data = evalCommand(ctx, tag.params[2]);

    if (typeof ctx.outLogs[indexLog] === "undefined"){
        ctx.outLogs[indexLog] = {
            data: [],
            topics: []
        }
    }

    if (isTopic) {
        ctx.outLogs[indexLog].topics.push(data.toString(16));
    } else {
        ctx.outLogs[indexLog].data.push(data.toString(16));
    }
    if(fullTracer) fullTracer.handleEvent(ctx, tag)
    return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_log(ctx, tag) {
    const frLog = ctx[tag.params[0].regName];
    const label = typeof tag.params[1] === "undefined" ? "notset" : tag.params[1].varName;
    if(typeof(frLog) == "number") {
        console.log(frLog)
    } else {
        let scalarLog;
        let hexLog;
        if (tag.params[0].regName !== "HASHPOS" && tag.params[0].regName !== "GAS"){
            scalarLog = fea2scalar(ctx.Fr, frLog);
            hexLog = `0x${scalarLog.toString(16)}`;
        } else {
            scalarLog = Scalar.e(frLog);
            hexLog = `0x${scalarLog.toString(16)}`;
        }

        console.log(`Log regname ${tag.params[0].regName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
        if (label !== "notset")
            console.log("       Label: ", label);
        console.log("       Scalar: ", scalarLog);
        console.log("       Hex:    ", hexLog);
        console.log("--------------------------");
    }
    return scalar2fea(ctx.Fr, Scalar.e(0));
}

function eval_breakPoint(ctx, tag) {
    console.log(`Breakpoint: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    return scalar2fea(ctx.Fr, Scalar.e(0));
}

// Helpers MemAlign

function eval_memAlignWR_W0(ctx, tag) {
    // parameters: M0, value, offset
    const m0 = evalCommand(ctx, tag.params[0]);
    const value = evalCommand(ctx, tag.params[1]);
    const offset = evalCommand(ctx, tag.params[2]);

    return scalar2fea(ctx.Fr, Scalar.bor(  Scalar.band(m0, Scalar.shl(Mask256, (32n - offset) * 8n)),
                        Scalar.band(Mask256, Scalar.shr(value, offset * 8n))));
}

function eval_memAlignWR_W1(ctx, tag) {
    // parameters: M1, value, offset
    const m1 = evalCommand(ctx, tag.params[0]);
    const value = evalCommand(ctx, tag.params[1]);
    const offset = evalCommand(ctx, tag.params[2]);

    return scalar2fea(ctx.Fr, Scalar.bor(  Scalar.band(m1, Scalar.shr(Mask256, offset * 8n)),
                        Scalar.band(Mask256, Scalar.shl(value, (32n - offset) * 8n))));
}

function eval_memAlignWR8_W0(ctx, tag) {
    // parameters: M0, value, offset
    const m0 = evalCommand(ctx, tag.params[0]);
    const value = evalCommand(ctx, tag.params[1]);
    const offset = evalCommand(ctx, tag.params[2]);
    const bits = (31n - offset) * 8n;

    return scalar2fea(ctx.Fr, Scalar.bor(  Scalar.band(m0, Scalar.sub(Mask256, Scalar.shl(0xFFn, bits))),
                        Scalar.shl(Scalar.band(0xFFn, value), bits)));
}

function eval_saveContractBytecode(ctx, tag) {
    const addr = evalCommand(ctx, tag.params[0]);
    ctx.input.contractsBytecode[ctx.hashP[addr].digest] = "0x"+byteArray2HexString(ctx.hashP[addr].data);
    return scalar2fea(ctx.Fr, Scalar.e(0));
}

function checkParams(ctx, tag, expectedParams){
    if (tag.params.length != expectedParams) throw new Error(`Invalid number of parameters function ${tag.funcName}: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
}

function eval_dumpRegs(ctx, tag) {

    console.log(`dumpRegs ${ctx.fileName}:${ctx.line}`);

    console.log(['A', fea2scalar(ctx.Fr, ctx.A)]);
    console.log(['B', fea2scalar(ctx.Fr, ctx.B)]);
    console.log(['C', fea2scalar(ctx.Fr, ctx.C)]);
    console.log(['D', fea2scalar(ctx.Fr, ctx.D)]);
    console.log(['E', fea2scalar(ctx.Fr, ctx.E)]);

    return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_dump(ctx, tag) {
    console.log("\x1b[38;2;175;175;255mDUMP on " + ctx.fileName + ":" + ctx.line+"\x1b[0m");

    tag.params.forEach((value) => {
        let name = value.varName || value.paramName || value.regName || value.offsetLabel;
        if (typeof name == 'undefined' && value.path) {
            name = value.path.join('.');
        }
        console.log("\x1b[35m"+ name +"\x1b[0;35m: "+evalCommand(ctx, value)+"\x1b[0m");
    });

    return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}

function eval_dumphex(ctx, tag) {
    console.log("\x1b[38;2;175;175;255mDUMP on " + ctx.fileName + ":" + ctx.line+"\x1b[0m");

    tag.params.forEach((value) => {
        let name = value.varName || value.paramName || value.regName;
        if (typeof name == 'undefined' && value.path) {
            name = value.path.join('.');
        }
        console.log("\x1b[35m"+ name +"\x1b[0;35m: 0x"+evalCommand(ctx, value).toString(16)+"\x1b[0m");
    });

    return [ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero, ctx.Fr.zero];
}
function eval_inverseFpEc(ctx, tag) {
    const a = evalCommand(ctx, tag.params[0]);
    if (ctx.Fec.isZero(a)) {
        throw new Error(`inverseFpEc: Division by zero  on: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    }
    return ctx.Fec.inv(a);
}

function eval_inverseFnEc(ctx, tag) {
    const a = evalCommand(ctx, tag.params[0]);
    if (ctx.Fnec.isZero(a)) {
        throw new Error(`inverseFpEc: Division by zero  on: ${ctx.ln} at ${ctx.fileName}:${ctx.line}`);
    }
    return ctx.Fnec.inv(a);
}

function eval_sqrtFpEc(ctx, tag) {
    const a = evalCommand(ctx, tag.params[0]);
    const r = ctx.Fec.sqrt(a);
    if (r === null) {
        return 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFn;
    }
    return r;
}

function eval_xAddPointEc(ctx, tag) {
    return eval_AddPointEc(ctx, tag, false)[0];
}

function eval_yAddPointEc(ctx, tag) {
    return eval_AddPointEc(ctx, tag, false)[1];
}

function eval_xDblPointEc(ctx, tag) {
    return eval_AddPointEc(ctx, tag, true)[0];
}

function eval_yDblPointEc(ctx, tag) {
    return eval_AddPointEc(ctx, tag, true)[1];
}

function eval_AddPointEc(ctx, tag, dbl)
{
    const x1 = evalCommand(ctx, tag.params[0]);
    const y1 = evalCommand(ctx, tag.params[1]);
    const x2 = evalCommand(ctx, tag.params[dbl ? 0 : 2]);
    const y2 = evalCommand(ctx, tag.params[dbl ? 1 : 3]);

    if (dbl) {
        // TODO: y1 == 0 => division by zero ==> how manage?
        s = ctx.Fec.div(ctx.Fec.mul(3n, ctx.Fec.mul(x1, x1)), ctx.Fec.add(y1, y1));
    }
    else {
        let deltaX = ctx.Fec.sub(x2, x1)
        // TODO: deltaX == 0 => division by zero ==> how manage?
        s = ctx.Fec.div(ctx.Fec.sub(y2, y1), deltaX );
    }

    const x3 = ctx.Fec.sub(ctx.Fec.mul(s, s), ctx.Fec.add(x1, x2));
    const y3 = ctx.Fec.sub(ctx.Fec.mul(s, ctx.Fec.sub(x1,x3)), y1);

    return [x3, y3];
}

function preprocessTxs(ctx) {

    const {
        numBatch,
        sequencerAddr,
        oldLocalExitRoot,
        newLocalExitRoot,
        oldStateRoot,
        newStateRoot,
        globalExitRoot,
        timestamp,
        chainID
    } = ctx.input;

    ctx.input.batchHashData = calculateBatchHashData(
        ctx.input.batchL2Data,
        globalExitRoot,
        sequencerAddr
    );

    ctx.globalHash = calculateStarkInput(
            oldStateRoot,
            oldLocalExitRoot,
            newStateRoot,
            newLocalExitRoot,
            ctx.input.batchHashData,
            numBatch,
            timestamp,
            chainID
    );

    ctx.input.accessedStorage = [new Map()]
}

function printRegs(Fr, ctx) {
    printReg8(Fr, "A", ctx.A);
    printReg8(Fr, "B", ctx.B);
    printReg8(Fr, "C", ctx.C);
    printReg8(Fr, "D", ctx.D);
    printReg8(Fr, "E", ctx.E);
    printReg4(Fr,  "SR", ctx.SR);
    printReg1("CTX", ctx.CTX);
    printReg1("SP", ctx.SP);
    printReg1("PC", ctx.PC);
    printReg1("MAXMEM", ctx.MAXMEM);
    printReg1("GAS", ctx.GAS);
    printReg1("zkPC", ctx.zkPC);
    printReg1("RR", ctx.RR);
    printReg1("STEP", ctx.step, false, true);
    console.log(ctx.fileName + ":" + ctx.line);
}

function printReg4(Fr, name, V) {
    printReg(Fr, name+"7", V[7], true);
    printReg(Fr, name+"6", V[6], true);
    printReg(Fr, name+"5", V[5], true);
    printReg(Fr, name+"4", V[4], true);
    printReg(Fr, name+"3", V[3], true);
    printReg(Fr, name+"2", V[2], true);
    printReg(Fr, name+"1", V[1], true);
    printReg(Fr, name+"0", V[0]);
    console.log("");
}


function printReg4(Fr, name, V) {

    printReg(Fr, name+"3", V[3], true);
    printReg(Fr, name+"2", V[2], true);
    printReg(Fr, name+"1", V[1], true);
    printReg(Fr, name+"0", V[0]);
    console.log("");
}

function printReg(Fr, name, V, h, short) {
    const maxInt = Scalar.e("0x7FFFFFFF");
    const minInt = Scalar.sub(Fr.p, Scalar.e("0x80000000"));

    let S;
    S = name.padEnd(6) +": ";

    let S2;
    if (!h) {
        const o = Fr.toObject(V);
        if (Scalar.gt(o, maxInt)) {
            const on = Scalar.sub(Fr.p, o);
            if (Scalar.gt(o, minInt)) {
                S2 = "-" + Scalar.toString(on);
            } else {
                S2 = "LONG";
            }
        } else {
            S2 = Scalar.toString(o);
        }
    } else {
        S2 = "";
    }

    S += S2.padStart(8, " ");

    if (!short) {
        const o = Fr.toObject(V);
        S+= "   " + o.toString(16).padStart(32, "0");
    }

    console.log(S);


}


function printReg1(name, V, h, short) {
    let S;
    S = name.padEnd(6) +": ";

    let S2 = V.toString();

    S += S2.padStart(16, " ");

    console.log(S);
}

function sr8to4(F, SR) {
    const r=[];
    r[0] = F.add(SR[0], F.mul(SR[1], F.e("0x100000000")));
    r[1] = F.add(SR[2], F.mul(SR[3], F.e("0x100000000")));
    r[2] = F.add(SR[4], F.mul(SR[5], F.e("0x100000000")));
    r[3] = F.add(SR[6], F.mul(SR[7], F.e("0x100000000")));
    return r;
}

function sr4to8(F, r) {
    const sr=[];
    sr[0] = r[0] & 0xFFFFFFFFn;
    sr[1] = r[0] >> 32n;
    sr[2] = r[1] & 0xFFFFFFFFn;
    sr[3] = r[1] >> 32n;
    sr[4] = r[2] & 0xFFFFFFFFn;
    sr[5] = r[2] >> 32n;
    sr[6] = r[3] & 0xFFFFFFFFn;
    sr[7] = r[3] >> 32n;
    return sr;
}

async function poseidonLinear(inp) {
    const poseidon = await buildPoseidon()
    const F = poseidon.F;

    bytes = inp.slice();
    bytes.push(0x01);
    while((bytes.length % 56) !== 0) bytes.push(0);

    bytes[bytes.length-1] |= 0x80;

    let st = [F.zero, F.zero, F.zero, F.zero];
    for (let j=0; j<bytes.length;j+=56) {
        let A = [F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero];
        for (k=0; k<56; k++) {
            const e = Math.floor(k / 7);
            const pe = k % 7;
            A[e] = F.add(A[e], F.e(BigInt(bytes[j+k]) << BigInt(pe * 8)));
        }
        st = poseidon(A, st);
    }

    return h4toString(st);
}