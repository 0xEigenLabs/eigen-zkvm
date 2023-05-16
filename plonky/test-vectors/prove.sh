# EigenZKit
set -ex

cargo build --release

CIRCUIT=$1
CUR_DIR=$(cd $(dirname $0);pwd)
POWER=12
ZKIT="${CUR_DIR}/../../target/release/eigen-zkit"
WORKSPACE=/tmp/$CIRCUIT
rm -rf $WORKSPACE && mkdir -p $WORKSPACE

SRS=${CUR_DIR}/../../keys/setup_2^${POWER}.key

cd $CUR_DIR

if [ ! -f $SRS ]; then
    ${ZKIT} setup -p ${POWER} -s ${SRS}
fi

echo "1. Compile the circuit"
${ZKIT} compile -i $CIRCUIT.circom --O2=full -o $WORKSPACE

echo "2. Generate witness"
node ${WORKSPACE}/${CIRCUIT}_js/generate_witness.js ${WORKSPACE}/${CIRCUIT}_js/$CIRCUIT.wasm $2 $WORKSPACE/witness.wtns

exit

echo "3. Export verification key"
${ZKIT} export_verification_key -s ${SRS}  -c $WORKSPACE/$CIRCUIT.r1cs -v $WORKSPACE/vk.bin

echo "4. prove"
${ZKIT} prove -c $WORKSPACE/$CIRCUIT.r1cs -w $WORKSPACE/witness.wtns -s ${SRS} -b $WORKSPACE/proof.bin

echo "5. Verify the proof"
${ZKIT} verify -p $WORKSPACE/proof.bin -v $WORKSPACE/vk.bin

echo "6. Generate verifier"
${ZKIT} generate_verifier -v $WORKSPACE/vk.bin -s single/contracts/verifier.sol

echo "7. run verifier test"
cd $CUR_DIR/single && npm run test
