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

## Result for fibonacci

Security bits: 63
Generate stark proof and proof verifier: ~8s
Snark setup key power: 20
Snark proof size: 1.1k
Snark time cost: 
