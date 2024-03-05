#!/bin/bash
set -ex

export NODE_OPTIONS="--max-old-space-size=163840"

CUR_DIR=$(cd $(dirname $0);pwd)
CURVE="BN128"
CIRCUIT_NAME="circuit"
WORK_DIR=/tmp/$CIRCUIT_NAME
mkdir -p $WORK_DIR
SNARK_CIRCOM=${CUR_DIR}/single/circuit/$CIRCUIT_NAME.circom
SNARK_INPUT=${CUR_DIR}/single/input/circuit.json
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"

echo "1. compile circom and generate wasm and r1cs"
$ZKIT compile -i $SNARK_CIRCOM -p $CURVE  -l "../starkjs/node_modules/pil-stark/circuits.bn128" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORK_DIR

echo "2. groth16 setup"
$ZKIT groth16_setup -c $CURVE --r1cs $WORK_DIR/$CIRCUIT_NAME.r1cs -p $WORK_DIR/g16.zkey -v $WORK_DIR/verification_key.json

echo "3. groth16 fullprove"
$ZKIT groth16_prove -c $CURVE --r1cs $WORK_DIR/$CIRCUIT_NAME.r1cs -w $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm -p $WORK_DIR/g16.zkey -i $SNARK_INPUT --public-input $WORK_DIR/public_input.json --proof $WORK_DIR/proof.json

echo "4. verify groth16 proof"
$ZKIT  groth16_verify -c $CURVE -v $WORK_DIR/verification_key.json --public-input ${CUR_DIR}/single/input/groth16_public_input.json --proof ${CUR_DIR}/single/input/groth16_proof.json

echo "5. generate verifier contract (CURVE: BN128)"
$ZKIT generate_verifier  $WORK_DIR/verification_key.json ${CUR_DIR}/single/contracts/groth16_verifier.sol

echo "6. verifier contract test"
cd single && npx hardhat test test/single.test.ts
