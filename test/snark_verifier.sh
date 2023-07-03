#!/bin/bash
set -e

CUR_DIR=$(cd $(dirname $0);pwd)

POWER=22
SRS=${CUR_DIR}/../keys/setup_2^${POWER}.ptau

CIRCUIT_NAME=fibonacci.final

WORK_DIR=${CUR_DIR}/aggregation/$CIRCUIT_NAME
# mkdir -p $WORK_DIR

SNARK_CIRCOM=$WORK_DIR/$CIRCUIT_NAME.circom
SNARK_INPUT=$WORK_DIR/final_input.zkin.json 

RUNDIR="${CUR_DIR}/../starkjs"

ZKIT="${CUR_DIR}/../target/release/eigen-zkit"

echo "1. compile circom and generate wasm and r1cs"
# ${ZKIT} compile -p bn128 -i $CUR_DIR/../starkjs/circuits/$CIRCUIT_NAME.circom -l "../starkjs/node_modules/pil-stark/circuits.bn128" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORK_DIR
circom $CUR_DIR/../starkjs/circuits/$CIRCUIT_NAME.circom --wasm --r1cs -p bn128  -l "../starkjs/node_modules/pil-stark/circuits.bn128" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORK_DIR

cp $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm /tmp/aggregation/circuits.wasm

if [ ! -f $SRS ]; then
    echo "downloading powersOfTau28_hez_final_${POWER}.ptau"
    curl https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_${POWER}.ptau -o $SRS
fi

echo "2. groth16 setup"
snarkjs g16s $WORK_DIR/$CIRCUIT_NAME.r1cs $SRS  $WORK_DIR/g16.zkey

# snarkjs wc $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm  $WORK_DIR/r2_input.zkin.json   $WORK_DIR/$CIRCUIT_NAME.wtns

echo "3. groth16 fullprove"
snarkjs g16f $SNARK_INPUT $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm  $WORK_DIR/g16.zkey $WORK_DIR/proof.json $WORK_DIR/public.json

echo "4. generate verification_key"
snarkjs zkev  $WORK_DIR/g16.zkey  $WORK_DIR/verification_key.json

echo "5. verify groth16 proof"
snarkjs g16v $WORK_DIR/verification_key.json $WORK_DIR/public.json $WORK_DIR/proof.json