use crate::compressor12::compressor12_pil;
use crate::errors::{EigenError, Result};
use crate::polsarray;
use crate::r1cs2plonk::{r1cs2plonk, PlonkAdd, PlonkGate};
use crate::types::PIL;
use plonky::circom_circuit::R1CS;
use plonky::scalar_gl::GL;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub struct Options {
    pub force_bits: usize,
}

fn get_normal_plonkinfo(
    r1cs: &R1CS<GL>,
    pa: &Vec<PlonkGate>,
    pc: &Vec<PlonkAdd>,
) -> (usize, usize, usize, usize) {
    let mut uses: HashMap<String, usize> = HashMap::new();
    for (i, c) in pa.iter().enumerate() {
        if (i % 10000) == 0 {
            println!("Plonk info constraint processing... {}/{}", i, pa.len());
        }
        let k = c.str_key();
        if uses.get(&k).is_none() {
            uses.insert(k.clone(), 0);
        }
        *uses.get_mut(&k).unwrap() += 1;
    }
    let mut result = uses
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect::<Vec<(String, usize)>>();
    result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut N = 0;
    result.iter().for_each(|e| {
        N += (e.1 - 1) / 4 + 1;
    });

    // N, nConstaints, nPlonkGates, nPlonkAdds
    (N, r1cs.constraints.len(), pa.len(), pc.len())
}

fn get_custom_gate_info(
    r1cs: &R1CS<GL>,
    pa: &Vec<PlonkGate>,
    pc: &Vec<PlonkAdd>,
) -> (u64, u64, u64, u64) {
    let mut cmul_id = 0;
    let mut cmds_id = 0;
    let mut bcmul = false;
    let mut bcmds = false;
    assert_eq!(r1cs.custom_gates.len(), 2);
    for (i, c) in r1cs.custom_gates.iter().enumerate() {
        match c.template_name.as_str() {
            "CMul" => {
                cmul_id = i as u64;
                bcmul = true;
                assert!(c.parameters.len() == 0);
            }
            "MDS" => {
                cmds_id = i as u64;
                bcmds = true;
                assert!(c.parameters.len() == 0);
            }
            _ => panic!("Invalid custom gate {}", c.template_name),
        }
    }
    if !bcmul {
        panic!("CMul custom gate not defined");
    }
    if !bcmds {
        panic!("cmds_id custom gate not defined");
    }

    let mut n_cmul = 0;
    let mut n_mds = 0;
    for (i, c) in r1cs.custom_gates_uses.iter().enumerate() {
        if c.id == cmul_id {
            n_cmul += 1;
        } else if c.id == cmds_id {
            n_mds += 1;
        } else {
            panic!("Custom gate not defined {}", c.id);
        }
    }
    (cmul_id, cmds_id, n_cmul, n_mds)
}

pub fn plonk_setup_render(r1cs: &R1CS<GL>, opts: &Options, com_pil_file: &String) {
    let pc = r1cs2plonk(r1cs);
    let plonkinfo = get_normal_plonkinfo(r1cs, &pc.0, &pc.1);
    let custom_gates_info = get_custom_gate_info(r1cs, &pc.0, &pc.1);

    let n_publics = r1cs.num_inputs + r1cs.num_outputs;
    let n_public_rows = (n_publics - 1) / 12 + 1;

    let n_uses = n_public_rows + plonkinfo.0 as usize /*N*/ + custom_gates_info.2 as usize +  custom_gates_info.3 as usize * 2;
    let mut n_bits = crate::helper::log2_any(n_uses - 1) + 1;
    if opts.force_bits > 0 {
        n_bits = opts.force_bits;
    }
    let com_pil = compressor12_pil::render(n_bits, n_publics);
    let mut file = File::create(&com_pil_file).unwrap();
    write!(file, "{}", com_pil).unwrap();
}

pub fn plonk_setup_fix_compressor(r1cs: &R1CS<GL>, opts: &Options, pil: &PIL) {
    let const_pols = polsarray::PolsArray::new(pil, polsarray::PolKind::Constant);
    let n_publics = r1cs.num_inputs + r1cs.num_outputs;
    let n_public_rows = (n_publics - 1) / 12 + 1;
    let mut r = 0;
    for i in 0..n_public_rows {
        //
    }
}
