//! Strict calls, unmapped arguments, receiver preservation, and lexical captures.

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
fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(expected.into())),
        "{source}"
    );
}

#[test]
fn strict_functions_execute_declarations_closures_and_returns() {
    number("function f(a,b){'use strict';return a+b;} f(3,4)", 7.0);
    number("'use strict';f(7);function f(x){return x;}", 7.0);
    number(
        "'use strict';let f=function local(n){return n<2?1:n*local(n-1);};f(5)",
        120.0,
    );
    number(
        "'use strict';function make(x){let y=x;return function(){return ++y;};}let f=make(1);f()+f()",
        5.0,
    );
    number(
        "'use strict';function f(x){return g();function g(){return x+1;}}f(6)",
        7.0,
    );
    number(
        "'use strict';function f(){return g();function g(){return 1;}function g(){return 7;}}f()",
        7.0,
    );
    number(
        "'use strict';function f(x=3,get=()=>x){var x=9;return get()+x;}f()",
        12.0,
    );
    number(
        "'use strict';function f(){try{return 1;}finally{return 7;}}f()",
        7.0,
    );
    number(
        "'use strict';function f(){try{throw 7;}catch(e){return e;}}f()",
        7.0,
    );
    number(
        "function f(){'use strict';return 1;}f();undeclared=7;undeclared",
        7.0,
    );
    number(
        "'use strict';let f=function local(){try{local=1;}catch{return 7;}};f()",
        7.0,
    );
    assert_eq!(
        Realm::default().eval("function f(){'use strict';9;}f()"),
        Ok(Value::Undefined)
    );
}

#[test]
fn strict_this_preserves_primitives_nullish_values_and_explicit_receivers() {
    for (argument, value) in [
        ("undefined", Value::Undefined),
        ("null", Value::Null),
        ("7", Value::Number(7.0)),
        ("true", Value::Boolean(true)),
        ("'x'", Value::String("x".into())),
    ] {
        assert_eq!(
            Realm::default().eval(&format!(
                "function f(){{'use strict';return this;}}f.call({argument})"
            )),
            Ok(value)
        );
    }
    number(
        "function f(){'use strict';return this.x;}let o={x:7,f};o.f()",
        7.0,
    );
    number(
        "function f(){'use strict';return this;} f()===undefined?1:0",
        1.0,
    );
    number(
        "function f(){'use strict';return this;}let o={f};(0,o.f)()===undefined?1:0",
        1.0,
    );
    number(
        "function f(){'use strict';return this.x;}f.apply({x:7},{length:0})",
        7.0,
    );
    number(
        "function f(){'use strict';return this.x;}f.bind({x:7}).call({x:9})",
        7.0,
    );
    number(
        "function f(){'use strict';return ()=>this.x;}let g=f.call({x:7});g.call({x:9})",
        7.0,
    );
    number(
        "'use strict';function f(g=()=>this){return g;}let o={};f.call(o)()===o?1:0",
        1.0,
    );
    number("function f(){'use strict';return delete this;}f()?1:0", 1.0);
    number("'use strict';this===globalThis?1:0", 1.0);
}

#[test]
fn arguments_are_unmapped_and_include_all_original_arguments() {
    number(
        "function f(a){'use strict';a=9;return arguments[0];}f(7)",
        7.0,
    );
    number(
        "function f(a){'use strict';arguments[0]=9;return a;}f(7)",
        7.0,
    );
    number(
        "function f(a){'use strict';return arguments.length;}f(1,2,3)",
        3.0,
    );
    number(
        "function f(a){'use strict';return arguments.length;}f()",
        0.0,
    );
    number(
        "function f(a){'use strict';return arguments[1];}f(1,7)",
        7.0,
    );
    number(
        "'use strict';function f(a=arguments.length){return a;}f(undefined,1)",
        2.0,
    );
    number(
        "'use strict';function f(a=7){return arguments[0]===undefined?arguments.length:99;}f(undefined)",
        1.0,
    );
    number(
        "function f(){'use strict';return arguments;}let a=f(1,2);a.length=0; a[0]+a[1]",
        3.0,
    );
    number(
        "function f(){'use strict';delete arguments[0];return arguments.length;}f(1)",
        1.0,
    );
    number(
        "function f(){'use strict';return arguments;}function g(a,b){'use strict';return a+b;}g.apply(null,f(3,4))",
        7.0,
    );
    number(
        "function f(){'use strict';let g=()=>arguments[0];return g(9);}f(7)",
        7.0,
    );
    number(
        "function f(){'use strict';let g=function(){return arguments[0];};return g(9);}f(7)",
        9.0,
    );
    string(
        "function f(){'use strict';return arguments.toString();}f()",
        "[object Arguments]",
    );
    assert_eq!(
        Realm::default().eval("function f(){'use strict';return arguments.caller;}f()"),
        Ok(Value::Undefined)
    );
    for expression in [
        "arguments.callee",
        "arguments.callee=1",
        "delete arguments.callee",
    ] {
        assert!(matches!(
            Realm::default().eval(&format!("function f(){{'use strict';{expression};}}f()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn arguments_descriptors_and_property_order_follow_the_unmapped_algorithm() {
    let mut realm = Realm::default();
    let Value::Object(arguments) = realm
        .eval("function f(){'use strict';return arguments;}f(1,2)")
        .unwrap()
    else {
        panic!()
    };
    let record = realm.inspect_object(&arguments).unwrap();
    let keys: Vec<_> = record
        .own_keys()
        .iter()
        .map(|key| key.to_utf8().unwrap())
        .collect();
    assert_eq!(keys, ["0", "1", "length", "callee"]);
    for name in ["0", "1"] {
        let property = record
            .own_property(&name.into())
            .unwrap()
            .as_data()
            .unwrap();
        assert!(property.writable && property.enumerable && property.configurable);
    }
    let length = record
        .own_property(&"length".into())
        .unwrap()
        .as_data()
        .unwrap();
    assert!(length.writable && !length.enumerable && length.configurable);
    let spite_runtime::object::Property::Accessor(callee) =
        record.own_property(&"callee".into()).unwrap()
    else {
        panic!("accessor");
    };
    assert!(!callee.enumerable && !callee.configurable);
    assert_eq!(callee.get, callee.set);
    assert!(callee.get.is_some());
}

#[test]
fn captured_this_arguments_and_named_function_cycles_survive_collection() {
    let mut realm = Realm::default();
    realm.eval("let make=function local(x){'use strict';return ()=>this.value+arguments[0].value;};let saved=make.call({value:3},{value:4});make=null;").unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(realm.eval("saved()"), Ok(Value::Number(7.0)));
    realm.eval("saved=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, REALM_ENTRIES);
    realm.eval("let escaped;let f;{ 'unused';f=(()=>{'use strict';return function(a=(escaped=()=>this),b=missing){};})();}try{f.call({value:7});}catch{}f=null;").unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(realm.eval("escaped().value"), Ok(Value::Number(7.0)));
    realm.eval("escaped=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, REALM_ENTRIES);
}

#[test]
fn abrupt_calls_restore_caller_state_and_host_limits_skip_finalizers() {
    let mut realm = Realm::default();
    realm
        .eval(
            "let flag=0;function f(){'use strict';try{return f();}catch{flag=1;}finally{flag=2;}}",
        )
        .unwrap();
    assert!(matches!(realm.eval("f()"), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("undeclared=7"), Ok(Value::Number(7.0)));
    assert_eq!(
        realm.eval("(function(){'use strict';return 9;})()"),
        Ok(Value::Number(9.0))
    );
    assert!(matches!(
        realm.eval("(function(){'use strict';throw 3;})()"),
        Err(Error::Thrown(Value::Number(3.0)))
    ));
    assert_eq!(realm.eval("another=8"), Ok(Value::Number(8.0)));
    let mut realm = Realm::new(Limits {
        max_properties: 8,
        ..Limits::default()
    });
    realm
        .eval("let flag=0;function f(){'use strict';flag=9;}")
        .unwrap();
    assert!(matches!(
        realm.eval("try{f(1,2,3,4,5,6,7);}finally{flag=1;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
