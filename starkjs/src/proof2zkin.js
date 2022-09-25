// copied and modified from pil-stark
module.exports.proof2zkin = function proof2zkin(p) {
    const zkin = {};
    zkin.root1 = p.root1;
    zkin.root2 = p.root2;
    zkin.root3 = p.root3;
    zkin.root4 = p.root4;
    zkin.evals = p.evals;

    const friProof = p.fri;

    for (let i=1; i<friProof.length-1; i++) {
        zkin[`s${i}_root`] = friProof[i].root;
        zkin[`s${i}_vals`] = [];
        zkin[`s${i}_siblings`] = [];
        for (let q=0; q<friProof[0].polQueries.length; q++) {
            zkin[`s${i}_vals`][q] =friProof[i].polQueries[q][0];
            zkin[`s${i}_siblings`][q] =friProof[i].polQueries[q][1];
        }
    }

    zkin.s0_vals1 = [];
    if (friProof[0].polQueries[0][1][0].length) {
        zkin.s0_vals2 = [];
    }
    if (friProof[0].polQueries[0][2][0].length) {
        zkin.s0_vals3 = [];
    }
    zkin.s0_vals4 = [];
    zkin.s0_valsC = [];
    zkin.s0_siblings1 = [];
    if (friProof[0].polQueries[0][1][0].length) {
        zkin.s0_siblings2 = [];
    }
    if (friProof[0].polQueries[0][2][0].length) {
        zkin.s0_siblings3 = [];
    }
    zkin.s0_siblings4 = [];
    zkin.s0_siblingsC = [];
/*
    zkin.s0_valsDown = [];
    zkin.s0_siblingsDownL = [];
    zkin.s0_siblingsDownH = [];

    let stepProof = friProof[0];
    zkin.s0_rootDown = stepProof.root2;
*/

    for (let i=0; i<friProof[0].polQueries.length; i++) {

        zkin.s0_vals1[i] = friProof[0].polQueries[i][0][0];
        zkin.s0_siblings1[i] = friProof[0].polQueries[i][0][1];

        if (friProof[0].polQueries[0][1][0].length) {
            zkin.s0_vals2[i] = friProof[0].polQueries[i][1][0];
            zkin.s0_siblings2[i] = friProof[0].polQueries[i][1][1];
        }
        if (friProof[0].polQueries[0][2][0].length) {
            zkin.s0_vals3[i] = friProof[0].polQueries[i][2][0];
            zkin.s0_siblings3[i] = friProof[0].polQueries[i][2][1];
        }

        zkin.s0_vals4[i] = friProof[0].polQueries[i][3][0];
        zkin.s0_siblings4[i] = friProof[0].polQueries[i][3][1];

        zkin.s0_valsC[i] = friProof[0].polQueries[i][4][0];
        zkin.s0_siblingsC[i] = friProof[0].polQueries[i][4][1];
/*
        zkin.s0_valsDown[i] = stepProof.pol2Queries[i][0];
        zkin.s0_siblingsDownL[i] = stepProof.pol2Queries[i][1][0];
        zkin.s0_siblingsDownH[i] = stepProof.pol2Queries[i][1][1];
*/
    }
/*
    const nSteps = p.length - 4;

    for (s=1; s<p[3].length-1; s++) {
        let stepProof = friProof[s];
        zkin[`s${s}_valsUp`] = [];
        zkin[`s${s}_siblingsUp`] = [];
        zkin[`s${s}_valsDown`] = [];
        zkin[`s${s}_siblingsDownL`] = [];
        zkin[`s${s}_siblingsDownH`] = [];

        zkin[`s${s}_rootDown`] = stepProof.root2;

        for (let i=0; i<stepProof.polQueries.length; i++) {

            zkin[`s${s}_valsUp`][i] = stepProof.polQueries[i][0];
            zkin[`s${s}_siblingsUp`][i] = stepProof.polQueries[i][1];

            zkin[`s${s}_valsDown`][i] = stepProof.pol2Queries[i][0];
            zkin[`s${s}_siblingsDownL`][i] = stepProof.pol2Queries[i][1][0];
            zkin[`s${s}_siblingsDownH`][i] =  stepProof.pol2Queries[i][1][1];
        }
    }

*/
    zkin.finalPol = friProof[friProof.length-1];

    return zkin;
}

module.exports.zkin2proof = function zkin2proof(zkin) {
    const p = [];
    p[0] = zkin.s0_rootUp1;
    p[1] = zkin.s0_rootUp2;
    p[2] = zkin.s0_rootUp3;
    p[3] = [];

    const pStep = {};
    pStep.root2 = zkin.s0_rootDown;
    pStep.polQueries = [];
    pStep.pol2Queries = [];
    for (let i=0; i<zkin.s0_valsUp1.length; i++) {
        pStep.polQueries[i] = [
            [ zkin.s0_valsUp1[i], zkin.s0_siblingsUp1[i] ],
            [ zkin.s0_valsUp2[i], zkin.s0_siblingsUp2[i] ],
            [ zkin.s0_valsUp3[i], zkin.s0_siblingsUp3[i] ],
            [ zkin.s0_valsUpC[i], zkin.s0_siblingsUpC[i] ],
            [ zkin.s0_valsUp1p[i], zkin.s0_siblingsUp1p[i] ],
            [ zkin.s0_valsUp2p[i], zkin.s0_siblingsUp2p[i] ],
            [ zkin.s0_valsUp3p[i], zkin.s0_siblingsUp3p[i] ],
            [ zkin.s0_valsUpCp[i], zkin.s0_siblingsUpCp[i] ]
        ];
        pStep.pol2Queries[i] = [
            zkin.s0_valsDown[i],
            [zkin.s0_siblingsDownL[i], zkin.s0_siblingsDownH[i]]
        ];
    }
    p[3].push(pStep);

    for (let s=1; typeof (zkin[`s${s}_rootDown`]) != "undefined"; s++) {
        const pStep = {};
        pStep.root2 = zkin[`s${s}_rootDown`];
        pStep.polQueries = [];
        pStep.pol2Queries = [];
        for (let i=0; i<zkin[`s${s}_valsUp`].length; i++) {
            pStep.polQueries[i] = [ zkin[`s${s}_valsUp`][i], zkin[`s${s}_siblingsUp`][i] ];

            pStep.pol2Queries[i] = [
                zkin[`s${s}_valsDown`][i],
                [zkin[`s${s}_siblingsDownL`][i], zkin[`s${s}_siblingsDownH`][i]]
            ];
        }
        p[3].push(pStep);
    }

    p[3].push(zkin.lastVals);

    return p;
}
