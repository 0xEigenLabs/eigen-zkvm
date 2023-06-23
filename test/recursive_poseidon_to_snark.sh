INPUT_NUM=$1
echo "input: $INPUT_NUM"
WORKSPACE="/tmp"
RUNDIR="../starkjs"
OutputNamePrefix="poseidon"
PILCACHE=$WORKSPACE/$OutputNamePrefix
PILEXECJS="poseidon/main_poseidon.js"

FIRST_VERIFIER=$OutputNamePrefix".first_verifier"
SECOND_VERIFIER=$OutputNamePrefix".second_verifier"

mkdir -p $RUNDIR/circuits && node $RUNDIR/$PILEXECJS -w circuits -i $INPUT_NUM --pc $PILCACHE

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

mkdir -p ./aggregation/$SECOND_VERIFIER/$INPUT_NUM

../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
    -p $WORKSPACE/$FIRST_VERIFIER.pil.json \
    --o $WORKSPACE/$FIRST_VERIFIER.const \
    --m $WORKSPACE/$FIRST_VERIFIER.cm -c $RUNDIR/circuits/$SECOND_VERIFIER.circom --i ./aggregation/$SECOND_VERIFIER/$INPUT_NUM/input.json --norm_stage 