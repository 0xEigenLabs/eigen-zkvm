#!/bin/bash
set -e

## build
cargo build --release

BIG_POWER=26
NUM_PROOF=2
NUM_INPUT=2
CUR_DIR=$(cd $(dirname $0);pwd)
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
CIRCUIT="fibonacci"
PILEXECJS="fibonacci/fibonacci.js"
RUNDIR="${CUR_DIR}/../starkjs"

WORKSPACE=/tmp/aggregation_$CIRCUIT
if [ $1 = "restart" ]; then
    rm -rf $WORKSPACE && mkdir -p $WORKSPACE
fi 

RECURSIVE_CIRCUIT=$CIRCUIT.recursive1
RECURSIVE2_CIRCUIT=$CIRCUIT.recursive2
FINAL_CIRCUIT=$CIRCUIT.final

input0=$CUR_DIR/aggregation/0/${RECURSIVE_CIRCUIT} && mkdir -p $input0
input1=$CUR_DIR/aggregation/1/${RECURSIVE_CIRCUIT} && mkdir -p $input1


# test poseidon
#CIRCUIT="poseidon"
#PILEXECJS="poseidon/main_poseidon.js"

cd ${CUR_DIR} && npm i
for (( i=0; i<$NUM_PROOF; i++ ))
do
    ./recursive_proof_to_snark.sh $i $WORKSPACE $CIRCUIT $PILEXECJS "stark"
done

echo " ==> aggregation stage <== "
if [ ! -f "$WORKSPACE/$RECURSIVE_CIRCUIT.r1cs" ]; then
    echo "1. compile circuit, use task 0 by default"
    ${ZKIT} compile -p goldilocks -i $CUR_DIR/../starkjs/circuits/0/$RECURSIVE_CIRCUIT.circom -l "../starkjs/node_modules/pil-stark/circuits.gl" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE
fi
echo "1.no need compile circom : "$WORKSPACE/$RECURSIVE_CIRCUIT.r1cs" already generated"

echo "2. combine input1.zkin.json with input2.zkin.json "
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

mkdir -p ./aggregation/$RECURSIVE2_CIRCUIT 

echo "5. generate recursive2 proof  "
# generate the stark proof and the circom circuits to verify stark proof.
# input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
# output files :  $RECURSIVE2_CIRCUIT.circom  $RECURSIVE2_CIRCUIT/r2_input.json
# Remark: the N of r2.starkStruct must be 2^20 , because the degree of $RECURSIVE_CIRCUIT.pil is 2^20 which determined by the proocess of converting  $RECURSIVE_CIRCUIT.circom to  $RECURSIVE_CIRCUIT.pil
../target/release/eigen-zkit stark_prove -s ../starky/data/r2.starkStruct.json \
    -p $WORKSPACE/$RECURSIVE_CIRCUIT.pil.json \
    --o $WORKSPACE/$RECURSIVE_CIRCUIT.const \
    --m $WORKSPACE/$RECURSIVE_CIRCUIT.cm -c $RUNDIR/circuits/$RECURSIVE2_CIRCUIT.circom --i ./aggregation/$RECURSIVE2_CIRCUIT/r2_input.zkin.json 



# final recursive stage 
echo " ==> final recursive stage <== "
if [ ! -f "$WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs" ]; then
    echo "1. compile circuit and generate r1cs and wasm"
    ${ZKIT} compile -p goldilocks -i $RUNDIR/circuits/$RECURSIVE2_CIRCUIT.circom -l "../starkjs/node_modules/pil-stark/circuits.gl" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE 
fi
echo "1.no need compile circom : "$WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs" already generated"

echo "2. generate the pil files and  const polynomicals files "
node $RUNDIR/src/compressor12/main_compressor12_setup.js \
    -r $WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs \
    -c $WORKSPACE/$RECURSIVE2_CIRCUIT.const \
    -p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil \
    -e $WORKSPACE/$RECURSIVE2_CIRCUIT.exec 

echo "3. generate the commit polynomicals files  "
node $RUNDIR/src/compressor12/main_compressor12_exec.js \
    -w $WORKSPACE/$RECURSIVE2_CIRCUIT"_js"/$RECURSIVE2_CIRCUIT.wasm  \
    -i ./aggregation/$RECURSIVE2_CIRCUIT/r2_input.zkin.json   \
    -p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil  \
    -e $WORKSPACE/$RECURSIVE2_CIRCUIT.exec \
    -m $WORKSPACE/$RECURSIVE2_CIRCUIT.cm


mkdir -p ./aggregation/$FINAL_CIRCUIT

echo "4. generate final proof  "
# Remark: the N of final.starkStruct must be 2^20 , because the degree of $RECURSIVE2_CIRCUIT.pil is 2^18 which determined by the proocess of converting  $RECURSIVE_CIRCUIT2.circom to  $RECURSIVE_CIRCUIT2.pil
../target/release/eigen-zkit stark_prove -s ../starky/data/final.starkStruct.bn128.json \
    -p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil.json \
    --o $WORKSPACE/$RECURSIVE2_CIRCUIT.const \
    --m $WORKSPACE/$RECURSIVE2_CIRCUIT.cm -c $RUNDIR/circuits/$FINAL_CIRCUIT.circom --i ./aggregation/$FINAL_CIRCUIT/final_input.zkin.json 
