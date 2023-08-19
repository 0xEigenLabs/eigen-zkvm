pragma circom 2.0.6;

include "gl.circom";

template EvalPol(n, n_limb) {
    signal input pol[n][n_limb];
    signal input x[n_limb];
    signal output out[n_limb];

    component cmul[n-1];

    for (var i=1; i<n; i++) {
        cmul[i-1] = GLCMulAdd();
        if (i==1) {
            for (var j = 0; j < n_limb; j ++) {
                cmul[i-1].ina[j] <== pol[n-1][j];
            }
        } else {
            for (var j = 0; j < n_limb; j ++) {
                cmul[i-1].ina[j] <== cmul[i-2].out[j];
            }
        }
        for (var j = 0; j < n_limb; j ++) {
            cmul[i-1].inb[j] <== x[j];
        }

        for (var j = 0; j < n_limb; j ++) {
            cmul[i-1].inc[j] <== pol[n-i-1][j];
        }
    }

    if (n>1) {
        for (var j = 0; j < n_limb; j ++) {
            out[j] <== cmul[n-2].out[j];
        }
    } else {
        for (var j = 0; j < n_limb; j ++) {
            out[j] <== pol[n-1][j];
        }
    }
}
