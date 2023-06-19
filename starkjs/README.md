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
../target/release/eigen-zkit stark_prove -s ../starky/data/starkStruct.json.gl \
    -p /tmp/fib.pil.json \
    -o /tmp/fib.const \
    -m /tmp/fib.cm -c circuits/fib.verifier.circom -i circuits/fib.verifier.zkin.json
```

### Recursive Layer: FRI Proof

```
../target/release/eigen-zkit compile -p goldilocks -i circuits/fib.verifier.circom -l node_modules/pil-stark/circuits.gl --O2=full -o /tmp/
## the above commands is equivalent to
# circom --r1cs --wasm -p goldilocks circuits/fib.circom \
#    -l node_modules/pil-stark/circuits.gl \
#    --O2=full \
#    -o /tmp/


// Circom to Stark  
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
    -o /tmp/c12.const \
    -m /tmp/c12.cm -c circuits/c12a.verifier.circom -i circuits/c12a.verifier.zkin.json
```

### Normalization Layer [bn128]
```
../target/release/eigen-zkit compile -p goldilocks -i circuits/c12a.verifier.circom -l node_modules/pil-stark/circuits.gl  --O2=full -o /tmp/
```


But you need to do some hardcode work here:

- (1) modify the end line of circuits/c12a.verifier.circom 
```
    "component main {public [publics]}= StarkVerifier();" ==> component main {public [publics,rootC]}= StarkVerifier()
``` 
- (2) add signal input to rootC at circuits/c12a.verifier.circom 

origin version 
```
    signal rootC[4];
    rootC[0] <== 2144474125363499765;
    rootC[1] <== 1583360444347119487;
    rootC[2] <== 8407973231335465230;
    rootC[3] <== 15954052097301235018;
```
update version 
```
    signal input rootC[4];
    // rootC[0] <== 2144474125363499765;
    // rootC[1] <== 1583360444347119487;
    // rootC[2] <== 8407973231335465230;
    // rootC[3] <== 15954052097301235018;
```
- (3) add the data of rootC into circuits/c12a.verifier.zkin.json
```
    {...."publics":["1","2","74469561660084004"],"rootC":["2144474125363499765","1583360444347119487","8407973231335465230","15954052097301235018"]}
```

Convert r1cs to pil and generate stark proof 
```
 node src/compressor12/main_compressor12_setup.js \
    -r /tmp/c12a.verifier.r1cs \
    -c /tmp/c12a.verifier.const \
    -p /tmp/c12a.verifier.pil \
    -e /tmp/c12a.verifier.exec

node src/compressor12/main_compressor12_exec.js \
    -w /tmp/c12a.verifier_js/c12a.verifier.wasm  \
    -i circuits/c12a.verifier.zkin.json \
    -p /tmp/c12a.verifier.pil  \
    -e /tmp/c12a.verifier.exec \
    -m /tmp/c12a.verifier.cm
```


Generate recursive1_verify stark proof
```
../target/release/eigen-zkit stark_prove -s ../starky/data/recursive.starkstruct.json \
    -p /tmp/c12a.verifier.pil.json \
    -o /tmp/c12a.verifier.const \
    -m /tmp/c12a.verifier.cm -c circuits/recursive1.verifier.circom -i circuits/recursive1.verifier.zkin.json
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
