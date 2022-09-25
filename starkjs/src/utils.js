module.exports = {
  // Security parameter:
  // * Size of the prime field q
  // * Length of the exectuion trace t
  // * Maximum degree of constraints d
  // * Domain blowup factor b
  // * Collision resistance of the hash function c
  // * Nummber of queries n
  // Security Level(bits):
  //   min(log2(q/(t*b)), log2(b/d)*n, c)
  security_test(starkStruct, execution_trace) {
    let q = 64;
    if (starkStruct.verificationHashType == "BN128") {
      q = 254;
    }

    //is the N in the fibonacci sequence example
    let t = Math.log2(execution_trace);

    let d = starkStruct.nBits;
    let b = starkStruct.nBitsExt;

    let c = 128; //https://www.poseidon-hash.info/
    let n = starkStruct.nQueries;

    console.log(d,b,n,q-t-b, (b-d)*n, c);

    return Math.min(q - t - b, (b - d) * n, c)
  },
  log2( V )
  {
    return( ( ( V & 0xFFFF0000 ) !== 0 ? ( V &= 0xFFFF0000, 16 ) : 0 ) | ( ( V & 0xFF00FF00 ) !== 0 ? ( V &= 0xFF00FF00, 8 ) : 0 ) | ( ( V & 0xF0F0F0F0 ) !== 0 ? ( V &= 0xF0F0F0F0, 4 ) : 0 ) | ( ( V & 0xCCCCCCCC ) !== 0 ? ( V &= 0xCCCCCCCC, 2 ) : 0 ) | ( ( V & 0xAAAAAAAA ) !== 0 ) );
  }
}