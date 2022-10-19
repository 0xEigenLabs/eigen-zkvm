const { h4toString, h4toScalar } = require("@0xpolygonhermez/zkevm-commonjs").smtUtils;
const Scalar = require("ffjavascript").Scalar;

const LOG_STORAGE_EXECUTOR = false;

function scalar2fea4(Fr, scalar) {

    function scalarltPrime (r) {
        if (r>=Fr.p) {
            logger("Error: scalar2fea4() found value higher than prime: " + r.toString(16));
            return false;
        } else return true;
    }

    scalar = Scalar.e(scalar);
    const r0 = Scalar.band(scalar, Scalar.e('0xFFFFFFFFFFFFFFFF'));
    if (!scalarltPrime(r0)) return;
    const r1 = Scalar.band(Scalar.shr(scalar, 32), Scalar.e('0xFFFFFFFFFFFFFFFF'));
    if (!scalarltPrime(r1)) return;
    const r2 = Scalar.band(Scalar.shr(scalar, 64), Scalar.e('0xFFFFFFFFFFFFFFFF'));
    if (!scalarltPrime(r2)) return;
    const r3 = Scalar.band(Scalar.shr(scalar, 96), Scalar.e('0xFFFFFFFFFFFFFFFF'));
    if (!scalarltPrime(r3)) return;

    return [Fr.e(r0), Fr.e(r1), Fr.e(r2), Fr.e(r3)];
}

function fea42String10(Fr, fea4) {
    const sc = h4toScalar(fea4);
    return sc.toString();
}

function fea42String(Fr, fea4) {
    const sc = h4toScalar(fea4);
    return `${Scalar.toString(sc, 16).padStart(64, '0')}`;
}

function fea4IsEq(Fr, f1, f2) {
    return (Fr.eq(f1[0], f2[0])
        && Fr.eq(f1[1], f2[1])
        && Fr.eq(f1[2], f2[2])
        && Fr.eq(f1[3], f2[3]));
}

function logger(m) {
    if (LOG_STORAGE_EXECUTOR) console.log(m);
}

const isLogging = LOG_STORAGE_EXECUTOR;

module.exports = {isLogging, logger, fea42String, fea42String10, scalar2fea4, fea4IsEq};