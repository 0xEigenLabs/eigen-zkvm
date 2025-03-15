#![allow(non_snake_case, dead_code)]
#![allow(clippy::needless_range_loop)]

use crate::constant::{get_max_workers, MAX_OPS_PER_THREAD, MG, MIN_OPS_PER_THREAD, SHIFT};
use crate::fft::FFT;
use crate::fft_p::{fft, ifft, interpolate};
use crate::fri::FRIProof;
use crate::fri::FRI;
use crate::helper::pretty_print_array;
use crate::interpreter::compile_code;
use crate::polsarray::PolsArray;
use crate::polutils::batch_inverse;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{Polynom, Segment};
use crate::traits::{FieldExtension, MTNodeType, MerkleTree, Transcript};
use crate::types::{StarkStruct, PIL};
use anyhow::Result;
use fields::field_gl::Fr as FGL;
use profiler_macro::time_profiler;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct StarkContext<F: FieldExtension> {
    pub nbits: usize,
    pub nbits_ext: usize,
    pub N: usize,
    pub Next: usize,
    pub challenge: Vec<F>,
    pub tmp: Vec<F>,
    pub cm1_n: Vec<F>,
    pub cm2_n: Vec<F>,
    pub cm3_n: Vec<F>,
    pub cm4_n: Vec<F>,
    pub tmpexp_n: Vec<F>,
    pub cm1_2ns: Vec<F>,
    pub cm2_2ns: Vec<F>,
    pub cm3_2ns: Vec<F>,
    pub cm4_2ns: Vec<F>,
    pub q_2ns: Vec<F>,
    pub f_2ns: Vec<F>,
    pub x_n: Vec<F>,
    pub x_2ns: Vec<F>,
    pub Zi: Box<dyn Fn(usize) -> F>,
    pub const_n: Vec<F>,
    pub const_2ns: Vec<F>,
    pub publics: Vec<F>,
    pub xDivXSubXi: Vec<FGL>,
    pub xDivXSubWXi: Vec<FGL>,
    pub evals: Vec<F>,

    pub exps_n: Vec<F>,
    pub exps_2ns: Vec<F>,

    pub Z: F,
    pub Zp: F,
    pub tree1: Vec<FGL>,
    pub tree2: Vec<FGL>,
    pub tree3: Vec<FGL>,
    pub tree4: Vec<FGL>,
    pub consts: Vec<FGL>,
}

impl<F: FieldExtension> std::fmt::Debug for StarkContext<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "n {}", self.N)?;
        writeln!(f, "nBits {}", self.nbits)?;
        writeln!(f, "nBitsExt {}", self.nbits_ext)?;
        writeln!(f, "evals {}", pretty_print_array(&self.evals))?;
        writeln!(f, "publics {}", pretty_print_array(&self.publics))?;
        writeln!(f, "challenge {}", pretty_print_array(&self.challenge))?;
        writeln!(f, r#"cm1_n {}"#, pretty_print_array(&self.cm1_n))?;
        writeln!(f, "cm2_n {}", pretty_print_array(&self.cm2_n))?;
        writeln!(f, "cm3_n {}", pretty_print_array(&self.cm3_n))?;
        writeln!(f, "cm4_n {}", pretty_print_array(&self.cm4_n))?;
        writeln!(f, "cm1_2ns {}", pretty_print_array(&self.cm1_2ns))?;
        writeln!(f, "cm2_2ns {}", pretty_print_array(&self.cm2_2ns))?;
        writeln!(f, "cm3_2ns {}", pretty_print_array(&self.cm3_2ns))?;
        writeln!(f, "cm4_2ns {}", pretty_print_array(&self.cm4_2ns))?;
        writeln!(f, "const_n {}", pretty_print_array(&self.const_n))?;
        writeln!(f, "const_2ns {}", pretty_print_array(&self.const_2ns))?;
        writeln!(f, "x_n {}", pretty_print_array(&self.x_n))?;
        writeln!(f, "x_2ns {}", pretty_print_array(&self.x_2ns))?;
        writeln!(f, "xDivXSubXi {}", pretty_print_array(&self.xDivXSubXi))?;
        writeln!(f, "xDivXSubWXi {}", pretty_print_array(&self.xDivXSubWXi))?;
        writeln!(f, "q_2ns {}", pretty_print_array(&self.q_2ns))?;
        writeln!(f, "f_2ns {}", pretty_print_array(&self.f_2ns))?;
        writeln!(f, "tmp {}", pretty_print_array(&self.tmp))?;

        Ok(())
    }
}

unsafe impl<F: FieldExtension> Send for StarkContext<F> {}

unsafe impl<F: FieldExtension> Sync for StarkContext<F> {}

impl<F: FieldExtension> Default for StarkContext<F> {
    fn default() -> Self {
        StarkContext {
            nbits: 0,
            nbits_ext: 0,
            N: 0,
            Next: 0,
            challenge: vec![F::ZERO; 8],
            tmp: Vec::new(),
            cm1_n: Vec::new(),
            cm2_n: Vec::new(),
            cm3_n: Vec::new(),
            cm4_n: Vec::new(),
            tmpexp_n: Vec::new(),
            cm1_2ns: Vec::new(),
            cm2_2ns: Vec::new(),
            cm3_2ns: Vec::new(),
            cm4_2ns: Vec::new(),
            q_2ns: Vec::new(),
            f_2ns: Vec::new(),
            x_n: Vec::new(),
            x_2ns: Vec::new(),
            Zi: Box::new(|_: usize| F::ZERO),
            const_n: Vec::new(),
            const_2ns: Vec::new(),
            publics: Vec::new(),
            xDivXSubXi: Vec::new(),
            xDivXSubWXi: Vec::new(),
            evals: Vec::new(),
            exps_n: Vec::new(),
            exps_2ns: Vec::new(),
            Z: F::ZERO,
            Zp: F::ZERO,
            tree1: Vec::new(),
            tree2: Vec::new(),
            tree3: Vec::new(),
            tree4: Vec::new(),
            consts: Vec::new(),
        }
    }
}

impl<F: FieldExtension> StarkContext<F> {
    pub fn get_mut_base(&mut self, section: &str) -> &mut Vec<FGL> {
        match section {
            "xDivXSubXi" => &mut self.xDivXSubXi,
            "xDivXSubWXi" => &mut self.xDivXSubWXi,
            _ => panic!("invalid symbol {:?}", section),
        }
    }
    pub fn get_mut(&mut self, section: &str) -> &mut Vec<F> {
        match section {
            "tmp" => &mut self.tmp,
            "cm1_n" => &mut self.cm1_n,
            "cm1_2ns" => &mut self.cm1_2ns,
            "cm2_n" => &mut self.cm2_n,
            "cm2_2ns" => &mut self.cm2_2ns,
            "cm3_n" => &mut self.cm3_n,
            "cm4_n" => &mut self.cm4_n,
            "cm3_2ns" => &mut self.cm3_2ns,
            "cm4_2ns" => &mut self.cm4_2ns,
            "q_2ns" => &mut self.q_2ns,
            "f_2ns" => &mut self.f_2ns,
            "exps_n" => &mut self.exps_n,
            "exps_2ns" => &mut self.exps_2ns,
            "const_n" => &mut self.const_n,
            "const_2ns" => &mut self.const_2ns,
            "evals" => &mut self.evals,
            "publics" => &mut self.publics,
            "challenge" => &mut self.challenge,
            "tmpexp_n" => &mut self.tmpexp_n,
            "x_n" => &mut self.x_n,
            "x_2ns" => &mut self.x_2ns,
            _ => {
                panic!("invalid symbol {:?}", section);
            }
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct StarkProof<M: MerkleTree> {
    pub root1: M::MTNode,
    pub root2: M::MTNode,
    pub root3: M::MTNode,
    pub root4: M::MTNode,
    pub fri_proof: FRIProof<M::ExtendField, M>,
    pub evals: Vec<M::ExtendField>,
    pub publics: Vec<M::ExtendField>,
    pub rootC: Option<M::MTNode>,
    pub prover_addr: String,
}

impl<M: MerkleTree> StarkProof<M> {
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    #[time_profiler()]
    pub fn stark_gen<T: Transcript>(
        cm_pols: PolsArray,
        const_pols: PolsArray,
        const_tree: &M,
        starkinfo: &StarkInfo,
        program: &Program,
        _pil: &PIL,
        stark_struct: &StarkStruct,
        prover_addr: &str,
    ) -> Result<StarkProof<M>> {
        let mut ctx = StarkContext::<M::ExtendField>::default();

        let mut fftobj = FFT::new();
        ctx.nbits = stark_struct.nBits;
        ctx.nbits_ext = stark_struct.nBitsExt;
        ctx.N = 1 << stark_struct.nBits;
        ctx.Next = 1 << stark_struct.nBitsExt;
        assert_eq!(1 << ctx.nbits, ctx.N, "N must be a power of 2");

        let mut n_cm = starkinfo.n_cm1;

        log::trace!("Alloc context memory");
        ctx.cm1_n = cm_pols.write_buff();
        drop(cm_pols);

        ctx.cm2_n = vec![M::ExtendField::ZERO; (starkinfo.map_sectionsN.cm2_n) * ctx.N];
        ctx.cm3_n = vec![M::ExtendField::ZERO; (starkinfo.map_sectionsN.cm3_n) * ctx.N];
        ctx.tmpexp_n = vec![M::ExtendField::ZERO; (starkinfo.map_sectionsN.tmpexp_n) * ctx.N];

        ctx.cm1_2ns = vec![M::ExtendField::ZERO; starkinfo.map_sectionsN.cm1_n * ctx.Next];
        ctx.cm2_2ns = vec![M::ExtendField::ZERO; starkinfo.map_sectionsN.cm2_n * ctx.Next];
        ctx.cm3_2ns = vec![M::ExtendField::ZERO; starkinfo.map_sectionsN.cm3_n * ctx.Next];
        ctx.cm4_2ns = vec![M::ExtendField::ZERO; starkinfo.map_sectionsN.cm4_n * ctx.Next];
        ctx.const_2ns = vec![M::ExtendField::ZERO; const_tree.element_size()];

        ctx.q_2ns = vec![M::ExtendField::ZERO; starkinfo.q_dim * ctx.Next];
        ctx.f_2ns = vec![M::ExtendField::ZERO; 3 * ctx.Next];

        ctx.x_n = vec![M::ExtendField::ZERO; ctx.N];

        let xx = M::ExtendField::ONE;
        // Using the precomputing value
        let w_nbits: M::ExtendField = M::ExtendField::from(MG.0[ctx.nbits]);
        ctx.x_n.par_iter_mut().enumerate().for_each(|(k, xb)| {
            *xb = xx * w_nbits.exp(k);
        });

        let extend_bits = ctx.nbits_ext - ctx.nbits;
        ctx.x_2ns = vec![M::ExtendField::ZERO; ctx.N << extend_bits];

        let shift_ext: M::ExtendField = M::ExtendField::from(*SHIFT);
        let w_nbits_ext: M::ExtendField = M::ExtendField::from(MG.0[ctx.nbits_ext]);
        ctx.x_2ns.par_iter_mut().enumerate().for_each(|(k, xb)| {
            *xb = shift_ext * w_nbits_ext.exp(k);
        });

        ctx.Zi = build_Zh_Inv::<M::ExtendField>(ctx.nbits, extend_bits, 0);

        log::trace!("Convert const pols to array");
        ctx.const_n = const_pols.write_buff();
        const_tree.to_extend(&mut ctx.const_2ns);
        drop(const_pols);

        ctx.publics = vec![M::ExtendField::ZERO; starkinfo.publics.len()];
        for (i, pe) in starkinfo.publics.iter().enumerate() {
            if pe.polType.as_str() == "cmP" {
                ctx.publics[i] = ctx.cm1_n[pe.idx * starkinfo.map_sectionsN.cm1_n + pe.polId];
            } else if pe.polType.as_str() == "imP" {
                ctx.publics[i] = Self::calculate_exp_at_point::<M::ExtendField>(
                    &mut ctx,
                    starkinfo,
                    &program.publics_code[i],
                    pe.idx,
                );
            } else {
                panic!("Invalid public type {}", pe.polType);
            }
        }

        let mut transcript = T::new();
        for i in 0..starkinfo.publics.len() {
            let b =
                ctx.publics[i].as_elements().iter().map(|e| vec![*e]).collect::<Vec<Vec<FGL>>>();
            transcript.put(&b[..])?;
        }

        //Do pre-allocation
        let mut result = vec![M::ExtendField::ZERO; (1 << stark_struct.nBitsExt) * 8];
        log::trace!("Merkelizing 1....");
        let tree1 = extend_and_merkelize::<M>(&mut ctx, starkinfo, "cm1_n", &mut result)?;
        tree1.to_extend(&mut ctx.cm1_2ns);

        log::trace!(
            "tree1 root: {}",
            //crate::helper::fr_to_biguint(&tree1.root().into())
            tree1.root(),
        );
        transcript.put(&[tree1.root().as_elements().to_vec()])?;
        // 2.- Calculate plookups h1 and h2
        ctx.challenge[0] = transcript.get_field(); //u
        ctx.challenge[1] = transcript.get_field(); //defVal

        log::trace!("challenge[0] {}", ctx.challenge[0]);
        log::trace!("challenge[1] {}", ctx.challenge[1]);

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step2prev, "n", "step2prev");

        for pu in starkinfo.pu_ctx.iter() {
            let f_pol = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.f_exp_id]);
            let t_pol = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.t_exp_id]);
            let (h1, h2) = calculate_H1H2(f_pol, t_pol);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], h1);
            n_cm += 1;
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], h2);
            n_cm += 1;
        }

        log::trace!("Merkelizing 2....");
        let tree2 = extend_and_merkelize::<M>(&mut ctx, starkinfo, "cm2_n", &mut result)?;
        tree2.to_extend(&mut ctx.cm2_2ns);
        transcript.put(&[tree2.root().as_elements().to_vec()])?;
        log::trace!(
            "tree2 root: {}",
            // crate::helper::fr_to_biguint(&tree2.root().into())
            tree2.root(),
        );

        // 3.- Compute Z polynomials
        ctx.challenge[2] = transcript.get_field(); // gamma
        ctx.challenge[3] = transcript.get_field();
        // beta
        log::trace!("challenge[2] {}", ctx.challenge[2]);
        log::trace!("challenge[3] {}", ctx.challenge[3]);

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step3prev, "n", "step3prev");

        for (i, pu) in starkinfo.pu_ctx.iter().enumerate() {
            log::trace!("Calculating z for plookup {}", i);
            let p_num = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.num_id]);
            let p_den = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pu.den_id]);
            let z = calculate_Z(p_num, p_den);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }

        for (i, pe) in starkinfo.pe_ctx.iter().enumerate() {
            log::trace!("Calculating z for permutation {}", i);
            let p_num = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pe.num_id]);
            let p_den = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&pe.den_id]);
            let z = calculate_Z(p_num, p_den);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }
        for (i, ci) in starkinfo.ci_ctx.iter().enumerate() {
            log::trace!("Calculating z for connection {}", i);
            let p_num = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&ci.num_id]);
            let p_den = get_pol(&mut ctx, starkinfo, starkinfo.exp2pol[&ci.den_id]);
            let z = calculate_Z(p_num, p_den);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step3, "n", "step3");

        log::trace!("Merkelizing 3....");

        let tree3 = extend_and_merkelize::<M>(&mut ctx, starkinfo, "cm3_n", &mut result)?;
        tree3.to_extend(&mut ctx.cm3_2ns);
        transcript.put(&[tree3.root().as_elements().to_vec()])?;

        log::trace!(
            "tree3 root: {}",
            // crate::helper::fr_to_biguint(&tree3.root().into())
            tree3.root(),
        );

        // 4. Compute C Polynomial
        ctx.challenge[4] = transcript.get_field(); // vc

        calculate_exps_parallel(&mut ctx, starkinfo, &program.step42ns, "2ns", "step4");

        log::trace!("Calculate c polynomial");
        let mut qq1 = vec![M::ExtendField::ZERO; ctx.q_2ns.len()];
        let mut qq2 = vec![M::ExtendField::ZERO; starkinfo.q_dim * ctx.Next * starkinfo.q_deg];
        ifft(&ctx.q_2ns, starkinfo.q_dim, ctx.nbits_ext, &mut qq1);

        let mut cur_s = M::ExtendField::ONE;
        let shift_inv = (M::ExtendField::inv(&shift_ext)).exp(ctx.N);

        log::trace!("Calculate qq2");
        for p in 0..starkinfo.q_deg {
            for i in 0..ctx.N {
                for k in 0..starkinfo.q_dim {
                    qq2[i * starkinfo.q_dim * starkinfo.q_deg + starkinfo.q_dim * p + k] =
                        qq1[p * ctx.N * starkinfo.q_dim + i * starkinfo.q_dim + k] * cur_s;
                }
            }
            cur_s *= shift_inv;
        }

        // powdr may produce constant polynomial only
        if starkinfo.q_deg > 0 {
            fft(&qq2, starkinfo.q_dim * starkinfo.q_deg, ctx.nbits_ext, &mut ctx.cm4_2ns);
        }

        log::trace!("Merkelizing 4....");
        let tree4 = merkelize::<M>(&mut ctx, starkinfo, "cm4_2ns").unwrap();
        log::trace!(
            "tree4 root: {}",
            // crate::helper::fr_to_biguint(&tree4.root().into())
            tree4.root(),
        );
        transcript.put(&[tree4.root().as_elements().to_vec()])?;

        //if ctx.cm4_2ns.len() > 0 {
        //    log::trace!("tree4[0] {}", ctx.cm4_2ns[0]);
        //}

        ///////////
        // 5. Compute FRI Polynomial
        ///////////
        ctx.challenge[7] = transcript.get_field(); // xi

        let mut LEv = vec![M::ExtendField::ZERO; ctx.N];
        let mut LpEv = vec![M::ExtendField::ZERO; ctx.N];
        LEv[0] = M::ExtendField::from(FGL::from(1u64));
        LpEv[0] = M::ExtendField::from(FGL::from(1u64));

        let xis = ctx.challenge[7] / shift_ext;
        let wxis = (ctx.challenge[7] * w_nbits) / shift_ext;

        for i in 1..ctx.N {
            LEv[i] = LEv[i - 1] * xis;
            LpEv[i] = LpEv[i - 1] * wxis;
        }

        let LEv = fftobj.ifft(&LEv);
        let LpEv = fftobj.ifft(&LpEv);

        ctx.evals = vec![M::ExtendField::ZERO; starkinfo.ev_map.len()];
        log::trace!("Evals");
        let N = ctx.N;
        for (i, ev) in starkinfo.ev_map.iter().enumerate() {
            let p = match ev.type_.as_str() {
                "const" => Polynom {
                    buffer: &mut ctx.const_2ns,
                    deg: 1 << ctx.nbits_ext,
                    offset: ev.id,
                    size: starkinfo.n_constants,
                    dim: 1,
                },
                "cm" => get_pol_ref(&mut ctx, starkinfo, starkinfo.cm_2ns[ev.id]),
                _ => {
                    panic!("Invalid ev type: {}", ev.type_);
                }
            };
            let l = if ev.prime { &LpEv } else { &LEv };
            let acc = (0..N)
                .into_par_iter()
                .map(|k| {
                    let v = match p.dim {
                        1 => p.buffer[(k << extend_bits) * (p.size) + (p.offset)],
                        // TODO: We need to support F5G
                        _ => M::ExtendField::from_vec(vec![
                            p.buffer[p.offset + (k << extend_bits) * (p.size)].to_be(),
                            p.buffer[p.offset + (k << extend_bits) * (p.size) + 1].to_be(),
                            p.buffer[p.offset + (k << extend_bits) * (p.size) + 2].to_be(),
                        ]),
                    };
                    v * l[k]
                })
                .reduce(|| M::ExtendField::ZERO, |a, b| a + b);
            ctx.evals[i] = acc;
        }

        log::trace!("Add evals to transcript");
        for i in 0..ctx.evals.len() {
            let b = ctx.evals[i].as_elements().iter().map(|e| vec![*e]).collect::<Vec<Vec<FGL>>>();
            transcript.put(&b)?;
        }

        ctx.challenge[5] = transcript.get_field(); // v1
        ctx.challenge[6] = transcript.get_field();
        // v2
        log::trace!("ctx.challenge[5] {}", ctx.challenge[5]);
        log::trace!("ctx.challenge[6] {}", ctx.challenge[6]);
        log::trace!("ctx.challenge[7] {}", ctx.challenge[7]);

        // Calculate xDivXSubXi, xDivXSubWXi
        let xi = ctx.challenge[7];
        let wxi = ctx.challenge[7] * M::ExtendField::from(MG.0[ctx.nbits]);

        let extend_size = N << extend_bits;

        ctx.xDivXSubXi = vec![FGL::ZERO; extend_size * 3];
        ctx.xDivXSubWXi = vec![FGL::ZERO; extend_size * 3];
        let mut tmp_den = vec![M::ExtendField::ZERO; extend_size];
        let mut tmp_denw = vec![M::ExtendField::ZERO; extend_size];

        let mut x_buff = vec![M::ExtendField::ZERO; extend_size];

        let w_ext = M::ExtendField::from(MG.0[ctx.nbits + extend_bits]);
        x_buff.par_iter_mut().enumerate().for_each(|(k, xb)| {
            *xb = shift_ext * w_ext.exp(k);
        });

        tmp_den.par_iter_mut().zip_eq(tmp_denw.par_iter_mut()).enumerate().for_each(
            |(k, (td, tdw))| {
                *td = x_buff[k] - xi;
                *tdw = x_buff[k] - wxi;
            },
        );

        tmp_den = batch_inverse(&tmp_den);
        tmp_denw = batch_inverse(&tmp_denw);
        ctx.xDivXSubXi
            .par_chunks_mut(3)
            .zip_eq(ctx.xDivXSubWXi.par_chunks_mut(3))
            .enumerate()
            .for_each(|(k, (xxx, xxwx))| {
                let v = (tmp_den[k] * x_buff[k]).as_elements();
                xxx[0] = v[0];
                xxx[1] = v[1];
                xxx[2] = v[2];

                let vw = (tmp_denw[k] * x_buff[k]).as_elements();
                xxwx[0] = vw[0];
                xxwx[1] = vw[1];
                xxwx[2] = vw[2];
            });
        calculate_exps_parallel(&mut ctx, starkinfo, &program.step52ns, "2ns", "step5");

        let mut fri_pol = vec![M::ExtendField::ZERO; N << extend_bits];
        fri_pol.par_iter_mut().enumerate().for_each(|(i, o)| {
            *o = M::ExtendField::from_vec(vec![
                ctx.f_2ns[i * 3].to_be(),
                ctx.f_2ns[i * 3 + 1].to_be(),
                ctx.f_2ns[i * 3 + 2].to_be(),
            ]);
        });

        let query_pol = |idx: usize| -> Vec<(Vec<FGL>, Vec<Vec<M::BaseField>>)> {
            vec![
                tree1.get_group_proof(idx).unwrap(),
                tree2.get_group_proof(idx).unwrap(),
                tree3.get_group_proof(idx).unwrap(),
                tree4.get_group_proof(idx).unwrap(),
                const_tree.get_group_proof(idx).unwrap(),
            ]
        };
        let mut fri = FRI::new(stark_struct);
        let friProof = fri.prove::<M::ExtendField, M, T>(&mut transcript, &fri_pol, query_pol)?;

        Ok(StarkProof {
            rootC: Some(const_tree.root()),
            root1: tree1.root(),
            root2: tree2.root(),
            root3: tree3.root(),
            root4: tree4.root(),
            fri_proof: friProof,
            evals: ctx.evals.clone(),
            publics: ctx.publics.clone(),
            prover_addr: prover_addr.to_string(),
        })
    }

    pub fn calculate_exp_at_point<T: FieldExtension>(
        ctx: &mut StarkContext<T>,
        starkinfo: &StarkInfo,
        seg: &Segment,
        idx: usize,
    ) -> T {
        ctx.tmp = vec![T::ZERO; seg.tmp_used];
        let t = compile_code(ctx, starkinfo, &seg.first, "n", true);
        //log::trace!("calculate_exp_at_point compile_code ctx.first:\n{}", t);

        // just let public codegen run multiple times
        //log::trace!("{} = {} @ {}", res, ctx.cm1_n[1 + 2 * idx], idx);
        t.eval(ctx, idx)
    }
}

pub fn build_Zh_Inv<T: FieldExtension>(
    nBits: usize,
    extend_bits: usize,
    offset: usize,
) -> Box<dyn Fn(usize) -> T + 'static> {
    let mut w = T::ONE;
    let mut sn = T::from(*SHIFT);
    for _ in 0..nBits {
        sn = sn * sn;
    }
    let mut ZHInv = vec![T::ZERO; 1 << extend_bits];

    for zi in &mut ZHInv.iter_mut() {
        *zi = T::inv(&(sn * w - T::ONE));
        w *= T::from(MG.0[extend_bits]);
    }
    Box::new(move |i: usize| ZHInv[(i + offset) % ZHInv.len()])
}

fn set_pol<F: FieldExtension>(
    ctx: &mut StarkContext<F>,
    starkinfo: &StarkInfo,
    id_pol: &usize,
    pol: Vec<F>,
) {
    let id_pol = *id_pol;
    let p = get_pol_ref(ctx, starkinfo, id_pol);
    if p.dim == 1 {
        for i in 0..p.deg {
            p.buffer[p.offset + i * p.size] = pol[i];
        }
    } else if p.dim == 3 {
        for i in 0..p.deg {
            let elems = pol[i].as_elements();
            if elems.len() > 1 {
                p.buffer[p.offset + i * p.size] = elems[0].into();
                p.buffer[p.offset + i * p.size + 1] = elems[1].into();
                p.buffer[p.offset + i * p.size + 2] = elems[2].into();
            } else {
                p.buffer[p.offset + i * p.size] = elems[0].into();
                p.buffer[p.offset + i * p.size + 1] = F::ZERO;
                p.buffer[p.offset + i * p.size + 2] = F::ZERO;
            }
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
}

#[time_profiler("calculate_H1H2")]
fn calculate_H1H2<F: FieldExtension>(f: Vec<F>, t: Vec<F>) -> (Vec<F>, Vec<F>) {
    let mut idx_t: HashMap<F, usize> = HashMap::with_capacity(t.len());
    let mut s: Vec<(F, usize)> = vec![(F::ZERO, 0); t.len() + f.len()];

    for (i, e) in t.iter().enumerate() {
        idx_t.insert(*e, i);
        s[i] = (*e, i);
    }

    for (i, e) in f.iter().enumerate() {
        let idx = idx_t.get(e);
        if idx.is_none() {
            panic!("Number not included: {:?}", e);
        }
        s[i + t.len()] = (*e, *idx.unwrap());
    }

    s.sort_by(|a, b| a.1.cmp(&b.1));

    let mut h1 = vec![F::ZERO; f.len()];
    let mut h2 = vec![F::ZERO; f.len()];
    h1.par_iter_mut().zip(h2.par_iter_mut()).enumerate().for_each(|(i, (h1_, h2_))| {
        *h1_ = s[2 * i].0;
        *h2_ = s[2 * i + 1].0;
    });
    (h1, h2)
}

fn calculate_Z<F: FieldExtension>(num: Vec<F>, den: Vec<F>) -> Vec<F> {
    let N = num.len();
    assert_eq!(N, den.len());
    let den_inv = batch_inverse(&den);
    let mut z = vec![F::ZERO; N];
    z[0] = F::ONE;
    for i in 1..N {
        z[i] = z[i - 1] * (num[i - 1] * den_inv[i - 1]);
    }

    let check_val = z[N - 1] * (num[N - 1] * den_inv[N - 1]);
    assert!(check_val._eq(&F::one()));
    z
}

fn get_pol_ref<'a, F: FieldExtension>(
    ctx: &'a mut StarkContext<F>,
    starkinfo: &StarkInfo,
    id_pol: usize,
) -> Polynom<'a, F> {
    let p = &starkinfo.var_pol_map[id_pol];
    Polynom {
        buffer: ctx.get_mut(&p.section),
        deg: starkinfo.map_deg.get(&p.section),
        offset: p.section_pos,
        size: starkinfo.map_sectionsN.get(&p.section),
        dim: p.dim,
    }
}

pub fn get_pol<F: FieldExtension>(
    ctx: &mut StarkContext<F>,
    starkinfo: &StarkInfo,
    id_pol: usize,
) -> Vec<F> {
    let p = get_pol_ref(ctx, starkinfo, id_pol);
    let mut res = vec![F::ZERO; p.deg];
    // TODO: Support F5G
    if p.dim == 1 {
        for i in 0..p.deg {
            res[i] = p.buffer[p.offset + i * p.size];
        }
    } else if p.dim == 3 {
        for i in 0..p.deg {
            res[i] = F::from_vec(vec![
                p.buffer[p.offset + i * p.size].to_be(),
                p.buffer[p.offset + i * p.size + 1].to_be(),
                p.buffer[p.offset + i * p.size + 2].to_be(),
            ]);
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
    res
}

#[time_profiler("extend_and_merkelize")]
pub fn extend_and_merkelize<M: MerkleTree>(
    ctx: &mut StarkContext<M::ExtendField>,
    starkinfo: &StarkInfo,
    section_name: &'static str,
    result: &mut Vec<M::ExtendField>,
) -> Result<M> {
    let nBitsExt = ctx.nbits_ext;
    let nBits = ctx.nbits;
    let n_pols = starkinfo.map_sectionsN.get(section_name);

    let curr_size = (1 << nBitsExt) * n_pols;
    result.resize(curr_size, M::ExtendField::ZERO);

    let p = ctx.get_mut(section_name);
    interpolate(p, n_pols, nBits, result, nBitsExt);
    let mut p_be = vec![FGL::ZERO; result.len()];
    p_be.par_iter_mut().zip(result).for_each(|(be_out, f3g_in)| {
        *be_out = f3g_in.to_be();
    });
    let mut tree = M::new();
    tree.merkelize(p_be, n_pols, 1 << nBitsExt)?;
    Ok(tree)
}

#[time_profiler("merkelize")]
pub fn merkelize<M: MerkleTree>(
    ctx: &mut StarkContext<M::ExtendField>,
    starkinfo: &StarkInfo,
    section_name: &'static str,
) -> Result<M> {
    let nBitsExt = ctx.nbits_ext;
    let n_pols = starkinfo.map_sectionsN.get(section_name);
    let p = ctx.get_mut(section_name);
    let mut p_be = vec![FGL::ZERO; p.len()];
    p_be.par_iter_mut().zip(p).for_each(|(be_out, f3g_in)| {
        *be_out = f3g_in.to_be();
    });
    let mut tree = M::new();
    tree.merkelize(p_be, n_pols, 1 << nBitsExt)?;
    Ok(tree)
}

pub fn calculate_exps<F: FieldExtension>(
    ctx: &mut StarkContext<F>,
    starkinfo: &StarkInfo,
    seg: &Segment,
    dom: &str,
    //step: &str,
    N: usize,
) {
    ctx.tmp = vec![F::ZERO; seg.tmp_used];
    let c_first = compile_code(ctx, starkinfo, &seg.first, dom, false);
    /*
    log::trace!(
        "calculate_exps compile_code {} ctx.first:\n{}",
        step,
        c_first
    );

    let mut N = if dom == "n" { ctx.N } else { ctx.Next };
    let _c_i = compile_code(ctx, starkinfo, &seg.i, dom, false);
    let _c_last = compile_code(ctx, starkinfo, &seg.last, dom, false);
    let next = if dom =="n" { 1 } else { 1<< (ctx.nBitsExt - ctx.nBits) };
    */
    // 0 ~ next: c_first
    // next ~ N-next: c_i
    // N-next ~ N: c_last
    for i in 0..N {
        c_first.eval(ctx, i);
        if (i % 10000) == 0 {
            log::trace!("Calculating expression.. {}/{}", i, N);
        }
    }
}

#[time_profiler()]
pub fn calculate_exps_parallel<F: FieldExtension>(
    ctx: &mut StarkContext<F>,
    starkinfo: &StarkInfo,
    seg: &Segment,
    _dom: &str,
    step: &str,
) {
    #[derive(Debug)]
    struct ExecItem {
        name: String,
        width: usize,
    }

    #[derive(Debug)]
    struct ExecInfo {
        input_sections: Vec<ExecItem>,
        output_sections: Vec<ExecItem>,
    }

    let mut exec_info = ExecInfo { input_sections: vec![], output_sections: vec![] };

    let dom = match step {
        "step2prev" => {
            exec_info.input_sections.push(ExecItem { name: "cm1_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "const_n".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "cm2_n".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "cm3_n".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "tmpexp_n".to_string(), width: 0 });
            "n"
        }
        "step3prev" => {
            exec_info.input_sections.push(ExecItem { name: "cm1_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm2_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm3_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "const_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "x_n".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "cm3_n".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "tmpexp_n".to_string(), width: 0 });
            "n"
        }
        "step3" => {
            exec_info.input_sections.push(ExecItem { name: "cm1_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm2_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm3_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "const_n".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "x_n".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "cm3_n".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "tmpexp_n".to_string(), width: 0 });
            "n"
        }
        "step4" => {
            exec_info.input_sections.push(ExecItem { name: "cm1_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm2_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm3_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "const_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "x_2ns".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "q_2ns".to_string(), width: 0 });
            "2ns"
        }
        "step5" => {
            exec_info.input_sections.push(ExecItem { name: "cm1_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm2_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm3_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "cm4_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "const_2ns".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "xDivXSubXi".to_string(), width: 0 });
            exec_info.input_sections.push(ExecItem { name: "xDivXSubWXi".to_string(), width: 0 });
            exec_info.output_sections.push(ExecItem { name: "f_2ns".to_string(), width: 0 });
            "2ns"
        }
        _ => panic!("Invalid step {}", step),
    };

    let set_width = |section: &mut ExecItem| {
        let name: &str = section.name.as_str();
        if name == "const_n" || name == "const_2ns" {
            section.width = starkinfo.n_constants;
        } else if starkinfo.map_sectionsN.get(name) != usize::MAX {
            section.width = starkinfo.map_sectionsN.get(name);
        } else if ["x_n", "x_2ns"].contains(&name) {
            section.width = 1;
        } else if ["xDivXSubXi", "xDivXSubWXi", "f_2ns"].contains(&name) {
            section.width = 3;
        } else if ["q_2ns"].contains(&name) {
            section.width = starkinfo.q_dim;
        } else {
            panic!("Invalid section name {}", name)
        }
    };

    for i in 0..exec_info.input_sections.len() {
        set_width(&mut exec_info.input_sections[i]);
    }
    for i in 0..exec_info.output_sections.len() {
        set_width(&mut exec_info.output_sections[i]);
    }

    let extend_bits = ctx.nbits_ext - ctx.nbits;
    let n = if dom == "n" { ctx.N } else { ctx.Next };
    let next = if dom == "n" { 1 } else { 1 << extend_bits };

    let mut n_per_thread = (n - 1) / get_max_workers() + 1;
    if n_per_thread > MAX_OPS_PER_THREAD {
        n_per_thread = MAX_OPS_PER_THREAD
    };
    if n_per_thread < MIN_OPS_PER_THREAD {
        n_per_thread = MIN_OPS_PER_THREAD
    };

    let mut ctx_chunks: Vec<StarkContext<F>> = vec![];

    for i in (0..n).step_by(n_per_thread) {
        let cur_n = std::cmp::min(n_per_thread, n - i);
        let mut tmp_ctx = StarkContext::<F> {
            N: n,
            Next: next,
            nbits: ctx.nbits,
            nbits_ext: ctx.nbits_ext,
            evals: ctx.evals.clone(),
            publics: ctx.publics.clone(),
            challenge: ctx.challenge.clone(),
            ..Default::default()
        };

        for si in &exec_info.input_sections {
            if si.name.as_str() == "xDivXSubXi" || si.name.as_str() == "xDivXSubWXi" {
                let tmp = tmp_ctx.get_mut_base(si.name.as_str());
                // for GL(p)
                *tmp = vec![FGL::ZERO; (cur_n + next) * si.width];
                let ori_sec = ctx.get_mut_base(si.name.as_str());
                for j in 0..(cur_n * si.width) {
                    tmp[j] = ori_sec[i * si.width + j]
                }
                // next
                for j in 0..(next * si.width) {
                    tmp[cur_n * si.width + j] = ori_sec[((i + cur_n) % n) * si.width + j]
                }
            } else {
                let tmp = tmp_ctx.get_mut(si.name.as_str());
                // for field extension GL(p^3)
                *tmp = vec![F::ZERO; (cur_n + next) * si.width];
                let ori_sec = ctx.get_mut(si.name.as_str());
                for j in 0..(cur_n * si.width) {
                    tmp[j] = ori_sec[i * si.width + j]
                }
                // next
                for j in 0..(next * si.width) {
                    tmp[cur_n * si.width + j] = ori_sec[((i + cur_n) % n) * si.width + j]
                }
            }
        }
        ctx_chunks.push(tmp_ctx);
    }

    ctx_chunks.par_iter_mut().enumerate().for_each(|(i, tmp_ctx)| {
        let cur_n = std::cmp::min(n_per_thread, n - i * n_per_thread);
        log::trace!("execute trace LDE {}/{}", i * n_per_thread, n);
        tmp_ctx.Zi = build_Zh_Inv(ctx.nbits, extend_bits, i * n_per_thread);
        for so in &exec_info.output_sections {
            let tmp = tmp_ctx.get_mut(so.name.as_str());
            if tmp.is_empty() {
                *tmp = vec![F::ZERO; so.width * (cur_n + next)];
            }
        }
        calculate_exps(tmp_ctx, starkinfo, seg, dom, cur_n);
    });

    // write back the output
    for i in 0..ctx_chunks.len() {
        for so in &exec_info.output_sections {
            let tmp = ctx_chunks[i].get_mut(so.name.as_str());
            let out = ctx.get_mut(so.name.as_str());
            for k in 0..(tmp.len() - so.width * next) {
                out[i * n_per_thread * so.width + k] = tmp[k];
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::field_bn128::Fr;
    use crate::merklehash::MerkleTreeGL;
    use crate::merklehash_bn128::MerkleTreeBN128;
    use crate::polsarray::{PolKind, PolsArray};
    use crate::stark_gen::StarkProof;
    use crate::stark_setup::StarkSetup;
    use crate::stark_verify::stark_verify;
    use crate::traits::MTNodeType;
    use crate::transcript::TranscriptGL;
    use crate::transcript_bn128::TranscriptBN128;
    use crate::types::load_json;
    use crate::types::{StarkStruct, PIL};
    use ark_std::{end_timer, start_timer};

    #[test]
    fn test_stark_gen() {
        let mut pil = load_json::<PIL>("data/fib.pil.json").unwrap();

        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);

        const_pol.load("data/fib.const").unwrap();

        let start_new_pols_array = start_timer!(|| "new_pols_array.commit");
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        end_timer!(start_new_pols_array);

        cm_pol.load("data/fib.cm").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();

        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let serialized = serde_json::to_string(&setup).unwrap();
        let setup: StarkSetup<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();

        let fr_root: Fr = Fr(setup.const_root.as_scalar::<Fr>());
        log::trace!("setup {}", fr_root);

        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();
        let ser = serde_json::to_string(&starkproof).unwrap();
        let de: StarkProof<MerkleTreeBN128> = serde_json::from_str(&ser).unwrap();
        log::trace!("verify the proof...");

        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &de,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &setup.program,
        )
        .unwrap();
        assert!(result);
    }

    #[test]
    fn test_stark_permutation() {
        let mut pil = load_json::<PIL>("data/pe.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/pe.const").unwrap();

        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/pe.cm").unwrap();

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let serialized = serde_json::to_string(&setup).unwrap();
        let setup: StarkSetup<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();

        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();
        let ser = serde_json::to_string(&starkproof).unwrap();
        let de: StarkProof<MerkleTreeBN128> = serde_json::from_str(&ser).unwrap();
        log::trace!("verify the proof...");

        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &de,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &setup.program,
        )
        .unwrap();
        assert!(result);
    }

    #[test]
    fn test_stark_plookup_bn128() {
        let mut pil = load_json::<PIL>("data/plookup.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/plookup.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/plookup.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let setup =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();
        let serialized = serde_json::to_string(&setup).unwrap();
        let setup: StarkSetup<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();
        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();
        let ser = serde_json::to_string(&starkproof).unwrap();
        let de: StarkProof<MerkleTreeBN128> = serde_json::from_str(&ser).unwrap();
        log::trace!("verify the proof...");
        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &de,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &setup.program,
        )
        .unwrap();
        assert!(result);
    }

    #[test]
    fn test_stark_connection() {
        let mut pil = load_json::<PIL>("data/connection.pil.json").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/connection.const").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/connection.cm").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json").unwrap();
        let setup_ =
            StarkSetup::<MerkleTreeBN128>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();

        let serialized = serde_json::to_string(&setup_).unwrap();
        let setup: StarkSetup<MerkleTreeBN128> = serde_json::from_str(&serialized).unwrap();

        let starkproof = StarkProof::<MerkleTreeBN128>::stark_gen::<TranscriptBN128>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();
        let ser = serde_json::to_string(&starkproof).unwrap();
        let de: StarkProof<MerkleTreeBN128> = serde_json::from_str(&ser).unwrap();
        log::trace!("verify the proof...");
        let result = stark_verify::<MerkleTreeBN128, TranscriptBN128>(
            &de,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &setup.program,
        )
        .unwrap();
        assert!(result);
    }

    #[test]
    fn test_stark_plookup_gl() {
        let mut pil = load_json::<PIL>("data/plookup.pil.json.gl").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant);
        const_pol.load("data/plookup.const.gl").unwrap();
        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit);
        cm_pol.load("data/plookup.cm.gl").unwrap();
        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.gl").unwrap();
        let setup_ =
            StarkSetup::<MerkleTreeGL>::new(&const_pol, &mut pil, &stark_struct, None).unwrap();

        let serialized = serde_json::to_string(&setup_).unwrap();
        let setup: StarkSetup<MerkleTreeGL> = serde_json::from_str(&serialized).unwrap();

        let starkproof = StarkProof::<MerkleTreeGL>::stark_gen::<TranscriptGL>(
            cm_pol,
            const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
            "273030697313060285579891744179749754319274977764",
        )
        .unwrap();
        let ser = serde_json::to_string(&starkproof).unwrap();
        let de: StarkProof<MerkleTreeGL> = serde_json::from_str(&ser).unwrap();
        log::trace!("verify the proof...");
        let result = stark_verify::<MerkleTreeGL, TranscriptGL>(
            &starkproof,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &setup.program,
        )
        .unwrap();
        assert!(result);

        let result = stark_verify::<MerkleTreeGL, TranscriptGL>(
            &de,
            &setup.const_root,
            &setup.starkinfo,
            &stark_struct,
            &setup.program,
        )
        .unwrap();
        assert!(result);
    }
}
