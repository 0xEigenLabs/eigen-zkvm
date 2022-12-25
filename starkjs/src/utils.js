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
    let field_security = q - this.trailing_zeros(b);

    let security_per_query = b - d;
    let query_security = security_per_query * starkStruct.nQueries;

    let c = 128; //https://www.poseidon-hash.info/

    return Math.min(field_security, query_security, c)
  },

  trailing_zeros(v) {
    var c = 32
    v &= -v
    if (v) c--
    if (v & 0x0000FFFF) c -= 16
    if (v & 0x00FF00FF) c -= 8
    if (v & 0x0F0F0F0F) c -= 4
    if (v & 0x33333333) c -= 2
    if (v & 0x55555555) c -= 1
    return c
  },

  log2( V )
  {
    return( ( ( V & 0xFFFF0000 ) !== 0 ? ( V &= 0xFFFF0000, 16 ) : 0 ) | ( ( V & 0xFF00FF00 ) !== 0 ? ( V &= 0xFF00FF00, 8 ) : 0 ) | ( ( V & 0xF0F0F0F0 ) !== 0 ? ( V &= 0xF0F0F0F0, 4 ) : 0 ) | ( ( V & 0xCCCCCCCC ) !== 0 ? ( V &= 0xCCCCCCCC, 2 ) : 0 ) | ( ( V & 0xAAAAAAAA ) !== 0 ) );
  },

  elapse(phase, res) {
    var end = new Date().getTime()
    var cost = 0;
    var total = 0;
    if (res.length > 0) {
      cost = end - res[res.length - 1][3];
      total = end - res[0][3];
    }
    console.log(phase, cost/1000, total/1000);
    res.push([phase, cost/1000, total/1000, end]);
  },

  buildConstantsGlobal(pols) {
    const N = pols.L1.length;
    for ( let i=0; i<N; i++) {
      pols.L1[i] = (i == 0)? 1n : 0n;
    }
  }
}
