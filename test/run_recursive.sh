#!/bin/bash

## build

cargo build

CUR_DIR=$(cd $(dirname $0);pwd)
ZKIT="${CUR_DIR}/../target/debug/zkit"

WORKSPACE=/tmp/recursive
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

#SRS=${CUR_DIR}/../keys/setup_2^10.key
SRS=${CUR_DIR}/../setup_2^24.key

echo "1. compile circuit"
${ZKIT} compile -i multiplier.circom --O2=full -o $WORKSPACE

echo "2. generate witness"
for wtns in `ls $CUR_DIR/recursive/input`
do
    input=$CUR_DIR/recursive/input/$wtns
    node ${WORKSPACE}/multiplier_js/generate_witness.js \
        ${WORKSPACE}/multiplier_js/multiplier.wasm \
        $input/input.json $input/witness.wtns
    ${ZKIT} prove -c $WORKSPACE/multiplier.r1cs -w $input/witness.wtns -b $input/proof.bin -s ${SRS}
done

echo "3. collect old proof list"
OLD_PROOF_LIST=$WORKSPACE/old_proof_list.txt
> $OLD_PROOF_LIST

i=0
for wtns in `ls $CUR_DIR/recursive/input`
do
    input=${CUR_DIR}/recursive/input/$wtns
    echo $input/proof.bin >> $OLD_PROOF_LIST
    let "i++"
done

echo "4. export vk"
${ZKIT} export_recursive_verification_key -c $i -i 2 -s ${SRS} -o $WORKSPACE/recursive_vk.bin

echo "5. generate recursive proof"
${ZKIT} recursive_prove -s ${SRS} -f $OLD_PROOF_LIST  -v $WORKSPACE/vk.bin -n $WORKSPACE/recursive_proof.bin  -j $WORKSPACE/proof.json

echo "6. verify"
${ZKIT} recursive_verify -p $WORKSPACE/recursive_proof.bin -v $WORKSPACE/recursive_vk.bin
