#!/bin/bash
set -ex
CUR_DIR=$(cd $(dirname $0);pwd)
TASK_NO=$1
WORKSPACE=$2
CIRCUIT=$3
RUNDIR="${CUR_DIR}/../starkjs"
PILCACHE=$WORKSPACE/$TASK_NO/$CIRCUIT
PILEXECJS=$4
GENERATE_PROOF_TYPE=$5
WORKSPACE=$6
RUST_LOG=info
#  CIRCUIT="fibonacci"
#  PILEXECJS="fibonacci/fibonacci.js"
#    ./recursive_proof_to_snark.sh $i $WORKSPACE $CIRCUIT $PILEXECJS "stark"

mkdir -p $WORKSPACE/$TASK_NO
mkdir -p $WORKSPACE/circuits/$TASK_NO

C12_VERIFIER=$TASK_NO/${CIRCUIT}".c12"
RECURSIVE1_VERIFIER=$TASK_NO/${CIRCUIT}".recursive1"


##### PIL-STARK Setup Phase

# Compilation With PILCOM
# compile .pil file into .pil.json by pilcom.
# input files :  .pil file
# output files :  .pil.json, .const, .cm
cd $RUNDIR && npm i && cd $CUR_DIR

mkdir -p $RUNDIR/circuits && node $RUNDIR/$PILEXECJS -w $RUNDIR/circuits -i $TASK_NO --pc $PILCACHE

# generate .circom file.
# input files :  .pil json & starkStruct.json.gl
# output files : .circom
../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p $WORKSPACE/$TASK_NO/$CIRCUIT.pil.json \
    --o $WORKSPACE/$TASK_NO/$CIRCUIT.const \
    --m $WORKSPACE/$TASK_NO/$CIRCUIT.cm -c $WORKSPACE/circuits/$C12_VERIFIER.circom --i $WORKSPACE/circuits/$C12_VERIFIER.zkin.json

../target/release/eigen-zkit compile -p goldilocks -i $WORKSPACE/circuits/$C12_VERIFIER.circom -l $RUNDIR/node_modules/pil-stark/circuits.gl --O2=full -o $WORKSPACE/$TASK_NO

# generate the pil files and  const constant polynomial files
# input files :  $C12_VERIFIER.r1cs
# output files :  $C12_VERIFIER.exec, $C12_VERIFIER.const  $C12_VERIFIER.pil
../target/release/eigen-zkit compressor12_setup \
    --r $WORKSPACE/$C12_VERIFIER.r1cs \
    --c $WORKSPACE/$C12_VERIFIER.const \
    --p $WORKSPACE/$C12_VERIFIER.pil \
    --e $WORKSPACE/$C12_VERIFIER.exec

# generate the commit polynomials files
# input files :  $CIRCUIT.c12.wasm  $C12_VERIFIER.zkin.json  $C12_VERIFIER.pil  $C12_VERIFIER.exec
# output files :  $C12_VERIFIER.cm
../target/release/eigen-zkit compressor12_exec \
    --w $WORKSPACE/$C12_VERIFIER"_js"/$CIRCUIT.c12.wasm  \
    --i $WORKSPACE/circuits/$C12_VERIFIER.zkin.json  \
    --p $WORKSPACE/$C12_VERIFIER.pil  \
    --e $WORKSPACE/$C12_VERIFIER.exec \
    --m $WORKSPACE/$C12_VERIFIER.cm

mkdir -p $WORKSPACE/aggregation/$RECURSIVE1_VERIFIER/

if [ "$GENERATE_PROOF_TYPE" = "stark" ]; then 
    echo "Generate stark proof"
    # generate the stark proof and the circom circuits to verify stark proof.
    # input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
    # output files :  $RECURSIVE1_VERIFIER.circom  $RECURSIVE1_VERIFIER/input.json
    ../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.json \
        -p $WORKSPACE/$C12_VERIFIER.pil.json \
        --o $WORKSPACE/$C12_VERIFIER.const \
        --m $WORKSPACE/$C12_VERIFIER.cm -c $WORKSPACE/circuits/$RECURSIVE1_VERIFIER.circom --i $WORKSPACE/aggregation/$RECURSIVE1_VERIFIER/input.zkin.json --norm_stage --agg_stage

else 
    echo "Generate snark proof"
     # generate the stark proof and the circom circuits to verify stark proof.
    # input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
    # output files :  $RECURSIVE1_VERIFIER.circom  $RECURSIVE1_VERIFIER/input.json
    ../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
        -p $WORKSPACE/$C12_VERIFIER.pil.json \
        --o $WORKSPACE/$C12_VERIFIER.const \
        --m $WORKSPACE/$C12_VERIFIER.cm -c $WORKSPACE/circuits/$RECURSIVE1_VERIFIER.circom --i $WORKSPACE/aggregation/$RECURSIVE1_VERIFIER/input.json --norm_stage
fi 
