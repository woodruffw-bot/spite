//! Exact native-word construction and allocation-free modulo reduction.

use spite_bigint::{BigInt, Budget, Error};

fn unlimited() -> Budget {
    Budget::with_limits(None, None)
}

#[test]
fn native_construction_preserves_exact_values_across_the_binary64_boundary() {
    for value in [
        0,
        1,
        u32::MAX as u64,
        1_u64 << 32,
        (1_u64 << 53) + 1,
        (1_u64 << 63) - 1,
        1_u64 << 63,
        u64::MAX,
    ] {
        let integer = BigInt::from_u64(value, &mut unlimited()).unwrap();
        assert_eq!(
            integer.to_radix(10, &mut unlimited()).unwrap(),
            value.to_string()
        );
        assert_eq!(integer.modulo_u64(&mut unlimited()), Ok(value));
    }
    for value in [
        i64::MIN,
        i64::MIN + 1,
        -(1_i64 << 53) - 1,
        -1,
        0,
        1,
        (1_i64 << 53) + 1,
        i64::MAX,
    ] {
        let integer = BigInt::from_i64(value, &mut unlimited()).unwrap();
        assert_eq!(
            integer.to_radix(10, &mut unlimited()).unwrap(),
            value.to_string()
        );
        assert_eq!(integer.modulo_u64(&mut unlimited()), Ok(value as u64));
    }
    let mut random = 1_u64;
    for _ in 0..512 {
        random = random
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let value = BigInt::from_u64(random, &mut unlimited()).unwrap();
        assert_eq!(
            value.to_radix(10, &mut unlimited()).unwrap(),
            random.to_string()
        );
        assert_eq!(value.modulo_u64(&mut unlimited()), Ok(random));
    }
}

#[test]
fn modulo_ignores_high_words_and_uses_twos_complement_for_negative_values() {
    for (digits, expected) in [
        ("10000000000000000", 0_u64),
        ("10000000000000001", 1),
        ("1ffffffffffffffff", u64::MAX),
        ("123456789abcdef00123456789abcdef", 0x0123_4567_89ab_cdef),
        ("100000000000000000000000000000000", 0),
    ] {
        let value = BigInt::parse_digits(digits, 16, &mut unlimited()).unwrap();
        assert_eq!(value.modulo_u64(&mut unlimited()), Ok(expected));
        assert_eq!(
            value
                .neg(&mut unlimited())
                .unwrap()
                .modulo_u64(&mut unlimited()),
            Ok(expected.wrapping_neg())
        );
    }
}

#[test]
fn magnitude_and_work_limits_check_results_without_modifying_inputs() {
    assert_eq!(
        BigInt::from_u64(0, &mut Budget::new(0, 1)),
        Ok(BigInt::default())
    );
    assert_eq!(
        BigInt::from_i64(-1, &mut Budget::new(1, 2)),
        Ok(BigInt::from(-1))
    );
    assert_eq!(
        BigInt::from_u64(u64::MAX, &mut Budget::new(63, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        BigInt::from_i64(i64::MIN, &mut Budget::new(63, 100)),
        Err(Error::Limit)
    );
    assert!(BigInt::from_i64(i64::MIN, &mut Budget::new(64, 3)).is_ok());
    assert_eq!(
        BigInt::from_u64(u64::MAX, &mut Budget::new(64, 2)),
        Err(Error::Limit)
    );
    let value =
        BigInt::parse_digits("123456789abcdef00123456789abcdef", 16, &mut unlimited()).unwrap();
    let original = value.clone();
    for work in [0, 1, 2] {
        assert_eq!(
            value.modulo_u64(&mut Budget::new(0, work)),
            Err(Error::Limit)
        );
        assert_eq!(value, original);
    }
    assert_eq!(
        value.modulo_u64(&mut Budget::new(0, 3)),
        Ok(0x0123_4567_89ab_cdef)
    );
}
