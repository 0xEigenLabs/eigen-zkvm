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
	"github.com/succinctlabs/sp1-recursion-gnark/rec"
)

func main() {}

//export VerifyBN254InBLS12381
func VerifyBN254InBLS12381() {
	rec.VerifyBN254InBLS12381()
}

//export FreeString
func FreeString(s *C.char) {
	C.free(unsafe.Pointer(s))
}
