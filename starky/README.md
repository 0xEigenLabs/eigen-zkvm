# starky

Generates a STARK proof from a State Machine written in PIL Language. Rusty Polygon Hermez [pil-stark](https://github.com/0xPolygonHermez/pil-stark).

# Example

[starkjs](../starkjs)


# Design

## The background
After performance test on starkjs, we observe that the SM takes hours to generate a proof, and this makes it much impractical as a prototype of zk-zkVM.
So we plan to build a stark proving service to shift the computing intensive workload.

A general process to generate a stark proof includes:

1. compose the PIL program

2. compile the PIL program by pilcom

3. build the constant polynomial and execution trace polynomial by user-defined function

4. PIL Codegen
> 1. generate the plookup identities
> 2. generate the permutation check identities (linear constraints)
> 3. generate the commitment polynomial
> 4. generate FRI polynomial (composite polynomial)

5. [SLOW]calculate the merkle proof of quotient polynomial(Q), the plookup polynomial(H1H2), the target polynomial (Z), and the composite polynomial(C)

6. [XSLOW]evaluate the above constraint polynomials

7. build the FRI proof

8. generate the consttree

To be specific, the main calculation in step 4 is `extendAndMerkelize` over 256bits scalar field(BN128). For step 5, the main computation comes from polynomial evaluation.


* Extending

Extending is the process of generating a new polynomial from the old polynomial with point-value representation, by multiplying the x-axis of the points with 2^(nExtBits - nBits) then interpolate on the new points.

* Merkelization

Merkelization calculates the Merkel proof of a polynomial, where the leaf nodes of the Merkel Tree is the points generating the polynomial. The hash function is Linearhash on 256bits scalar field, and Linearhash is built on linear Poseidon hash with arbitrary length message as input under fixed-size=12 MDS.

>* MerkleHash performance(128 cores, 1T RAM):

|height|n_pols| starky(s)|pil-stark JS(s)|
|---|---|---|---|
|2^24|10|11.04| 74.77|
|2^24|100| 85.12| 582 |
|2^24|600| 482 | -|


>* Comparison with CPP prover:

|height|n_pols| starky(s)|pil-stark CPP(s)|
|---|---|---|---|
|2^24|12|11.1 | 8.24507 |
|2^24|79| 66.01 |49.9154 |


* Polynomial evaluation

The evaluation is to calculate by:

```
let acc = 0n; // zero on GF(2^3)

let v be the point to evaluate;
let l be the coefficient vector of the polynomial;
for (let k=0; k<2^nExtBits; k++) {
    acc = F.add(acc, F.mul(v, l[k]));
}
```

Because the nExtBits reaches up to 24 so this step would be very slow, especially when the `starkInfo.evMap.length` is bigger than 2^16 in SM.

## Optimization

* reimpl pil-stark by Rust
* hardware acceleration

### Progress

- [x] Fully PIL syntax support
- [x] Parallel Merklehash and Cooley–Tukey FFT by Rayon
- [x] Codegen (arithmetization)
- [x] Verification hash type
> - [x] BN128
> - [x] GL(F64)
- [x] Parallel reduce for polynomial evaluation
- [x] Recursive FRI
- [x] Poseidon Hash on GPU/Multicore for BN128
- [] Polynomial evaluation on GPU

## Profiling

```
cargo bench --bench merklehash -- --profile-time=5
```

* https://www.jibbow.com/posts/criterion-flamegraphs/


