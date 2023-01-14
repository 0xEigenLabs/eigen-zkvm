use crate::f3g::F3G;
use plonky::r1cs_file::R1CSFile;
use plonky::reader::load_r1cs;
use plonky::scalar_gl::GL;

pub fn r1cs2plonk(r1cs: &R1CSFile<GL>) {

}

#[cfg(test)]
pub mod tests {
    use plonky::reader::load_r1cs;
    use plonky::scalar_gl::GL;

    #[test]
    fn test_r1cs2plonk() {
        let r1cs = load_r1cs::<GL>("/tmp//circuit.gl.r1cs");
        println!("{:?}", r1cs);
    }
}

