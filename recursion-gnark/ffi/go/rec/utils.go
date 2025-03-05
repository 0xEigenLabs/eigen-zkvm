package rec

import (
	"encoding/gob"
	"encoding/json"
	"fmt"
	"math/big"
	"os"

	bls12381fr "github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	bn254fr "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/backend/groth16"
	groth16_bls12381 "github.com/consensys/gnark/backend/groth16/bls12-381"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	stdgroth16 "github.com/consensys/gnark/std/recursion/groth16"
)

func NewSP1Groth16Proof(proof groth16.Proof, vk groth16.VerifyingKey, publicWitness witness.Witness) GnarkProof {
	vec, ok := publicWitness.Vector().(bls12381fr.Vector)
	if !ok {
		panic("unexpected public witness type")
	}

	gnarkProof := GnarkProof{
		Proof:        proof.(*groth16_bls12381.Proof),
		VerifyingKey: vk.(*groth16_bls12381.VerifyingKey),
		PublicInputs: vec,
	}

	return gnarkProof
}

func ConvertWitness(publicWitness witness.Witness) ([]string, error) {
	// Extract the underlying vector from the public witness.
	vec, ok := publicWitness.Vector().(bls12381fr.Vector)
	if !ok {
		return nil, fmt.Errorf("expected public witness vector to be of type bn254fr.Vector, got %T", publicWitness.Vector())
	}
	// Convert public inputs from bn254fr.Element to decimal strings.
	publicSignals := make([]string, len(vec))
	for i, input := range vec {
		publicSignals[i] = elementToString(input)
	}

	return publicSignals, nil
}

// elementToString converts a bls12381fr.Element to its decimal string representation.
func elementToString(e bls12381fr.Element) string {
	return e.BigInt(new(big.Int)).String()
}

func (c *VerifyCircomProofCircuit) Define(api frontend.API) error {
	verifier, err := stdgroth16.NewVerifier[sw_bn254.ScalarField, sw_bn254.G1Affine, sw_bn254.G2Affine, sw_bn254.GTEl](api)
	if err != nil {
		return fmt.Errorf("new verifier: %w", err)
	}
	return verifier.AssertProof(c.verifyingKey, c.Proof, c.PublicInputs, stdgroth16.WithCompleteArithmetic())
}

// UnmarshalCircomPublicSignalsJSON parses the JSON-encoded public signals data into a slice of strings.
func UnmarshalCircomPublicSignalsJSON(data []byte) ([]string, error) {
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
