const SlotSize = 158418;


module.exports.buildConstants = async function (pols) {
    const N = pols.Field9latch.length;

    const nSlots9 = Math.floor((N-1)/SlotSize);

    for (i=0; i<N; i++) {
        let slot = -1;
        let iRel;
        if (i>0) {
            slot = Math.floor( (i - 1) / SlotSize );
            if (slot < nSlots9) {
                iRel = (i-1) % SlotSize;
            } else {
                slot = -1;
            }
        }
        if (slot >= 0) {
            if ( ((iRel%9) == 0) && (iRel<=9*3200) && (iRel>0) ) {
                pols.Field9latch[i] = 1n;
            } else {
                pols.Field9latch[i] = 0n;
            }
            if (iRel < 9*3200) {
                pols.Factor[i] = 1n << BigInt(7 * (iRel%9));
            } else {
                pols.Factor[i] = 0n;
            }
        } else {
            pols.Field9latch[i] = 0n;
            pols.Factor[i] = 0n;
        }

    }

}

module.exports.execute = async function (pols, input) {

    const required = {
        KeccakF: []
    };

    const N = pols.bit.length;

    const nSlots9 = Math.floor((N-1)/SlotSize);

    let p=0;

    pols.bit[p] = 0n;
    pols.field9[p] = 0n;
    p += 1;

    let accField9 = 0n;


    for (let i=0; i<nSlots9; i++) {
        const keccakFSlot = [];
        for (j=0; j<1600; j++) {
            for (k=0; k<9; k++) {
                pols.bit[p] = getBit(i*9+k, false, j);
                pols.field9[p] = accField9;
                accField9 = k==0 ? pols.bit[p] :
                    accField9 +  (pols.bit[p] << (7n * BigInt(k)));
                p += 1;
            }
            keccakFSlot.push(accField9);
        }
        for (j=0; j<1600; j++) {
            for (k=0; k<9; k++) {
                pols.bit[p] = getBit(i*9+k, true, j);
                pols.field9[p] = accField9;
                accField9 = k==0 ? pols.bit[p] :
                    accField9 +  (pols.bit[p] << (7n * BigInt(k)));
                p += 1;
            }
        }

        pols.bit[p] = 0n;
        pols.field9[p] = accField9;
        accField9 = 0n;
        p += 1;

        for (j=3200*9+1; j<SlotSize; j++) {
            pols.bit[p] = 0n;
            pols.field9[p] = 0n;
            p += 1;
        }

        required.KeccakF.push(keccakFSlot);
    }

    while (p<N) {
        pols.bit[p] = 0n;
        pols.field9[p] = 0n;
        p += 1;
    }

    return required;


    function getBit(block, isOut, pos) {
        if (block>=input.length) return 0n;
        const st = isOut ? input[block][1] : input[block][0]
        return BigInt(bitFromState(st, pos ));
    }
}

function bitFromState(st, i) {

    const y = Math.floor(i / 320);
    const x = Math.floor((i % 320) / 64);
    const z = i % 64
    const z1 = Math.floor(z / 32);
    const z2 = z%32;

    return BigInt((st[x][y][z1] >> z2) & 1);

}
