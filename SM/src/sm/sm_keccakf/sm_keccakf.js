const { assert } = require("console");
const { fstat } = require("fs");
const fs = require("fs");
const { connect } = require("http2");
const path = require("path");
const { log2 } = require("@0xpolygonhermez/zkevm-commonjs").utils;
const { F1Field } = require("ffjavascript");
const getKs = require("pilcom").getKs;

const SlotSize = 155286;
const chunks = 4;
const chunkBits = 11n;

module.exports.buildConstants = async function (pols) {
    const N = pols.ConnA.length;


    const F = new F1Field("0xFFFFFFFF00000001");

    const nSlots = Math.floor((N-1)/SlotSize);

    const conns = JSON.parse(await fs.promises.readFile(path.join(__dirname, "keccak_connections.json"), "utf8"));
    const script = JSON.parse(await fs.promises.readFile(path.join(__dirname, "keccak_script.json"), "utf8"));

    assert(conns.length == script.program.length +1);

    const pow = log2(N);
    assert(1<<pow == N);

    const ks = getKs(F, 2);

    let w = F.one;
    for (let i=0; i<N; i++) {
        pols.ConnA[i] = w;
        pols.ConnB[i] = F.mul(w, ks[0]);
        pols.ConnC[i] = F.mul(w, ks[1]);
        w = F.mul(w, F.FFT.w[pow]);
    }

    pols.GateType[0] = 0n

    for (let i=0; i<nSlots; i++) {
        const offset = i*SlotSize;

        for (let j=0; j<conns.length; j++) {
            const l = conns[j];
            let r1 = j;
            if (j>0) r1 += offset;
            if (l.A) {
                for (let k=0; k<l.A.length; k++) {
                    const ll = conns[j].A[k];
                    let r2 = ll[1];
                    if (r2>0) r2 += offset;
                    if (ll[0] == "A") {
                        connect(pols.ConnA, r1, pols.ConnA, r2);
                    } else if (ll[0] == "B") {
                        connect(pols.ConnA, r1, pols.ConnB, r2);
                    } else if (ll[0] == "C") {
                        connect(pols.ConnA, r1, pols.ConnC, r2);
                    } else {
                        assert(false, "Invalid pin");
                    }
                }
            }
            if (l.B) {
                for (let k=0; k<l.B.length; k++) {
                    const ll = conns[j].B[k];
                    let r2 = ll[1];
                    if (r2>0) r2 += offset;
                    if (ll[0] == "A") {
                        connect(pols.ConnB, r1, pols.ConnA, r2);
                    } else if (ll[0] == "B") {
                        connect(pols.ConnB, r1, pols.ConnB, r2);
                    } else if (ll[0] == "C") {
                        connect(pols.ConnB, r1, pols.ConnC, r2);
                    } else {
                        assert(false, "Invalid pin");
                    }
                }
            }
            if (l.C) {
                for (let k=0; k<l.C.length; k++) {
                    const ll = conns[j].C[k];
                    let r2 = ll[1];
                    if (r2>0) r2 += offset;
                    if (ll[0] == "A") {
                        connect(pols.ConnC, r1, pols.ConnA, r2);
                    } else if (ll[0] == "B") {
                        connect(pols.ConnC, r1, pols.ConnB, r2);
                    } else if (ll[0] == "C") {
                        connect(pols.ConnC, r1, pols.ConnC, r2);
                    } else {
                        assert(false, "Invalid pin");
                    }
                }
            }
        }
        for (let j=0; j<script.program.length; j++) {
            const lp = script.program[j];
            r1 = lp.ref;
            if (r1>0) r1 += offset;
            if (lp.op == "xor") {
                pols.GateType[r1] = 0n;
            } else if (lp.op == "andp") {
                pols.GateType[r1] = 1n;
            } else {
                assert(false, "Invalid op");
            }
        }
    }

    for (let k=1 + nSlots*SlotSize; k<N; k++) {
        pols.GateType[k] = 0n;
    }

    const mask = (2n**chunkBits)-1n;
    let c = 0;
    for (let a=0n; a<2n**chunkBits; a++) {
        for (let b=0n; b<2n**chunkBits; b++) {
            pols.kGateType[c] = 0n;
            pols.kA[c] = a;
            pols.kB[c] = b;
            pols.kC[c] = a^b;
            c++;
            pols.kGateType[c] = 1n;
            pols.kA[c] = a;
            pols.kB[c] = b;
            pols.kC[c] = (a^mask)&b;
            c++;
        }
    }
    while (c<N)
    {
        pols.kGateType[c] = 0n;
        pols.kA[c] = 0n;
        pols.kB[c] = 0n;
        pols.kC[c] = 0n;
        c++;
    }

    console.log("keccak build constants done");

    function connect(p1, i1, p2, i2) {
        [p1[i1], p2[i2]] = [p2[i2], p1[i1]];
    }
}

module.exports.execute = async function (pols, input) {
    const N = pols.a[0].length;

    const script = JSON.parse(await fs.promises.readFile(path.join(__dirname, "keccak_script.json"), "utf8"));

    let c_xor=0;
    let c_andp=0;

    assert(script.program.length == SlotSize);

    const nSlots = Math.floor((N-1)/SlotSize);

    for (let ichunk=0; ichunk < chunks; ++ichunk) {
        pols.a[ichunk][0] = 0n;
        pols.b[ichunk][0] = 0x7FFn;
        pols.c[ichunk][0] = 0x7FFn;
    }

    let p=1;
    let offset = 0;

    for (let i=0; i<nSlots; i++) {
        for (let j=0; j<SlotSize; j++) {
            const l = script.program[j];
            const r = l.ref + i*SlotSize;

            if (l.a.type === "input") {
                setPol(pols.a, r, input[i][l.a.bit]);
            } else if (l.a.type === "wired") {
                let g = l.a.gate;
                if (g>0) g+=offset;
                if (l.a.pin=="a") {
                    setPol(pols.a, r, getPol(pols.a,g));
                } else if (l.a.pin=="b") {
                    setPol(pols.a, r, getPol(pols.b,g));
                } else if (l.a.pin=="c") {
                    setPol(pols.a, r, getPol(pols.c,g));
                } else {
                    assert(false, "Invalid pin");
                }
            } else {
                assert(false, "Invalid field type");
            }

            if (l.b.type === "input") {
                setPol(pols.b, r, input[i][l.b.bit]);
            } else if (l.b.type === "wired") {
                let g = l.b.gate;
                if (g>0) g+=offset;
                if (l.b.pin=="a") {
                    setPol(pols.b, r, getPol(pols.a,g));
                } else if (l.b.pin=="b") {
                    setPol(pols.b, r, getPol(pols.b,g));
                } else if (l.b.pin=="c") {
                    setPol(pols.b, r, getPol(pols.c,g));
                } else {
                    assert(false, "Invalid pin");
                }
            } else {
                assert(false, "Invalid field type");
            }

            const mask = 0xFFFFFFFFFFFn;
            if (l.op === "xor") {
                setPol(pols.c, r,(getPol(pols.a,r) & mask) ^ (getPol(pols.b,r) &  mask));
            } else if (l.op === "andp") {
                setPol(pols.c, r, ((getPol(pols.a,r)  &  mask) ^ mask) & (getPol(pols.b,r)  &  mask));
            }

        }

        offset += SlotSize;
    }

    for (let i= 1+ nSlots*SlotSize; i<N; i++) {
        for (let ichunk=0; ichunk < chunks; ++ichunk) {
            pols.a[ichunk][i] = 0n;
            pols.b[ichunk][i] = 0n;
            pols.c[ichunk][i] = 0n;
        }
    }

    function setPol(pol, index, value)
    {
        pol[0][index] = value & 0x7FFn;
        value = value >> 11n;
        pol[1][index] = value & 0x7FFn;
        value = value >> 11n;
        pol[2][index] = value & 0x7FFn;
        value = value >> 11n;
        pol[3][index] = value & 0x7FFn;
    }

    function getPol(pol, index)
    {
       return 2n**33n * pol[3][index] + 2n**22n * pol[2][index] + 2n**11n * pol[1][index] + pol[0][index];
    }
}