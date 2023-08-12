#[cfg(test)]
mod tests {
    use crate::field_gl::exp;
    use crate::field_gl::*;
    use crate::ff::*;
    use crate::rand::Rand;
    use num_bigint::BigUint;
    use proptest::prelude::*;
    const MODULUS: FrRepr = FrRepr([18446744069414584321u64]);
    const ROOT_OF_UNITY: FrRepr = FrRepr([959634606461954525u64]);
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
        let mut a: FrRepr = MODULUS.clone();
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
        let mut a = MODULUS.clone();
        a.sub_noborrow(&FrRepr::from(2));
        let expected = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(expected, Fr::from_str("3").unwrap() - Fr::from_str("5").unwrap());
    }

    #[test]
    fn neg() {
        assert_eq!(Fr::zero(), -Fr::zero());
        let mut a = MODULUS.clone();
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
        let mut a = MODULUS.clone();
        a.sub_noborrow(&FrRepr::from(1));
        let t1 = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(Fr::one(), t1 * t1);
        a.sub_noborrow(&FrRepr::from(1));
        let t2 = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(t2, t1 * Fr::from_str("2").unwrap());
        a.sub_noborrow(&FrRepr::from(2));
        let t3 = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(t3, t1 * Fr::from_str("4").unwrap());

        a = MODULUS.clone();
        a.add_nocarry(&FrRepr::from(1));
        a.div2();
        let t = Fr::from_str(&a.0[0].to_string()).unwrap();
        assert_eq!(
            Fr::one(),
            t * Fr::from_str("2").unwrap()
        );
    }

    // #[test]
    // fn mul_small() {
    //     // test overflow
    //     let mut m = MODULUS.clone();
    //     m.sub_noborrow(&FrRepr::from(1));
    //     let t = Fr::from_str(&m.0[0].to_string()).unwrap();
    //     let a = u32::MAX;
    //     let expected = BaseElement::new(a as u64) * t;
    //     assert_eq!(expected, t.mul_small(a));
    // }

    #[test]
    fn exp_test() {
        let a = Fr::zero();
        assert_eq!(exp(a,0), Fr::one());
        assert_eq!(exp(a,1), Fr::zero());

        let a = Fr::one();
        assert_eq!(exp(a,0), Fr::one());
        assert_eq!(exp(a,1), Fr::one());
        assert_eq!(exp(a,3), Fr::one());
        assert_eq!(exp(a,7), Fr::one());

        let mut rng = rand::thread_rng();
        let a = Fr::rand(&mut rng);
        assert_eq!(exp(a,3), a * a * a);
    }

    #[test]
    fn inv() {
        // identity
        assert_eq!(Fr::one(), Fr::one().inverse().unwrap());
        assert_eq!(Fr::zero(), Fr::zero().inverse().unwrap());
    }

    #[test]
    // fn element_as_int() {
    fn element_as_u64() {
        let a = u32::MAX;
        let b = a as u64;
        let v = u64::MAX - b + 1;
        let e = Fr::from_str(&v.to_string()).unwrap();
        assert_eq!(v % MODULUS.0[0], e.into_repr().0[0]);

        let e1 = Fr::zero();
        let e2: Fr = Fr::from_str(&MODULUS.0[0].to_string()).unwrap();
        assert_eq!(e1.into_repr().0[0], e2.into_repr().0[0]);
    }

    #[test]
    fn equals() {
        let a = Fr::one();
        let mut m: FrRepr = MODULUS.clone();
        m.sub_noborrow(&FrRepr::from(1));
        let t = Fr::from_str(&m.0[0].to_string()).unwrap();
        let b = t * t;

        // elements are equal
        assert_eq!(a, b);
        assert_eq!(a.into_repr().0[0], b.into_repr().0[0]);
        // assert_eq!(a.to_bytes(), b.to_bytes());
    }

    // ROOTS OF UNITY
    // ------------------------------------------------------------------------------------------------

    #[test]
    fn get_root_of_unity() {
        let root = Fr::root_of_unity();
        assert_eq!(Fr(ROOT_OF_UNITY), root);
        assert_eq!(Fr::one(), exp(root, 1u64 << 32));

        // let root_31 = BaseElement::get_root_of_unity(31);
        // let expected = root_32.exp(2);
        // assert_eq!(expected, root_31);
        // assert_eq!(Fr::one(), root_31.exp(1u64 << 31));
    }

    // SERIALIZATION AND DESERIALIZATION
    // ------------------------------------------------------------------------------------------------

    // #[test]
    // fn from_u128() {
    //     let v = u128::MAX;
    //     let e = BaseElement::from(v);
    //     assert_eq!((v % super::M as u128) as u64, e.as_int());
    // }

    // #[test]
    // fn try_from_slice() {
    //     let bytes = vec![1, 0, 0, 0, 0, 0, 0, 0];
    //     let result = BaseElement::try_from(bytes.as_slice());
    //     assert!(result.is_ok());
    //     assert_eq!(1, result.unwrap().as_int());

    //     let bytes = vec![1, 0, 0, 0, 0, 0, 0];
    //     let result = BaseElement::try_from(bytes.as_slice());
    //     assert!(result.is_err());

    //     let bytes = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];
    //     let result = BaseElement::try_from(bytes.as_slice());
    //     assert!(result.is_err());

    //     let bytes = vec![255, 255, 255, 255, 255, 255, 255, 255];
    //     let result = BaseElement::try_from(bytes.as_slice());
    //     assert!(result.is_err());
    // }

    // #[test]
    // fn elements_as_bytes() {
    //     let source = vec![
    //         BaseElement::new(1),
    //         Fr::from_str("2").unwrap(),
    //         Fr::from_str("3").unwrap(),
    //         BaseElement::new(4),
    //     ];

    //     let mut expected = vec![];
    //     expected.extend_from_slice(&source[0].0.to_le_bytes());
    //     expected.extend_from_slice(&source[1].0.to_le_bytes());
    //     expected.extend_from_slice(&source[2].0.to_le_bytes());
    //     expected.extend_from_slice(&source[3].0.to_le_bytes());

    //     assert_eq!(expected, BaseElement::elements_as_bytes(&source));
    // }

    // #[test]
    // fn bytes_as_elements() {
    //     let elements = vec![
    //         BaseElement::new(1),
    //         Fr::from_str("2").unwrap(),
    //         Fr::from_str("3").unwrap(),
    //         BaseElement::new(4),
    //     ];

    //     let mut bytes = vec![];
    //     bytes.extend_from_slice(&elements[0].0.to_le_bytes());
    //     bytes.extend_from_slice(&elements[1].0.to_le_bytes());
    //     bytes.extend_from_slice(&elements[2].0.to_le_bytes());
    //     bytes.extend_from_slice(&elements[3].0.to_le_bytes());
    //     bytes.extend_from_slice(&Fr::from_str("5").unwrap().0.to_le_bytes());

    //     let result = unsafe { BaseElement::bytes_as_elements(&bytes[..32]) };
    //     assert!(result.is_ok());
    //     assert_eq!(elements, result.unwrap());

    //     let result = unsafe { BaseElement::bytes_as_elements(&bytes[..33]) };
    //     assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));

    //     let result = unsafe { BaseElement::bytes_as_elements(&bytes[1..33]) };
    //     assert!(matches!(result, Err(DeserializationError::InvalidValue(_))));
    // }

    // INITIALIZATION
    // ------------------------------------------------------------------------------------------------

    // #[test]
    // fn zeroed_vector() {
    //     let result = Fr::zero()ed_vector(4);
    //     assert_eq!(4, result.len());
    //     for element in result.into_iter() {
    //         assert_eq!(Fr::zero(), element);
    //     }
    // }

    // QUADRATIC EXTENSION
    // ------------------------------------------------------------------------------------------------
    // #[test]
    // fn quad_mul() {
    //     // identity
    //     let r: QuadExtension<BaseElement> = rand_value();
    //     assert_eq!(
    //         <QuadExtension<BaseElement>>::ZERO,
    //         r * <QuadExtension<BaseElement>>::ZERO
    //     );
    //     assert_eq!(r, r * <QuadExtension<BaseElement>>::ONE);

    //     // test multiplication within bounds
    //     let a = <QuadExtension<BaseElement>>::new(Fr::from_str("3").unwrap(), Fr::one());
    //     let b = <QuadExtension<BaseElement>>::new(BaseElement::new(4), Fr::from_str("2").unwrap());
    //     let expected = <QuadExtension<BaseElement>>::new(BaseElement::new(8), BaseElement::new(12));
    //     assert_eq!(expected, a * b);

    //     // test multiplication with overflow
    //     let m = BaseElement::MODULUS;
    //     let a = <QuadExtension<BaseElement>>::new(Fr::from_str("3").unwrap(), BaseElement::new(m - 1));
    //     let b = <QuadExtension<BaseElement>>::new(BaseElement::new(m - 3), Fr::from_str("5").unwrap());
    //     let expected = <QuadExtension<BaseElement>>::new(Fr::one(), BaseElement::new(13));
    //     assert_eq!(expected, a * b);

    //     let a = <QuadExtension<BaseElement>>::new(Fr::from_str("3").unwrap(), BaseElement::new(m - 1));
    //     let b = <QuadExtension<BaseElement>>::new(BaseElement::new(10), BaseElement::new(m - 2));
    //     let expected = <QuadExtension<BaseElement>>::new(
    //         BaseElement::new(26),
    //         BaseElement::new(18446744069414584307),
    //     );
    //     assert_eq!(expected, a * b);
    // }

// #[test]
// fn quad_mul_base() {
//     let a = <QuadExtension<BaseElement>>::new(rand_value(), rand_value());
//     let b0 = rand_value();
//     let b = <QuadExtension<BaseElement>>::new(b0, Fr::zero());

//     let expected = a * b;
//     assert_eq!(expected, a.mul_base(b0));
// }

// #[test]
// fn quad_conjugate() {
//     let m = BaseElement::MODULUS;

//     let a = <QuadExtension<BaseElement>>::new(BaseElement::new(m - 1), Fr::from_str("3").unwrap());
//     let expected = <QuadExtension<BaseElement>>::new(
//         Fr::from_str("2").unwrap(),
//         BaseElement::new(18446744069414584318),
//     );
//     assert_eq!(expected, a.conjugate());

//     let a = <QuadExtension<BaseElement>>::new(BaseElement::new(m - 3), BaseElement::new(m - 2));
//     let expected = <QuadExtension<BaseElement>>::new(
//         BaseElement::new(18446744069414584316),
//         Fr::from_str("2").unwrap(),
//     );
//     assert_eq!(expected, a.conjugate());

//     let a = <QuadExtension<BaseElement>>::new(BaseElement::new(4), BaseElement::new(7));
//     let expected = <QuadExtension<BaseElement>>::new(
//         BaseElement::new(11),
//         BaseElement::new(18446744069414584314),
//     );
//     assert_eq!(expected, a.conjugate());
// }

// // CUBIC EXTENSION
// // ------------------------------------------------------------------------------------------------
// #[test]
// fn cube_mul() {
//     // identity
//     let r: CubeExtension<BaseElement> = rand_value();
//     assert_eq!(
//         <CubeExtension<BaseElement>>::ZERO,
//         r * <CubeExtension<BaseElement>>::ZERO
//     );
//     assert_eq!(r, r * <CubeExtension<BaseElement>>::ONE);

//     // test multiplication within bounds
//     let a = <CubeExtension<BaseElement>>::new(
//         Fr::from_str("3").unwrap(),
//         Fr::from_str("5").unwrap(),
//         Fr::from_str("2").unwrap(),
//     );
//     let b = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(320),
//         BaseElement::new(68),
//         Fr::from_str("3").unwrap(),
//     );
//     let expected = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(1111),
//         BaseElement::new(1961),
//         BaseElement::new(995),
//     );
//     assert_eq!(expected, a * b);

//     // test multiplication with overflow
//     let a = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(18446744069414584267),
//         BaseElement::new(18446744069414584309),
//         BaseElement::new(9223372034707292160),
//     );
//     let b = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(18446744069414584101),
//         BaseElement::new(420),
//         BaseElement::new(18446744069414584121),
//     );
//     let expected = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(14070),
//         BaseElement::new(18446744069414566571),
//         BaseElement::new(5970),
//     );
//     assert_eq!(expected, a * b);

//     let a = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(18446744069414584266),
//         BaseElement::new(18446744069412558094),
//         BaseElement::new(5268562),
//     );
//     let b = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(18446744069414583589),
//         BaseElement::new(1226),
//         BaseElement::new(5346),
//     );
//     let expected = <CubeExtension<BaseElement>>::new(
//         BaseElement::new(18446744065041672051),
//         BaseElement::new(25275910656),
//         BaseElement::new(21824696736),
//     );
//     assert_eq!(expected, a * b);
// }

// #[test]
// fn cube_mul_base() {
//     let a = <CubeExtension<BaseElement>>::new(rand_value(), rand_value(), rand_value());
//     let b0 = rand_value();
//     let b = <CubeExtension<BaseElement>>::new(b0, Fr::zero(), Fr::zero());

//     let expected = a * b;
//     assert_eq!(expected, a.mul_base(b0));
// }

    // RANDOMIZED TESTS
    // ================================================================================================

    proptest! {
        #[test]
        fn add_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let result = v1 + v2;
            let m = MODULUS.clone().0[0];
            let expected = (((a as u128) + (b as u128)) % (m as u128)) as u64;
            prop_assert_eq!(expected, result.into_repr().0[0]);
        }

        #[test]
        fn sub_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let result = v1 - v2;
            let m = MODULUS.clone().0[0];
            let a = a % m; 
            let b = b % m;
            let expected = if a < b { m - b + a } else { a - b };

            prop_assert_eq!(expected, result.into_repr().0[0]);
        }

        #[test]
        fn neg_proptest(a in any::<u64>()) {
            let v = Fr::from_str(&a.to_string()).unwrap();
            let m = MODULUS.clone().0[0];
            let expected = m - (a % m);

            prop_assert_eq!(expected, (-v).into_repr().0[0]);
        }

        #[test]
        fn mul_proptest(a in any::<u64>(), b in any::<u64>()) {
            let v1 = Fr::from_str(&a.to_string()).unwrap();
            let v2 = Fr::from_str(&b.to_string()).unwrap();
            let result = v1 * v2;
            let m = MODULUS.clone().0[0];
            
            let expected = ((a as u128) * (b as u128) % (m as u128))as u64;
            prop_assert_eq!(expected, result.into_repr().0[0]);
        }

        // #[test]
        // fn mul_small_proptest(a in any::<u64>(), b in any::<u32>()) {
        //     let v1 = BaseElement::from(a);
        //     let v2 = b;
        //     let result = v1.mul_small(v2);

        //     let expected = (((a as u128) * (b as u128)) % super::M as u128) as u64;
        //     prop_assert_eq!(expected, result.as_int());
        // }

        #[test]
        fn double_proptest(x in any::<u64>()) {
            let mut v = Fr::from_str(&x.to_string()).unwrap();
            v.0.mul2();
            let m = MODULUS.clone().0[0];

            let expected = (((x as u128) * 2) % m as u128) as u64;
            prop_assert_eq!(expected, v.into_repr().0[0]);
        }

        #[test]
        fn exp_proptest(a in any::<u64>(), b in any::<u64>()) {
            let result = exp(Fr::from_str(&a.to_string()).unwrap(), b);
            
            let b = BigUint::from(b);
            let _m = MODULUS.clone().0[0];
            let m = BigUint::from(_m);
            let expected = BigUint::from(a).modpow(&b, &m).to_u64_digits()[0];
            prop_assert_eq!(expected, result.into_repr().0[0]);
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
            let m = MODULUS.clone().0[0];
            prop_assert_eq!(a % m, e.into_repr().0[0]);
        }

        #[test]
        fn from_u128_proptest(v in any::<u128>()) {
            let e = Fr::from_str(&v.to_string()).unwrap();
            let m = MODULUS.clone().0[0];
            assert_eq!((v % m as u128) as u64, e.into_repr().0[0]);
        }

//     // QUADRATIC EXTENSION
//     // --------------------------------------------------------------------------------------------
//     #[test]
//     fn quad_mul_inv_proptest(a0 in any::<u64>(), a1 in any::<u64>()) {
//         let a = QuadExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1));
//         let b = a.inv();

//         let expected = if a == QuadExtension::<BaseElement>::ZERO {
//             QuadExtension::<BaseElement>::ZERO
//         } else {
//             QuadExtension::<BaseElement>::ONE
//         };
//         prop_assert_eq!(expected, a * b);
//     }

//     #[test]
//     fn quad_square_proptest(a0 in any::<u64>(), a1 in any::<u64>()) {
//         let a = QuadExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1));
//         let expected = a * a;

//         prop_assert_eq!(expected, a.square());
//     }

//     // CUBIC EXTENSION
//     // --------------------------------------------------------------------------------------------
//     #[test]
//     fn cube_mul_inv_proptest(a0 in any::<u64>(), a1 in any::<u64>(), a2 in any::<u64>()) {
//         let a = CubeExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1), BaseElement::from(a2));
//         let b = a.inv();

//         let expected = if a == CubeExtension::<BaseElement>::ZERO {
//             CubeExtension::<BaseElement>::ZERO
//         } else {
//             CubeExtension::<BaseElement>::ONE
//         };
//         prop_assert_eq!(expected, a * b);
//     }

//     #[test]
//     fn cube_square_proptest(a0 in any::<u64>(), a1 in any::<u64>(), a2 in any::<u64>()) {
//         let a = CubeExtension::<BaseElement>::new(BaseElement::from(a0), BaseElement::from(a1), BaseElement::from(a2));
//         let expected = a * a;

//         prop_assert_eq!(expected, a.square());
//     }
}
}