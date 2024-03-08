# EigenZKit
set -ex

cargo build --release

CIRCUIT=${1:-circuit}
CUR_DIR=$(cd $(dirname $0); pwd)

if [ "$CIRCUIT" = "circuit" ]; then
    POWER=24
elif [ "$CIRCUIT" = "poseidon" ]; then
    POWER=25
else
    echo "Error: Unsupported CIRCUIT value '$CIRCUIT'"
    exit 1
fi

export RUST_BACKTRACE=1
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
WORKSPACE=/tmp/${CIRCUIT}
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

SRS=${CUR_DIR}/../keys/setup_2^${POWER}.key
if [ ! -f $SRS ]; then
    ${ZKIT} setup -p ${POWER} -s ${SRS}
fi

cd $CUR_DIR

echo "1. Compile the circuit"
# BN128
if [ "$CIRCUIT" = "circuit" ]; then
    CIRCOM_FILE=single/circuit/$CIRCUIT.circom
    INPUT_FILE=single/input/${CIRCUIT}.json
elif [ "$CIRCUIT" = "poseidon" ]; then
    cd "../starkjs"
    npm run poseidon
    echo "========== 0. generate stark proof and then generate the stark verifier circom ============"
    ${ZKIT} stark_prove -s ../starky/data/starkStruct.json \
        -p ./poseidon/build/poseidon_test.pil.json \
        --o ./poseidon/build/poseidon_test.const \
        --m ./poseidon/build/poseidon_test.cm \
        -c ./poseidon/circuits/stark_verify.circom \
        --i ./poseidon/circuits/stark_proof.json \
        --norm_stage
    cd $CUR_DIR
    CIRCOM_FILE=../starkjs/poseidon/circuits/stark_verify.circom
    INPUT_FILE=../starkjs/poseidon/circuits/stark_proof.json
    CIRCUIT=stark_verify
fi
${ZKIT} compile -i ${CIRCOM_FILE} -l "../starkjs/node_modules/pil-stark/circuits.bn128" -l "../starkjs/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE

echo "2. Generate witness"
${ZKIT} calculate_witness -w ${WORKSPACE}/${CIRCUIT}_js/$CIRCUIT.wasm -i ${INPUT_FILE} -o $WORKSPACE/witness.wtns

echo "3. Export verification key"
${ZKIT} export_verification_key -s ${SRS}  -c $WORKSPACE/$CIRCUIT.r1cs --v $WORKSPACE/vk.bin

echo "4. prove"
${ZKIT} prove -c $WORKSPACE/$CIRCUIT.r1cs -w $WORKSPACE/witness.wtns -s ${SRS} --b $WORKSPACE/proof.bin

echo "5. Verify the proof"
${ZKIT} verify -p $WORKSPACE/proof.bin -v $WORKSPACE/vk.bin

echo "6. Generate verifier"
mkdir -p ${WORKSPACE}/contracts
${ZKIT} generate_verifier -v $WORKSPACE/vk.bin -s ${WORKSPACE}/contracts/verifier.sol
