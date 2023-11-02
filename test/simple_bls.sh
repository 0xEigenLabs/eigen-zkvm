#!/bin/bash

cd ../starkjs
CIRCUIT=fib
npm run $CIRCUIT

../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p /tmp/$CIRCUIT.pil.json \
    --o /tmp/$CIRCUIT.const \
    --m /tmp/$CIRCUIT.cm -c circuits/$CIRCUIT.verifier.circom --i circuits/$CIRCUIT.verifier.zkin.json

../target/release/eigen-zkit compile -p goldilocks -i circuits/$CIRCUIT.verifier.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/

# Circom to Stark
#../target/release/eigen-zkit compressor12_setup \
#    --r /tmp/$CIRCUIT.verifier.r1cs \
#    --c /tmp/c12.const \
#    --p /tmp/c12.pil \
#    --e /tmp/c12.exec
#
#../target/release/eigen-zkit compressor12_exec \
#    --w /tmp/$CIRCUIT.verifier_js/$CIRCUIT.verifier.wasm  \
#    --i circuits/$CIRCUIT.verifier.zkin.json  \
#    --p /tmp/c12.pil  \
#    --e /tmp/c12.exec \
#    --m /tmp/c12.cm

../target/release/eigen-zkit compressor12 \
    --r /tmp/$CIRCUIT.verifier.r1cs \
    --w /tmp/$CIRCUIT.verifier_js/$CIRCUIT.verifier.wasm  \
    --i circuits/$CIRCUIT.verifier.zkin.json  \
    --c /tmp/c12.const \
    --m /tmp/c12.cm \
    --p /tmp/c12.pil.json

../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bls12381.json \
    -p /tmp/c12.pil.json \
    --o /tmp/c12.const \
    --m /tmp/c12.cm -c circuits/c12a.verifier.circom --i circuits/c12a.verifier.zkin.json --norm_stage

cd ../test

# FIXME 
CUR_DIR=$(cd $(dirname $0);pwd)
CIRCUIT_NAME=c12a.verifier
WORK_DIR=${CUR_DIR}/aggregation2
mkdir -p $WORK_DIR/$CIRCUIT_NAME
cp ../starkjs/circuits/c12a.verifier.zkin.json $WORK_DIR/final_input.zkin.json
cp ../starkjs/circuits/c12a.verifier.circom $WORK_DIR/

bash -x ./snark_verifier.sh groth16 true BLS12381 $CIRCUIT_NAME $WORK_DIR
