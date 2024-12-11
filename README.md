# eigen-zkvm

eigen-zkvm is a zkVM on a layered proof system, allowing the developers to write Zero-knowledge applications, proving with the layered proof system to achieve no trusted setup, constant on-chain proof size and low gas cost, and finally generating the solidity verifier.

- [x] zkit: universal command line for stark, plonk, and groth16.

- [x] Circom 2.x support;

- [x] Proof composition: proof aggregation and recursion on Stark;

- [X] Proof Recursion with Snark on Stark;

- [x] Solidity verifier generation;

- [x] GPU acceleration for proving, not open-sourced;

- [x] Eigen zkVM: basing on RISCV ISA.

## How the layered proof system works

![mixed-proof-system](./docs/mixed-proof-system.png)

## Tutorial
* Generate universal setup key
```
zkit setup -p 13 -s setup_2^13.key
```
For the power from 20 to 26, you can download directly from [universal-setup hub](https://universal-setup.ams3.digitaloceanspaces.com).

* Snark aggregation proof

> [test_aggregation.sh](./test/test_aggregation.sh)

* Stark aggregation proof

> [stark_aggregation.sh yes BN128](./test/stark_aggregation.sh)

> [stark_aggregation.sh yes BLS12381](./test/stark_aggregation.sh)

* Stark proof and recursive stark prove
> [starky](./starky)

* Layered proof

> [starkjs](./starkjs)

## Applications
* [Rust zkVM/Risc V/REVM)](https://docs.powdr.org/backends/estark.html)
* [REVM](https://github.com/powdr-labs/powdr/tree/main/riscv/tests/riscv_data/evm)
* [zkml-rust](https://github.com/eigmax/zkml-rust)
* [eigen-secret](https://github.com/0xEigenLabs/eigen-secret)
* [zk-mixer](https://github.com/0xEigenLabs/zk-mixer)
