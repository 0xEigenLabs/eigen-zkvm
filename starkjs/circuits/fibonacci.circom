pragma circom 2.0.6;

include "gl.circom";
include "poseidon.circom";
include "bitify.circom";
include "sha256/sha256.circom";
include "fft.circom";
include "merklehash.circom";
include "evalpol.circom";
include "treeselector.circom";
include "bn1togl3.circom";
include "compconstant64.circom";





template VerifyEvaluations() {
    signal input challenges[8][3];
    signal input evals[6][3];
    signal input publics[1];

    var p = 0xFFFFFFFF00000001;

    component zMul[4];
    for (var i=0; i< 4 ; i++) {
        zMul[i] = GLCMul();
        if (i==0) {
            zMul[i].ina[0] <== challenges[7][0];
            zMul[i].ina[1] <== challenges[7][1];
            zMul[i].ina[2] <== challenges[7][2];
            zMul[i].inb[0] <== challenges[7][0];
            zMul[i].inb[1] <== challenges[7][1];
            zMul[i].inb[2] <== challenges[7][2];
        } else {
            zMul[i].ina[0] <== zMul[i-1].out[0];
            zMul[i].ina[1] <== zMul[i-1].out[1];
            zMul[i].ina[2] <== zMul[i-1].out[2];
            zMul[i].inb[0] <== zMul[i-1].out[0];
            zMul[i].inb[1] <== zMul[i-1].out[1];
            zMul[i].inb[2] <== zMul[i-1].out[2];
        }
    }

    signal Z[3];

    Z[0] <== zMul[3].out[0] -1 + p;
    Z[1] <== zMul[3].out[1];
    Z[2] <== zMul[3].out[2];

    signal tmp_0[3];

    tmp_0[0] <== 1 - evals[0][0] + p;
    tmp_0[1] <== -evals[0][1] + p;
    tmp_0[2] <== -evals[0][2] + p;
//    log(0);
    signal tmp_1[3];

    tmp_1[0] <== evals[1][0] - evals[2][0] + p;
    tmp_1[1] <== evals[1][1] - evals[2][1] + p;
    tmp_1[2] <== evals[1][2] - evals[2][2] + p;
//    log(1);
    signal tmp_2[3];

    component cmul_0 = GLCMul();
    cmul_0.ina[0] <== tmp_0[0];
    cmul_0.ina[1] <== tmp_0[1];
    cmul_0.ina[2] <== tmp_0[2];
    cmul_0.inb[0] <== tmp_1[0];
    cmul_0.inb[1] <== tmp_1[1];
    cmul_0.inb[2] <== tmp_1[2];
    tmp_2[0] <== cmul_0.out[0];
    tmp_2[1] <== cmul_0.out[1];
    tmp_2[2] <== cmul_0.out[2];
//    log(2);
    signal tmp_14[3];

    tmp_14[0] <== tmp_2[0] - 0 + p;
    tmp_14[1] <== tmp_2[1];
    tmp_14[2] <== tmp_2[2];
//    log(3);
    signal tmp_3[3];

    tmp_3[0] <== 1 - evals[0][0] + p;
    tmp_3[1] <== -evals[0][1] + p;
    tmp_3[2] <== -evals[0][2] + p;
//    log(4);
    signal tmp_4[3];

    tmp_4[0] <== evals[3][0] + evals[2][0];
    tmp_4[1] <== evals[3][1] + evals[2][1];
    tmp_4[2] <== evals[3][2] + evals[2][2];
//    log(5);
    signal tmp_5[3];

    tmp_5[0] <== evals[4][0] - tmp_4[0] + p;
    tmp_5[1] <== evals[4][1] - tmp_4[1] + p;
    tmp_5[2] <== evals[4][2] - tmp_4[2] + p;
//    log(6);
    signal tmp_6[3];

    component cmul_1 = GLCMul();
    cmul_1.ina[0] <== tmp_3[0];
    cmul_1.ina[1] <== tmp_3[1];
    cmul_1.ina[2] <== tmp_3[2];
    cmul_1.inb[0] <== tmp_5[0];
    cmul_1.inb[1] <== tmp_5[1];
    cmul_1.inb[2] <== tmp_5[2];
    tmp_6[0] <== cmul_1.out[0];
    tmp_6[1] <== cmul_1.out[1];
    tmp_6[2] <== cmul_1.out[2];
//    log(7);
    signal tmp_15[3];

    tmp_15[0] <== tmp_6[0] - 0 + p;
    tmp_15[1] <== tmp_6[1];
    tmp_15[2] <== tmp_6[2];
//    log(8);
    signal tmp_7[3];

    tmp_7[0] <== evals[2][0] - publics[0] + p;
    tmp_7[1] <== evals[2][1];
    tmp_7[2] <== evals[2][2];
//    log(9);
    signal tmp_8[3];

    component cmul_2 = GLCMul();
    cmul_2.ina[0] <== evals[0][0];
    cmul_2.ina[1] <== evals[0][1];
    cmul_2.ina[2] <== evals[0][2];
    cmul_2.inb[0] <== tmp_7[0];
    cmul_2.inb[1] <== tmp_7[1];
    cmul_2.inb[2] <== tmp_7[2];
    tmp_8[0] <== cmul_2.out[0];
    tmp_8[1] <== cmul_2.out[1];
    tmp_8[2] <== cmul_2.out[2];
//    log(10);
    signal tmp_16[3];

    tmp_16[0] <== tmp_8[0] - 0 + p;
    tmp_16[1] <== tmp_8[1];
    tmp_16[2] <== tmp_8[2];
//    log(11);
    signal tmp_9[3];

    component cmul_3 = GLCMul();
    cmul_3.ina[0] <== challenges[4][0];
    cmul_3.ina[1] <== challenges[4][1];
    cmul_3.ina[2] <== challenges[4][2];
    cmul_3.inb[0] <== tmp_14[0];
    cmul_3.inb[1] <== tmp_14[1];
    cmul_3.inb[2] <== tmp_14[2];
    tmp_9[0] <== cmul_3.out[0];
    tmp_9[1] <== cmul_3.out[1];
    tmp_9[2] <== cmul_3.out[2];
//    log(12);
    signal tmp_10[3];

    tmp_10[0] <== tmp_9[0] + tmp_15[0];
    tmp_10[1] <== tmp_9[1] + tmp_15[1];
    tmp_10[2] <== tmp_9[2] + tmp_15[2];
//    log(13);
    signal tmp_11[3];

    component cmul_4 = GLCMul();
    cmul_4.ina[0] <== challenges[4][0];
    cmul_4.ina[1] <== challenges[4][1];
    cmul_4.ina[2] <== challenges[4][2];
    cmul_4.inb[0] <== tmp_10[0];
    cmul_4.inb[1] <== tmp_10[1];
    cmul_4.inb[2] <== tmp_10[2];
    tmp_11[0] <== cmul_4.out[0];
    tmp_11[1] <== cmul_4.out[1];
    tmp_11[2] <== cmul_4.out[2];
//    log(14);
    signal tmp_12[3];

    tmp_12[0] <== tmp_11[0] + tmp_16[0];
    tmp_12[1] <== tmp_11[1] + tmp_16[1];
    tmp_12[2] <== tmp_11[2] + tmp_16[2];
//    log(15);
    signal tmp_13[3];

    component cmul_5 = GLCMul();
    cmul_5.ina[0] <== evals[5][0];
    cmul_5.ina[1] <== evals[5][1];
    cmul_5.ina[2] <== evals[5][2];
    cmul_5.inb[0] <== Z[0];
    cmul_5.inb[1] <== Z[1];
    cmul_5.inb[2] <== Z[2];
    tmp_13[0] <== cmul_5.out[0];
    tmp_13[1] <== cmul_5.out[1];
    tmp_13[2] <== cmul_5.out[2];
//    log(16);
    signal tmp_17[3];

    tmp_17[0] <== tmp_12[0] - tmp_13[0] + p;
    tmp_17[1] <== tmp_12[1] - tmp_13[1] + p;
    tmp_17[2] <== tmp_12[2] - tmp_13[2] + p;
//    log(17);

// Final Verification
    component normC = GLCNorm();
    normC.in[0] <== tmp_17[0];
    normC.in[1] <== tmp_17[1];
    normC.in[2] <== tmp_17[2];

    normC.out[0] === 0;
    normC.out[1] === 0;
    normC.out[2] === 0;

}

template VerifyQuery() {
    signal input ys[13];
    signal input challenges[8][3];
    signal input evals[6][3];
    signal input tree1[2];


    signal input tree4[3];
    signal input consts[1];
    signal output out[3];

///////////
// Mapping
///////////
    component mapValues = MapValues();

    for (var i=0; i< 2; i++ ) {
        mapValues.vals1[i] <== tree1[i];
    }
    for (var i=0; i< 3; i++ ) {
        mapValues.vals4[i] <== tree4[i];
    }


    var p = 0xFFFFFFFF00000001;

    component xacc[13-1];
    for (var i=1; i<13; i++ ) {
        xacc[i-1] = GLMul();
        if (i==1) {
            xacc[i-1].ina <== ys[0]*(49 * roots(13)-49) + 49;
        } else {
            xacc[i-1].ina <== xacc[i-2].out;
        }
        xacc[i-1].inb <== ys[i]*(roots(13 - i) - 1) +1;
    }

    signal X <== xacc[11].out;


    component den1inv = GLCInv();
    den1inv.in[0] <== X - challenges[7][0] + p;
    den1inv.in[1] <== -challenges[7][1] + p;
    den1inv.in[2] <== -challenges[7][2] + p;


    component xDivXSubXi = GLCMul();
    xDivXSubXi.ina[0] <== X;
    xDivXSubXi.ina[1] <== 0;
    xDivXSubXi.ina[2] <== 0;
    xDivXSubXi.inb[0] <== den1inv.out[0];
    xDivXSubXi.inb[1] <== den1inv.out[1];
    xDivXSubXi.inb[2] <== den1inv.out[2];

    component wXi = GLCMul();
    wXi.ina[0] <== roots(4);
    wXi.ina[1] <== 0;
    wXi.ina[2] <== 0;
    wXi.inb[0] <== challenges[7][0];
    wXi.inb[1] <== challenges[7][1];
    wXi.inb[2] <== challenges[7][2];


    component den2inv = GLCInv();
    den2inv.in[0] <== X - wXi.out[0] + p;
    den2inv.in[1] <== -wXi.out[1] + p;
    den2inv.in[2] <== -wXi.out[2] + p;

    component xDivXSubWXi = GLCMul();
    xDivXSubWXi.ina[0] <== X;
    xDivXSubWXi.ina[1] <== 0;
    xDivXSubWXi.ina[2] <== 0;
    xDivXSubWXi.inb[0] <== den2inv.out[0];
    xDivXSubWXi.inb[1] <== den2inv.out[1];
    xDivXSubWXi.inb[2] <== den2inv.out[2];

        signal tmp_0[3];

        component cmul_0 = GLCMul();
    cmul_0.ina[0] <== challenges[5][0];
    cmul_0.ina[1] <== challenges[5][1];
    cmul_0.ina[2] <== challenges[5][2];
    cmul_0.inb[0] <== mapValues.tree1_0;
    cmul_0.inb[1] <== 0;
    cmul_0.inb[2] <== 0;
    tmp_0[0] <== cmul_0.out[0];
    tmp_0[1] <== cmul_0.out[1];
    tmp_0[2] <== cmul_0.out[2];
//    log(0);
    signal tmp_1[3];

    tmp_1[0] <== tmp_0[0] + mapValues.tree1_1;
    tmp_1[1] <== tmp_0[1];
    tmp_1[2] <== tmp_0[2];
//    log(1);
    signal tmp_2[3];

    component cmul_1 = GLCMul();
    cmul_1.ina[0] <== challenges[5][0];
    cmul_1.ina[1] <== challenges[5][1];
    cmul_1.ina[2] <== challenges[5][2];
    cmul_1.inb[0] <== tmp_1[0];
    cmul_1.inb[1] <== tmp_1[1];
    cmul_1.inb[2] <== tmp_1[2];
    tmp_2[0] <== cmul_1.out[0];
    tmp_2[1] <== cmul_1.out[1];
    tmp_2[2] <== cmul_1.out[2];
//    log(2);
    signal tmp_3[3];

    tmp_3[0] <== tmp_2[0] + mapValues.tree4_0[0];
    tmp_3[1] <== tmp_2[1] + mapValues.tree4_0[1];
    tmp_3[2] <== tmp_2[2] + mapValues.tree4_0[2];
//    log(3);
    signal tmp_4[3];

    component cmul_2 = GLCMul();
    cmul_2.ina[0] <== challenges[5][0];
    cmul_2.ina[1] <== challenges[5][1];
    cmul_2.ina[2] <== challenges[5][2];
    cmul_2.inb[0] <== tmp_3[0];
    cmul_2.inb[1] <== tmp_3[1];
    cmul_2.inb[2] <== tmp_3[2];
    tmp_4[0] <== cmul_2.out[0];
    tmp_4[1] <== cmul_2.out[1];
    tmp_4[2] <== cmul_2.out[2];
//    log(4);
    signal tmp_5[3];

    tmp_5[0] <== consts[0] - evals[0][0] + p;
    tmp_5[1] <== -evals[0][1] + p;
    tmp_5[2] <== -evals[0][2] + p;
//    log(5);
    signal tmp_6[3];

    component cmul_3 = GLCMul();
    cmul_3.ina[0] <== tmp_5[0];
    cmul_3.ina[1] <== tmp_5[1];
    cmul_3.ina[2] <== tmp_5[2];
    cmul_3.inb[0] <== challenges[6][0];
    cmul_3.inb[1] <== challenges[6][1];
    cmul_3.inb[2] <== challenges[6][2];
    tmp_6[0] <== cmul_3.out[0];
    tmp_6[1] <== cmul_3.out[1];
    tmp_6[2] <== cmul_3.out[2];
//    log(6);
    signal tmp_7[3];

    tmp_7[0] <== mapValues.tree1_1 - evals[2][0] + p;
    tmp_7[1] <== -evals[2][1] + p;
    tmp_7[2] <== -evals[2][2] + p;
//    log(7);
    signal tmp_8[3];

    tmp_8[0] <== tmp_6[0] + tmp_7[0];
    tmp_8[1] <== tmp_6[1] + tmp_7[1];
    tmp_8[2] <== tmp_6[2] + tmp_7[2];
//    log(8);
    signal tmp_9[3];

    component cmul_4 = GLCMul();
    cmul_4.ina[0] <== tmp_8[0];
    cmul_4.ina[1] <== tmp_8[1];
    cmul_4.ina[2] <== tmp_8[2];
    cmul_4.inb[0] <== challenges[6][0];
    cmul_4.inb[1] <== challenges[6][1];
    cmul_4.inb[2] <== challenges[6][2];
    tmp_9[0] <== cmul_4.out[0];
    tmp_9[1] <== cmul_4.out[1];
    tmp_9[2] <== cmul_4.out[2];
//    log(9);
    signal tmp_10[3];

    tmp_10[0] <== mapValues.tree1_0 - evals[3][0] + p;
    tmp_10[1] <== -evals[3][1] + p;
    tmp_10[2] <== -evals[3][2] + p;
//    log(10);
    signal tmp_11[3];

    tmp_11[0] <== tmp_9[0] + tmp_10[0];
    tmp_11[1] <== tmp_9[1] + tmp_10[1];
    tmp_11[2] <== tmp_9[2] + tmp_10[2];
//    log(11);
    signal tmp_12[3];

    component cmul_5 = GLCMul();
    cmul_5.ina[0] <== tmp_11[0];
    cmul_5.ina[1] <== tmp_11[1];
    cmul_5.ina[2] <== tmp_11[2];
    cmul_5.inb[0] <== challenges[6][0];
    cmul_5.inb[1] <== challenges[6][1];
    cmul_5.inb[2] <== challenges[6][2];
    tmp_12[0] <== cmul_5.out[0];
    tmp_12[1] <== cmul_5.out[1];
    tmp_12[2] <== cmul_5.out[2];
//    log(12);
    signal tmp_13[3];

    tmp_13[0] <== mapValues.tree4_0[0] - evals[5][0] + p;
    tmp_13[1] <== mapValues.tree4_0[1] - evals[5][1] + p;
    tmp_13[2] <== mapValues.tree4_0[2] - evals[5][2] + p;
//    log(13);
    signal tmp_14[3];

    tmp_14[0] <== tmp_12[0] + tmp_13[0];
    tmp_14[1] <== tmp_12[1] + tmp_13[1];
    tmp_14[2] <== tmp_12[2] + tmp_13[2];
//    log(14);
    signal tmp_15[3];

    component cmul_6 = GLCMul();
    cmul_6.ina[0] <== tmp_14[0];
    cmul_6.ina[1] <== tmp_14[1];
    cmul_6.ina[2] <== tmp_14[2];
    cmul_6.inb[0] <== xDivXSubXi.out[0];
    cmul_6.inb[1] <== xDivXSubXi.out[1];
    cmul_6.inb[2] <== xDivXSubXi.out[2];
    tmp_15[0] <== cmul_6.out[0];
    tmp_15[1] <== cmul_6.out[1];
    tmp_15[2] <== cmul_6.out[2];
//    log(15);
    signal tmp_16[3];

    tmp_16[0] <== tmp_4[0] + tmp_15[0];
    tmp_16[1] <== tmp_4[1] + tmp_15[1];
    tmp_16[2] <== tmp_4[2] + tmp_15[2];
//    log(16);
    signal tmp_17[3];

    component cmul_7 = GLCMul();
    cmul_7.ina[0] <== challenges[5][0];
    cmul_7.ina[1] <== challenges[5][1];
    cmul_7.ina[2] <== challenges[5][2];
    cmul_7.inb[0] <== tmp_16[0];
    cmul_7.inb[1] <== tmp_16[1];
    cmul_7.inb[2] <== tmp_16[2];
    tmp_17[0] <== cmul_7.out[0];
    tmp_17[1] <== cmul_7.out[1];
    tmp_17[2] <== cmul_7.out[2];
//    log(17);
    signal tmp_18[3];

    tmp_18[0] <== mapValues.tree1_0 - evals[1][0] + p;
    tmp_18[1] <== -evals[1][1] + p;
    tmp_18[2] <== -evals[1][2] + p;
//    log(18);
    signal tmp_19[3];

    component cmul_8 = GLCMul();
    cmul_8.ina[0] <== tmp_18[0];
    cmul_8.ina[1] <== tmp_18[1];
    cmul_8.ina[2] <== tmp_18[2];
    cmul_8.inb[0] <== challenges[6][0];
    cmul_8.inb[1] <== challenges[6][1];
    cmul_8.inb[2] <== challenges[6][2];
    tmp_19[0] <== cmul_8.out[0];
    tmp_19[1] <== cmul_8.out[1];
    tmp_19[2] <== cmul_8.out[2];
//    log(19);
    signal tmp_20[3];

    tmp_20[0] <== mapValues.tree1_1 - evals[4][0] + p;
    tmp_20[1] <== -evals[4][1] + p;
    tmp_20[2] <== -evals[4][2] + p;
//    log(20);
    signal tmp_21[3];

    tmp_21[0] <== tmp_19[0] + tmp_20[0];
    tmp_21[1] <== tmp_19[1] + tmp_20[1];
    tmp_21[2] <== tmp_19[2] + tmp_20[2];
//    log(21);
    signal tmp_22[3];

    component cmul_9 = GLCMul();
    cmul_9.ina[0] <== tmp_21[0];
    cmul_9.ina[1] <== tmp_21[1];
    cmul_9.ina[2] <== tmp_21[2];
    cmul_9.inb[0] <== xDivXSubWXi.out[0];
    cmul_9.inb[1] <== xDivXSubWXi.out[1];
    cmul_9.inb[2] <== xDivXSubWXi.out[2];
    tmp_22[0] <== cmul_9.out[0];
    tmp_22[1] <== cmul_9.out[1];
    tmp_22[2] <== cmul_9.out[2];
//    log(22);
    signal tmp_23[3];

    tmp_23[0] <== tmp_17[0] + tmp_22[0];
    tmp_23[1] <== tmp_17[1] + tmp_22[1];
    tmp_23[2] <== tmp_17[2] + tmp_22[2];
//    log(23);


    // Final Normalization
    component normC = GLCNorm();
    normC.in[0] <== tmp_23[0];
    normC.in[1] <== tmp_23[1];
    normC.in[2] <== tmp_23[2];

    out[0] <== normC.out[0];
    out[1] <== normC.out[1];
    out[2] <== normC.out[2];
}


template MapValues() {
    signal input vals1[2];
    signal input vals4[3];

    signal output tree1_0;
    signal output tree1_1;
    signal output tree4_0[3];

    tree1_0 <== vals1[0];
    tree1_1 <== vals1[1];
    tree4_0[0] <== vals4[0];
    tree4_0[1] <== vals4[1];
    tree4_0[2] <== vals4[2];
}


template StarkVerifier() {
    signal input proverAddr;
    signal input publics[1];
    signal input root1;
    signal input root2;
    signal input root3;
    signal input root4;
    signal input evals[6][3];

    signal input s0_vals1[7][2];


    signal input s0_vals4[7][3];
    signal input s0_valsC[7][1];
    signal input s0_siblings1[7][4][16];


    signal input s0_siblings4[7][4][16];
    signal input s0_siblingsC[7][4][16];

    signal input s1_root;
    signal input s2_root;

    signal input s1_vals[7][12];
    signal input s1_siblings[7][3][16];
    signal input s2_vals[7][768];
    signal input s2_siblings[7][1][16];

    signal input finalPol[8][3];

    signal output publicsHash;

    signal challenges[8][3];
    signal s0_specialX[3];
    signal s1_specialX[3];
    signal s2_specialX[3];

    signal ys[7][13];

    var p = 0xFFFFFFFF00000001;

///////////
// challenge calculation
///////////


    component tcHahs_0 = PoseidonEx(16,17);
    tcHahs_0.inputs[0] <== root1;
    tcHahs_0.inputs[1] <== 0;
    tcHahs_0.inputs[2] <== 0;
    tcHahs_0.inputs[3] <== 0;
    tcHahs_0.inputs[4] <== 0;
    tcHahs_0.inputs[5] <== 0;
    tcHahs_0.inputs[6] <== 0;
    tcHahs_0.inputs[7] <== 0;
    tcHahs_0.inputs[8] <== 0;
    tcHahs_0.inputs[9] <== 0;
    tcHahs_0.inputs[10] <== 0;
    tcHahs_0.inputs[11] <== 0;
    tcHahs_0.inputs[12] <== 0;
    tcHahs_0.inputs[13] <== 0;
    tcHahs_0.inputs[14] <== 0;
    tcHahs_0.inputs[15] <== 0;
    tcHahs_0.initialState <== 0;
    component bn1togl3_0 = BN1toGL3();
    bn1togl3_0.in <== tcHahs_0.out[0];
    challenges[0][0] <== bn1togl3_0.out[0];
    challenges[0][1] <== bn1togl3_0.out[1];
    challenges[0][2] <== bn1togl3_0.out[2];
    component bn1togl3_1 = BN1toGL3();
    bn1togl3_1.in <== tcHahs_0.out[1];
    challenges[1][0] <== bn1togl3_1.out[0];
    challenges[1][1] <== bn1togl3_1.out[1];
    challenges[1][2] <== bn1togl3_1.out[2];
    component tcHahs_1 = PoseidonEx(16,17);
    tcHahs_1.inputs[0] <== root2;
    tcHahs_1.inputs[1] <== 0;
    tcHahs_1.inputs[2] <== 0;
    tcHahs_1.inputs[3] <== 0;
    tcHahs_1.inputs[4] <== 0;
    tcHahs_1.inputs[5] <== 0;
    tcHahs_1.inputs[6] <== 0;
    tcHahs_1.inputs[7] <== 0;
    tcHahs_1.inputs[8] <== 0;
    tcHahs_1.inputs[9] <== 0;
    tcHahs_1.inputs[10] <== 0;
    tcHahs_1.inputs[11] <== 0;
    tcHahs_1.inputs[12] <== 0;
    tcHahs_1.inputs[13] <== 0;
    tcHahs_1.inputs[14] <== 0;
    tcHahs_1.inputs[15] <== 0;
    tcHahs_1.initialState <== tcHahs_0.out[0];
    component bn1togl3_2 = BN1toGL3();
    bn1togl3_2.in <== tcHahs_1.out[0];
    challenges[2][0] <== bn1togl3_2.out[0];
    challenges[2][1] <== bn1togl3_2.out[1];
    challenges[2][2] <== bn1togl3_2.out[2];
    component bn1togl3_3 = BN1toGL3();
    bn1togl3_3.in <== tcHahs_1.out[1];
    challenges[3][0] <== bn1togl3_3.out[0];
    challenges[3][1] <== bn1togl3_3.out[1];
    challenges[3][2] <== bn1togl3_3.out[2];
    component tcHahs_2 = PoseidonEx(16,17);
    tcHahs_2.inputs[0] <== root3;
    tcHahs_2.inputs[1] <== 0;
    tcHahs_2.inputs[2] <== 0;
    tcHahs_2.inputs[3] <== 0;
    tcHahs_2.inputs[4] <== 0;
    tcHahs_2.inputs[5] <== 0;
    tcHahs_2.inputs[6] <== 0;
    tcHahs_2.inputs[7] <== 0;
    tcHahs_2.inputs[8] <== 0;
    tcHahs_2.inputs[9] <== 0;
    tcHahs_2.inputs[10] <== 0;
    tcHahs_2.inputs[11] <== 0;
    tcHahs_2.inputs[12] <== 0;
    tcHahs_2.inputs[13] <== 0;
    tcHahs_2.inputs[14] <== 0;
    tcHahs_2.inputs[15] <== 0;
    tcHahs_2.initialState <== tcHahs_1.out[0];
    component bn1togl3_4 = BN1toGL3();
    bn1togl3_4.in <== tcHahs_2.out[0];
    challenges[4][0] <== bn1togl3_4.out[0];
    challenges[4][1] <== bn1togl3_4.out[1];
    challenges[4][2] <== bn1togl3_4.out[2];
    component tcHahs_3 = PoseidonEx(16,17);
    tcHahs_3.inputs[0] <== root4;
    tcHahs_3.inputs[1] <== 0;
    tcHahs_3.inputs[2] <== 0;
    tcHahs_3.inputs[3] <== 0;
    tcHahs_3.inputs[4] <== 0;
    tcHahs_3.inputs[5] <== 0;
    tcHahs_3.inputs[6] <== 0;
    tcHahs_3.inputs[7] <== 0;
    tcHahs_3.inputs[8] <== 0;
    tcHahs_3.inputs[9] <== 0;
    tcHahs_3.inputs[10] <== 0;
    tcHahs_3.inputs[11] <== 0;
    tcHahs_3.inputs[12] <== 0;
    tcHahs_3.inputs[13] <== 0;
    tcHahs_3.inputs[14] <== 0;
    tcHahs_3.inputs[15] <== 0;
    tcHahs_3.initialState <== tcHahs_2.out[0];
    component bn1togl3_5 = BN1toGL3();
    bn1togl3_5.in <== tcHahs_3.out[0];
    challenges[5][0] <== bn1togl3_5.out[0];
    challenges[5][1] <== bn1togl3_5.out[1];
    challenges[5][2] <== bn1togl3_5.out[2];
    component bn1togl3_6 = BN1toGL3();
    bn1togl3_6.in <== tcHahs_3.out[1];
    challenges[6][0] <== bn1togl3_6.out[0];
    challenges[6][1] <== bn1togl3_6.out[1];
    challenges[6][2] <== bn1togl3_6.out[2];
    component bn1togl3_7 = BN1toGL3();
    bn1togl3_7.in <== tcHahs_3.out[2];
    challenges[7][0] <== bn1togl3_7.out[0];
    challenges[7][1] <== bn1togl3_7.out[1];
    challenges[7][2] <== bn1togl3_7.out[2];
    component bn1togl3_8 = BN1toGL3();
    bn1togl3_8.in <== tcHahs_3.out[3];
    s0_specialX[0] <== bn1togl3_8.out[0];
    s0_specialX[1] <== bn1togl3_8.out[1];
    s0_specialX[2] <== bn1togl3_8.out[2];
    component tcHahs_4 = PoseidonEx(16,17);
    tcHahs_4.inputs[0] <== s1_root;
    tcHahs_4.inputs[1] <== 0;
    tcHahs_4.inputs[2] <== 0;
    tcHahs_4.inputs[3] <== 0;
    tcHahs_4.inputs[4] <== 0;
    tcHahs_4.inputs[5] <== 0;
    tcHahs_4.inputs[6] <== 0;
    tcHahs_4.inputs[7] <== 0;
    tcHahs_4.inputs[8] <== 0;
    tcHahs_4.inputs[9] <== 0;
    tcHahs_4.inputs[10] <== 0;
    tcHahs_4.inputs[11] <== 0;
    tcHahs_4.inputs[12] <== 0;
    tcHahs_4.inputs[13] <== 0;
    tcHahs_4.inputs[14] <== 0;
    tcHahs_4.inputs[15] <== 0;
    tcHahs_4.initialState <== tcHahs_3.out[0];
    component bn1togl3_9 = BN1toGL3();
    bn1togl3_9.in <== tcHahs_4.out[0];
    s1_specialX[0] <== bn1togl3_9.out[0];
    s1_specialX[1] <== bn1togl3_9.out[1];
    s1_specialX[2] <== bn1togl3_9.out[2];
    component tcHahs_5 = PoseidonEx(16,17);
    tcHahs_5.inputs[0] <== s2_root;
    tcHahs_5.inputs[1] <== 0;
    tcHahs_5.inputs[2] <== 0;
    tcHahs_5.inputs[3] <== 0;
    tcHahs_5.inputs[4] <== 0;
    tcHahs_5.inputs[5] <== 0;
    tcHahs_5.inputs[6] <== 0;
    tcHahs_5.inputs[7] <== 0;
    tcHahs_5.inputs[8] <== 0;
    tcHahs_5.inputs[9] <== 0;
    tcHahs_5.inputs[10] <== 0;
    tcHahs_5.inputs[11] <== 0;
    tcHahs_5.inputs[12] <== 0;
    tcHahs_5.inputs[13] <== 0;
    tcHahs_5.inputs[14] <== 0;
    tcHahs_5.inputs[15] <== 0;
    tcHahs_5.initialState <== tcHahs_4.out[0];
    component bn1togl3_10 = BN1toGL3();
    bn1togl3_10.in <== tcHahs_5.out[0];
    s2_specialX[0] <== bn1togl3_10.out[0];
    s2_specialX[1] <== bn1togl3_10.out[1];
    s2_specialX[2] <== bn1togl3_10.out[2];
    component tcHahs_6 = PoseidonEx(16,17);
    tcHahs_6.inputs[0] <== finalPol[0][0];
    tcHahs_6.inputs[1] <== finalPol[0][1];
    tcHahs_6.inputs[2] <== finalPol[0][2];
    tcHahs_6.inputs[3] <== finalPol[1][0];
    tcHahs_6.inputs[4] <== finalPol[1][1];
    tcHahs_6.inputs[5] <== finalPol[1][2];
    tcHahs_6.inputs[6] <== finalPol[2][0];
    tcHahs_6.inputs[7] <== finalPol[2][1];
    tcHahs_6.inputs[8] <== finalPol[2][2];
    tcHahs_6.inputs[9] <== finalPol[3][0];
    tcHahs_6.inputs[10] <== finalPol[3][1];
    tcHahs_6.inputs[11] <== finalPol[3][2];
    tcHahs_6.inputs[12] <== finalPol[4][0];
    tcHahs_6.inputs[13] <== finalPol[4][1];
    tcHahs_6.inputs[14] <== finalPol[4][2];
    tcHahs_6.inputs[15] <== finalPol[5][0];
    tcHahs_6.initialState <== tcHahs_5.out[0];
    component tcHahs_7 = PoseidonEx(16,17);
    tcHahs_7.inputs[0] <== finalPol[5][1];
    tcHahs_7.inputs[1] <== finalPol[5][2];
    tcHahs_7.inputs[2] <== finalPol[6][0];
    tcHahs_7.inputs[3] <== finalPol[6][1];
    tcHahs_7.inputs[4] <== finalPol[6][2];
    tcHahs_7.inputs[5] <== finalPol[7][0];
    tcHahs_7.inputs[6] <== finalPol[7][1];
    tcHahs_7.inputs[7] <== finalPol[7][2];
    tcHahs_7.inputs[8] <== 0;
    tcHahs_7.inputs[9] <== 0;
    tcHahs_7.inputs[10] <== 0;
    tcHahs_7.inputs[11] <== 0;
    tcHahs_7.inputs[12] <== 0;
    tcHahs_7.inputs[13] <== 0;
    tcHahs_7.inputs[14] <== 0;
    tcHahs_7.inputs[15] <== 0;
    tcHahs_7.initialState <== tcHahs_6.out[0];
    component tcN2b_0 = Num2Bits_strict();
    tcN2b_0.in <== tcHahs_7.out[0];
    ys[0][0] <== tcN2b_0.out[0];
    ys[0][1] <== tcN2b_0.out[1];
    ys[0][2] <== tcN2b_0.out[2];
    ys[0][3] <== tcN2b_0.out[3];
    ys[0][4] <== tcN2b_0.out[4];
    ys[0][5] <== tcN2b_0.out[5];
    ys[0][6] <== tcN2b_0.out[6];
    ys[0][7] <== tcN2b_0.out[7];
    ys[0][8] <== tcN2b_0.out[8];
    ys[0][9] <== tcN2b_0.out[9];
    ys[0][10] <== tcN2b_0.out[10];
    ys[0][11] <== tcN2b_0.out[11];
    ys[0][12] <== tcN2b_0.out[12];
    ys[1][0] <== tcN2b_0.out[13];
    ys[1][1] <== tcN2b_0.out[14];
    ys[1][2] <== tcN2b_0.out[15];
    ys[1][3] <== tcN2b_0.out[16];
    ys[1][4] <== tcN2b_0.out[17];
    ys[1][5] <== tcN2b_0.out[18];
    ys[1][6] <== tcN2b_0.out[19];
    ys[1][7] <== tcN2b_0.out[20];
    ys[1][8] <== tcN2b_0.out[21];
    ys[1][9] <== tcN2b_0.out[22];
    ys[1][10] <== tcN2b_0.out[23];
    ys[1][11] <== tcN2b_0.out[24];
    ys[1][12] <== tcN2b_0.out[25];
    ys[2][0] <== tcN2b_0.out[26];
    ys[2][1] <== tcN2b_0.out[27];
    ys[2][2] <== tcN2b_0.out[28];
    ys[2][3] <== tcN2b_0.out[29];
    ys[2][4] <== tcN2b_0.out[30];
    ys[2][5] <== tcN2b_0.out[31];
    ys[2][6] <== tcN2b_0.out[32];
    ys[2][7] <== tcN2b_0.out[33];
    ys[2][8] <== tcN2b_0.out[34];
    ys[2][9] <== tcN2b_0.out[35];
    ys[2][10] <== tcN2b_0.out[36];
    ys[2][11] <== tcN2b_0.out[37];
    ys[2][12] <== tcN2b_0.out[38];
    ys[3][0] <== tcN2b_0.out[39];
    ys[3][1] <== tcN2b_0.out[40];
    ys[3][2] <== tcN2b_0.out[41];
    ys[3][3] <== tcN2b_0.out[42];
    ys[3][4] <== tcN2b_0.out[43];
    ys[3][5] <== tcN2b_0.out[44];
    ys[3][6] <== tcN2b_0.out[45];
    ys[3][7] <== tcN2b_0.out[46];
    ys[3][8] <== tcN2b_0.out[47];
    ys[3][9] <== tcN2b_0.out[48];
    ys[3][10] <== tcN2b_0.out[49];
    ys[3][11] <== tcN2b_0.out[50];
    ys[3][12] <== tcN2b_0.out[51];
    ys[4][0] <== tcN2b_0.out[52];
    ys[4][1] <== tcN2b_0.out[53];
    ys[4][2] <== tcN2b_0.out[54];
    ys[4][3] <== tcN2b_0.out[55];
    ys[4][4] <== tcN2b_0.out[56];
    ys[4][5] <== tcN2b_0.out[57];
    ys[4][6] <== tcN2b_0.out[58];
    ys[4][7] <== tcN2b_0.out[59];
    ys[4][8] <== tcN2b_0.out[60];
    ys[4][9] <== tcN2b_0.out[61];
    ys[4][10] <== tcN2b_0.out[62];
    ys[4][11] <== tcN2b_0.out[63];
    ys[4][12] <== tcN2b_0.out[64];
    ys[5][0] <== tcN2b_0.out[65];
    ys[5][1] <== tcN2b_0.out[66];
    ys[5][2] <== tcN2b_0.out[67];
    ys[5][3] <== tcN2b_0.out[68];
    ys[5][4] <== tcN2b_0.out[69];
    ys[5][5] <== tcN2b_0.out[70];
    ys[5][6] <== tcN2b_0.out[71];
    ys[5][7] <== tcN2b_0.out[72];
    ys[5][8] <== tcN2b_0.out[73];
    ys[5][9] <== tcN2b_0.out[74];
    ys[5][10] <== tcN2b_0.out[75];
    ys[5][11] <== tcN2b_0.out[76];
    ys[5][12] <== tcN2b_0.out[77];
    ys[6][0] <== tcN2b_0.out[78];
    ys[6][1] <== tcN2b_0.out[79];
    ys[6][2] <== tcN2b_0.out[80];
    ys[6][3] <== tcN2b_0.out[81];
    ys[6][4] <== tcN2b_0.out[82];
    ys[6][5] <== tcN2b_0.out[83];
    ys[6][6] <== tcN2b_0.out[84];
    ys[6][7] <== tcN2b_0.out[85];
    ys[6][8] <== tcN2b_0.out[86];
    ys[6][9] <== tcN2b_0.out[87];
    ys[6][10] <== tcN2b_0.out[88];
    ys[6][11] <== tcN2b_0.out[89];
    ys[6][12] <== tcN2b_0.out[90];

///////////
// Constrain polynomial check in vauations
///////////
    component verifyEvaluations = VerifyEvaluations();
    for (var i=0; i<8; i++) {
        for (var k=0; k<3; k++) {
            verifyEvaluations.challenges[i][k] <== challenges[i][k];
        }
    }
    for (var i=0; i<1; i++) {
        verifyEvaluations.publics[i] <== publics[i];
    }
    for (var i=0; i<6; i++) {
        for (var k=0; k<3; k++) {
            verifyEvaluations.evals[i][k] <== evals[i][k];
        }
    }

///////////
// Step0 Check and evaluate queries
///////////

    component verifyQueries[7];
    component s0_merkle1[7];


    component s0_merkle4[7];
    component s0_merkleC[7];
    component s0_lowValues[7];

    for (var q=0; q<7; q++) {
        verifyQueries[q] = VerifyQuery();
        s0_merkle1[q] = MerkleHash(1, 2, 8192);


        s0_merkle4[q] = MerkleHash(1, 3, 8192);
        s0_merkleC[q] = MerkleHash(1, 1, 8192);
        s0_lowValues[q] = TreeSelector(2, 3) ;

        for (var i=0; i<13; i++ ) {
            verifyQueries[q].ys[i] <== ys[q][i];
            s0_merkle1[q].key[i] <== ys[q][i];


            s0_merkle4[q].key[i] <== ys[q][i];
            s0_merkleC[q].key[i] <== ys[q][i];
        }
        for (var i=0; i<2; i++ ) {
            verifyQueries[q].tree1[i] <== s0_vals1[q][i];
            s0_merkle1[q].values[i][0] <== s0_vals1[q][i];
        }


        for (var i=0; i<3; i++ ) {
            verifyQueries[q].tree4[i] <== s0_vals4[q][i];
            s0_merkle4[q].values[i][0] <== s0_vals4[q][i];
        }
        for (var i=0; i<1; i++ ) {
            verifyQueries[q].consts[i] <== s0_valsC[q][i];
            s0_merkleC[q].values[i][0] <== s0_valsC[q][i];
        }
        for (var i=0; i<8; i++) {
            for (var e=0; e<3; e++) {
                verifyQueries[q].challenges[i][e] <== challenges[i][e];
            }
        }
        for (var i=0; i<6; i++) {
            for (var e=0; e<3; e++) {
                verifyQueries[q].evals[i][e] <== evals[i][e];
            }
        }
        for (var i=0; i<4;i++) {
            for (var j=0; j<16; j++) {
                s0_merkle1[q].siblings[i][j] <== s0_siblings1[q][i][j];


                s0_merkle4[q].siblings[i][j] <== s0_siblings4[q][i][j];
                s0_merkleC[q].siblings[i][j] <== s0_siblingsC[q][i][j];
            }
        }
        s0_merkle1[q].root === root1;


        s0_merkle4[q].root === root4;
        s0_merkleC[q].root === 4591939959486076946717512284814288781798881227728245492925236643363464951398;

        for (var i=0; i<4; i++) {
            for (var e=0; e<3; e++) {
                s0_lowValues[q].values[i][e] <== s1_vals[q][i*3+e];
            }
        }
        for (var i=0; i<2; i++) {
            s0_lowValues[q].key[i] <== ys[q][i + 11];
        }
        for (var e=0; e<3; e++) {
            s0_lowValues[q].out[e] === verifyQueries[q].out[e];
        }

    }

    component s1_merkle[7];
    component s1_fft[7];
    component s1_evalPol[7];
    component s1_lowValues[7];
    component s1_cNorm[7];
    component s1_sx[7][10];
    component s1_evalXprime[7];
    signal s1_X[7];

    for (var q=0; q<7; q++) {
        s1_merkle[q] = MerkleHash(3, 4, 2048);
        s1_fft[q] = FFT(2, 1);
        s1_evalPol[q] = EvalPol(4);
        s1_lowValues[q] = TreeSelector(8, 3) ;
        for (var i=0; i< 4; i++) {
            for (var e=0; e<3; e++) {
                s1_merkle[q].values[i][e] <== s1_vals[q][i*3+e];
                s1_fft[q].in[i][e] <== s1_vals[q][i*3+e];
            }
        }
        for (var i=0; i<3; i++) {
            for (var j=0; j<16; j++) {
                s1_merkle[q].siblings[i][j] <== s1_siblings[q][i][j];
            }
        }
        for (var i=0; i<11; i++) {
            s1_merkle[q].key[i] <== ys[q][i];
        }

        for (var i=1; i<11; i++ ) {
            s1_sx[q][i-1] = GLMul();
            if (i==1) {
                s1_sx[q][i-1].ina <== ys[q][0] * (4222092901088788069 - 5646962470228954384) + 5646962470228954384;
            } else {
                s1_sx[q][i-1].ina <== s1_sx[q][i-2].out;
            }
            s1_sx[q][i-1].inb <== ys[q][i] * (_inv1(roots(13 -i)) -1) +1;
        }

        s1_X[q] <== s1_sx[q][9].out;

/*
        s1_sx[q][0] <==  5646962470228954384 *  ( ys[q][0] * 3968367389790187849 +1);
        for (var i=1; i<11; i++) {
            s1_sx[q][i] <== s1_sx[q][i-1] *  ( ys[q][i] * ((1/roots(13 -i)) -1) +1);
        }
*/

        for (var i=0; i< 4; i++) {
            for (var e=0; e<3; e++) {
                s1_evalPol[q].pol[i][e] <== s1_fft[q].out[i][e];
            }
        }

        s1_evalXprime[q] = GLCMul();
        s1_evalXprime[q].ina[0] <== s1_specialX[0];
        s1_evalXprime[q].ina[1] <== s1_specialX[1];
        s1_evalXprime[q].ina[2] <== s1_specialX[2];
        s1_evalXprime[q].inb[0] <== s1_X[q];
        s1_evalXprime[q].inb[1] <== 0;
        s1_evalXprime[q].inb[2] <== 0;
        for (var e=0; e<3; e++) {
            s1_evalPol[q].x[e] <== s1_evalXprime[q].out[e];
        }
        for (var i=0; i<256; i++) {
            for (var e=0; e<3; e++) {
                s1_lowValues[q].values[i][e] <== s2_vals[q][i*3+e];
            }
        }
        for (var i=0; i<8; i++) {
            s1_lowValues[q].key[i] <== ys[q][i + 3];
        }
        s1_cNorm[q] = GLCNorm();
        for (var e=0; e<3; e++) {
            s1_cNorm[q].in[e] <== s1_evalPol[q].out[e] - s1_lowValues[q].out[e] + p;
        }
        for (var e=0; e<3; e++) {
            s1_cNorm[q].out[e] === 0;
        }

        s1_merkle[q].root === s1_root;
    }
    component s2_merkle[7];
    component s2_fft[7];
    component s2_evalPol[7];
    component s2_lowValues[7];
    component s2_cNorm[7];
    component s2_sx[7][2];
    component s2_evalXprime[7];
    signal s2_X[7];

    for (var q=0; q<7; q++) {
        s2_merkle[q] = MerkleHash(3, 256, 8);
        s2_fft[q] = FFT(8, 1);
        s2_evalPol[q] = EvalPol(256);
        s2_lowValues[q] = TreeSelector(3, 3) ;
        for (var i=0; i< 256; i++) {
            for (var e=0; e<3; e++) {
                s2_merkle[q].values[i][e] <== s2_vals[q][i*3+e];
                s2_fft[q].in[i][e] <== s2_vals[q][i*3+e];
            }
        }
        for (var i=0; i<1; i++) {
            for (var j=0; j<16; j++) {
                s2_merkle[q].siblings[i][j] <== s2_siblings[q][i][j];
            }
        }
        for (var i=0; i<3; i++) {
            s2_merkle[q].key[i] <== ys[q][i];
        }

        for (var i=1; i<3; i++ ) {
            s2_sx[q][i-1] = GLMul();
            if (i==1) {
                s2_sx[q][i-1].ina <== ys[q][0] * (5167815234923408097 - 12421013511830570338) + 12421013511830570338;
            } else {
                s2_sx[q][i-1].ina <== s2_sx[q][i-2].out;
            }
            s2_sx[q][i-1].inb <== ys[q][i] * (_inv1(roots(11 -i)) -1) +1;
        }

        s2_X[q] <== s2_sx[q][1].out;

/*
        s2_sx[q][0] <==  12421013511830570338 *  ( ys[q][0] * 8548973421900915980 +1);
        for (var i=1; i<3; i++) {
            s2_sx[q][i] <== s2_sx[q][i-1] *  ( ys[q][i] * ((1/roots(11 -i)) -1) +1);
        }
*/

        for (var i=0; i< 256; i++) {
            for (var e=0; e<3; e++) {
                s2_evalPol[q].pol[i][e] <== s2_fft[q].out[i][e];
            }
        }

        s2_evalXprime[q] = GLCMul();
        s2_evalXprime[q].ina[0] <== s2_specialX[0];
        s2_evalXprime[q].ina[1] <== s2_specialX[1];
        s2_evalXprime[q].ina[2] <== s2_specialX[2];
        s2_evalXprime[q].inb[0] <== s2_X[q];
        s2_evalXprime[q].inb[1] <== 0;
        s2_evalXprime[q].inb[2] <== 0;
        for (var e=0; e<3; e++) {
            s2_evalPol[q].x[e] <== s2_evalXprime[q].out[e];
        }
        for (var i=0; i<8; i++) {
            for (var e=0; e<3; e++) {
                s2_lowValues[q].values[i][e] <== finalPol[i][e];
            }
        }
        for (var i=0; i<3; i++) {
            s2_lowValues[q].key[i] <== ys[q][i];
        }
        s2_cNorm[q] = GLCNorm();
        for (var e=0; e<3; e++) {
            s2_cNorm[q].in[e] <== s2_evalPol[q].out[e] - s2_lowValues[q].out[e] + p;
        }
        for (var e=0; e<3; e++) {
            s2_cNorm[q].out[e] === 0;
        }

        s2_merkle[q].root === s2_root;
    }

///////
// Check Degree last pol
///////
// Last FFT
    component lastIFFT = FFT(3, 1 );

    for (var k=0; k< 8; k++ ){
        for (var e=0; e<3; e++) {
            lastIFFT.in[k][e] <== finalPol[k][e];
        }
    }

    for (var k= 67108864; k< 8; k++ ) {
        for (var e=0; e<3; e++) {
            lastIFFT.out[k][e] === 0;
        }
    }

//////
// Calculate Publics Hash
//////

    component publicsHasher = Sha256(224);
    component n2bProverAddr = Num2Bits(160);
    component n2bPublics[1 ];
    component cmpPublics[1 ];

    n2bProverAddr.in <== proverAddr;
    for (var i=0; i<160; i++) {
        publicsHasher.in[160 - 1 -i] <== n2bProverAddr.out[i];
    }

    var offset = 160;

    for (var i=0; i<1; i++) {
        n2bPublics[i] = Num2Bits(64);
        cmpPublics[i] = CompConstant64(0xFFFFFFFF00000000);
        n2bPublics[i].in <== publics[i];
        for (var j=0; j<64; j++) {
            publicsHasher.in[offset + 64 - 1 -j] <== n2bPublics[i].out[j];
            cmpPublics[i].in[j] <== n2bPublics[i].out[j];
        }
        cmpPublics[i].out === 0;
        offset += 64;
    }

    component n2bPublicsHash = Bits2Num(256);
    for (var i = 0; i < 256; i++) {
        n2bPublicsHash.in[i] <== publicsHasher.out[255-i];
    }

    publicsHash <== n2bPublicsHash.out;
}

component main = StarkVerifier();

