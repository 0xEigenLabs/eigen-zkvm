# recursion-gnark

Convert the Gnark Groth16 proof over BN254 to ArkWorks Groth16 over BLS12381 


## E2E test

```bash
cd cli 
cargo run -r test --system groth16 --vk-path ../ffi/data/groth16_vk.bin --output-dir ../ffi/data --proof-path ../ffi/data/proof.bin

cd ffi 
cargo test -r
```