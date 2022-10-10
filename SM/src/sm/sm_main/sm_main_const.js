

module.exports = async function (pols) {

    const N = pols.STEP.length;

    for ( let i=0; i<N; i++) {
        pols.STEP[i] = BigInt(i);
    }
}









