package sp1

import (
	bn254fr "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	groth16_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	recursion "github.com/consensys/gnark/std/recursion/groth16"
	stdgroth16 "github.com/consensys/gnark/std/recursion/groth16"
)

var srsFile string = "srs.bin"
var srsLagrangeFile string = "srs_lagrange.bin"
var constraintsJsonFile string = "constraints.json"
var plonkCircuitPath string = "plonk_circuit.bin"
var groth16CircuitPath string = "groth16_circuit.bin"
var plonkVkPath string = "plonk_vk.bin"
var groth16VkPath string = "groth16_vk.bin"
var plonkPkPath string = "plonk_pk.bin"
var groth16PkPath string = "groth16_pk.bin"
var plonkWitnessPath string = "plonk_witness.json"
var groth16WitnessPath string = "groth16_witness.json"

// VerifyCircomProofCircuit is the circuit that verifies the Circom proof inside Gnark
type VerifyCircomProofCircuit struct {
	Proof        stdgroth16.Proof[sw_bn254.G1Affine, sw_bn254.G2Affine]
	verifyingKey stdgroth16.VerifyingKey[sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl] `gnark:"-"`
	PublicInputs stdgroth16.Witness[sw_bn254.ScalarField]                                     `gnark:",public"`
}

// type Constraint struct {
// 	Opcode string     `json:"opcode"`
// 	Args   [][]string `json:"args"`
// }

// type WitnessInput struct {
// 	Vars                  []string   `json:"vars"`
// 	Felts                 []string   `json:"felts"`
// 	Exts                  [][]string `json:"exts"`
// 	VkeyHash              string     `json:"vkey_hash"`
// 	CommittedValuesDigest string     `json:"committed_values_digest"`
// }

type Proof struct {
	PublicInputs [2]string `jsocn:"public_inputs"`
	EncodedProof string    `json:"encoded_proof"`
	RawProof     string    `json:"raw_proof"`
}

// CircomProof represents the proof structure output by SnarkJS.
type CircomProof struct {
	PiA      []string   `json:"pi_a"`
	PiB      [][]string `json:"pi_b"`
	PiC      []string   `json:"pi_c"`
	Protocol string     `json:"protocol"`
}

// CircomVerificationKey represents the verification key structure output by SnarkJS.
type CircomVerificationKey struct {
	Protocol      string       `json:"protocol"`
	Curve         string       `json:"curve"`
	NPublic       int          `json:"nPublic"`
	VkAlpha1      []string     `json:"vk_alpha_1"`
	VkBeta2       [][]string   `json:"vk_beta_2"`
	VkGamma2      [][]string   `json:"vk_gamma_2"`
	VkDelta2      [][]string   `json:"vk_delta_2"`
	IC            [][]string   `json:"IC"`
	VkAlphabeta12 [][][]string `json:"vk_alphabeta_12"` // Not used in verification
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
	Proof        *groth16_bn254.Proof
	VerifyingKey *groth16_bn254.VerifyingKey
	PublicInputs []bn254fr.Element
}
