// copied and modified from pil-stark
const GL3 = require("./f3g.js");
const workerpool = require('workerpool');

const F = new GL3();

function interpolatePrepareBlock(buff, width, start, inc, st_i, st_n) {
    console.log(`linear interpolatePrepare start.... ${st_i}/${st_n}`);

    const heigth = buff.length/width;
    let w = start;
    for (let i = 0; i<heigth; i++) {
        for (let j=0; j<width; j++) {
            buff[i*width+j] = F.mul(  buff[i*width+j], w);
        }
        w = F.mul(w, inc);
    }
    console.log(`linear interpolatePrepare end.... ${st_i}/${st_n}`);
    return buff;
}

function _fft_block(buff, rel_pos, start_pos, nPols, nBits, s, blockBits, layers) {


    const n = 1 << nBits;
    const m = 1 << blockBits;
    const md2 = m >> 1;

    if (layers < blockBits) {
        _fft_block(buff, rel_pos      , start_pos      , nPols, nBits, s, blockBits-1, layers);
        _fft_block(buff, rel_pos      , start_pos + md2, nPols, nBits, s, blockBits-1, layers);
        return;
    }
    if (layers > 1) {
        _fft_block(buff, rel_pos      , start_pos      , nPols, nBits, s-1, blockBits-1, layers-1);
        _fft_block(buff, rel_pos      , start_pos + md2, nPols, nBits, s-1, blockBits-1, layers-1);
    }

    let w;
    if (s>blockBits) {
        const width = 1 << (s-layers);
        const heigth = n / width;
        const y = Math.floor(start_pos / heigth);
        const x = start_pos % heigth;
        const p = x*width + y;
        w = F.exp(F.w[s], p);
    } else {
        w = 1n;
    }

    for (let i=0; i<md2; i++) {
        for (let j=0; j<nPols; j++) {
            // console.log(`${j} ${s} ${start_pos - rel_pos+i} ${start_pos - rel_pos +md2+i} ${buff[(start_pos - rel_pos +i)*nPols+j]}  ${buff[(start_pos- rel_pos+md2+ i)*nPols+j]} ${w}`)
            const t = F.mul(w, buff[(start_pos - rel_pos + md2 + i)*nPols+j]);
            const u = buff[(start_pos - rel_pos+i)*nPols+j];
            buff[(start_pos - rel_pos+i)*nPols+j] = F.add(u, t);
            buff[(start_pos - rel_pos+md2+ i)*nPols+j] = F.sub(u, t);
        }
        w = F.mul(w, F.w[layers])
    }
}

function fft_block(buff, start_pos, nPols, nBits, s, blockBits, layers) {
    console.log(`start block ${s} ${start_pos}`)
    _fft_block(buff, start_pos, start_pos, nPols, nBits, s, blockBits, layers);
    console.log(`end block ${s} ${start_pos}`)
    return buff;
}

if (!workerpool.isMainThread) {
    workerpool.worker({
        fft_block: fft_block,
        interpolatePrepareBlock: interpolatePrepareBlock
    });
}
module.exports.fft_block = fft_block;
