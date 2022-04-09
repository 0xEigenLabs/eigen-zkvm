# EigenZKit

Transpile R1CS for Plonk and Halo2.

## Tutorial

### Plonk

1. evaluate the circuits num, and setup $POWER, then download monomial form SRS from `https://universal-setup.ams3.digitaloceanspaces.com/setup_2^${POWER}.key`

2. Generate proof and vk, then verify the proof.
```
cargo run ZKMixer/circuit/mixer_js/mixer.r1cs ZKMixer/circuit/mixer_js/witness.wtns setup_2\^20.key
```
