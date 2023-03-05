

const {Scalar, F1Field}  = require("ffjavascript");

const { scalar2fea } = require("@0xpolygonhermez/zkevm-commonjs").smtUtils;

module.exports.buildConstants = async function buildConstants(pols, rom) {

    const F = new F1Field("0xFFFFFFFF00000001");

    const N = pols.offset.length;

    const twoTo31 = Scalar.e(0x80000000);
    const maxInt = 2147483647;
    const minInt = -2147483648;
    const maxUInt = 0xFFFFFFFF;
    const minUInt = 0;

    if (rom.program.length>N) throw new Error("Rom is too big for this N");

    for (let i=0; i<N; i++) {
        const pIndex = i < rom.program.length ? i:(rom.program.length-1);
        if (rom.program[pIndex].CONST) {
            if (rom.program[pIndex].CONSTL) throw new Error("Program mixed with long and short constants");
            pols.CONST0[i] = rom.program[pIndex].CONST ? F.e(rom.program[pIndex].CONST) : F.zero;
            pols.CONST1[i] = F.zero;
            pols.CONST2[i] = F.zero;
            pols.CONST3[i] = F.zero;
            pols.CONST4[i] = F.zero;
            pols.CONST5[i] = F.zero;
            pols.CONST6[i] = F.zero;
            pols.CONST7[i] = F.zero;
        } else if (rom.program[pIndex].CONSTL) {
            [
                pols.CONST0[i],
                pols.CONST1[i],
                pols.CONST2[i],
                pols.CONST3[i],
                pols.CONST4[i],
                pols.CONST5[i],
                pols.CONST6[i],
                pols.CONST7[i],
            ] = scalar2fea(F, BigInt(rom.program[pIndex].CONSTL));
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
        pols.offset[i] = rom.program[pIndex].offset ? BigInt(rom.program[pIndex].offset) : 0n;

        pols.inA[i] = rom.program[pIndex].inA ? F.e(rom.program[pIndex].inA) : F.zero;
        pols.inB[i] = rom.program[pIndex].inB ? F.e(rom.program[pIndex].inB) : F.zero;
        pols.inC[i] = rom.program[pIndex].inC ? F.e(rom.program[pIndex].inC) : F.zero;
        pols.inD[i] = rom.program[pIndex].inD ? F.e(rom.program[pIndex].inD) : F.zero;
        pols.inE[i] = rom.program[pIndex].inE ? F.e(rom.program[pIndex].inE) : F.zero;
        pols.inSR[i] = rom.program[pIndex].inSR ? F.e(rom.program[pIndex].inSR) : F.zero;
        pols.inCTX[i] = rom.program[pIndex].inCTX ? F.e(rom.program[pIndex].inCTX) : F.zero;
        pols.inSP[i] = rom.program[pIndex].inSP ? F.e(rom.program[pIndex].inSP) : F.zero;
        pols.inPC[i] = rom.program[pIndex].inPC ? F.e(rom.program[pIndex].inPC) : F.zero;
        pols.inMAXMEM[i] = rom.program[pIndex].inMAXMEM ? F.e(rom.program[pIndex].inMAXMEM) : F.zero;
        pols.inSTEP[i] = rom.program[pIndex].inSTEP ? F.e(rom.program[pIndex].inSTEP) : F.zero;
        pols.inFREE[i] = rom.program[pIndex].inFREE ? F.e(rom.program[pIndex].inFREE) : F.zero;
        pols.inGAS[i] = rom.program[pIndex].inGAS ? F.e(rom.program[pIndex].inGAS) : F.zero;
        pols.inRR[i] = rom.program[pIndex].inRR ? F.e(rom.program[pIndex].inRR) : F.zero;
        pols.inHASHPOS[i] = rom.program[pIndex].inHASHPOS ? F.e(rom.program[pIndex].inHASHPOS) : F.zero;
        pols.inROTL_C[i] = rom.program[pIndex].inROTL_C ? F.e(rom.program[pIndex].inROTL_C) : F.zero;
        pols.inRCX[i] = rom.program[pIndex].inRCX ? F.e(rom.program[pIndex].inRCX) : F.zero;

        pols.inCntArith[i] = rom.program[pIndex].inCntArith ? F.e(rom.program[pIndex].inCntArith) : F.zero;
        pols.inCntBinary[i] = rom.program[pIndex].inCntBinary ? F.e(rom.program[pIndex].inCntBinary) : F.zero;
        pols.inCntKeccakF[i] = rom.program[pIndex].inCntKeccakF ? F.e(rom.program[pIndex].inCntKeccakF) : F.zero;
        pols.inCntMemAlign[i] = rom.program[pIndex].inCntMemAlign ? F.e(rom.program[pIndex].inCntMemAlign) : F.zero;
        pols.inCntPaddingPG[i] = rom.program[pIndex].inCntPaddingPG ? F.e(rom.program[pIndex].inCntPaddingPG) : F.zero;
        pols.inCntPoseidonG[i] = rom.program[pIndex].inCntPoseidonG ? F.e(rom.program[pIndex].inCntPoseidonG) : F.zero;

        /*
            code generated with:
            node tools/pil_pol_table/bits_compose.js "arithEq0,arithEq1,arithEq2,assert,bin,hashK,hashKDigest,hashKLen,hashP,hashPDigest,hashPLen,ind,indRR,isMem,isStack,JMP,JMPC,JMPN,memAlignRD,memAlignWR,memAlignWR8,mOp,mWR,repeat,setA,setB,setC,setCTX,setD,setE,setGAS,setHASHPOS,setMAXMEM,setPC,setRCX,setRR,setSP,setSR,sRD,sWR,useCTX,useJmpAddr,JMPZ,call,return,useElseAddr" -B -e -p "rom.program[pIndex]."
        */

        pols.operations[i] =
          (rom.program[pIndex].arithEq0 ? (2n**0n  * BigInt(rom.program[pIndex].arithEq0)) : 0n)
        + (rom.program[pIndex].arithEq1 ? (2n**1n  * BigInt(rom.program[pIndex].arithEq1)) : 0n)
        + (rom.program[pIndex].arithEq2 ? (2n**2n  * BigInt(rom.program[pIndex].arithEq2)) : 0n)
        + (rom.program[pIndex].assert ? (2n**3n  * BigInt(rom.program[pIndex].assert)) : 0n)
        + (rom.program[pIndex].bin ? (2n**4n  * BigInt(rom.program[pIndex].bin)) : 0n)
        + (rom.program[pIndex].hashK ? (2n**5n  * BigInt(rom.program[pIndex].hashK)) : 0n)
        + (rom.program[pIndex].hashKDigest ? (2n**6n  * BigInt(rom.program[pIndex].hashKDigest)) : 0n)
        + (rom.program[pIndex].hashKLen ? (2n**7n  * BigInt(rom.program[pIndex].hashKLen)) : 0n)
        + (rom.program[pIndex].hashP ? (2n**8n  * BigInt(rom.program[pIndex].hashP)) : 0n)
        + (rom.program[pIndex].hashPDigest ? (2n**9n  * BigInt(rom.program[pIndex].hashPDigest)) : 0n)
        + (rom.program[pIndex].hashPLen ? (2n**10n * BigInt(rom.program[pIndex].hashPLen)) : 0n)
        + (rom.program[pIndex].ind ? (2n**11n * BigInt(rom.program[pIndex].ind)) : 0n)
        + (rom.program[pIndex].indRR ? (2n**12n * BigInt(rom.program[pIndex].indRR)) : 0n)
        + (rom.program[pIndex].isMem ? (2n**13n * BigInt(rom.program[pIndex].isMem)) : 0n)
        + (rom.program[pIndex].isStack ? (2n**14n * BigInt(rom.program[pIndex].isStack)) : 0n)
        + (rom.program[pIndex].JMP ? (2n**15n * BigInt(rom.program[pIndex].JMP)) : 0n)
        + (rom.program[pIndex].JMPC ? (2n**16n * BigInt(rom.program[pIndex].JMPC)) : 0n)
        + (rom.program[pIndex].JMPN ? (2n**17n * BigInt(rom.program[pIndex].JMPN)) : 0n)
        + (rom.program[pIndex].memAlignRD ? (2n**18n * BigInt(rom.program[pIndex].memAlignRD)) : 0n)
        + (rom.program[pIndex].memAlignWR ? (2n**19n * BigInt(rom.program[pIndex].memAlignWR)) : 0n)
        + (rom.program[pIndex].memAlignWR8 ? (2n**20n * BigInt(rom.program[pIndex].memAlignWR8)) : 0n)
        + (rom.program[pIndex].mOp ? (2n**21n * BigInt(rom.program[pIndex].mOp)) : 0n)
        + (rom.program[pIndex].mWR ? (2n**22n * BigInt(rom.program[pIndex].mWR)) : 0n)
        + (rom.program[pIndex].repeat ? (2n**23n * BigInt(rom.program[pIndex].repeat)) : 0n)
        + (rom.program[pIndex].setA ? (2n**24n * BigInt(rom.program[pIndex].setA)) : 0n)
        + (rom.program[pIndex].setB ? (2n**25n * BigInt(rom.program[pIndex].setB)) : 0n)
        + (rom.program[pIndex].setC ? (2n**26n * BigInt(rom.program[pIndex].setC)) : 0n)
        + (rom.program[pIndex].setCTX ? (2n**27n * BigInt(rom.program[pIndex].setCTX)) : 0n)
        + (rom.program[pIndex].setD ? (2n**28n * BigInt(rom.program[pIndex].setD)) : 0n)
        + (rom.program[pIndex].setE ? (2n**29n * BigInt(rom.program[pIndex].setE)) : 0n)
        + (rom.program[pIndex].setGAS ? (2n**30n * BigInt(rom.program[pIndex].setGAS)) : 0n)
        + (rom.program[pIndex].setHASHPOS ? (2n**31n * BigInt(rom.program[pIndex].setHASHPOS)) : 0n)
        + (rom.program[pIndex].setMAXMEM ? (2n**32n * BigInt(rom.program[pIndex].setMAXMEM)) : 0n)
        + (rom.program[pIndex].setPC ? (2n**33n * BigInt(rom.program[pIndex].setPC)) : 0n)
        + (rom.program[pIndex].setRCX ? (2n**34n * BigInt(rom.program[pIndex].setRCX)) : 0n)
        + (rom.program[pIndex].setRR ? (2n**35n * BigInt(rom.program[pIndex].setRR)) : 0n)
        + (rom.program[pIndex].setSP ? (2n**36n * BigInt(rom.program[pIndex].setSP)) : 0n)
        + (rom.program[pIndex].setSR ? (2n**37n * BigInt(rom.program[pIndex].setSR)) : 0n)
        + (rom.program[pIndex].sRD ? (2n**38n * BigInt(rom.program[pIndex].sRD)) : 0n)
        + (rom.program[pIndex].sWR ? (2n**39n * BigInt(rom.program[pIndex].sWR)) : 0n)
        + (rom.program[pIndex].useCTX ? (2n**40n * BigInt(rom.program[pIndex].useCTX)) : 0n)
        + (rom.program[pIndex].useJmpAddr ? (2n**41n * BigInt(rom.program[pIndex].useJmpAddr)) : 0n)
        + (rom.program[pIndex].JMPZ ? (2n**42n * BigInt(rom.program[pIndex].JMPZ)) : 0n)
        + (rom.program[pIndex].call ? (2n**43n * BigInt(rom.program[pIndex].call)) : 0n)
        + (rom.program[pIndex].return ? (2n**44n * BigInt(rom.program[pIndex].return)) : 0n)
        + (rom.program[pIndex].hashK1 ? (2n**45n * BigInt(rom.program[pIndex].hashK1)) : 0n)
        + (rom.program[pIndex].hashP1 ? (2n**46n * BigInt(rom.program[pIndex].hashP1)) : 0n)
        + (rom.program[pIndex].useElseAddr ? (2n**47n * BigInt(rom.program[pIndex].useElseAddr)) : 0n);

        pols.incStack[i] = rom.program[pIndex].incStack ? BigInt(rom.program[pIndex].incStack) : 0n;

        pols.binOpcode[i] = rom.program[pIndex].binOpcode ? BigInt(rom.program[pIndex].binOpcode) : 0n;
        pols.jmpAddr[i] = rom.program[pIndex].jmpAddr ? BigInt(rom.program[pIndex].jmpAddr) : 0n;
        pols.elseAddr[i] = rom.program[pIndex].elseAddr ? BigInt(rom.program[pIndex].elseAddr) : 0n;
        pols.line[i] = BigInt(pIndex);
    }
}