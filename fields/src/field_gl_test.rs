#[cfg(test)]
mod tests {
    use crate::ff::*;
    use crate::field_gl::*;
    use crate::rand::Rand;
    use num_bigint::BigUint;
    use proptest::prelude::*;

    #[test]
    #[allow(clippy::eq_op)]
    fn add() {
        // identity
        let mut rng = rand::thread_rng();
        let r = Fr::rand(&mut rng);
        assert_eq!(r, r + Fr::zero());

        // test addition within bounds
        assert_eq!(
            Fr::from_str("5").unwrap(),
            Fr::from_str("2").unwrap() + Fr::from_str("3").unwrap()
        );

        // test overflow
        let mut a: FrRepr = crate::field_gl::MODULUS;
        a.sub_noborrow(&FrRepr::from(1));
        let t = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(Fr::zero(), t + Fr::one());
        assert_eq!(Fr::one(), t + Fr::from_str("2").unwrap());
    }

    #[test]
    fn sub() {
        // identity
        let mut rng = rand::thread_rng();
        let r = Fr::rand(&mut rng);
        assert_eq!(r, r - Fr::zero());

        // test subtraction within bounds
        assert_eq!(
            Fr::from_str("2").unwrap(),
            Fr::from_str("5").unwrap() - Fr::from_str("3").unwrap()
        );

        // test underflow
        let mut a = crate::field_gl::MODULUS;
        a.sub_noborrow(&FrRepr::from(2));
        let expected = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(expected, Fr::from_str("3").unwrap() - Fr::from_str("5").unwrap());
    }

    #[test]
    fn neg() {
        assert_eq!(Fr::zero(), -Fr::zero());
        let mut a = crate::field_gl::MODULUS;
        a.sub_noborrow(&FrRepr::from(1));
        let t = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(t, -Fr::one());

        let mut rng = rand::thread_rng();
        let r = Fr::rand(&mut rng);
        assert_eq!(r, -(-r));
    }

    #[test]
    fn mul() {
        // identity
        let mut rng = rand::thread_rng();
        let r = Fr::rand(&mut rng);
        assert_eq!(Fr::zero(), r * Fr::zero());
        assert_eq!(r, r * Fr::one());

        // test multiplication within bounds
        assert_eq!(
            Fr::from_str("15").unwrap(),
            Fr::from_str("5").unwrap() * Fr::from_str("3").unwrap()
        );

        // test overflow
        let mut a = crate::field_gl::MODULUS;
        a.sub_noborrow(&FrRepr::from(1));
        let t1 = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(Fr::one(), t1 * t1);
        a.sub_noborrow(&FrRepr::from(1));
        let t2 = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(t2, t1 * Fr::from_str("2").unwrap());
        a.sub_noborrow(&FrRepr::from(2));
        let t3 = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(t3, t1 * Fr::from_str("4").unwrap());

        a = crate::field_gl::MODULUS;
        a.add_nocarry(&FrRepr::from(1));
        a.div2();
        let t = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(Fr::one(), t * Fr::from_str("2").unwrap());
    }

    #[test]
    fn exp_test() {
        let a = Fr::zero();
        assert_eq!(a.exp(0), Fr::one());
        assert_eq!(a.exp(1), Fr::zero());

        let a = Fr::one();
        assert_eq!(a.exp(0), Fr::one());
        assert_eq!(a.exp(1), Fr::one());
        assert_eq!(a.exp(3), Fr::one());
        assert_eq!(a.exp(7), Fr::one());

        let mut rng = rand::thread_rng();
        let a = Fr::rand(&mut rng);
        assert_eq!(a.exp(3), a * a * a);
    }

    #[test]
    fn inv() {
        // identity
        assert_eq!(Fr::one(), Fr::one().inverse().unwrap());
        assert_eq!(Fr::zero(), Fr::zero().inverse().unwrap());
    }

    #[test]
    fn element_as_int() {
        let a = u32::MAX;
        let b = a as u64;
        let v = u64::MAX - b + 1;
        let e = Fr::from_str(&v.to_string()).unwrap();
        assert_eq!(v % crate::field_gl::MODULUS.0[0], e.as_int());

        let e1 = Fr::zero();
        let e2: Fr = Fr::from_str(&crate::field_gl::MODULUS.0[0].to_string()).unwrap();
        assert_eq!(e1.as_int(), e2.as_int());
    }

    #[test]
    fn equals() {
        let a = Fr::one();
        let mut m: FrRepr = crate::field_gl::MODULUS;
        m.sub_noborrow(&FrRepr::from(1));
        let t = Fr::from_str(&m.0[0].to_string()).unwrap();
        let b = t * t;

        // elements are equal
        assert_eq!(a, b);
        assert_eq!(a.as_int(), b.as_int());
        // assert_eq!(a.to_bytes(), b.to_bytes());
    }

    // ROOTS OF UNITY
    // ------------------------------------------------------------------------------------------------

    #[test]
    fn get_root_of_unity() {
        let root = Fr::root_of_unity();
        assert_eq!(Fr(crate::field_gl::ROOT_OF_UNITY), root);
        assert_eq!(Fr::one(), root.exp(1u64 << 32));
    }

    // RANDOMIZED TESTS
    // ================================================================================================

    proptest! {
        #[test]
        fn add_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let result = v1 + v2;
            let m = crate::field_gl::MODULUS.0[0];
            let expected = (((a as u128) + (b as u128)) % (m as u128)) as u64;
            prop_assert_eq!(expected, result.as_int());
        }

        #[test]
        fn sub_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let result = v1 - v2;
            let m = crate::field_gl::MODULUS.0[0];
            let a = a % m;
            let b = b % m;
            let expected = if a < b { m - b + a } else { a - b };

            prop_assert_eq!(expected, result.as_int());
        }

        #[test]
        fn neg_proptest(a in any::<u64>()) {
            let v = Fr::from_str(&a.to_string()).unwrap();
            let m = crate::field_gl::MODULUS.0[0];
            let expected = m - (a % m);

            prop_assert_eq!(expected, (-v).as_int());
        }

        #[test]
        fn mul_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let result = v1 * v2;
            let m = crate::field_gl::MODULUS.0[0];

            let expected = ((a as u128) * (b as u128) % (m as u128))as u64;
            prop_assert_eq!(expected, result.as_int());
        }

        #[test]
        fn double_proptest(x in any::<u64>()) {
            let mut v = Fr::from_str(&x.to_string()).unwrap();
            v.double();
            let m = crate::field_gl::MODULUS.0[0];

            let expected = (((x as u128) * 2) % m as u128) as u64;
            prop_assert_eq!(v.as_int(), expected);
        }

        #[test]
        fn exp_proptest(a in any::<u64>(), b in any::<u64>()) {
            let result = Fr::from_str(&a.to_string()).unwrap().exp( b);

            let b = BigUint::from(b);
            let _m = crate::field_gl::MODULUS.0[0];
            let m = BigUint::from(_m);
            let expected = BigUint::from(a).modpow(&b, &m).to_u64_digits()[0];
            prop_assert_eq!(expected, result.as_int());
        }

        #[test]
        fn inv_proptest(a in any::<u64>()) {
            let a = Fr::from_str(&a.to_string()).unwrap();
            let b = a.inverse().unwrap();

            let expected = if a == Fr::zero() { Fr::zero() } else { Fr::one() };
            prop_assert_eq!(expected, a * b);
        }

        #[test]
        fn element_as_int_proptest(a in any::<u64>()) {
            let e = Fr::from_str(&a.to_string()).unwrap();
            let m = crate::field_gl::MODULUS.0[0];
            prop_assert_eq!(a % m, e.as_int());
        }

        #[test]
        fn from_u128_proptest(v in any::<u128>()) {
            let e = Fr::from_str(&v.to_string()).unwrap();
            let m = crate::field_gl::MODULUS.0[0];
            assert_eq!((v % m as u128) as u64, e.as_int());
        }
    }
}
