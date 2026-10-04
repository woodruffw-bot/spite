//! Correctly rounded integer-to-binary64 conversion without intermediate rounding.

use spite_bigint::{BigInt, Budget, Error};

fn budget() -> Budget {
    Budget::new(4096, 1_000_000)
}

fn check(digits: &str, radix: u32, expected: f64) {
    let value = BigInt::parse_digits(digits, radix, &mut budget()).unwrap();
    assert_eq!(
        value.to_f64(&mut budget()).unwrap().to_bits(),
        expected.to_bits(),
        "{digits}"
    );
    if !value.is_zero() {
        let negative = value.neg(&mut budget()).unwrap();
        assert_eq!(
            negative.to_f64(&mut budget()).unwrap().to_bits(),
            (-expected).to_bits(),
            "-{digits}"
        );
    }
}

#[test]
fn exact_values_and_halfway_cases_round_to_the_even_significand() {
    for (digits, expected) in [
        ("0", 0.0),
        ("1", 1.0),
        ("4294967295", 4_294_967_295.0),
        ("9007199254740991", 9_007_199_254_740_991.0),
        ("9007199254740992", 9_007_199_254_740_992.0),
        ("9007199254740993", 9_007_199_254_740_992.0),
        ("9007199254740994", 9_007_199_254_740_994.0),
        ("9007199254740995", 9_007_199_254_740_996.0),
        ("18014398509481983", 18_014_398_509_481_984.0),
        ("18446744073709551615", 18_446_744_073_709_551_616.0),
    ] {
        check(digits, 10, expected);
    }
}

#[test]
fn distant_sticky_bits_distinguish_a_tie_from_a_value_above_it() {
    // At 2**100, one ULP is 2**48. Keep the least significant bit far
    // from the rounding bit to catch word-at-a-time double rounding.
    let base = BigInt::parse_digits(&format!("1{}", "0".repeat(100)), 2, &mut budget()).unwrap();
    let half = BigInt::parse_digits(&format!("1{}", "0".repeat(47)), 2, &mut budget()).unwrap();
    let one = BigInt::parse_digits("1", 10, &mut budget()).unwrap();
    let halfway = base.add(&half, &mut budget()).unwrap();
    let above = halfway.add(&one, &mut budget()).unwrap();
    let below = halfway.sub(&one, &mut budget()).unwrap();
    let power = f64::from_bits((1023 + 100) << 52);
    for (value, expected) in [
        (below, power),
        (halfway, power),
        (above, f64::from_bits(power.to_bits() + 1)),
    ] {
        assert_eq!(value.to_f64(&mut budget()).unwrap(), expected);
        assert_eq!(
            value
                .neg(&mut budget())
                .unwrap()
                .to_f64(&mut budget())
                .unwrap(),
            -expected
        );
    }
}

#[test]
fn overflow_rounds_at_the_midpoint_above_the_largest_finite_value() {
    check(
        &format!("{}{}", "1".repeat(53), "0".repeat(971)),
        2,
        f64::MAX,
    );
    // 2**1024 - 2**970 is exactly halfway to the next power of two.
    let midpoint = format!("{}{}", "1".repeat(54), "0".repeat(970));
    check(&midpoint, 2, f64::INFINITY);
    let value = BigInt::parse_digits(&midpoint, 2, &mut budget()).unwrap();
    let one = BigInt::parse_digits("1", 2, &mut budget()).unwrap();
    assert_eq!(
        value
            .sub(&one, &mut budget())
            .unwrap()
            .to_f64(&mut budget())
            .unwrap(),
        f64::MAX
    );
    check(&format!("1{}", "0".repeat(1024)), 2, f64::INFINITY);
    check(&format!("1{}", "0".repeat(2048)), 2, f64::INFINITY);
}

#[test]
fn u64_conversions_match_rust_binary64_rounding() {
    let mut integer = 1u64;
    for _ in 0..4096 {
        integer = integer
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        check(&integer.to_string(), 10, integer as f64);
    }
}

#[test]
fn work_failure_preserves_the_integer_and_conversion_needs_no_integer_capacity() {
    let value = BigInt::parse_digits("9007199254740995", 10, &mut budget()).unwrap();
    let original = value.clone();
    for work in [0, 1, value.bit_length()] {
        assert_eq!(value.to_f64(&mut Budget::new(0, work)), Err(Error::Limit));
        assert_eq!(value, original);
    }
    assert_eq!(
        value.to_f64(&mut Budget::new(0, value.bit_length() + 1)),
        Ok(9_007_199_254_740_996.0)
    );
}

#[test]
fn binary64_to_integer_is_exact_across_every_integral_exponent() {
    for exponent in 0..=1023u64 {
        for fraction in [0, 1, (1u64 << 52) - 1] {
            let number = f64::from_bits(((1023 + exponent) << 52) | fraction);
            for number in [number, -number] {
                let result = BigInt::from_f64(number, &mut budget()).unwrap();
                if number.trunc() != number {
                    assert_eq!(result, None);
                    continue;
                }
                let result = result.unwrap();
                assert_eq!(
                    result.to_radix(10, &mut budget()).unwrap(),
                    format!("{number:.0}")
                );
                assert_eq!(
                    result.to_f64(&mut budget()).unwrap().to_bits(),
                    number.to_bits()
                );
            }
        }
    }
}

#[test]
fn number_to_integer_rejects_fractions_and_nonfinite_values_and_checks_quotas() {
    for number in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        0.5,
        -1.5,
        f64::MIN_POSITIVE,
        f64::from_bits(1),
    ] {
        assert_eq!(BigInt::from_f64(number, &mut budget()), Ok(None));
    }
    for number in [0.0, -0.0] {
        assert_eq!(
            BigInt::from_f64(number, &mut Budget::new(0, 1)),
            Ok(Some(BigInt::default()))
        );
    }
    assert_eq!(
        BigInt::from_f64(1.0, &mut Budget::new(0, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from_f64(1.0, &mut Budget::new(1, 0)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from_f64(f64::MAX, &mut Budget::new(1023, 100)),
        Err(Error::Limit)
    );
    let maximum = BigInt::from_f64(f64::MAX, &mut Budget::with_limits(None, None))
        .unwrap()
        .unwrap();
    assert_eq!(
        maximum.to_radix(2, &mut budget()).unwrap(),
        format!("{}{}", "1".repeat(53), "0".repeat(971))
    );
}
