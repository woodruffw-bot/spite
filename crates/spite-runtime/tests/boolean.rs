//! Boolean constructor, internal slots, inherited methods, and primitive receivers.

mod common;
use common::REALM_ENTRIES;

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn calls_use_truthiness_without_coercing_objects() {
    for source in ["undefined", "null", "false", "0", "-0", "NaN", "0n", "''"] {
        check(&format!("Boolean({source}) === false"));
    }
    for source in [
        "true",
        "1",
        "-1",
        "Infinity",
        "1n",
        "'false'",
        "{}",
        "new Boolean(false)",
    ] {
        check(&format!("Boolean({source}) === true"));
    }
    check("Boolean() === false && Boolean.call(null, true) === true");
    check(
        "let touched=false; let o={valueOf:()=>{touched=true;throw 1;},toString:()=>{touched=true;throw 2;}}; Boolean(o) && new Boolean(o).valueOf() && !touched",
    );
    check("let n=0; Boolean(false,n=1) === false && n===1");
}

#[test]
fn construction_creates_fresh_boolean_objects_and_bound_calls_preserve_rules() {
    check(
        "let a=new Boolean, b=new Boolean(false); typeof a==='object' && a!==b && a.valueOf()===false && b.valueOf()===false && a instanceof Boolean && b.constructor===Boolean",
    );
    check(
        "let a=new Boolean(true); a.valueOf()===true && a.toString()==='true' && Boolean.prototype.valueOf()===false && Boolean.prototype.toString()==='false'",
    );
    check(
        "let B=Boolean.bind({valueOf:()=>true},false); B(true)===false && new B(true).valueOf()===false && new B instanceof Boolean && new B instanceof B",
    );
    check(
        "let B=Boolean.bind(null); B.prototype={}; new B(true).constructor===Boolean && new B(true).valueOf()===true",
    );
    check(
        "let o={__proto__:Boolean.prototype}; ({}).toString.call(o)==='[object Object]' && ({}).toString.call(new Boolean)==='[object Boolean]'",
    );
}

#[test]
fn methods_require_a_boolean_primitive_or_own_internal_slot() {
    check(
        "true.valueOf()===true && false.valueOf()===false && true.toString()==='true' && false.toString()==='false'",
    );
    for method in ["valueOf", "toString"] {
        for receiver in [
            "undefined",
            "null",
            "0",
            "1n",
            "''",
            "{}",
            "({__proto__:Boolean.prototype})",
            "(()=>true)",
        ] {
            let source = format!("Boolean.prototype.{method}.call({receiver})");
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{source}"
            );
        }
        for source in [
            format!("let f=Boolean.prototype.{method}; f()"),
            format!("new Boolean.prototype.{method}"),
        ] {
            assert!(
                matches!(
                    Realm::default().eval(&source),
                    Err(Error::Exception {
                        kind: ExceptionKind::TypeError,
                        ..
                    })
                ),
                "{source}"
            );
        }
    }
}

#[test]
fn wrapper_coercion_obeys_ordinary_conversion_and_object_truthiness() {
    check(
        "let f=new Boolean(false),t=new Boolean(true); !!f && f==false && f!==false && +f===0 && +t===1 && f+2===2 && t+2===3 && `${f}`==='false'",
    );
    check("let o=new Boolean(false); o.valueOf=()=>7; +o===7 && `${o}`==='false'");
    check("let o=new Boolean(true); o.valueOf=()=>({}); o.toString=()=> '8'; +o===8");
}

#[test]
fn non_strict_receivers_box_fresh_values_while_strict_receivers_stay_primitive() {
    check(
        "function box(){return this;} let a=box.call(true),b=box.apply(true); typeof a==='object' && a instanceof Boolean && a.valueOf()===true && a!==b",
    );
    check(
        "function strict(){'use strict';return this;} strict.call(true)===true && strict.apply(false)===false",
    );
    check(
        "function capture(){return ()=>this;} let f=capture.call(false), a=f(); a===f() && a instanceof Boolean && a.valueOf()===false",
    );
    check(
        "Boolean.prototype.receiver=function(){return this;}; let a=true.receiver(); a instanceof Boolean && a.valueOf()===true",
    );
    check(
        "Boolean.prototype.receiver=function(){'use strict';return this;}; true.receiver()===true",
    );
    check(
        "let valueOf=({}).valueOf, a=valueOf.call(false),b=valueOf.call(false); a!==b && a.valueOf()===false && a instanceof Boolean && valueOf.call(a)===a",
    );
}

#[test]
fn primitive_properties_follow_the_prototype_and_reject_data_writes() {
    check(
        "Boolean.prototype.extra=7; true.extra===7 && false.extra===7 && delete true.extra && false.extra===7",
    );
    check(
        "Boolean.prototype.extra=7; true.extra=9; false.own=1; true.extra===7 && false.own===undefined",
    );
    check(
        "let b=new Boolean; b.extra=9; b.extra===9 && true.extra===undefined && delete b.extra && b.extra===undefined",
    );
    for source in [
        "'use strict'; true.extra=9",
        "'use strict'; Boolean.prototype.extra=7; false.extra=9",
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
    check("'use strict'; delete true.valueOf && true.valueOf()===true");
    check(
        "delete Boolean.prototype.valueOf; typeof true.valueOf()==='object' && ({}).toString.call(true.valueOf())==='[object Boolean]'",
    );
    check("delete Boolean.prototype.toString; true.toString()==='[object Boolean]'");
}

#[test]
fn constructor_metadata_and_descriptors_are_standard() {
    let mut realm = Realm::default();
    for (source, name, length, constructor) in [
        ("Boolean", "Boolean", 1.0, true),
        ("Boolean.prototype.valueOf", "valueOf", 0.0, false),
        ("Boolean.prototype.toString", "toString", 0.0, false),
    ] {
        let Value::Object(handle) = realm.eval(source).unwrap() else {
            panic!("function")
        };
        let object = realm.inspect_object(&handle).unwrap();
        assert!(object.is_callable());
        assert_eq!(object.is_constructor(), constructor);
        for (key, expected) in [
            ("name", Value::String(JsString::from(name))),
            ("length", Value::Number(length)),
        ] {
            let property = object
                .own_property(&JsString::from(key))
                .unwrap()
                .as_data()
                .unwrap();
            assert_eq!(property.value, expected);
            assert!(!property.writable && !property.enumerable && property.configurable);
        }
        if constructor {
            let prototype = object
                .own_property(&JsString::from("prototype"))
                .unwrap()
                .as_data()
                .unwrap();
            assert!(!prototype.writable && !prototype.enumerable && !prototype.configurable);
        } else {
            assert!(object.own_property(&JsString::from("prototype")).is_none());
        }
    }
    for (source, names) in [
        ("globalThis", &["Boolean"][..]),
        (
            "Boolean.prototype",
            &["constructor", "toString", "valueOf"][..],
        ),
    ] {
        let Value::Object(handle) = realm.eval(source).unwrap() else {
            panic!("object")
        };
        let object = realm.inspect_object(&handle).unwrap();
        for name in names {
            let property = object
                .own_property(&JsString::from(*name))
                .unwrap()
                .as_data()
                .unwrap();
            assert!(property.writable && !property.enumerable && property.configurable);
        }
    }
    check(
        "Boolean.toString()==='function Boolean() { [native code] }' && Boolean.prototype.valueOf.toString()==='function valueOf() { [native code] }'",
    );
    check(
        "let p=Boolean.prototype; Boolean.prototype={}; Boolean.prototype===p && !delete Boolean.prototype",
    );
}

#[test]
fn deleting_or_replacing_the_global_does_not_change_intrinsic_boxing() {
    check(
        "let B=Boolean; delete globalThis.Boolean; typeof Boolean==='undefined' && true.constructor===B && ({}).valueOf.call(false) instanceof B",
    );
    check("let B=Boolean; Boolean=7; true.constructor===B && ({}).valueOf.call(true) instanceof B");
    let mut realm = Realm::default();
    realm
        .eval("let B=Boolean; delete globalThis.Boolean; B=null;")
        .unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES);
    assert_eq!(
        realm.eval("true.constructor(false)"),
        Ok(Value::Boolean(false))
    );
}

#[test]
fn roots_retain_wrappers_and_unreachable_wrappers_are_reclaimed() {
    let mut realm = Realm::default();
    let value = realm.eval("new Boolean(true)").unwrap();
    let root = realm.root_value(value, 100).unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES + 1);
    drop(root);
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES);
    realm
        .eval("let saved=(function(){return ()=>this;}).call(false)")
        .unwrap();
    realm.collect(10_000).unwrap();
    assert_eq!(realm.eval("saved().valueOf()"), Ok(Value::Boolean(false)));
    realm.eval("saved=null").unwrap();
    assert_eq!(realm.collect(10_000).unwrap().live, REALM_ENTRIES);
}

#[test]
fn wrapper_allocation_limits_remain_uncatchable_host_failures() {
    let mut realm = Realm::new(Limits {
        max_heap_entries: REALM_ENTRIES,
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{new Boolean;}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("Boolean(false)"), Ok(Value::Boolean(false)));
}
