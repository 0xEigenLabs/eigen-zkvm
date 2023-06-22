mkdir -p circuits && node fibonacci/fibonacci.js -w circuits -i "./input1.json" 

../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p /tmp/fib.pil.json \
    --o /tmp/fib.const \
    --m /tmp/fib.cm -c circuits/fib.verifier.circom --i circuits/fib.verifier.zkin.json


../target/release/eigen-zkit compile -p goldilocks -i circuits/fib.verifier.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/


node src/compressor12/main_compressor12_setup.js \
    -r /tmp/fib.verifier.r1cs \
    -c /tmp/c12.const \
    -p /tmp/c12.pil \
    -e /tmp/c12.exec

node src/compressor12/main_compressor12_exec.js \
    -w /tmp/fib.verifier_js/fib.verifier.wasm  \
    -i circuits/fib.verifier.zkin.json  \
    -p /tmp/c12.pil  \
    -e /tmp/c12.exec \
    -m /tmp/c12.cm

../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.bn128.json \
    -p /tmp/c12.pil.json \
    --o /tmp/c12.const \
    --m /tmp/c12.cm -c circuits/fibonacci.verifier_0.circom --i ../test/aggregation/fibonacci/000/input.json --norm_stage

# ../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.json \
#     -p /tmp/c12.pil.json \
#     --o /tmp/c12.const \
#     --m /tmp/c12.cm -c circuits/c12a.verifier.circom --i circuits/c12a.verifier.zkin.json --norm_stage

# ../target/release/eigen-zkit compile -p goldilocks -i circuits/c12a.verifier.circom -l node_modules/pil-stark/circuits.gl  --O2=full -o /tmp/

# node src/compressor12/main_compressor12_setup.js \
#     -r /tmp/c12a.verifier.r1cs \
#     -c /tmp/c12a.verifier.const \
#     -p /tmp/c12a.verifier.pil \
#     -e /tmp/c12a.verifier.exec

# node src/compressor12/main_compressor12_exec.js \
#     -w /tmp/c12a.verifier_js/c12a.verifier.wasm  \
#     -i circuits/c12a.verifier.zkin.json \
#     -p /tmp/c12a.verifier.pil  \
#     -e /tmp/c12a.verifier.exec \
#     -m /tmp/c12a.verifier.cm

# ../target/release/eigen-zkit stark_prove -s ../starky/data/recursive.starkstruct.json \
#     -p /tmp/c12a.verifier.pil.json \
#     --o /tmp/c12a.verifier.const \
#     --m /tmp/c12a.verifier.cm -c circuits/fibonacci.verifier1.circom --i ../test/aggregation/fibonacci/000/input.json
