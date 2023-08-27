# Stark Circuit

Stark circuit is the valilla Stark verifier implementation over both BN254 and BLS12-381 by Circom 2. We call the scalar field of bn254 or BLS12-381 as big field for short.
This code orignals from [Hermez pil-stark](https://github.com/0xPolygonHermez/pil-stark), and here we generalize the basic blocks for big fields, using `n_limb` as the number of Godilocks elements to indicate one of big field.
To be specific, one BN254 element can contain 3 Godilocks elements, and one BLS12-381 contains 5.

## Basic Blocks

* bn1togl3.circom: convert elemens in big field to Godilocks elements
* evalpol.circom: evaluate polynomial on some point
* gl.circom: Godilocks field computation
* merkle.circom: calculate the Merkle tree root
* treeselector.circom: select the leaf node on Merkle Tree
* compconstant64.circom: comparison operators for Godilocks
* fft.circom:
* linearhash.circom: calculathe the hash of arbitrary vector by Poseidon Hash.
* merklehash.circom: merkelization

## Rationale

### Irreducible polinomial

For the element mapping from Goldilocks field to Scalar field of BLS12-381, a field switch is need.
The scalar field of bls381 could be presented by 5-64bits, by refering to the switch for Godilocks-BN254, we choose the valilla Irreducible Polynomail atop GF(2^5).

* BN254: `x^3 - x + 1`
* BLS12-381: `x^5 - x^2 + 1`

use sage to verify this:

```
sage: p = 2
sage: R.<x> = GF(p)[]
sage: (x^5 -x^2 + 1).is_irreducible()
True
sage: (x^5 - 3).is_irreducible()
False
sage: (x^3 + x + 1).is_irreducible()
True
```

Read [more](https://www.partow.net/programming/polynomials/index.html).

### Arithmetic

Given `p = 2^64 - 2^32 + 1`,

* Add/Sub: pc = pa +/- pb, pc[i] = (pa[i] +/- pb[i]) mod p

* Mul

For BN254

```
pa * pb = (a, b, c) * (d, e, f) = (a*d+b*f+c*e, a*e+b*d+b*f+c*e+c*f, a*f+b*e+c*d+c*f) mod p
```

For BLS12-381


```
pa * pb = (a, b, c, d, e) * (f, g, h, i, j) = ((a + bx + cx^2 + dx^3 + ex^4)*(f + gx + hx^2 + ix^3 + jx^4)) % (x^5 - x^2 + 1)
        = (
        af + eg + dh + ci + (bj + ej),
        bf + ag + eh + di + cj,
        cf + (bg + eg) + (bh + dh) + (ci + ei) + (bj + dj + ej),
        df + cg + (bh + eh) + (ai + di) + (cj + ej),
        ef + dg + ch + (bi + ei) + (aj + dj),
        )
```

* Inv

Using a * b = 1 to solve `a^-1`. For a multivariable polynomial, it's the solution to a multivariable equations.

For BN254, it just need to solve below multivariable polynomial to get `d, e, f`.

```
f*a+b*e+d*c+c*f = 0
d*b+e*a+c*f+b*f+e*c=0
a*d+b*f+e*c=1
```

The solver can be found [here](https://www.polymathlove.com/polymonials/midpoint-of-a-line/symbolic-equation-solving.html#c=solve_algstepsequationsolvesystem&v247=d%252Ce%252Cf&v248=3&v249=f*a%2Bb*e%2Bd*c%2B%2520c*f%2520%253D%25200&v250=d*b%2Be*a%2Bc*f%2Bb*f%2Be*c%253D0&v251=a*d%2Bb*f%2Be*c%253D1).

For BLS12-381,

```
af + eg + dh + ci + (bj + ej) = 1
bf + ag + eh + di + cj = 0
cf + (bg + eg) + (bh + dh) + (ci + ei) + (bj + dj + ej) = 0
df + cg + (bh + eh) + (ai + di) + (cj + ej) = 0
ef + dg + ch + (bi + ei) + (aj + dj) = 0
```

The solver can be found [here](https://www.polymathlove.com/polymonials/midpoint-of-a-line/symbolic-equation-solving.html#c=solve_algstepsequationsolvesystem&v247=f%252Cg%252Ch%252Ci%252Cj&v248=5&v249=af%2520%2B%2520eg%2520%2B%2520dh%2520%2B%2520ci%2520%2B%2520%2528bj%2520%2B%2520ej%2529%2520%253D%25201&v250=bf%2520%2B%2520ag%2520%2B%2520eh%2520%2B%2520di%2520%2B%2520cj%2520%253D%25200&v251=cf%2520%2B%2520%2528bg%2520%2B%2520eg%2529%2520%2B%2520%2528bh%2520%2B%2520dh%2529%2520%2B%2520%2528ci%2520%2B%2520ei%2529%2520%2B%2520%2528bj%2520%2B%2520dj%2520%2B%2520ej%2529%2520%253D%25200&v252=df%2520%2B%2520cg%2520%2B%2520%2528bh%2520%2B%2520eh%2529%2520%2B%2520%2528ai%2520%2B%2520di%2529%2520%2B%2520%2528cj%2520%2B%2520ej%2529%2520%253D%25200&v253=ef%2520%2B%2520dg%2520%2B%2520ch%2520%2B%2520%2528bi%2520%2B%2520ei%2529%2520%2B%2520%2528aj%2520%2B%2520dj%2529%2520%253D%25200)

### Generic big field operations

Observe that the multiplication and inversion for scalar field in BN254 and BLS12-381 is quite different, so we can implement two templates for each operator, and choose the right one when rendering the `stark_verifier`.

### Merkle

For the Merkel Circuit, each leaf is N-elements on GL field, where N is 4 for BN254, and 6 for BLS12-381. Before we calculate the merkle root, we need convert the N-elements to one element in big field.
As a refernece, [to\_bn128](https://github.com/0xEigenLabs/eigen-zkvm/blob/main/starky/src/digest.rs#L73) is present, and same conversion should be applied to bls-12381.

## ElementDigest

the struct `ElementDigest` stands for the value of node in Merkle tree. For BN254, each node, including the root, contains 4 Godilocks elements, while BLS12-381 is 6.
