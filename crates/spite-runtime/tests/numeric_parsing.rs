//! parseFloat/parseInt prefix selection, coercion, alias identity, and exact rounding.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn number(source: &str, expected: f64) {
    let value = Realm::default()
        .eval(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    assert!(
        value.same_value(&Value::Number(expected)),
        "{source}: {value:?} expected {expected:?}"
    );
}

#[test]
fn parsefloat_selects_the_longest_decimal_prefix() {
    for (input, expected) in [
        ("12.5more", 12.5),
        ("+.5x", 0.5),
        ("-0x1", -0.0),
        ("0x10", 0.0),
        ("1.e2junk", 100.0),
        ("1e+", 1.0),
        ("1e-", 1.0),
        ("1e", 1.0),
        (".1e2", 10.0),
        ("2.3.4", 2.3),
        ("1_000", 1.0),
        ("1e2_3", 100.0),
        ("08", 8.0),
        ("Infinitymore", f64::INFINITY),
        ("-Infinityx", f64::NEG_INFINITY),
        ("+Infinity", f64::INFINITY),
        ("1e999999999999999999999", f64::INFINITY),
        ("-1e-999999999999999999999", -0.0),
    ] {
        number(&format!("parseFloat('{input}')"), expected);
    }
    for input in [
        "", " ", ".", ".e2", "+", "-", "+-1", "NaN", "infinity", "Inf", "１２", "n1",
    ] {
        number(&format!("parseFloat('{input}')"), f64::NAN);
    }
    number("parseFloat()", f64::NAN);
    number("parseFloat(12.5)", 12.5);
    number("parseFloat(9007199254740993n)", 9_007_199_254_740_992.0);
}

#[test]
fn parsers_trim_ecmascript_whitespace_but_stop_at_non_ascii_or_surrogate_tails() {
    for function in ["parseFloat", "parseInt"] {
        number(
            &format!("{function}('\\uFEFF\\u2000\\u2028\\u2029\\t -12junk')"),
            -12.0,
        );
        number(&format!("{function}('12\\uD800rest')"), 12.0);
        number(&format!("{function}('12\\uDC00rest')"), 12.0);
        number(&format!("{function}('12é')"), 12.0);
        number(&format!("{function}('\\uD80012')"), f64::NAN);
        number(&format!("{function}('\\u180E12')"), f64::NAN);
        number(&format!("{function}('\\u008512')"), f64::NAN);
        number(&format!("{function}('1 2')"), 1.0);
    }
}

#[test]
fn parseint_radix_prefix_and_to_int32_rules_are_exact() {
    for (input, radix, expected) in [
        ("0x10", "undefined", 16.0),
        ("-0X10", "0", -16.0),
        ("+0x10", "16", 16.0),
        ("0x10", "10", 0.0),
        ("0x10", "36", 42804.0),
        ("0b11", "undefined", 0.0),
        ("0o11", "undefined", 0.0),
        ("077", "undefined", 77.0),
        ("08", "undefined", 8.0),
        ("1012", "2", 5.0),
        ("zz!", "36", 1295.0),
        ("Zz", "36", 1295.0),
        ("12.9", "10", 12.0),
        ("1e2", "10", 1.0),
        ("-0tail", "10", -0.0),
        ("-0x0", "0", -0.0),
        ("10", "2.9", 2.0),
        ("10", "4294967298", 2.0),
        ("10", "-4294967280", 16.0),
        ("10", "NaN", 10.0),
        ("10", "Infinity", 10.0),
        ("0x10", "null", 16.0),
        ("10", "'2'", 2.0),
    ] {
        number(&format!("parseInt('{input}',{radix})"), expected);
    }
    for radix in ["1", "-1", "37", "4294967297", "2147483648"] {
        number(&format!("parseInt('10',{radix})"), f64::NAN);
    }
    for source in [
        "parseInt()",
        "parseInt('0x')",
        "parseInt('0xg')",
        "parseInt('xyz',10)",
        "parseInt('+')",
        "parseInt('.1')",
    ] {
        number(source, f64::NAN);
    }
    number("parseInt(12.9)", 12.0);
    number("parseInt(1e21)", 1.0);
    number("parseInt(9007199254740993n)", 9_007_199_254_740_992.0);
}

#[test]
fn parseint_accumulates_exact_integers_before_binary64_rounding() {
    for (source, expected) in [
        ("parseInt('9007199254740993')", 9_007_199_254_740_992.0),
        ("parseInt('9007199254740995')", 9_007_199_254_740_996.0),
        ("parseInt('-9007199254740995')", -9_007_199_254_740_996.0),
        (
            "parseInt('ffffffffffffffff',16)",
            18_446_744_073_709_551_616.0,
        ),
        ("parseInt('120000000000000000000102',3)", 156_905_298_056.0),
    ] {
        number(source, expected);
    }
    number(
        &format!("parseInt('{}',2)", "1".repeat(1024)),
        f64::INFINITY,
    );
    number(
        &format!("parseInt('-{}',2)", "1".repeat(1024)),
        f64::NEG_INFINITY,
    );
}

#[test]
fn input_string_conversion_precedes_radix_conversion_and_preserves_abrupt_results() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let order=''; let s={toString:()=>{order+='s';return '11';},valueOf:()=>{throw 1;}};let r={valueOf:()=>{order+='r';return 2;}};parseInt(s,r)===3 && order==='sr'"),Ok(Value::Boolean(true)));
    assert_eq!(
        realm.eval("order='';parseFloat(s)===11 && order==='s'"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.eval("order='';parseInt({toString:()=>{throw 7;}},r)"),
        Err(Error::Thrown(Value::Number(7.0)))
    );
    assert_eq!(realm.eval("order"), Ok(Value::String(JsString::from(""))));
    assert_eq!(
        realm.eval("parseInt(s,{valueOf:()=>{throw 8;}})"),
        Err(Error::Thrown(Value::Number(8.0)))
    );
    assert_eq!(realm.eval("order"), Ok(Value::String(JsString::from("s"))));
    assert!(matches!(
        realm.eval("parseInt('10',2n)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    for function in ["parseFloat", "parseInt"] {
        assert!(matches!(
            realm.eval(&format!("{function}({{__proto__:null}})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn global_and_number_methods_share_identity_and_standard_metadata() {
    let mut realm = Realm::default();
    for (name, length) in [("parseFloat", 1.0), ("parseInt", 2.0)] {
        assert_eq!(
            realm.eval(&format!("{name}===Number.{name}")),
            Ok(Value::Boolean(true))
        );
        let Value::Object(handle) = realm.eval(name).unwrap() else {
            panic!("function")
        };
        let object = realm.inspect_object(&handle).unwrap();
        assert!(object.is_callable() && !object.is_constructor());
        for (key, value) in [
            ("name", Value::String(JsString::from(name))),
            ("length", Value::Number(length)),
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
            realm.eval(&format!("new {name}('1')")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval(&format!(
                "delete globalThis.{name}; typeof {name}==='undefined' && Number.{name}('1')===1"
            )),
            Ok(Value::Boolean(true))
        );
    }
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("Number.parseInt('10',2)+Number.parseFloat('1.5')"),
        Ok(Value::Number(3.5))
    );
}
