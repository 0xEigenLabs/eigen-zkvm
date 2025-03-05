package sp1

import (
	"encoding/json"
	"fmt"
	"math/big"

	bn254fr "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/sw_bn254"
	stdgroth16 "github.com/consensys/gnark/std/recursion/groth16"
)

// func NewSP1Groth16Proof(proof *groth16.Proof, witnessInput WitnessInput) Proof {
// 	var buf bytes.Buffer
// 	(*proof).WriteRawTo(&buf)
// 	proofBytes := buf.Bytes()

// 	var publicInputs [2]string
// 	publicInputs[0] = witnessInput.VkeyHash
// 	publicInputs[1] = witnessInput.CommittedValuesDigest

// 	// Cast groth16 proof into groth16_bn254 proof so we can call MarshalSolidity.
// 	p := (*proof).(*groth16_bn254.Proof)

// 	encodedProof := p.MarshalSolidity()

// 	return Proof{
// 		PublicInputs: publicInputs,
// 		EncodedProof: hex.EncodeToString(encodedProof),
// 		RawProof:     hex.EncodeToString(proofBytes),
// 	}
// }

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
