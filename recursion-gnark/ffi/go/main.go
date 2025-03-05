package main

/*
#include <stdlib.h>

*/
import "C"
import (
	"unsafe"

	"github.com/0xEigenLabs/eigen-recursion-gnark/rec"
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
	rec.BuildGroth16(dataDirString, proofString)
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
