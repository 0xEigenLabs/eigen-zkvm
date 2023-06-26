#!/bin/bash
set -ex
CUR_DIR=$(cd $(dirname $0);pwd)
INPUT_NUM=$1
WORKSPACE="$2"
RUNDIR="${CUR_DIR}/../starkjs"
OutputNamePrefix="$INPUT_NUM/$3"
PILCACHE=$WORKSPACE/$OutputNamePrefix
PILEXECJS=$4

mkdir -p $WORKSPACE/$INPUT_NUM
mkdir -p $RUNDIR/circuits/$INPUT_NUM

FIRST_VERIFIER=$OutputNamePrefix".first_verifier"
SECOND_VERIFIER=$3

mkdir -p $RUNDIR/circuits && node $RUNDIR/$PILEXECJS -w $RUNDIR/circuits -i $INPUT_NUM --pc $PILCACHE

../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p $WORKSPACE/$OutputNamePrefix.pil.json \
    --o $WORKSPACE/$OutputNamePrefix.const \
    --m $WORKSPACE/$OutputNamePrefix.cm -c $RUNDIR/circuits/$FIRST_VERIFIER.circom --i $RUNDIR/circuits/$FIRST_VERIFIER.zkin.json


../target/release/eigen-zkit compile -p goldilocks -i $RUNDIR/circuits/$FIRST_VERIFIER.circom -l $RUNDIR/node_modules/pil-stark/circuits.gl --O2=full -o $WORKSPACE/


node $RUNDIR/src/compressor12/main_compressor12_setup.js \
    -r $WORKSPACE/$FIRST_VERIFIER.r1cs \
    -c $WORKSPACE/$FIRST_VERIFIER.const \
    -p $WORKSPACE/$FIRST_VERIFIER.pil \
    -e $WORKSPACE/$FIRST_VERIFIER.exec

node $RUNDIR/src/compressor12/main_compressor12_exec.js \
    -w $WORKSPACE/$FIRST_VERIFIER"_js"/$FIRST_VERIFIER.wasm  \
    -i $RUNDIR/circuits/$FIRST_VERIFIER.zkin.json  \
    -p $WORKSPACE/$FIRST_VERIFIER.pil  \
    -e $WORKSPACE/$FIRST_VERIFIER.exec \
    -m $WORKSPACE/$FIRST_VERIFIER.cm

mkdir -p ./aggregation/$SECOND_VERIFIER/

../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
    -p $WORKSPACE/$FIRST_VERIFIER.pil.json \
    --o $WORKSPACE/$FIRST_VERIFIER.const \
    --m $WORKSPACE/$FIRST_VERIFIER.cm -c $RUNDIR/circuits/$SECOND_VERIFIER.circom --i ./aggregation/$SECOND_VERIFIER/input.json --norm_stage
