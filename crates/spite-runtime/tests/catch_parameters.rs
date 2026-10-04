//! Catch binding initialization, mutable scopes, and restoration across completions.

use spite_core::{DiagnosticKind, JsString};
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn evaluates(source: &str, expected: Value) {
    for prefix in ["", "'use strict'; "] {
        let actual = Realm::default().eval(&format!("{prefix}{source}")).unwrap();
        assert!(actual.same_value(&expected), "{source}: {actual:?}");
    }
}

#[test]
fn catch_binding_preserves_every_primitive_type_and_special_number() {
    for (expression, expected) in [
        ("undefined", Value::Undefined),
        ("null", Value::Null),
        ("true", Value::Boolean(true)),
        ("false", Value::Boolean(false)),
        ("-0", Value::Number(-0.0)),
        ("NaN", Value::Number(f64::NAN)),
        ("Infinity", Value::Number(f64::INFINITY)),
        (
            "'\\ud800'",
            Value::String(JsString::from_code_units(vec![0xd800])),
        ),
    ] {
        evaluates(
            &format!("try {{ throw {expression}; }} catch (e) {{ e; }}"),
            expected,
        );
    }
    evaluates(
        "try { throw 9007199254740993n; } catch (e) { e === 9007199254740993n; }",
        Value::Boolean(true),
    );
    evaluates("try { throw 7; } catch (e) {}", Value::Undefined);
    evaluates(
        "try { throw undefined; } catch (e) { e = 8; e; }",
        Value::Number(8.0),
    );
}

#[test]
fn parameters_are_mutable_shadow_outer_bindings_and_do_not_leak() {
    evaluates(
        "try { throw 7; } catch (e) { e += 1; e++; e; }",
        Value::Number(9.0),
    );
    evaluates(
        "let e = 3; try { throw 7; } catch (e) { e = 8; } e",
        Value::Number(3.0),
    );
    evaluates(
        "var e = 3; try { throw 7; } catch (e) { e = 8; } e",
        Value::Number(3.0),
    );
    evaluates(
        "try { throw 7; } catch (e) { { let e = 8; e++; } e; }",
        Value::Number(7.0),
    );
    evaluates(
        "try { throw 7; } catch (e) { var result = e; } result",
        Value::Number(7.0),
    );
    evaluates(
        "try { throw 7; } catch (e) {} typeof e",
        Value::String(JsString::from("undefined")),
    );
    evaluates(
        "try { throw 7; } catch (e) { e; } let e = 8; e",
        Value::Number(8.0),
    );
    assert_eq!(
        Realm::default().eval("try { throw 7; } catch (e) { delete e; }"),
        Ok(Value::Boolean(false))
    );
    assert!(matches!(
        Realm::default().eval("try { throw 7; } catch (e) { { e; let e; } }"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn normal_and_control_completions_bypass_catch_binding_initialization() {
    evaluates("try { 7; } catch (e) { missing; }", Value::Number(7.0));
    evaluates(
        "label: { try { 8; break label; } catch (e) { missing; } }",
        Value::Number(8.0),
    );
    evaluates(
        "let i = 0; while (i++ < 2) { try { continue; } catch (e) { missing; } } i",
        Value::Number(3.0),
    );
    evaluates("try {} catch (e) { missing; }", Value::Undefined);
}

#[test]
fn rethrows_and_finalizers_observe_restored_environments() {
    evaluates(
        "try { try { throw 7; } catch (e) { e++; throw e; } } catch (e) { e; }",
        Value::Number(8.0),
    );
    evaluates(
        "try { throw 7; } catch (e) { e++; e; } finally { 99; }",
        Value::Number(8.0),
    );
    evaluates(
        "let e = 3; let seen; try { throw 7; } catch (e) { e++; } finally { seen = e; } seen",
        Value::Number(3.0),
    );
    evaluates(
        "let seen; try { throw 7; } catch (e) {} finally { seen = typeof e; } seen",
        Value::String(JsString::from("undefined")),
    );
    evaluates(
        "let total = 0; for (let i = 0; i < 3; i++) { try { throw i; } catch (e) { total += e; continue; } finally { total += 10; } } total",
        Value::Number(33.0),
    );
    let mut realm = Realm::default();
    realm.eval("let seen").unwrap();
    assert_eq!(
        realm.eval(
            "label: { try { throw 7; } catch (e) { e; break label; } finally { seen = typeof e; } }"
        ),
        Ok(Value::Number(7.0))
    );
    assert_eq!(
        realm.eval("seen"),
        Ok(Value::String(JsString::from("undefined")))
    );
    assert_eq!(
        realm.eval("try { throw 7; } catch (e) { throw e + 1; } finally { 99; }"),
        Err(Error::Thrown(Value::Number(8.0)))
    );
    assert_eq!(
        realm.eval("try { throw 7; } catch (e) { throw e; } finally { throw 9; }"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
}

#[test]
fn host_abort_restores_both_catch_scopes_and_skips_pending_handlers() {
    for body in [
        "while (true) {}",
        "Object",
        "try { throw 2; } catch (inner) { while (true) {} }",
    ] {
        let mut realm = Realm::new(Limits {
            // Leave room for global property lookup after the abort; only the
            // deliberate infinite loops should exhaust this work allowance.
            max_steps: 1_000,
            ..Limits::default()
        });
        realm.eval("let e = 3; let flag = 0;").unwrap();
        let result = realm.eval(&format!("try {{ try {{ throw 7; }} catch (e) {{ {body} }} }} catch {{ flag = 1; }} finally {{ flag = 2; }}"));
        assert!(
            matches!(result, Err(Error::Limit { .. } | Error::Unsupported { .. })),
            "{body}"
        );
        assert_eq!(realm.eval("e"), Ok(Value::Number(3.0)));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        assert_eq!(
            realm.eval("typeof inner"),
            Ok(Value::String(JsString::from("undefined")))
        );
    }
}

#[test]
fn built_in_exception_bindings_receive_error_objects_and_restore_scopes() {
    for (expression, name) in [
        ("missing", "ReferenceError"),
        ("+1n", "TypeError"),
        ("1n / 0n", "RangeError"),
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag = 0; let e = 3").unwrap();
        let result = realm.eval(&format!(
            "try {{ {expression}; }} catch (e) {{ flag = Error.isError(e) && e instanceof {name} && e.constructor==={name} && e.name==='{name}' && typeof e.message==='string'; }} finally {{ if(flag===true)flag = 2; }}"
        ));
        assert!(result.is_ok(), "{expression}: {result:?}");
        assert_eq!(realm.eval("flag"), Ok(Value::Number(2.0)));
        assert_eq!(realm.eval("e"), Ok(Value::Number(3.0)));
        assert_eq!(
            realm.eval(&format!("try {{ {expression}; }} catch {{ 9; }}")),
            Ok(Value::Number(9.0))
        );
    }
}

#[test]
fn materialized_errors_preserve_identity_across_rethrows_and_finalizers() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let saved;let flag=0;try{try{+1n;}catch(e){saved=e;throw e;}finally{flag=1;}}catch(e){e===saved && e instanceof TypeError && Error.isError(e) && flag===1}"),Ok(Value::Boolean(true)));
    let Value::Object(error) = realm.eval("saved").unwrap() else {
        panic!("error");
    };
    let message = realm
        .inspect_object(&error)
        .unwrap()
        .own_property(&JsString::from("message"))
        .unwrap()
        .as_data()
        .unwrap();
    assert!(message.writable && !message.enumerable && message.configurable);
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.eval("saved instanceof TypeError && Error.isError(saved)"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn error_creation_uses_intrinsics_after_global_bindings_are_replaced() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let T=TypeError;let R=ReferenceError;TypeError=()=>{throw 1;};delete globalThis.ReferenceError;let a,b;try{+1n;}catch(e){a=e;}try{missing;}catch(e){b=e;}a instanceof T && a.constructor===T && b instanceof R && b.constructor===R"),Ok(Value::Boolean(true)));
    assert!(matches!(
        realm.eval("+1n"),
        Err(Error::Exception {
            kind: spite_runtime::ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn catch_parameter_early_errors_prevent_all_execution() {
    for source in [
        "flag = 1; try {} catch (e) { let e; }",
        "flag = 1; try {} catch (e) { var e; }",
        "'use strict'; flag = 1; try {} catch (eval) {}",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag = 0").unwrap();
        assert!(
            matches!(realm.eval(source), Err(Error::Parse(error)) if error.kind == DiagnosticKind::Syntax)
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
