#!/bin/bash
set -ex

CUR_DIR=$(cd $(dirname $0);pwd)
cd "$CUR_DIR/../zkit"
# build
if [ "x${USE_AVX2}" = "xyes" ]; then
    # build with avx2 feature
    RUSTFLAGS="-C target-feature=+avx2" cargo build --release --features  profiler
elif [ "x${USE_AVX512}" = "xyes" ]; then
    # build with avx512 feature
    RUSTFLAGS='-C target-feature=+avx512f,+avx512bw,+avx512cd,+avx512dq,+avx512vl' cargo build --features avx512 --features profiler --release
else
    cargo build --release --features profiler
fi
cd "$CUR_DIR"

export NODE_OPTIONS="--max-old-space-size=81920"

BIG_POWER=26
NUM_PROOF=4
NUM_INPUT=2
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
CIRCUIT="fibonacci"
PILEXECJS="fibonacci/fibonacci.js"

first_run=${1-no}
CURVE=${2-BN128}
WORKSPACE=/tmp/aggregation_${CURVE}_$CIRCUIT
if [ $first_run = "yes" ]; then
    rm -rf $WORKSPACE && mkdir -p $WORKSPACE
fi

RECURSIVE1_CIRCUIT=$CIRCUIT.recursive1
RECURSIVE2_CIRCUIT=$CIRCUIT.recursive2
FINAL_CIRCUIT=$CIRCUIT.final
FINAL_CIRCUIT_VERIFIER=$CIRCUIT.final.verifier

input0=$WORKSPACE/aggregation/0 && mkdir -p $input0
input1=$WORKSPACE/aggregation/1 && mkdir -p $input1
input2=$WORKSPACE/aggregation/2 && mkdir -p $input2
input3=$WORKSPACE/aggregation/3 && mkdir -p $input3

mkdir -p $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT
mkdir -p $WORKSPACE/aggregation/$FINAL_CIRCUIT
mkdir -p $WORKSPACE/aggregation/$FINAL_CIRCUIT_VERIFIER

# test poseidon
#CIRCUIT="poseidon"
#PILEXECJS="poseidon/main_poseidon.js"

c12_start=$(date +%s)
cd ${CUR_DIR}/../starkjs && npm i && cd $CUR_DIR

for (( i=0; i<$NUM_PROOF; i++ ))
do
    ./recursive_proof_to_snark.sh $i $WORKSPACE $CIRCUIT $PILEXECJS "stark" $WORKSPACE
done
c12_end=$(date +%s)


aggregation_start=$(date +%s)
echo " ==> aggregation proof 0 and 1 <== "
echo "1. combine input1.zkin.json with input2.zkin.json "
${ZKIT} join_zkin --zkin1 $input0/$RECURSIVE1_CIRCUIT.zkin.json --zkin2 $input1/$RECURSIVE1_CIRCUIT.zkin.json  --zkinout $WORKSPACE/aggregation/r01_input.zkin.json

if [ $first_run = "yes" ]; then
    echo "2. compile circuit and generate r1cs and wasm"
    ${ZKIT} compile -p goldilocks -i $WORKSPACE/circuits/0/$RECURSIVE2_CIRCUIT.circom -l "../starkjs/node_modules/pil-stark/circuits.gl" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE 
else
    echo "2.no need compile circom : "$WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs" already generated"
fi

echo "3. generate the pil files and  const polynomicals files "
if [ $first_run = "yes" ]; then
    ${ZKIT} compressor12_setup \
        --r $WORKSPACE/$RECURSIVE2_CIRCUIT.r1cs \
        --c $WORKSPACE/$RECURSIVE2_CIRCUIT.const \
        --p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil \
        --e $WORKSPACE/$RECURSIVE2_CIRCUIT.exec \
        --force-n-bits 18
fi

echo "4. generate the commit polynomicals files "
${ZKIT} compressor12_exec \
    --w $WORKSPACE/$RECURSIVE2_CIRCUIT"_js"/$RECURSIVE2_CIRCUIT.wasm  \
    --i $WORKSPACE/aggregation/r01_input.zkin.json  \
    --p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil  \
    --e $WORKSPACE/$RECURSIVE2_CIRCUIT.exec \
    --m $WORKSPACE/$RECURSIVE2_CIRCUIT.cm

echo "5. generate recursive2 proof "
${ZKIT} stark_prove -s ../starky/data/r1.starkStruct.json \
    -p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil.json \
    --o $WORKSPACE/$RECURSIVE2_CIRCUIT.const \
    --m $WORKSPACE/$RECURSIVE2_CIRCUIT.cm -c $WORKSPACE/aggregation/$FINAL_CIRCUIT.circom \
    --i $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT"_0".zkin.json  --norm_stage

echo " ==> aggregation proof 01, 2, and 3 <== "
for (( i=0; i<NUM_PROOF-2; i++ ))
do
    suffix=$(printf "_%d" $((i)))
    next_suffix=$(printf "_%d" $((i + 1)))
    input="$WORKSPACE/aggregation/$((i + 2))"

    ${ZKIT} join_zkin \
        --zkin1 $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT$suffix.zkin.json \
        --zkin2 $input/$RECURSIVE1_CIRCUIT.zkin.json \
        --zkinout $WORKSPACE/aggregation/r$suffix"_input".zkin.json

    ${ZKIT} compressor12_exec \
        --w $WORKSPACE/$RECURSIVE2_CIRCUIT"_js"/$RECURSIVE2_CIRCUIT.wasm \
        --i $WORKSPACE/aggregation/r$suffix"_input".zkin.json \
        --p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil \
        --e $WORKSPACE/$RECURSIVE2_CIRCUIT.exec \
        --m $WORKSPACE/$RECURSIVE2_CIRCUIT$suffix.cm

    ${ZKIT} stark_prove \
        -s ../starky/data/r1.starkStruct.json \
        -p $WORKSPACE/$RECURSIVE2_CIRCUIT.pil.json \
        --o $WORKSPACE/$RECURSIVE2_CIRCUIT.const \
        --m $WORKSPACE/$RECURSIVE2_CIRCUIT$suffix.cm \
        -c $WORKSPACE/aggregation/$FINAL_CIRCUIT.circom \
        --i $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT$next_suffix.zkin.json --norm_stage
done

aggregation_end=$(date +%s)

final_start=$(date +%s)
# final recursive stage 
echo " ==> final recursive stage <== "

if [ $first_run = "yes" ]; then
    echo "1. compile circuit and generate r1cs and wasm"
    ${ZKIT} compile -p goldilocks -i $WORKSPACE/aggregation/$FINAL_CIRCUIT.circom -l "../starkjs/node_modules/pil-stark/circuits.gl" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE 
else
    echo "1.no need compile circom : "$WORKSPACE/aggregation/$FINAL_CIRCUIT.r1cs" already generated"
fi

echo "2. generate the pil files and  const polynomicals files "
if [ $first_run = "yes" ]; then
    ${ZKIT} compressor12_setup \
        --r $WORKSPACE/$FINAL_CIRCUIT.r1cs \
        --c $WORKSPACE/$FINAL_CIRCUIT.const \
        --p $WORKSPACE/$FINAL_CIRCUIT.pil \
        --e $WORKSPACE/$FINAL_CIRCUIT.exec
fi

echo "3. generate the commit polynomicals files "
${ZKIT} compressor12_exec \
    --w $WORKSPACE/$FINAL_CIRCUIT"_js"/$FINAL_CIRCUIT.wasm  \
    --i $WORKSPACE/aggregation/$RECURSIVE2_CIRCUIT"_"$((NUM_PROOF - 2)).zkin.json \
    --p $WORKSPACE/$FINAL_CIRCUIT.pil  \
    --e $WORKSPACE/$FINAL_CIRCUIT.exec \
    --m $WORKSPACE/$FINAL_CIRCUIT.cm

# # Remark: the N of final.starkStruct must be 2^20 , because the degree of $RECURSIVE2_CIRCUIT.pil is 2^20 which determined by the proocess of converting  $RECURSIVE1_CIRCUIT2.circom to  $RECURSIVE1_CIRCUIT2.pil
STARK_STRUCT=$CUR_DIR/../starky/data/final.starkStruct.bls12381.json
if [ $CURVE = "BN128" ]; then
    STARK_STRUCT=$CUR_DIR/../starky/data/final.starkStruct.bn128.json
fi
echo "4. generate recursivef proof "
${ZKIT} stark_prove -s $STARK_STRUCT \
    -p $WORKSPACE/$FINAL_CIRCUIT.pil.json \
    --o $WORKSPACE/$FINAL_CIRCUIT.const \
    --m $WORKSPACE/$FINAL_CIRCUIT.cm -c $WORKSPACE/aggregation/$FINAL_CIRCUIT_VERIFIER.circom \
    --i $WORKSPACE/aggregation/$FINAL_CIRCUIT.zkin.json

final_end=$(date +%s)

snark_start=$(date +%s)

WORK_DIR=${WORKSPACE}/aggregation
if [ $first_run = "yes" ]; then
    $CUR_DIR/snark_verifier.sh groth16 true $CURVE $FINAL_CIRCUIT_VERIFIER $WORK_DIR
else
    $CUR_DIR/snark_verifier.sh groth16 false $CURVE $FINAL_CIRCUIT_VERIFIER $WORK_DIR
fi

snark_end=$(date +%s)

echo "C12 Stage Time Cost ($((c12_end - c12_start))s)"
echo "Aggregation Stage Time Cost ($((aggregation_end - aggregation_start))s)"
echo "Final Stage Time Cost ($((final_end - final_start))s)"
echo "Recursive Snark Stage Time Cost ($((snark_end - snark_start))s)"
echo "Full Process Time Cost ($((snark_end - c12_start))s)"
