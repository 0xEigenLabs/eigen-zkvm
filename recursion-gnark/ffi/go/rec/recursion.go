package rec

import (
	"fmt"
	"os"
	"math/big"
	recursion_groth16 "github.com/consensys/gnark/std/recursion/groth16"
	"github.com/consensys/gnark-crypto/ecc"
//	bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381"
//	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/backend/groth16"
//	groth16backend_bls12381 "github.com/consensys/gnark/backend/groth16/bls12-381"
	//groth16backend_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/std/algebra"
//	"github.com/consensys/gnark/std/algebra/emulated/sw_bls12381"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
//	"github.com/consensys/gnark/std/algebra/emulated/sw_bw6761"
//	"github.com/consensys/gnark/std/algebra/native/sw_bls12377"
//	"github.com/consensys/gnark/std/algebra/native/sw_bls24315"
	"github.com/consensys/gnark/std/math/emulated"
//	"github.com/consensys/gnark/std/math/emulated/emparams"
	"github.com/consensys/gnark/test"
)

func NoError(err error) {
	if err != nil {
		panic(err)
	}
}

type InnerCircuit struct {
	P, Q frontend.Variable
	N    frontend.Variable `gnark:",public"`
}

func (c *InnerCircuit) Define(api frontend.API) error {
	res := api.Mul(c.P, c.Q)
	api.AssertIsEqual(res, c.N)
	return nil
}

type OuterCircuit[FR emulated.FieldParams, G1El algebra.G1ElementT, G2El algebra.G2ElementT, GtEl algebra.GtElementT] struct {
	Proof        recursion_groth16.Proof[G1El, G2El]
	VerifyingKey recursion_groth16.VerifyingKey[G1El, G2El, GtEl]
	InnerWitness recursion_groth16.Witness[FR]
}

func (c *OuterCircuit[FR, G1El, G2El, GtEl]) Define(api frontend.API) error {
	verifier, err := recursion_groth16.NewVerifier[FR, G1El, G2El, GtEl](api)
	if err != nil {
		return fmt.Errorf("new verifier: %w", err)
	}
	return verifier.AssertProof(c.VerifyingKey, c.Proof, c.InnerWitness)
}

func getInner(field *big.Int) (constraint.ConstraintSystem, groth16.VerifyingKey, witness.Witness, groth16.Proof) {
	innerCcs, err := frontend.Compile(field, r1cs.NewBuilder, &InnerCircuit{})
	NoError(err)
	innerPK, innerVK, err := groth16.Setup(innerCcs)
	NoError(err)

	// inner proof
	innerAssignment := &InnerCircuit{
		P: 3,
		Q: 5,
		N: 15,
	}
	innerWitness, err := frontend.NewWitness(innerAssignment, field)
	NoError(err)
	innerProof, err := groth16.Prove(innerCcs, innerPK, innerWitness)
	NoError(err)
	innerPubWitness, err := innerWitness.Public()
	NoError(err)
	err = groth16.Verify(innerProof, innerVK, innerPubWitness)
	NoError(err)
	return innerCcs, innerVK, innerPubWitness, innerProof
}

func VerifyBN254InBLS12381() {
	innerCcs, innerVK, innerWitness, innerProof := getInner(ecc.BN254.ScalarField())

	// outer proof
	circuitVk, err := recursion_groth16.ValueOfVerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](innerVK)
	NoError(err)
	circuitWitness, err := recursion_groth16.ValueOfWitness[sw_bn254.ScalarField](innerWitness)
	NoError(err)
	circuitProof, err := recursion_groth16.ValueOfProof[sw_bn254.G1Affine, sw_bn254.G2Affine](innerProof)
	NoError(err)

	outerCircuit := &OuterCircuit[sw_bn254.ScalarField, sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl]{
		Proof:        recursion_groth16.PlaceholderProof[sw_bn254.G1Affine, sw_bn254.G2Affine](innerCcs),
		InnerWitness: recursion_groth16.PlaceholderWitness[sw_bn254.ScalarField](innerCcs),
		VerifyingKey: recursion_groth16.PlaceholderVerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](innerCcs),
	}
	outerAssignment := &OuterCircuit[sw_bn254.ScalarField, sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl]{
		InnerWitness: circuitWitness,
		Proof:        circuitProof,
		VerifyingKey: circuitVk,
	}
	err = test.IsSolved(outerCircuit, outerAssignment, ecc.BLS12_381.ScalarField())
	NoError(err)

	// compile the outer circuit. because we are using 2-chains then the outer
	// curve must correspond to the inner curve. For inner BLS12-377 the outer
	// curve is BW6-761.
	ccs, err := frontend.Compile(ecc.BLS12_381.ScalarField(), r1cs.NewBuilder, outerCircuit)
	if err != nil {
		panic("compile failed: " + err.Error())
	}

	// create Groth16 setup. NB! UNSAFE
	pk, vk, err := groth16.Setup(ccs) // UNSAFE! Use MPC
	if err != nil {
		panic("setup failed: " + err.Error())
	}

	// create prover witness from the assignment
	secretWitness, err := frontend.NewWitness(outerAssignment, ecc.BW6_761.ScalarField())
	if err != nil {
		panic("secret witness failed: " + err.Error())
	}

	// create public witness from the assignment
	publicWitness, err := secretWitness.Public()
	if err != nil {
		panic("public witness failed: " + err.Error())
	}

	// construct the groth16 proof of verifying Groth16 proof in-circuit
	outerProof, err := groth16.Prove(ccs, pk, secretWitness)
	if err != nil {
		panic("proving failed: " + err.Error())
	}

	// verify the Groth16 proof
	err = groth16.Verify(outerProof, vk, publicWitness)
	if err != nil {
		panic("circuit verification failed: " + err.Error())
	}

	f, err := os.Open("/tmp/recursion_proof")
    NoError(err)
	_, err = outerProof.WriteTo(f)
    NoError(err)
}
