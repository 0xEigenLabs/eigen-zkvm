# EigenZKit
Write by Circom, prove by Plonk+.

EigenZKit is a ZK DAPP development toolkit,  enabling the developer to write circuits(R1CS) by Circom, transpile the circuits to PLONKish Arithmetization, optimize the proving with the Lookup table and aggregation proof, and finally generate the solidity verifier. 

* [x] Transpile R1CS to PlonK prove system, with recursive proof support;

* [x] Generate solidity verifier;

* [x] GPU acceleration for proving, not opensourced; 

## Tutorial
* Single proof
[test_single.sh](./test/test_single.sh)

* Recursive proof
[test_recursive.sh](./test/test_recursive.sh)

## Applications
* [ZKZRU](https://github.com/ieigen/ZKZRU)

## Acknowledgement

Thanks to the previous work from:

1. https://github.com/iden3/circom
2. https://github.com/fluidex/plonkit
3. https://github.com/matter-labs/recursive_aggregation_circuit
4. https://github.com/matter-labs/zksync/tree/master/core/bin/key_generator
