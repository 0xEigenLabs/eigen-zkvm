# EigenZKit

1. Transpile R1CS to PlonK prove system, with recursive and lookup table support;

2. Generate solidity verifier;

## Tutorial

```
cargo build
```

1. Compile the circuit

```
export WORKSPACE=/tmp/abc
./target/debug/zkit compile -i test/multiplier.circom --O2=full -o $WORKSPACE
```

2. Generate witness

```
node ${WORKSPACE}/multiplier_js/generate_witness.js ${WORKSPACE}/multiplier_js/multiplier.wasm test/input.json $WORKSPACE/witness.wtns
```

3. Export verification key

```
./target/debug/zkit export_verification_key -s zklib/keys/setup_2\^20.key  -c $WORKSPACE/multiplier.r1cs
```

4. evaluate the circuits num, and setup $POWER, then download monomial form SRS from `https://universal-setup.ams3.digitaloceanspaces.com/setup_2^${POWER}.key`

```
./target/debug/zkit prove -c $WORKSPACE/multiplier.r1cs -w $WORKSPACE/witness.wtns -s zklib/keys/setup_2\^20.key

```

5. Verify the proof.

```
./target/debug/zkit verify -p proof.bin -v vk.bin
```

6. Generate verifier

```
./target/debug/zkit generate_verifier
```


## Reference

1. https://github.com/fluidex/plonkit
2. https://github.com/matter-labs/recursive_aggregation_circuit
3. https://github.com/matter-labs/zksync/tree/master/core/bin/key_generator
