use crate::scalar_gl::Fr;
use crate::ff::*;
use core::ops::{Add, Mul, Neg, Sub};
use rand::Rand;

impl Add for Fr {
    type Output = Fr;

    fn add(self, other: Fr) -> Fr {
        let mut result = self;
        result.add_assign(&other);
        result
    }
}

impl Sub for Fr {
    type Output = Fr;

    fn sub(self, other: Fr) -> Fr {
        let mut result = self;
        result.sub_assign(&other);
        result
    }
}

impl Mul for Fr {
    type Output = Fr;

    fn mul(self, other: Fr) -> Fr {
        let mut result = self;
        result.mul_assign(&other);
        result
    }
}

impl Neg for Fr {
    type Output = Fr;

    fn neg(self) -> Self::Output {
        let mut result = self;
        result.negate();
        result
    }
}

pub(crate) fn test_add_neg_sub_mul() {
    let mut rng = rand::thread_rng();
    let x = Fr::rand(&mut rng);
    let y = Fr::rand(&mut rng);
    let z = Fr::rand(&mut rng);
    let mut x_clone = x.clone();
    x_clone.square();
    let x_squared = x_clone;
    assert_eq!(x + (-x), Fr::zero());
    assert_eq!(-x, Fr::zero() - x);
    assert_eq!(x, x * Fr::one());
    assert_eq!(x * (-x), -x_squared);
    assert_eq!(x + y, y + x);
    assert_eq!(x * y, y * x);
    assert_eq!(x * (y * z), (x * y) * z);
    assert_eq!(x - (y + z), (x - y) - z);
    assert_eq!((x + y) - z, x + (y - z));
    assert_eq!(x * (y + z), x * y + x * z);
}

pub(crate) fn test_inv() {
    let mut rng = rand::thread_rng();
    let x = Fr::rand(&mut rng);
    let y = Fr::rand(&mut rng);
    let z = Fr::rand(&mut rng);
    let x_inversed = x.inverse().unwrap();
    assert_eq!(x, x_inversed);
    // assert_eq!(x * x_inversed, Fr::one());
    // assert_eq!(x_inversed * x, Fr::one());
    // x_clone.square();
    // let x_inversed_squared = x_clone;
    // let mut x_clone1 = x.clone();
    // x_clone1.square();
    // x_clone1.inverse();
    // let x_squared_inversed = x_clone1;
    // assert_eq!(x_squared_inversed, x_inversed_squared);
}


#[cfg(test)]
mod tests {
    use super::test_add_neg_sub_mul;
    use super::test_inv;
    use crate::ff::*;
    use crate::scalar_gl::*;
    use rand::Rand;

    #[test]
    #[allow(clippy::eq_op)]
    fn check_add_neg_sub_mul() {
        test_add_neg_sub_mul();
    }
    #[test]
    #[ignore]
    #[allow(clippy::eq_op)]
    fn check_inv() {
        test_inv();
    }
    
    #[test]
    fn test_fr_repr_ordering() {
        fn assert_equality(a: FrRepr, b: FrRepr) {
            assert_eq!(a, b);
            assert!(a.cmp(&b) == ::std::cmp::Ordering::Equal);
        }

        fn assert_lt(a: FrRepr, b: FrRepr) {
            assert!(a < b);
            assert!(b > a);
        }

        assert_equality(
            FrRepr([9999]),
            FrRepr([9999]),
        );
        assert_lt(
            FrRepr([9998]),
            FrRepr([9999]),
        );
    }

    #[test]
    fn test_fr_repr_from() {
        assert_eq!(FrRepr::from(100), FrRepr([100]));
    }

    #[test]
    fn test_fr_repr_is_odd() {
        assert!(!FrRepr::from(0).is_odd());
        assert!(FrRepr::from(0).is_even());
        assert!(FrRepr::from(1).is_odd());
        assert!(!FrRepr::from(1).is_even());
        assert!(!FrRepr::from(324834872).is_odd());
        assert!(FrRepr::from(324834872).is_even());
        assert!(FrRepr::from(324834873).is_odd());
        assert!(!FrRepr::from(324834873).is_even());
    }

    #[test]
    fn test_fr_repr_is_zero() {
        assert!(FrRepr::from(0).is_zero());
        assert!(!FrRepr::from(1).is_zero());
        assert!(FrRepr([0]).is_zero());
    }

    #[test]
    fn test_fr_repr_div2() {
        let mut a = FrRepr([
            0xcb67c072733beefc
        ]);
        a.div2();
        assert_eq!(
            a,
            FrRepr([
                0x65b3e039399df77e
            ])
        );
        for _ in 0..10 {
            a.div2();
        }
        assert_eq!(
            a,
            FrRepr([
                0x196cf80e4e677d
            ])
        );
        for _ in 0..200 {
            a.div2();
        }
        assert!(a.is_zero());
    }

    #[test]
    fn test_fr_repr_shr() {
        let mut a = FrRepr([
            0x36003ab08de70da1
        ]);
        a.shr(0);
        assert_eq!(
            a,
            FrRepr([
                0x36003ab08de70da1
            ])
        );
        a.shr(1);
        assert_eq!(
            a,
            FrRepr([
                0x1b001d5846f386d0
            ])
        );
        a.shr(50);
        assert_eq!(
            a,
            FrRepr([
                0x6c0
            ])
        );
    }

    #[test]
    fn test_fr_repr_mul2() {
        let mut a = FrRepr::from(23712937547);
        a.mul2();
        assert_eq!(a, FrRepr([0xb0acd6c96]));
        for _ in 0..60 {
            a.mul2();
        }
        assert_eq!(a, FrRepr([0x6000000000000000]));
        for _ in 0..7 {
            a.mul2();
        }
        assert!(a.is_zero());
    }

    #[test]
    fn test_fr_repr_num_bits() {
        let mut a = FrRepr::from(0);
        assert_eq!(0, a.num_bits());
        a = FrRepr::from(1);
        for i in 1..65 {
            assert_eq!(i, a.num_bits());
            a.mul2();
        }
        assert_eq!(0, a.num_bits());
    }

    #[test]
    fn test_fr_repr_sub_noborrow() {
        let mut rng = rand::thread_rng();
        let mut t = FrRepr([
            0x8e62a7e85264e2c3
        ]);
        t.sub_noborrow(&FrRepr([
            0xd64f669809cbc6a4
        ]));
        assert!(
            t == FrRepr([
                0xb813415048991c1f
            ])
        );

        for _ in 0..1000 {
            let mut a = FrRepr::rand(&mut rng);
            a.0[0] >>= 30;
            let mut b = a;
            for _ in 0..10 {
                b.mul2();
            }
            let mut c = b;
            for _ in 0..10 {
                c.mul2();
            }

            assert!(a < b);
            assert!(b < c);

            let mut csub_ba = c;
            csub_ba.sub_noborrow(&b);
            csub_ba.sub_noborrow(&a);

            let mut csub_ab = c;
            csub_ab.sub_noborrow(&a);
            csub_ab.sub_noborrow(&b);

            assert_eq!(csub_ab, csub_ba);
        }

        // Subtracting r+1 from r should produce -1 (mod 2**64)
        let mut qplusone = FrRepr([
            0xffffffff00000001
        ]);
        qplusone.sub_noborrow(&FrRepr([
            0xffffffff00000002
        ]));
        assert_eq!(
            qplusone,
            FrRepr([
                0xffffffffffffffff
            ])
        );

    }

    #[test]
    #[ignore]
    fn test_fr_legendre() {
        // assert_eq!(Fr::one().into_repr(), Fr::one().pow([4611686016279904256]).into_repr());
        assert_eq!(crate::ff::LegendreSymbol::QuadraticResidue, Fr::one().legendre());
        assert_eq!(crate::ff::LegendreSymbol::Zero, Fr::zero().legendre());

        let e = FrRepr([
            0xfffffffe00000001
        ]);
        assert_eq!(crate::ff::LegendreSymbol::QuadraticResidue, Fr::from_repr(e).unwrap().legendre());
        let e = FrRepr([
            0x96341aefd047c045
        ]);
        assert_eq!(crate::ff::LegendreSymbol::QuadraticNonResidue, Fr::from_repr(e).unwrap().legendre());
    }

    #[test]
    fn test_fr_repr_add_nocarry() {
        let mut rng = rand::thread_rng();
        let mut t = FrRepr([
            0xd64f669809cbc6a4
        ]);
        t.add_nocarry(&FrRepr([
            0x8e62a7e85264e2c3
        ]));
        assert_eq!(
            t,
            FrRepr([
                0x64b20e805c30a967
            ])
        );

        // Test for the associativity of addition.
        for _ in 0..1000 {
            let mut a = FrRepr::rand(&mut rng);
            let mut b = FrRepr::rand(&mut rng);
            let mut c = FrRepr::rand(&mut rng);

            // Unset the first few bits, so that overflow won't occur.
            a.0[0] >>= 3;
            b.0[0] >>= 3;
            c.0[0] >>= 3;

            let mut abc = a;
            abc.add_nocarry(&b);
            abc.add_nocarry(&c);

            let mut acb = a;
            acb.add_nocarry(&c);
            acb.add_nocarry(&b);

            let mut bac = b;
            bac.add_nocarry(&a);
            bac.add_nocarry(&c);

            let mut bca = b;
            bca.add_nocarry(&c);
            bca.add_nocarry(&a);

            let mut cab = c;
            cab.add_nocarry(&a);
            cab.add_nocarry(&b);

            let mut cba = c;
            cba.add_nocarry(&b);
            cba.add_nocarry(&a);

            assert_eq!(abc, acb);
            assert_eq!(abc, bac);
            assert_eq!(abc, bca);
            assert_eq!(abc, cab);
            assert_eq!(abc, cba);
        }

        // Adding 1 to (2^256 - 1) should produce zero
        let mut x = FrRepr([
            0xffffffffffffffff
        ]);
        x.add_nocarry(&FrRepr::from(1));
        assert!(x.is_zero());
    }

    #[test]
    fn test_fr_add_assign() {
        {
            // Random number
            let mut tmp = Fr(FrRepr([
                0x437ce7616d580765
            ]));
            // Test that adding zero has no effect.
            tmp.add_assign(&Fr(FrRepr::from(0)));
            assert_eq!(
                tmp,
                Fr(FrRepr([
                    0x437ce7616d580765
                ]))
            );
            // Add one and test for the result.
            tmp.add_assign(&Fr(FrRepr::from(1)));
            assert_eq!(
                tmp,
                Fr(FrRepr([
                    0x437ce7616d580766
                ]))
            );
            // Add another random number that exercises the reduction.
            tmp.add_assign(&Fr(FrRepr([
                0x946f435944f7dc79
            ])));
            assert_eq!(
                tmp,
                Fr(FrRepr([
                    0xd7ec2abab24fe3df
                ]))
            );
            // Add one to (r - 1) and test for the result.
            tmp = Fr(FrRepr([
                0xffffffff00000000
            ]));
            tmp.add_assign(&Fr(FrRepr::from(1)));
            assert!(tmp.0.is_zero());
            // Add a random number to another one such that the result is r - 1
            tmp = Fr(FrRepr([
                0xade5adacdccb6190
            ]));
            tmp.add_assign(&Fr(FrRepr([
                0x521a525223349e70
            ])));
            assert_eq!(
                tmp,
                Fr(FrRepr([
                    0xffffffff00000000
                ]))
            );
            // Add one to the result and test for it.
            tmp.add_assign(&Fr(FrRepr::from(1)));
            assert!(tmp.0.is_zero());
        }

        // Test associativity
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Generate a, b, c and ensure (a + b) + c == a + (b + c).
            let a = Fr::rand(&mut rng);
            let b = Fr::rand(&mut rng);
            let c = Fr::rand(&mut rng);

            let mut tmp1 = a;
            tmp1.add_assign(&b);
            tmp1.add_assign(&c);

            let mut tmp2 = b;
            tmp2.add_assign(&c);
            tmp2.add_assign(&a);
            assert_eq!(tmp1, tmp2);
        }
    }

    #[test]
    fn test_fr_sub_assign() {
        {
            // Test arbitrary subtraction that tests reduction.
            let mut tmp = Fr(FrRepr([
                0x6a68c64b6f735a2b
            ]));
            tmp.sub_assign(&Fr(FrRepr([
                0xade5adacdccb6190
            ])));
            assert_eq!(
                tmp,
                Fr(FrRepr([
                    0xbc83189d92a7f89c
                ]))
            );

            // Test the opposite subtraction which doesn't test reduction.
            tmp = Fr(FrRepr([
                0xade5adacdccb6190
            ]));
            tmp.sub_assign(&Fr(FrRepr([
                0x6a68c64b6f735a2b
            ])));
            assert_eq!(
                tmp,
                Fr(FrRepr([
                    0x437ce7616d580765
                ]))
            );

            // Test for sensible results with zero
            tmp = Fr(FrRepr::from(0));
            tmp.sub_assign(&Fr(FrRepr::from(0)));
            assert!(tmp.is_zero());

            tmp = Fr(FrRepr([
                0x437ce7616d580765
            ]));
            tmp.sub_assign(&Fr(FrRepr::from(0)));
            assert_eq!(
                tmp,
                Fr(FrRepr([
                    0x437ce7616d580765
                ]))
            );
        }

        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Ensure that (a - b) + (b - a) = 0.
            let a = Fr::rand(&mut rng);
            let b = Fr::rand(&mut rng);

            let mut tmp1 = a;
            tmp1.sub_assign(&b);

            let mut tmp2 = b;
            tmp2.sub_assign(&a);

            tmp1.add_assign(&tmp2);
            assert!(tmp1.is_zero());
        }
    }

    #[test]
    #[ignore]
    fn test_fr_mul_assign() {
        let mut tmp = Fr::from_str("1").unwrap();
        tmp.mul_assign(&tmp.clone());
        assert_eq!(
            tmp,
            Fr::from_str("1").unwrap()
        ); 

        // let mut rng = rand::thread_rng();

        // for _ in 0..1000000 {
        //     // Ensure that (a * b) * c = a * (b * c)
        //     let a = Fr::rand(&mut rng);
        //     let b = Fr::rand(&mut rng);
        //     let c = Fr::rand(&mut rng);

        //     let mut tmp1 = a;
        //     tmp1.mul_assign(&b);
        //     tmp1.mul_assign(&c);

        //     let mut tmp2 = b;
        //     tmp2.mul_assign(&c);
        //     tmp2.mul_assign(&a);

        //     assert_eq!(tmp1, tmp2);
        // }

        // for _ in 0..1000000 {
        //     // Ensure that r * (a + b + c) = r*a + r*b + r*c

        //     let r = Fr::rand(&mut rng);
        //     let mut a = Fr::rand(&mut rng);
        //     let mut b = Fr::rand(&mut rng);
        //     let mut c = Fr::rand(&mut rng);

        //     let mut tmp1 = a;
        //     tmp1.add_assign(&b);
        //     tmp1.add_assign(&c);
        //     tmp1.mul_assign(&r);

        //     a.mul_assign(&r);
        //     b.mul_assign(&r);
        //     c.mul_assign(&r);

        //     a.add_assign(&b);
        //     a.add_assign(&c);

        //     assert_eq!(tmp1, a);
        // }
    }

    #[test]
    #[ignore]
    fn test_fr_squaring() {
        let mut a = Fr(FrRepr([
            0xffffffffffffffff
        ]));
        a.square();
        assert_eq!(
            a,
            Fr::from_repr(FrRepr([
                0xc0d698e7bde077b8
            ])).unwrap()
        );

        let mut rng = rand::thread_rng();

        for _ in 0..1000000 {
            // Ensure that (a * a) = a^2
            let a = Fr::rand(&mut rng);

            let mut tmp = a;
            tmp.square();

            let mut tmp2 = a;
            tmp2.mul_assign(&a);

            assert_eq!(tmp, tmp2);
        }
    }

    #[test]
    #[ignore]
    fn test_fr_inverse() {
        assert!(Fr::zero().inverse().is_none());

        let mut rng = rand::thread_rng();

        let one = Fr::one();

        for _ in 0..1000 {
            // Ensure that a * a^-1 = 1
            let mut a = Fr::rand(&mut rng);
            let ainv = a.inverse().unwrap();
            a.mul_assign(&ainv);
            assert_eq!(a, one);
        }
    }

    #[test]
    fn test_fr_double() {
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Ensure doubling a is equivalent to adding a to itself.
            let mut a = Fr::rand(&mut rng);
            let mut b = a;
            b.add_assign(&a);
            a.double();
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_fr_negate() {
        {
            let mut a = Fr::zero();
            a.negate();

            assert!(a.is_zero());
        }

        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Ensure (a - (-a)) = 0.
            let mut a = Fr::rand(&mut rng);
            let mut b = a;
            b.negate();
            a.add_assign(&b);

            assert!(a.is_zero());
        }
    }

    #[test]
    #[ignore]
    fn test_fr_pow() {
        let mut rng = rand::thread_rng();

        for i in 0..1000 {
            // Exponentiate by various small numbers and ensure it consists with repeated
            // multiplication.
            let a = Fr::rand(&mut rng);
            let target = a.pow(&[i]);
            let mut c = Fr::one();
            for _ in 0..i {
                c.mul_assign(&a);
            }
            assert_eq!(c, target);
        }

        for _ in 0..1000 {
            // Exponentiating by the modulus should have no effect in a prime field.
            let a = Fr::rand(&mut rng);

            assert_eq!(a, a.pow(Fr::char()));
        }
    }

    #[test]
    #[ignore]
    fn test_fr_sqrt() {
        let mut rng = rand::thread_rng();

        assert_eq!(Fr::zero().sqrt().unwrap(), Fr::zero());

        for _ in 0..1000 {
            // Ensure sqrt(a^2) = a or -a
            let a = Fr::rand(&mut rng);
            println!("Value of a: {:?}", a);
            let mut nega = a;
            nega.negate();
            let mut b = a;
            b.square();

            let b = b.sqrt().unwrap();

            assert!(a == b || nega == b);
        }

        for _ in 0..1000 {
            // Ensure sqrt(a)^2 = a for random a
            let a = Fr::rand(&mut rng);

            if let Some(mut tmp) = a.sqrt() {
                tmp.square();

                assert_eq!(a, tmp);
            }
        }
    }

    #[test]
    #[ignore]
    fn test_fr_from_into_repr() {
        // r + 1 should not be in the field
        assert!(
            Fr::from_repr(FrRepr([
                0xffffffff00000002
            ])).is_err()
        );

        // r should not be in the field
        assert!(Fr::from_repr(Fr::char()).is_err());

        // // Multiply some arbitrary representations to see if the result is as expected.
        // let a = FrRepr([
        //     0x25ebe3a3ad3c0c6a
        // ]);
        // let mut a_fr = Fr::from_repr(a).unwrap();
        // let b = FrRepr([
        //     0x264e9454885e2475
        // ]);
        // let b_fr = Fr::from_repr(b).unwrap();
        // let c = FrRepr([
        //     0x48a09ab93cfc740d
        // ]);
        // a_fr.mul_assign(&b_fr);
        // assert_eq!(a_fr.into_repr(), c);

        // Zero should be in the field.
        assert!(Fr::from_repr(FrRepr::from(0)).unwrap().is_zero());

        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            // Try to turn Fr elements into representations and back again, and compare.
            let a = Fr::rand(&mut rng);
            let a_repr = a.into_repr();
            let b_repr = FrRepr::from(a);
            assert_eq!(a_repr, b_repr);
            let a_again = Fr::from_repr(a_repr).unwrap();

            assert_eq!(a, a_again);
        }
    }

// #[test]
// fn test_fr_repr_display() {
//     assert_eq!(
//         format!(
//             "{}",
//             FrRepr([
//                 0x2829c242fa826143,
//                 0x1f32cf4dd4330917,
//                 0x932e4e479d168cd9,
//                 0x513c77587f563f64
//             ])
//         ),
//         "0x513c77587f563f64932e4e479d168cd91f32cf4dd43309172829c242fa826143".to_string()
//     );
//     assert_eq!(
//         format!(
//             "{}",
//             FrRepr([
//                 0x25ebe3a3ad3c0c6a,
//                 0x6990e39d092e817c,
//                 0x941f900d42f5658e,
//                 0x44f8a103b38a71e0
//             ])
//         ),
//         "0x44f8a103b38a71e0941f900d42f5658e6990e39d092e817c25ebe3a3ad3c0c6a".to_string()
//     );
//     assert_eq!(
//         format!(
//             "{}",
//             FrRepr([
//                 0xffffffffffffffff,
//                 0xffffffffffffffff,
//                 0xffffffffffffffff,
//                 0xffffffffffffffff
//             ])
//         ),
//         "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string()
//     );
//     assert_eq!(
//         format!("{}", FrRepr([0, 0, 0, 0])),
//         "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
//     );
// }

    #[test]
    #[ignore]
    fn test_fr_display() {
        assert_eq!(
            // format!(
            //     "{}",
                Fr::from_repr(FrRepr([
                    0xc3cae746a3b5ecc7
                ])).unwrap().to_string()
            ,
            "Fr(0xc3cae746a3b5ecc7)".to_string()
        );
        // assert_eq!(
        //     format!(
        //         "{}",
        //         Fr::from_repr(FrRepr([
        //             0x44c71298ff198106
        //         ])).unwrap()
        //     ),
        //     "Fr(0x44c71298ff198106)".to_string()
        // );
    }

    #[test]
    fn test_fr_num_bits() {
        assert_eq!(Fr::NUM_BITS, 64);
        assert_eq!(Fr::CAPACITY, 63);
    }

// #[test]
// fn test_fr_root_of_unity() {
//     assert_eq!(Fr::S, 32);
//     assert_eq!(
//         Fr::multiplicative_generator(),
//         Fr::from_repr(FrRepr::from(7)).unwrap()
//     );
//     assert_eq!(
//         Fr::multiplicative_generator().pow([
//             0xfffe5bfeffffffff,
//             0x9a1d80553bda402,
//             0x299d7d483339d808,
//             0x73eda753
//         ]),
//         Fr::root_of_unity()
//     );
//     assert_eq!(Fr::root_of_unity().pow([1 << Fr::S]), Fr::one());
//     assert!(Fr::multiplicative_generator().sqrt().is_none());
// }

// #[test]
// fn fr_field_tests() {
//     ::tests::field::random_field_tests::<Fr>();
//     ::tests::field::random_sqrt_tests::<Fr>();
//     ::tests::field::random_frobenius_tests::<Fr, _>(Fr::char(), 13);
//     ::tests::field::from_str_tests::<Fr>();
// }

// #[test]
// fn fr_repr_tests() {
//     ::tests::repr::random_repr_tests::<FrRepr>();
// }

}