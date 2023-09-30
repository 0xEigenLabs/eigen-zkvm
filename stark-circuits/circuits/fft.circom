pragma circom 2.0.6;

include "gl.circom";
include "bitify.circom";

function roots(i) {
    var roots[33] = [
        1,
        18446744069414584320,
        281474976710656,
        18446744069397807105,
        17293822564807737345,
        70368744161280,
        549755813888,
        17870292113338400769,
        13797081185216407910,
        1803076106186727246,
        11353340290879379826,
        455906449640507599,
        17492915097719143606,
        1532612707718625687,
        16207902636198568418,
        17776499369601055404,
        6115771955107415310,
        12380578893860276750,
        9306717745644682924,
        18146160046829613826,
        3511170319078647661,
        17654865857378133588,
        5416168637041100469,
        16905767614792059275,
        9713644485405565297,
        5456943929260765144,
        17096174751763063430,
        1213594585890690845,
        6414415596519834757,
        16116352524544190054,
        9123114210336311365,
        4614640910117430873,
        1753635133440165772
    ];
    return roots[i];
}

template parallel FFT(nBits, inv) {

    var p = 0xFFFFFFFF00000001;
    var N = 1<<nBits;

    signal input in[N][3];
    signal output out[N][3];

    signal k[N][3];

    var w;
    var ws[N];
    if (inv) {
        w = _inv1(roots(nBits));
        ws[0] = _inv1(N);
    } else {
        w = roots(nBits);
        ws[0] = 1;
    }
    for (var i=1; i<N; i++) {
        ws[i] = ( ws[i-1] * w ) % p;
    }

    var sum[N][3];
    for (var i=0; i<N; i++) {
        for (var e=0; e<3; e++) {
            sum[i][e] = 0;
            for (var j=0; j<N; j++) {
                sum[i][e] = sum[i][e] + ws[(i*j)%N]* in[j][e];
            }
        }
    }

    component n2bK[N][3];
    component n2bO[N][3];
    for (var i=0; i<N; i++) {
        for (var e=0; e<3; e++) {
            k[i][e] <-- sum[i][e] \ p;
            out[i][e] <-- sum[i][e] % p;

            k[i][e]*p + out[i][e] === sum[i][e];

            n2bK[i][e] = Num2Bits(64+nBits+1);
            n2bK[i][e].in <== k[i][e];
            n2bO[i][e] = Num2Bits(64);
            n2bO[i][e].in <== out[i][e];
        }
    }
}