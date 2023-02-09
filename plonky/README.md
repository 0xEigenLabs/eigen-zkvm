# Plonky

Generate final block validity proof to L1

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
starkStruct.verificationHashType: BN128

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
