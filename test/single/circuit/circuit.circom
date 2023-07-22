pragma circom 2.0.0;
include "../../node_modules/circomlib/circuits/poseidon.circom";

template Circuit() {
    signal input foo;
    signal input bar;
    signal input pi;
    signal input alpha;
    signal output out;

    component hasher = Poseidon(4);
    hasher.inputs[0] <== foo;
    hasher.inputs[1] <== bar;
    hasher.inputs[2] <== pi;
    hasher.inputs[3] <== alpha;
    out <== hasher.out;
}

component main { public [ bar, foo, pi ] } = Circuit();
