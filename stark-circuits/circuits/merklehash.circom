pragma circom 2.0.6;

include "linearhash.circom";
include "merkle.circom";
include "utils.circom";

template parallel MerkleHash(eSize, elementsInLinear, nLinears) {
    var nBits = log2(nLinears);
    assert(1 << nBits == nLinears);
    var nLevels = (nBits - 1)\4 +1;
    signal input values[elementsInLinear][eSize];
    signal input siblings[nLevels][16];
    signal input key[nBits];
    signal output root;

    component linearHash = LinearHash(elementsInLinear, eSize);

    for (var i=0; i<elementsInLinear; i++) {
        for (var e=0; e<eSize; e++) {
            linearHash.in[i][e] <== values[i][e];
        }
    }

    component merkle = Merkle(nBits);

    merkle.value <== linearHash.out;
    for (var i=0; i<nBits; i++) {
        merkle.key[i] <== key[i];
    }
    for (var i=0; i<nLevels; i++) {
        for (var j=0; j<16; j++) {
            merkle.siblings[i][j] <== siblings[i][j];
        }
    }

    root <== merkle.root;
}
