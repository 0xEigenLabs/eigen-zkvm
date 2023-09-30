pragma circom 2.0.6;

include "bitify.circom";

// out = remainder of the (in + 16*p) by p
template GLNorm() {
    signal input in;
    signal output out;

    var p=0xFFFFFFFF00000001;
    signal k <-- (in + 16*p)\p;
    out <-- (in+16*p) - k*p;

    component n2bK = Num2Bits(10);
    component n2bO = Num2Bits(64);

    n2bK.in <== k;
    n2bO.in <== out;

    (in+16*p) === k*p + out;
}

template GLCNorm() {
    signal input in[3];
    signal output out[3];

    signal k[3];
    component n2bK[3];
    component n2bO[3];

    var p=0xFFFFFFFF00000001;

    for (var i=0; i<3; i++) {
        k[i] <-- (in[i]+16*p)\p;
        out[i] <-- (in[i]+16*p) - k[i]*p;
        n2bK[i] = Num2Bits(10);
        n2bO[i] = Num2Bits(64);
        n2bK[i].in <== k[i];
        n2bO[i].in <== out[i];
        in[i]+16*p === k[i]*p + out[i];
    }
}

template GLMul() {
    signal input ina;
    signal input inb;
    signal output out;

    var p=0xFFFFFFFF00000001;
    signal k;
    signal m;

    m <== (ina+16*p)*(inb+16*p);

    k <-- m\p;
    out <-- m-k*p;

    component n2bK = Num2Bits(80);
    component n2bO = Num2Bits(64);

    n2bK.in <== k;
    n2bO.in <== out;

    m === k*p + out;
}

template GLMulAdd() {
    signal input ina;
    signal input inb;
    signal input inc;
    signal output out;

    var p=0xFFFFFFFF00000001;
    signal k;
    signal m;

    m <== (ina + 16*p)*(inb + 16*p) + inc;

    k <-- m\p;
    out <-- m-k*p;

    component n2bK = Num2Bits(80);
    component n2bO = Num2Bits(64);

    n2bK.in <== k;
    n2bO.in <== out;

    m === k*p + out;
}


template GLCMul() {
    signal input ina[3];
    signal input inb[3];
    signal output out[3];

    var p=0xFFFFFFFF00000001;

    signal A,B,C,D,E,F,G;
    signal m[3];

    A <== ((ina[0]+16*p) + (ina[1]+16*p))  * ((inb[0]+16*p) + (inb[1]+16*p));
    B <== ((ina[0]+16*p) + (ina[2]+16*p))  * ((inb[0]+16*p) + (inb[2]+16*p));
    C <== ((ina[1]+16*p) + (ina[2]+16*p))  * ((inb[1]+16*p) + (inb[2]+16*p));
    D <== (ina[0]+16*p) * (inb[0]+16*p);
    E <== (ina[1]+16*p) * (inb[1]+16*p);
    F <== (ina[2]+16*p) * (inb[2]+16*p);
    G <== D-E;
    m[0] <== C+G-F;
    m[1] <== A+C-E-E-D;
    m[2] <== B-G;

    signal k[3];

    k[0] <-- m[0] \ p;
    k[1] <-- m[1] \ p;
    k[2] <-- m[2] \ p;

    out[0] <-- m[0] -k[0]*p;
    out[1] <-- m[1] -k[1]*p;
    out[2] <-- m[2] -k[2]*p;

    component n2bK0 = Num2Bits(80);
    component n2bK1 = Num2Bits(80);
    component n2bK2 = Num2Bits(80);

    component n2bO0 = Num2Bits(64);
    component n2bO1 = Num2Bits(64);
    component n2bO2 = Num2Bits(64);

    n2bK0.in <== k[0];
    n2bK1.in <== k[1];
    n2bK2.in <== k[2];

    n2bO0.in <== out[0];
    n2bO1.in <== out[1];
    n2bO2.in <== out[2];

    m[0]  === k[0]*p + out[0];
    m[1]  === k[1]*p + out[1];
    m[2]  === k[2]*p + out[2];

}


template GLCMulAdd() {
    signal input ina[3];
    signal input inb[3];
    signal input inc[3];
    signal output out[3];

    var p=0xFFFFFFFF00000001;

    signal A,B,C,D,E,F,G;
    signal m[3];

    A <== ((ina[0]+16*p) + (ina[1]+16*p))  * ((inb[0]+16*p) + (inb[1]+16*p));
    B <== ((ina[0]+16*p) + (ina[2]+16*p))  * ((inb[0]+16*p) + (inb[2]+16*p));
    C <== ((ina[1]+16*p) + (ina[2]+16*p))  * ((inb[1]+16*p) + (inb[2]+16*p));
    D <== (ina[0]+16*p) * (inb[0]+16*p);
    E <== (ina[1]+16*p) * (inb[1]+16*p);
    F <== (ina[2]+16*p) * (inb[2]+16*p);
    G <== D-E;
    m[0] <== C+G-F + inc[0]+16*p;
    m[1] <== A+C-E-E-D + inc[1]+16*p;
    m[2] <== B-G + inc[2]+16*p;

    signal k[3];

    k[0] <-- m[0] \ p;
    k[1] <-- m[1] \ p;
    k[2] <-- m[2] \ p;

    out[0] <-- m[0] -k[0]*p;
    out[1] <-- m[1] -k[1]*p;
    out[2] <-- m[2] -k[2]*p;

    component n2bK0 = Num2Bits(80);
    component n2bK1 = Num2Bits(80);
    component n2bK2 = Num2Bits(80);

    component n2bO0 = Num2Bits(64);
    component n2bO1 = Num2Bits(64);
    component n2bO2 = Num2Bits(64);

    n2bK0.in <== k[0];
    n2bK1.in <== k[1];
    n2bK2.in <== k[2];

    n2bO0.in <== out[0];
    n2bO1.in <== out[1];
    n2bO2.in <== out[2];

    m[0]  === k[0]*p + out[0];
    m[1]  === k[1]*p + out[1];
    m[2]  === k[2]*p + out[2];

}


function _inv1(a) {
    assert(a!=0);
    var p = 0xFFFFFFFF00000001;
    var t = 0;
    var r = p;
    var newt = 1;
    var newr = a % p;
    while (newr) {
        var q = r \ newr;
        var aux1 = newt;
        var aux2 = t-q*newt;
        t = aux1;
        newt = aux2;
        aux1 = newr;
        aux2 = r-q*newr;
        r = aux1;
        newr = aux2;
    }
    if (t<0) t += p;
    return t;
}

template GLInv() {
    signal input in;
    signal output out;

    out <-- _inv1(in);

    component check = GLMul();

    check.ina <== in;
    check.inb <== out;

    check.out === 1;

    // Check that the output is 64 bits TODO: May bi it's not required

    component n2bO = Num2Bits(64);

    n2bO.in <== out;

}


template GLCInv() {
    signal input in[3];
    signal output out[3];

    var p = 0xFFFFFFFF00000001;

    var aa = (in[0] * in[0]) % p;
    var ac = (in[0] * in[2]) % p;
    var ba = (in[1] * in[0]) % p;
    var bb = (in[1] * in[1]) % p;
    var bc = (in[1] * in[2]) % p;
    var cc = (in[2] * in[2]) % p;

    var aaa = (aa * in[0]) % p;
    var aac = (aa * in[2]) % p;
    var abc = (ba * in[2]) % p;
    var abb = (ba * in[1]) % p;
    var acc = (ac * in[2]) % p;
    var bbb = (bb * in[1]) % p;
    var bcc = (bc * in[2]) % p;
    var ccc = (cc * in[2]) % p;

    var t = (-aaa -aac-aac +abc+abc+abc + abb - acc - bbb + bcc - ccc);
    while (t<0) t = t + p;
    t = t % p;
    var tinv = _inv1(t);

    var i1 = (-aa -ac-ac +bc + bb - cc);
    while (i1 <0) i1 = i1 + p;
    i1 = i1*tinv % p;

    var i2 = (ba -cc);
    while (i2<0) i2 = i2 + p;
    i2 = i2*tinv % p;

    var i3 =  (-bb +ac + cc);
    while (i3 <0) i3 = i3 + p;
    i3 = i3*tinv % p;

    out[0] <--  i1;
    out[1] <--  i2;
    out[2] <--  i3;

    component check = GLCMul();
    check.ina[0] <== in[0];
    check.ina[1] <== in[1];
    check.ina[2] <== in[2];
    check.inb[0] <== out[0];
    check.inb[1] <== out[1];
    check.inb[2] <== out[2];
    check.out[0] === 1;
    check.out[1] === 0;
    check.out[2] === 0;

    // Check that the output is 64 bits TODO: May bi it's not required

    component n2bO0 = Num2Bits(64);
    component n2bO1 = Num2Bits(64);
    component n2bO2 = Num2Bits(64);

    n2bO0.in <== out[0];
    n2bO1.in <== out[1];
    n2bO2.in <== out[2];
}
