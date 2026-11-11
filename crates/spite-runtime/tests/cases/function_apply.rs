//! Function.prototype.apply and ordered, bounded CreateListFromArrayLike.

use crate::common::assert_true;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(expected.into())),
        "{source}"
    );
}

#[test]
fn nullish_lists_are_empty_and_receivers_pass_through_unchanged() {
    for source in [
        "({}).toString.apply()",
        "({}).toString.apply(undefined)",
        "({}).toString.apply(undefined, null)",
        "({}).toString.apply(undefined, undefined)",
    ] {
        string(source, "[object Undefined]");
    }
    string("({}).toString.apply(null, {})", "[object Null]");
    string("({}).toString.apply(1n, {length: 0})", "[object BigInt]");
    assert_true("let o = {}; ({}).valueOf.apply(o, {}) === o");
}

#[test]
fn lists_include_inherited_indices_and_undefined_holes() {
    for list in [
        "{0: null, length: 1}",
        "{__proto__: {0: null, length: 1}}",
        "{0: null, length: '1.9'}",
    ] {
        string(
            &format!("({{}}).toString.call.apply(({{}}).toString, {list})"),
            "[object Null]",
        );
    }
    string(
        "({}).toString.call.apply(({}).toString, {length: 1})",
        "[object Undefined]",
    );
    string(
        "({}).toString.call.apply(({}).toString, {0: true, 1: false, length: 2})",
        "[object Boolean]",
    );
    string(
        "({}).toString.apply.call(({}).toString, 'x', {length: 0})",
        "[object String]",
    );
}

#[test]
fn length_uses_tolength_and_does_not_coerce_index_values() {
    for length in [
        "undefined",
        "null",
        "NaN",
        "-Infinity",
        "-1",
        "-0",
        "0.9",
        "''",
        "'invalid'",
        "{}",
    ] {
        string(
            &format!("({{}}).toString.call.apply(({{}}).toString, {{0: null, length: {length}}})"),
            "[object Undefined]",
        );
    }
    for length in ["true", "1.9", "'1'", "{valueOf: ({}).toString.call}"] {
        let source =
            format!("({{}}).toString.call.apply(({{}}).toString, {{0: null, length: {length}}})");
        if length.starts_with('{') {
            assert!(matches!(
                Realm::default().eval(&source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ));
        } else {
            string(&source, "[object Null]");
        }
    }
    for length in ["1n", "{__proto__: null}"] {
        assert!(matches!(
            Realm::default().eval(&format!(
                "({{}}).toString.apply(null, {{length: {length}}})"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    string(
        "({}).toString.call.apply(({}).toString, {0: {__proto__: null}, length: 1})",
        "[object Object]",
    );
}

#[test]
fn nonobjects_are_rejected_and_callable_check_precedes_list_inspection() {
    for list in ["0", "1n", "true", "'abc'"] {
        assert!(matches!(
            Realm::default().eval(&format!("({{}}).toString.apply(null, {list})")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    assert!(matches!(
        Realm::default()
            .eval("let o = {apply: ({}).toString.apply}; o.apply(null, {length: Infinity})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm
        .eval("let flag = 0; let o = {apply: ({}).toString.apply}")
        .unwrap();
    assert!(matches!(
        realm.eval("o.apply(null, (flag = 1, {}))"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
}

#[test]
fn unlimited_argument_lists_preserve_full_width_getter_failures() {
    assert_true(
        r#"
            let reads = 0,
                o = {
                    length: Infinity,
                    get 0() {
                        reads++;
                        throw 17;
                    }
                };
            let caught = false;
            try {
                Boolean.apply(null, o);
            } catch (e) {
                caught = e === 17;
            }
            caught && reads === 1
        "#,
    );
    // CreateListFromArrayLike reads each index before appending its value:
    // https://262.ecma-international.org/17.0/#sec-createlistfromarraylike
    for length in ["Infinity", "4294967296", "9007199254740991", "1e100"] {
        for operation in [
            "target.apply(null,args)",
            "Reflect.apply(target,null,args)",
            "Reflect.construct(target,args)",
            "Reflect.apply(new Proxy(target,{apply(){throw 99}}),null,args)",
            "Reflect.construct(new Proxy(target,{construct(){throw 99}}),args)",
            "Reflect.construct(target,args,new Proxy(function(){},{get(){throw 99}}))",
        ] {
            for proxy in [false, true] {
                let source = r#"
                    let log = '',
                        marker = {},
                        args = {
                            get length() {
                                if (this !== args) throw 99;
                                log += 'l';
                                return LENGTH
                            },
                            get 0() {
                                if (this !== args) throw 99;
                                log += '0';
                                throw marker
                            }
                        };
                    if (PROXY) args = new Proxy(args, {
                        get(t, k, r) {
                            log += 'g' + String(k) + ';';
                            return Reflect.get(t, k, r)
                        }
                    });

                    function target() {
                        throw 99
                    }
                    let caught = false;
                    try {
                        CALL
                    } catch (e) {
                        caught = e === marker
                    }
                    caught && log === TRACE
                "#
                .replace("LENGTH", length)
                .replace("PROXY", if proxy { "true" } else { "false" })
                .replace("CALL", operation)
                .replace("TRACE", if proxy { "'glength;lg0;0'" } else { "'l0'" });
                assert_eq!(
                    Realm::default().eval(&source),
                    Ok(Value::Boolean(true)),
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn list_lengths_and_direct_calls_have_explicit_host_argument_limits() {
    for length in ["4", "Infinity", "9007199254740991", "1e100"] {
        let mut realm = Realm::new(Limits {
            max_arguments: Some(3),
            ..Limits::default()
        });
        realm.eval("let flag = 0").unwrap();
        assert!(matches!(realm.eval(&format!("try {{ ({{}}).toString.apply(null, {{length: {length}}}); }} catch {{ flag = 1; }} finally {{ flag = 2; }}")), Err(Error::Limit { .. })));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    let mut realm = Realm::new(Limits {
        max_arguments: Some(1),
        ..Limits::default()
    });
    assert_eq!(
        realm.eval("({}).toString(null)"),
        Ok(Value::String("[object Object]".into()))
    );
    assert!(matches!(
        realm.eval("({}).toString(null, null)"),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn apply_metadata_and_native_source_are_standard() {
    assert_true(
        "let a = ({}).toString.apply; a.name === 'apply' && a.length === 2 && typeof a === 'function' && !('prototype' in a)",
    );
    string(
        "({}).toString.apply.toString()",
        "function apply() { [native code] }",
    );
}

// Completed failures retain no argument captures after the host checkpoint.
#[test]
fn failed_argument_lists_and_calls_release_object_and_symbol_captures() {
    for invoke in [
        "Reflect.apply(target,null,list)",
        "Function.prototype.apply.call(target,null,list)",
    ] {
        for path in ["get", "call", "class", "revoked"] {
            let source = r#"
                let state = (() => {
                    const path = 'PATH';
                    let object = {},
                        symbol = Symbol(),
                        weak = [new WeakRef(object), new WeakRef(symbol)],
                        caught = false;
                    let target = path === 'class' ? class {} : function(a, b) {
                        throw 7
                    };
                    if (path === 'revoked') {
                        let pair = Proxy.revocable(target, {});
                        pair.revoke();
                        target = pair.proxy
                    }
                    let list = {
                        length: path === 'get' ? 3 : 2,
                        get 0() {
                            return object
                        },
                        get 1() {
                            return symbol
                        },
                        get 2() {
                            throw 7
                        }
                    };
                    try {
                        INVOKE
                    } catch (e) {
                        caught = path === 'get' || path === 'call' ? e === 7 : e instanceof TypeError
                    }
                    return {
                        weak,
                        caught
                    }
                })();
                state.caught && state.weak.every(r => r.deref() !== undefined)
            "#
            .replace("PATH", path)
            .replace("INVOKE", invoke);
            let mut realm = Realm::default();
            let before = realm.eval(&source);
            assert_eq!(before, Ok(Value::Boolean(true)), "{invoke}/{path}: before");
            realm.clear_kept_objects().unwrap();
            realm.collect(usize::MAX).unwrap();
            let after = realm.eval("state.weak.every(r=>r.deref()===undefined)");
            assert_eq!(after, Ok(Value::Boolean(true)), "{invoke}/{path}: after");
        }
    }
}
