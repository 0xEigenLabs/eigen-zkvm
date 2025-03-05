package main

/*
#include <stdlib.h>

*/
import "C"
import (
	"unsafe"

	//	"github.com/consensys/gnark-crypto/ecc"
	//	"github.com/consensys/gnark/backend/groth16"
	//	"github.com/consensys/gnark/backend/plonk"
	//	"github.com/consensys/gnark/frontend"
	//	"github.com/consensys/gnark/frontend/cs/r1cs"
	//	"github.com/consensys/gnark/frontend/cs/scs"
	//	"github.com/consensys/gnark/test/unsafekzg"
	"github.com/0xEigenLabs/eigen-recursion-gnark/rec"
	"github.com/0xEigenLabs/eigen-recursion-gnark/sp1"
)

func main() {}

//export VerifyBN254InBLS12381
func VerifyBN254InBLS12381() {
	rec.VerifyBN254InBLS12381()
}

//export BuildGroth16
func BuildGroth16(dataDir *C.char, proof *C.char) *C.char {
	dataDirString := C.GoString(dataDir)
	proofString := C.GoString(proof)
	sp1.BuildGroth16(dataDirString, proofString)
	// err := sp1.BuildGroth16(dataDirString, proofString)
	// if err != nil {
	// 	return C.CString(err.Error())
	// }
	return nil
}

//export FreeString
func FreeString(s *C.char) {
	C.free(unsafe.Pointer(s))
}
