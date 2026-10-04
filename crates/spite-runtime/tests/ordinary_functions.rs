//! Function identity, declaration instantiation, standard metadata, and tracing.

mod common;
use common::REALM_ENTRIES;

use spite_runtime::{Error, ExceptionKind, Realm, Value};

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
fn ordinary_functions_have_standard_names_lengths_and_exact_source() {
    string("typeof function(){}", "function");
    string("(function(){}).name", "");
    string("(function named(){}).name", "named");
    string("let f=function(){}; f.name", "f");
    string("let f=function local(){};f.name", "local");
    string("({f:function(){}}).f.name", "f");
    string("let f; f ||= function(){};f.name", "f");
    string("((a=function(){})=>a.name)()", "a");
    string(
        "(function(a /*x*/){ /*y*/ return a; }).toString()",
        "function(a /*x*/){ /*y*/ return a; }",
    );
    string(
        "function f(a){} f.name='other';f.toString()",
        "function f(a){}",
    );
    string("function f(a,b=1){} f.bind(null).name", "bound f");
    number("(function(a,a,b){}).length", 3.0);
    number("(function(a,b=1,c){}).length", 1.0);
    number("(function(a,b=1,c){}).bind(null,1).length", 0.0);
    number(
        "let f=function(){};typeof local==='undefined' && f!==function(){} ? 1:0",
        1.0,
    );
}

#[test]
fn prototype_constructor_cycles_and_descriptors_are_standard() {
    let mut realm = Realm::default();
    let Value::Object(function) = realm.eval("function f(a){} f").unwrap() else {
        panic!()
    };
    let record = realm.inspect_object(&function).unwrap();
    let keys: Vec<_> = record
        .own_keys()
        .into_iter()
        .map(|key| key.as_string().expect("string key").to_utf8().unwrap())
        .collect();
    assert_eq!(keys, ["length", "name", "prototype"]);
    for name in ["name", "length"] {
        let property = record
            .own_property(&spite_core::JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert!(!property.writable && !property.enumerable && property.configurable);
    }
    let property = record
        .own_property(&spite_core::JsString::from("prototype"))
        .unwrap()
        .as_data()
        .unwrap();
    assert!(property.writable && !property.enumerable && !property.configurable);
    let Value::Object(prototype) = &property.value else {
        panic!()
    };
    let constructor = realm
        .inspect_object(prototype)
        .unwrap()
        .own_property(&spite_core::JsString::from("constructor"))
        .unwrap()
        .as_data()
        .unwrap();
    assert!(constructor.writable && !constructor.enumerable && constructor.configurable);
    assert_eq!(constructor.value, Value::Object(function.clone()));
    assert_eq!(realm.eval("delete f.prototype"), Ok(Value::Boolean(false)));
    assert_eq!(
        realm.eval("f.prototype.constructor===f"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.eval("f.prototype=3;f.prototype"),
        Ok(Value::Number(3.0))
    );
    assert_eq!(
        realm.eval("'prototype' in f.bind(null)"),
        Ok(Value::Boolean(false))
    );
    assert!(matches!(
        realm.eval("f.caller"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert!(matches!(
        realm.eval("f.arguments"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    string(
        "(function(){}).call.call(({}).toString,function(){})",
        "[object Function]",
    );
}

#[test]
fn script_functions_hoist_use_the_last_declaration_and_do_not_reinitialize_on_evaluation() {
    number("f.length;function f(a,b){}", 2.0);
    number(
        "let before=f;function f(a){}function f(a,b){}before===f?f.length:99",
        2.0,
    );
    number("function f(a){}var f;f.length", 1.0);
    number("var f=3;function f(a){}f", 3.0);
    number("f=3;function f(){}f", 3.0);
    number("let f=9;{function f(a){}f=1;}f", 9.0);
    number("function f(a){}delete f?99:f.length", 1.0);
    number("Object.length;var Object;function Object(a,b){}", 2.0);
    let mut realm = Realm::default();
    realm.eval("f=1").unwrap();
    realm.eval("function f(a){}").unwrap();
    assert_eq!(realm.eval("delete f"), Ok(Value::Boolean(false)));
    realm.eval("function f(a,b){}").unwrap();
    assert_eq!(realm.eval("f.length"), Ok(Value::Number(2.0)));
    assert!(matches!(
        realm.eval("let f;"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
}

#[test]
fn global_conflicts_precede_all_instantiation_effects() {
    for name in ["NaN", "Infinity", "undefined"] {
        let mut realm = Realm::default();
        assert!(matches!(
            realm.eval(&format!("let added=1;function {name}(){{}}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(
            realm.eval("typeof added"),
            Ok(Value::String("undefined".into()))
        );
    }
    let mut realm = Realm::default();
    realm.eval("let f=1;let flag=0").unwrap();
    assert!(matches!(
        realm.eval("flag=2;function f(){}"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert!(matches!(
        realm.eval("function NaN(){} let f;"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
}

#[test]
fn block_switch_and_function_body_declarations_initialize_at_scope_entry() {
    number("let n=0;{n=f.length;function f(a,b){} } n", 2.0);
    number(
        "let n=0;switch(1){case 1:n=f.length;break;case 2:function f(a,b){} } n",
        2.0,
    );
    number("(()=>{return f.length;function f(a,b){} })()", 2.0);
    number(
        "(()=>{'use strict';return f.length;function f(a,b){} })()",
        2.0,
    );
    number("((f)=>{function f(a,b){}return f.length;})(9)", 2.0);
    number("((f=9)=>{function f(a,b){}return f.length;})()", 2.0);
    number(
        "let f={}; ((get=()=>f)=>{function f(){}return get()===f?0:1;})()",
        1.0,
    );
    number(
        "(()=>{function f(a){}function f(a,b){} return f.length;})()",
        2.0,
    );
    number(
        "let n=0;for(let i=0;i<2;i++){n+=f.length;function f(a){} }n",
        2.0,
    );
    string("{function f(){}}typeof f", "undefined");
    string("if(false){function f(){}}typeof f", "undefined");
    string("(()=>{return function named(){};})().name", "named");
}

#[test]
fn private_names_and_captured_environments_are_traced_with_prototype_cycles() {
    let mut realm = Realm::default();
    let value = realm
        .eval("{let payload={value:7};(function local(){return payload;})}")
        .unwrap();
    let root = realm.root_value(value, 10000).unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, REALM_ENTRIES + 5);
    let Value::Object(function) = root.value() else {
        panic!()
    };
    assert!(realm.inspect_object(function).is_ok());
    drop(root);
    assert_eq!(realm.collect(10000).unwrap().live, REALM_ENTRIES);
    realm
        .eval("let saved=(()=>{let payload={value:7};function f(){return payload;}return f;})();")
        .unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(
        realm.eval("saved.prototype.constructor===saved"),
        Ok(Value::Boolean(true))
    );
    realm.eval("saved=null").unwrap();
    assert_eq!(realm.collect(10000).unwrap().live, REALM_ENTRIES);
}
