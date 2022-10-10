const RC = [
    [0x00000001 >> 0, 0x00000000 >> 0],  
    [0x00008082 >> 0, 0x00000000 >> 0],  
    [0x0000808a >> 0, 0x80000000 >> 0],  
    [0x80008000 >> 0, 0x80000000 >> 0],  
    [0x0000808b >> 0, 0x00000000 >> 0],  
    [0x80000001 >> 0, 0x00000000 >> 0],  
    [0x80008081 >> 0, 0x80000000 >> 0],  
    [0x00008009 >> 0, 0x80000000 >> 0],  
    [0x0000008a >> 0, 0x00000000 >> 0],  
    [0x00000088 >> 0, 0x00000000 >> 0],  
    [0x80008009 >> 0, 0x00000000 >> 0],  
    [0x8000000a >> 0, 0x00000000 >> 0],  
    [0x8000808b >> 0, 0x00000000 >> 0],  
    [0x0000008b >> 0, 0x80000000 >> 0],  
    [0x00008089 >> 0, 0x80000000 >> 0],  
    [0x00008003 >> 0, 0x80000000 >> 0],  
    [0x00008002 >> 0, 0x80000000 >> 0],  
    [0x00000080 >> 0, 0x80000000 >> 0],  
    [0x0000800a >> 0, 0x00000000 >> 0],  
    [0x8000000a >> 0, 0x80000000 >> 0],  
    [0x80008081 >> 0, 0x80000000 >> 0],  
    [0x00008080 >> 0, 0x80000000 >> 0],  
    [0x80000001 >> 0, 0x00000000 >> 0],   
    [0x80008008 >> 0, 0x80000000 >> 0]
];

const AllOnes = 0xFFFFFFFF >> 0;


function rotl(a, r) {
    if (r>=32) {
        res = [ a[1], a[0]];
        r-=32;
    } else {
        res = [ a[0], a[1]];
    }

    if (r) {
        const a0h = res[0] >>>  (32-r);
        const a0l = res[0] << r;
    
        const a1h = res[1] >>>  (32-r);
        const a1l = res[1] << r;
    
        return [a0l + a1h, a1l + a0h];
    } else {
        return res;
    }
}

function theta(A) {
    const C = new Array(5);
    for (let x=0; x<5; x++) {
        C[x] = [ 
            A[x][0][0] ^ A[x][1][0] ^ A[x][2][0] ^ A[x][3][0] ^ A[x][4][0],
            A[x][0][1] ^ A[x][1][1] ^ A[x][2][1] ^ A[x][3][1] ^ A[x][4][1]
        ];
    }
    const Cr = new Array(5);
    for (let x=0; x<5; x++) {
        Cr[x] = rotl(C[x], 1);
    }
    const Ap = [[ [], [], [], [], [] ], [ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ]];
    for (let x=0; x<5; x++) {
        for (let y=0; y<5; y++) {
            Ap[x][y] = [
                A[x][y][0] ^ (C[(x+4)%5][0] ^ Cr[(x+1)%5][0]),
                A[x][y][1] ^ (C[(x+4)%5][1] ^ Cr[(x+1)%5][1])
            ]
        }
    }
    return Ap;
}

function ro(A) {
    const Ap = [[ [], [], [], [], [] ], [ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ]];
    Ap[0][0] = [A[0][0][0], A[0][0][1]];
    let x=1, y=0;
    for (let t=0; t<24; t++) {
        Ap[x][y] = rotl(A[x][y], ((t+1)*(t+2)/2) & 0x3F);
        [x, y] =  [y, (2*x+3*y) % 5];
    }
    return Ap;
}

function pi(A) {
    const Ap = [[ [], [], [], [], [] ], [ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ]];
    for (let x=0; x<5; x++) {
        for (let y=0; y<5; y++) {
            Ap[x][y] = [ ...A[(x+3*y)%5][x]];
        }
    }           
    return Ap;
}

function xi(A) {
    const Ap = [[ [], [], [], [], [] ], [ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ]];
    for (let x=0; x<5; x++) {
        for (let y=0; y<5; y++) {
            Ap[x][y] = [ 
                A[x][y][0] ^ ( (A[(x+1)%5][y][0] ^ AllOnes)   & A[(x+2)%5][y][0]),
                A[x][y][1] ^ ( (A[(x+1)%5][y][1] ^ AllOnes)   & A[(x+2)%5][y][1])
            ];
        }
    }           
    return Ap;
}

function iota(A,i) {
    const Ap = [[ [], [], [], [], [] ], [ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ],[ [], [], [], [], [] ]];
    for (let x=0; x<5; x++) {
        for (let y=0; y<5; y++) {
            Ap[x][y] = [ ...A[x][y]];
        }
    }           
    Ap[0][0] = [
        Ap[0][0][0] ^ RC[i][0],
        Ap[0][0][1] ^ RC[i][1],
    ]
    return Ap;
}

function keccakF(A) {
    let Ap = A;
    for (let r=0; r<24; r++) {
        Ap = iota(xi(pi(ro(theta(Ap)))) ,r);
    }
    return Ap;
}


function keccak(inp) {
    bytes = inp.slice();
    bytes.push(0x01);
    while((bytes.length % 136) !== 0) bytes.push(0);

    bytes[bytes.length-1] |= 0x80;

    let A = [
        [[0,0],[0,0],[0,0],[0,0],[0,0]],
        [[0,0],[0,0],[0,0],[0,0],[0,0]],
        [[0,0],[0,0],[0,0],[0,0],[0,0]],
        [[0,0],[0,0],[0,0],[0,0],[0,0]],
        [[0,0],[0,0],[0,0],[0,0],[0,0]]
    ];
    for (let j=0; j<bytes.length;j+=136) {
        for (k=j; k<j+136; k+=8) {
            const y = Math.floor(k / 40);
            const x = (k % 40) / 8;
            const s = [
                (bytes[k] + (bytes[k+1] << 8) + (bytes[k+2] << 16) + (bytes[k+3] << 24)) >> 0,
                (bytes[k+4] + (bytes[k+5] << 8) + (bytes[k+6] << 16) + (bytes[k+7] << 24)) >> 0
            ];
            A[x][y] = [
                A[x][y][0] ^ s[0],
                A[x][y][1] ^ s[1],
            ]
        }
        A = keccakF(A);
    }

    const res = [];
    pushRes(0,0);
    pushRes(1,0);
    pushRes(2,0);
    pushRes(3,0);

    function pushRes(x,y) {
        res.push(A[x][y][0] & 0xFF);
        res.push((A[x][y][0] >>> 8) & 0xFF);
        res.push((A[x][y][0] >>> 16) & 0xFF);
        res.push((A[x][y][0] >>> 24) & 0xFF);
        res.push(A[x][y][1] & 0xFF);
        res.push((A[x][y][1] >>> 8) & 0xFF);
        res.push((A[x][y][1] >>> 16) & 0xFF);
        res.push((A[x][y][1] >>> 24) & 0xFF);
    }

    return res;
}

module.exports.keccak = keccak;
module.exports.keccakF = keccakF;


