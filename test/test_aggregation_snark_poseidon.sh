#!/bin/bash
set -e

# build
cargo build --release

BIG_POWER=23

POWER=23
CUR_DIR=$(cd $(dirname $0);pwd)
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
CIRCUIT="poseidon_snark"
PROOFDIT=$CUR_DIR

cd ${CUR_DIR} && npm i 

WORKSPACE=/tmp/aggregation_poseidon_snark
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
${ZKIT} compile -i ../starkjs/poseidon_snark/$CIRCUIT.circom  --O2=full -o $WORKSPACE

echo "2. export verification key"
${ZKIT} export_verification_key -s ${SRS} -c $WORKSPACE/${CIRCUIT}.r1cs --v $WORKSPACE/vk.bin


for((j=0;j<2;j++));
do   
    mkdir -p $CUR_DIR/aggregation/${CIRCUIT}/$j
    node ../starkjs/poseidon_snark/generate_input.js --i1 $j --i2 $((j+100)) -o $CUR_DIR/aggregation/${CIRCUIT}/$j/input.json 
done  


echo "3. generate each proof"
for wtns in `ls $CUR_DIR/aggregation/${CIRCUIT}`
do
    input=$CUR_DIR/aggregation/${CIRCUIT}/$wtns 
    ${ZKIT} calculate_witness -w ${WORKSPACE}/${CIRCUIT}_js/$CIRCUIT.wasm -i ${input}/input.json -o $input/witness.wtns
    ${ZKIT} prove -c $WORKSPACE/${CIRCUIT}.r1cs -w $input/witness.wtns --b $input/proof.bin -s ${SRS} -t rescue
done

echo "4. collect old proof list"
OLD_PROOF_LIST=$WORKSPACE/old_proof_list.txt
> $OLD_PROOF_LIST

i=0
for wtns in `ls $CUR_DIR/aggregation/${CIRCUIT}`
do
    input=${CUR_DIR}/aggregation/${CIRCUIT}/$wtns
    echo $input/proof.bin >> $OLD_PROOF_LIST
    i=$((i+1))
done

cat $OLD_PROOF_LIST

echo "5. export aggregation vk"
${ZKIT} export_aggregation_verification_key --c $i --i 3 -s ${BIG_SRS} --v $WORKSPACE/aggregation_vk.bin

echo "6. generate aggregation proof"
${ZKIT} aggregation_prove -s ${BIG_SRS} --f $OLD_PROOF_LIST  --v $WORKSPACE/vk.bin --n $WORKSPACE/aggregation_proof.bin  --j $WORKSPACE/aggregation_proof.json

echo "7. verify"
${ZKIT} aggregation_verify --p $WORKSPACE/aggregation_proof.bin --v $WORKSPACE/aggregation_vk.bin

echo "8. generate verifier"
${ZKIT} generate_aggregation_verifier -o $WORKSPACE/vk.bin --n $WORKSPACE/aggregation_vk.bin --num_inputs 3 -s aggregation/contracts/verifier.sol

echo "9. run verifier test"
cp  $WORKSPACE/aggregation_proof.json /tmp/aggregation/aggregation_proof.json
cd $CUR_DIR/aggregation && npm install && npx hardhat test 