
pragma circom 2.0.6;

include "bitify.circom";

template BN1toGL3(n_limb) {
    signal input in;
    signal output out[n_limb];

    component n2b = Num2Bits((n_limb+1) * 64);

    n2b.in <== in;

    component b2n[n_limb];

    for (var i=0; i<n_limb; i++) {
        b2n[i] = Bits2Num(64);
        for (var j=0; j<64; j++) {
            b2n[i].in[j] <== n2b.out[64*i+j];
        }
        out[i] <== b2n[i].out;
    }

}
