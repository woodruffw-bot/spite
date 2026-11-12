//! Reflect property reads, presence checks, and boolean deletion.

use crate::common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Realm, Value};

use crate::common::assert_true;

#[test]
fn primitive_targets_fail_before_key_conversion_and_keys_use_string_hint() {
    for method in ["get", "has", "deleteProperty"] {
        for target in ["undefined", "null", "true", "1", "'s'", "1n", "Symbol()"] {
            assert_true(&format!(
                r#"
                    let reads = 0,
                        caught = false,
                        key = {{
                            toString() {{
                                reads++;
                                throw 7;
                            }}
                        }};
                    try {{
                        Reflect.{method}({target}, key);
                    }} catch (e) {{
                        caught = e instanceof TypeError;
                    }}
                    caught && reads === 0
                "#
            ));
        }
        assert_true(&format!(
            r#"
                let calls = 0,
                    s = Symbol(),
                    o = {{
                        [s]: 1
                    }},
                    key = {{
                        [Symbol.toPrimitive](hint) {{
                            if (hint !== 'string') throw 7;
                            calls++;
                            return s;
                        }},
                        toString() {{
                            throw 8;
                        }}
                    }};
                Reflect.{method}(o, key);
                calls === 1
            "#
        ));
        assert_true(&format!(
            "let caught=false;try{{Reflect.{method}({{}},{{toString(){{throw 7;}}}});}}catch(e){{caught=e===7;}}caught"
        ));
    }
    assert_true(
        r#"
            let o = {
                undefined: 1,
                null: 2,
                true: 3,
                1: 4
            };
            Reflect.get(o) === 1 && Reflect.get(o, null) === 2 && Reflect.get(o, true) === 3 && Reflect
                .get(o, 1n) === 4
        "#,
    );
}

#[test]
fn get_preserves_exact_receivers_and_only_uses_them_for_getters() {
    assert_true(
        r#"
            let o = {
                get x() {
                    'use strict';
                    if (arguments.length !== 0) throw 7;
                    return this;
                }
            };
            Reflect.get(o, 'x') === o && Reflect.get(o, 'x', undefined) === undefined && Reflect.get(o,
                'x', null) === null && Reflect.get(o, 'x', 3) === 3 && Reflect.get(o, 'x', 1n) === 1n
        "#,
    );
    assert_true(
        r#"
            let receiver = {
                    y: 7
                },
                p = {
                    get x() {
                        return this.y;
                    }
                },
                o = Object.create(p);
            Reflect.get(o, 'x', receiver) === 7 && Reflect.get(p, 'x', receiver) === 7
        "#,
    );
    assert_true(
        r#"
            let reads = 0,
                receiver = {
                    get x() {
                        reads++;
                        throw 7;
                    }
                },
                o = {
                    x: 3
                };
            Reflect.get(o, 'x', receiver) === 3 && Reflect.get(o, 'absent', receiver) === undefined &&
                reads === 0
        "#,
    );
    assert_true("let o={set x(v){throw 7;}};Reflect.get(o,'x')===undefined");
    assert_true(
        r#"
            let o = {
                    x: 1
                },
                key = {
                    toString() {
                        delete o.x;
                        Object.setPrototypeOf(o, {
                            x: 7
                        });
                        return 'x';
                    }
                };
            Reflect.get(o, key) === 7
        "#,
    );
    assert_true(
        r#"
            let caught = false,
                p = {
                    get x() {
                        throw 7;
                    }
                },
                o = Object.create(p);
            try {
                Reflect.get(o, 'x', {});
            } catch (e) {
                caught = e === 7;
            }
            caught
        "#,
    );
}

#[test]
fn has_checks_own_and_inherited_descriptors_without_getter_calls() {
    assert_true(
        r#"
            let reads = 0,
                p = {
                    get x() {
                        reads++;
                        throw 7;
                    }
                },
                o = Object.create(p);
            Object.defineProperty(o, 'y', {
                value: undefined
            });
            Reflect.has(o, 'x') && Reflect.has(o, 'y') && !Reflect.has(o, 'z') && reads === 0
        "#,
    );
    assert_true(
        r#"
            let s = Symbol('x'),
                t = Symbol('x'),
                o = Object.create({
                    [s]: 1
                });
            Reflect.has(o, s) && !Reflect.has(o, t) && Reflect.get(o, s) === 1
        "#,
    );
    assert_true(
        r#"
            let o = {
                    x: 1
                },
                key = {
                    toString() {
                        delete o.x;
                        Object.setPrototypeOf(o, {
                            x: 7
                        });
                        return 'x';
                    }
                };
            Reflect.has(o, key)
        "#,
    );
    assert_true(
        "Reflect.has(Array,'fromAsync') && Reflect.has(Reflect,'set') && Reflect.has(globalThis,'Proxy')",
    );
}

#[test]
fn delete_only_removes_configurable_own_properties_and_returns_boolean() {
    assert_true(
        r#"
            let reads = 0,
                p = {
                    x: 7
                },
                o = Object.create(p);
            Object.defineProperty(o, 'x', {
                get() {
                    reads++;
                    throw 8;
                },
                configurable: true
            });
            Reflect.deleteProperty(o, 'x') === true && o.x === 7 && !Object.hasOwn(o, 'x') && Reflect
                .deleteProperty(o, 'x') === true && p.x === 7 && reads === 0
        "#,
    );
    assert_true(
        r#"
            'use strict';
            let o = {};
            Object.defineProperty(o, 'x', {
                value: 7
            });
            Reflect.deleteProperty(o, 'x') === false && o.x === 7 && Reflect.deleteProperty(o,
                'absent') === true
        "#,
    );
    assert_true(
        r#"
            let o = {
                x: 7
            };
            Object.preventExtensions(o);
            Reflect.deleteProperty(o, 'x') === true && Reflect.deleteProperty(o, 'x') === true && !Reflect
                .has(o, 'x')
        "#,
    );
    assert_true("let s=Symbol(),o={[s]:7};Reflect.deleteProperty(o,s)===true && !Reflect.has(o,s)");
    assert_true(
        r#"
            let o = {
                    x: 7
                },
                key = {
                    toString() {
                        Object.defineProperty(o, 'x', {
                            configurable: false
                        });
                        return 'x';
                    }
                };
            Reflect.deleteProperty(o, key) === false && o.x === 7
        "#,
    );
}

#[test]
fn array_string_and_mapped_argument_properties_retain_exotic_behavior() {
    assert_true(
        r#"
            let a = [1, 2];
            Reflect.get(a, 'length') === 2 && Reflect.has(a, 0) && Reflect.deleteProperty(a, 0) && !Reflect
                .has(a, 0) && a.length === 2 && Reflect.deleteProperty(a, 'length') === false
        "#,
    );
    assert_true(
        r#"
            let s = Object('ab');
            Reflect.get(s, 0) === 'a' && Reflect.has(s, 1) && !Reflect.has(s, 2) && Reflect.deleteProperty(
                    s, 0) === false && Reflect.deleteProperty(s, 'length') === false && Reflect
                .deleteProperty(s, 2) === true
        "#,
    );
    assert_true(
        r#"
            function f(a) {
                a = 2;
                if (Reflect.get(arguments, 0) !== 2) throw 7;
                Reflect.deleteProperty(arguments, 0);
                a = 3;
                return !Reflect.has(arguments, 0) && Reflect.get(arguments, 0) === undefined && arguments
                    .length === 1;
            }
            f(1)
        "#,
    );
}

#[test]
fn metadata_and_retained_methods_survive_public_deletion() {
    for method in ["get", "has", "deleteProperty"] {
        assert_true(&format!(
            r#"
                let d = Object.getOwnPropertyDescriptor(Reflect, '{method}'),
                    n = Object.getOwnPropertyDescriptor(d.value, 'name'),
                    l = Object.getOwnPropertyDescriptor(d.value, 'length');
                d.writable && !d.enumerable && d.configurable && n.value === '{method}' && !n.writable &&
                    !n.enumerable && n.configurable && l.value === 2 && !l.writable && !l.enumerable && l
                    .configurable && !Object.hasOwn(d.value, 'prototype')
            "#
        ));
        assert_true(&format!(
            "let caught=false;try{{new Reflect.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                let get = Reflect.get,
                    has = Reflect.has,
                    remove = Reflect.deleteProperty;
                delete Reflect.get;
                delete Reflect.has;
                delete Reflect.deleteProperty;
                delete globalThis.Reflect
            "#,
        )
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("let o={x:7};get(o,'x')===7 && has(o,'x') && remove(o,'x') && !has(o,'x')"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unsupported_keys_getters_and_proxy_mutations_skip_pending_handlers() {
    for operation in [
        "Reflect.get({get x(){gap;}},'x')",
        "Reflect.has({},{toString(){gap;}})",
        "Reflect.deleteProperty({},{toString(){gap;}})",
        "Reflect.get(new Proxy(Array,{get(){gap;}}),'fromAsync')",
        "Reflect.deleteProperty(new Proxy(Array,{deleteProperty(){gap;}}),'fromAsync')",
    ] {
        let mut realm = Realm::default();
        if operation.contains("gap") {
            realm.reserve_unsupported_global("gap");
        }
        realm.eval("let flag=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{{operation};}}catch{{flag=1;}}finally{{flag=2;}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{operation}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
