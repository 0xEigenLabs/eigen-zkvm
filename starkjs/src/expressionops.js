// copied and modified from pil-stark
const challengeMap = {
    "u": 0,
    "defVal": 1,
    "gamma": 2,
    "beta": 3,
    "vc": 4,
    "vf1": 5,
    "vf2": 6,
    "xi": 7
};

class ExpressionOps {

    add(a, b) {
        if (!a) return b;
        if (!b) return a;
        return {
            op: "add",
            values: [ a, b]
        }
    }

    sub(a, b) {
        if (!a) return b;
        if (!b) return a;
        return {
            op: "sub",
            values: [ a, b]
        }
    }

    mul(a, b) {
        if (!a) return b;
        if (!b) return a;
        return {
            op: "mul",
            values: [ a, b]
        }
    }

    neg(a) {
        return {
            op: "neg",
            values: [a]
        }
    }

    exp(id, next) {
        return {
            op: "exp",
            id: id,
            next: !!next
        }
    }

    cm(id, next) {
        return {
            op: "cm",
            id: id,
            next: !!next
        }
    }

    const(id, next) {
        return {
            op: "const",
            id: id,
            next: !!next
        }
    }

    q(id, next) {
        return {
            op: "q",
            id: id,
            next: !!next
        }
    }

    challenge(name) {
        if (typeof challengeMap[name] == "undefined") {
            throw new Error("challenge not defined "+name);
        }
        return {
            op: "challenge",
            id: challengeMap[name]
        };
    }

    number(n) {
        return {
            op: "number",
            value: BigInt(n)
        }
    }

    eval(n) {
        return {
            op: "eval",
            id: n
        }
    }

    tmp(n) {
        return {
            op: "tmp",
            id: n
        }
    }

    xDivXSubXi() {
        return {
            op: "xDivXSubXi"
        }
    }

    xDivXSubWXi() {
        return {
            op: "xDivXSubWXi"
        }
    }

    x() {
        return {
            op: "x"
        }
    }

}

module.exports = ExpressionOps;
