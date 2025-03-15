# recursion-gnark

Convert the Gnark Groth16 proof over BN254 to ArkWorks Groth16 over BLS12381 


## E2E test

```
cd cli 
cargo run -r test --system groth16 ../ffi/data/ ../ffi/data/proof.bin

cd ffi 
cargo test -r
```