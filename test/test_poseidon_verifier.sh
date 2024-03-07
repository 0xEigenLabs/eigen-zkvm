# EigenZKit
set -ex

cargo build --release

CIRCOM_FILE=stark_verify
CUR_DIR=$(cd $(dirname $0);pwd)
POWER=25
export RUST_BACKTRACE=1
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
WORKSPACE=../starkjs/poseidon/circuits
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

SRS=${CUR_DIR}/../keys/setup_2^${POWER}.key
if [ ! -f $SRS ]; then
    ${ZKIT} setup -p ${POWER} -s ${SRS}
fi

echo "npm run poseidon "
cd "../starkjs"
npm run poseidon 

echo "========== 0. generate stark proof and then generate the stark verifier circom ============"
${ZKIT} stark_prove -s ../starky/data/starkStruct.json.gl \
    -p ./poseidon/build/poseidon_test.pil.json \
    --o ./poseidon/build/poseidon_test.const \
    --m ./poseidon/build/poseidon_test.cm \
    -c ./poseidon/circuits/stark_verify.circom \
    --i ./poseidon/circuits/stark_proof.json \
    --norm_stage
# cd eigen-zkevm/test 
cd $CUR_DIR

echo "=========== 1. Compile the circuit =================="
${ZKIT} compile \
    -p goldilocks \
    -i ../starkjs/poseidon/circuits/$CIRCOM_FILE.circom \
    -l "../starkjs/node_modules/pil-stark/circuits.gl" \
    -l "../starkjs/node_modules/circomlib/circuits" \
    --O2=full \
    -o $WORKSPACE

echo "===========2. Generate witness ==========="
${ZKIT} calculate_witness -w ${WORKSPACE}/${CIRCOM_FILE}_js/$CIRCOM_FILE.wasm -i ${WORKSPACE}/stark_proof.json -o $WORKSPACE/witness.wtns

echo "===========3. Export verification key ==========="
${ZKIT} export_verification_key -s ${SRS}  -c $WORKSPACE/$CIRCOM_FILE.r1cs --v $WORKSPACE/vk.bin

echo "===========4. prove ==========="
${ZKIT} prove -c $WORKSPACE/$CIRCOM_FILE.r1cs -w $WORKSPACE/witness.wtns -s ${SRS} --b $WORKSPACE/proof.bin

echo "===========5. Verify the proof ==========="
${ZKIT} verify -p $WORKSPACE/proof.bin -v $WORKSPACE/vk.bin

echo "===========6. Generate verifier ==========="
mkdir -p $WORKSPACE/contracts
${ZKIT} generate_verifier -v $WORKSPACE/vk.bin -s $WORKSPACE/contracts/stark_verifier.sol
