#!/bin/bash
set -e

export NODE_OPTIONS="--max-old-space-size=16384"
source ~/.bashrc 

CUR_DIR=$(cd $(dirname $0);pwd)

POWER=22
BIG_POWER=27
SRS=${CUR_DIR}/../keys/setup_2^${POWER}.ptau
BIG_SRS=${CUR_DIR}/../keys/setup_2^${BIG_POWER}.ptau
BIG_SRS_FINAL=${CUR_DIR}/../keys/setup_2^${BIG_POWER}.ptau

CIRCUIT_NAME=fibonacci.final

WORK_DIR=${CUR_DIR}/aggregation/$CIRCUIT_NAME

SNARK_CIRCOM=$WORK_DIR/$CIRCUIT_NAME.circom
SNARK_INPUT=$WORK_DIR/final_input.zkin.json 

RUNDIR="${CUR_DIR}/../starkjs"

ZKIT="${CUR_DIR}/../target/release/eigen-zkit"

if [ "$2" = "true" ]; then 
    echo "compile circom and generate wasm and r1cs"
    circom $CUR_DIR/../starkjs/circuits/$CIRCUIT_NAME.circom --wasm --r1cs -p bn128  -l "../starkjs/node_modules/pil-stark/circuits.bn128" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORK_DIR
    # cp $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm /tmp/aggregation/circuits.wasm
fi 


if [ "$1" = "groth16" ]; then
    if [ ! -f $SRS ]; then
        echo "downloading powersOfTau28_hez_final_${POWER}.ptau"
        curl https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_${POWER}.ptau -o $SRS
    fi
    
    echo ">>> groth16 scheme <<< "
    if [  "$2" = "true" ]; then
        echo "1. generate groth16 zkey"
        snarkjs g16s $WORK_DIR/$CIRCUIT_NAME.r1cs $SRS  $WORK_DIR/g16.zkey
    else 
        echo "1. groth16 zkey already generated"
    fi

    echo "2. groth16 fullprove"
    snarkjs g16f $SNARK_INPUT $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm  $WORK_DIR/g16.zkey $WORK_DIR/proof.json $WORK_DIR/public.json

    echo "3. generate verification_key"
    snarkjs zkev  $WORK_DIR/g16.zkey  $WORK_DIR/verification_key.json

    echo "4. verify groth16 proof"
    snarkjs g16v $WORK_DIR/verification_key.json $WORK_DIR/public.json $WORK_DIR/proof.json

    cp $WORK_DIR/public.json /tmp/aggregation/final_public.json 
    cp $WORK_DIR/proof.json /tmp/aggregation/final_proof.json

    echo "5. generate verifier contract"
    snarkjs zkesv  $WORK_DIR/g16.zkey  ${CUR_DIR}/aggregation/contracts/final_verifier.sol

    echo "6. calculate verify gas cost"
    cd aggregation && npm install && npx hardhat test test/final.test.ts
else 
    if [ ! -f $SRS ]; then
        echo "downloading powersOfTau28_hez_final_${POWER}.ptau"
        curl https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_${POWER}.ptau -o $BIG_SRS
    fi

    echo ">>> fflonk scheme <<< "
    echo "1. fflonk setup "
    
     if [ ! -f "$WORK_DIR/fflonk.zkey" ]; then
        echo "1. generate groth16 zkey"
        # nohup snarkjs ffs $WORK_DIR/$CIRCUIT_NAME.r1cs  $BIG_SRS $WORK_DIR/fflonk.zkey &
        snarkjs ffs $WORK_DIR/$CIRCUIT_NAME.r1cs  $BIG_SRS $WORK_DIR/fflonk.zkey
    else 
        echo "1. fflonk zkey already generated"
    fi

    echo "2. fflonk fullprove"
    snarkjs ffs $WORK_DIR/$CIRCUIT_NAME.r1cs  $BIG_SRS_FINAL  $WORK_DIR/fflonk.zkey
    snarkjs pkf $SNARK_INPUT $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm  $WORK_DIR/fflonk.zkey $WORK_DIR/proof.fflonk.json $WORK_DIR/public.fflonk.
    echo "3. generate verification_key"
    snarkjs zkev  $WORK_DIR/fflonk.zkey  $WORK_DIR/verification_key.fflonk.json

    echo "4. verify fflonk proof"
    snarkjs ffv $WORK_DIR/verification_key.fflonk.json $WORK_DIR/public.fflonk.json $WORK_DIR/proof.fflonk.json

    cp $WORK_DIR/public.fflonk.json  /tmp/aggregation/final_public.fflonk.json 
    cp $WORK_DIR/proof.fflonk.json /tmp/aggregation/final_proof.fflonk.json

    echo "5. generate verifier contract"
    snarkjs zkesv $WORK_DIR/fflonk.zkey  ${CUR_DIR}/aggregation/contracts/final_verifier_fflonk.sol
fi 