pragma circom 2.0.0;
include "./node_modules/circomlib/circuits/poseidon.circom";

template Circuit() {
    signal input foo;
    signal input bar;
    signal input pi;
    signal output out;

    component hasher = Poseidon(3);
    hasher.inputs[0] <== foo;
    hasher.inputs[1] <== bar;
    hasher.inputs[2] <== pi;
    out <== hasher.out;
}

component main { public [ foo, bar ] } = Circuit();
