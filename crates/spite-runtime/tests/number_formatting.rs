//! Exact fixed-point decimal rounding and observable argument conversion order.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(JsString::from(expected))),
        "{source}"
    );
}

#[test]
fn fixed_rounding_uses_the_exact_binary_value_and_rounds_halfway_up() {
    for (source, expected) in [
        ("(2.5).toFixed()", "3"),
        ("(2.5).toFixed(0)", "3"),
        ("(-2.5).toFixed(0)", "-3"),
        ("(1.25).toFixed(1)", "1.3"),
        ("(-1.25).toFixed(1)", "-1.3"),
        ("(2.55).toFixed(1)", "2.5"),
        ("(1.005).toFixed(2)", "1.00"),
        ("(2.9999).toFixed(3)", "3.000"),
        ("(0.0000009).toFixed(6)", "0.000001"),
        ("(1000000000000000128).toFixed(0)", "1000000000000000128"),
        (
            "(0.3).toFixed(50)",
            "0.29999999999999998889776975374843459576368331909180",
        ),
        ("new Number(1.25).toFixed(1)", "1.3"),
        ("Number.prototype.toFixed(2)", "0.00"),
    ] {
        string(source, expected);
    }
}

#[test]
fn fixed_signs_special_values_and_large_number_fallback_are_distinct() {
    for (source, expected) in [
        ("(-0).toFixed(3)", "0.000"),
        ("(-0.0001).toFixed(3)", "-0.000"),
        ("(-Number.MIN_VALUE).toFixed(3)", "-0.000"),
        ("Number.MIN_VALUE.toFixed(3)", "0.000"),
        ("NaN.toFixed(3)", "NaN"),
        ("Infinity.toFixed(3)", "Infinity"),
        ("(-Infinity).toFixed(3)", "-Infinity"),
        ("(1e21).toFixed(3)", "1e+21"),
        ("(-1e21).toFixed(3)", "-1e+21"),
        ("Number.MAX_VALUE.toFixed(100)", "1.7976931348623157e+308"),
    ] {
        string(source, expected);
    }
    string("(0).toFixed(100)", &format!("0.{}", "0".repeat(100)));
    string(
        "Number.MIN_VALUE.toFixed(100)",
        &format!("0.{}", "0".repeat(100)),
    );
    string(
        "(-Number.MIN_VALUE).toFixed(100)",
        &format!("-0.{}", "0".repeat(100)),
    );
}

#[test]
fn fraction_digits_use_tointegerorinfinity_before_range_checks() {
    for (argument, expected) in [
        ("undefined", "2"),
        ("NaN", "2"),
        ("null", "2"),
        ("false", "2"),
        ("-0.9", "2"),
        ("'2'", "1.50"),
        ("2.9", "1.50"),
        ("true", "1.5"),
    ] {
        string(&format!("(1.5).toFixed({argument})"), expected);
    }
    for argument in ["-1", "101", "Infinity", "-Infinity"] {
        for value in ["1", "NaN", "Infinity", "1e21"] {
            let source = format!("({value}).toFixed({argument})");
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::RangeError,
                        ..
                    })
                ),
                "{source}"
            );
        }
    }
    assert!(matches!(
        Realm::default().eval("(1).toFixed(2n)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn receiver_validation_precedes_fraction_coercion_and_special_values_do_not_skip_it() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0; let fraction={valueOf:()=>{flag++;return 2;}};")
        .unwrap();
    for receiver in [
        "undefined",
        "null",
        "true",
        "'1'",
        "1n",
        "{}",
        "({__proto__:Number.prototype})",
    ] {
        assert!(matches!(
            realm.eval(&format!(
                "Number.prototype.toFixed.call({receiver},fraction)"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    for (value, expected) in [
        ("1.25", "1.25"),
        ("NaN", "NaN"),
        ("Infinity", "Infinity"),
        ("1e21", "1e+21"),
    ] {
        assert_eq!(
            realm.eval(&format!("({value}).toFixed(fraction)")),
            Ok(Value::String(JsString::from(expected)))
        );
    }
    assert_eq!(realm.eval("flag"), Ok(Value::Number(4.0)));
    assert_eq!(
        realm.eval("NaN.toFixed({valueOf:()=>{throw 7;}})"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(
        realm.eval("(1).toFixed(2,flag=9)"),
        Ok(Value::String(JsString::from("1.00")))
    );
    assert_eq!(realm.eval("flag"), Ok(Value::Number(9.0)));
}

#[test]
fn fixed_metadata_is_standard_and_the_method_is_not_a_constructor() {
    let mut realm = Realm::default();
    let Value::Object(handle) = realm.eval("Number.prototype.toFixed").unwrap() else {
        panic!("function")
    };
    let object = realm.inspect_object(&handle).unwrap();
    assert!(object.is_callable());
    assert!(!object.is_constructor());
    for (key, value) in [
        ("name", Value::String(JsString::from("toFixed"))),
        ("length", Value::Number(1.0)),
    ] {
        let property = object
            .own_property(&JsString::from(key))
            .unwrap()
            .as_data()
            .unwrap();
        assert_eq!(property.value, value);
        assert!(!property.writable && !property.enumerable && property.configurable);
    }
    assert!(object.own_property(&JsString::from("prototype")).is_none());
    assert!(matches!(
        realm.eval("new Number.prototype.toFixed(2)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}
