const F1Field = require("ffjavascript").F1Field;
const t = 12;
const nRoundsF = 8;
const nRoundsP = 22;

const C = [
    0xb585f766f2144405n, 0x7746a55f43921ad7n, 0xb2fb0d31cee799b4n, 0x0f6760a4803427d7n,
    0xe10d666650f4e012n, 0x8cae14cb07d09bf1n, 0xd438539c95f63e9fn, 0xef781c7ce35b4c3dn,
    0xcdc4a239b0c44426n, 0x277fa208bf337bffn, 0xe17653a29da578a1n, 0xc54302f225db2c76n,
    0x86287821f722c881n, 0x59cd1a8a41c18e55n, 0xc3b919ad495dc574n, 0xa484c4c5ef6a0781n,
    0x308bbd23dc5416ccn, 0x6e4a40c18f30c09cn, 0x9a2eedb70d8f8cfan, 0xe360c6e0ae486f38n,
    0xd5c7718fbfc647fbn, 0xc35eae071903ff0bn, 0x849c2656969c4be7n, 0xc0572c8c08cbbbadn,
    0xe9fa634a21de0082n, 0xf56f6d48959a600dn, 0xf7d713e806391165n, 0x8297132b32825dafn,
    0xad6805e0e30b2c8an, 0xac51d9f5fcf8535en, 0x502ad7dc18c2ad87n, 0x57a1550c110b3041n,
    0x66bbd30e6ce0e583n, 0x0da2abef589d644en, 0xf061274fdb150d61n, 0x28b8ec3ae9c29633n,
    0x92a756e67e2b9413n, 0x70e741ebfee96586n, 0x019d5ee2af82ec1cn, 0x6f6f2ed772466352n,
    0x7cf416cfe7e14ca1n, 0x61df517b86a46439n, 0x85dc499b11d77b75n, 0x4b959b48b9c10733n,
    0xe8be3e5da8043e57n, 0xf5c0bc1de6da8699n, 0x40b12cbf09ef74bfn, 0xa637093ecb2ad631n,
    0x3cc3f892184df408n, 0x2e479dc157bf31bbn, 0x6f49de07a6234346n, 0x213ce7bede378d7bn,
    0x5b0431345d4dea83n, 0xa2de45780344d6a1n, 0x7103aaf94a7bf308n, 0x5326fc0d97279301n,
    0xa9ceb74fec024747n, 0x27f8ec88bb21b1a3n, 0xfceb4fda1ded0893n, 0xfac6ff1346a41675n,
    0x7131aa45268d7d8cn, 0x9351036095630f9fn, 0xad535b24afc26bfbn, 0x4627f5c6993e44ben,
    0x645cf794b8f1cc58n, 0x241c70ed0af61617n, 0xacb8e076647905f1n, 0x3737e9db4c4f474dn,
    0xe7ea5e33e75fffb6n, 0x90dee49fc9bfc23an, 0xd1b1edf76bc09c92n, 0x0b65481ba645c602n,
    0x99ad1aab0814283bn, 0x438a7c91d416ca4dn, 0xb60de3bcc5ea751cn, 0xc99cab6aef6f58bcn,
    0x69a5ed92a72ee4ffn, 0x5e7b329c1ed4ad71n, 0x5fc0ac0800144885n, 0x32db829239774ecan,
    0x0ade699c5830f310n, 0x7cc5583b10415f21n, 0x85df9ed2e166d64fn, 0x6604df4fee32bcb1n,
    0xeb84f608da56ef48n, 0xda608834c40e603dn, 0x8f97fe408061f183n, 0xa93f485c96f37b89n,
    0x6704e8ee8f18d563n, 0xcee3e9ac1e072119n, 0x510d0e65e2b470c1n, 0xf6323f486b9038f0n,
    0x0b508cdeffa5ceefn, 0xf2417089e4fb3cbdn, 0x60e75c2890d15730n, 0xa6217d8bf660f29cn,
    0x7159cd30c3ac118en, 0x839b4e8fafead540n, 0x0d3f3e5e82920adcn, 0x8f7d83bddee7bba8n,
    0x780f2243ea071d06n, 0xeb915845f3de1634n, 0xd19e120d26b6f386n, 0x016ee53a7e5fecc6n,
    0xcb5fd54e7933e477n, 0xacb8417879fd449fn, 0x9c22190be7f74732n, 0x5d693c1ba3ba3621n,
    0xdcef0797c2b69ec7n, 0x3d639263da827b13n, 0xe273fd971bc8d0e7n, 0x418f02702d227ed5n,
    0x8c25fda3b503038cn, 0x2cbaed4daec8c07cn, 0x5f58e6afcdd6ddc2n, 0x284650ac5e1b0eban,
    0x635b337ee819dab5n, 0x9f9a036ed4f2d49fn, 0xb93e260cae5c170en, 0xb0a7eae879ddb76dn,
    0xd0762cbc8ca6570cn, 0x34c6efb812b04bf5n, 0x40bf0ab5fa14c112n, 0xb6b570fc7c5740d3n,
    0x5a27b9002de33454n, 0xb1a5b165b6d2b2d2n, 0x8722e0ace9d1be22n, 0x788ee3b37e5680fbn,
    0x14a726661551e284n, 0x98b7672f9ef3b419n, 0xbb93ae776bb30e3an, 0x28fd3b046380f850n,
    0x30a4680593258387n, 0x337dc00c61bd9ce1n, 0xd5eca244c7a4ff1dn, 0x7762638264d279bdn,
    0xc1e434bedeefd767n, 0x0299351a53b8ec22n, 0xb2d456e4ad251b80n, 0x3e9ed1fda49cea0bn,
    0x2972a92ba450bed8n, 0x20216dd77be493den, 0xadffe8cf28449ec6n, 0x1c4dbb1c4c27d243n,
    0x15a16a8a8322d458n, 0x388a128b7fd9a609n, 0x2300e5d6baedf0fbn, 0x2f63aa8647e15104n,
    0xf1c36ce86ecec269n, 0x27181125183970c9n, 0xe584029370dca96dn, 0x4d9bbc3e02f1cfb2n,
    0xea35bc29692af6f8n, 0x18e21b4beabb4137n, 0x1e3b9fc625b554f4n, 0x25d64362697828fdn,
    0x5a3f1bb1c53a9645n, 0xdb7f023869fb8d38n, 0xb462065911d4e1fcn, 0x49c24ae4437d8030n,
    0xd793862c112b0566n, 0xaadd1106730d8febn, 0xc43b6e0e97b0d568n, 0xe29024c18ee6fca2n,
    0x5e50c27535b88c66n, 0x10383f20a4ff9a87n, 0x38e8ee9d71a45af8n, 0xdd5118375bf1a9b9n,
    0x775005982d74d7f7n, 0x86ab99b4dde6c8b0n, 0xb1204f603f51c080n, 0xef61ac8470250ecfn,
    0x1bbcd90f132c603fn, 0x0cd1dabd964db557n, 0x11a3ae5beb9d1ec9n, 0xf755bfeea585d11dn,
    0xa3b83250268ea4d7n, 0x516306f4927c93afn, 0xddb4ac49c9efa1dan, 0x64bb6dec369d4418n,
    0xf9cc95c22b4c1fccn, 0x08d37f755f4ae9f6n, 0xeec49b613478675bn, 0xf143933aed25e0b0n,
    0xe4c5dd8255dfc622n, 0xe7ad7756f193198en, 0x92c2318b87fff9cbn, 0x739c25f8fd73596dn,
    0x5636cac9f16dfed0n, 0xdd8f909a938e0172n, 0xc6401fe115063f5bn, 0x8ad97b33f1ac1455n,
    0x0c49366bb25e8513n, 0x0784d3d2f1698309n, 0x530fb67ea1809a81n, 0x410492299bb01f49n,
    0x139542347424b9acn, 0x9cb0bd5ea1a1115en, 0x02e3f615c38f49a1n, 0x985d4f4a9c5291efn,
    0x775b9feafdcd26e7n, 0x304265a6384f0f2dn, 0x593664c39773012cn, 0x4f0a2e5fb028f2cen,
    0xdd611f1000c17442n, 0xd8185f9adfea4fd0n, 0xef87139ca9a3ab1en, 0x3ba71336c34ee133n,
    0x7d3a455d56b70238n, 0x660d32e130182684n, 0x297a863f48cd1f43n, 0x90e0a736a751ebb7n,
    0x549f80ce550c4fd3n, 0x0f73b2922f38bd64n, 0x16bf1f73fb7a9c3fn, 0x6d1f5a59005bec17n,
    0x02ff876fa5ef97c4n, 0xc5cb72a2a51159b0n, 0x8470f39d2d5c900en, 0x25abb3f1d39fcb76n,
    0x23eb8cc9b372442fn, 0xd687ba55c64f6364n, 0xda8d9e90fd8ff158n, 0xe3cbdc7d2fe45ea7n,
    0xb9a8c9b3aee52297n, 0xc0d28a5c10960bd3n, 0x45d7ac9b68f71a34n, 0xeeb76e397069e804n,
    0x3d06c8bd1514e2d9n, 0x9c9c98207cb10767n, 0x65700b51aedfb5efn, 0x911f451539869408n,
    0x7ae6849fbc3a0ec6n, 0x3bb340eba06afe7en, 0xb46e9d8b682ea65en, 0x8dcf22f9a3b34356n,
    0x77bdaeda586257a7n, 0xf19e400a5104d20dn, 0xc368a348e46d950fn, 0x9ef1cd60e679f284n,
    0xe89cd854d5d01d33n, 0x5cd377dc8bb882a2n, 0xa7b0fb7883eee860n, 0x7684403ec392950dn,
    0x5fa3f06f4fed3b52n, 0x8df57ac11bc04831n, 0x2db01efa1e1e1897n, 0x54846de4aadb9ca2n,
    0xba6745385893c784n, 0x541d496344d2c75bn, 0xe909678474e687fen, 0xdfe89923f6c9c2ffn,
    0xece5a71e0cfedc75n, 0x5ff98fd5d51fe610n, 0x83e8941918964615n, 0x5922040b47f150c1n,
    0xf97d750e3dd94521n, 0x5080d4c2b86f56d7n, 0xa7de115b56c78d70n, 0x6a9242ac87538194n,
    0xf7856ef7f9173e44n, 0x2265fc92feb0dc09n, 0x17dfc8e4f7ba8a57n, 0x9001a64209f21db8n,
    0x90004c1371b893c5n, 0xb932b7cf752e5545n, 0xa0b1df81b6fe59fcn, 0x8ef1dd26770af2c2n,
    0x0541a4f9cfbeed35n, 0x9e61106178bfc530n, 0xb3767e80935d8af2n, 0x0098d5782065af06n,
    0x31d191cd5c1466c7n, 0x410fefafa319ac9dn, 0xbdf8f242e316c4abn, 0x9e8cd55b57637ed0n,
    0xde122bebe9a39368n, 0x4d001fd58f002526n, 0xca6637000eb4a9f8n, 0x2f2339d624f91f78n,
    0x6d1a7918c80df518n, 0xdf9a4939342308e9n, 0xebc2151ee6c8398cn, 0x03cc2ba8a1116515n,
    0xd341d037e840cf83n, 0x387cb5d25af4afccn, 0xbba2515f22909e87n, 0x7248fe7705f38e47n,
    0x4d61e56a525d225an, 0x262e963c8da05d3dn, 0x59e89b094d220ec2n, 0x055d5b52b78b9c5en,
    0x82b27eb33514ef99n, 0xd30094ca96b7ce7bn, 0xcf5cb381cd0a1535n, 0xfeed4db6919e5a7cn,
    0x41703f53753be59fn, 0x5eeea940fcde8b6fn, 0x4cd1f1b175100206n, 0x4a20358574454ec0n,
    0x1478d361dbbf9facn, 0x6f02dc07d141875cn, 0x296a202ed8e556a2n, 0x2afd67999bf32ee5n,
    0x7acfd96efa95491dn, 0x6798ba0c0abb2c6dn, 0x34c6f57b26c92122n, 0x5736e1bad206b5den,
    0x20057d2a0056521bn, 0x3dea5bd5d0578bd7n, 0x16e50d897d4634acn, 0x29bff3ecb9b7a6e3n,
    0x475cd3205a3bdcden, 0x18a42105c31b7e88n, 0x023e7414af663068n, 0x15147108121967d7n,
    0xe4a3dff1d7d6fef9n, 0x01a8d1a588085737n, 0x11b4c74eda62beefn, 0xe587cc0d69a73346n,
    0x1ff7327017aa2a6en, 0x594e29c42473d06bn, 0xf6f31db1899b12d5n, 0xc02ac5e47312d3can,
    0xe70201e960cb78b8n, 0x6f90ff3b6a65f108n, 0x42747a7245e7fa84n, 0xd1f507e43ab749b2n,
    0x1c86d265f15750cdn, 0x3996ce73dd832c1cn, 0x8e7fba02983224bdn, 0xba0dec7103255dd4n,
    0x9e9cbd781628fc5bn, 0xdae8645996edd6a5n, 0xdebe0853b1a1d378n, 0xa49229d24d014343n,
    0x7be5b9ffda905e1cn, 0xa3c95eaec244aa30n, 0x0230bca8f4df0544n, 0x4135c2bebfe148c6n,
    0x166fc0cc438a3c72n, 0x3762b59a8ae83efan, 0xe8928a4c89114750n, 0x2a440b51a4945ee5n,
    0x80cefd2b7d99ff83n, 0xbb9879c6e61fd62an, 0x6e7c8f1a84265034n, 0x164bb2de1bbeddc8n,
    0xf3c12fe54d5c653bn, 0x40b9e922ed9771e2n, 0x551f5b0fbe7b1840n, 0x25032aa7c4cb1811n,
    0xaaed34074b164346n, 0x8ffd96bbf9c9c81dn, 0x70fc91eb5937085cn, 0x7f795e2a5f915440n,
    0x4543d9df5476d3cbn, 0xf172d73e004fc90dn, 0xdfd1c4febcc81238n, 0xbc8dfb627fe558fcn,
    0n, 0n, 0, 0n,
    0n, 0n, 0, 0n,
    0n, 0n, 0, 0n
];

const F = new F1Field("0xFFFFFFFF00000001");

for (let i = 0; i < C.length; i++) C[i] = F.e(C[i]);

const MCIRC = [17n, 15n, 41n, 16n, 2n, 28n, 13n, 13n, 39n, 18n, 34n, 20n];
const MDIAG = [8n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n, 0n];

const M = [];
for (let i = 0; i < 12; i++) {
    M[i] = [];
    for (let j = 0; j < 12; j++) {
        M[i][j] = F.e(MCIRC[(-i + j + 12) % 12]);
        if (i === j) M[i][j] = F.add(M[i][j], MDIAG[i]);
    }
}


module.exports.buildConstants = async function (pols) {
    const N = pols.LAST.length;

    const maxHashes = Math.floor(N / (nRoundsF + nRoundsP + 1));

    for (let i=0; i<N; i++) {
        const iH = Math.floor(i/(nRoundsF + nRoundsP + 1));
        const r = i%(nRoundsF + nRoundsP + 1);
        pols.LAST[i] = ((i == N-1) || (i % (nRoundsF + nRoundsP + 1)) == (nRoundsF + nRoundsP)) ? 1n : 0n;
        pols.LATCH[i] = ((iH<maxHashes) && (i % (nRoundsF + nRoundsP + 1) == 0)) ? 1n : 0n;
        pols.LASTBLOCK[i] = ((i % (nRoundsF + nRoundsP + 1)) == (nRoundsF + nRoundsP)) ? 1n : 0n;
        for (j=0; j<12; j++) {
            pols.C[j][i] = C[12*r + j]
        }
        pols.PARTIAL[i] = (r>=nRoundsF/2 && r<(nRoundsF/2 + nRoundsP)) ? 1n : 0n;
    }
};


module.exports.execute = async function (pols, input) {

    const N = pols.in0.length;

    const pow7 = (a) => {
        const a2 = F.square(a);
        const a4 = F.square(a2);
        const a3 = F.mul(a, a2);

        return F.mul(a3, a4);
    };

    const maxHashes = Math.floor(N / (nRoundsF + nRoundsP + 1));

    if (input.length> maxHashes) {
        throw new Error("Not enough Poseidon slots");
    }

    let p=0;

    for (let i=0; i<input.length; i++) {
        pols.in0[p] = F.e(input[i][0]);
        pols.in1[p] = F.e(input[i][1]);
        pols.in2[p] = F.e(input[i][2]);
        pols.in3[p] = F.e(input[i][3]);
        pols.in4[p] = F.e(input[i][4]);
        pols.in5[p] = F.e(input[i][5]);
        pols.in6[p] = F.e(input[i][6]);
        pols.in7[p] = F.e(input[i][7]);
        pols.hashType[p] = F.e(input[i][8]);
        pols.cap1[p] = F.e(input[i][9]);
        pols.cap2[p] = F.e(input[i][10]);
        pols.cap3[p] = F.e(input[i][11]);
        pols.hash0[p] = F.e(input[i][12]);
        pols.hash1[p] = F.e(input[i][13]);
        pols.hash2[p] = F.e(input[i][14]);
        pols.hash3[p] = F.e(input[i][15]);
        pols.result1[p] = (input[i][16] == 1) ? 1n:0n;
        pols.result2[p] = (input[i][16] == 2) ? 1n:0n;
        pols.result3[p] = (input[i][16] == 3) ? 1n:0n;

        p += 1;
        let state = [
            pols.in0[p-1],
            pols.in1[p-1],
            pols.in2[p-1],
            pols.in3[p-1],
            pols.in4[p-1],
            pols.in5[p-1],
            pols.in6[p-1],
            pols.in7[p-1],
            pols.hashType[p-1],
            pols.cap1[p-1],
            pols.cap2[p-1],
            pols.cap3[p-1]
        ];

        for (r=0; r<nRoundsF + nRoundsP; r++) {
            state = state.map((a, i) => F.add(a, C[r * t + i]));

            if (r < nRoundsF / 2 || r >= nRoundsF / 2 + nRoundsP) {
                state = state.map((a) => pow7(a));
            } else {
                state[0] = pow7(state[0]);
            }
            state = state.map((_, i) => state.reduce((acc, a, j) => F.add(acc, F.mul(M[i][j], a)), F.zero));

            pols.in0[p] = state[0];
            pols.in1[p] = state[1];
            pols.in2[p] = state[2];
            pols.in3[p] = state[3];
            pols.in4[p] = state[4];
            pols.in5[p] = state[5];
            pols.in6[p] = state[6];
            pols.in7[p] = state[7];
            pols.hashType[p] = state[8];
            pols.cap1[p] = state[9];
            pols.cap2[p] = state[10];
            pols.cap3[p] = state[11];
            pols.hash0[p] = input[i][12];
            pols.hash1[p] = input[i][13];
            pols.hash2[p] = input[i][14];
            pols.hash3[p] = input[i][15];
            pols.result1[p] = 0n;
            pols.result2[p] = 0n;
            pols.result3[p] = 0n;
            p+=1;
        }
    }

    let st0 = [];
    st0[0] = [F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero, F.zero];

    for (r=0; r<nRoundsF + nRoundsP; r++) {
        st0[r+1] = st0[r].map((a, i) => F.add(a, C[r * t + i]));

        if (r < nRoundsF / 2 || r >= nRoundsF / 2 + nRoundsP) {
            st0[r+1] = st0[r+1].map((a) => pow7(a));
        } else {
            st0[r+1][0] = pow7(st0[r+1][0]);
        }
        st0[r+1] = st0[r+1].map((_, i) => st0[r+1].reduce((acc, a, j) => F.add(acc, F.mul(M[i][j], a)), F.zero));
    }

    while (p<N) {
        pols.in0[p] = st0[p%(nRoundsP + nRoundsF + 1)][0];
        pols.in1[p] = st0[p%(nRoundsP + nRoundsF + 1)][1];
        pols.in2[p] = st0[p%(nRoundsP + nRoundsF + 1)][2];
        pols.in3[p] = st0[p%(nRoundsP + nRoundsF + 1)][3];
        pols.in4[p] = st0[p%(nRoundsP + nRoundsF + 1)][4];
        pols.in5[p] = st0[p%(nRoundsP + nRoundsF + 1)][5];
        pols.in6[p] = st0[p%(nRoundsP + nRoundsF + 1)][6];
        pols.in7[p] = st0[p%(nRoundsP + nRoundsF + 1)][7];
        pols.hashType[p] = st0[p%(nRoundsP + nRoundsF + 1)][8];
        pols.cap1[p] = st0[p%(nRoundsP + nRoundsF + 1)][9];
        pols.cap2[p] = st0[p%(nRoundsP + nRoundsF + 1)][10];
        pols.cap3[p] = st0[p%(nRoundsP + nRoundsF + 1)][11];
        pols.hash0[p] = st0[nRoundsP + nRoundsF][0];
        pols.hash1[p] = st0[nRoundsP + nRoundsF][1];
        pols.hash2[p] = st0[nRoundsP + nRoundsF][2];
        pols.hash3[p] = st0[nRoundsP + nRoundsF][3];
        pols.result1[p] = 0n;
        pols.result2[p] = 0n;
        pols.result3[p] = 0n;
        p+=1;
    }

}

