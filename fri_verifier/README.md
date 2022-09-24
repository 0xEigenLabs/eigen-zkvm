# fri_verifier
FRI proof verification by Circom


## Run Fibonacci
### Generate Stark verifier

```
node fibonacci/fibonacci.js -w circuits
```

### Compile verifier and generate snark proof
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
starkStruct.nBitsExt: 13 // extent 9
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
starkStruct.nBitsExt: 17 // extent 7
starkStruct.nQueries: 11
starkStruct.verificationHashType: BN128 //AKA. BN256 in Ethereum

Security bits: 77
Generate stark proof and proof verifier: ~39s
Snark setup key power: 24
Snark proof size: 1.2k
Snark time cost: 144s
```
