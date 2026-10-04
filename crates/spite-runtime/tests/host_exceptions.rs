//! Bounded host access to exception values and JavaScript properties.

use spite_core::{JsString, Span};
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

#[test]
fn host_conversion_preserves_diagnostics_and_exposes_standard_error_values() {
    let mut realm = Realm::default();
    let error = realm.eval("+1n").unwrap_err();
    let original = error.clone();
    let value = realm.exception_value(&error).unwrap().unwrap();
    assert_eq!(error, original);
    assert_eq!(
        realm.read_property(&value, &JsString::from("name")),
        Ok(Value::String(JsString::from("TypeError")))
    );
    let Value::Object(handle) = &value else {
        panic!("error");
    };
    assert!(realm.inspect_object(handle).unwrap().is_error());
    let root = realm.root_value(value.clone(), 1_000).unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.read_property(root.value(), &JsString::from("message")),
        Ok(Value::String(JsString::from(
            "cannot convert BigInt to Number"
        )))
    );
    let second = realm.exception_value(&error).unwrap().unwrap();
    assert!(!value.same_value(&second));
}

#[test]
fn explicit_throws_retain_identity_and_host_failures_have_no_javascript_value() {
    let mut realm = Realm::default();
    let error = realm.eval("throw new RangeError('explicit')").unwrap_err();
    let Error::Thrown(value) = &error else {
        panic!("throw");
    };
    assert_eq!(realm.exception_value(&error), Ok(Some(value.clone())));
    assert_eq!(
        realm.exception_value(&Error::Thrown(Value::Number(7.0))),
        Ok(Some(Value::Number(7.0)))
    );
    for error in [
        realm.eval("let =").unwrap_err(),
        realm.eval("Math").unwrap_err(),
        Error::Limit {
            span: Span::new(0, 0),
            message: "host abort".into(),
        },
    ] {
        assert_eq!(realm.exception_value(&error), Ok(None));
    }
}

#[test]
fn property_reads_check_handles_and_keep_host_work_and_string_limits() {
    let mut realm = Realm::default();
    assert_eq!(
        realm.read_property(
            &Value::String(JsString::from("abc")),
            &JsString::from("length")
        ),
        Ok(Value::Number(3.0))
    );
    assert!(matches!(
        realm.read_property(&Value::Null, &JsString::from("name")),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let object = realm.eval("({x:1})").unwrap();
    let mut other = Realm::default();
    assert!(matches!(
        other.read_property(&object, &JsString::from("x")),
        Err(Error::InvalidObject(_))
    ));
    realm.collect(100_000).unwrap();
    assert!(matches!(
        realm.read_property(&object, &JsString::from("x")),
        Err(Error::InvalidObject(_))
    ));
    assert!(matches!(
        realm.exception_value(&Error::Thrown(object)),
        Err(Error::InvalidObject(_))
    ));
    let mut limited = Realm::new(Limits {
        max_steps: Some(0),
        ..Limits::default()
    });
    assert!(matches!(
        limited.read_property(&Value::Number(1.0), &JsString::from("x")),
        Err(Error::Limit { .. })
    ));
    let mut limited = Realm::new(Limits {
        max_string_units: 1,
        ..Limits::default()
    });
    assert!(matches!(
        limited.read_property(&Value::Number(1.0), &JsString::from("long")),
        Err(Error::Limit { .. })
    ));
}
