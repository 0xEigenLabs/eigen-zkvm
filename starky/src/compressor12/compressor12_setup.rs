use crate::r1cs2plonk::{r1cs2plonk, PlonkAdd, PlonkGate};
use plonky::circom_circuit::R1CS;
use plonky::scalar_gl::GL;
use std::collections::HashMap;

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
) {
    let mut cmul_id = 0;
    let mut cmds_id = 0;
    //assert_eq!(r1c);
}

pub fn plonk_setup(r1cs: &R1CS<GL>, opts: &Options) {
    let pc = r1cs2plonk(r1cs);
}
