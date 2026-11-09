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
            assert_eq!(
                BigInt::parse_digits(
                    &format!("{}{digits}", "0".repeat(37)),
                    radix,
                    &mut Budget::with_limits(None, None),
                ),
                Ok(integer)
            );
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
fn power_of_two_radices_preserve_cross_word_digits_without_leading_zeroes() {
    let value = parse("123456789abcdef0123456789abcdef", 16);
    let negative = value.neg(&mut budget()).unwrap();
    for (radix, expected) in [
        (
            2,
            "1001000110100010101100111100010011010101111001101111011110000000100100011010001010110011110001001101010111100110111101111",
        ),
        (
            4,
            "1020310111213202122233031323300010203101112132021222330313233",
        ),
        (8, "11064254742325715736004432126361152746757"),
        (16, "123456789abcdef0123456789abcdef"),
        (32, "14d2mf2dbpnng28q5cu4qnjff"),
    ] {
        assert_eq!(value.to_radix(radix, &mut budget()).unwrap(), expected);
        for digits in [expected.to_owned(), expected.to_uppercase()] {
            assert_eq!(
                BigInt::parse_digits(
                    &format!("{}{digits}", "0".repeat(37)),
                    radix,
                    &mut Budget::with_limits(None, None),
                ),
                Ok(value.clone())
            );
        }
        assert_eq!(
            negative.to_radix(radix, &mut budget()).unwrap(),
            format!("-{expected}")
        );
        assert_eq!(
            BigInt::default().to_radix(radix, &mut budget()).unwrap(),
            "0"
        );
        // Formatting creates no integer result; its magnitude quota does not
        // constrain an existing value, while the work quota remains active.
        assert_eq!(
            value
                .to_radix(radix, &mut Budget::with_work_limit(0, None))
                .unwrap(),
            expected
        );
        assert_eq!(
            value.to_radix(radix, &mut Budget::new(4096, 5)),
            Err(Error::Limit)
        );
    }
    let all_ones = parse(&"f".repeat(256), 16);
    for (radix, prefix, digit, count) in [
        (2, "", "1", 1024),
        (4, "", "3", 512),
        (8, "1", "7", 341),
        (16, "", "f", 256),
        (32, "f", "v", 204),
    ] {
        assert_eq!(
            all_ones.to_radix(radix, &mut budget()).unwrap(),
            format!("{prefix}{}", digit.repeat(count))
        );
    }
}

#[test]
fn grouped_radix_output_pads_lower_groups_and_keeps_the_high_group_minimal() {
    for (radix, input, expected) in [
        (10, "12a05f200", "5000000000"),
        (10, "12a05f201", "5000000001"),
        (10, "de0b6b3a7640000", "1000000000000000000"),
        (10, "de0b6b3a7640001", "1000000000000000001"),
        (10, "de0b6b3a763ffff", "999999999999999999"),
        (36, "1037e2001", "2000001"),
        (36, "41c21cb8e1000000", "1000000000000"),
        (36, "41c21cb8e1000001", "1000000000001"),
        (36, "41c21cb8e0ffffff", "zzzzzzzzzzzz"),
        (
            3,
            "a8b8b452291fe822",
            "10000000000000000000000000000000000000001",
        ),
        (
            3,
            "a8b8b452291fe820",
            "2222222222222222222222222222222222222222",
        ),
    ] {
        let value = parse(input, 16);
        assert_eq!(value.to_radix(radix, &mut budget()).unwrap(), expected);
        assert_eq!(
            value
                .neg(&mut budget())
                .unwrap()
                .to_radix(radix, &mut budget())
                .unwrap(),
            format!("-{expected}")
        );
        assert_eq!(
            value.to_radix(radix, &mut Budget::new(4096, 1)),
            Err(Error::Limit)
        );
        assert_eq!(value.to_radix(radix, &mut budget()).unwrap(), expected);
    }
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
    for (digits, radix) in [
        ("340282366920938463463374607431768211455".to_owned(), 10),
        ("f".repeat(32), 16),
        ("1".repeat(128), 2),
        (format!("3{}", "7".repeat(42)), 8),
    ] {
        assert_eq!(
            BigInt::parse_digits(&digits, radix, &mut Budget::with_limits(None, None)),
            Ok(max.clone())
        );
    }
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
    assert_eq!(empty.remaining_work(), Some(0));
}

#[test]
fn unlimited_work_preserves_the_integer_size_limit() {
    let mut budget = Budget::with_work_limit(8, None);
    budget.charge(usize::MAX).unwrap();
    budget.charge(usize::MAX).unwrap();
    assert_eq!(
        BigInt::from(1).add(&BigInt::from(2), &mut budget),
        Ok(BigInt::from(3))
    );
    assert_eq!(
        BigInt::parse_digits("256", 10, &mut budget),
        Err(Error::Limit)
    );
    assert_eq!(budget.remaining_work(), None);
    let invalid_tail = format!("{}!", "1".repeat(40));
    assert_eq!(
        BigInt::parse_digits(&invalid_tail, 10, &mut budget),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::parse_digits(&invalid_tail, 10, &mut Budget::with_limits(None, None)),
        Err(Error::InvalidDigit)
    );
}

#[test]
fn optional_size_limits_preserve_checked_shift_arithmetic() {
    let mut budget = Budget::with_limits(None, None);
    let count = BigInt::parse_digits(&usize::MAX.to_string(), 10, &mut budget).unwrap();
    assert_eq!(BigInt::from(1).shl(&count, &mut budget), Err(Error::Limit));
    let count = BigInt::from(65_536);
    let shifted = BigInt::from(1).shl(&count, &mut budget).unwrap();
    assert_eq!(shifted.bit_length(), 65_537);
    assert_eq!(shifted.shr(&count, &mut budget), Ok(BigInt::from(1)));
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
                let (unlimited_q, unlimited_r) =
                    x.div_rem(&y, &mut Budget::with_limits(None, None)).unwrap();
                assert_eq!(unlimited_q, q);
                assert_eq!(unlimited_r, r);
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
fn single_word_division_preserves_carries_and_signed_remainders() {
    // Independent integer constants include maximum numerator words and
    // remainders immediately below the divisor, across several limb boundaries.
    for (dividend, divisor, quotient, remainder) in [
        ("ffffffffffffffff", 4294967295, "100000001", 0),
        ("fffffffffffffffe", 4294967295, "100000000", 4294967294),
        (
            "ffffffffffffffffffffffff",
            4294967295,
            "10000000100000001",
            0,
        ),
        (
            "ffffffffffffffffffffffff",
            2147483649,
            "1fffffffc00000007",
            2147483640,
        ),
        ("800000000000000000000001", 3, "2aaaaaaaaaaaaaaaaaaaaaab", 0),
        ("10000000000000000", 2, "8000000000000000", 0),
        (
            "1000000010000000100000001",
            65537,
            "ffff0001fffe0002fffd",
            4,
        ),
        (
            "ffffffff00000000ffffffff",
            4294967294,
            "10000000100000003",
            5,
        ),
        ("fffffffe", 4294967295, "0", 4294967294),
        ("0", 1, "0", 0),
    ] {
        for negative_dividend in [false, true] {
            for negative_divisor in [false, true] {
                let signed = |value: BigInt, negative| {
                    if negative {
                        value.neg(&mut budget()).unwrap()
                    } else {
                        value
                    }
                };
                let dividend = signed(parse(dividend, 16), negative_dividend);
                let divisor = signed(BigInt::from(divisor), negative_divisor);
                let quotient = signed(parse(quotient, 16), negative_dividend != negative_divisor);
                let remainder = signed(BigInt::from(remainder), negative_dividend);
                assert_eq!(
                    dividend.div_rem(&divisor, &mut budget()).unwrap(),
                    (quotient, remainder)
                );
            }
        }
    }
}

#[test]
fn multiplication_and_division_limits_are_checked() {
    let large = parse(&format!("1{}", "0".repeat(256)), 2);
    for value in [
        BigInt::default(),
        BigInt::from(1),
        BigInt::from(-1),
        large.clone(),
        large.neg(&mut budget()).unwrap(),
    ] {
        for unit in [-1, 1] {
            let factor = BigInt::from(unit);
            let expected = if unit < 0 {
                value.neg(&mut budget()).unwrap()
            } else {
                value.clone()
            };
            for (left, right) in [(&value, &factor), (&factor, &value)] {
                let n = left.bit_length().div_ceil(32);
                let m = right.bit_length().div_ceil(32);
                let work = if value.is_zero() { 1 } else { 1 + n * m + n };
                let mut exact = Budget::new(expected.bit_length(), work);
                assert_eq!(left.mul(right, &mut exact), Ok(expected.clone()));
                assert_eq!(exact.remaining_work(), Some(0));
                assert_eq!(
                    left.mul(right, &mut Budget::new(expected.bit_length(), work - 1)),
                    Err(Error::Limit)
                );
                if !expected.is_zero() {
                    let mut small = Budget::new(expected.bit_length() - 1, work);
                    assert_eq!(left.mul(right, &mut small), Err(Error::Limit));
                    assert_eq!(small.remaining_work(), Some(work - 1));
                }
            }
        }
    }
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
