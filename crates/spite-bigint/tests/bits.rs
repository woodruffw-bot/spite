//! Bit operations agree with exact native arithmetic and large independent patterns.

use spite_bigint::{BigInt, BitwiseOp, Budget, Error};

fn budget() -> Budget {
    Budget::new(4096, 1_000_000)
}
fn parse(s: &str, radix: u32) -> BigInt {
    BigInt::parse_digits(s, radix, &mut budget()).unwrap()
}
fn decimal(value: &BigInt) -> String {
    value.to_radix(10, &mut budget()).unwrap()
}

#[test]
fn shifts_match_signed_native_arithmetic_without_masking_counts() {
    for a in [i64::MIN, -4294967297, -9, -1, 0, 1, 9, 4294967297, i64::MAX] {
        let x = BigInt::from(a);
        for count in 0..64 {
            let n = BigInt::from(count);
            let minus_n = BigInt::from(-count);
            let left = (i128::from(a) << count).to_string();
            let right = (i128::from(a) >> count).to_string();
            assert_eq!(decimal(&x.shl(&n, &mut budget()).unwrap()), left);
            assert_eq!(decimal(&x.shr(&minus_n, &mut budget()).unwrap()), left);
            assert_eq!(decimal(&x.shr(&n, &mut budget()).unwrap()), right);
            assert_eq!(decimal(&x.shl(&minus_n, &mut budget()).unwrap()), right);
        }
    }
    let x = parse(&format!("1{}1", "0".repeat(512)), 2);
    let negative = x.neg(&mut budget()).unwrap();
    let n = BigInt::from(513);
    assert_eq!(x.shr(&n, &mut budget()).unwrap(), BigInt::from(1));
    assert_eq!(negative.shr(&n, &mut budget()).unwrap(), BigInt::from(-2));
    let exact = x.sub(&BigInt::from(1), &mut budget()).unwrap();
    assert_eq!(
        exact
            .neg(&mut budget())
            .unwrap()
            .shr(&n, &mut budget())
            .unwrap(),
        BigInt::from(-1)
    );
}

#[test]
fn arbitrarily_large_counts_saturate_only_right_shifts() {
    let huge = parse(&"f".repeat(256), 16);
    let negative_huge = huge.neg(&mut budget()).unwrap();
    for a in [-9, -1, 0, 1, 9] {
        let x = BigInt::from(a);
        let expected = BigInt::from(if a < 0 { -1 } else { 0 });
        assert_eq!(x.shr(&huge, &mut budget()).unwrap(), expected);
        assert_eq!(x.shl(&negative_huge, &mut budget()).unwrap(), expected);
        for result in [
            x.shl(&huge, &mut budget()),
            x.shr(&negative_huge, &mut budget()),
        ] {
            assert_eq!(
                result,
                if a == 0 {
                    Ok(BigInt::default())
                } else {
                    Err(Error::Limit)
                }
            );
        }
    }
    assert_eq!(
        BigInt::from(1).shl(&BigInt::from(8), &mut Budget::new(8, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(-1).shr(&huge, &mut Budget::new(0, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(1).shl(&BigInt::from(512), &mut Budget::new(1024, 2)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(1).shr(&BigInt::from(1), &mut Budget::new(1024, 0)),
        Err(Error::Limit)
    );
}

#[test]
fn infinite_bitwise_operations_match_signed_native_integers() {
    let values = [
        i64::MIN,
        -4294967297,
        -4294967296,
        -9,
        -1,
        0,
        1,
        9,
        4294967295,
        i64::MAX,
    ];
    for a in values {
        let x = BigInt::from(a);
        assert_eq!(x.bitnot(&mut budget()).unwrap(), BigInt::from(!a));
        for b in values {
            let y = BigInt::from(b);
            for (op, expected) in [
                (BitwiseOp::And, a & b),
                (BitwiseOp::Or, a | b),
                (BitwiseOp::Xor, a ^ b),
            ] {
                assert_eq!(
                    x.bitwise(&y, op, &mut budget()).unwrap(),
                    BigInt::from(expected)
                );
            }
        }
    }
    let even_bits = parse(&"a".repeat(128), 16);
    let odd_bits = parse(&"5".repeat(128), 16);
    let ones = parse(&"f".repeat(128), 16);
    assert_eq!(
        even_bits
            .bitwise(&odd_bits, BitwiseOp::And, &mut budget())
            .unwrap(),
        BigInt::default()
    );
    assert_eq!(
        even_bits
            .bitwise(&odd_bits, BitwiseOp::Or, &mut budget())
            .unwrap(),
        ones
    );
    assert_eq!(
        even_bits
            .bitwise(&ones, BitwiseOp::Xor, &mut budget())
            .unwrap(),
        odd_bits
    );
    assert_eq!(
        ones.bitnot(&mut budget()).unwrap(),
        parse(&format!("1{}", "0".repeat(128)), 16)
            .neg(&mut budget())
            .unwrap()
    );
    let complement = even_bits.bitnot(&mut budget()).unwrap();
    assert_eq!(
        even_bits
            .bitwise(&complement, BitwiseOp::And, &mut budget())
            .unwrap(),
        BigInt::default()
    );
    assert_eq!(
        even_bits
            .bitwise(&complement, BitwiseOp::Or, &mut budget())
            .unwrap(),
        BigInt::from(-1)
    );
    assert_eq!(complement.bitnot(&mut budget()).unwrap(), even_bits);
}

#[test]
fn bitwise_limits_apply_to_results_and_work() {
    assert_eq!(
        BigInt::from(-1).bitnot(&mut Budget::new(0, 100)).unwrap(),
        BigInt::default()
    );
    assert_eq!(
        BigInt::from(255).bitnot(&mut Budget::new(8, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(1).bitnot(&mut Budget::new(128, 0)),
        Err(Error::Limit)
    );
    let x = parse(&"f".repeat(128), 16);
    assert_eq!(
        x.bitwise(&x, BitwiseOp::Xor, &mut Budget::new(0, 1000))
            .unwrap(),
        BigInt::default()
    );
    assert_eq!(
        x.bitwise(&x, BitwiseOp::And, &mut Budget::new(128, 1000)),
        Err(Error::Limit)
    );
    assert_eq!(
        x.bitwise(&x, BitwiseOp::Or, &mut Budget::new(1024, 1)),
        Err(Error::Limit)
    );
}

#[test]
fn exponentiation_matches_exact_native_arithmetic_and_large_constants() {
    for base in -10i64..=10 {
        for exponent in 0..=30u32 {
            assert_eq!(
                decimal(
                    &BigInt::from(base)
                        .pow(&BigInt::from(i64::from(exponent)), &mut budget())
                        .unwrap()
                ),
                i128::from(base).pow(exponent).to_string()
            );
        }
    }
    assert_eq!(
        BigInt::from(2)
            .pow(&BigInt::from(1024), &mut budget())
            .unwrap(),
        parse(&format!("1{}", "0".repeat(256)), 16)
    );
    let huge = parse(&"f".repeat(256), 16);
    for (base, expected) in [(0, 0), (1, 1), (-1, -1)] {
        assert_eq!(
            BigInt::from(base).pow(&huge, &mut budget()).unwrap(),
            BigInt::from(expected)
        );
    }
    let even = huge.add(&BigInt::from(1), &mut budget()).unwrap();
    assert_eq!(
        BigInt::from(-1).pow(&even, &mut budget()).unwrap(),
        BigInt::from(1)
    );
    assert_eq!(BigInt::from(2).pow(&huge, &mut budget()), Err(Error::Limit));
    for base in [-1, 0, 1, 2] {
        assert_eq!(
            BigInt::from(base).pow(&BigInt::from(-1), &mut budget()),
            Err(Error::NegativeExponent)
        );
    }
    assert_eq!(
        BigInt::from(2).pow(&BigInt::from(8), &mut Budget::new(8, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(3).pow(&BigInt::from(100), &mut Budget::new(1024, 5)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::default().pow(&BigInt::default(), &mut Budget::new(0, 100)),
        Err(Error::Limit)
    );
}

#[test]
fn native_index_conversion_is_exact_and_unsigned() {
    assert_eq!(BigInt::default().to_usize(), Some(0));
    assert_eq!(BigInt::from(-1).to_usize(), None);
    assert_eq!(
        parse(&usize::MAX.to_string(), 10).to_usize(),
        Some(usize::MAX)
    );
    let overflow = parse(&usize::MAX.to_string(), 10)
        .add(&BigInt::from(1), &mut budget())
        .unwrap();
    assert_eq!(overflow.to_usize(), None);
    for value in -9..=9 {
        assert_eq!(BigInt::from(value).is_odd(), value % 2 != 0);
    }
}
