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

## Time used

CPU: 11th Gen Intel(R) Core(TM) i9-11900 @ 2.50GHz, 16core

MEM: 32G

| Step            | time(s) |
| ---             | ---     |
| Arithmetization | 0.021   |
| Bottom Layer    | 0.34    |
| Recursive Layer | 81.55   |
| Top Layer    | 267.2   |


## Recursive Poseidon Test 

```
cd starkjs
```


### 1. generate stark proof
```
npm run poseidon 
```

## 2. stark proof --> stark_verify process --> verify circuits(circom)
```
../target/release/zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p ./poseidon/build/poseidon_test.pil.json \
    -o ./poseidon/build/poseidon_test.const \
    -m ./poseidon/build/poseidon_test.cm -c ./poseidon/circuits/stark_verify.circom -i ./poseidon/circuits/stark_proof.json
```

### 3. generate .wasm and .r1cs through circom 
```
../target/release/zkit compile -p goldilocks -i ./poseidon/circuits/stark_verify.circom -l node_modules/pil-stark/circuits.gl --O2=full -o ./poseidon/circuits/
```



### 4. r1cs --> pil 
```
node src/compressor12/main_compressor12_setup.js \
    -r ./poseidon/circuits/stark_verify.r1cs \
    -c ./poseidon/build/c12.const \
    -p ./poseidon/build/c12.pil \
    -e ./poseidon/build/c12.exec
```

```
node src/compressor12/main_compressor12_exec.js \
    -w ./poseidon/circuits/stark_verify_js/stark_verify.wasm  \
    -i ./poseidon/circuits/stark_proof.json  \
    -p ./poseidon/build/c12.pil  \
    -e ./poseidon/build/c12.exec \
    -m ./poseidon/build/c12.cm
```


### 5. pil --> stark proof --> stark_verify process --> verify circuits(circom)
```
../target/release/zkit stark_prove -s ../starky/data/c12.starkStruct.json  \
    -p ./poseidon/build/c12.pil.json \
    -o ./poseidon/build/c12.const \
    -m ./poseidon/build/c12.cm -c ./poseidon/circuits/c12_verify.circom -i ./poseidon/circuits/c12_stark_proof.json
```