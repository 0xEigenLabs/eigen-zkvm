use crate::f3g::F3G;
//use crate::fft;
use crate::constant::{get_max_workers, MG, SHIFT};
use crate::fft_worker::{fft_block, interpolatePrepareBlock};
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
    buffDst: &mut Vec<F3G>,
    buffSrc: &Vec<F3G>,
    nPols: usize,
    nBits: usize,
    transposeBits: usize,
) {
    let n = 1 << nBits;
    let w = 1 << transposeBits;
    let h = n / w;
    for i in 0..w {
        for j in 0..h {
            let fi = j * w + i;
            let di = i * h + j;
            for k in 0..nPols {
                buffDst[di * nPols + k] = buffSrc[fi * nPols + k];
            }
        }
    }
}

pub fn bitReverse(buffDst: &mut Vec<F3G>, buffSrc: &Vec<F3G>, nPols: usize, nBits: usize) {
    let n = 1 << nBits;
    for i in 0..n {
        let ri = BR(i, nBits);
        for k in 0..nPols {
            buffDst[i * nPols + k] = buffSrc[ri * nPols + k];
        }
    }
}

pub fn interpolateBitReverse(
    buffDst: &mut Vec<F3G>,
    buffSrc: &Vec<F3G>,
    nPols: usize,
    nBits: usize,
) {
    let n = 1 << nBits;
    for i in 0..n {
        let ri = BR(i, nBits);
        let rii = (n - ri) % n;
        for k in 0..nPols {
            buffDst[i * nPols + k] = buffSrc[rii * nPols + k];
        }
    }
}

pub fn invBitReverse(buffDst: &mut Vec<F3G>, buffSrc: &Vec<F3G>, nPols: usize, nBits: usize) {
    let n = 1 << nBits;
    let nInv = F3G::inv(F3G::from(n));
    for i in 0..n {
        let ri = BR(i, nBits);
        let rii = (n - ri) % n;
        for p in 0..nPols {
            buffDst[i * nPols + p] = buffSrc[rii * nPols + p] * nInv;
        }
    }
}

pub fn interpolatePrepare(buff: &mut Vec<F3G>, nPols: usize, nBits: usize, nBitsExt: usize) {
    let n = 1 << nBits;
    let invN = F3G::inv(F3G::from(n)); //F.inv(BigInt(n));
    let maxNPerThread = 1 << 18;
    let minNPerThread = 1 << 12;
    let mut nPerThreadF = (n - 1) / get_max_workers() + 1;
    let maxCorrected = maxNPerThread / nPols;
    let minCorrected = minNPerThread / nPols;

    if (nPerThreadF > maxCorrected) {
        nPerThreadF = maxCorrected
    };
    if (nPerThreadF < minCorrected) {
        nPerThreadF = minCorrected
    };

    rayon::scope(|s| {
        //const curN = Math.min(nPerThreadF, n-i);
        //const bb = buff.slice(i*nPols, (i+curN)*nPols);
        buff.par_chunks_mut(nPerThreadF * nPols)
            .enumerate()
            .for_each(|(i, bb)| {
                let start = invN * (SHIFT.clone().exp(i)); //F.mul(invN, F.exp(F.shift, i));
                let inc = SHIFT.clone();
                interpolatePrepareBlock(bb, nPols, start, inc, i / nPerThreadF, (n / nPerThreadF));
            });
    });
}

pub fn _fft(buffSrc: &Vec<F3G>, nPols: usize, nBits: usize, buffDst: &mut Vec<F3G>, inverse: bool) {
    let maxBlockBits = 16;
    let minBlockBits = 12;
    let blocksPerThread = 8;
    let n = 1 << nBits;
    let mut tmpBuff: Vec<F3G> = vec![F3G::ZERO; n * nPols]; //new BigBuffer(n*nPols);
    let outBuff = buffDst;

    let mut bIn: &mut Vec<F3G>;
    let mut bOut: &mut Vec<F3G>;

    //const pool = workerpool.pool(__dirname + '/fft_worker.js');
    //let pool = rayon::ThreadPoolBuilder::new().num_threads(get_max_workers()).build().unwrap();

    let idealNBlocks = get_max_workers() * blocksPerThread;
    let mut blockBits = log2_any(n * nPols / idealNBlocks);
    if (blockBits < minBlockBits) {
        blockBits = minBlockBits
    };
    if (blockBits > maxBlockBits) {
        blockBits = maxBlockBits
    };
    blockBits = min(nBits, blockBits);
    let blockSize = 1 << blockBits;
    let nBlocks = n / blockSize;

    let mut nTrasposes = 0;
    if (nBits == blockBits) {
        nTrasposes = 0;
    } else {
        nTrasposes = ((nBits - 1) / blockBits) + 1;
    }

    if (nTrasposes & 1 > 0) {
        bOut = &mut tmpBuff;
        bIn = outBuff;
    } else {
        bOut = outBuff;
        bIn = &mut tmpBuff;
    }

    if (inverse) {
        invBitReverse(bOut, buffSrc, nPols, nBits);
    } else {
        bitReverse(bOut, buffSrc, nPols, nBits);
    }
    (bIn, bOut) = (bOut, bIn);

    rayon::scope(|s| {
        for i in (0..nBits).step_by(blockBits) {
            let sInc = min(blockBits, nBits - i);

            //for (j=0; j<nBlocks; j++) {
            //    const bb = bIn.slice(j*blockSize*nPols, (j+1)*blockSize*nPols);
            //    promisesFFT.push(pool.exec("fft_block", [bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc]));

            //    // results[j] = await fft_block(bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc);
            //}
            //const results = await Promise.all(promisesFFT);
            //for (let i=0; i<results.length; i++) {
            //    bIn.set(results[i], i*blockSize*nPols)
            //}
            bIn.par_chunks_mut(blockSize * nPols)
                .enumerate()
                .for_each(|(j, bb)| {
                    fft_block(bb, j * blockSize, nPols, nBits, i + sInc, blockBits, sInc);
                });

            if (sInc < nBits) {
                // Do not transpose if it's the same
                transpose(&mut bOut, &bIn, nPols, nBits, sInc);
                (bIn, bOut) = (bOut, bIn);
            }
        }
    });
}

pub fn fft(buffSrc: &Vec<F3G>, nPols: usize, nBits: usize, buffDst: &mut Vec<F3G>) {
    _fft(buffSrc, nPols, nBits, buffDst, false)
}

pub fn ifft(buffSrc: &Vec<F3G>, nPols: usize, nBits: usize, buffDst: &mut Vec<F3G>) {
    _fft(buffSrc, nPols, nBits, buffDst, true)
}

pub fn interpolate(
    buffSrc: &Vec<F3G>,
    nPols: usize,
    nBits: usize,
    buffDst: &mut Vec<F3G>,
    nBitsExt: usize,
) {
    let n = 1 << nBits;
    let nExt = 1 << nBitsExt;
    let mut tmpBuff: Vec<F3G> = vec![F3G::ZERO; nExt * nPols]; //new BigBuffer(n*nPols);
    let outBuff = buffDst;

    let mut bIn: &mut Vec<F3G>;
    let mut bOut: &mut Vec<F3G>;

    let maxBlockBits = 16;
    let minBlockBits = 12;
    let blocksPerThread = 8;
    let idealNBlocks = get_max_workers() * blocksPerThread;
    let mut nTrasposes = 0;

    let mut blockBits = log2_any(n * nPols / idealNBlocks);
    if (blockBits < minBlockBits) {
        blockBits = minBlockBits
    };
    if (blockBits > maxBlockBits) {
        blockBits = maxBlockBits
    };
    blockBits = min(nBits, blockBits);
    let blockSize = 1 << blockBits;
    let nBlocks = n / blockSize;

    if (blockBits < nBits) {
        nTrasposes += ((nBits - 1) / blockBits) + 1;
    }

    nTrasposes += 1; // The middle convertion

    let mut blockBitsExt = log2_any(nExt * nPols / idealNBlocks);
    if (blockBitsExt < minBlockBits) {
        blockBitsExt = minBlockBits
    };
    if (blockBitsExt > maxBlockBits) {
        blockBitsExt = maxBlockBits
    };
    blockBitsExt = min(nBitsExt, blockBitsExt);
    let blockSizeExt = 1 << blockBitsExt;
    let nBlocksExt = nExt / blockSizeExt;

    if (blockBitsExt < nBitsExt) {
        nTrasposes += (nBitsExt - 1) / blockBitsExt + 1;
    }

    if nTrasposes & 1 > 0 {
        bOut = &mut tmpBuff;
        bIn = outBuff;
    } else {
        bOut = outBuff;
        bIn = &mut tmpBuff;
    }

    println!("len: in {} out {}", bIn.len(), bOut.len());
    println!("Interpolating reverse....");
    interpolateBitReverse(bOut, buffSrc, nPols, nBits);
    (bIn, bOut) = (bOut, bIn);
    println!(
        "after bitversrse len: in {} out {}, nBlocks {} blockSize {}, nBits {} blockBits {}",
        bIn.len(),
        bOut.len(),
        nBlocks,
        blockSize,
        nBits,
        blockBits
    );

    for i in (0..nBits).step_by(blockBits) {
        println!("Layer ifft {}", i);
        let sInc = min(blockBits, nBits - i);
        //const promisesFFT = [];

        // let results = [];
        //for (j=0; j<nBlocks; j++) {
        //    const bb = bIn.slice(j*blockSize*nPols, (j+1)*blockSize*nPols);
        //    promisesFFT.push(pool.exec("fft_block", [bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc]));

        //    // results[j] = await fft_block(bb, j*blockSize, nPols, nBits, i+sInc, blockBits, sInc);
        //}
        //const results = await Promise.all(promisesFFT);
        //for (let i=0; i<results.length; i++) {
        //    bIn.set(results[i], i*blockSize*nPols)
        //}
        bIn.par_chunks_mut(blockSize * nPols)
            .enumerate()
            .for_each(|(j, bb)| {
                fft_block(bb, j * blockSize, nPols, nBits, i + sInc, blockBits, sInc);
            });

        if (sInc < nBits) {
            // Do not transpose if it's the same
            transpose(bOut, bIn, nPols, nBits, sInc);
            (bIn, bOut) = (bOut, bIn);
        }
    }

    println!("Interpolating prepare....");
    interpolatePrepare(bIn, nPols, nBits, nBitsExt);
    println!("Bit reverse....");
    bitReverse(bOut, bIn, nPols, nBitsExt);
    (bIn, bOut) = (bOut, bIn);

    for i in (0..nBitsExt).step_by(blockBitsExt) {
        println!("Layer fft {}", i);
        let sInc = min(blockBitsExt, nBitsExt - i);
        //const promisesFFT = [];

        //// let results = [];
        //for (j=0; j<nBlocksExt; j++) {
        //    const bb = bIn.slice(j*blockSizeExt*nPols, (j+1)*blockSizeExt*nPols);
        //    promisesFFT.push(pool.exec("fft_block", [bb, j*blockSizeExt, nPols, nBitsExt, i+sInc, blockBitsExt, sInc]));

        //    // results[j] = await fft_block(bb, j*blockSizeExt, nPols, nBitsExt, i+sInc, blockBitsExt, sInc);
        //}
        //const results = await Promise.all(promisesFFT);
        //for (let i=0; i<results.length; i++) {
        //    bIn.set(results[i], i*blockSizeExt*nPols)
        //}

        bIn.par_chunks_mut(blockSizeExt * nPols)
            .enumerate()
            .for_each(|(j, bb)| {
                fft_block(
                    bb,
                    j * blockSizeExt,
                    nPols,
                    nBitsExt,
                    i + sInc,
                    blockBitsExt,
                    sInc,
                );
            });

        if sInc < nBitsExt {
            // Do not transpose if it's the same
            transpose(bOut, bIn, nPols, nBitsExt, sInc);
            (bIn, bOut) = (bOut, bIn);
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
        let nBits = 18;
        let nPols = 3;
        let extBits = 1;

        let n = 1 << nBits;
        let mut buff1 = vec![F3G::ZERO; (n * nPols)];
        let mut buff2 = vec![F3G::ZERO; (n * nPols * (1 << extBits))];

        println!("Initializing...");
        for i in 0..nPols {
            for j in 0..n {
                let v = F3G::from(j);
                buff1[j * nPols + i] = v;
            }
        }

        println!("interpolate...");
        interpolate(&buff1, nPols, nBits, &mut buff2, nBits + extBits);
    }

    #[test]
    fn test_p_fft() {
        let nBits = 5;
        let nPols = 2;

        let n = 1 << nBits;
        let mut buff = vec![F3G::ZERO; (n * nPols)];
        let mut buffOut = vec![F3G::ZERO; (n * nPols)];

        let mut F = FFT::new();
        println!("Initializing...");
        let mut pols = vec![Vec::new(); nPols];
        for i in 0..nPols {
            pols[i] = vec![F3G::ZERO; n];
            for j in 0..n {
                let v = F3G::from(j);
                pols[i][j] = v;
                buff[j * nPols + i] = v;
            }
        }
        let mut polsV = vec![Vec::new(); nPols];
        for i in 0..nPols {
            println!("legacy fft ... {}", i);
            polsV[i] = F.fft(&pols[i]);
        }

        println!("fft...");
        fft(&buff, nPols, nBits, &mut buffOut);

        println!("check...");
        for i in 0..nPols {
            for j in 0..n {
                assert_eq!(polsV[i][j], buffOut[j * nPols + i]);
            }
        }
    }

    #[test]
    fn test_p_ifft() {
        let nBits = 18;
        let nPols = 5;

        let n = 1 << nBits;
        let mut buff = vec![F3G::ZERO; (n*nPols)];
        let mut buffOut = vec![F3G::ZERO; (n*nPols)];


        println!("Initializing...");
        let mut pols = vec![vec![]; nPols];
        for i in 0..nPols {
            pols[i] = vec![F3G::ZERO; n];
            for j in 0..n {
                let v = F3G::from(j);
                pols[i][j] = v;
                buff[j*nPols + i] = v;
            }
        }
        let mut F = FFT::new();
        let mut polsV = vec![vec![]; nPols];
        for i in 0..nPols {
            println!("legacy ifft ... {}", i);
            polsV[i] = F.ifft(&pols[i]);
        }

        println!("ifft...");
        ifft(&buff, nPols, nBits, &mut buffOut);

        println!("check...");
        for i in 0..nPols {
            for j in 0..n {
                assert_eq!(polsV[i][j], buffOut[j*nPols + i]);
            }
        }
    }
}
