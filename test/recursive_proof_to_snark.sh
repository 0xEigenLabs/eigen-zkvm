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
RECURSIVE1_CIRCUIT=${CIRCUIT}".recursive1"
RECURSIVE2_CIRCUIT=${CIRCUIT}.recursive2


##### 0. PIL-STARK Setup Phase

# Compilation With PILCOM
# compile .pil file into .pil.json by pilcom.
# input files :  .pil file
# output files :  .pil.json, .const, .cm
mkdir -p $RUNDIR/circuits && node $RUNDIR/$PILEXECJS -w $RUNDIR/circuits -i $TASK_NO --pc $PILCACHE

#### 1. Generate stark proof
# generate .circom file.
# input files :  .pil json & starkStruct.json.gl
# output files : .circom
../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p $WORKSPACE/$TASK_NO/$CIRCUIT.pil.json \
    --o $WORKSPACE/$TASK_NO/$CIRCUIT.const \
    --m $WORKSPACE/$TASK_NO/$CIRCUIT.cm -c $WORKSPACE/circuits/$TASK_NO/fibonacci.verifier.circom --i $WORKSPACE/circuits/$TASK_NO/fibonacci.zkin.json --skip_main

../target/release/eigen-zkit compile -p goldilocks -i $WORKSPACE/circuits/$TASK_NO/fibonacci.verifier.circom -l $RUNDIR/node_modules/pil-stark/circuits.gl --O2=full -o $WORKSPACE/$TASK_NO

# generate the pil files and  const constant polynomial files
# input files :  $C12_VERIFIER.r1cs
# output files :  $C12_VERIFIER.exec, $C12_VERIFIER.const  $C12_VERIFIER.pil
../target/release/eigen-zkit compressor12_setup \
    --r $WORKSPACE/0/fibonacci.verifier.r1cs \
    --c $WORKSPACE/$C12_VERIFIER.const \
    --p $WORKSPACE/$C12_VERIFIER.pil \
    --e $WORKSPACE/$C12_VERIFIER.exec

# generate the commit polynomials files
# input files :  $CIRCUIT.c12.wasm  $C12_VERIFIER.zkin.json  $C12_VERIFIER.pil  $C12_VERIFIER.exec
# output files :  $C12_VERIFIER.cm(c12a.commit(Figure 24))
../target/release/eigen-zkit compressor12_exec \
    --w $WORKSPACE/$TASK_NO/fibonacci.verifier_js/fibonacci.verifier.wasm  \
    --i $WORKSPACE/circuits/$TASK_NO/fibonacci.zkin.json  \
    --p $WORKSPACE/$C12_VERIFIER.pil  \
    --e $WORKSPACE/$C12_VERIFIER.exec \
    --m $WORKSPACE/$C12_VERIFIER.cm

#### 2. Genrate c12a stark proof
# generate the stark proof and the circom circuits to verify stark proof.
# input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
# output files :  $RECURSIVE1_CIRCUIT.circom  $RECURSIVE1_CIRCUIT/input.json
../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.json \
    -p $WORKSPACE/$C12_VERIFIER.pil.json \
    --o $WORKSPACE/$C12_VERIFIER.const \
    --m $WORKSPACE/$C12_VERIFIER.cm -c $WORKSPACE/circuits/$TASK_NO/$RECURSIVE1_CIRCUIT.circom --i $WORKSPACE/circuits/$TASK_NO/c12a.zkin.json

../target/release/eigen-zkit compile -p goldilocks -i $WORKSPACE/circuits/$TASK_NO/$RECURSIVE1_CIRCUIT.circom -l $RUNDIR/node_modules/pil-stark/circuits.gl --O2=full -o $WORKSPACE/$TASK_NO
# generate the pil files and  const polynomicals files
# input files :  $RECURSIVE1_CIRCUIT.r1cs
# output files :  $RECURSIVE1_CIRCUIT.const  $RECURSIVE1_CIRCUIT.pil  $RECURSIVE1_CIRCUIT.exec
../target/release/eigen-zkit compressor12_setup \
    --r $WORKSPACE/0/$RECURSIVE1_CIRCUIT.r1cs \
    --c $WORKSPACE/$RECURSIVE1_CIRCUIT.const \
    --p $WORKSPACE/$RECURSIVE1_CIRCUIT.pil \
    --e $WORKSPACE/$RECURSIVE1_CIRCUIT.exec \
    --force-n-bits 18

# generate the commit polynomicals files 
# input files :  $RECURSIVE1_CIRCUIT.wasm  $input0/r1_input.zkin.json  $RECURSIVE1_CIRCUIT.pil  $RECURSIVE1_CIRCUIT.exec
# output files :  $RECURSIVE1_CIRCUIT.cm
../target/release/eigen-zkit compressor12_exec \
    --w $WORKSPACE/0/$RECURSIVE1_CIRCUIT"_js"/$RECURSIVE1_CIRCUIT.wasm  \
    --i $WORKSPACE/circuits/$TASK_NO/c12a.zkin.json  \
    --p $WORKSPACE/$RECURSIVE1_CIRCUIT.pil  \
    --e $WORKSPACE/$RECURSIVE1_CIRCUIT.exec \
    --m $WORKSPACE/$RECURSIVE1_CIRCUIT.cm

../target/release/eigen-zkit stark_prove -s ../starky/data/r1.starkStruct.json \
    -p $WORKSPACE/$RECURSIVE1_CIRCUIT.pil.json \
    --o $WORKSPACE/$RECURSIVE1_CIRCUIT.const \
    --m $WORKSPACE/$RECURSIVE1_CIRCUIT.cm -c $WORKSPACE/circuits/$TASK_NO/$RECURSIVE2_CIRCUIT.circom \
    --i $WORKSPACE/aggregation/$TASK_NO/$RECURSIVE1_CIRCUIT.zkin.json --norm_stage --agg_stage

# if [ "$GENERATE_PROOF_TYPE" = "stark" ]; then 
# else 
#     echo "Generate snark proof"
#      # generate the stark proof and the circom circuits to verify stark proof.
#     # input files : $C12_VERIFIER.pil.json(stark proof)  $C12_VERIFIER.const(const polynomials)  $C12_VERIFIER.cm (commit polynomials)
#     # output files :  $RECURSIVE1_CIRCUIT.circom  $RECURSIVE1_CIRCUIT/input.json
#     ../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
#         -p $WORKSPACE/$C12_VERIFIER.pil.json \
#         --o $WORKSPACE/$C12_VERIFIER.const \
#         --m $WORKSPACE/$C12_VERIFIER.cm -c $WORKSPACE/circuits/$RECURSIVE1_CIRCUIT.circom --i $WORKSPACE/aggregation/$RECURSIVE1_CIRCUIT/input.json
# fi 
