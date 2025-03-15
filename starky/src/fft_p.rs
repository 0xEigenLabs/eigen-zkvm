#![allow(dead_code, non_snake_case, clippy::ptr_arg)]
use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MIN_OPS_PER_THREAD, SHIFT};
use crate::fft_worker::{fft_block, interpolate_prepare_block};
use crate::helper::log2_any;
use crate::traits::FieldExtension;
use core::cmp::min;
use lazy_static::lazy_static;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref BR_CACHE: Mutex<HashMap<usize, Vec<usize>>> = Mutex::new(HashMap::new());
}
pub fn BR(x: usize, domain_pow: usize) -> usize {
    assert!(domain_pow <= 32);
    let cal = |x: usize, domain_pow: usize| -> usize {
        let mut x = x;
        x = (x >> 16) | (x << 16);
        x = ((x & 0xFF00FF00) >> 8) | ((x & 0x00FF00FF) << 8);
        x = ((x & 0xF0F0F0F0) >> 4) | ((x & 0x0F0F0F0F) << 4);
        x = ((x & 0xCCCCCCCC) >> 2) | ((x & 0x33333333) << 2);
        (((x & 0xAAAAAAAA) >> 1) | ((x & 0x55555555) << 1)) >> (32 - domain_pow)
    };

    // get cache by domain_pow
    let mut map = BR_CACHE.lock().unwrap();
    let mut cache = if map.contains_key(&domain_pow) {
        map.remove(&domain_pow).unwrap() // get and remove the old values.
    } else {
        vec![]
    };
    // check if need append more to cache
    let cache_len = cache.len();
    let n = 1 << domain_pow;
    if cache_len <= n || cache_len < x {
        let end = if n >= x { n } else { x };
        // todo parallel
        for i in cache_len..=end {
            let a = cal(i, domain_pow);
            cache.push(a);
        }
    }
    let res = cache[x];
    // update map with cache
    map.insert(domain_pow, cache);
    res
}
fn BRs(start: usize, end: usize, domain_pow: usize) -> Vec<usize> {
    assert!(end > start);
    // 1. obtain a useless one to precompute the cache.
    //      to make sure the cache existed and its len >= end.
    BR(end, domain_pow);

    // 2. get cache by domain_pow
    let map = BR_CACHE.lock().unwrap();
    let cache = if map.contains_key(&domain_pow) {
        map.get(&domain_pow).unwrap()
    } else {
        // double check
        BR(end, domain_pow);
        map.get(&domain_pow).unwrap()
    };

    (start..end).map(|i| cache[i]).collect()
}

pub fn transpose<F: FieldExtension>(
    buffdst: &mut Vec<F>,
    buffsrc: &Vec<F>,
    n_pols: usize,
    nbits: usize,
    transpose_bits: usize,
) {
    let n = 1 << nbits;
    let w = 1 << transpose_bits;
    let h = n / w;
    for i in 0..w {
        for j in 0..h {
            let fi = j * w + i;
            let di = i * h + j;
            for k in 0..n_pols {
                buffdst[di * n_pols + k] = buffsrc[fi * n_pols + k];
            }
        }
    }
}

pub fn bit_reverse<F: FieldExtension>(
    buffdst: &mut Vec<F>,
    buffsrc: &Vec<F>,
    n_pols: usize,
    nbits: usize,
) {
    let n = 1 << nbits;
    let ris = BRs(0, n, nbits); // move it outside the loop. obtain it from cache.

    let len = n * n_pols;
    assert_eq!(len, buffdst.len());
    buffdst[0..len].par_iter_mut().enumerate().for_each(|(j, out)| {
        let i = j / n_pols;
        let k = j % n_pols;
        *out = buffsrc[ris[i] * n_pols + k];
    });
}

pub fn interpolate_bit_reverse<F: FieldExtension>(
    buffdst: &mut Vec<F>,
    buffsrc: &Vec<F>,
    n_pols: usize,
    nbits: usize,
) {
    let n = 1 << nbits;
    let ris = BRs(0, n, nbits); // move it outside the loop. obtain it from cache.

    buffdst[0..n * n_pols].par_chunks_mut(n_pols).enumerate().for_each(|(i, out)| {
        let rii = (n - ris[i]) % n;
        for k in 0..n_pols {
            out[k] = buffsrc[rii * n_pols + k];
        }
    });
}

pub fn inv_bit_reverse<F: FieldExtension>(
    buffdst: &mut Vec<F>,
    buffsrc: &Vec<F>,
    n_pols: usize,
    nbits: usize,
) {
    let n = 1 << nbits;
    let n_inv = F::inv(&F::from(n));
    let ris = BRs(0, n, nbits); // move it outside the loop. obtain it from cache.

    let len = n * n_pols;
    assert_eq!(len, buffdst.len());
    buffdst[0..len].par_iter_mut().enumerate().for_each(|(j, out)| {
        let i = j / n_pols;
        let k = j % n_pols;
        let rii = (n - ris[i]) % n;
        *out = buffsrc[rii * n_pols + k] * n_inv;
    });
}

pub fn interpolate_prepare<F: FieldExtension>(buff: &mut Vec<F>, n_pols: usize, nbits: usize) {
    let n = 1 << nbits;
    let inv_n = F::inv(&F::from(n));
    let mut n_per_thread_f = (n - 1) / get_max_workers() + 1;
    let max_corrected = MAX_OPS_PER_THREAD / n_pols;
    let min_corrected = MIN_OPS_PER_THREAD / n_pols;

    if n_per_thread_f > max_corrected {
        n_per_thread_f = max_corrected
    };
    if n_per_thread_f < min_corrected {
        n_per_thread_f = min_corrected
    };

    /*
    for i in (0..n).step_by(n_per_thread_f) {
        let cur_n = min(n_per_thread_f, n - i);
        let mut bb = &mut buff[i * n_pols..(i + cur_n) * n_pols];
        let start = inv_n * (SHIFT.clone().exp(i));
        interpolate_prepare_block(&mut bb, n_pols, start, SHIFT.clone(), i, n);
    }
    */
    let tmp_buff = &mut buff[0..(n * n_pols)];
    tmp_buff.par_chunks_mut(n_per_thread_f * n_pols).enumerate().for_each(|(j, bb)| {
        let i = j * n_per_thread_f;
        let start = inv_n * (F::from(*SHIFT).exp(i));
        interpolate_prepare_block(bb, n_pols, start, F::from(*SHIFT), i, n);
    });
}

pub fn _fft<F: FieldExtension>(
    buffsrc: &Vec<F>,
    n_pols: usize,
    nbits: usize,
    buffdst: &mut Vec<F>,
    inverse: bool,
) {
    let maxblockbits = 16;
    let minblockbits = 12;
    let blocks_per_thread = 8;
    let n = 1 << nbits;
    let mut tmpbuff: Vec<F> = vec![F::ZERO; n * n_pols];
    let outbuff = buffdst;

    let mut bin: &mut Vec<F>;
    let mut bout: &mut Vec<F>;

    let ideal_n_blocks = get_max_workers() * blocks_per_thread;
    let mut blockbits = log2_any(n * n_pols / ideal_n_blocks);
    if blockbits < minblockbits {
        blockbits = minblockbits
    };
    if blockbits > maxblockbits {
        blockbits = maxblockbits
    };
    blockbits = min(nbits, blockbits);
    let blocksize = 1 << blockbits;
    //let n_blocks = n / blocksize;

    #[allow(unused_assignments)]
    let mut n_transposes = 0;
    if nbits == blockbits {
        n_transposes = 0;
    } else {
        n_transposes = ((nbits - 1) / blockbits) + 1;
    }

    if n_transposes & 1 > 0 {
        bout = &mut tmpbuff;
        bin = outbuff;
    } else {
        bout = outbuff;
        bin = &mut tmpbuff;
    }

    if inverse {
        inv_bit_reverse(bout, buffsrc, n_pols, nbits);
    } else {
        bit_reverse(bout, buffsrc, n_pols, nbits);
    }
    (bin, bout) = (bout, bin);

    rayon::scope(|_s| {
        for i in (0..nbits).step_by(blockbits) {
            let s_inc = min(blockbits, nbits - i);
            bin.par_chunks_mut(blocksize * n_pols).enumerate().for_each(|(j, bb)| {
                fft_block(bb, j * blocksize, n_pols, nbits, i + s_inc, blockbits, s_inc);
            });

            if s_inc < nbits {
                // Do not transpose if it's the same
                transpose(bout, bin, n_pols, nbits, s_inc);
                (bin, bout) = (bout, bin);
            }
        }
    });
}

pub fn fft<F: FieldExtension>(buffsrc: &Vec<F>, n_pols: usize, nbits: usize, buffdst: &mut Vec<F>) {
    _fft(buffsrc, n_pols, nbits, buffdst, false)
}

pub fn ifft<F: FieldExtension>(
    buffsrc: &Vec<F>,
    n_pols: usize,
    nbits: usize,
    buffdst: &mut Vec<F>,
) {
    _fft(buffsrc, n_pols, nbits, buffdst, true)
}

pub fn interpolate<F: FieldExtension>(
    buffsrc: &Vec<F>,
    n_pols: usize,
    nbits: usize,
    buffdst: &mut Vec<F>,
    nbitsext: usize,
) {
    if buffsrc.is_empty() {
        return;
    }
    let n = 1 << nbits;
    let n_ext = 1 << nbitsext;
    let mut tmpbuff: Vec<F> = vec![F::ZERO; n_ext * n_pols]; //new BigBuffer(n*n_pols);
    let outbuff = buffdst;

    let mut bin: &mut Vec<F>;
    let mut bout: &mut Vec<F>;

    let maxblockbits = 16;
    let minblockbits = 12;
    let blocks_per_thread = 8;
    let ideal_n_blocks = get_max_workers() * blocks_per_thread;
    let mut n_transposes = 0;

    let mut blockbits = log2_any(n * n_pols / ideal_n_blocks);
    if blockbits < minblockbits {
        blockbits = minblockbits
    };
    if blockbits > maxblockbits {
        blockbits = maxblockbits
    };
    blockbits = min(nbits, blockbits);
    let blocksize = 1 << blockbits;
    //let n_blocks = n / blocksize;

    if blockbits < nbits {
        n_transposes += ((nbits - 1) / blockbits) + 1;
    }

    n_transposes += 1; // The middle conversion

    let mut blockbitsext = log2_any(n_ext * n_pols / ideal_n_blocks);
    if blockbitsext < minblockbits {
        blockbitsext = minblockbits
    };
    if blockbitsext > maxblockbits {
        blockbitsext = maxblockbits
    };
    blockbitsext = min(nbitsext, blockbitsext);
    let blocksizeext = 1 << blockbitsext;

    if blockbitsext < nbitsext {
        n_transposes += (nbitsext - 1) / blockbitsext + 1;
    }

    if (n_transposes & 1) > 0 {
        bout = &mut tmpbuff;
        bin = outbuff;
    } else {
        bout = outbuff;
        bin = &mut tmpbuff;
    }

    log::trace!("Interpolating reverse....");
    interpolate_bit_reverse(bout, buffsrc, n_pols, nbits);
    (bin, bout) = (bout, bin);

    for i in (0..nbits).step_by(blockbits) {
        log::trace!("Layer ifft {}", i);
        let s_inc = min(blockbits, nbits - i);
        bin.par_chunks_mut(blocksize * n_pols).enumerate().for_each(|(j, bb)| {
            fft_block(bb, j * blocksize, n_pols, nbits, i + s_inc, blockbits, s_inc);
        });

        if s_inc < nbits {
            // Do not transpose if it's the same
            transpose(bout, bin, n_pols, nbits, s_inc);
            (bin, bout) = (bout, bin);
        }
    }
    log::trace!("Interpolating prepare....");
    interpolate_prepare(bin, n_pols, nbits);
    log::trace!("Bit reverse....");

    bit_reverse(bout, bin, n_pols, nbitsext);
    (bin, bout) = (bout, bin);

    for i in (0..nbitsext).step_by(blockbitsext) {
        log::trace!("Layer fft {}", i);
        let s_inc = min(blockbitsext, nbitsext - i);
        bin.par_chunks_mut(blocksizeext * n_pols).enumerate().for_each(|(j, bb)| {
            fft_block(bb, j * blocksizeext, n_pols, nbitsext, i + s_inc, blockbitsext, s_inc);
        });
        if s_inc < nbitsext {
            // Do not transpose if it's the same
            transpose(bout, bin, n_pols, nbitsext, s_inc);
            (bin, bout) = (bout, bin);
        }
    }
    log::trace!("interpolation terminated");
}

#[cfg(test)]
mod tests {
    use crate::f3g::F3G;
    use crate::fft::FFT;
    use crate::fft_p::{fft, ifft, interpolate, BR};
    use crate::polutils::extend_pol;
    use crate::traits::FieldExtension;

    #[test]
    fn test_BR() {
        assert_eq!(BR(10, 2), 1);
        assert_eq!(BR(10, 11), 640);
    }

    #[test]
    fn test_big_interpolate() {
        let nbits = 18;
        let n_pols = 5;
        let extbits = 1;

        let n = 1 << nbits;
        let mut buff1 = vec![F3G::ZERO; n * n_pols];
        let mut buff2 = vec![F3G::ZERO; n * n_pols * (1 << extbits)];

        let mut pols: Vec<Vec<F3G>> = vec![Vec::new(); n_pols];
        for i in 0..n_pols {
            pols[i] = vec![F3G::ZERO; n];
            for j in 0..n {
                let v = F3G::from(j);
                pols[i][j] = v;
                buff1[j * n_pols + i] = v;
            }
        }

        let mut pols_v: Vec<Vec<F3G>> = vec![Vec::new(); n_pols];
        for i in 0..n_pols {
            pols_v[i] = extend_pol(&pols[i], extbits);
        }

        interpolate(&buff1, n_pols, nbits, &mut buff2, nbits + extbits);
        let n_ext = 1 << (nbits + extbits);
        for i in 0..n_pols {
            for j in 0..n_ext {
                assert_eq!(pols_v[i][j], buff2[j * n_pols + i]);
            }
        }
    }

    #[test]
    fn test_p_fft() {
        let nbits = 5;
        let n_pols = 2;

        let n = 1 << nbits;
        let mut buff = vec![F3G::ZERO; n * n_pols];
        let mut buffout = vec![F3G::ZERO; n * n_pols];

        let mut sfft = FFT::new();
        log::trace!("Initializing...");
        let mut pols = vec![Vec::new(); n_pols];
        for i in 0..n_pols {
            pols[i] = vec![F3G::ZERO; n];
            for j in 0..n {
                let v = F3G::from(j);
                pols[i][j] = v;
                buff[j * n_pols + i] = v;
            }
        }
        let mut pols_v = vec![Vec::new(); n_pols];
        for i in 0..n_pols {
            log::trace!("legacy fft ... {}", i);
            pols_v[i] = sfft.fft(&pols[i]);
        }

        log::trace!("fft...");
        fft(&buff, n_pols, nbits, &mut buffout);

        log::trace!("check...");
        for i in 0..n_pols {
            for j in 0..n {
                assert_eq!(pols_v[i][j], buffout[j * n_pols + i]);
            }
        }
    }

    #[test]
    fn test_p_ifft() {
        let nbits = 21;
        let n_pols = 5;

        let n = 1 << nbits;
        let mut buff = vec![F3G::ZERO; n * n_pols];
        let mut buffout = vec![F3G::ZERO; n * n_pols];

        log::trace!("Initializing...");
        let mut pols = vec![vec![]; n_pols];
        for i in 0..n_pols {
            pols[i] = vec![F3G::ZERO; n];
            for j in 0..n {
                let v = F3G::from(j);
                pols[i][j] = v;
                buff[j * n_pols + i] = v;
            }
        }
        let mut sfft = FFT::new();
        let mut pols_v = vec![vec![]; n_pols];
        for i in 0..n_pols {
            log::trace!("legacy ifft ... {}", i);
            pols_v[i] = sfft.ifft(&pols[i]);
        }

        log::trace!("ifft...");
        ifft(&buff, n_pols, nbits, &mut buffout);

        log::trace!("check...");
        for i in 0..n_pols {
            for j in 0..n {
                assert_eq!(pols_v[i][j], buffout[j * n_pols + i]);
            }
        }
    }
}
