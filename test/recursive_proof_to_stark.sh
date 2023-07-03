#!/bin/bash
set -ex
CUR_DIR=$(cd $(dirname $0);pwd)
TASK_NO=$1
WORKSPACE=$2
CIRCUIT=$3
RUNDIR="${CUR_DIR}/../starkjs"
PILCACHE=$WORKSPACE/$TASK_NO/$CIRCUIT
PILEXECJS=$4

mkdir -p $WORKSPACE/$TASK_NO
mkdir -p $RUNDIR/circuits/$TASK_NO

C12_VERIFIER=$TASK_NO/${CIRCUIT}".c12"
RECURSIVE1_VERIFIER=$TASK_NO/${CIRCUIT}".recursive1"

mkdir -p $RUNDIR/circuits && node $RUNDIR/$PILEXECJS -w $RUNDIR/circuits -i $TASK_NO --pc $PILCACHE

../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p $WORKSPACE/$TASK_NO/$CIRCUIT.pil.json \
    --o $WORKSPACE/$TASK_NO/$CIRCUIT.const \
    --m $WORKSPACE/$TASK_NO/$CIRCUIT.cm -c $RUNDIR/circuits/$C12_VERIFIER.circom --i $RUNDIR/circuits/$C12_VERIFIER.zkin.json

../target/release/eigen-zkit compile -p goldilocks -i $RUNDIR/circuits/$C12_VERIFIER.circom -l $RUNDIR/node_modules/pil-stark/circuits.gl --O2=full -o $WORKSPACE/$TASK_NO

# generate the pil files and  const polynomicals files
# input files :  $C12_VERIFIER.r1cs  $C12_VERIFIER.const  $C12_VERIFIER.pil
# output files :  $C12_VERIFIER.exec
node $RUNDIR/src/compressor12/main_compressor12_setup.js \
    -r $WORKSPACE/$C12_VERIFIER.r1cs \
    -c $WORKSPACE/$C12_VERIFIER.const \
    -p $WORKSPACE/$C12_VERIFIER.pil \
    -e $WORKSPACE/$C12_VERIFIER.exec

# generate the commit polynomicals files 
# input files :  $CIRCUIT.c12.wasm  $C12_VERIFIER.zkin.json  $C12_VERIFIER.pil /$C12_VERIFIER.exec
# output files :  $C12_VERIFIER.cm
node $RUNDIR/src/compressor12/main_compressor12_exec.js \
    -w $WORKSPACE/$C12_VERIFIER"_js"/$CIRCUIT.c12.wasm  \
    -i $RUNDIR/circuits/$C12_VERIFIER.zkin.json  \
    -p $WORKSPACE/$C12_VERIFIER.pil  \
    -e $WORKSPACE/$C12_VERIFIER.exec \
    -m $WORKSPACE/$C12_VERIFIER.cm

mkdir -p ./aggregation/$RECURSIVE1_VERIFIER/

# generate the stark proof and the circom circuits to verify stark proof.
# input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
# output files :  $RECURSIVE1_VERIFIER.circom  $RECURSIVE1_VERIFIER/input.json
../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.json \
    -p $WORKSPACE/$C12_VERIFIER.pil.json \
    --o $WORKSPACE/$C12_VERIFIER.const \
    --m $WORKSPACE/$C12_VERIFIER.cm -c $RUNDIR/circuits/$RECURSIVE1_VERIFIER.circom --i ./aggregation/$RECURSIVE1_VERIFIER/input.zkin.json --agg_stage --norm_stage
 