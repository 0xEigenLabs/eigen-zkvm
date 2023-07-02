#!/bin/bash
set -e

## build
cargo build --release

BIG_POWER=26
POWER=22
NUM_PROOF=2
NUM_INPUT=2
CUR_DIR=$(cd $(dirname $0);pwd)
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
CIRCUIT="fibonacci"
PILEXECJS="fibonacci/fibonacci.js"
RUNDIR="${CUR_DIR}/../starkjs"

# test poseidon
#CIRCUIT="poseidon"
#PILEXECJS="poseidon/main_poseidon.js"

WORKSPACE=/tmp/aggregation_$CIRCUIT
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

cd ${CUR_DIR} && npm i
for (( i=0; i<$NUM_PROOF; i++ ))
do
    ./recursive_proof_to_stark.sh $i $WORKSPACE $CIRCUIT $PILEXECJS 
done


RECURSIVE_CIRCUIT=$CIRCUIT.recursive1
RECURSIVE2_CIRCUIT=$CIRCUIT.recursive2
# echo "1. compile circuit, use task 0 by default"
${ZKIT} compile -i ../starkjs/circuits/0/$RECURSIVE_CIRCUIT.circom -l "../starkjs/node_modules/pil-stark/circuits.gl" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE

echo "2. combine input1.zkin.json with input2.zkin.json "
input0=$CUR_DIR/aggregation/0/${RECURSIVE_CIRCUIT} && mkdir -p $input0
input1=$CUR_DIR/aggregation/1/${RECURSIVE_CIRCUIT} && mkdir -p $input1

node $RUNDIR/src/recursive/main_joinzkin.js  --zkin1 $input0/input.zkin.json --zkin2 $input1/input.zkin.json  --zkinout $input0/r1_input.zkin.json


echo "3. generate the pil files and  const polynomicals files "
# generate the pil files and  const polynomicals files
# input files :  $C12_VERIFIER.r1cs  $C12_VERIFIER.const  $C12_VERIFIER.pil
# output files :  $C12_VERIFIER.exec
node $RUNDIR/src/compressor12/main_compressor12_setup.js \
    -r $WORKSPACE/$RECURSIVE_CIRCUIT.r1cs \
    -c $WORKSPACE/$RECURSIVE_CIRCUIT.const \
    -p $WORKSPACE/$RECURSIVE_CIRCUIT.pil \
    -e $WORKSPACE/$RECURSIVE_CIRCUIT.exec 

echo "4. generate the commit polynomicals files  "
# generate the commit polynomicals files 
# input files :  $CIRCUIT.c12.wasm  $C12_VERIFIER.zkin.json  $C12_VERIFIER.pil /$C12_VERIFIER.exec
# output files :  $C12_VERIFIER.cm
node $RUNDIR/src/compressor12/main_compressor12_exec.js \
    -w $WORKSPACE/$RECURSIVE_CIRCUIT"_js"/$RECURSIVE_CIRCUIT.wasm  \
    -i $input0/r1_input.zkin.json  \
    -p $WORKSPACE/$RECURSIVE_CIRCUIT.pil  \
    -e $WORKSPACE/$RECURSIVE_CIRCUIT.exec \
    -m $WORKSPACE/$RECURSIVE_CIRCUIT.cm

mkdir -p ./aggregation/$RECURSIVE2_CIRCUIT/

# generate the stark proof and the circom circuits to verify stark proof.
# input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
# output files :  $RECURSIVE1_VERIFIER.circom  $RECURSIVE1_VERIFIER/input.json
../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
    -p $WORKSPACE/$RECURSIVE_CIRCUIT.pil.json \
    --o $WORKSPACE/$RECURSIVE_CIRCUIT.const \
    --m $WORKSPACE/$RECURSIVE_CIRCUIT.cm -c $RUNDIR/circuits/$RECURSIVE2_CIRCUIT.circom --i ./aggregation/$RECURSIVE2_CIRCUIT/r2_input.zkin.json 

../target/release/eigen-zkit compile -p bn128 -i $RUNDIR/circuits/$RECURSIVE2_CIRCUIT.circom -l $RUNDIR/node_modules/pil-stark/circuits.bn128 --O2=full -o ./aggregation/$RECURSIVE2_CIRCUIT/

cp ./aggregation/$RECURSIVE2_CIRCUIT/$RECURSIVE2_CIRCUIT.wasm /tmp/aggregation/circuits.wasm

SRS=${CUR_DIR}/../keys/setup_2^${POWER}.ptau
if [ ! -f $SRS ]; then
    echo "download powersOfTau28_hez_final_${POWER}.ptau"
    curl https://hermez.s3-eu-west-1.amazonaws.com/powersOfTau28_hez_final_${POWER}.ptau -o $SRS
fi

snarkjs g16s ./aggregation/$RECURSIVE2_CIRCUIT/$RECURSIVE2_CIRCUIT.r1cs $SRS  /tmp/aggregation/g16.zkey

