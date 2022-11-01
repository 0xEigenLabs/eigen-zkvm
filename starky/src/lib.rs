pub mod errors;
pub mod polsarray;
pub mod stark_setup;
pub mod types;

pub mod linearhash_bn128;
mod poseidon_bn128_constants;

pub mod poseidon_bn128;

mod merklehash_bn128;

mod digest_bn128;
pub use digest_bn128::ElementDigest;

#[macro_use]
extern crate ff;

#[macro_use]
extern crate lazy_static;

#[cfg(test)]
pub mod tests {
    use rand_utils::{rand_value, rand_vector};
    use winter_math::fft::evaluate_poly;
    use winter_math::fft::get_twiddles;
    use winter_math::get_power_series;
    use winter_math::log2;
    use winter_math::polynom::{self, eval};
    use winter_math::StarkField;
    use winter_math::{fields::f128::BaseElement, FieldElement};

    #[test]
    fn test_fft_eval() {
        let n = 2i32.pow(8) as usize;

        // build a random polynomial
        let mut p: Vec<BaseElement> = rand_vector(n);

        // evaluate the polynomial over the domain using regular polynomial evaluation
        let g = BaseElement::get_root_of_unity(log2(n));
        let domain = get_power_series(g, n);
        let expected = polynom::eval_many(&p, &domain);

        // evaluate the polynomial over the domain using FFT-based evaluation
        let twiddles = get_twiddles::<BaseElement>(p.len());
        evaluate_poly(&mut p, &twiddles);

        assert_eq!(expected, p);
    }
}
