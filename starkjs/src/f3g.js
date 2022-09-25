// copied and modified from pil-stark
/*

This is a field extension 3 of the goldilocks:

Prime: 0xFFFFFFFF00000001
Irreducible polynomial: x^3 - x -1

*/
const crypto = require("crypto");
const buildFFT = require("./fft.js");
const buildSqrt = require("./sqrt.js");

module.exports = class F3G {

    constructor() {
        this.p = 0xFFFFFFFF00000001n
        this.zero = 0n;
        this.one = 1n;
        this.nqr = 7n;
        this.shift = 7n;
        this.shiftInv = this.inv(this.shift);
        this.half = 0xFFFFFFFF00000001n >> 1n;
        this.negone = 0xFFFFFFFF00000000n;
        this.k = 12275445934081160404n;  // 7^(2^32) => Generetor of the group 3x5x17x257x65537
        this.s = 32;
        this.t = (this.p-1n) / BigInt(2**this.s);
        this.n8 = 8;
        this.n32 = 2;
        this.n64 = 1;

        this.m = 3;

        this.bitLength = 0;
        for (let a =this.p; a>0n; a = a >> 1n) this.bitLength += 1;

        buildSqrt(this);

        buildFFT(this);
    }

    add(a, b) {
        if (typeof(a) == "bigint") {
            if (typeof(b) == "bigint") {
                return (a+b) % this.p
            } else {
                return [(a+b[0]) % this.p, b[1], b[2]];
            }
        } else if (typeof(b) == "bigint") {
            return [(a[0]+b) % this.p, a[1], a[2]];
        } else {
            return [(a[0]+b[0]) % this.p, (a[1]+b[1]) % this.p, (a[2]+b[2]) % this.p];
        }
    }

    sub(a, b) {
        if (typeof(a) == "bigint") {
            if (typeof(b) == "bigint") {
                return (a >= b) ? a-b : this.p-b+a;
            } else {
                return [(a >= b[0]) ? a-b[0] : this.p-b[0]+a, b[1] > 0n ? this.p-b[1] : b[1], b[2] > 0n ? this.p-b[2] : b[2]];
            }
        } else if (typeof(b) == "bigint") {
            return [(a[0] >= b) ? a[0]-b : this.p-b+a[0], a[1], a[2]];
        } else {
            return [(a[0] >= b[0]) ? a[0]-b[0] : this.p-b[0]+a[0], (a[1] >= b[1]) ? a[1]-b[1] : this.p-b[1]+a[1], (a[2] >= b[2]) ? a[2]-b[2] : this.p-b[2]+a[2]];
        }
    }

    neg(a) {
        if (typeof(a) == "bigint") {
            return a > 0n ? this.p-a : a;
        } else {
            return [a[0] > 0n ? this.p-a[0] : a[0], a[1] > 0n ? this.p-a[1] : a[1], a[2] > 0n ? this.p-a[2] : a[2]];
        }
    }


    mul(a, b) {
        if (typeof(a) == "bigint") {
            if (typeof(b) == "bigint") {
                return (a*b) % this.p;
            } else {
                return [(a*b[0]) % this.p,  (a*b[1]) % this.p, (a*b[2]) % this.p];
            }
        } else if (typeof(b) == "bigint") {
            return [(a[0]*b) % this.p,  (a[1]*b) % this.p, (a[2]*b) % this.p];
        } else {
            const A = (a[0] + a[1])  * (b[0] + b[1]);
            const B = (a[0] + a[2])  * (b[0] + b[2]);
            const C = (a[1] + a[2])  * (b[1] + b[2]);
            const D = a[0]*b[0];
            const E = a[1]*b[1];
            const F = a[2]*b[2];
            const G = D - E;

            return [ (C + G - F)%this.p,  (A + C - E -E - D )%this.p,(B-G)%this.p ];
        }
    }


    mulScalar(a, b) {
        b = BigInt(b);
        if (typeof(a) == "bigint") {
            return (a*b) % this.p;
        } else {
            return [(a[0]*b) % this.p,  (a[1]*b) % this.p, (a[2]*b) % this.p];
        }
    }

    square(a) {
        if (typeof(a) == "bigint") {
            return (a*a) % this.p;
        } else {
            const A = (a[0] + a[1])  * (a[0] + a[1]);
            const B = (a[0] + a[2])  * (a[0] + a[2]);
            const C = (a[1] + a[2])  * (a[1] + a[2]);
            const D = a[0]*a[0];
            const E = a[1]*a[1];
            const F = a[2]*a[2];
            const G = D - E;

            return [ (C + G - F)%this.p,  (A + C - E -E - D )%this.p,(B-G)%this.p ];
        }

    }

    /*
        Formula deducted here: https://www.polymathlove.com/polymonials/midpoint-of-a-line/symbolic-equation-solving.html#c=solve_algstepsequationsolvesystem&v247=d%252Ce%252Cf&v248=3&v249=f*a%2Bb*e%2Bd*c%2B%2520c*f%2520%253D%25200&v250=d*b%2Be*a%2Bc*f%2Bb*f%2Be*c%253D0&v251=a*d%2Bb*f%2Be*c%253D1
    */
    inv(a) {
        if (typeof(a) == "bigint") {
            return this._inv1(a);
        } else {
            const aa = a[0] * a[0];
            const ac = a[0] * a[2];
            const ba = a[1] * a[0];
            const bb = a[1] * a[1];
            const bc = a[1] * a[2];
            const cc = a[2] * a[2];

            const aaa = aa * a[0];
            const aac = aa * a[2];
            const abc = ba * a[2];
            const abb = ba * a[1];
            const acc = ac * a[2];
            const bbb = bb * a[1];
            const bcc = bc * a[2];
            const ccc = cc * a[2];

            let t = (-aaa -aac-aac +abc+abc+abc + abb - acc - bbb + bcc - ccc)%this.p;

            if (t<0n) t = t + this.p;

            const tinv = this._inv1(t);

            let i1 = ((-aa -ac-ac +bc + bb - cc)*tinv) % this.p;
            let i2 = ((ba -cc)*tinv) % this.p;
            let i3 = ((-bb +ac + cc)*tinv) % this.p;

            if (i1<0) i1 = this.p+i1;
            if (i2<0) i2 = this.p+i2;
            if (i3<0) i3 = this.p+i3;

            return [i1, i2, i3];
        }
    }

    _inv1(a) {
        if (!a) throw new Error("Division by zero");

        let t = this.zero;
        let r = this.p;
        let newt = this.one;
        let newr = a % this.p;
        while (newr) {
            let q = r/newr;
            [t, newt] = [newt, t-q*newt];
            [r, newr] = [newr, r-q*newr];
        }
        if (t<this.zero) t += this.p;
        return t;
    }

    div(a,b) {
        return this.mul(a, this.inv(b));
    }

    eq(a, b) {
        if (typeof(a) == "bigint") {
            if (typeof(b) == "bigint") {
                return a == b;
            } else {
                return (a == b[0]) && (b[1]== 0)  && (b[2]==0);
            }
        } else if (typeof(b) == "bigint") {
            return (a[0] == b) && (a[1]== 0)  && (a[2]==0);
        } else {
            return (a[0] == b[0]) && (a[1]== b[1])  && (a[2]==b[2]);
        }
    }

    gt(a, b) {
        const self = this;
        a = norm(a);
        b = norm(b);
        if (typeof(a) == "bigint") {
            if (typeof(b) == "bigint") {
                return a > b;
            } else {
                return a > b[0];
            }
        } else if (typeof(b) == "bigint") {
            return (a[0] > b) ||
                   ((a[0] == b) && (a[1] > 0n)) ||
                   ((a[0] == b) && (a[1] == 0n) && a[2] >0n)
        } else {
            return  (a[0] > b[0]) ||
                    ((a[0] == b[0]) && (a[1] > b[1])) ||
                    ((a[0] == b[0]) && (a[1] == b[1]) && a[2] >b[2])
        }

        function norm(a) {
            if (typeof(a) == "bigint") {
                return [norm1(a), 0n, 0n];
            } else {
                return [norm1(a[0]), norm1(a[1]), norm1(a[2])];
            }
        }

        function norm1(a) {
            return  (a > (self.half)) ? a-self.p : a;
        }

    }


    geq(a, b) {
        return this.gt(a, b) || this.eq(a, b);
    }

    lt(a, b) {
        return !this.geq(a,b);
    }

    leq(a, b) {
        return !this.gt(a,b);
    }

    neq(a, b) {
        return !this.eq(a,b);
    }


    isZero(a) {
        if (typeof(a) == "bigint") {
            return a == 0n;
        } else {
            return (a[0] == 0n) && (a[1]== 0n)  && (a[2]==0n);
        }
    }

    e(a,b) {
        if (Array.isArray(a)) {
            return [this.e(a[0],b), this.e(a[1],b), this.e(a[2],b)];
        }
        let res;
        if (!b) {
            res = BigInt(a);
        } else if (b==16) {
            res = BigInt("0x"+a);
        }
        if (res < 0) {
            let nres = -res;
            if (nres >= this.p) nres = nres % this.p;
            return this.p - nres;
        } else {
            return (res>= this.p) ? res%this.p : res;
        }
    }

    exp(base, e) {
        e = BigInt(e);
        if (e === 0n) return this.one;

        const n = this._bits(e);

        if (n.length==0) return this.one;

        let res = base;

        for (let i=n.length-2; i>=0; i--) {

            res = this.square(res);

            if (n[i]) {
                res = this.mul(res, base);
            }
        }

        return res;
    }

    pow(base, e) {
        return this.exp(base, e);
    }

    toString(a, base) {
        base = base || 10;
        if (typeof(a) == "bigint") {
            return a.toString(base);
        } else {
            return [this.toString(a[0], base), this.toString(a[1], base), this.toString(a[2], base)]
        }
    }

    _bits(n) {
        let E = BigInt(n);
        const res = [];
        while (E) {
            if (E & 1n) {
                res.push(1);
            } else {
                res.push( 0 );
            }
            E = E >> 1n;
        }
        return res;
    }

    random() {
        return [this._random1(), this._random1(), this._random1()];
    }

    _random1() {
        const nBytes = (this.bitLength*2 / 8);
        let res =this.zero;
        for (let i=0; i<nBytes; i++) {
            res = (res << BigInt(8)) + BigInt(this._getRandomBytes(1)[0]);
        }
        return res % this.p;
    }

    _getRandomBytes(n) {
        let array = new Uint8Array(n);
        if (process.browser) { // Browser
            if (typeof globalThis.crypto !== "undefined") { // Supported
                globalThis.crypto.getRandomValues(array);
            } else { // fallback
                for (let i=0; i<n; i++) {
                    array[i] = (Math.random()*4294967296)>>>0;
                }
            }
        }
        else { // NodeJS
            crypto.randomFillSync(array);
        }
        return array;
    }

    batchInverse(a) {
        if (a.length == 0) return [];
        const tmp = [];
        tmp[0] = a[0];
        for (let i=1; i<a.length; i++) {
            tmp[i] = this.mul(tmp[i-1],a[i]);
        }
        let z = this.inv(tmp[tmp.length-1]);
        const res = new Array(a.length);
        for (let i=a.length-1; i>0; i--) {
            res[i] = this.mul(z, tmp[i-1]);
            z = this.mul(z, a[i]);
        }
        res[0] = z;
        return res;
    }

    fromRprLE(buff, o) {
        if (o & 7 == 0) {
            const v = new BigUint64Array(buff.buffer, o || 0, 1);
            return v[0];
        } else if ((o & 3)==0) {
            const v = new Uint32Array(buff.buffer, o || 0, 2);
            return BigInt(v[0]) |  (BigInt(v[1]) << 32n);
        } else if ((o & 1)==0) {
            const v = new Uint16Array(buff.buffer, o || 0, 8);
            return   BigInt(v[0])         |
                    (BigInt(v[1]) << 16n) |
                    (BigInt(v[2]) << 32n) |
                    (BigInt(v[3]) << 48n);
        } else {
            const v = new Uint8Array(buff.buffer, o || 0, 8);
            return   BigInt(v[0])         |
                    (BigInt(v[1]) <<  8n) |
                    (BigInt(v[2]) << 16n) |
                    (BigInt(v[3]) << 24n) |
                    (BigInt(v[4]) << 32n) |
                    (BigInt(v[5]) << 40n) |
                    (BigInt(v[6]) << 48n) |
                    (BigInt(v[7]) << 56n);
        }
    }

}


