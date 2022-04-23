# EigenZKit

1. Transpile R1CS to general proof system, like Plonk and Halo2;

2. Generate solidity verifier;

## Tutorial

### Plonk

1. evaluate the circuits num, and setup $POWER, then download monomial form SRS from `https://universal-setup.ams3.digitaloceanspaces.com/setup_2^${POWER}.key`

2. Generate proof and vk, then verify the proof.
```
cargo run test/ZKMixer/circuit/mixer_js/mixer.r1cs test/ZKMixer/circuit/mixer_js/witness.wtns keys/setup_2\^20.key
```

## Reference

1. https://github.com/fluidex/plonkit
2. https://github.com/matter-labs/recursive_aggregation_circuit
3. https://github.com/matter-labs/zksync/tree/master/core/bin/key_generator
