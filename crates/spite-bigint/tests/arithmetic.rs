//! Arithmetic oracles use exact native integers and independent large constants.

use spite_bigint::{BigInt, Budget, Error};

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
fn radix_round_trips_and_sign_normalization() {
    for radix in 2..=36 {
        for value in [0, 1, 35, u32::MAX as i64, u32::MAX as i64 + 1, i64::MAX] {
            let integer = BigInt::from(value);
            let digits = integer.to_radix(radix, &mut budget()).unwrap();
            assert_eq!(parse(&digits, radix), integer);
            assert_eq!(parse(&digits.to_uppercase(), radix), integer);
            assert_eq!(parse(&format!("000{digits}"), radix), integer);
        }
    }
    for value in [i64::MIN, -1, 0, 1, i64::MAX] {
        assert_eq!(decimal(&BigInt::from(value)), value.to_string());
    }
    assert_eq!(
        BigInt::default().neg(&mut budget()).unwrap(),
        BigInt::default()
    );
    assert!(!BigInt::default().is_negative());
    assert_eq!(BigInt::default().bit_length(), 0);
    assert_eq!(parse("100000000", 16).bit_length(), 33);
}

#[test]
fn large_radix_constants_cross_many_words() {
    let max = parse("340282366920938463463374607431768211455", 10);
    assert_eq!(max, parse("ffffffffffffffffffffffffffffffff", 16));
    assert_eq!(max.to_radix(2, &mut budget()).unwrap(), "1".repeat(128));
    assert_eq!(
        decimal(&max.add(&BigInt::from(1), &mut budget()).unwrap()),
        "340282366920938463463374607431768211456"
    );
    assert_eq!(max.bit_length(), 128);
}

#[test]
fn signed_addition_subtraction_and_order_match_native_integers() {
    let values = [
        i64::MIN,
        -4294967296,
        -4294967295,
        -7,
        -1,
        0,
        1,
        7,
        4294967295,
        4294967296,
        i64::MAX,
    ];
    for a in values {
        for b in values {
            let x = BigInt::from(a);
            let y = BigInt::from(b);
            assert_eq!(
                decimal(&x.add(&y, &mut budget()).unwrap()),
                (i128::from(a) + i128::from(b)).to_string()
            );
            assert_eq!(
                decimal(&x.sub(&y, &mut budget()).unwrap()),
                (i128::from(a) - i128::from(b)).to_string()
            );
            assert_eq!(x.cmp(&y), a.cmp(&b));
        }
    }
    for a in -64..=64 {
        for b in -64..=64 {
            assert_eq!(
                BigInt::from(a)
                    .add(&BigInt::from(b), &mut budget())
                    .unwrap(),
                BigInt::from(a + b)
            );
            assert_eq!(
                BigInt::from(a)
                    .sub(&BigInt::from(b), &mut budget())
                    .unwrap(),
                BigInt::from(a - b)
            );
        }
    }
}

#[test]
fn carries_borrows_and_cancellation_cross_word_boundaries() {
    let x = parse(&"f".repeat(128), 16);
    let next = x.add(&BigInt::from(1), &mut budget()).unwrap();
    assert_eq!(
        next.to_radix(16, &mut budget()).unwrap(),
        format!("1{}", "0".repeat(128))
    );
    assert_eq!(next.sub(&BigInt::from(1), &mut budget()).unwrap(), x);
    assert_eq!(
        x.add(&x.neg(&mut budget()).unwrap(), &mut budget())
            .unwrap(),
        BigInt::default()
    );
    assert_eq!(x.sub(&x, &mut budget()).unwrap(), BigInt::default());
}

#[test]
fn invalid_digits_radices_and_resource_exhaustion_are_distinct() {
    for (s, radix) in [
        ("", 10),
        ("-1", 10),
        ("+1", 10),
        ("1_0", 10),
        ("0x1", 16),
        ("2", 2),
        (" 1", 10),
        ("💩", 36),
    ] {
        assert_eq!(
            BigInt::parse_digits(s, radix, &mut budget()),
            Err(Error::InvalidDigit)
        );
    }
    assert_eq!(
        BigInt::parse_digits("1", 1, &mut budget()),
        Err(Error::InvalidRadix)
    );
    assert_eq!(
        BigInt::from(1).to_radix(37, &mut budget()),
        Err(Error::InvalidRadix)
    );
    assert_eq!(
        BigInt::parse_digits("256", 10, &mut Budget::new(8, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(255).add(&BigInt::from(1), &mut Budget::new(8, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(255).sub(&BigInt::from(255), &mut Budget::new(0, 100)),
        Ok(BigInt::default())
    );
    let mut empty = Budget::new(4096, 0);
    assert_eq!(BigInt::from(1).neg(&mut empty), Err(Error::Limit));
    assert_eq!(
        parse(&"f".repeat(128), 16).to_radix(10, &mut Budget::new(4096, 4)),
        Err(Error::Limit)
    );
    assert_eq!(empty.remaining_work(), 0);
}

#[test]
fn multiplication_and_division_match_exact_native_arithmetic() {
    let values = [
        i64::MIN,
        -4294967296,
        -4294967295,
        -7,
        -1,
        0,
        1,
        7,
        4294967295,
        4294967296,
        i64::MAX,
    ];
    for a in values {
        for b in values {
            let x = BigInt::from(a);
            let y = BigInt::from(b);
            assert_eq!(
                decimal(&x.mul(&y, &mut budget()).unwrap()),
                (i128::from(a) * i128::from(b)).to_string()
            );
            if b != 0 {
                let (q, r) = x.div_rem(&y, &mut budget()).unwrap();
                assert_eq!(decimal(&q), (i128::from(a) / i128::from(b)).to_string());
                assert_eq!(decimal(&r), (i128::from(a) % i128::from(b)).to_string());
                assert_eq!(
                    q.mul(&y, &mut budget())
                        .unwrap()
                        .add(&r, &mut budget())
                        .unwrap(),
                    x
                );
            }
        }
    }
    for a in -64..=64 {
        for b in -64..=64 {
            if b == 0 {
                continue;
            }
            let (q, r) = BigInt::from(a)
                .div_rem(&BigInt::from(b), &mut budget())
                .unwrap();
            assert_eq!(q, BigInt::from(a / b));
            assert_eq!(r, BigInt::from(a % b));
        }
    }
}

#[test]
fn large_products_and_division_reconstruct_the_dividend() {
    let x = parse(&"f".repeat(256), 16);
    let product = x.mul(&x, &mut budget()).unwrap();
    let expected = format!("{}e{}1", "f".repeat(255), "0".repeat(255));
    assert_eq!(product.to_radix(16, &mut budget()).unwrap(), expected);
    assert_eq!(
        product.div_rem(&x, &mut budget()).unwrap(),
        (x.clone(), BigInt::default())
    );
    let dividend = parse(
        "123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        16,
    );
    for divisor in [
        "80000000",
        "ffffffff",
        "100000000",
        "ffffffffffffffff",
        "10000000000000001",
        "123456789abcdef",
    ] {
        let divisor = parse(divisor, 16);
        let (q, r) = dividend.div_rem(&divisor, &mut budget()).unwrap();
        assert!(r < divisor);
        assert_eq!(
            q.mul(&divisor, &mut budget())
                .unwrap()
                .add(&r, &mut budget())
                .unwrap(),
            dividend
        );
    }
}

#[test]
fn multiplication_and_division_limits_are_checked() {
    assert_eq!(
        BigInt::from(1).div_rem(&BigInt::default(), &mut budget()),
        Err(Error::DivisionByZero)
    );
    assert_eq!(
        BigInt::default().div_rem(&BigInt::default(), &mut budget()),
        Err(Error::DivisionByZero)
    );
    assert_eq!(
        BigInt::from(16).mul(&BigInt::from(16), &mut Budget::new(8, 100)),
        Err(Error::Limit)
    );
    let x = parse(&"f".repeat(256), 16);
    assert_eq!(x.mul(&x, &mut Budget::new(4096, 4)), Err(Error::Limit));
    assert_eq!(
        x.div_rem(&BigInt::from(3), &mut Budget::new(4096, 40)),
        Err(Error::Limit)
    );
    assert_eq!(
        x.mul(&BigInt::default(), &mut Budget::new(0, 1)),
        Ok(BigInt::default())
    );
}
