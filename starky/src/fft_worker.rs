use crate::constant::MG;
use crate::traits::FieldExtension;

pub fn interpolate_prepare_block<F: FieldExtension>(
    buff: &mut [F],
    width: usize,
    start: F,
    inc: F,
    st_i: usize,
    st_n: usize,
) {
    log::trace!("linear interpolatePrepare start....{}/{}", st_i, st_n);
    let heigth = buff.len() / width;
    let mut w = start;
    for i in 0..heigth {
        for j in 0..width {
            buff[i * width + j] *= w;
        }
        w *= inc;
    }
    log::trace!("linear interpolatePrepare end.... {}/{}", st_i, st_n);
}

#[allow(clippy::too_many_arguments)]
fn _fft_block<F: FieldExtension>(
    buff: &mut [F],
    rel_pos: usize,
    start_pos: usize,
    n_pols: usize,
    nbits: usize,
    s: usize,
    blockbits: usize,
    layers: usize,
) {
    //log::trace!("fft_block rel_pos:{} start_pos:{} shift: {} blockbits: {} layers: {}", rel_pos, start_pos, s, blockbits, layers);
    let n = 1 << nbits;
    let m = 1 << blockbits;
    let md2 = m >> 1;

    if layers < blockbits {
        _fft_block(buff, rel_pos, start_pos, n_pols, nbits, s, blockbits - 1, layers);
        _fft_block(buff, rel_pos, start_pos + md2, n_pols, nbits, s, blockbits - 1, layers);
        return;
    }
    if layers > 1 {
        _fft_block(buff, rel_pos, start_pos, n_pols, nbits, s - 1, blockbits - 1, layers - 1);
        _fft_block(buff, rel_pos, start_pos + md2, n_pols, nbits, s - 1, blockbits - 1, layers - 1);
    }

    #[allow(unused_assignments)]
    let mut w = F::ZERO;
    if s > blockbits {
        let width = 1 << (s - layers);
        let heigth = n / width;
        let y = start_pos / heigth;
        let x = start_pos % heigth;
        let p = x * width + y;
        w = F::from(MG.0[s].exp(p as u64));
    } else {
        w = F::ONE;
    }

    for i in 0..md2 {
        for j in 0..n_pols {
            let t = w * buff[(start_pos - rel_pos + md2 + i) * n_pols + j];
            let u = buff[(start_pos - rel_pos + i) * n_pols + j];
            buff[(start_pos - rel_pos + i) * n_pols + j] = u + t;
            buff[(start_pos - rel_pos + md2 + i) * n_pols + j] = u - t;
        }
        w *= F::from(MG.0[layers])
    }
}

pub fn fft_block<F: FieldExtension>(
    buff: &mut [F],
    start_pos: usize,
    n_pols: usize,
    nbits: usize,
    s: usize,
    blockbits: usize,
    layers: usize,
) {
    log::trace!("start block {} {}", s, start_pos);
    _fft_block(buff, start_pos, start_pos, n_pols, nbits, s, blockbits, layers);
    log::trace!("end block {} {}", s, start_pos);
}
