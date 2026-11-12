//! Reflect apply/construct, array-like argument lists, and newTarget forwarding.

use crate::common;
use common::REALM_ENTRIES;
use spite_runtime::{Error, Limits, Realm, Value};

use crate::common::assert_true;

#[test]
fn apply_calls_targets_with_exact_receiver_sparse_arguments_and_results() {
    assert_true(
        r#"
            let receiver = {},
                marker = {};

            function f(a, b, c) {
                'use strict';
                return this === receiver && arguments.length === 3 && a === 1 && b === undefined && c ===
                    marker;
            }
            Reflect.apply(f, receiver, [1, , marker])
        "#,
    );
    assert_true(
        r#"
            function f() {
                'use strict';
                return this;
            }
            Reflect.apply(f, 3, []) === 3 && Reflect.apply(f, null, []) === null && Reflect.apply(f,
                undefined, []) === undefined
        "#,
    );
    assert_true(
        r#"
            function f() {
                return this;
            }
            Reflect.apply(f, null, []) === globalThis && Reflect.apply(f, 2, []).valueOf() === 2
        "#,
    );
    assert_true(
        r#"
            let receiver = {};

            function f(a, b) {
                return this === receiver && a === 1 && b === 2;
            }
            Reflect.apply(f.bind(receiver, 1), null, [2])
        "#,
    );
    assert_true(
        r#"
            let marker = {};
            Reflect.apply(() => marker, undefined, {}) === marker && Reflect.apply(Array.of, null, [1, 2])
                .join(',') === '1,2'
        "#,
    );
    assert_true(
        r#"
            let f = () => 7;
            Object.defineProperty(f, 'call', {
                get() {
                    throw 8;
                }
            });
            Object.defineProperty(f, 'apply', {
                get() {
                    throw 9;
                }
            });
            Reflect.apply(f, null, []) === 7
        "#,
    );
    assert_true(
        "let caught=false;try{Reflect.apply(()=>{throw 7;},undefined,[]);}catch(e){caught=e===7;}caught",
    );
}

#[test]
fn apply_validates_target_before_argument_list_and_requires_object_lists() {
    for target in [
        "undefined",
        "null",
        "1",
        "true",
        "'s'",
        "1n",
        "Symbol()",
        "{}",
    ] {
        assert_true(&format!(
            r#"
                let reads = 0,
                    caught = false;
                try {{
                    Reflect.apply({target}, null, {{
                        get length() {{
                            reads++;
                            throw 7;
                        }}
                    }});
                }} catch (e) {{
                    caught = e instanceof TypeError;
                }}
                caught && reads === 0
            "#
        ));
    }
    for list in ["undefined", "null", "1", "true", "'ab'", "1n", "Symbol()"] {
        assert_true(&format!(
            r#"
                let calls = 0,
                    caught = false;
                try {{
                    Reflect.apply(() => {{
                        calls++;
                    }}, null, {list});
                }} catch (e) {{
                    caught = e instanceof TypeError;
                }}
                caught && calls === 0
            "#
        ));
    }
    assert_true(
        "let caught=false;try{Reflect.apply(()=>7,null);}catch(e){caught=e instanceof TypeError;}caught",
    );
    assert_true("Reflect.apply((...args)=>args.join(','),null,Object('ab'))==='a,b'");
}

#[test]
fn argument_lists_read_length_once_and_live_indices_in_order_without_iteration() {
    assert_true(
        r#"
            let log = '',
                list = {
                    get length() {
                        log += 'l';
                        return {
                            valueOf() {
                                log += 'v';
                                return 3.9;
                            }
                        };
                    },
                    get 0() {
                        log += 'a';
                        this[1] = 2;
                        return 1;
                    },
                    2: 3,
                    get [Symbol.iterator]() {
                        throw 7;
                    }
                };
            Reflect.apply((...args) => args.join(','), null, list) === '1,2,3' && log === 'lva'
        "#,
    );
    assert_true(
        r#"
            let log = '',
                list = {
                    length: 2,
                    get 0() {
                        log += 'a';
                        delete this[1];
                        return 1;
                    },
                    1: 2
                };
            Reflect.apply((a, b) => a === 1 && b === undefined, null, list) && log === 'a'
        "#,
    );
    assert_true(
        r#"
            let reads = 0,
                calls = 0,
                caught = false,
                list = {
                    length: 9007199254740991,
                    get 0() {
                        reads++;
                        throw 7;
                    }
                };
            try {
                Reflect.apply(() => {
                    calls++;
                }, null, list);
            } catch (e) {
                caught = e === 7;
            }
            caught && reads === 1 && calls === 0
        "#,
    );
    assert_true(
        r#"
            let calls = 0,
                caught = false;
            try {
                Reflect.apply(() => {
                    calls++;
                }, null, {
                    length: Symbol()
                });
            } catch (e) {
                caught = e instanceof TypeError;
            }
            caught && calls === 0
        "#,
    );
    // 7.2.3, 20.2.3.1 and 28.1.1: revocation preserves cached callability.
    // Gather the list before [[Call]], allowing a later getter throw to win.
    for target in ["function(){}", "class{}"] {
        for throws in [false, true] {
            for invoke in [
                "Reflect.apply(target,null,list)",
                "Function.prototype.apply.call(target,null,list)",
            ] {
                let source = r#"
                    let log='',calls=0,marker={},caught;
                    let pair=Proxy.revocable(TARGET,{apply(){calls++;return 7}});
                    let target=pair.proxy,list={
                        get length(){log+='l';return 2},
                        get 0(){log+='0';AT_ZERO return 1},
                        get 1(){log+='1';THROW return 2}
                    };
                    BEFORE
                    try{INVOKE}catch(e){caught=EXPECTED}
                    caught&&log==='l01'&&calls===0
                "#
                .replace("TARGET", target)
                .replace("THROW", if throws { "throw marker;" } else { "" })
                .replace("BEFORE", if throws { "" } else { "pair.revoke();" })
                .replace("AT_ZERO", if throws { "pair.revoke();" } else { "" })
                .replace("INVOKE", invoke)
                .replace(
                    "EXPECTED",
                    if throws {
                        "e===marker"
                    } else {
                        "e instanceof TypeError"
                    },
                );
                assert_true(&source);
            }
        }
    }
}

#[test]
fn construct_validates_target_and_present_newtarget_before_reading_list() {
    for target in [
        "undefined",
        "null",
        "1",
        "{}",
        "()=>7",
        "({m(){}}).m",
        "Array.of",
        "Reflect.construct",
    ] {
        assert_true(&format!(
            r#"
                let reads = 0,
                    caught = false;
                try {{
                    Reflect.construct({target}, {{
                        get length() {{
                            reads++;
                            throw 7;
                        }}
                    }}, {{}});
                }} catch (e) {{
                    caught = e instanceof TypeError;
                }}
                caught && reads === 0
            "#
        ));
    }
    for target in ["undefined", "null", "1", "{}", "()=>7", "Array.of"] {
        assert_true(&format!(
            r#"
                let reads = 0,
                    caught = false;
                try {{
                    Reflect.construct(function() {{}}, {{
                        get length() {{
                            reads++;
                            throw 7;
                        }}
                    }}, {target});
                }} catch (e) {{
                    caught = e instanceof TypeError;
                }}
                caught && reads === 0
            "#
        ));
    }
    assert_true(
        r#"
            let calls = 0,
                caught = false;
            try {
                Reflect.construct(function() {
                    calls++;
                }, null);
            } catch (e) {
                caught = e instanceof TypeError;
            }
            caught && calls === 0
        "#,
    );
    assert_true(
        r#"
            let log = '',
                caught = false;
            try {
                Reflect.construct(function() {
                    log += 'c';
                }, {
                    length: 2,
                    get 0() {
                        log += 'a';
                        throw 7;
                    },
                    get 1() {
                        log += 'b';
                        throw 8;
                    }
                });
            } catch (e) {
                caught = e === 7;
            }
            caught && log === 'a'
        "#,
    );
    // 7.2.4, 28.1.2 and 10.5.13: revocation preserves [[Construct]]. A
    // construct trap supplying its result never reads newTarget.prototype.
    for target in ["function(){throw 9}", "class{constructor(){throw 9}}"] {
        for revoke_before in [false, true] {
            let source = r#"
                let log='',calls=0,marker={},pair=Proxy.revocable(function(){},{get(){throw 7}});
                let target=new Proxy(TARGET,{construct(t,args,n){
                    log+='t';calls++;
                    if(n!==pair.proxy||args.join(',')!=='1,2')throw 8;
                    return marker;
                }});
                let list={get length(){log+='l';return 2},
                          get 0(){log+='0';AT_ZERO return 1},
                          get 1(){log+='1';return 2}};
                BEFORE
                Reflect.construct(target,list,pair.proxy)===marker&&calls===1&&log==='l01t'
            "#
            .replace("TARGET", target)
            .replace("BEFORE", if revoke_before { "pair.revoke();" } else { "" })
            .replace("AT_ZERO", if revoke_before { "" } else { "pair.revoke();" });
            assert_true(&source);
        }
    }
}

#[test]
fn construct_preserves_newtarget_prototypes_and_constructor_return_rules() {
    assert_true(
        r#"
            function F(a) {
                this.a = a;
                this.target = new.target;
                return 3;
            }

            function N() {
                throw 7;
            }
            let x = Reflect.construct(F, [2], N);
            x.a === 2 && x.target === N && Object.getPrototypeOf(x) === N.prototype && !(x instanceof F) &&
                x instanceof N
        "#,
    );
    assert_true(
        "function F(){this.target=new.target;}let x=Reflect.construct(F,[]);x.target===F && x instanceof F",
    );
    assert_true("let marker={};function F(){return marker;}Reflect.construct(F,[])===marker");
    assert_true(
        r#"
            let log = '',
                N = (function() {}).bind(null),
                proto = {};
            Object.defineProperty(N, 'prototype', {
                get() {
                    log += 'p';
                    return proto;
                }
            });

            function F(a) {
                log += 'c';
                this.a = a;
            }
            let x = Reflect.construct(F, {
                get length() {
                    log += 'l';
                    return 1;
                },
                get 0() {
                    log += 'a';
                    return 2;
                }
            }, N);
            x.a === 2 && Object.getPrototypeOf(x) === proto && log === 'lapc'
        "#,
    );
    assert_true(
        r#"
            let calls = 0,
                caught = false,
                N = (function() {}).bind(null);
            Object.defineProperty(N, 'prototype', {
                get() {
                    throw 7;
                }
            });
            try {
                Reflect.construct(function() {
                    calls++;
                }, [], N);
            } catch (e) {
                caught = e === 7;
            }
            caught && calls === 0
        "#,
    );
    assert_true(
        r#"
            function F() {}
            let N = (function() {}).bind(null);
            N.prototype = 3;
            Object.getPrototypeOf(Reflect.construct(F, [], N)) === Object.prototype
        "#,
    );
}

#[test]
fn bound_construct_forwarding_substitutes_newtarget_only_when_it_is_the_bound_target() {
    assert_true(
        r#"
            let receiver = {};

            function F(a, b, c) {
                this.args = [a, b, c];
                this.target = new.target;
            }
            let B = F.bind(receiver, 1),
                C = B.bind(null, 2),
                x = Reflect.construct(C, [3]);
            x.args.join(',') === '1,2,3' && x.target === F && x instanceof F && receiver.args === undefined
        "#,
    );
    assert_true(
        r#"
            function F() {
                this.target = new.target;
            }

            function N() {}
            let B = F.bind(null),
                x = Reflect.construct(B, [], N);
            x.target === N && Object.getPrototypeOf(x) === N.prototype
        "#,
    );
    assert_true(
        r#"
            function F() {
                this.target = new.target;
            }
            let B = F.bind(null),
                x = Reflect.construct(F, [], B);
            x.target === B && Object.getPrototypeOf(x) === Object.prototype
        "#,
    );
}

#[test]
fn builtin_construction_uses_custom_newtarget_and_intrinsic_fallbacks() {
    assert_true(
        r#"
            function N() {}
            let a = Reflect.construct(Array, [1, 2], N),
                s = Reflect.construct(String, ['abc'], N),
                b = Reflect.construct(Boolean, [1], N),
                n = Reflect.construct(Number, [3], N),
                e = Reflect.construct(TypeError, ['message'], N);
            Array.isArray(a) && a.length === 2 && a[1] === 2 && Object.getPrototypeOf(a) === N.prototype &&
                String.prototype.valueOf.call(s) === 'abc' && Boolean.prototype.valueOf.call(b) === true &&
                Number.prototype.valueOf.call(n) === 3 && e.message === 'message' && Object.getPrototypeOf(
                    s) === N.prototype && Object.getPrototypeOf(b) === N.prototype && Object
                .getPrototypeOf(n) === N.prototype && Object.getPrototypeOf(e) === N.prototype
        "#,
    );
    assert_true(
        r#"
            let N = (function() {}).bind(null);
            N.prototype = null;
            let a = Reflect.construct(Array, [], N),
                s = Reflect.construct(String, [], N),
                b = Reflect.construct(Boolean, [], N),
                n = Reflect.construct(Number, [], N);
            Object.getPrototypeOf(a) === Array.prototype && Object.getPrototypeOf(s) === String
                .prototype && Object.getPrototypeOf(b) === Boolean.prototype && Object.getPrototypeOf(
                    n) === Number.prototype
        "#,
    );
    assert_true(
        r#"
            function N() {}
            let marker = {},
                o = Reflect.construct(Object, [marker], N);
            o !== marker && Object.getPrototypeOf(o) === N.prototype && Reflect.construct(Object, [
                marker]) === marker
        "#,
    );
}

#[test]
fn reflect_metadata_collection_and_large_argument_lists_use_default_limits() {
    assert_true(
        r#"
            let d = Object.getOwnPropertyDescriptor(globalThis, 'Reflect'),
                t = Object.getOwnPropertyDescriptor(Reflect, Symbol.toStringTag);
            typeof Reflect === 'object' && d.value === Reflect && d.writable && !d.enumerable && d
                .configurable && Object.getPrototypeOf(Reflect) === Object.prototype && Reflect
            .toString() === '[object Reflect]' && t.value === 'Reflect' && !t.writable && !t.enumerable &&
                t.configurable
        "#,
    );
    for (method, length) in [("apply", 3), ("construct", 2)] {
        assert_true(&format!(
            "let d=Object.getOwnPropertyDescriptor(Reflect,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value==={length} && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        assert_true(&format!(
            "let caught=false;try{{new Reflect.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    assert_true(
        r#"
            let list = {
                length: 5000
            };
            Reflect.apply((...args) => args.length, null, list) === 5000 && Reflect.construct(function() {
                this.length = arguments.length;
            }, list).length === 5000
        "#,
    );
    let mut realm = Realm::default();
    realm.eval("let apply=Reflect.apply,construct=Reflect.construct;delete Reflect.apply;delete Reflect.construct;delete globalThis.Reflect").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("apply((x)=>x+1,null,[2])===3 && construct(function(x){this.x=x;},[2]).x===2 && typeof Reflect==='undefined'"),Ok(Value::Boolean(true)));
    assert_true(
        "typeof Array.fromAsync==='function'&&Object.getOwnPropertyNames(Array).includes('fromAsync')",
    );
}

#[test]
fn opt_in_argument_limits_and_unsupported_targets_skip_pending_handlers() {
    let mut realm = Realm::new(Limits {
        max_arguments: Some(4),
        ..Limits::default()
    });
    realm
        .eval("let flag=0,reads=0,calls=0,list={length:5,get 0(){reads++;return 1;}}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{Reflect.apply(()=>{calls++;},null,list);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && reads===0 && calls===0"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(
            "try{Reflect.construct(function(){calls++;},list);}catch{flag=1;}finally{flag=2;}"
        ),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0 && reads===0 && calls===0"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("gap");
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{Reflect.apply(()=>{gap;},null,[]);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
