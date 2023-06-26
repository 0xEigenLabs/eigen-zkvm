pragma circom 2.0.6;
include "../node_modules/circomlib/circuits/poseidon.circom";
template test_poseidon(){
    signal input a;
    signal input b;
    signal output c;
    component poseidon;
    poseidon = PoseidonEx(2, 1);
    poseidon.inputs[0] <== a;
    poseidon.inputs[1] <== b;
    poseidon.initialState <== 0;

    c <== poseidon.out[0];

}
component main{public[a,b]} = test_poseidon();