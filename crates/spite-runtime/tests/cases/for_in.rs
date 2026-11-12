//! For-in own/prototype enumeration, live descriptors, and completions (14.7.5).

use spite_runtime::{Error, Limits, Realm, Value};

use crate::common::assert_true;

#[test]
fn disabled_annex_b_initializers_are_catchable_syntax_errors_in_eval() {
    assert_true(
        r#"
        let touched=0,caught=0;
        const source='for(var forInLeaked=(touched=1) in {key:1}){}';
        for(const compile of [()=>eval(source),()=>(0,eval)(source),()=>eval('"use strict";'+source)]){
            try{compile();throw 1;}
            catch(error){if(!(error instanceof SyntaxError))throw error;caught++;}
        }
        caught===3&&touched===0&&typeof forInLeaked==='undefined'&&!Object.hasOwn(globalThis,'forInLeaked')
        "#,
    );
}

#[test]
fn dynamic_function_families_reject_disabled_for_in_initializers_during_compilation() {
    assert_true(
        r#"
            let touched=0,caught=0;
            const source='for(var forInLeaked=(touched=1) in {key:1}){}';
            const constructors=[Function,Object.getPrototypeOf(function*(){}).constructor,
                Object.getPrototypeOf(async function(){}).constructor,
                Object.getPrototypeOf(async function*(){}).constructor];
            for(const C of constructors){
                for(const compile of [()=>C(source),()=>Reflect.construct(C,[source])]){
                    try{compile();throw 1;}
                    catch(error){if(!(error instanceof SyntaxError))throw error;caught++;}
                }
            }
            caught===8&&touched===0&&typeof forInLeaked==='undefined'
        "#,
    );
}

#[test]
fn own_and_inherited_strings_are_ordered_and_symbols_are_skipped() {
    assert_true(
        r#"
            let proto = {
                    9: 1,
                    p: 2
                },
                obj = {
                    2: 3,
                    1: 4,
                    z: 5,
                    a: 6,
                    [Symbol()]: 7
                };
            Object.setPrototypeOf(obj, proto);
            let keys = [];
            for (let k in obj) keys.push(k);
            keys.join(',') === '1,2,z,a,9,p'
        "#,
    );
    assert_true(
        r#"
            let obj = {
                get a() {
                    throw 7;
                },
                set b(v) {
                    throw 8;
                },
                get [Symbol.iterator]() {
                    throw 9;
                }
            };
            let keys = [];
            for (let k in obj) keys.push(k);
            keys.join(',') === 'a,b'
        "#,
    );
    assert_true(
        "let obj=Object.create(null);obj.x=1;let keys=[];for(let k in obj)keys.push(k);keys.join(',')==='x'",
    );
}

#[test]
fn nonenumerable_own_keys_suppress_inherited_names_without_reads() {
    assert_true(
        r#"
            let proto = {
                    x: 1,
                    y: 2
                },
                obj = Object.create(proto);
            Object.defineProperty(obj, 'x', {
                get() {
                    throw 7;
                }
            });
            let keys = [];
            for (let k in obj) keys.push(k);
            keys.join(',') === 'y'
        "#,
    );
    assert_true(
        r#"
            let base = {
                    x: 1
                },
                proto = Object.create(base),
                obj = Object.create(proto);
            Object.defineProperty(proto, 'x', {
                value: 2
            });
            let keys = [];
            for (let k in obj) keys.push(k);
            keys.length === 0
        "#,
    );
    assert_true(
        r#"
            let proto = {
                    x: 1
                },
                obj = {
                    x: 2
                };
            Object.setPrototypeOf(obj, proto);
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                delete obj.x;
            }
            keys.join(',') === 'x'
        "#,
    );
}

#[test]
fn deletions_and_enumerability_changes_are_live_between_bodies() {
    assert_true(
        r#"
            let proto = {
                    b: 7,
                    c: 8
                },
                obj = {
                    a: 1,
                    b: 2
                };
            Object.setPrototypeOf(obj, proto);
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') delete obj.b;
            }
            keys.join(',') === 'a,b,c'
        "#,
    );
    assert_true(
        r#"
            let proto = {
                    b: 7
                },
                obj = {
                    a: 1,
                    b: 2
                };
            Object.setPrototypeOf(obj, proto);
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') Object.defineProperty(obj, 'b', {
                    enumerable: false
                });
            }
            keys.join(',') === 'a'
        "#,
    );
    assert_true(
        r#"
            let obj = {
                a: 1
            };
            Object.defineProperty(obj, 'b', {
                value: 2,
                configurable: true
            });
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') Object.defineProperty(obj, 'b', {
                    enumerable: true
                });
            }
            keys.join(',') === 'a,b'
        "#,
    );
    assert_true(
        r#"
            let obj = {
                    a: 1,
                    b: 2,
                    c: 3
                },
                keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') {
                    delete obj.b;
                    obj.newKey = 4;
                }
            }
            keys.join(',') === 'a,c'
        "#,
    );
}

#[test]
fn prototype_keys_are_snapshotted_when_the_prototype_is_reached() {
    assert_true(
        r#"
            let proto = {
                    p: 1
                },
                obj = {
                    a: 2
                };
            Object.setPrototypeOf(obj, proto);
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') {
                    delete proto.p;
                    proto.q = 3;
                }
            }
            keys.join(',') === 'a,q'
        "#,
    );
    assert_true(
        r#"
            let old = {
                    p: 1
                },
                next = {
                    q: 2
                },
                obj = {
                    a: 3
                };
            Object.setPrototypeOf(obj, old);
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') Object.setPrototypeOf(obj, next);
            }
            keys.join(',') === 'a,q'
        "#,
    );
    assert_true(
        r#"
            let obj = {
                    a: 1
                },
                proto = {
                    p: 2
                };
            Object.setPrototypeOf(obj, proto);
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') Object.setPrototypeOf(obj, null);
            }
            keys.join(',') === 'a'
        "#,
    );
}

#[test]
fn nullish_inputs_skip_and_other_primitives_box_without_coercion() {
    assert_true(
        r#"
            let calls = 0;
            for (let k in null) calls++;
            for (let k in undefined) calls++;
            for (let k in 7) calls++;
            for (let k in true) calls++;
            for (let k in 7n) calls++;
            for (let k in Symbol()) calls++;
            calls === 0
        "#,
    );
    assert_true(
        r#"
            let calls = 0,
                obj = {
                    [Symbol.toPrimitive]() {
                        calls++;
                        throw 7;
                    },
                    valueOf() {
                        calls++;
                        throw 8;
                    }
                };
            let keys = [];
            for (let k in obj) keys.push(k);
            keys.join(',') === 'valueOf' && calls === 0
        "#,
    );
    assert_true(
        "Number.prototype.x=1;let keys=[];for(let k in 7)keys.push(k);keys.join(',')==='x'",
    );
    // String indices can be enumerated before an incomplete prototype is reached.
    assert_true("let key;for(key in 'abc')break;key==='0'");
}

#[test]
fn arrays_include_only_present_indices_and_enumerable_custom_properties() {
    assert_true(
        "let obj=[,2,3];obj.x=4;let keys=[];for(let k in obj)keys.push(k);keys.join(',')==='1,2,x'",
    );
    assert_true(
        r#"
            let obj = [1, 2, 3],
                keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === '0') obj.length = 1;
            }
            keys.join(',') === '0'
        "#,
    );
}

#[test]
fn assignments_vars_and_lexical_bindings_have_the_required_scope() {
    assert_true("var let;for(let in {a:1}){};let==='a'");
    assert_true(
        r#"
            let obj = {
                    a: 1,
                    b: 2
                },
                target = {},
                calls = 0;

            function key() {
                calls++;
                return 'x';
            }
            for (target[key()] in obj) {};
            target.x === 'b' && calls === 2
        "#,
    );
    assert_true(
        "let before=x;for(var x in {a:1,b:2}){};before===undefined && x==='b' && globalThis.x==='b'",
    );
    assert_true(
        r#"
            let x = {
                    a: 1
                },
                caught = false;
            try {
                for (let x in x) {};
            } catch (e) {
                caught = e instanceof ReferenceError;
            }
            caught && x.a === 1
        "#,
    );
    assert_true(
        r#"
            let x = 'outer',
                closures = [];
            for (let x in {
                    a: 1,
                    b: 2
                }) {
                closures.push(() => x);
                x += '!';
            }
            x === 'outer' && closures[0]() === 'a!' && closures[1]() === 'b!'
        "#,
    );
    assert_true(
        r#"
            let closures = [];
            for (const x in {
                    a: 1,
                    b: 2
                }) closures.push(() => x);
            closures[0]() === 'a' && closures[1]() === 'b' && typeof x === 'undefined'
        "#,
    );
    assert_true(
        r#"
            let rhs, caught = false;
            for (let x in (rhs = () => x, {})) {};
            try {
                rhs();
            } catch (e) {
                caught = e instanceof ReferenceError;
            }
            caught
        "#,
    );
}

#[test]
fn control_transfers_and_body_throws_do_not_call_user_iterator_hooks() {
    assert_true(
        r#"
            let obj = {
                a: 1,
                b: 2,
                get [Symbol.iterator]() {
                    throw 7;
                },
                return () {
                    throw 8;
                }
            };
            let keys = [];
            for (let k in obj) {
                keys.push(k);
                if (k === 'a') continue;
                break;
            }
            keys.join(',') === 'a,b'
        "#,
    );
    assert_true("let obj={a:1,b:2};function f(){for(let k in obj)return k;}f()==='a'");
    assert_true(
        r#"
            let marker = {},
                caught = false,
                x = 'outer';
            try {
                for (let x in {
                        a: 1
                    }) throw marker;
            } catch (e) {
                caught = e === marker;
            }
            caught && x === 'outer'
        "#,
    );
    assert_true(
        r#"
            let keys = [];
            outer: for (let x in {
                    a: 1,
                    b: 2
                }) {
                for (let y in {
                        c: 3
                    }) {
                    keys.push(x + y);
                    continue outer;
                }
            }
            keys.join(',') === 'ac,bc'
        "#,
    );
    assert_true(
        r#"
            let obj = {
                    a: 1
                },
                target = {
                    set x(v) {
                        throw 7;
                    }
                },
                caught = false;
            try {
                for (target.x in obj) throw 8;
            } catch (e) {
                caught = e === 7;
            }
            caught
        "#,
    );
}

#[test]
fn completion_values_and_captured_bindings_survive_collection() {
    for (source, value) in [
        ("for(let x in null) {7;}", Value::Undefined),
        ("for(let x in {}) {7;}", Value::Undefined),
        ("for(let x in {a:1,b:2}) {x;}", Value::String("b".into())),
        ("a:for(let x in {a:1,b:2}) {7;break a;}", Value::Number(7.0)),
    ] {
        assert_eq!(Realm::default().eval(source), Ok(value), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval("let closures=[];for(const key in {a:1,b:2})closures.push(()=>key)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("closures[0]()+closures[1]()"),
        Ok(Value::String("ab".into()))
    );
}

#[test]
fn opted_in_work_and_incomplete_intrinsics_are_host_failures() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(5000),
        ..Limits::default()
    });
    realm.eval("let flag=0,source={}").unwrap();
    for i in 0..100 {
        realm.eval(&format!("source[{i}]={i}")).unwrap();
    }
    assert!(matches!(
        realm.eval("try{for(let k in source){};}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("gap");
    realm
        .eval("let flag=0,keys=[],o=Object.create(new Proxy(Array,{ownKeys(){gap;}}));o[0]='a';o[1]='b';")
        .unwrap();
    assert!(matches!(
        realm.eval("try{for(let k in o)keys.push(k);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && keys.join(',')==='0,1'"),
        Ok(Value::Boolean(true))
    );
}
