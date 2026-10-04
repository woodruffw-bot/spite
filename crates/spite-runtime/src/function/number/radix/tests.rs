use super::*;
use crate::{Error as RuntimeError, Realm, Value};
use spite_core::JsString;

#[test]
fn every_radix_matches_independent_rational_reference_at_binary64_boundaries() {
    let fixtures = include_str!("../../../../tests/fixtures/number_radix.tsv");
    let mut count = 0;
    for line in fixtures.lines().filter(|line| !line.starts_with('#')) {
        let mut fields = line.split('\t');
        let bits = u64::from_str_radix(fields.next().unwrap(), 16).unwrap();
        let radix = fields.next().unwrap().parse().unwrap();
        let expected = fields.next().unwrap();
        assert!(fields.next().is_none());
        let value = f64::from_bits(bits);
        for (input, output) in [
            (value, expected.to_owned()),
            (-value, format!("-{expected}")),
        ] {
            assert_eq!(
                format(input, radix, &mut Budget::new(65_536, 100_000)).unwrap(),
                output,
                "{bits:x} radix {radix}"
            );
        }
        count += 1;
    }
    assert_eq!(count, 20 * 34);
}

#[test]
fn general_digit_generation_agrees_with_independent_shortest_decimal_formatter() {
    // Decimal output uses the existing formatter in production. Exercising the
    // general algorithm here provides a second, independent rounding oracle.
    let mut seed = 0x1234_5678_9abc_def0u64;
    for _ in 0..256 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let value = f64::from_bits(seed & 0x7fef_ffff_ffff_ffff);
        if value == 0.0 {
            continue;
        }
        assert_eq!(
            format(value, 10, &mut Budget::new(65_536, 100_000)).unwrap(),
            value.to_string(),
            "{:x}",
            value.to_bits()
        );
    }
}

#[test]
fn work_integer_and_output_limits_abort_without_running_handlers() {
    assert_eq!(
        format(0.1, 3, &mut Budget::new(65_536, 0)),
        Err(Error::Limit)
    );
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_bigint_bits = Some(8);
    assert!(matches!(
        realm.eval("try{(0.1).toString(3);}catch{flag=1;}finally{flag=2;}"),
        Err(RuntimeError::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    realm.limits.max_bigint_bits = Some(65_536);
    realm.limits.max_string_units = Some(8);
    assert!(matches!(
        realm.eval("try{Number.MIN_VALUE.toString(2);}catch{flag=1;}finally{flag=2;}"),
        Err(RuntimeError::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("(15).toString(16)"),
        Ok(Value::String(JsString::from("f")))
    );
}
