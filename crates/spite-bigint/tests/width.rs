//! Width conversion against exact native modulo arithmetic and boundary values.

use spite_bigint::{BigInt, Budget, Error};

fn budget() -> Budget {
    Budget::with_limits(None, None)
}

#[test]
fn signed_and_unsigned_reductions_match_native_modulo_for_every_small_width() {
    let mut random = 1u64;
    for _ in 0..128 {
        random = random
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        for value in [random as i64, i64::MIN, i64::MAX, -1, 0, 1] {
            let bigint = BigInt::from(value);
            for bits in 0..=64 {
                let modulus = 1i128 << bits;
                let unsigned = (value as i128).rem_euclid(modulus);
                let signed = if bits != 0 && unsigned >= modulus / 2 {
                    unsigned - modulus
                } else {
                    unsigned
                };
                assert_eq!(
                    bigint
                        .as_uint_n(bits, &mut budget())
                        .unwrap()
                        .to_radix(10, &mut budget())
                        .unwrap(),
                    unsigned.to_string(),
                    "unsigned {value}, {bits}"
                );
                assert_eq!(
                    bigint
                        .as_int_n(bits, &mut budget())
                        .unwrap()
                        .to_radix(10, &mut budget())
                        .unwrap(),
                    signed.to_string(),
                    "signed {value}, {bits}"
                );
            }
        }
    }
}

#[test]
fn sign_bits_partial_words_and_exact_modulus_multiples_are_preserved() {
    for bits in [1, 31, 32, 33, 63, 64, 65, 127, 128, 129] {
        let power = BigInt::from(1)
            .shl(&BigInt::from(bits as i64), &mut budget())
            .unwrap();
        let sign = BigInt::from(1)
            .shl(&BigInt::from(bits as i64 - 1), &mut budget())
            .unwrap();
        let maximum = power.sub(&BigInt::from(1), &mut budget()).unwrap();
        assert_eq!(
            power.as_uint_n(bits, &mut budget()).unwrap(),
            BigInt::default()
        );
        assert_eq!(
            power
                .neg(&mut budget())
                .unwrap()
                .as_int_n(bits, &mut budget())
                .unwrap(),
            BigInt::default()
        );
        assert_eq!(
            maximum.as_int_n(bits, &mut budget()).unwrap(),
            BigInt::from(-1)
        );
        assert_eq!(
            BigInt::from(-1).as_uint_n(bits, &mut budget()).unwrap(),
            maximum
        );
        assert_eq!(
            sign.as_int_n(bits, &mut budget()).unwrap(),
            sign.neg(&mut budget()).unwrap()
        );
        assert_eq!(
            sign.neg(&mut budget())
                .unwrap()
                .as_int_n(bits, &mut budget())
                .unwrap(),
            sign.neg(&mut budget()).unwrap()
        );
    }
}

#[test]
fn huge_widths_return_fitting_values_without_width_sized_storage() {
    for bits in [9_007_199_254_740_991, u64::MAX] {
        for value in [i64::MIN, -1, 0, 1, i64::MAX] {
            let value = BigInt::from(value);
            assert_eq!(value.as_int_n(bits, &mut budget()).unwrap(), value);
            if !value.is_negative() {
                assert_eq!(value.as_uint_n(bits, &mut budget()).unwrap(), value);
            }
        }
        assert_eq!(
            BigInt::from(-1).as_uint_n(bits, &mut Budget::with_limits(Some(128), None)),
            Err(Error::Limit)
        );
    }
}

#[test]
fn quotas_apply_to_actual_outputs_and_work_failures_preserve_the_input() {
    let value = BigInt::from(-255);
    let original = value.clone();
    assert_eq!(
        value.as_uint_n(8, &mut Budget::new(1, 100)),
        Ok(BigInt::from(1))
    );
    assert_eq!(
        value.as_int_n(8, &mut Budget::new(1, 100)),
        Ok(BigInt::from(1))
    );
    for work in [0, 1] {
        assert_eq!(
            value.as_uint_n(8, &mut Budget::new(8, work)),
            Err(Error::Limit)
        );
        assert_eq!(value, original);
    }
    assert_eq!(
        BigInt::from(1).as_int_n(64, &mut Budget::new(0, 100)),
        Err(Error::Limit)
    );
    assert_eq!(
        value.as_uint_n(0, &mut Budget::new(0, 1)),
        Ok(BigInt::default())
    );
}
