

const {Scalar, F1Field}  = require("ffjavascript");

const { scalar2fea } = require("@0xpolygonhermez/zkevm-commonjs").smtUtils;

module.exports.buildConstants = async function buildConstants(pols, rom) {

    const F = new F1Field("0xFFFFFFFF00000001");

    const N = pols.inA.length;

    const twoTo31 = Scalar.e(0x80000000);
    const maxInt = 2147483647;
    const minInt = -2147483648;
    const maxUInt = 0xFFFFFFFF;
    const minUInt = 0;

    if (rom.program.length>N) throw new Error("Rom is too big for this N");

    for (let i=0; i<rom.program.length; i++) {

        if (rom.program[i].CONST) {
            if (rom.program[i].CONSTL) throw new Error("Program mixed with long and short constants");
            pols.CONST0[i] = rom.program[i].CONST ? F.e(rom.program[i].CONST) : F.zero;
            pols.CONST1[i] = F.zero;
            pols.CONST2[i] = F.zero;
            pols.CONST3[i] = F.zero;
            pols.CONST4[i] = F.zero;
            pols.CONST5[i] = F.zero;
            pols.CONST6[i] = F.zero;
            pols.CONST7[i] = F.zero;
        } else if (rom.program[i].CONSTL) {
            [
                pols.CONST0[i],
                pols.CONST1[i],
                pols.CONST2[i],
                pols.CONST3[i],
                pols.CONST4[i],
                pols.CONST5[i],
                pols.CONST6[i],
                pols.CONST7[i],
            ] = scalar2fea(F, BigInt(rom.program[i].CONSTL));
        } else {
            pols.CONST0[i] = F.zero;
            pols.CONST1[i] = F.zero;
            pols.CONST2[i] = F.zero;
            pols.CONST3[i] = F.zero;
            pols.CONST4[i] = F.zero;
            pols.CONST5[i] = F.zero;
            pols.CONST6[i] = F.zero;
            pols.CONST7[i] = F.zero;
        }
        pols.offset[i] = rom.program[i].offset ? BigInt(rom.program[i].offset) : 0n;

        pols.inA[i] = rom.program[i].inA ? F.e(rom.program[i].inA) : F.zero;
        pols.inB[i] = rom.program[i].inB ? F.e(rom.program[i].inB) : F.zero;
        pols.inC[i] = rom.program[i].inC ? F.e(rom.program[i].inC) : F.zero;
        pols.inD[i] = rom.program[i].inD ? F.e(rom.program[i].inD) : F.zero;
        pols.inE[i] = rom.program[i].inE ? F.e(rom.program[i].inE) : F.zero;
        pols.inSR[i] = rom.program[i].inSR ? F.e(rom.program[i].inSR) : F.zero;
        pols.inCTX[i] = rom.program[i].inCTX ? F.e(rom.program[i].inCTX) : F.zero;
        pols.inSP[i] = rom.program[i].inSP ? F.e(rom.program[i].inSP) : F.zero;
        pols.inPC[i] = rom.program[i].inPC ? F.e(rom.program[i].inPC) : F.zero;
        pols.inMAXMEM[i] = rom.program[i].inMAXMEM ? F.e(rom.program[i].inMAXMEM) : F.zero;
        pols.inSTEP[i] = rom.program[i].inSTEP ? F.e(rom.program[i].inSTEP) : F.zero;
        pols.inFREE[i] = rom.program[i].inFREE ? F.e(rom.program[i].inFREE) : F.zero;
        pols.inGAS[i] = rom.program[i].inGAS ? F.e(rom.program[i].inGAS) : F.zero;
        pols.inRR[i] = rom.program[i].inRR ? F.e(rom.program[i].inRR) : F.zero;
        pols.inHASHPOS[i] = rom.program[i].inHASHPOS ? F.e(rom.program[i].inHASHPOS) : F.zero;
        pols.inROTL_C[i] = rom.program[i].inROTL_C ? F.e(rom.program[i].inROTL_C) : F.zero;

        pols.setA[i] = rom.program[i].setA ? 1n : 0n;
        pols.setB[i] = rom.program[i].setB ? 1n : 0n;
        pols.setC[i] = rom.program[i].setC ? 1n : 0n;
        pols.setD[i] = rom.program[i].setD ? 1n : 0n;
        pols.setE[i] = rom.program[i].setE ? 1n : 0n;
        pols.setSR[i] = rom.program[i].setSR ? 1n : 0n;
        pols.setCTX[i] = rom.program[i].setCTX ? 1n : 0n;
        pols.setSP[i] = rom.program[i].setSP ? 1n : 0n;
        pols.setPC[i] = rom.program[i].setPC ? 1n : 0n;
        pols.setGAS[i] = rom.program[i].setGAS ? 1n : 0n;
        pols.setMAXMEM[i] = rom.program[i].setMAXMEM ? 1n : 0n;
        pols.setRR[i] = rom.program[i].setRR ? 1n : 0n;
        pols.setHASHPOS[i] = rom.program[i].setHASHPOS ? 1n : 0n;

        pols.JMP[i] = rom.program[i].JMP ? 1n : 0n;
        pols.JMPC[i] = rom.program[i].JMPC ? 1n : 0n;
        pols.JMPN[i] = rom.program[i].JMPN ? 1n : 0n;

        pols.incStack[i] = rom.program[i].incStack ? BigInt(rom.program[i].incStack) : 0n;
        pols.incCode[i] = rom.program[i].incCode ? BigInt(rom.program[i].incCode) : 0n;

        pols.isStack[i] = rom.program[i].isStack ? 1n : 0n;
        pols.isCode[i] = rom.program[i].isCode ? 1n : 0n;
        pols.isMem[i] = rom.program[i].isMem ? 1n : 0n;
        pols.ind[i] = rom.program[i].ind ? 1n : 0n;
        pols.indRR[i] = rom.program[i].indRR ? 1n : 0n;
        pols.useCTX[i] = rom.program[i].useCTX ? 1n : 0n;

        pols.mOp[i] = rom.program[i].mOp ? 1n : 0n;
        pols.mWR[i] = rom.program[i].mWR ? 1n : 0n;
        pols.sRD[i] = rom.program[i].sRD ? 1n : 0n;
        pols.sWR[i] = rom.program[i].sWR ? 1n : 0n;
        pols.arith[i] = rom.program[i].arith ? 1n : 0n;
        pols.arithEq0[i] = rom.program[i].arithEq0 ? 1n : 0n;
        pols.arithEq1[i] = rom.program[i].arithEq1 ? 1n : 0n;
        pols.arithEq2[i] = rom.program[i].arithEq2 ? 1n : 0n;
        pols.arithEq3[i] = rom.program[i].arithEq3 ? 1n : 0n;
        pols.memAlign[i] = rom.program[i].memAlign ? 1n : 0n;
        pols.memAlignWR[i] = rom.program[i].memAlignWR ? 1n : 0n;
        pols.memAlignWR8[i] = rom.program[i].memAlignWR8 ? 1n : 0n;
        pols.hashK[i] = rom.program[i].hashK ? 1n : 0n;
        pols.hashKLen[i] = rom.program[i].hashKLen ? 1n : 0n;
        pols.hashKDigest[i] = rom.program[i].hashKDigest ? 1n : 0n;
        pols.hashP[i] = rom.program[i].hashP ? 1n : 0n;
        pols.hashPLen[i] = rom.program[i].hashPLen ? 1n : 0n;
        pols.hashPDigest[i] = rom.program[i].hashPDigest ? 1n : 0n;
        pols.bin[i] = rom.program[i].bin ? 1n : 0n;
        pols.binOpcode[i] = rom.program[i].binOpcode ? BigInt(rom.program[i].binOpcode) : 0n;
        pols.assert[i] = rom.program[i].assert ? 1n : 0n;


        pols.line[i] = BigInt(i);

    }

    for (let i= rom.program.length; i<N; i++) {
        pols.CONST0[i] = F.zero;
        pols.CONST1[i] = F.zero;
        pols.CONST2[i] = F.zero;
        pols.CONST3[i] = F.zero;
        pols.CONST4[i] = F.zero;
        pols.CONST5[i] = F.zero;
        pols.CONST6[i] = F.zero;
        pols.CONST7[i] = F.zero;
        pols.offset[i] = F.zero;

        pols.inA[i] = F.zero;
        pols.inB[i] = F.zero;
        pols.inC[i] = F.zero;
        pols.inD[i] = F.zero;
        pols.inE[i] = F.zero;
        pols.inSR[i] = F.zero;
        pols.inCTX[i] = F.zero;
        pols.inSP[i] = F.zero;
        pols.inPC[i] = F.zero;
        pols.inMAXMEM[i] = F.zero;
        pols.inSTEP[i] = F.zero;
        pols.inFREE[i] = F.zero;
        pols.inGAS[i] = F.zero;
        pols.inRR[i] = F.zero;
        pols.inHASHPOS[i] = F.zero;
        pols.inROTL_C[i] = F.zero;

        pols.setA[i] = F.zero;
        pols.setB[i] = F.zero;
        pols.setC[i] = F.zero;
        pols.setD[i] = F.zero;
        pols.setE[i] = F.zero;
        pols.setSR[i] = F.zero;
        pols.setCTX[i] = F.zero;
        pols.setSP[i] = F.zero;
        pols.setPC[i] = F.zero;
        pols.setGAS[i] = F.zero;
        pols.setMAXMEM[i] = F.zero;
        pols.setRR[i] = F.zero;
        pols.setHASHPOS[i] = F.zero;

        pols.JMP[i] = F.zero;
        pols.JMPC[i] = F.zero;
        pols.JMPN[i] = F.zero;

        pols.incStack[i] = F.zero;
        pols.incCode[i] = F.zero;

        pols.isStack[i] = F.zero;
        pols.isCode[i] = F.zero;
        pols.isMem[i] = F.zero;
        pols.ind[i] = F.zero;
        pols.indRR[i] = F.zero;
        pols.useCTX[i] = F.zero;

        pols.mOp[i] = F.zero;
        pols.mWR[i] = F.zero;
        pols.sRD[i] = F.zero;
        pols.sWR[i] = F.zero;
        pols.arith[i] = F.zero;
        pols.arithEq0[i] = F.zero;
        pols.arithEq1[i] = F.zero;
        pols.arithEq2[i] = F.zero;
        pols.arithEq3[i] = F.zero;
        pols.memAlign[i] = F.zero;
        pols.memAlignWR[i] = F.zero;
        pols.memAlignWR8[i] = F.zero;
        pols.hashK[i] = F.zero;
        pols.hashKLen[i] = F.zero;
        pols.hashKDigest[i] = F.zero;
        pols.hashP[i] = F.zero;
        pols.hashPLen[i] = F.zero;
        pols.hashPDigest[i] = F.zero;
        pols.bin[i] = F.zero;
        pols.binOpcode[i] = F.zero;
        pols.assert[i] = F.zero;

        pols.line[i] = BigInt(i);
    }

}