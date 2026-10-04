//! Property references and edition-17 GetValue/PutValue ordering.

mod common;
use common::REALM_ENTRIES;

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn number(source: &str, expected: f64) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Number(expected)),
        "{source}"
    );
}

fn boolean(source: &str, expected: bool) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(expected)),
        "{source}"
    );
}

#[test]
fn own_and_inherited_properties_support_dotted_and_computed_reads() {
    number(
        "let base = {x: 3}; let o = {__proto__: base, y: 4}; o.x + o['y']",
        7.0,
    );
    number("({a: {b: {c: 5}}}).a['b'].c", 5.0);
    number(
        "({if: 1, null: 2, true: 3, false: 4}).if + ({null: 2}).null",
        3.0,
    );
    number(r"({'\ud800': 6})['\ud800']", 6.0);
    number("({16: 7})[0x10n]", 7.0);
    boolean("({x: undefined}).x === undefined", true);
    boolean("({}).missing === undefined", true);
    boolean("({__proto__: null}).toString === undefined", true);
    boolean("({}).__proto__ === undefined", true); // optional legacy accessor is disabled
    boolean("({toString: undefined}).toString === undefined", true);
}

#[test]
fn assignment_captures_base_and_key_once_and_puts_after_rhs() {
    number(
        "let a = {x: 1}; let old = a; a.x = (a = {x: 3}, 2); old.x * 10 + a.x",
        23.0,
    );
    number(
        "let o = {}; let key = 0; o[key += 1] = (key += 1); o[1] * 10 + key",
        22.0,
    );
    number(
        "let o = {}; let sequence = 0; (sequence = 1, o)[sequence = 2] = (sequence = 3); o[2] * 10 + sequence",
        33.0,
    );
    number("let o = {}; (o.x) = 4; o.x", 4.0);
    number("let a = {}, b = {}; a.x = b.y = 9; a.x + b.y", 18.0);
    number(
        "let base = {x: 1}; let child = {__proto__: base}; child.x = 7; child.x * 10 + base.x",
        71.0,
    );
}

#[test]
fn compound_assignment_reads_before_rhs_and_logical_forms_short_circuit() {
    number("let o = {x: 1}; o.x += (o.x = 10, 2); o.x", 3.0);
    number(
        "let o = {1: 2}; let key = 0; o[++key] *= (key += 1, 3); o[1] * 10 + key",
        62.0,
    );
    for (operator, initial, expected) in [("&&=", "0", 0.0), ("||=", "4", 4.0), ("??=", "0", 0.0)] {
        number(
            &format!("let o = {{x: {initial}}}; o.x {operator} missing; o.x"),
            expected,
        );
    }
    number("let o = {x: null}; o.x ??= 7; o.x", 7.0);
    number("let o = {x: 2}; o.x &&= 4; o.x", 4.0);
    number("let o = {x: 0}; o.x ||= 4; o.x", 4.0);
    number("let o = {x: 5}; o.x <<= 2; o.x ^= 3; o.x", 23.0);
}

#[test]
fn updates_preserve_numeric_types_and_postfix_returns_the_old_numeric_value() {
    number("let o = {x: '2'}; let old = o.x++; old * 10 + o.x", 23.0);
    number("let o = {x: 3}; --o['x']", 2.0);
    boolean(
        "let o = {x: 1n}; let old = o.x++; old === 1n && o.x === 2n",
        true,
    );
    boolean(
        "let o = {x: 0n}; ++o.x === 1n && o.x-- === 1n && o.x === 0n",
        true,
    );
    number(
        "let base = {x: 3}; let o = {__proto__: base}; ++o.x; o.x * 10 + base.x",
        43.0,
    );
}

#[test]
fn deletion_avoids_getvalue_and_removes_only_own_properties() {
    boolean(
        "let base = {x: 1}; let o = {__proto__: base, x: 2}; delete o.x && o.x === 1 && base.x === 1",
        true,
    );
    boolean("let o = {}; delete o.toString", true); // inherited properties are not deleted
    boolean(
        "'use strict'; let o = {x: 1}; delete (o.x) && o.x === undefined",
        true,
    );
    number("let o = {1: 4}; let key = 0; delete o[++key]; key", 1.0);
    boolean("let o = {x: 1}; delete (0, o.x) && o.x === 1", true);
    boolean(
        "let o = {}; o.__proto__ = 3; delete o.__proto__ && o.__proto__ === undefined",
        true,
    );
}

#[test]
fn nullish_bases_fail_at_get_put_or_delete_after_key_expression_evaluation() {
    for source in [
        "null.x",
        "undefined['x']",
        "typeof null.x",
        "delete null.x",
        "null.x++",
        "++undefined.x",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    number(
        "let flag = 0; try { null[(flag = 1, 'x')] = (flag = 2); } catch {} flag",
        2.0,
    );
    number(
        "let flag = 0; try { null[(flag = 1, 'x')] += (flag = 2); } catch {} flag",
        1.0,
    );
    number(
        "let flag = 0; try { delete null[(flag = 1, 'x')]; } catch {} flag",
        1.0,
    );
    assert!(matches!(
        Realm::default().eval("null.x = 1n / 0n"),
        Err(Error::Exception {
            kind: ExceptionKind::RangeError,
            ..
        })
    ));
    assert!(matches!(
        Realm::default().eval("null[missing] = 1n / 0n"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    // ToObject precedes ToPropertyKey, so failed object-key coercion
    // cannot mask the required null-base TypeError.
    assert!(matches!(
        Realm::default().eval("null[{__proto__: null}]"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn simple_assignment_defers_key_conversion_but_compound_assignment_converts_before_rhs() {
    for (operator, expected) in [("=", 1.0), ("+=", 0.0), ("??=", 0.0)] {
        let mut realm = Realm::default();
        realm.eval("let flag = 0; let o = {};").unwrap();
        let result = realm.eval(&format!("o[{{__proto__: null}}] {operator} (flag = 1)"));
        assert!(matches!(
            result,
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(expected)));
    }
}

#[test]
fn incomplete_intrinsic_methods_report_unsupported() {
    for name in [
        "create",
        "defineProperties",
        "getPrototypeOf",
        "freeze",
        "keys",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag = 0").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try {{ Object.{name}; }} catch {{ flag = 1; }} finally {{ flag = 2; }}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
        number(&format!("({{__proto__: {{ {name}: 7 }} }}).{name}"), 7.0);
        number(&format!("let o = {{}}; o.{name} = 3; o.{name}"), 3.0);
    }
    for source in ["'abc'.slice", "1n.toString"] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
}

#[test]
fn property_references_preserve_identity_through_control_flow_and_collection() {
    let mut realm = Realm::default();
    assert_eq!(
        realm
            .eval("let o = {}; o.self = o; try { throw o; } catch (e) { e.flag = 7; } o.self.flag"),
        Ok(Value::Number(7.0))
    );
    assert_eq!(realm.collect(1000).unwrap().live, REALM_ENTRIES + 1);
    assert_eq!(realm.eval("o === o.self"), Ok(Value::Boolean(true)));
    realm.eval("o = null").unwrap();
    assert_eq!(realm.collect(1000).unwrap().reclaimed, 1);
}

#[test]
fn long_prototype_reads_are_bounded_and_host_failures_skip_finalizers() {
    let mut realm = Realm::new(Limits {
        max_steps: 512,
        ..Limits::default()
    });
    realm.eval("let p = null; let flag = 0").unwrap();
    for _ in 0..180 {
        realm.eval("p = {__proto__: p}").unwrap();
    }
    assert!(matches!(
        realm.eval("try { p.missing; } finally { flag = 1; }"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
