//! Call evaluation ordering before callable function values are introduced.

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

#[test]
fn noncallable_values_produce_typeerror_after_evaluating_arguments() {
    for callee in [
        "undefined",
        "null",
        "true",
        "1",
        "1n",
        "'x'",
        "({})",
        "({x: 1}).x",
    ] {
        let mut realm = Realm::default();
        realm.eval("let sequence = 0").unwrap();
        let source =
            format!("({callee})(sequence = sequence * 10 + 1, sequence = sequence * 10 + 2,)");
        assert!(
            matches!(
                realm.eval(&source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("sequence"), Ok(Value::Number(12.0)));
    }
}

#[test]
fn callee_getvalue_precedes_arguments_and_argument_errors_precede_callable_check() {
    let mut realm = Realm::default();
    realm.eval("let flag = 0").unwrap();
    for (source, kind) in [
        ("missing(flag = 1)", ExceptionKind::ReferenceError),
        ("null.x(flag = 1)", ExceptionKind::TypeError),
        (
            "({})[{__proto__: null}](flag = 1)",
            ExceptionKind::TypeError,
        ),
        ("(0)(1n / 0n, flag = 1)", ExceptionKind::RangeError),
    ] {
        assert!(
            matches!(realm.eval(source), Err(Error::Exception { kind: actual, .. }) if actual == kind),
            "{source}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    assert!(matches!(
        realm.eval("(flag = 1, {})[flag = 2](flag = 3)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(3.0)));
}

#[test]
fn language_call_failures_are_catchable_but_missing_intrinsics_are_host_gaps() {
    assert_eq!(
        Realm::default().eval(
            "let flag = 0; try { ({})(flag = 1); } catch { flag += 2; } finally { flag += 4; } flag"
        ),
        Ok(Value::Number(7.0))
    );
    let mut realm = Realm::default();
    realm.eval("let flag = 0").unwrap();
    assert!(matches!(
        realm.eval("try { ({}).constructor(flag = 1); } catch { flag = 2; } finally { flag = 3; }"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("true || missing()"), Ok(Value::Boolean(true)));
}

#[test]
fn argument_evaluation_obeys_host_work_limits() {
    let mut realm = Realm::new(Limits {
        max_steps: 1000,
        ..Limits::default()
    });
    realm.eval("let flag = 0").unwrap();
    let source = format!(
        "try {{ (0)({}); }} finally {{ flag = 1; }}",
        vec!["1"; 2000].join(",")
    );
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
