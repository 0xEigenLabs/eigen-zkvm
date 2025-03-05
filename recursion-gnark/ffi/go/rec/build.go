package rec

import (
	"bytes"
	"encoding/hex"
	"fmt"
	"log"
	"os"
	"sync"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend/groth16"
	groth16_bn254 "github.com/consensys/gnark/backend/groth16/bn254"
	"github.com/consensys/gnark/constraint"
)

var globalMutex sync.RWMutex
var globalR1cs constraint.ConstraintSystem = groth16.NewCS(ecc.BN254)
var globalR1csInitialized = false
var globalPk groth16.ProvingKey = groth16.NewProvingKey(ecc.BN254)
var globalPkInitialized = false

func BuildGroth16(dataDir string, verifyCmdProof string) {
	// Load proof
	proofDecodedBytes, err := hex.DecodeString(verifyCmdProof)
	if err != nil {
		panic(err)
	}
	proof := groth16.NewProof(ecc.BN254)
	if _, err := proof.ReadFrom(bytes.NewReader(proofDecodedBytes)); err != nil {
		panic(err)
	}
	proofBN254, ok := proof.(*groth16_bn254.Proof)
	if !ok {
		panic("failed to convert proof")
	}

	// Load vkey
	vkFile, err := os.Open(fmt.Sprintf("%s/groth16_vk.bin", dataDir))
	if err != nil {
		panic(err)
	}
	vk := groth16.NewVerifyingKey(ecc.BN254)
	vk.ReadFrom(vkFile)
	vkBN254, ok := vk.(*groth16_bn254.VerifyingKey)
	if !ok {
		panic("failed to convert vk")
	}

	// Load public signals
	publicSignalsData, err := os.ReadFile(fmt.Sprintf("%s/public_inputs.json", dataDir))
	if err != nil {
		log.Fatalf("failed to read public signals: %v", err)
	}
	publicSignals, err := UnmarshalCircomPublicSignalsJSON(publicSignalsData)
	if err != nil {
		log.Fatalf("failed to unmarshal public signals: %v", err)
	}
	publicInputs, err := ConvertPublicInputs(publicSignals)
	if err != nil {
		panic(err)
	}

	// Verify the proof outside a recursive circuit
	groth16_bn254.Verify(proofBN254, vkBN254, publicInputs)
	if err != nil {
		panic(err)
	}
	outerProof, verifyingKey, publicWitness, _ := VerifyBN254InBLS12381(proof.(*groth16_bn254.Proof), vk.(*groth16_bn254.VerifyingKey), len(publicSignals), publicInputs, true)

	// Write the verifier key.
	vkBlsFile, err := os.Create(dataDir + "/groth16_vk_bls12381.bin")
	if err != nil {
		panic(err)
	}
	defer vkBlsFile.Close()
	_, err = vk.WriteTo(vkBlsFile)
	if err != nil {
		panic(err)
	}
	// Write the public inputs.
	data, err := ConvertWitness(publicWitness)
	if err != nil {
		panic(err)
	}
	var buffer bytes.Buffer
	for _, line := range data {
		buffer.WriteString(line + "\n")
	}
	os.WriteFile(dataDir+"/public_inputs_bls12381.json", buffer.Bytes(), 0644)

	// Write the proof.
	proofBls := NewSP1Groth16Proof(outerProof, verifyingKey, publicWitness)
	proofBls.SaveToFile(dataDir + "/proof_bls12381.bin")
}
