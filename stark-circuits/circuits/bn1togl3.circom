
pragma circom 2.0.6;

include "bitify.circom";

template BN1toGL5() {
    signal input in;
    signal output out[5];

    component n2b = Num2Bits(6 * 64);

    n2b.in <== in;

    component b2n[5];

    for (var i=0; i<5; i++) {
        b2n[i] = Bits2Num(64);
        for (var j=0; j<64; j++) {
            b2n[i].in[j] <== n2b.out[64*i+j];
        }
        out[i] <== b2n[i].out;
    }

}
