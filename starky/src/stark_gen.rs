#![allow(non_snake_case)]
use crate::constant::{SHIFT, W};
use crate::digest_bn128::ElementDigest;
use crate::errors::Result;
use crate::f3g::F3G;
use crate::fri::FRIProof;
use crate::fri::FRI;
use crate::interpreter::compile_code;
use crate::merklehash_bn128::MerkleTree;
use crate::polsarray::PolsArray;
use crate::poseidon_bn128::Fr;
use crate::stark_setup::interpolate_in_pil;
use crate::stark_setup::StarkSetup;
use crate::starkinfo::{Program, StarkInfo};
use crate::starkinfo_codegen::{Polynom, Segment};
use crate::transcript_bn128::TranscriptBN128;
use crate::types::{StarkStruct, PIL};
use std::collections::HashMap;
use std::rc::Rc;
use winter_math::fft;
use winter_math::fields::f64::BaseElement;
use winter_math::{FieldElement, StarkField};

pub struct StarkContext {
    pub nbits: usize,
    pub nbits_ext: usize,
    pub N: usize,
    pub Next: usize,
    pub challenges: Vec<F3G>,
    pub tmp: Vec<F3G>,
    pub cm1_n: Vec<F3G>,
    pub cm2_n: Vec<F3G>,
    pub cm3_n: Vec<F3G>,
    pub exps_withq_n: Vec<F3G>,
    pub exps_withoutq_n: Vec<F3G>,
    pub cm1_2ns: Vec<F3G>,
    pub cm2_2ns: Vec<F3G>,
    pub cm3_2ns: Vec<F3G>,
    pub q_2ns: Vec<F3G>,
    pub exps_withq_2ns: Vec<F3G>,
    pub exps_withoutq_2ns: Vec<F3G>,
    pub x_n: Vec<F3G>,
    pub x_2ns: Vec<F3G>,
    pub Zi: Box<dyn Fn(usize) -> F3G>,
    pub const_n: Vec<F3G>,
    pub const_2ns: Vec<F3G>,
    pub publics: Vec<F3G>,
    pub xDivXSubXi: Vec<BaseElement>,
    pub xDivXSubWXi: Vec<BaseElement>,
    pub evals: Vec<F3G>,

    pub exps_n: Vec<F3G>,
    pub exps_2ns: Vec<F3G>,

    pub Z: F3G,
    pub Zp: F3G,
    pub tree1: Vec<BaseElement>,
    pub tree2: Vec<BaseElement>,
    pub tree3: Vec<BaseElement>,
    pub tree4: Vec<BaseElement>,
    pub consts: Vec<BaseElement>,
}

impl Default for StarkContext {
    fn default() -> Self {
        StarkContext {
            nbits: 0,
            nbits_ext: 0,
            N: 0,
            Next: 0,
            challenges: Vec::new(),
            tmp: Vec::new(),
            cm1_n: Vec::new(),
            cm2_n: Vec::new(),
            cm3_n: Vec::new(),
            exps_withq_n: Vec::new(),
            exps_withoutq_n: Vec::new(),
            cm1_2ns: Vec::new(),
            cm2_2ns: Vec::new(),
            cm3_2ns: Vec::new(),
            q_2ns: Vec::new(),
            exps_withq_2ns: Vec::new(),
            exps_withoutq_2ns: Vec::new(),
            x_n: Vec::new(),
            x_2ns: Vec::new(),
            Zi: Box::new(|i: usize| F3G::ZERO),
            const_n: Vec::new(),
            const_2ns: Vec::new(),
            publics: Vec::new(),
            xDivXSubXi: Vec::new(),
            xDivXSubWXi: Vec::new(),
            evals: Vec::new(),
            exps_n: Vec::new(),
            exps_2ns: Vec::new(),
            Z: F3G::ZERO,
            Zp: F3G::ZERO,
            tree1: Vec::new(),
            tree2: Vec::new(),
            tree3: Vec::new(),
            tree4: Vec::new(),
            consts: Vec::new(),
        }
    }
}

impl StarkContext {
    pub fn get_mut(&mut self, section: &str) -> &mut Vec<F3G> {
        match section {
            "tmp" => &mut self.tmp,
            "cm1_n" => &mut self.cm1_n,
            "cm1_2ns" => &mut self.cm1_2ns,
            "cm2_n" => &mut self.cm2_n,
            "cm2_2ns" => &mut self.cm2_2ns,
            "cm3_n" => &mut self.cm3_n,
            "cm3_2ns" => &mut self.cm3_2ns,
            "q_2ns" => &mut self.q_2ns,
            "exps_n" => &mut self.exps_n,
            "exps_2ns" => &mut self.exps_2ns,
            "exps_withq_n" => &mut self.exps_withq_n,
            "exps_withq_2ns" => &mut self.exps_withq_2ns,
            _ => {
                panic!("invalid symbol {:?}", section);
            }
        }
    }
}

pub struct StarkProof {
    pub root1: ElementDigest,
    pub root2: ElementDigest,
    pub root3: ElementDigest,
    pub root4: ElementDigest,
    pub fri_proof: FRIProof,
    pub evals: Vec<F3G>,
    pub publics: Vec<F3G>,
}

impl<'a> StarkProof {
    pub fn stark_gen(
        cm_pols: &PolsArray,
        const_pols: &PolsArray,
        const_tree: &MerkleTree,
        starkinfo: &'a StarkInfo,
        program: &Program,
        pil: &PIL,
        stark_struct: &StarkStruct,
    ) -> Result<StarkProof> {
        let mut ctx = StarkContext::default();

        ctx.nbits = stark_struct.nBits;
        ctx.nbits_ext = stark_struct.nBitsExt;
        ctx.N = 1 << stark_struct.nBits;
        ctx.Next = 1 << stark_struct.nBitsExt;
        assert_eq!(1 << ctx.nbits, ctx.N, "N must be a power of 2");

        let mut n_cm = starkinfo.n_cm1;

        ctx.cm1_n = cm_pols.write_buff();
        ctx.cm2_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm2_n) * ctx.N];
        ctx.cm3_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm3_n) * ctx.N];
        ctx.exps_withq_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.exps_withq_n) * ctx.N];
        ctx.exps_withoutq_n = vec![F3G::ZERO; (starkinfo.map_sectionsN.exps_withoutq_n) * ctx.N];

        ctx.cm1_2ns = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm1_n) * ctx.Next];
        ctx.cm2_2ns = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm2_n) * ctx.Next];
        ctx.cm3_2ns = vec![F3G::ZERO; (starkinfo.map_sectionsN.cm3_n) * ctx.Next];

        ctx.q_2ns = vec![F3G::ZERO; starkinfo.map_sectionsN.q_2ns * ctx.Next];
        ctx.exps_withq_2ns = vec![F3G::ZERO; starkinfo.map_sectionsN.exps_withq_2ns * ctx.Next];
        ctx.exps_withoutq_2ns =
            vec![F3G::ZERO; starkinfo.map_sectionsN.exps_withoutq_2ns * ctx.Next];

        ctx.x_n = vec![F3G::ZERO; ctx.N];
        let mut xx = F3G::ONE;
        for i in 0..ctx.N {
            ctx.x_n[i] = xx;
            xx = xx * W.0[ctx.nbits];
        }

        let extendBits = ctx.nbits_ext - ctx.nbits;
        ctx.x_2ns = vec![F3G::ZERO; ctx.N];
        let mut xx = SHIFT.clone();
        for i in 0..(1 << (ctx.nbits_ext - ctx.nbits)) {
            ctx.x_2ns[i] = xx;
            xx = xx * W.0[ctx.nbits_ext];
        }

        ctx.Zi = Self::build_Zh_Inv(ctx.nbits, extendBits);

        ctx.const_n = const_pols.write_buff();
        ctx.const_2ns = const_tree.write_buff();

        ctx.publics = vec![F3G::ZERO; starkinfo.publics.len()];
        for (i, pe) in starkinfo.publics.iter().enumerate() {
            if pe.polType.as_str() == "cmP" {
                ctx.publics[i] = ctx.cm1_n[(pe.idx * starkinfo.map_sectionsN.cm1_n + pe.polId)];
            } else if pe.polType.as_str() == "imP" {
                ctx.publics[i] = Self::calculate_exp_at_point(
                    &mut ctx,
                    starkinfo,
                    &program.publics_code[i],
                    pe.idx,
                );
            } else {
                panic!("Invalid public type {}", pe.polType);
            }
        }

        let mut transcript = TranscriptBN128::new();
        println!("Merkeling 1....");
        let tree1 = extend_and_merkelize(&mut ctx, starkinfo, "cm1_n").unwrap();
        ctx.cm1_2ns = to_array(&tree1.elements);
        let root: Fr = tree1.root().into();
        transcript.put(&vec![root])?;

        ///////////
        // 2.- Caluculate plookups h1 and h2
        ///////////
        ctx.challenges[0] = transcript.get_field(); //u
        ctx.challenges[1] = transcript.get_field(); //defVal

        //TODO parallel execution
        calculate_exps(&mut ctx, starkinfo, &program.step2prev, "2ns");

        for pu in starkinfo.pu_ctx.iter() {
            let f_pol = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[pu.f_exp_id]);
            let t_pol = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[pu.t_exp_id]);
            let (h1, h2) = calculate_H1H2(f_pol, t_pol);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], h1);
            n_cm += 1;
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], h2);
            n_cm += 1;
        }

        println!("Merkeling 2....");
        let tree2 = extend_and_merkelize(&mut ctx, starkinfo, "cm2_n").unwrap();
        ctx.cm2_2ns = to_array(&tree2.elements);
        let root: Fr = tree2.root().into();
        transcript.put(&vec![root])?;

        ///////////
        // 3.- Compute Z polynomials
        ///////////
        ctx.challenges[2] = transcript.get_field(); // gamma
        ctx.challenges[3] = transcript.get_field(); // betta

        calculate_exps(&mut ctx, starkinfo, &program.step3prev, "n");

        for (i, pu) in starkinfo.pu_ctx.iter().enumerate() {
            println!("Calculating z for plookup {}", i);
            let pNum = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[pu.num_id]);
            let pDen = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[pu.den_id]);
            let z = calculate_Z(pNum, pDen);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }

        for (i, pe) in starkinfo.pe_ctx.iter().enumerate() {
            println!("Calculating z for permutation {}", i);
            let pNum = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[pe.num_id]);
            let pDen = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[pe.den_id]);
            let z = calculate_Z(pNum, pDen);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }
        for (i, ci) in starkinfo.ci_ctx.iter().enumerate() {
            println!("Calculating z for connection {}", i);
            let pNum = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[ci.num_id]);
            let pDen = get_pol(&mut ctx, starkinfo, starkinfo.exps_n[ci.den_id]);
            let z = calculate_Z(pNum, pDen);
            set_pol(&mut ctx, starkinfo, &starkinfo.cm_n[n_cm], z);
            n_cm += 1;
        }

        println!("Merkelizing 3....");

        let tree3 = extend_and_merkelize(&mut ctx, starkinfo, "cm3_n").unwrap();
        ctx.cm3_2ns = to_array(&tree3.elements);

        let root: Fr = tree3.root().into();
        transcript.put(&vec![root])?;

        ///////////
        // 4. Compute C Polynomial
        ///////////
        ctx.challenges[4] = transcript.get_field(); // vc

        calculate_exps(&mut ctx, starkinfo, &program.step4, "n");

        //await extend(ctx.exps_withq_n, ctx.exps_withq_2ns, starkInfo.mapSectionsN.exps_withq_n, ctx.nBits, ctx.nBitsExt);
        ctx.exps_withq_2ns = extend(&mut ctx, starkinfo, "exps_withq_n").unwrap();

        calculate_exps(&mut ctx, starkinfo, &program.step42ns, "2ns");

        println!("Merkelizing 4....");
        let tree4 = merkelize(&mut ctx, starkinfo, "q_2ns").unwrap();
        let root: Fr = tree4.root().into();
        transcript.put(&vec![root])?;

        ///////////
        // 5. Compute FRI Polynomial
        ///////////
        ctx.challenges[5] = transcript.get_field(); // v1
        ctx.challenges[6] = transcript.get_field(); // v2
        ctx.challenges[7] = transcript.get_field(); // xi

        let mut LEv = vec![F3G::ZERO; ctx.N];
        let mut LpEv = vec![F3G::ZERO; ctx.N];
        LEv[0] = F3G::from(BaseElement::from(1u64));
        LpEv[0] = F3G::from(BaseElement::from(1u64));

        let xis = ctx.challenges[7] / SHIFT.clone();
        let wxis = (ctx.challenges[7] * W.0[ctx.nbits]) / SHIFT.clone();

        for i in 1..ctx.N {
            LEv[i] = LEv[i - 1] * xis;
            LpEv[i] = LpEv[i - 1] * wxis;
        }

        //ifft
        let inv_twiddles = fft::get_inv_twiddles(ctx.N);
        fft::interpolate_poly(&mut LEv, &inv_twiddles);
        fft::interpolate_poly(&mut LpEv, &inv_twiddles);

        ctx.evals = vec![F3G::ZERO; starkinfo.ev_map.len()];
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
                "q" => get_pol_ref(&mut ctx, starkinfo, starkinfo.qs[ev.id]),
                _ => {
                    panic!("Invalid ev type: {}", ev.type_);
                }
            };
            let l = if ev.prime { &LpEv } else { &LEv };
            let mut acc = F3G::ZERO;
            for k in 0..N {
                let v = match p.dim {
                    1 => p.buffer[(k << extendBits) * (p.size) + (p.offset)],
                    _ => F3G::new(
                        p.buffer[(p.offset + (k << extendBits) * (p.size))].to_be(),
                        p.buffer[(p.offset + (k << extendBits) * (p.size)) + 1].to_be(),
                        p.buffer[(p.offset + (k << extendBits) * (p.size)) + 2].to_be(),
                    ),
                };

                acc = acc + (v * l[k])
            }
            ctx.evals[i] = acc;
        }

        // Calculate xDivXSubXi, xDivXSubWXi

        let xi = ctx.challenges[7];
        let wxi = ctx.challenges[7] * W.0[ctx.nbits];

        ctx.xDivXSubXi = vec![BaseElement::ZERO; (ctx.N << extendBits) * 3];
        ctx.xDivXSubWXi = vec![BaseElement::ZERO; (ctx.N << extendBits) * 3];
        let mut tmp_den = vec![F3G::ZERO; ctx.N << extendBits];
        let mut tmp_denw = vec![F3G::ZERO; ctx.N << extendBits];
        let mut x = SHIFT.clone();
        for k in 0..(N << extendBits) {
            tmp_den[k] = x - xi;
            tmp_denw[k] = x - wxi;
            x = x * W.0[ctx.nbits + extendBits];
        }
        tmp_den = F3G::batch_inverse(&tmp_den);
        tmp_denw = F3G::batch_inverse(&tmp_denw);
        x = SHIFT.clone();
        for k in 0..(N << extendBits) {
            let v = (tmp_den[k] * x).as_base_elements();
            ctx.xDivXSubXi[3 * k] = v[0];
            ctx.xDivXSubXi[3 * k + 1] = v[1];
            ctx.xDivXSubXi[3 * k + 2] = v[2];

            let vw = (tmp_denw[k] * x).as_base_elements();
            ctx.xDivXSubWXi[3 * k] = vw[0];
            ctx.xDivXSubWXi[3 * k + 1] = vw[1];
            ctx.xDivXSubWXi[3 * k + 2] = vw[2];

            x = x * W.0[ctx.nbits + extendBits];
        }

        calculate_exps(&mut ctx, starkinfo, &program.step52ns, "2ns");

        let friPol = get_pol(
            &mut ctx,
            starkinfo,
            starkinfo.exps_2ns[starkinfo.fri_exp_id],
        );

        //let mut trees = vec![&tree1, &tree2, &tree3, &tree4, const_tree];
        let query_pol = |idx: usize| -> Vec<(Vec<BaseElement>, Vec<Vec<Fr>>)> {
            vec![
                tree1.get_group_proof(idx).unwrap(),
                tree2.get_group_proof(idx).unwrap(),
                tree3.get_group_proof(idx).unwrap(),
                tree4.get_group_proof(idx).unwrap(),
                const_tree.get_group_proof(idx).unwrap(),
            ]
        };
        let mut fri = FRI::new(stark_struct);

        let friProof = fri.prove(&mut transcript, &friPol, query_pol)?;

        Ok(StarkProof {
            root1: tree1.root(),
            root2: tree1.root(),
            root3: tree1.root(),
            root4: tree1.root(),
            fri_proof: friProof,
            evals: ctx.evals.clone(),
            publics: ctx.publics.clone(),
        })
    }

    pub fn calculate_exp_at_point(
        ctx: &mut StarkContext,
        starkinfo: &StarkInfo,
        seg: &Segment,
        idx: usize,
    ) -> F3G {
        ctx.tmp = vec![F3G::ZERO; seg.tmp_used];
        let t = compile_code(ctx, starkinfo, &seg.first, "n", true);
        t.eval(ctx, idx)
    }

    pub fn build_Zh_Inv(nBits: usize, extendBits: usize) -> Box<dyn Fn(usize) -> F3G + 'static> {
        let mut w = F3G::ONE;
        let mut sn = SHIFT.clone();
        for i in 0..nBits {
            sn = sn * sn;
        }
        let mut ZHInv = vec![F3G::ZERO; (1<<extendBits)];
        for i in 0..(1 << extendBits) {
            ZHInv[i] = -(sn * w - F3G::ONE);
            w = w * W.0[extendBits];
        }
        Box::new(move |i: usize| ZHInv[i].clone())
    }
}

fn set_pol(ctx: &mut StarkContext, starkinfo: &StarkInfo, id_pol: &usize, pol: Vec<F3G>) {
    let id_pol = *id_pol;
    let p = get_pol_ref(ctx, starkinfo, id_pol);
    if p.dim == 1 {
        for i in 0..p.deg {
            p.buffer[(p.offset + i * p.size)] = pol[i];
        }
    } else if p.dim == 3 {
        for i in 0..p.deg {
            let elems = pol[i].as_base_elements();
            p.buffer[(p.offset + i * p.size)] = elems[0].into();
            p.buffer[(p.offset + i * p.size) + 1] = elems[1].into();
            p.buffer[(p.offset + i * p.size) + 2] = elems[2].into();
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
}

fn calculate_H1H2(f: Vec<F3G>, t: Vec<F3G>) -> (Vec<F3G>, Vec<F3G>) {
    let mut idx_t: HashMap<F3G, usize> = HashMap::new();
    let mut s: Vec<(F3G, usize)> = vec![];

    for (i, e) in t.iter().enumerate() {
        idx_t.insert(*e, i);
        s.push((*e, i));
    }

    for (i, e) in f.iter().enumerate() {
        let idx = idx_t.get(e);
        if idx.is_none() {
            panic!("Number not included: {:?}", e);
        }
        s.push((e.clone(), *idx.unwrap()));
    }

    s.sort_by(|a, b| a.1.cmp(&b.1));

    let mut h1 = vec![F3G::ZERO; f.len()];
    let mut h2 = vec![F3G::ZERO; f.len()];
    for i in 0..f.len() {
        h1[i] = s[2 * i].0;
        h2[i] = s[2 * i + 1].0;
    }
    (h1, h2)
}

fn calculate_Z(num: Vec<F3G>, den: Vec<F3G>) -> Vec<F3G> {
    let N = num.len();
    assert_eq!(N, den.len());
    let den_inv = F3G::batch_inverse(&den);
    let mut z = vec![F3G::ZERO; N];
    z[0] = F3G::ONE;
    for i in 1..N {
        z[i] = z[i - 1] * (num[i - 1] * den_inv[i - 1]);
    }

    let check_val = z[N - 1] * (num[N - 1] * den_inv[N - 1]);
    assert_eq!(check_val.eq(&F3G::ONE), true);
    z
}

fn get_pol_ref<'a>(ctx: &'a mut StarkContext, starkinfo: &StarkInfo, id_pol: usize) -> Polynom<'a> {
    let p = &starkinfo.var_pol_map[id_pol];
    Polynom {
        buffer: ctx.get_mut(&p.section),
        deg: starkinfo.map_deg.get(&p.section),
        offset: p.section_pos,
        size: starkinfo.map_sectionsN.get(&p.section),
        dim: p.dim,
    }
}

pub fn get_pol(ctx: &mut StarkContext, starkinfo: &StarkInfo, id_pol: usize) -> Vec<F3G> {
    let p = get_pol_ref(ctx, starkinfo, id_pol);
    let mut res = vec![F3G::ZERO; p.deg];
    if p.dim == 1 {
        for i in 0..p.deg {
            res[i] = p.buffer[(p.offset + i * p.size)];
        }
    } else if p.dim == 3 {
        for i in 0..p.deg {
            res[i] = F3G::new(
                p.buffer[(p.offset + i * p.size)].to_be(),
                p.buffer[(p.offset + i * p.size) + 1].to_be(),
                p.buffer[(p.offset + i * p.size) + 2].to_be(),
            );
        }
    } else {
        panic!("Invalid dim {}", p.dim);
    }
    res
}

pub fn extend_and_merkelize(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    section_name: &'static str,
) -> Result<MerkleTree> {
    let nBitsExt = ctx.nbits_ext;
    let nBits = ctx.nbits;
    let n_pols = starkinfo.map_sectionsN.get(section_name);
    let columns = to_matrix(ctx, starkinfo, section_name);
    let n = columns[0].len();

    let m = interpolate_in_pil(&columns, 1 << (nBitsExt - nBits));

    Ok(MerkleTree::merkelize(m, n << (nBitsExt - nBits), n_pols)?)
}

pub fn extend(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    section_name: &'static str,
) -> Result<Vec<F3G>> {
    let nBitsExt = ctx.nbits_ext;
    let nBits = ctx.nbits;
    let n_pols = starkinfo.map_sectionsN.get(section_name);
    let columns = to_matrix(ctx, starkinfo, section_name);
    let n = columns[0].len();

    let m = interpolate_in_pil(&columns, 1 << (nBitsExt - nBits));
    Ok(to_array(&m))
}

pub fn merkelize(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    section_name: &'static str,
) -> Result<MerkleTree> {
    let nBitsExt = ctx.nbits_ext;
    let nBits = ctx.nbits;
    let n_pols = starkinfo.map_sectionsN.get(section_name);
    let columns = to_matrix(ctx, starkinfo, section_name);
    let n = columns[0].len();

    Ok(MerkleTree::merkelize(
        columns,
        n << (nBitsExt - nBits),
        n_pols,
    )?)
}

fn to_matrix(
    ctx: &mut StarkContext,
    starkinfo: &StarkInfo,
    section_name: &'static str,
) -> Vec<Vec<BaseElement>> {
    let n_pols = starkinfo.map_sectionsN.get(section_name);
    let p = ctx.get_mut(section_name);
    let n = p.len() / n_pols; // width
    let mut columns: Vec<Vec<BaseElement>> = vec![Vec::new(); n_pols];

    for i in 0..n_pols {
        columns[i] = vec![BaseElement::ZERO; n];
        for j in 0..n {
            columns[i][j] = p[i * n_pols + j].to_be();
        }
    }
    columns
}

fn to_array(columns: &Vec<Vec<BaseElement>>) -> Vec<F3G> {
    let mut res = vec![F3G::ZERO; columns.len() * columns[0].len()];
    let n_pols = columns.len();
    let n = columns[0].len();
    for i in 0..n_pols {
        for j in 0..n {
            res[i * n_pols + j] = columns[i][j].into();
        }
    }
    res
}

pub fn calculate_exps(ctx: &mut StarkContext, starkinfo: &StarkInfo, seg: &Segment, dom: &str) {
    ctx.tmp = vec![F3G::ZERO; seg.tmp_used];

    let cFirst = compile_code(ctx, starkinfo, &seg.first, "n", true);
    let cI = compile_code(ctx, starkinfo, &seg.first, "n", true);
    let cLast = compile_code(ctx, starkinfo, &seg.first, "n", true);

    let next = if dom == "n" {
        1
    } else {
        1 << (ctx.nbits_ext - ctx.nbits)
    };
    let N = if dom == "n" { ctx.N } else { ctx.Next };

    for i in 0..next {
        cFirst.eval(ctx, i);
    }

    for i in next..(N - next) {
        // cI(ctx, i);
        cFirst.eval(ctx, i);
    }
    for i in (N - next)..N {
        // cLast(ctx, i);
        cFirst.eval(ctx, i);
    }
}

#[cfg(test)]
pub mod tests {
    use crate::constant::SHIFT;
    use crate::f3g::F3G;
    use crate::polsarray::{PolKind, PolsArray};
    use crate::stark_gen::StarkProof;
    use crate::stark_setup::StarkSetup;
    use crate::types::load_json;
    use crate::types::{StarkStruct, PIL};
    use winter_math::{fft, fields::f64::BaseElement};
    use winter_math::{FieldElement, StarkField};

    #[test]
    fn test_fft() {
        let expected: Vec<BaseElement> = vec![1u32, 2u32, 3u32, 5u32]
            .iter()
            .map(|e| BaseElement::from(*e))
            .collect();
        let mut points = expected.clone();

        // FFT
        let twiddles = fft::get_twiddles(4);
        fft::evaluate_poly(&mut points, &twiddles);
        //println!("eoff {:?} {:?}", points[0].as_int(), points[1].as_int());

        // IFFT
        let inv_twiddles = fft::get_inv_twiddles(4);
        fft::interpolate_poly(&mut points, &inv_twiddles);
        assert_eq!(expected, points);
    }
    #[test]
    fn test_stark_gen() {
        let mut pil = load_json::<PIL>("data/fib.pil.json.2").unwrap();
        let mut const_pol = PolsArray::new(&pil, PolKind::Constant, 32);
        const_pol.load("data/fib.const.2").unwrap();

        let mut cm_pol = PolsArray::new(&pil, PolKind::Commit, 32);
        cm_pol.load("data/fib.cm.2");

        let stark_struct = load_json::<StarkStruct>("data/starkStruct.json.2").unwrap();
        let setup = StarkSetup::new(&const_pol, &mut pil, &stark_struct).unwrap();

        let starkproof = StarkProof::stark_gen(
            &cm_pol,
            &const_pol,
            &setup.const_tree,
            &setup.starkinfo,
            &setup.program,
            &pil,
            &stark_struct,
        )
        .unwrap();
    }
}
