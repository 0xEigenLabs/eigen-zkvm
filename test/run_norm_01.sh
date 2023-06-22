
WORKSPACE=/tmp
RUNDIR=../starkjs

mkdir -p $RUNDIR/circuits && node $RUNDIR/fibonacci/fibonacci.js -w circuits -i "./inputs/input2.json" 

../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p $WORKSPACE/fib.pil.json \
    --o $WORKSPACE/fib.const \
    --m $WORKSPACE/fib.cm -c $RUNDIR/circuits/fib.verifier.circom --i $RUNDIR/circuits/fib.verifier.zkin.json


../target/release/eigen-zkit compile -p goldilocks -i $RUNDIR/circuits/fib.verifier.circom -l $RUNDIR/node_modules/pil-stark/circuits.gl --O2=full -o $WORKSPACE/


node $RUNDIR/src/compressor12/main_compressor12_setup.js \
    -r $WORKSPACE/fib.verifier.r1cs \
    -c $WORKSPACE/c12.const \
    -p $WORKSPACE/c12.pil \
    -e $WORKSPACE/c12.exec

node $RUNDIR/src/compressor12/main_compressor12_exec.js \
    -w $WORKSPACE/fib.verifier_js/fib.verifier.wasm  \
    -i $RUNDIR/circuits/fib.verifier.zkin.json  \
    -p $WORKSPACE/c12.pil  \
    -e $WORKSPACE/c12.exec \
    -m $WORKSPACE/c12.cm

../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
    -p $WORKSPACE/c12.pil.json \
    --o $WORKSPACE/c12.const \
    --m $WORKSPACE/c12.cm -c $RUNDIR/circuits/fibonacci.circom --i ./aggregation/fibonacci/001/input.json --norm_stage 