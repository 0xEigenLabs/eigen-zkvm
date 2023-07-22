# EigenZKit
set -ex

cargo build --release --features build

CIRCUIT=${1-circuit} # use circuit or mnist as the first parameter
CUR_DIR=$(cd $(dirname $0);pwd)
POWER=${2-12} # 15 for zkMinist
ZKIT="${CUR_DIR}/../target/release/eigen-zkit"
WORKSPACE=/tmp/single
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

SRS=${CUR_DIR}/../keys/setup_2^${POWER}.key

cd $CUR_DIR

if [ ! -f $SRS ]; then
    ${ZKIT} setup -p ${POWER} -s ${SRS}
fi

echo "1. Compile the circuit"
${ZKIT} compile -i ${CUR_DIR}/single/circuit/${CIRCUIT}.circom --O2=full -o $WORKSPACE

echo "2. Generate witness"
${ZKIT} calculate_witness -i ${CUR_DIR}/single/input/${CIRCUIT}.json -w ${WORKSPACE}/${CIRCUIT}_js/${CIRCUIT}.wasm -o $WORKSPACE/witness.wtns

echo "3. Export verification key"
${ZKIT} export_verification_key -s ${SRS}  -c $WORKSPACE/$CIRCUIT.r1cs --v $WORKSPACE/vk.bin

echo "4. prove"
${ZKIT} prove -c $WORKSPACE/$CIRCUIT.r1cs -w $WORKSPACE/witness.wtns -s ${SRS} --b $WORKSPACE/proof.bin

echo "5. Verify the proof"
${ZKIT} verify -p $WORKSPACE/proof.bin -v $WORKSPACE/vk.bin

echo "6. Generate verifier"
${ZKIT} generate_verifier -v $WORKSPACE/vk.bin --s single/contracts/verifier.sol

echo "7. run verifier test"
cd $CUR_DIR/single && npm run test
