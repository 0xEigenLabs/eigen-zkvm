use crate::constant::MG;
use crate::f3g::F3G;
use winter_math::FieldElement;

pub fn interpolate_prepare_block(
    buff: &mut [F3G],
    width: usize,
    start: F3G,
    inc: F3G,
    st_i: usize,
    st_n: usize,
) {
    log::info!("linear interpolatePrepare start....{}/{}", st_i, st_n);
    let heigth = buff.len() / width;
    let mut w = start;
    for i in 0..heigth {
        for j in 0..width {
            buff[i * width + j] = buff[i * width + j] * w;
        }
        w = w * inc;
    }
    log::info!("linear interpolatePrepare end.... {}/{}", st_i, st_n);
}

fn _fft_block(
    buff: &mut [F3G],
    rel_pos: usize,
    start_pos: usize,
    n_pols: usize,
    nbits: usize,
    s: usize,
    blockbits: usize,
    layers: usize,
) {
    //log::debug!("fft_block rel_pos:{} start_pos:{} shift: {} blockbits: {} layers: {}", rel_pos, start_pos, s, blockbits, layers);
    //crate::helper::pretty_print_array(&buff.to_vec());
    let n = 1 << nbits;
    let m = 1 << blockbits;
    let md2 = m >> 1;

    if layers < blockbits {
        _fft_block(
            buff,
            rel_pos,
            start_pos,
            n_pols,
            nbits,
            s,
            blockbits - 1,
            layers,
        );
        _fft_block(
            buff,
            rel_pos,
            start_pos + md2,
            n_pols,
            nbits,
            s,
            blockbits - 1,
            layers,
        );
        return;
    }
    if layers > 1 {
        _fft_block(
            buff,
            rel_pos,
            start_pos,
            n_pols,
            nbits,
            s - 1,
            blockbits - 1,
            layers - 1,
        );
        _fft_block(
            buff,
            rel_pos,
            start_pos + md2,
            n_pols,
            nbits,
            s - 1,
            blockbits - 1,
            layers - 1,
        );
    }

    #[allow(unused_assignments)]
    let mut w = F3G::ZERO;
    if s > blockbits {
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
        for j in 0..n_pols {
            let t = w * buff[(start_pos - rel_pos + md2 + i) * n_pols + j];
            let u = buff[(start_pos - rel_pos + i) * n_pols + j];
            buff[(start_pos - rel_pos + i) * n_pols + j] = u + t;
            buff[(start_pos - rel_pos + md2 + i) * n_pols + j] = u - t;
        }
        w = w * MG.0[layers]
    }
}

pub fn fft_block(
    buff: &mut [F3G],
    start_pos: usize,
    n_pols: usize,
    nbits: usize,
    s: usize,
    blockbits: usize,
    layers: usize,
) {
    log::info!("start block {} {}", s, start_pos);
    _fft_block(
        buff, start_pos, start_pos, n_pols, nbits, s, blockbits, layers,
    );
    log::info!("end block {} {}", s, start_pos);
}
