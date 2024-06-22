#[cfg(test)]
mod tests {
    use crate::ff::*;
    use crate::field_gl::{Goldilocks, GoldilocksRepr};
    use num_bigint::BigUint;
    use proptest::prelude::*;
    use rand::rngs::OsRng;

    #[test]
    #[allow(clippy::eq_op)]
    fn add() {
        // identity
        let r = Goldilocks::random(OsRng);
        assert_eq!(r, r + Goldilocks::ZERO);

        // test addition within bounds
        assert_eq!(
            Goldilocks::from_str_vartime("5").unwrap(),
            Goldilocks::from_str_vartime("2").unwrap() + Goldilocks::from_str_vartime("3").unwrap()
        );

        // test overflow
        let b = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16)
            .unwrap()
            .to_string();
        let t = Goldilocks::from_str_vartime(&b).unwrap();
        assert_eq!(Goldilocks::ZERO, t - Goldilocks::ONE + Goldilocks::ONE);
    }

    #[test]
    fn sub() {
        // identity
        let r = Goldilocks::random(OsRng);
        assert_eq!(r, r - Goldilocks::ZERO);

        // test subtraction within bounds
        assert_eq!(
            Goldilocks::from_str_vartime("2").unwrap(),
            Goldilocks::from_str_vartime("5").unwrap() - Goldilocks::from_str_vartime("3").unwrap()
        );

        // test underflow
        let b = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16)
            .unwrap()
            .to_string();
        let t = Goldilocks::from_str_vartime(&b).unwrap();
        let expected = t - Goldilocks::from_str_vartime("2").unwrap();
        assert_eq!(
            expected,
            Goldilocks::from_str_vartime("3").unwrap() - Goldilocks::from_str_vartime("5").unwrap()
        );
    }

    #[test]
    fn neg() {
        assert_eq!(Goldilocks::ZERO, -Goldilocks::ZERO);
        let b = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16)
            .unwrap()
            .to_string();
        let t =
            Goldilocks::from_str_vartime(&b).unwrap() - Goldilocks::from_str_vartime("1").unwrap();
        assert_eq!(t, -Goldilocks::ONE);

        let r = Goldilocks::random(OsRng);
        assert_eq!(r, -(-r));
    }

    #[test]
    fn mul() {
        // identity
        let r = Goldilocks::random(OsRng);
        assert_eq!(Goldilocks::ZERO, r * Goldilocks::ZERO);
        assert_eq!(r, r * Goldilocks::ONE);

        // test multiplication within bounds
        assert_eq!(
            Goldilocks::from_str_vartime("15").unwrap(),
            Goldilocks::from_str_vartime("5").unwrap() * Goldilocks::from_str_vartime("3").unwrap()
        );

        // test overflow
        let b = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16)
            .unwrap()
            .to_string();
        let t1 = Goldilocks::from_str_vartime(&b).unwrap() - Goldilocks::ONE;
        assert_eq!(Goldilocks::ONE, t1 * t1);
        let t2 = t1 - Goldilocks::ONE;
        assert_eq!(t2, t1 * Goldilocks::from_str_vartime("2").unwrap());
    }

    #[test]
    fn exp_test() {
        let a = Goldilocks::ZERO;
        assert_eq!(a.pow([0]), Goldilocks::ONE);
        assert_eq!(a.pow([1]), Goldilocks::ZERO);

        let a = Goldilocks::ONE;
        assert_eq!(a.pow([0]), Goldilocks::ONE);
        assert_eq!(a.pow([1]), Goldilocks::ONE);
        assert_eq!(a.pow([3]), Goldilocks::ONE);
        assert_eq!(a.pow([7]), Goldilocks::ONE);

        let a = Goldilocks::random(OsRng);
        assert_eq!(a.pow([3]), a * a * a);
    }

    #[test]
    fn inv() {
        // identity
        assert_eq!(Goldilocks::ONE, Goldilocks::ONE.invert().unwrap());
        // Caution: Do not call `invert()` on `Goldilocks::ZERO`
        // assert_eq!(Goldilocks::ONE, Goldilocks::ZERO.invert().unwrap());
    }

    pub fn render_repr_to_str(repr: GoldilocksRepr) -> u64 {
        let hex_str = repr
            .as_ref()
            .iter()
            .rev()
            .map(|byte| format!("{:02X}", byte))
            .collect::<String>();
        u64::from_str_radix(&hex_str, 16).unwrap()
    }

    #[test]
    fn element_as_int() {
        let a = u32::MAX;
        let b = a as u64;
        let v = u64::MAX - b + 1;
        let e = Goldilocks::from(v).to_repr();
        assert_eq!(
            render_repr_to_str(Goldilocks::ZERO.into()),
            render_repr_to_str(e)
        );
    }

    #[test]
    fn equals() {
        let a = Goldilocks::ONE;
        let b = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16)
            .unwrap()
            .to_string();
        let t1 = Goldilocks::from_str_vartime(&b).unwrap() - Goldilocks::ONE;
        let b = t1 * t1;

        // elements are equal
        assert_eq!(a, b);
        assert_eq!(
            render_repr_to_str(a.to_repr()),
            render_repr_to_str(b.to_repr())
        );
    }

    // ROOTS OF UNITY
    // ------------------------------------------------------------------------------------------------

    #[test]
    fn get_fixed_value() {
        let modulus = Goldilocks::MODULUS;
        println!("MODULUS: {:?}", modulus);
        let num_bits = Goldilocks::NUM_BITS;
        println!("NUM_BITS: {:?}", num_bits);
        let capacity = Goldilocks::CAPACITY;
        println!("CAPACITY: {:?}", capacity);
        let two_inv = Goldilocks::TWO_INV;
        println!("TWO_INV: {:?}", render_repr_to_str(two_inv.to_repr()));
        let multiplicative_generator = Goldilocks::MULTIPLICATIVE_GENERATOR;
        println!(
            "MULTIPLICATIVE_GENERATOR: {:?}",
            render_repr_to_str(multiplicative_generator.to_repr())
        );
        let s = Goldilocks::S;
        println!("s: {:?}", s);
        let root = Goldilocks::ROOT_OF_UNITY;
        println!("root: {:?}", render_repr_to_str(root.to_repr()));
        let root_of_unity_inv = Goldilocks::ROOT_OF_UNITY_INV;
        println!(
            "root_of_unity_inv: {:?}",
            render_repr_to_str(root_of_unity_inv.to_repr())
        );
        let delta = Goldilocks::DELTA;
        println!("DELTA: {:?}", render_repr_to_str(delta.to_repr()));

        assert_eq!(Goldilocks::ONE, root.pow([1u64 << 32]));
    }

    // RANDOMIZED TESTS
    // ================================================================================================

    proptest! {
        #[test]
        fn add_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Goldilocks::from(a);
            let v2 = Goldilocks::from(b);
            let result = v1 + v2;
            let m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();
            let expected = (((a as u128) + (b as u128)) % (m as u128)) as u64;
            prop_assert_eq!(expected, render_repr_to_str(result.to_repr()));
        }

        #[test]
        fn sub_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Goldilocks::from(a);
            let v2 = Goldilocks::from(b);
            let result = v1 - v2;
            let m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();
            let a = a % m;
            let b = b % m;
            let expected = if a < b { m - b + a } else { a - b };

            prop_assert_eq!(expected, render_repr_to_str(result.to_repr()));
        }

        #[test]
        fn neg_proptest(a in any::<u64>()) {
            let v = Goldilocks::from(a);
            let m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();
            let expected = m - (a % m);

            prop_assert_eq!(expected, render_repr_to_str((-v).to_repr()));
        }

        #[test]
        fn mul_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Goldilocks::from(a);
            let v2 = Goldilocks::from(b);
            let result = v1 * v2;
            let m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();

            let expected = ((a as u128) * (b as u128) % (m as u128))as u64;
            prop_assert_eq!(expected, render_repr_to_str(result.to_repr()));
        }

        #[test]
        fn double_proptest(x in any::<u64>()) {
            let v = Goldilocks::from(x).double();
            let m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();

            let expected = (((x as u128) * 2) % m as u128) as u64;
            prop_assert_eq!(render_repr_to_str(v.to_repr()), expected);
        }

        #[test]
        fn exp_proptest(a in any::<u64>(), b in any::<u64>()) {
            let result = Goldilocks::from(a).pow([b]);

            let b = BigUint::from(b);
            let _m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();
            let m = BigUint::from(_m);
            let expected = BigUint::from(a).modpow(&b, &m).to_u64_digits()[0];
            prop_assert_eq!(expected, render_repr_to_str(result.to_repr()));
        }

        #[test]
        fn inv_proptest(a in any::<u64>()) {
            let a = Goldilocks::from(a);
            let b = a.invert().unwrap();

            let expected = if a == Goldilocks::ZERO { Goldilocks::ZERO } else { Goldilocks::ONE };
            prop_assert_eq!(expected, a * b);
        }

        #[test]
        fn element_as_int_proptest(a in any::<u64>()) {
            let e = Goldilocks::from(a);
            let m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();
            prop_assert_eq!(a % m, render_repr_to_str(e.to_repr()));
        }

        #[test]
        fn from_u128_proptest(v in any::<u128>()) {
            let e = Goldilocks::from_str_vartime(&v.to_string()).unwrap();
            let m = u64::from_str_radix(Goldilocks::MODULUS.trim_start_matches("0x"), 16).unwrap();
            assert_eq!((v % m as u128) as u64, render_repr_to_str(e.to_repr()));
        }
    }
}
