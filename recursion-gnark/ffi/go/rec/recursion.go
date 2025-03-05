package rec

import (
	"fmt"
	"log"
	"math/big"
	"os"

	"github.com/consensys/gnark-crypto/ecc"

	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"

	bn254fr "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	groth16_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/std/algebra"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	"github.com/consensys/gnark/std/math/emulated"
	recursion_groth16 "github.com/consensys/gnark/std/recursion/groth16"
	"github.com/consensys/gnark/test"
)

func NoError(err error) {
	if err != nil {
		panic(err)
	}
}

// PlaceholdersForRecursion creates placeholders for the recursion proof and
// verification key. If fixedVk is true, the verification key is fixed and must
// be defined as 'gnark:"-"' in the Circuit. It only needs the number of public
// inputs and the circom verification key.
func PlaceholdersForRecursion(vk *groth16_bn254.VerifyingKey, nPublicInputs int,
	fixedVk bool) (*GnarkRecursionPlaceholders, error) {
	// create the placeholder for the recursion circuit
	if fixedVk {
		return createPlaceholdersForRecursionWithFixedVk(vk, nPublicInputs)

	}
	return createPlaceholdersForRecursion(vk, nPublicInputs)
}

// createPlaceholdersForRecursion creates placeholders for the recursion proof
// and verification key. It returns a set of placeholders needed to define the
// recursive circuit. Use this function when the verification key is fixed
// (defined as 'gnark:"-"').
func createPlaceholdersForRecursionWithFixedVk(vk *groth16_bn254.VerifyingKey,
	nPublicInputs int) (*GnarkRecursionPlaceholders, error) {
	if vk == nil || nPublicInputs < 0 {
		return nil, fmt.Errorf("invalid inputs to create placeholders for recursion")
	}
	placeholderVk, err := recursion_groth16.ValueOfVerifyingKeyFixed[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](vk)
	if err != nil {
		return nil, fmt.Errorf("failed to convert verification key to recursion verification key: %w", err)
	}

	placeholderWitness := recursion_groth16.Witness[sw_bn254.ScalarField]{
		Public: make([]emulated.Element[sw_bn254.ScalarField], nPublicInputs),
	}
	placeholderProof := recursion_groth16.Proof[sw_bn254.G1Affine, sw_bn254.G2Affine]{}

	return &GnarkRecursionPlaceholders{
		Vk:      placeholderVk,
		Witness: placeholderWitness,
		Proof:   placeholderProof,
	}, nil
}

// createPlaceholdersForRecursion creates placeholders for the recursion proof
// and verification key. It returns a set of placeholders needed to define the
// recursive circuit. Use this function when the verification key is not fixed.
func createPlaceholdersForRecursion(vk *groth16_bn254.VerifyingKey,
	nPublicInputs int) (*GnarkRecursionPlaceholders, error) {
	placeholders, err := createPlaceholdersForRecursionWithFixedVk(vk, nPublicInputs)
	if err != nil {
		return nil, err
	}
	placeholders.Vk.G1.K = make([]sw_bn254.G1Affine, len(placeholders.Vk.G1.K))
	return placeholders, nil
}

// ConvertCircomToGnarkRecursion converts a Circom proof, verification key, and
// public signals to the Gnark recursion proof format. If fixedVk is true, the
// verification key is fixed and must be defined as 'gnark:"-"' in the Circuit.
func ConvertCircomToGnarkRecursion(vk *groth16_bn254.VerifyingKey,
	proof *groth16_bn254.Proof, publicInputs []bn254fr.Element, fixedVk bool,
) (*GnarkRecursionProof, error) {
	// Convert the proof and verification key to recursion types
	recursionProof, err := recursion_groth16.ValueOfProof[sw_bn254.G1Affine, sw_bn254.G2Affine](proof)
	if err != nil {
		return nil, fmt.Errorf("failed to convert proof to recursion proof: %w", err)
	}
	// Transform the public inputs to emulated elements for the recursion circuit
	publicInputElementsEmulated := make([]emulated.Element[sw_bn254.ScalarField], len(publicInputs))
	for i, input := range publicInputs {
		bigIntValue := input.BigInt(new(big.Int))
		elem := emulated.ValueOf[sw_bn254.ScalarField](bigIntValue)
		publicInputElementsEmulated[i] = elem
	}
	// Create assignments
	assignments := &GnarkRecursionProof{
		Proof: recursionProof,
		PublicInputs: recursion_groth16.Witness[sw_bn254.ScalarField]{
			Public: publicInputElementsEmulated,
		},
	}
	if !fixedVk {
		// Create the recursion types
		recursionVk, err := recursion_groth16.ValueOfVerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](vk)
		if err != nil {
			return nil, fmt.Errorf("failed to convert verification key to recursion verification key: %w", err)
		}
		assignments.Vk = recursionVk
	}
	return assignments, nil
}

func circom2gnarkRecursiveBls12381(proofData *groth16_bn254.Proof, vkData *groth16_bn254.VerifyingKey, len int, publicInputs []bn254fr.Element, runtest bool) (groth16.Proof, groth16.VerifyingKey, witness.Witness, constraint.ConstraintSystem) {
	// Build a new circuit to verify the Circom proof recursively
	// Get the recursion proof and placeholders
	recursionPlaceholders, err := PlaceholdersForRecursion(vkData, len, true)
	if err != nil {
		log.Fatalf("failed to create placeholders for recursion: %v", err)
	}
	recursionData, err := ConvertCircomToGnarkRecursion(vkData, proofData, publicInputs, true)
	if err != nil {
		log.Fatalf("failed to convert Circom proof to Gnark recursion proof: %v", err)
	}

	// Create placeholder circuit
	placeholderCircuit := &VerifyCircomProofCircuit{
		recursionPlaceholders.Proof,
		recursionPlaceholders.Vk,
		recursionPlaceholders.Witness,
	}
	// Create the circuit assignment with actual values
	circuitAssignment := &VerifyCircomProofCircuit{
		Proof:        recursionData.Proof,
		PublicInputs: recursionData.PublicInputs,
	}

	err = test.IsSolved(placeholderCircuit, circuitAssignment, ecc.BLS12_381.ScalarField())
	NoError(err)

	// compile the outer circuit. because we are using 2-chains then the outer
	// curve must correspond to the inner curve. For inner BLS12-377 the outer
	// curve is BW6-761.
	ccs, err := frontend.Compile(ecc.BLS12_381.ScalarField(), r1cs.NewBuilder, placeholderCircuit)
	if err != nil {
		panic("compile failed: " + err.Error())
	}

	// create Groth16 setup. NB! UNSAFE
	pk, vk, err := groth16.Setup(ccs) // UNSAFE! Use MPC
	if err != nil {
		panic("setup failed: " + err.Error())
	}

	// create prover witness from the assignment
	secretWitness, err := frontend.NewWitness(circuitAssignment, ecc.BLS12_381.ScalarField())
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

	return outerProof, vk, publicWitness, ccs
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
	secretWitness, err := frontend.NewWitness(outerAssignment, ecc.BLS12_381.ScalarField())
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

	f, err := os.Create("/tmp/recursion_proof.txt")
	NoError(err)
	_, err = outerProof.WriteTo(f)
	NoError(err)
}
