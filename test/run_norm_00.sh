WORKSPACE=/tmp
RUNDIR=../starkjs
OutputNamePrefix=fib
PILCACHE=$WORKSPACE/$OutputNamePrefix
PILEXECJS=fibonacci/fibonacci.js

mkdir -p $RUNDIR/circuits && node $RUNDIR/$PILEXECJS -w circuits -i "./inputs/input1.json" --pc $PILCACHE

../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p $WORKSPACE/$OutputNamePrefix.pil.json \
    --o $WORKSPACE/$OutputNamePrefix.const \
    --m $WORKSPACE/$OutputNamePrefix.cm -c $RUNDIR/circuits/$OutputNamePrefix.verifier.circom --i $RUNDIR/circuits/$OutputNamePrefix.verifier.zkin.json


../target/release/eigen-zkit compile -p goldilocks -i $RUNDIR/circuits/$OutputNamePrefix.verifier.circom -l $RUNDIR/node_modules/pil-stark/circuits.gl --O2=full -o $WORKSPACE/


node $RUNDIR/src/compressor12/main_compressor12_setup.js \
    -r $WORKSPACE/$OutputNamePrefix.verifier.r1cs \
    -c $WORKSPACE/c12.const \
    -p $WORKSPACE/c12.pil \
    -e $WORKSPACE/c12.exec

node $RUNDIR/src/compressor12/main_compressor12_exec.js \
    -w $WORKSPACE/$OutputNamePrefix.verifier_js/$OutputNamePrefix.verifier.wasm  \
    -i $RUNDIR/circuits/$OutputNamePrefix.verifier.zkin.json  \
    -p $WORKSPACE/c12.pil  \
    -e $WORKSPACE/c12.exec \
    -m $WORKSPACE/c12.cm

../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
    -p $WORKSPACE/c12.pil.json \
    --o $WORKSPACE/c12.const \
    --m $WORKSPACE/c12.cm -c $RUNDIR/circuits/fibonacci.verifier_0.circom --i ./aggregation/fibonacci/000/input.json --norm_stage 