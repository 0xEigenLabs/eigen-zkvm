# starkjs

PIL compiler and Circom transpiler. The stark prover is [starky](../starky).

## Run Example
### Arithmetization: Constraint Polynomial

```
npm run fib
```
will generate the PIL json, Commitment Polynomial file and Constant Polynomial file.

### Bottom Layer: FRI Proof

```
../target/release/zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p /tmp/fib.pil.json \
    -o /tmp/fib.const \
    -m /tmp/fib.cm -c circuits/fib.circom -i circuits/fib.zkin.json
```

### Recursive Layer: FRI Proof

```
../target/release/zkit compile -p goldilocks -i circuits/fib.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/
## the above commands is equivalent to
# circom --r1cs --wasm -p goldilocks circuits/fib.circom \
#    -l node_modules/pil-stark/circuits.gl \
#    --O2=full \
#    -o /tmp/

node src/compressor12/main_compressor12_setup.js \
    -r /tmp/fib.r1cs \
    -c /tmp/c12.const \
    -p /tmp/c12.pil \
    -e /tmp/c12.exec

node src/compressor12/main_compressor12_exec.js \
    -w /tmp/fib_js/fib.wasm  \
    -i circuits/fib.zkin.json  \
    -p /tmp/c12.pil  \
    -e /tmp/c12.exec \
    -m /tmp/c12.cm
../target/release/zkit stark_prove -s ../starky/data/c12.starkStruct.json \
    -p /tmp/c12.pil.json \
    -o /tmp/c12.const \
    -m /tmp/c12.cm -c circuits/circuit.circom -i circuits/circuit.zkin.json
```

### Top Layer: Snark proof
```
bash -x ../test/test_fibonacci_verifier.sh
```
