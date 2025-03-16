package main

/*
#include <stdlib.h>

*/
import "C"
import (
	"unsafe"

	"github.com/0xEigenLabs/eigen-recursion-gnark/eigen"
)

func main() {}

//export BuildGroth16
func BuildGroth16(vkPath *C.char, outputDir *C.char, proof *C.char, publicInputJson *C.char) {
	vkPathString := C.GoString(vkPath)
	outputDirString := C.GoString(outputDir)
	proofString := C.GoString(proof)
	publicInputJsonString := C.GoString(publicInputJson)
	eigen.BuildGroth16(vkPathString, outputDirString, proofString, publicInputJsonString)
}

//export FreeString
func FreeString(s *C.char) {
	C.free(unsafe.Pointer(s))
}
