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



