//! Public field evaluation and construction ordering (15.7.10/14, 7.3.32/33).

use spite_runtime::{Error, ExceptionKind, PromiseJobOutcome, Realm, Value};

use crate::common::{assert_true, assert_value, eval_string};

#[test]
fn instance_fields_define_properties_without_calling_inherited_setters() {
    let mut realm = Realm::default();
    let report = eval_string(
        &mut realm,
        r#"
            let setterCalls = 0;
            class Base {
                set x(value) {
                    setterCalls++;
                }
            }
            class Derived extends Base {
                x = 1;
                y;
                __proto__ = 7;
                x = 2;
            }

            const instance = new Derived();
            JSON.stringify({
                setterCalls,
                x: instance.x,
                hasY: Object.hasOwn(instance, "y"),
                yIsUndefined: instance.y === undefined,
                prototypeField: instance.__proto__,
                inheritsPrototype: Object.getPrototypeOf(instance) === Derived.prototype,
                descriptor: Object.getOwnPropertyDescriptor(instance, "x"),
            }, null, 2)
        "#,
    );

    insta::assert_snapshot!(report, @r#"
    {
      "setterCalls": 0,
      "x": 2,
      "hasY": true,
      "yIsUndefined": true,
      "prototypeField": 7,
      "inheritsPrototype": true,
      "descriptor": {
        "value": 2,
        "writable": true,
        "enumerable": true,
        "configurable": true
      }
    }
    "#);
}

#[test]
fn static_fields_overwrite_methods_with_enumerable_data_properties() {
    let mut realm = Realm::default();
    let descriptor = eval_string(
        &mut realm,
        r#"
            class Fields {
                static x = 1;
                static x() { return 2; }
                static x = 3;
            }
            JSON.stringify(Object.getOwnPropertyDescriptor(Fields, "x"), null, 2)
        "#,
    );

    insta::assert_snapshot!(descriptor, @r#"
    {
      "value": 3,
      "writable": true,
      "enumerable": true,
      "configurable": true
    }
    "#);
}

#[test]
fn field_keys_use_ordinary_property_order() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                class Fields {
                    2 = 2;
                    1 = 1;
                    z = 3;
                    ["constructor"] = 4;
                }
            "#,
        )
        .unwrap();

    assert_value(
        &mut realm,
        "Object.keys(new Fields()).join(',')",
        Value::String("1,2,z,constructor".into()),
    );
}

#[test]
fn computed_names_finish_before_static_values_and_instance_initialization() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                const events = [];
                let computedNames = 0;
                class Fields {
                    [(events.push("instance name"), ++computedNames)] =
                        (events.push("instance value"), 1);
                    static [(events.push("static name"), ++computedNames)] =
                        (events.push("static value"), 2);
                    [(events.push("method name"), "method")]() {}
                    static self = (events.push("class reference"), Fields);
                }
                const first = new Fields();
                const second = new Fields();
            "#,
        )
        .unwrap();

    let events = eval_string(&mut realm, "JSON.stringify(events, null, 2)");
    insta::assert_snapshot!(events, @r#"
    [
      "instance name",
      "static name",
      "method name",
      "static value",
      "class reference",
      "instance value",
      "instance value"
    ]
    "#);
    assert_value(&mut realm, "computedNames", Value::Number(2.0));
    assert_value(&mut realm, "Fields[2]", Value::Number(2.0));
    assert_value(&mut realm, "Fields.self === Fields", Value::Boolean(true));
    assert_value(&mut realm, "first[1]", Value::Number(1.0));
    assert_value(&mut realm, "second[1]", Value::Number(1.0));
}

#[test]
fn computed_names_cannot_read_the_uninitialized_class_binding() {
    let result = Realm::default().eval("class Fields { [Fields] = 1; }");
    assert!(
        matches!(
            &result,
            Err(Error::Exception {
                kind: ExceptionKind::ReferenceError,
                ..
            })
        ),
        "{result:?}"
    );
}

#[test]
fn static_fields_see_the_inferred_anonymous_class_name() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                const Fields = class {
                    static observedName = this.name;
                };
            "#,
        )
        .unwrap();
    assert_value(
        &mut realm,
        "Fields.observedName",
        Value::String("Fields".into()),
    );
}

#[test]
fn static_fields_and_methods_see_the_initialized_class_binding() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                class Fields {
                    static observedName = Fields.name;
                    static identity() { return Fields; }
                }
            "#,
        )
        .unwrap();
    assert_value(
        &mut realm,
        "Fields.observedName",
        Value::String("Fields".into()),
    );
    assert_value(
        &mut realm,
        "Fields.identity() === Fields",
        Value::Boolean(true),
    );
}

#[test]
fn a_line_break_after_get_starts_a_field_without_repeating_computed_names() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
        let events = "";
        class Accessors {
            get [(events += "A", "x")]() { return this.value; }
            set [(events += "B", "x")](value) { this.value = value; }

            // This line break makes `get` a field, rather than an accessor prefix.
            get
            [(events += "C", "field")] = 7;
            static [(events += "S", "ready")] = true;
        }
        const instance = new Accessors();
        instance.x = 8;
    "#,
        )
        .unwrap();

    assert_value(&mut realm, "events", Value::String("ABCS".into()));
    assert_value(&mut realm, "instance.x", Value::Number(8.0));
    assert_value(&mut realm, "instance.field", Value::Number(7.0));
    assert_value(
        &mut realm,
        "Object.hasOwn(instance, 'get')",
        Value::Boolean(true),
    );
    assert_value(&mut realm, "instance.get", Value::Undefined);
    assert_value(&mut realm, "Accessors.ready", Value::Boolean(true));
}

#[test]
fn computed_accessor_names_capture_enclosing_private_fields() {
    assert_true(
        r#"
            class Outer {
                #key = 7;
                makeClass() {
                    return class {
                        get [(() => this.#key)()]() { return 8; }
                    };
                }
            }
            const Inner = new Outer().makeClass();
            new Inner()[7] === 8
        "#,
    );
}

#[test]
fn computed_accessor_names_read_the_enclosing_arguments() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                function makeClass(key) {
                    return class {
                        get [arguments[0]]() { return 7; }
                        set [arguments[0]](value) { this.value = value; }
                    };
                }
                const Accessors = makeClass("x");
                const instance = new Accessors();
                instance.x = 8;
            "#,
        )
        .unwrap();

    assert_value(&mut realm, "instance.x", Value::Number(7.0));
    assert_value(&mut realm, "instance.value", Value::Number(8.0));
    assert_value(
        &mut realm,
        "Object.getOwnPropertyDescriptor(Accessors.prototype, 'x').get.name",
        Value::String("get x".into()),
    );
}

#[test]
fn base_fields_precede_parameter_defaults_and_capture_definition_scope() {
    assert_true(
        r#"
            let log = '',
                x = 7;
            class C {
                v = (log += 'F', x);
                constructor(x = (log += 'P', this.v)) {
                    log += 'B';
                    this.param = x;
                }
            }
            let c = new C;
            log === 'FPB' && c.v === 7 && c.param === 7
        "#,
    );
    assert_true(
        r#"
            let x = 7;
            class C {
                value = () => x;
                constructor(x = 9) {
                    this.parameter = () => x;
                }
            }
            let c = new C;
            c.value() === 7 && c.parameter() === 9
        "#,
    );
}

#[test]
fn derived_fields_follow_bind_this_and_precede_the_rest_of_the_body() {
    assert_true(
        r#"
            let log = '';
            class B {
                x = (log += 'b', 1);
                constructor() {
                    log += 'B';
                }
            }
            class D extends B {
                x = (log += 'd', 2);
                constructor() {
                    log += 'A';
                    super();
                    log += 'D';
                }
            }
            let d = new D;
            log === 'AbBdD' && d.x === 2
        "#,
    );
    assert_true(
        r#"
            let fields = 0,
                constructs = 0;
            class B {
                constructor() {
                    constructs++;
                }
            }
            class D extends B {
                x = ++fields;
                constructor() {
                    super();
                    try {
                        super();
                    } catch (e) {
                        if (!(e instanceof ReferenceError)) throw e;
                    }
                }
            }
            let d = new D;
            fields === 1 && constructs === 2 && d.x === 1
        "#,
    );
    assert_true(
        r#"
            let log = '';
            class D extends Object {
                a = (log += 'A', 1);
                b = (function() {
                    throw 7;
                })();
                c = (log += 'C', 3);
                constructor() {
                    try {
                        super();
                    } catch (e) {
                        if (e !== 7) throw e;
                        this.saved = this.a;
                    }
                }
            }
            let d = new D;
            log === 'A' && d.a === 1 && d.saved === 1 && !Object.hasOwn(d, 'b') && !Object.hasOwn(d, 'c')
        "#,
    );
}

#[test]
fn constructor_object_returns_observe_the_correct_initialization_receiver() {
    assert_true(
        r#"
            let receiver, o = {};
            class B {
                x = 1;
                constructor() {
                    receiver = this;
                    return o;
                }
            }
            new B === o && receiver.x === 1 && !Object.hasOwn(o, 'x')
        "#,
    );
    assert_true(
        r#"
            let n = 0,
                o = {};
            class D extends Object {
                x = ++n;
                constructor() {
                    return o;
                }
            }
            new D === o && n === 0 && !Object.hasOwn(o, 'x')
        "#,
    );
    assert_true(
        "let o={};class B{constructor(){return o;}}class D extends B{x=7;}new D===o && o.x===7",
    );
}

#[test]
fn default_forwarding_initializes_fields_from_base_to_derived_without_iteration() {
    assert_true(
        r#"
            let log = '';
            class A {
                x = (log += 'A', 1);
            }
            class B extends A {
                x = (log += 'B', 2);
            }
            class C extends B {
                x = (log += 'C', 3);
            }
            Array.prototype[Symbol.iterator] = function() {
                throw 7;
            };
            let c = new C;
            log === 'ABC' && c.x === 3
        "#,
    );
    assert_true(
        "let n=0,C=class{x=++n;};for(let i=0;i<100;i++){C=class extends C{x=++n;};}new C().x===101 && n===101",
    );
    assert_true(
        r#"
            class B {
                x = 7;
            }
            class D extends B {
                y = 8;
            }
            let F = D.bind(null);

            function N() {}
            let d = Reflect.construct(F, [], N);
            d.x === 7 && d.y === 8 && Object.getPrototypeOf(d) === N.prototype
        "#,
    );
}

#[test]
fn initializer_this_super_and_new_target_have_their_own_function_context() {
    assert_true(
        r#"
            class B {
                get x() {
                    return this.value + 1;
                }
                static x = 8;
            }
            class D extends B {
                value = 6;
                y = super.x;
                target = new.target;
                arrow = () => this;
                static y = super.x;
                static target = new.target;
                static arrow = () => this;
            }
            let d = new D;
            d.y === 7 && d.target === undefined && d.arrow() === d && D.y === 8 && D.target ===
                undefined && D.arrow() === D
        "#,
    );
    assert_true(
        r#"
            class B {
                get x() {
                    return this.v;
                }
            }
            class D extends B {
                v = 7;
                x = eval('super.x');
                t = eval('new.target');
                static t = eval('new.target');
            }
            let d = new D;
            d.x === 7 && d.t === undefined && D.t === undefined
        "#,
    );
    assert_true("function f(k){return class {[arguments[0]]=1;};}Object.hasOwn(new (f('x')),'x')");
    // Ordinary initializer semantics apply even to the first static field.
    // The published grammar issue and V8's ordering differences are recorded
    // under Public class fields in docs/design.md.
    assert_true("var await=7;class C{static x=await;}C.x===7");
    assert_true(
        r#"
            var await = () => 7, C;
            async function f() {
                C = class {
                    static #x = await ();
                    static value() {
                        return this.#x;
                    }
                };
            }
            f();
            C.value() === 7
        "#,
    );
    assert_true(
        "var await=7;class Outer{static{class C{static x=await;}this.value=C.x;}}Outer.value===7",
    );
    assert_true(
        r#"
            var await = 7;

            function* f() {
                class C {
                    x = await;
                    static x = await;
                    [yield 'key'] = 8;
                }
                yield 9;
                return C;
            }
            let g = f(),
                a = g.next(),
                b = g.next('chosen'),
                C = g.next().value;
            a.value === 'key' && b.value === 9 && C.x === 7 && new C().x === 7 && new C().chosen === 8
        "#,
    );
    for prefix in ["", "'use strict';"] {
        let mut realm = Realm::default();
        realm.eval(&format!(
            "{prefix}var await=7;let completed=0,failed;async function build(){{class C{{x=await;#x=await;value(){{return this.#x;}}static x=await;[await Promise.resolve('key')]=8;arrow=()=>await;future=async()=>await 9;}}await 0;return C;}}build().then(C=>{{let c=new C;if(c.x===7 && c.value()===7 && C.x===7 && c.key===8 && c.arrow()===7)completed++;return c.future();}}).then(v=>{{if(v===9)completed++;}},e=>{{failed=e;}});"
        )).unwrap();
        loop {
            match realm.run_next_promise_job().unwrap() {
                PromiseJobOutcome::NoJob => break,
                PromiseJobOutcome::Completed => {}
                PromiseJobOutcome::Reported(error) => panic!("{error}"),
            }
        }
        assert_eq!(
            realm.eval("completed===2 && failed===undefined"),
            Ok(Value::Boolean(true))
        );
    }
    for source in [
        "async function f(){class C{x=await 1;}}",
        "async function f(){class C{static x=await 1;}return C.x;}",
        "async function f(){class C{#x=await 1;}}",
        "function* f(){class C{x=yield 1;}}",
    ] {
        assert_true(&format!(
            r#"
                let effects = 0,
                    caught = false;
                try {{
                    eval('effects=1;{source}');
                }} catch (e) {{
                    caught = e instanceof SyntaxError;
                }}
                caught && effects === 0
            "#
        ));
    }
}

#[test]
fn direct_eval_rejects_arguments_across_arrows_but_respects_ordinary_boundaries() {
    for source in [
        "arguments",
        "()=>arguments",
        "({arguments})",
        "({[arguments](){}})",
    ] {
        assert_true(&format!(
            r#"
                let effects = 0;
                class C {{
                    x = (() => {{
                        try {{
                            eval("effects=1;{source}");
                        }} catch (e) {{
                            return e instanceof SyntaxError;
                        }}
                    }})();
                }}
                new C().x && effects === 0
            "#
        ));
    }
    assert_true(
        r#"
            class C {
                x = function() {
                    return arguments[0];
                };
                y = ({
                    m() {
                        return arguments[0];
                    }
                });
                z = eval('(function(){return arguments.length;})()');
            }
            let c = new C;
            c.x(7) === 7 && c.y.m(8) === 8 && c.z === 0
        "#,
    );
    assert_true(
        r#"
            class C {
                x = () => eval('arguments');
            }
            let f = new C().x,
                caught = false;
            try {
                f();
            } catch (e) {
                caught = e instanceof SyntaxError;
            }
            caught
        "#,
    );
    assert_true(
        r#"
            class C {
                x = (() => {
                    let caught = false;
                    try {
                        eval('super()');
                    } catch (e) {
                        caught = e instanceof SyntaxError;
                    }
                    return caught;
                })();
            }
            new C().x
        "#,
    );
}

#[test]
fn initializer_named_evaluation_uses_exact_string_and_symbol_names() {
    assert_true(
        r#"
            let s = Symbol('key');
            class C {
                f = function() {};
                a = () => {};
                c = class {
                    static nameAtCreation = this.name;
                };
                [s] = () => {};
                static f = function() {};
            }
            let c = new C;
            c.f.name === 'f' && c.a.name === 'a' && c.c.name === 'c' && c.c.nameAtCreation === 'c' && c[s]
                .name === '[key]' && C.f.name === 'f'
        "#,
    );
    assert_true("let key=String.fromCharCode(55296);class C{[key]=()=>{};}new C()[key].name===key");
}

#[test]
fn field_definition_errors_follow_initializer_effects_and_stop_later_fields() {
    assert_true(
        r#"
            let log = '',
                o = Object.preventExtensions({});
            class B {
                constructor() {
                    return o;
                }
            }
            class D extends B {
                x = (log += 'I', 1);
                y = (log += 'Y', 2);
            }
            let caught = false;
            try {
                new D;
            } catch (e) {
                caught = e instanceof TypeError;
            }
            caught && log === 'I'
        "#,
    );
    assert_true(
        r#"
            let log = '',
                o = {};
            Object.defineProperty(o, 'x', {
                value: 0
            });
            class B {
                constructor() {
                    return o;
                }
            }
            class D extends B {
                x = (log += 'I', 1);
            }
            let caught = false;
            try {
                new D;
            } catch (e) {
                caught = e instanceof TypeError;
            }
            caught && log === 'I' && o.x === 0
        "#,
    );
    assert_true(
        r#"
            let log = '',
                saved, caught = false;
            try {
                class C {
                    static[(saved = () => C, 'prototype')] = (log += 'I', 1);
                    static y = (log += 'Y', 2);
                }
            } catch (e) {
                caught = e instanceof TypeError;
            }
            caught && log === 'I' && typeof saved() === 'function'
        "#,
    );
    assert_true(
        "class C extends Array{length=2;}let caught=false;try{new C;}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn contextual_names_and_automatic_semicolons_remain_fields_or_methods() {
    assert_true(
        "class C{get\nx;set\ny;async\nm(){return 7;}static\nz=8;}let c=new C;Object.keys(c).join(',')==='get,x,set,y,async' && c.m()===7 && C.z===8",
    );
    assert_true(
        "class C{get\nx(){return 7;}set\nx(v){this.v=v;}}let c=new C;c.x=8;c.x===7 && c.v===8",
    );
}

#[test]
fn field_initializer_captures_and_escaped_arrows_survive_collection() {
    let mut realm = Realm::default();
    realm
        .eval(
            r#"
                let C = (function() {
                        let value = {
                            n: 7
                        };
                        return class {
                            x = value;
                            f = () => this.x.n;
                        };
                    })(),
                    c = new C,
                    f = c.f;
                C = undefined;
                c = undefined;
            "#,
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f()"), Ok(Value::Number(7.0)));
    realm.eval("f=undefined").unwrap();
    realm.collect(usize::MAX).unwrap();
    realm
        .eval("let D=(function(){let value={n:8};return class{x=value;};})();")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("new D().x.n"), Ok(Value::Number(8.0)));
}

#[test]
fn language_and_host_failures_restore_the_callers_context() {
    assert_true(
        r#"
            let x = 7,
                caught = false;
            try {
                class C {
                    static x = (() => {
                        throw 9;
                    })();
                }
            } catch (e) {
                caught = e === 9;
            }
            after = 8;
            caught && x === 7 && after === 8
        "#,
    );
    assert_true(
        "let flag=0;class C{x=eval('function* gap(){}');}try{new C}catch{flag=1}finally{flag=2}flag===2",
    );
    let mut realm = Realm::default();
    realm.reserve_unsupported_global("unsupportedHost");
    realm
        .eval("let flag=0;class C{x=eval('unsupportedHost');}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{new C;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("after=8"), Ok(Value::Number(8.0)));
}
