package rec

import (
	bls12381fr "github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	groth16_bls12381 "github.com/consensys/gnark/backend/groth16/bls12-381"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	recursion "github.com/consensys/gnark/std/recursion/groth16"
	stdgroth16 "github.com/consensys/gnark/std/recursion/groth16"
)

// VerifyCircomProofCircuit is the circuit that verifies the Circom proof inside Gnark
type VerifyCircomProofCircuit struct {
	Proof        stdgroth16.Proof[sw_bn254.G1Affine, sw_bn254.G2Affine]
	verifyingKey stdgroth16.VerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl] `gnark:"-"`
	PublicInputs stdgroth16.Witness[sw_bn254.ScalarField]                                     `gnark:",public"`
}

type Proof struct {
	PublicInputs [2]string `jsocn:"public_inputs"`
	EncodedProof string    `json:"encoded_proof"`
	RawProof     string    `json:"raw_proof"`
}

// GnarkRecursionPlaceholders is a set of placeholders that can be used to define recursive circuits.
type GnarkRecursionPlaceholders struct {
	Vk      recursion.VerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl]
	Witness recursion.Witness[sw_bn254.ScalarField]
	Proof   recursion.Proof[sw_bn254.G1Affine, sw_bn254.G2Affine]
}

// GnarkRecursionProof is a proof that can be used with recursive circuits.
type GnarkRecursionProof struct {
	Proof        recursion.Proof[sw_bn254.G1Affine, sw_bn254.G2Affine]
	Vk           recursion.VerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl]
	PublicInputs recursion.Witness[sw_bn254.ScalarField]
}

// GnarkProof is a proof that can be used with non-recursive circuits.
type GnarkProof struct {
	Proof        *groth16_bls12381.Proof
	VerifyingKey *groth16_bls12381.VerifyingKey
	PublicInputs []bls12381fr.Element
}
