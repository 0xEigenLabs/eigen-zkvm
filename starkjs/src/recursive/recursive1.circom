pragma circom 2.1.0;
pragma custom_templates;

include "c12a.verifier.circom";

template Main() {

    signal input publics[44];
    signal input rootC[4];

    signal input root1[4];
    signal input root2[4];
    signal input root3[4];
    signal input root4[4];
    signal input evals[94][3];

    signal input s0_vals1[64][12];
    signal input s0_vals3[64][41];
    signal input s0_vals4[64][12];
    signal input s0_valsC[64][34];
    signal input s0_siblings1[64][24][4];
    signal input s0_siblings3[64][24][4];
    signal input s0_siblings4[64][24][4];
    signal input s0_siblingsC[64][24][4];

    signal input s1_root[4];
    signal input s2_root[4];
    signal input s3_root[4];
    signal input s4_root[4];

    signal input s1_vals[64][48];
    signal input s1_siblings[64][20][4];
    signal input s2_vals[64][96];
    signal input s2_siblings[64][15][4];
    signal input s3_vals[64][96];
    signal input s3_siblings[64][10][4];
    signal input s4_vals[64][96];
    signal input s4_siblings[64][5][4];

    signal input finalPol[32][3];



    component vA = StarkVerifier();

    vA.publics <== publics;
    vA.root1 <== root1;
    vA.root2 <== root2;
    vA.root3 <== root3;
    vA.root4 <== root4;
    vA.evals <== evals;
    vA.s0_vals1 <== s0_vals1;
    vA.s0_vals3 <== s0_vals3;
    vA.s0_vals4 <== s0_vals4;
    vA.s0_valsC <== s0_valsC;
    vA.s0_siblings1 <== s0_siblings1;
    vA.s0_siblings3 <== s0_siblings3;
    vA.s0_siblings4 <== s0_siblings4;
    vA.s0_siblingsC <== s0_siblingsC;
    vA.s1_root <== s1_root;
    vA.s2_root <== s2_root;
    vA.s3_root <== s3_root;
    vA.s4_root <== s4_root;
    vA.s1_vals <== s1_vals;
    vA.s1_siblings <== s1_siblings;
    vA.s2_vals <== s2_vals;
    vA.s2_siblings <== s2_siblings;
    vA.s3_vals <== s3_vals;
    vA.s3_siblings <== s3_siblings;
    vA.s4_vals <== s4_vals;
    vA.s4_siblings <== s4_siblings;
    vA.finalPol <== finalPol;

}

component main {public [publics, rootC]}= Main();