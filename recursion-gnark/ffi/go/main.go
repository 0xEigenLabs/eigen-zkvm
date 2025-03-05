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

//export BuildGroth16
func BuildGroth16(dataDir *C.char, proof *C.char) {
	dataDirString := C.GoString(dataDir)
	proofString := C.GoString(proof)
	rec.BuildGroth16(dataDirString, proofString)
}

//export FreeString
func FreeString(s *C.char) {
	C.free(unsafe.Pointer(s))
}
