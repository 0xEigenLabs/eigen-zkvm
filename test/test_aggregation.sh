#!/bin/bash
set -ex

## build
cargo build --release

BIG_POWER=23
POWER=12

## number of private input: https://github.com/0xEigenLabs/eigen-zkvm/issues/49
NUM_INPUTS=4

CUR_DIR=$(cd $(dirname $0);pwd)
ZKIT="${CUR_DIR}/../target/release/zkit"
CIRCUIT="circuit"

cd ${CUR_DIR} && npm i 

WORKSPACE=/tmp/aggregation
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

SRS=${CUR_DIR}/../keys/setup_2^${POWER}.key
BIG_SRS=${CUR_DIR}/../keys/setup_2^${BIG_POWER}.key

if [ ! -f $SRS ]; then
#   curl https://universal-setup.ams3.digitaloceanspaces.com/setup_2^${POWER}.key -o $SRS
    ${ZKIT} setup -p ${POWER} -s ${SRS}
fi

if [ ! -f $BIG_SRS ]; then
#   curl https://universal-setup.ams3.digitaloceanspaces.com/setup_2^${BIG_POWER}.key -o $BIG_SRS
    ${ZKIT} setup -p ${BIG_POWER} -s ${BIG_SRS}
fi

echo "1. compile circuit"
${ZKIT} compile -i ${CIRCUIT}.circom --O2=full -o $WORKSPACE

echo "2. export verification key"
${ZKIT} export_verification_key -s ${SRS} -c $WORKSPACE/${CIRCUIT}.r1cs -v $WORKSPACE/vk.bin

echo "3. generate each proof"
for wtns in `ls $CUR_DIR/aggregation/input`
do
    input=$CUR_DIR/aggregation/input/$wtns
    node ${WORKSPACE}/${CIRCUIT}_js/generate_witness.js \
        ${WORKSPACE}/${CIRCUIT}_js/${CIRCUIT}.wasm \
        $input/input.json $input/witness.wtns
    ${ZKIT} prove -c $WORKSPACE/${CIRCUIT}.r1cs -w $input/witness.wtns -b $input/proof.bin -s ${SRS} -j $input/proof.json -t rescue
    ${ZKIT} verify -p $input/proof.bin -v $WORKSPACE/vk.bin -t rescue
done

echo "4. collect old proof list"
OLD_PROOF_LIST=$WORKSPACE/old_proof_list.txt
> $OLD_PROOF_LIST

i=0
for wtns in `ls $CUR_DIR/aggregation/input`
do
    input=${CUR_DIR}/aggregation/input/$wtns
    echo $input/proof.bin >> $OLD_PROOF_LIST
    i=$((i+1))
done

cat $OLD_PROOF_LIST

echo "5. export aggregation vk"
${ZKIT} export_aggregation_verification_key -c $i -i ${NUM_INPUTS} -s ${BIG_SRS} -v $WORKSPACE/aggregation_vk.bin

echo "6. generate aggregation proof"
${ZKIT} aggregation_prove -s ${BIG_SRS} -f $OLD_PROOF_LIST  -v $WORKSPACE/vk.bin -n $WORKSPACE/aggregation_proof.bin  -j $WORKSPACE/aggregation_proof.json

echo "7. verify"
${ZKIT} aggregation_verify -p $WORKSPACE/aggregation_proof.bin -v $WORKSPACE/aggregation_vk.bin

echo "8. generate verifier"
${ZKIT} generate_aggregation_verifier -o $WORKSPACE/vk.bin -n $WORKSPACE/aggregation_vk.bin -i ${NUM_INPUTS} -s aggregation/contracts/verifier.sol

echo "9. run verifier test"
cd $CUR_DIR/aggregation && npm run test
