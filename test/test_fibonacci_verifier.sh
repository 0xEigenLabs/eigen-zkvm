# EigenZKit
set -ex

cargo build --release

CIRCUIT=c12.verifier
CUR_DIR=$(cd $(dirname $0);pwd)
POWER=24
export RUST_BACKTRACE=1
ZKIT="${CUR_DIR}/../target/release/zkit"
WORKSPACE=/tmp/${CIRCUIT}
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

SRS=${CUR_DIR}/../keys/setup_2^${POWER}.key
if [ ! -f $SRS ]; then
    ${ZKIT} setup -p ${POWER} -s ${SRS}
fi

cd $CUR_DIR

echo "1. Compile the circuit"
${ZKIT} compile -i ../fri_verifier/circuits/$CIRCUIT.circom -l "../fri_verifier/node_modules/pil-stark/circuits.bn128" -l "../fri_verifier/node_modules/circomlib/circuits" --O2=full -o $WORKSPACE

echo "2. Generate witness"
node ${WORKSPACE}/${CIRCUIT}_js/generate_witness.js ${WORKSPACE}/${CIRCUIT}_js/$CIRCUIT.wasm  ../fri_verifier/circuits/c12.zkin.json $WORKSPACE/witness.wtns

echo "3. Export verification key"
${ZKIT} export_verification_key -s ${SRS}  -c $WORKSPACE/$CIRCUIT.r1cs -v $WORKSPACE/vk.bin

echo "4. prove"
${ZKIT} prove -c $WORKSPACE/$CIRCUIT.r1cs -w $WORKSPACE/witness.wtns -s ${SRS} -b $WORKSPACE/proof.bin

echo "5. Verify the proof"
${ZKIT} verify -p $WORKSPACE/proof.bin -v $WORKSPACE/vk.bin

echo "6. Generate verifier"
mkdir -p ${CIRCUIT}/contracts
${ZKIT} generate_verifier -v $WORKSPACE/vk.bin -s ${CIRCUIT}/contracts/verifier.sol
