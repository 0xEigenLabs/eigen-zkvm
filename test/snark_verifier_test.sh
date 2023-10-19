#!/bin/bash
set -ex

export NODE_OPTIONS="--max-old-space-size=163840"

CUR_DIR=$(cd $(dirname $0);pwd)
snark_type=${1-groth16}
first_run=${2-false}
#bls12381
CURVE=${3-bn128}
# POWER=22
# if [ $CURVE = "bls12381" ]; then
#     POWER=25
# fi
# BIG_POWER=28
# SRS=${CUR_DIR}/../keys/setup_2^${POWER}.${CURVE}.ptau
# BIG_SRS=${CUR_DIR}/../keys/setup_2^${BIG_POWER}.ptau

CIRCUIT_NAME=$4
WORK_DIR=$5/$CIRCUIT_NAME
mkdir -p $WORK_DIR

SNARK_CIRCOM=$5/$CIRCUIT_NAME.circom
SNARK_INPUT=$5/final_input.zkin.json

RUNDIR="${CUR_DIR}/../starkjs"

SNARKJS=${CUR_DIR}/aggregation/node_modules/snarkjs/build/cli.cjs
if [ ! -d "${CUR_DIR}/aggregation/node_modules/snarkjs" ]; then
    cd ${CUR_DIR}/aggregation && npm install
fi

ZKIT="${CUR_DIR}/../target/release/eigen-zkit"

if [ $first_run = "true" ]; then 
    echo "compile circom and generate wasm and r1cs"
    if [ $CURVE = "bn128" ]; then
        $ZKIT compile -i $SNARK_CIRCOM -p $CURVE  -l "../starkjs/node_modules/pil-stark/circuits.bn128" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORK_DIR
    elif [ $CURVE = "bls12381" ]; then
        $ZKIT compile -i $SNARK_CIRCOM -p $CURVE -l "../stark-circuits/circuits" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORK_DIR
    fi
    # cp $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm /tmp/aggregation/circuits.wasm
fi 


if [ $snark_type = "groth16" ]; then
    # if [ ! -f $SRS ]; then
    #     echo "downloading powersOfTau28_hez_final_${POWER}.ptau"
    #     #curl https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_${POWER}.ptau -o $SRS
    #     $SNARKJS powersoftau new $CURVE ${POWER} /tmp/pot${POWER}_0000.ptau -v
    #     $SNARKJS powersoftau contribute /tmp/pot${POWER}_0000.ptau /tmp/pot${POWER}_0001.ptau --name="First contribution" -v
    #     $SNARKJS powersoftau prepare phase2 /tmp/pot${POWER}_0001.ptau $SRS -v
    # fi

    if [ $first_run = "true" ]; then
        # $SNARKJS g16s $WORK_DIR/$CIRCUIT_NAME.r1cs $SRS $WORK_DIR/g16.zkey
        $ZKIT groth16_setup -c $CURVE --r1cs $WORK_DIR/$CIRCUIT_NAME.r1cs -p $WORK_DIR/g16.zkey -v $WORK_DIR/verification_key.bin
    fi

    echo "2. groth16 fullprove"
    #  $SNARKJS g16f $SNARK_INPUT $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm  $WORK_DIR/g16.zkey $WORK_DIR/proof.json $WORK_DIR/public.json
    $ZKIT groth16_prove -c $CURVE --r1cs $WORK_DIR/$CIRCUIT_NAME.r1cs -w $WORK_DIR/$CIRCUIT_NAME"_js"/$CIRCUIT_NAME.wasm -p $WORK_DIR/g16.zkey -i $SNARK_INPUT --input $WORK_DIR/public_input.bin --proof $WORK_DIR/proof.bin

    if [ $first_run = "true" ]; then
        # echo "3. generate verification_key"
        # $SNARKJS zkev  $WORK_DIR/g16.zkey  $WORK_DIR/verification_key.json

        echo "3. verify groth16 proof"
        # $SNARKJS g16v $WORK_DIR/verification_key.json $WORK_DIR/public.json $WORK_DIR/proof.json
        $ZKIT  groth16_verify -c $CURVE -v $WORK_DIR/verification_key.bin --input $WORK_DIR/public_input.bin --proof $WORK_DIR/proof.bin
        # if [ $CURVE = "bn128" ]; then
        #     echo "5. generate verifier contract"
        #     $SNARKJS zkesv  $WORK_DIR/g16.zkey  ${CUR_DIR}/aggregation/contracts/final_verifier.sol

        #     echo "6. calculate verify gas cost"
        #     cd aggregation && npx hardhat test test/final.test.ts
        # fi
    fi

else 
    if [ $CURVE != "bn128" ]; then
        echo "Not support ${CURVE}"
        exit -1
    fi
    if [ ! -f $BIG_SRS ]; then
        echo "downloading powersOfTau28_hez_final_${POWER}.ptau"
        curl https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final.ptau -o $BIG_SRS
    fi

    echo ">>> fflonk scheme <<< "
    echo "1. fflonk setup "
    
     if [ ! -f "$WORK_DIR/fflonk.zkey" ]; then
        echo "1. generate groth16 zkey"
        # nohup snarkjs ffs $WORK_DIR/$CIRCUIT_NAME.r1cs  $BIG_SRS $WORK_DIR/fflonk.zkey &
        $SNARKJS ffs $WORK_DIR/$CIRCUIT_NAME.r1cs  $BIG_SRS $WORK_DIR/fflonk.zkey
    else 
        echo "1. fflonk zkey already generated"
    fi

    echo "2. fflonk fullprove"
    $SNARKJS fff $SNARK_INPUT $WORK_DIR/${CIRCUIT_NAME}_js/$CIRCUIT_NAME.wasm $WORK_DIR/fflonk.zkey $WORK_DIR/proof.fflonk.json $WORK_DIR/public.fflonk.json

    echo "3. generate verification_key"
    $SNARKJS zkev  $WORK_DIR/fflonk.zkey  $WORK_DIR/verification_key.fflonk.json

    echo "4. verify fflonk proof"
    $SNARKJS ffv $WORK_DIR/verification_key.fflonk.json $WORK_DIR/public.fflonk.json $WORK_DIR/proof.fflonk.json

    echo "5. generate verifier contract"
    $SNARKJS zkesv $WORK_DIR/fflonk.zkey  ${CUR_DIR}/aggregation/contracts/final_verifier_fflonk.sol

    echo "6. calculate verify gas cost"
    cd aggregation && npx hardhat test test/final.fflonk.test.ts
fi 


# ./target/debug/eigen-zkit groth16_setup -c BN128 --r1cs /test/multiplier.r1cs -p  g16.zkey -v verification_key.bin
# ./target/debug/eigen-zkit groth16_prove -c BN128 --r1cs /test/multiplier.r1cs -w /test/multiplier.wasm -p g16.zkey -i /test/multiplier.input.json --input public_input.bin --proof proof.bin
# ./target/debug/eigen-zkit groth16_verify -c BN128 -v verification_key.bin --input /public_input.bin --proof /proof.bin