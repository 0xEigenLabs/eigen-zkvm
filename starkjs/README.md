# starkjs

PIL compiler and Circom transpiler. The stark prover is [starky](../starky).

## Run Example
### Arithmetization:  Generate Polynomial

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
# TODO: replace the tool `circom` by `zkit compile`.
# ../target/debug/zkit compile -p goldilocks -i circuits/circuit.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/
circom --r1cs --wasm -p goldilocks circuits/fib.circom \
    -l node_modules/pil-stark/circuits.gl \
    --O2=full \
    -o /tmp/

node src/compressor12/main_compressor12_setup.js \
    -r /tmp/fib.r1cs \
    -c /tmp/c12.const \
    -p /tmp/c12.pil \
    -e /tmp/c12.exec

// FIXME: -i should be `fib.zkin.json`
node src/compressor12/main_compressor12_exec.js \
    -w /tmp/fib_js/fib.wasm  \
    -i circuits/circuit.zkin.json  \
    -p /tmp/c12.pil  \
    -e /tmp/c12.exec \
    -m /tmp/c12.cm
../target/release/zkit stark_prove -s ../starky/data/starkStruct.json \
    -p /tmp/c12.pil.json \
    -o /tmp/c12.const \
    -m /tmp/c12.cm -c circuits/circuit.circom -i circuits/circuit.zkin.json
```

### Top Layer: Snark proof
```
cd ../test
bash -x test_fibonacci_verifier.sh
```

## Perf test for Fibonacci

### Server configuration
```
CPU: 2.3 GHz Quad-Core Intel Core i7
GPU: NVIDIA TESLA T4 X 4
```

### Experiment result

* e1 (CPU vs GPU):
```
starkStruct.nBits: 4
starkStruct.nBitsExt: 13 // extend 9
starkStruct.nQueries: 7
starkStruct.verificationHashType: BN128 //AKA. BN256

Security bits: 63
Generate stark proof and proof verifier: ~8s
Snark setup key power: 23
Snark proof size: 1.1k
Snark time cost (vs GPU): 590s(vs 81s)
```

* e2 (GPU only):
```
starkStruct.nBits: 10
starkStruct.nBitsExt: 17 // extend 7
starkStruct.nQueries: 11
starkStruct.verificationHashType: BN128 //AKA. BN256 in Ethereum

Security bits: 77
Generate stark proof and proof verifier: ~39s
Snark setup key power: 24
Snark proof size: 1.2k
Snark time cost(GPU): 144s
```

* e3 (GPU only):
```
starkStruct.nBits: 20
starkStruct.nBitsExt: 21 // extend 1
starkStruct.nQueries: 100
starkStruct.verificationHashType: BN128 //AKA. BN256 in Ethereum

Security bits: 100
Generate stark proof and proof verifier: ~298s
Snark setup key power: 26
Snark proof size: 1.2k
Snark time cost(GPU): 546s
```

* e4([aggregation proof](../test/test_aggregation_fibonacci_verifier.sh))

The starkStruct is same as e1.

|step| CPU(s)| GPU(s)|
|--|--|--|
|export_aggregation_verification_key | 540 | 58|
|aggregation_prove| 823 | 137|
