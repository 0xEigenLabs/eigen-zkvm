package eigen

import (
	"encoding/gob"
	"encoding/json"
	"fmt"
	"math/big"
	"os"

	bls12381curve "github.com/consensys/gnark-crypto/ecc/bls12-381"
	bls12381fr "github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	bn254fr "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/backend/groth16"
	groth16_bls12381 "github.com/consensys/gnark/backend/groth16/bls12-381"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	stdgroth16 "github.com/consensys/gnark/std/recursion/groth16"
)

func Convert(proof groth16.Proof, vk groth16.VerifyingKey, publicWitness witness.Witness) (*ArkGroth16Proof, *VerificationKey, []string, error) {
	// Extract the underlying vector from the public witness.
	vec, ok := publicWitness.Vector().(bls12381fr.Vector)
	if !ok {
		return nil, nil, nil, fmt.Errorf("expected public witness vector to be of type bls12381fr.Vector, got %T", publicWitness.Vector())
	}
	// Create a new GnarkProof with the proof, verifying key, and public inputs from Gnark.
	gnarkProof := &GnarkProof{
		Proof:        proof.(*groth16_bls12381.Proof),
		VerifyingKey: vk.(*groth16_bls12381.VerifyingKey),
		PublicInputs: vec,
	}

	// Convert the proof.
	piA, err := g1ToString(&gnarkProof.Proof.Ar)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to convert proof.Ar: %w", err)
	}
	piC, err := g1ToString(&gnarkProof.Proof.Krs)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to convert proof.Krs: %w", err)
	}
	piB, err := g2ToString(&gnarkProof.Proof.Bs)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to convert proof.Bs: %w", err)
	}

	arkProof := &ArkGroth16Proof{
		PiA:      piA,
		PiB:      piB,
		PiC:      piC,
		Protocol: "groth16",
		Curve:    "bls12381",
	}
	// Convert the verification key.
	vkey := gnarkProof.VerifyingKey
	if vk == nil {
		return nil, nil, nil, fmt.Errorf("VerifyingKey is nil in gnarkProof")
	}

	vkAlpha1, err := g1ToString(&vkey.G1.Alpha)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to convert vk.G1.Alpha: %w", err)
	}
	vkBeta2, err := g2ToString(&vkey.G2.Beta)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to convert vk.G2.Beta: %w", err)
	}
	vkGamma2, err := g2ToString(&vkey.G2.Gamma)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to convert vk.G2.Gamma: %w", err)
	}
	vkDelta2, err := g2ToString(&vkey.G2.Delta)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to convert vk.G2.Delta: %w", err)
	}

	// Convert the IC array (G1 points for public inputs).
	ic := make([][]string, len(vkey.G1.K))
	for i, pt := range vkey.G1.K {
		ptStr, err := g1ToString(&pt)
		if err != nil {
			return nil, nil, nil, fmt.Errorf("failed to convert IC[%d]: %w", i, err)
		}
		ic[i] = ptStr
	}
	// nPublic is expected to be the number of public inputs (IC should have length nPublic+1).
	nPublic := len(gnarkProof.PublicInputs)
	// For non-recursive proofs, nPublic = len(publicInputs). But in recursive proofs we may have
	// an extra element in IC. So if there are no public inputs and IC has more than one element,
	// trim IC to length 1.
	if nPublic == 0 && len(ic) > 1 {
		ic = ic[:1]
	}

	// Compute vk_alphabeta_12 = e(vk_alpha_1, vk_beta_2) in the ArkProof format.
	alphabeta, err := ComputeAlphabeta12(vkey.G1.Alpha, vkey.G2.Beta)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to compute vk_alphabeta_12: %w", err)
	}

	arkVk := &VerificationKey{
		Protocol:      "groth16",
		Curve:         "bls12381",
		NPublic:       nPublic,
		VkAlpha1:      vkAlpha1,
		VkBeta2:       vkBeta2,
		VkGamma2:      vkGamma2,
		VkDelta2:      vkDelta2,
		IC:            ic,
		VkAlphabeta12: alphabeta,
	}

	publicSignals := make([]string, len(gnarkProof.PublicInputs))
	for i, input := range gnarkProof.PublicInputs {
		publicSignals[i] = elementToString(input)
	}

	return arkProof, arkVk, publicSignals, nil
}

func g1ToString(p *bls12381curve.G1Affine) ([]string, error) {
	if p == nil {
		return nil, fmt.Errorf("nil G1 point")
	}
	xBig := p.X.BigInt(new(big.Int))
	yBig := p.Y.BigInt(new(big.Int))
	return []string{
		xBig.String(),
		yBig.String(),
		"1",
	}, nil
}

func g2ToString(p *bls12381curve.G2Affine) ([][]string, error) {
	if p == nil {
		return nil, fmt.Errorf("nil G2 point")
	}
	x0 := p.X.A0.BigInt(new(big.Int))
	x1 := p.X.A1.BigInt(new(big.Int))
	y0 := p.Y.A0.BigInt(new(big.Int))
	y1 := p.Y.A1.BigInt(new(big.Int))

	return [][]string{
		{x0.String(), x1.String()},
		{y0.String(), y1.String()},
		{"1", "0"},
	}, nil
}

// elementToString converts a bls12381fr.Element to its decimal string representation.
func elementToString(e bls12381fr.Element) string {
	return e.BigInt(new(big.Int)).String()
}

// computeAlphabeta12 computes vk_alphabeta_12 = e(vk_alpha_1, vk_beta_2)
// and returns a 2×3×2 slice of decimal strings.
// The output format is:
//
//	[
//	  [ [C0.B0.A0, C0.B0.A1], [C0.B1.A0, C0.B1.A1], [C0.B2.A0, C0.B2.A1] ],
//	  [ [C1.B0.A0, C1.B0.A1], [C1.B1.A0, C1.B1.A1], [C1.B2.A0, C1.B2.A1] ]
//	]
func ComputeAlphabeta12(alpha bls12381curve.G1Affine, beta bls12381curve.G2Affine) ([][][]string, error) {
	// Compute the pairing; Pair expects slices.
	gt, err := bls12381curve.Pair([]bls12381curve.G1Affine{alpha}, []bls12381curve.G2Affine{beta})
	if err != nil {
		return nil, fmt.Errorf("failed to compute pairing: %v", err)
	}

	out := make([][][]string, 2)
	for i := 0; i < 2; i++ {
		out[i] = make([][]string, 3)
		for j := 0; j < 3; j++ {
			out[i][j] = make([]string, 2)
		}
	}

	// Decompose GT
	c0 := gt.C0 // type E6
	c1 := gt.C1 // type E6

	// For C0
	out[0][0][0] = c0.B0.A0.BigInt(new(big.Int)).String()
	out[0][0][1] = c0.B0.A1.BigInt(new(big.Int)).String()
	out[0][1][0] = c0.B1.A0.BigInt(new(big.Int)).String()
	out[0][1][1] = c0.B1.A1.BigInt(new(big.Int)).String()
	out[0][2][0] = c0.B2.A0.BigInt(new(big.Int)).String()
	out[0][2][1] = c0.B2.A1.BigInt(new(big.Int)).String()

	// For C1
	out[1][0][0] = c1.B0.A0.BigInt(new(big.Int)).String()
	out[1][0][1] = c1.B0.A1.BigInt(new(big.Int)).String()
	out[1][1][0] = c1.B1.A0.BigInt(new(big.Int)).String()
	out[1][1][1] = c1.B1.A1.BigInt(new(big.Int)).String()
	out[1][2][0] = c1.B2.A0.BigInt(new(big.Int)).String()
	out[1][2][1] = c1.B2.A1.BigInt(new(big.Int)).String()

	return out, nil
}

func (c *VerifierBN254ProofCircuit) Define(api frontend.API) error {
	verifier, err := stdgroth16.NewVerifier[sw_bn254.ScalarField, sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](api)
	if err != nil {
		return fmt.Errorf("new verifier: %w", err)
	}
	return verifier.AssertProof(c.verifyingKey, c.Proof, c.PublicInputs, stdgroth16.WithCompleteArithmetic())
}

// UnmarshalPublicSignalsJSON parses the JSON-encoded public signals data into a slice of strings.
func UnmarshalPublicSignalsJSON(data []byte) ([]string, error) {
	// Parse public signals
	var publicSignals []string
	if err := json.Unmarshal(data, &publicSignals); err != nil {
		return nil, fmt.Errorf("error parsing public signals: %w", err)
	}
	return publicSignals, nil
}

// ConvertPublicInputs parses an array of strings representing public inputs
// into a slice of bn254fr.Element.
func ConvertPublicInputs(publicSignals []string) ([]bn254fr.Element, error) {
	publicInputs := make([]bn254fr.Element, len(publicSignals))
	for i, s := range publicSignals {
		bi, err := stringToBigInt(s)
		if err != nil {
			return nil, fmt.Errorf("failed to parse public input %d: %v", i, err)
		}
		publicInputs[i].SetBigInt(bi)
	}
	return publicInputs, nil
}

// stringToBigInt converts a string to a big.Int, handling both decimal and hexadecimal representations.
func stringToBigInt(s string) (*big.Int, error) {
	if len(s) >= 2 && s[:2] == "0x" {
		bi, ok := new(big.Int).SetString(s[2:], 16)
		if !ok {
			return nil, fmt.Errorf("failed to parse hex string %s", s)
		}
		return bi, nil
	}
	bi, ok := new(big.Int).SetString(s, 10)
	if !ok {
		return nil, fmt.Errorf("failed to parse decimal string %s", s)
	}
	return bi, nil
}

// SaveToFile writes the GnarkProof to a file using gob encoding.
func (gp *GnarkProof) SaveToFile(filename string) error {
	file, err := os.Create(filename)
	if err != nil {
		return err
	}
	defer file.Close()

	encoder := gob.NewEncoder(file)
	err = encoder.Encode(gp)
	if err != nil {
		return err
	}
	return nil
}

// LoadFromFile reads a GnarkProof from a file using gob decoding.
func LoadFromFile(filename string) (*GnarkProof, error) {
	file, err := os.Open(filename)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	var gp GnarkProof
	decoder := gob.NewDecoder(file)
	err = decoder.Decode(&gp)
	if err != nil {
		return nil, err
	}
	return &gp, nil
}
