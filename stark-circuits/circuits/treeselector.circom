pragma circom 2.0.6;

template parallel TreeSelector(nLevels, eSize) {

    var n = 1 << nLevels;
    signal input values[n][eSize];
    signal input key[nLevels];
    signal output out[eSize];

    signal im[n-1][eSize];

    var levelN = n\2;
    var o = 0;
    var lo = 0;
    for (var i=0; i<nLevels; i++) {
        for (var j=0; j<levelN; j++) {
            for (var k=0; k<eSize; k++) {
                if (i==0) {
                    im[o+j][k] <== key[i]*(values[2*j+1][k]  - values[2*j][k])  + values[2*j][k];
                } else {
                    im[o+j][k] <== key[i]*(im[lo + 2*j+1][k] - im[lo + 2*j][k]) + im[lo + 2*j][k];
                }
            }
        }
        lo = o;
        o = o + levelN;
        levelN = levelN\2;
    }

    for (var k=0; k<eSize; k++) {
        out[k] <== im[n-2][k];
    }

}

