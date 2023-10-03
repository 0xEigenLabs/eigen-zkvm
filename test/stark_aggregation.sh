#!/bin/zsh
set -ex

## build
cargo build --release

export NODE_OPTIONS="--max-old-space-size=81920"
source ~/.zshrc

BIG_POWER=26
NUM_PROOF=2
NUM_INPUT=2
CUR_DIR=$(cd $(dirname $0);pwd)
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
CIRCUIT="fibonacci"
PILEXECJS="fibonacci/fibonacci.js"
RUNDIR="${CUR_DIR}/../starkjs"

first_run=${1-no}
CURVE=${2-bn128}
WORKSPACE=/tmp/aggregation_${CURVE}_$CIRCUIT
if [ $first_run = "yes" ]; then
    rm -rf $WORKSPACE && mkdir -p $WORKSPACE
fi

RECURSIVE_CIRCUIT=$CIRCUIT.recursive1
RECURSIVE2_CIRCUIT=$CIRCUIT.recursive2
FINAL_CIRCUIT=$CIRCUIT.final

input0=$WORKSPACE/aggregation/0/${RECURSIVE_CIRCUIT} && mkdir -p $input0
input1=$WORKSPACE/aggregation/1/${RECURSIVE_CIRCUIT} && mkdir -p $input1

mkdir -p $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT
mkdir -p $WORKSPACE/aggregation/$FINAL_CIRCUIT

# test poseidon
#CIRCUIT="poseidon"
#PILEXECJS="poseidon/main_poseidon.js"

c12_start=$(date +%s)
cd ${CUR_DIR} && npm i
for (( i=0; i<$NUM_PROOF; i++ ))
do
    ./recursive_proof_to_snark.sh $i $WORKSPACE $CIRCUIT $PILEXECJS "stark" $WORKSPACE
done
c12_end=$(date +%s)


aggregation_start=$(date +%s)

echo " ==> aggregation stage <== "
if [ ! -f "$WORKSPACE/$RECURSIVE_CIRCUIT.r1cs" ]; then
    echo "1. compile circuit, use task 0 by default"
    ${ZKIT} compile -p goldilocks -i $CUR_DIR/../starkjs/circuits/0/$RECURSIVE_CIRCUIT.circom -l "../starkjs/node_modules/pil-stark/circuits.gl" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE
else
    echo "1.no need compile circom : "$WORKSPACE/$RECURSIVE_CIRCUIT.r1cs" already generated"
fi


echo "2. combine input1.zkin.json with input2.zkin.json "
${ZKIT} join_zkin --zkin1 $input0/input.zkin.json --zkin2 $input1/input.zkin.json  --zkinout $input0/r1_input.zkin.json


echo "3. generate the pil files and const polynomicals files "
# generate the pil files and  const polynomicals files
# input files :  $C12_VERIFIER.r1cs  $C12_VERIFIER.const  $C12_VERIFIER.pil
# output files :  $C12_VERIFIER.exec
if [ ! -f "$WORKSPACE/$RECURSIVE_CIRCUIT.pil" ]; then
    ${ZKIT} compressor12_setup  \
        --r $WORKSPACE/$RECURSIVE_CIRCUIT.r1cs \
        --c $WORKSPACE/$RECURSIVE_CIRCUIT.const \
        --p $WORKSPACE/$RECURSIVE_CIRCUIT.pil \
        --e $WORKSPACE/$RECURSIVE_CIRCUIT.exec
fi

echo "4. generate the commit polynomicals files  "
# generate the commit polynomicals files 
# input files :  $CIRCUIT.c12.wasm  $C12_VERIFIER.zkin.json  $C12_VERIFIER.pil  $C12_VERIFIER.exec
# output files :  $C12_VERIFIER.cm
${ZKIT} compressor12_exec \
    --w $WORKSPACE/$RECURSIVE_CIRCUIT"_js"/$RECURSIVE_CIRCUIT.wasm  \
    --i $input0/r1_input.zkin.json  \
    --p $WORKSPACE/$RECURSIVE_CIRCUIT.pil  \
    --e $WORKSPACE/$RECURSIVE_CIRCUIT.exec \
    --m $WORKSPACE/$RECURSIVE_CIRCUIT.cm

echo "5. generate recursive2 proof"
# generate the stark proof and the circom circuits to verify stark proof.
# input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
# output files :  $RECURSIVE2_CIRCUIT.circom  $RECURSIVE2_CIRCUIT/r2_input.json
# Remark: the N of r2.starkStruct must be 2^20 , because the degree of $RECURSIVE_CIRCUIT.pil is 2^20 which determined by the proocess of converting  $RECURSIVE_CIRCUIT.circom to  $RECURSIVE_CIRCUIT.pil
$ZKIT stark_prove -s ../starky/data/r2.starkStruct.json \
    -p $WORKSPACE/$RECURSIVE_CIRCUIT.pil.json \
    --o $WORKSPACE/$RECURSIVE_CIRCUIT.const \
    --m $WORKSPACE/$RECURSIVE_CIRCUIT.cm -c $RUNDIR/circuits/$RECURSIVE2_CIRCUIT.circom --i $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT/r2_input.zkin.json  --norm_stage

aggregation_end=$(date +%s)

final_start=$(date +%s)
# final recursive stage 
echo " ==> final recursive stage <== "
if [ ! -f "$WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs" ]; then
    echo "1. compile circuit and generate r1cs and wasm"
    ${ZKIT} compile -p goldilocks -i $RUNDIR/circuits/$RECURSIVE2_CIRCUIT.circom -l "../starkjs/node_modules/pil-stark/circuits.gl" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE 
else
    echo "1.no need compile circom : "$WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs" already generated"
fi


echo "2. generate the pil files and  const polynomicals files "
if [ ! -f "$WORKSPACE/$RECURSIVE2_CIRCUIT.pil" ]; then
    ${ZKIT} compressor12_setup \
        --r $WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs \
        --c $WORKSPACE/$RECURSIVE2_CIRCUIT.const \
        --p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil \
        --e $WORKSPACE/$RECURSIVE2_CIRCUIT.exec
fi

echo "3. generate the commit polynomicals files "
${ZKIT} compressor12_exec \
    --w $WORKSPACE/$RECURSIVE2_CIRCUIT"_js"/$RECURSIVE2_CIRCUIT.wasm  \
    --i $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT/r2_input.zkin.json   \
    --p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil  \
    --e $WORKSPACE/$RECURSIVE2_CIRCUIT.exec \
    --m $WORKSPACE/$RECURSIVE2_CIRCUIT.cm


echo "4. generate final proof  "
# Remark: the N of final.starkStruct must be 2^20 , because the degree of $RECURSIVE2_CIRCUIT.pil is 2^20 which determined by the proocess of converting  $RECURSIVE_CIRCUIT2.circom to  $RECURSIVE_CIRCUIT2.pil
STARK_STRUCT=$CUR_DIR/../starky/data/final.starkStruct.bls12381.json
if [ $CURVE = "bn128" ]; then
    STARK_STRUCT=$CUR_DIR/../starky/data/final.starkStruct.bn128.json
fi
$ZKIT stark_prove -s $STARK_STRUCT \
    -p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil.json \
    --o $WORKSPACE/$RECURSIVE2_CIRCUIT.const \
    --m $WORKSPACE/$RECURSIVE2_CIRCUIT.cm -c $RUNDIR/circuits/$FINAL_CIRCUIT.circom --i $WORKSPACE/aggregation/$FINAL_CIRCUIT/final_input.zkin.json  --norm_stage

final_end=$(date +%s)

snark_start=$(date +%s)

WORK_DIR=${WORKSPACE}/aggregation
if [ $first_run = "yes" ]; then
    $CUR_DIR/snark_verifier.sh groth16 true $CURVE $FINAL_CIRCUIT $WORK_DIR
else
    $CUR_DIR/snark_verifier.sh groth16 false $CURVE $FINAL_CIRCUIT $WORK_DIR
fi

snark_end=$(date +%s)

echo "C12 Stage Time Cost ($((c12_end - c12_start))s)"
echo "Aggregation Stage Time Cost ($((aggregation_end - aggregation_start))s)"
echo "Final Stage Time Cost ($((final_end - final_start))s)"
echo "Recursive Snark Stage Time Cost ($((snark_end - snark_start))s)"
echo "Full Process Time Cost ($((snark_end - c12_start))s)"
