use crate::f3g::F3G;
//use crate::fft;
use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MG, MIN_OPS_PER_THREAD, SHIFT};
use crate::fft_worker::{fft_block, interpolate_prepare_block};
use crate::helper::log2_any;
use core::cmp::min;
use rayon::prelude::*;
use winter_math::FieldElement;

pub fn BR(x: usize, domain_pow: usize) -> usize {
    assert_eq!(domain_pow <= 32, true);
    let mut x = x;
    x = (x >> 16) | (x << 16);
    x = ((x & 0xFF00FF00) >> 8) | ((x & 0x00FF00FF) << 8);
    x = ((x & 0xF0F0F0F0) >> 4) | ((x & 0x0F0F0F0F) << 4);
    x = ((x & 0xCCCCCCCC) >> 2) | ((x & 0x33333333) << 2);
    (((x & 0xAAAAAAAA) >> 1) | ((x & 0x55555555) << 1)) >> (32 - domain_pow)
}

pub fn transpose(
    buffdst: &mut Vec<F3G>,
    buffsrc: &Vec<F3G>,
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

pub fn bitReverse(buffdst: &mut Vec<F3G>, buffsrc: &Vec<F3G>, n_pols: usize, nbits: usize) {
    let n = 1 << nbits;
    for i in 0..n {
        let ri = BR(i, nbits);
        for k in 0..n_pols {
            buffdst[i * n_pols + k] = buffsrc[ri * n_pols + k];
        }
    }
}

pub fn interpolate_bit_reverse(
    buffdst: &mut Vec<F3G>,
    buffsrc: &Vec<F3G>,
    n_pols: usize,
    nbits: usize,
) {
    let n = 1 << nbits;
    for i in 0..n {
        let ri = BR(i, nbits);
        let rii = (n - ri) % n;
        for k in 0..n_pols {
            buffdst[i * n_pols + k] = buffsrc[rii * n_pols + k];
        }
    }
}

pub fn invBitReverse(buffdst: &mut Vec<F3G>, buffsrc: &Vec<F3G>, n_pols: usize, nbits: usize) {
    let n = 1 << nbits;
    let nInv = F3G::inv(F3G::from(n));
    for i in 0..n {
        let ri = BR(i, nbits);
        let rii = (n - ri) % n;
        for p in 0..n_pols {
            buffdst[i * n_pols + p] = buffsrc[rii * n_pols + p] * nInv;
        }
    }
}

pub fn interpolatePrepare(buff: &mut Vec<F3G>, n_pols: usize, nbits: usize, nbitsExt: usize) {
    let n = 1 << nbits;
    let invN = F3G::inv(F3G::from(n));
    let mut nPerThreadF = (n - 1) / get_max_workers() + 1;
    let maxCorrected = MIN_OPS_PER_THREAD / n_pols;
    let minCorrected = MAX_OPS_PER_THREAD / n_pols;

    if nPerThreadF > maxCorrected {
        nPerThreadF = maxCorrected
    };
    if nPerThreadF < minCorrected {
        nPerThreadF = minCorrected
    };

    rayon::scope(|s| {
        buff.par_chunks_mut(nPerThreadF * n_pols)
            .enumerate()
            .for_each(|(i, bb)| {
                let start = invN * (SHIFT.clone().exp(i));
                let inc = SHIFT.clone();
                interpolate_prepare_block(bb, n_pols, start, inc, i / nPerThreadF, n / nPerThreadF);
            });
    });
}

pub fn _fft(
    buffsrc: &Vec<F3G>,
    n_pols: usize,
    nbits: usize,
    buffdst: &mut Vec<F3G>,
    inverse: bool,
) {
    let maxBlockBits = 16;
    let minBlockBits = 12;
    let blocksPerThread = 8;
    let n = 1 << nbits;
    let mut tmpbuff: Vec<F3G> = vec![F3G::ZERO; n * n_pols]; //new BigBuffer(n*n_pols);
    let outbuff = buffdst;

    let mut bIn: &mut Vec<F3G>;
    let mut bout: &mut Vec<F3G>;

    let idealNBlocks = get_max_workers() * blocksPerThread;
    let mut blockBits = log2_any(n * n_pols / idealNBlocks);
    if blockBits < minBlockBits {
        blockBits = minBlockBits
    };
    if blockBits > maxBlockBits {
        blockBits = maxBlockBits
    };
    blockBits = min(nbits, blockBits);
    let blockSize = 1 << blockBits;
    let nBlocks = n / blockSize;

    let mut nTrasposes = 0;
    if nbits == blockBits {
        nTrasposes = 0;
    } else {
        nTrasposes = ((nbits - 1) / blockBits) + 1;
    }

    if nTrasposes & 1 > 0 {
        bout = &mut tmpbuff;
        bIn = outbuff;
    } else {
        bout = outbuff;
        bIn = &mut tmpbuff;
    }

    if inverse {
        invBitReverse(bout, buffsrc, n_pols, nbits);
    } else {
        bitReverse(bout, buffsrc, n_pols, nbits);
    }
    (bIn, bout) = (bout, bIn);

    rayon::scope(|s| {
        for i in (0..nbits).step_by(blockBits) {
            let sInc = min(blockBits, nbits - i);
            bIn.par_chunks_mut(blockSize * n_pols)
                .enumerate()
                .for_each(|(j, bb)| {
                    fft_block(bb, j * blockSize, n_pols, nbits, i + sInc, blockBits, sInc);
                });

            if sInc < nbits {
                // Do not transpose if it's the same
                transpose(&mut bout, &bIn, n_pols, nbits, sInc);
                (bIn, bout) = (bout, bIn);
            }
        }
    });
}

pub fn fft(buffsrc: &Vec<F3G>, n_pols: usize, nbits: usize, buffdst: &mut Vec<F3G>) {
    _fft(buffsrc, n_pols, nbits, buffdst, false)
}

pub fn ifft(buffsrc: &Vec<F3G>, n_pols: usize, nbits: usize, buffdst: &mut Vec<F3G>) {
    _fft(buffsrc, n_pols, nbits, buffdst, true)
}

pub fn interpolate(
    buffsrc: &Vec<F3G>,
    n_pols: usize,
    nbits: usize,
    buffdst: &mut Vec<F3G>,
    nbitsExt: usize,
) {
    if buffsrc.len() == 0 {
        return;
    }
    let n = 1 << nbits;
    let nExt = 1 << nbitsExt;
    let mut tmpbuff: Vec<F3G> = vec![F3G::ZERO; nExt * n_pols]; //new BigBuffer(n*n_pols);
    let outbuff = buffdst;

    let mut bIn: &mut Vec<F3G>;
    let mut bout: &mut Vec<F3G>;

    let maxBlockBits = 16;
    let minBlockBits = 12;
    let blocksPerThread = 8;
    let idealNBlocks = get_max_workers() * blocksPerThread;
    let mut nTrasposes = 0;

    let mut blockBits = log2_any(n * n_pols / idealNBlocks);
    if blockBits < minBlockBits {
        blockBits = minBlockBits
    };
    if blockBits > maxBlockBits {
        blockBits = maxBlockBits
    };
    blockBits = min(nbits, blockBits);
    let blockSize = 1 << blockBits;
    let nBlocks = n / blockSize;

    if blockBits < nbits {
        nTrasposes += ((nbits - 1) / blockBits) + 1;
    }

    nTrasposes += 1; // The middle convertion

    let mut blockBitsExt = log2_any(nExt * n_pols / idealNBlocks);
    if blockBitsExt < minBlockBits {
        blockBitsExt = minBlockBits
    };
    if blockBitsExt > maxBlockBits {
        blockBitsExt = maxBlockBits
    };
    blockBitsExt = min(nbitsExt, blockBitsExt);
    let blockSizeExt = 1 << blockBitsExt;
    let nBlocksExt = nExt / blockSizeExt;

    if blockBitsExt < nbitsExt {
        nTrasposes += (nbitsExt - 1) / blockBitsExt + 1;
    }

    if nTrasposes & 1 > 0 {
        bout = &mut tmpbuff;
        bIn = outbuff;
    } else {
        bout = outbuff;
        bIn = &mut tmpbuff;
    }

    println!("len: in {} out {}", bIn.len(), bout.len());
    println!("Interpolating reverse....");
    interpolate_bit_reverse(bout, buffsrc, n_pols, nbits);
    (bIn, bout) = (bout, bIn);
    println!(
        "after bitversrse len: in {} out {}, nBlocks {} blockSize {}, nbits {} blockBits {}",
        bIn.len(),
        bout.len(),
        nBlocks,
        blockSize,
        nbits,
        blockBits
    );

    for i in (0..nbits).step_by(blockBits) {
        println!("Layer ifft {}", i);
        let sInc = min(blockBits, nbits - i);
        bIn.par_chunks_mut(blockSize * n_pols)
            .enumerate()
            .for_each(|(j, bb)| {
                fft_block(bb, j * blockSize, n_pols, nbits, i + sInc, blockBits, sInc);
            });

        if sInc < nbits {
            // Do not transpose if it's the same
            transpose(bout, bIn, n_pols, nbits, sInc);
            (bIn, bout) = (bout, bIn);
        }
    }

    println!("Interpolating prepare....");
    interpolatePrepare(bIn, n_pols, nbits, nbitsExt);
    println!("Bit reverse....");
    bitReverse(bout, bIn, n_pols, nbitsExt);
    (bIn, bout) = (bout, bIn);

    for i in (0..nbitsExt).step_by(blockBitsExt) {
        println!("Layer fft {}", i);
        let sInc = min(blockBitsExt, nbitsExt - i);
        bIn.par_chunks_mut(blockSizeExt * n_pols)
            .enumerate()
            .for_each(|(j, bb)| {
                fft_block(
                    bb,
                    j * blockSizeExt,
                    n_pols,
                    nbitsExt,
                    i + sInc,
                    blockBitsExt,
                    sInc,
                );
            });

        if sInc < nbitsExt {
            // Do not transpose if it's the same
            transpose(bout, bIn, n_pols, nbitsExt, sInc);
            (bIn, bout) = (bout, bIn);
        }
    }
    println!("interpolation terminated");
}

#[cfg(test)]
mod tests {
    use crate::f3g::F3G;
    use crate::fft::FFT;
    use crate::fft_p::{fft, ifft, interpolate, BR};
    use winter_math::FieldElement;

    #[test]
    fn test_BR() {
        assert_eq!(BR(10, 2), 1);
        assert_eq!(BR(10, 11), 640);
    }

    #[test]
    fn test_big_interpolate() {
        let nbits = 18;
        let n_pols = 3;
        let extbits = 1;

        let n = 1 << nbits;
        let mut buff1 = vec![F3G::ZERO; n * n_pols];
        let mut buff2 = vec![F3G::ZERO; n * n_pols * (1 << extbits)];

        println!("Initializing...");
        for i in 0..n_pols {
            for j in 0..n {
                let v = F3G::from(j);
                buff1[j * n_pols + i] = v;
            }
        }

        println!("interpolate...");
        interpolate(&buff1, n_pols, nbits, &mut buff2, nbits + extbits);

        //TODO check the result
    }

    #[test]
    fn test_p_fft() {
        let nbits = 5;
        let n_pols = 2;

        let n = 1 << nbits;
        let mut buff = vec![F3G::ZERO; n * n_pols];
        let mut buffout = vec![F3G::ZERO; n * n_pols];

        let mut sfft = FFT::new();
        println!("Initializing...");
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
            println!("legacy fft ... {}", i);
            pols_v[i] = sfft.fft(&pols[i]);
        }

        println!("fft...");
        fft(&buff, n_pols, nbits, &mut buffout);

        println!("check...");
        for i in 0..n_pols {
            for j in 0..n {
                assert_eq!(pols_v[i][j], buffout[j * n_pols + i]);
            }
        }
    }

    #[test]
    fn test_p_ifft() {
        let nbits = 18;
        let n_pols = 5;

        let n = 1 << nbits;
        let mut buff = vec![F3G::ZERO; n * n_pols];
        let mut buffout = vec![F3G::ZERO; n * n_pols];

        println!("Initializing...");
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
            println!("legacy ifft ... {}", i);
            pols_v[i] = sfft.ifft(&pols[i]);
        }

        println!("ifft...");
        ifft(&buff, n_pols, nbits, &mut buffout);

        println!("check...");
        for i in 0..n_pols {
            for j in 0..n {
                assert_eq!(pols_v[i][j], buffout[j * n_pols + i]);
            }
        }
    }
}
