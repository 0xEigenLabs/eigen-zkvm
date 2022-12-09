use crate::field_bn128::Fr;
use crate::poseidon_bn128::Constants;
use crate::poseidon_bn128_constants_opt as constants;
use ff::from_hex;

pub fn load_constants() -> Constants {
    let (c_str, m_str, p_str, s_str) = constants::constants();
    let mut c: Vec<Vec<Fr>> = Vec::new();
    for v1 in c_str {
        let mut cci: Vec<Fr> = Vec::new();
        for v2 in v1 {
            let b: Fr = from_hex(v2).unwrap();
            cci.push(b);
        }
        c.push(cci);
    }
    let mut m: Vec<Vec<Vec<Fr>>> = Vec::new();
    for v1 in m_str {
        let mut mi: Vec<Vec<Fr>> = Vec::new();
        for v2 in v1 {
            let mut mij: Vec<Fr> = Vec::new();
            for s in v2 {
                let b: Fr = from_hex(s).unwrap();
                mij.push(b);
            }
            mi.push(mij);
        }
        m.push(mi);
    }

    let mut p: Vec<Vec<Vec<Fr>>> = Vec::new();
    for v1 in p_str {
        let mut mi: Vec<Vec<Fr>> = Vec::new();
        for v2 in v1 {
            let mut mij: Vec<Fr> = Vec::new();
            for s in v2 {
                let b: Fr = from_hex(s).unwrap();
                mij.push(b);
            }
            mi.push(mij);
        }
        p.push(mi);
    }

    let mut s: Vec<Vec<Fr>> = Vec::new();
    for v1 in s_str {
        let mut cci: Vec<Fr> = Vec::new();
        for v2 in v1 {
            let b: Fr = from_hex(v2).unwrap();
            cci.push(b);
        }
        s.push(cci);
    }

    Constants {
        c,
        m,
        p,
        s,
        n_rounds_f: 8,
        n_rounds_p: vec![
            56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
        ],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_constant_opt() {}
}
