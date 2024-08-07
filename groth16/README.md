# Groth16 Performance Analysis

Below, we provide a detailed description of the test circuit constraints used in our evaluations, as well as the proving times when utilizing CPU and GPU resources.

## Circuit Constraints

- Template Instances: 519
- Non-linear Constraints: 8,644,880
- Linear Constraints: 0
- Public Inputs: 0
- Private Inputs: 29,370 (29,366 belong to the witness)
- Public Outputs: 1
- Wires: 8,616,649
- Labels: 11,692,711

## Groth16 Proving Time Analysis

### Machine 1

#### CPU Environment (Machine 1)

- Processor: 13th Gen Intel(R) Core™ i7-13700, 16 cores, base clock 2.7 GHz
- Memory: 32 GB DDR4
- OS: Ubuntu 22.04.4 LTS (Jammy Jellyfish)
- Proving Time: 33.7 seconds

#### GPU Environment (Machine 1)

- GPU Model: NVIDIA GeForce RTX 4060
- Memory: 8 GB GDDR6 (8188 MiB)
- Proving Time: 14.7 seconds

The speedup achieved by using the GPU over the CPU is approximately 2.29.

### Machine 2

#### CPU Environment (Machine 2)

- Processor: AMD EPYC 9354 32-Core Processor, 64 cores, base clock 3.25 GHz
- Memory: 487 GB DDR4
- OS: Ubuntu 22.04.4 LTS (Jammy Jellyfish)
- Proving Time: 37.4 seconds

#### GPU Environment (Machine 2)

- GPU Model: 4 x NVIDIA GeForce RTX 4090
- Memory: 24 GB GDDR6X (24564 MiB)
- Proving Time: 11.2 seconds

The speedup achieved by using the GPU over the CPU is approximately 3.34.