pragma circom 2.0.6;

include "gl.circom";

template EvalPol(n) {
    signal input pol[n][3];
    signal input x[3];
    signal output out[3];

    component cmul[n-1];

    for (var i=1; i<n; i++) {
        cmul[i-1] = GLCMulAdd();
        if (i==1) {
            cmul[i-1].ina[0] <== pol[n-1][0];
            cmul[i-1].ina[1] <== pol[n-1][1];
            cmul[i-1].ina[2] <== pol[n-1][2];
        } else {
            cmul[i-1].ina[0] <== cmul[i-2].out[0];
            cmul[i-1].ina[1] <== cmul[i-2].out[1];
            cmul[i-1].ina[2] <== cmul[i-2].out[2];
        }
        cmul[i-1].inb[0] <== x[0];
        cmul[i-1].inb[1] <== x[1];
        cmul[i-1].inb[2] <== x[2];

        cmul[i-1].inc[0] <== pol[n-i-1][0];
        cmul[i-1].inc[1] <== pol[n-i-1][1];
        cmul[i-1].inc[2] <== pol[n-i-1][2];
    }

    if (n>1) {
        out[0] <== cmul[n-2].out[0];
        out[1] <== cmul[n-2].out[1];
        out[2] <== cmul[n-2].out[2];
    } else {
        out[0] <== pol[n-1][0];
        out[1] <== pol[n-1][1];
        out[2] <== pol[n-1][2];
    }
}