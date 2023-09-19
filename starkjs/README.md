# starkjs

PIL compiler and Circom transpiler. The stark prover is [starky](../starky).

## Run Example
### Arithmetization: Constraint Polynomial

```bash
npm run fib
```
will generate the PIL json, Commitment Polynomial file and Constant Polynomial file.

### Bottom Layer: FRI Proof

```bash
../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p /tmp/fib.pil.json \
    --o /tmp/fib.const \
    --m /tmp/fib.cm -c circuits/fib.verifier.circom --i circuits/fib.verifier.zkin.json
```

### Recursive Layer: FRI Proof
test compressor12

#### old script
```bash
../target/release/eigen-zkit compile -p goldilocks -i circuits/fib.verifier.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/
## the above commands is equivalent to
# circom --r1cs --wasm -p goldilocks circuits/fib.circom \
#    -l node_modules/pil-stark/circuits.gl \
#    --O2=full \
#    -o /tmp/

# Circom to Stark  
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
../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.json \
    -p /tmp/c12.pil.json \
    --o /tmp/c12.const \
    --m /tmp/c12.cm -c circuits/c12a.verifier.circom --i circuits/c12a.verifier.zkin.json --norm_stage
```

#### new script
> rust version script
```bash
../target/release/eigen-zkit compile -p goldilocks -i circuits/fib.verifier.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/
## the above commands is equivalent to
# circom --r1cs --wasm -p goldilocks circuits/fib.circom \
#    -l node_modules/pil-stark/circuits.gl \
#    --O2=full \
#    -o /tmp/

# Circom to Stark  
#node src/compressor12/main_compressor12_setup.js \
#    -r /tmp/fib.verifier.r1cs \
#    -c /tmp/c12.const \
#    -p /tmp/c12.pil \
#    -e /tmp/c12.exec

../target/release/eigen-zkit compressor12_setup 
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
    
../target/release/eigen-zkit stark_prove -s ../starky/data/c12.starkStruct.json \
    -p /tmp/c12.pil.json \
    --o /tmp/c12.const \
    --m /tmp/c12.cm -c circuits/c12a.verifier.circom --i circuits/c12a.verifier.zkin.json --norm_stage
```

### Top Layer: Snark proof
```bash
bash -x ../test/test_fibonacci_verifier.sh
```

### Snark proof aggregation

```
bash -x ../test/test_aggregation_verifier.sh
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
