use crate::constant::MG;
use crate::f3g::F3G;
use winter_math::FieldElement;

pub fn interpolatePrepareBlock(
    buff: &mut [F3G],
    width: usize,
    start: F3G,
    inc: F3G,
    st_i: usize,
    st_n: usize,
) {
    println!("linear interpolatePrepare start....{}/{}", st_i, st_n);
    let heigth = buff.len() / width;
    let mut w = start;
    for i in 0..heigth {
        for j in 0..width {
            buff[i * width + j] = buff[i * width + j] * w;
        }
        w = w * inc;
    }
    println!("linear interpolatePrepare end.... {}/{}", st_i, st_n);
}

fn _fft_block(
    buff: &mut [F3G],
    rel_pos: usize,
    start_pos: usize,
    nPols: usize,
    nBits: usize,
    s: usize,
    blockBits: usize,
    layers: usize,
) {
    //println!("fft_block rel_pos:{} start_pos:{} shift: {} blockBits: {} layers: {}", rel_pos, start_pos, s, blockBits, layers);
    let n = 1 << nBits;
    let m = 1 << blockBits;
    let md2 = m >> 1;

    if layers < blockBits {
        _fft_block(
            buff,
            rel_pos,
            start_pos,
            nPols,
            nBits,
            s,
            blockBits - 1,
            layers,
        );
        _fft_block(
            buff,
            rel_pos,
            start_pos + md2,
            nPols,
            nBits,
            s,
            blockBits - 1,
            layers,
        );
        return;
    }
    if layers > 1 {
        _fft_block(
            buff,
            rel_pos,
            start_pos,
            nPols,
            nBits,
            s - 1,
            blockBits - 1,
            layers - 1,
        );
        _fft_block(
            buff,
            rel_pos,
            start_pos + md2,
            nPols,
            nBits,
            s - 1,
            blockBits - 1,
            layers - 1,
        );
    }

    let mut w = F3G::ZERO;
    if s > blockBits {
        let width = 1 << (s - layers);
        let heigth = n / width;
        let y = start_pos / heigth;
        let x = start_pos % heigth;
        let p = x * width + y;
        w = MG.0[s].exp(p);
    } else {
        w = F3G::ONE;
    }

    for i in 0..md2 {
        for j in 0..nPols {
            let t = w * buff[(start_pos - rel_pos + md2 + i) * nPols + j];
            let u = buff[(start_pos - rel_pos + i) * nPols + j];
            buff[(start_pos - rel_pos + i) * nPols + j] = u + t;
            buff[(start_pos - rel_pos + md2 + i) * nPols + j] = u - t;
        }
        w = w * MG.0[layers]
    }
}

pub fn fft_block(
    buff: &mut [F3G],
    start_pos: usize,
    nPols: usize,
    nBits: usize,
    s: usize,
    blockBits: usize,
    layers: usize,
) {
    println!("start block {} {}", s, start_pos);
    _fft_block(
        buff, start_pos, start_pos, nPols, nBits, s, blockBits, layers,
    );
    println!("end block {} {}", s, start_pos);
}
