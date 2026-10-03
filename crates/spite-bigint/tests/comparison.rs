//! Mathematical integer/binary64 comparisons must not first round the integer.

use spite_bigint::{BigInt, Budget, Error};
use std::cmp::Ordering::{Equal, Greater, Less};

fn budget() -> Budget {
    Budget::new(4096, 100_000)
}
fn parse(s: &str) -> BigInt {
    BigInt::parse_digits(s, 10, &mut budget()).unwrap()
}

#[test]
fn exact_comparisons_across_rounding_boundaries_and_fractions() {
    for (integer, number, expected) in [
        ("9007199254740993", 9007199254740992.0, Greater),
        ("9007199254740993", 9007199254740994.0, Less),
        ("9007199254740992", 9007199254740992.0, Equal),
        ("18446744073709551615", 18446744073709551616.0, Less),
        ("18446744073709551617", 18446744073709551616.0, Greater),
        ("1", 1.5, Less),
        ("2", 1.5, Greater),
        ("1", 0.9999999999999999, Greater),
        ("4503599627370495", 4503599627370495.5, Less),
        ("1", f64::MIN_POSITIVE, Greater),
        ("1", f64::from_bits(1), Greater),
    ] {
        let x = parse(integer);
        assert_eq!(x.cmp_f64(number, &mut budget()).unwrap(), Some(expected));
        assert_eq!(
            x.neg(&mut budget())
                .unwrap()
                .cmp_f64(-number, &mut budget())
                .unwrap(),
            Some(expected.reverse())
        );
    }
    for integer in -100..=100 {
        for halves in -201..=201 {
            let number = f64::from(halves) / 2.0;
            assert_eq!(
                BigInt::from(integer)
                    .cmp_f64(number, &mut budget())
                    .unwrap(),
                (integer as f64).partial_cmp(&number)
            );
        }
    }
}

#[test]
fn nonfinite_zero_and_maximum_finite_boundaries() {
    for integer in [-1, 0, 1] {
        let x = BigInt::from(integer);
        assert_eq!(x.cmp_f64(f64::NAN, &mut budget()).unwrap(), None);
        assert_eq!(x.cmp_f64(f64::INFINITY, &mut budget()).unwrap(), Some(Less));
        assert_eq!(
            x.cmp_f64(f64::NEG_INFINITY, &mut budget()).unwrap(),
            Some(Greater)
        );
        for zero in [0.0, -0.0] {
            assert_eq!(
                x.cmp_f64(zero, &mut budget()).unwrap(),
                Some(integer.cmp(&0))
            );
        }
    }
    let max = BigInt::parse_digits(
        &format!("{}{}", "1".repeat(53), "0".repeat(971)),
        2,
        &mut budget(),
    )
    .unwrap();
    assert_eq!(max.cmp_f64(f64::MAX, &mut budget()).unwrap(), Some(Equal));
    assert_eq!(
        max.add(&BigInt::from(1), &mut budget())
            .unwrap()
            .cmp_f64(f64::MAX, &mut budget())
            .unwrap(),
        Some(Greater)
    );
    assert_eq!(
        max.sub(&BigInt::from(1), &mut budget())
            .unwrap()
            .cmp_f64(f64::MAX, &mut budget())
            .unwrap(),
        Some(Less)
    );
    assert_eq!(
        max.cmp_f64(f64::MAX, &mut Budget::new(0, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from(1)
            .cmp_f64(1.0, &mut Budget::new(0, 10))
            .unwrap(),
        Some(Equal)
    );
}

#[test]
fn hexadecimal_host_formatting_preserves_all_words_and_sign() {
    assert_eq!(format!("{:#x}", BigInt::default()), "0x0");
    assert_eq!(format!("{:#x}", BigInt::from(-4294967297)), "-0x100000001");
    assert_eq!(
        format!("{:x}", parse("18446744073709551617")),
        "10000000000000001"
    );
}
