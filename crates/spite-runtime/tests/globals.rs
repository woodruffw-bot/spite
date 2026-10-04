//! Global object, global lexical bindings, and global this (9.1.1.4, 19.1).

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn truth(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn script_and_arrow_this_retain_the_realm_global_identity() {
    for source in [
        "this===globalThis",
        "'use strict';this===globalThis",
        "(()=>this)()===globalThis",
        "'use strict';(()=>this).call({})===globalThis",
        "this.globalThis===this",
        "this.toString()==='[object Object]'",
        "this.valueOf()===this",
        "typeof console==='undefined' && typeof global==='undefined' && typeof process==='undefined'",
        "let saved=this;globalThis=7;this===saved && globalThis===7",
        "let saved=this;delete globalThis;this===saved && typeof globalThis==='undefined'",
        "let saved=this;let globalThis=7;saved===this && globalThis===7 && saved.globalThis===saved",
    ] {
        truth(source);
    }
}

#[test]
fn globals_and_properties_share_storage_but_lexical_bindings_remain_separate() {
    for source in [
        "var x=1;globalThis.x=7;x===7",
        "this.x=1;x=7;this.x===7",
        "let x=7;this.x=1;x===7 && this.x===1",
        "var x;this.x===undefined && 'x' in this && !delete x && !delete this.x",
        "x=7;delete this.x;typeof x==='undefined'",
        "this.x=7;delete x;!('x' in this)",
        "function f(){}f===this.f && !delete this.f",
        "var toString;toString===undefined && !delete toString",
        "delete toString;typeof toString==='function' && !delete this.undefined",
        "this.undefined=7;this.Infinity=7;undefined===this.undefined && Infinity===1/0",
    ] {
        truth(source);
    }
    let mut realm = Realm::default();
    realm.eval("this.x=1").unwrap();
    realm.eval("var x;").unwrap();
    // Edition 17 preserves configurable pre-existing properties (9.1.1.4.16).
    assert_eq!(
        realm.eval("let x=7;x===7 && this.x===1"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::default();
    realm.eval("var x=1").unwrap();
    assert!(matches!(
        realm.eval("let x;"),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    let mut realm = Realm::default();
    realm.eval("this.f=1;").unwrap();
    assert_eq!(
        realm.eval("function f(){} !delete f"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn non_strict_calls_use_the_global_object_independently_of_the_public_binding() {
    for call in [
        "f()",
        "f.call()",
        "f.call(null)",
        "f.apply(undefined,{length:0})",
        "f.bind(null)()",
    ] {
        truth(&format!("function f(){{return this;}}{call}===globalThis"));
    }
    truth("function f(){return this;}let saved=this;globalThis=null;f()===saved");
    truth("function f(){return this;}let saved=this;delete globalThis;f()===saved");
    truth("function f(){return ()=>this;}f()()===globalThis");
    truth("function f(){'use strict';return this;}f()===undefined && this.f()===this");
    truth("function f(a){a=7;return arguments[0];}f(1)===7");
    truth("function f(n){return n<2?1:n*f(n-1);}f(5)===120");
    truth("function f(a=()=>this){return a;}f()()===this");
}

#[test]
fn resolved_global_assignments_recheck_presence_after_the_right_hand_side() {
    truth("this.x=1;x=(delete this.x,7);x===7 && delete x");
    truth(
        "'use strict';this.x=1;let caught=false;try{x=(delete this.x,7);}catch{caught=true;}caught && !('x' in this)",
    );
    truth(
        "'use strict';this.x=1;let caught=false;try{x+=(delete this.x,7);}catch{caught=true;}caught && !('x' in this)",
    );
    truth(
        "'use strict';let caught=false;try{missing=(this.missing=1,7);}catch{caught=true;}caught && this.missing===1",
    );
    truth("missing=(this.missing=1,7);this.missing===7");
    for name in ["undefined", "Infinity", "NaN"] {
        assert!(matches!(
            Realm::default().eval(&format!("'use strict';this.{name}=7")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert!(matches!(
            Realm::default().eval(&format!("'use strict';delete this.{name}")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn global_descriptors_and_collection_preserve_required_attributes_and_identity() {
    let mut realm = Realm::default();
    let Value::Object(global) = realm.eval("var x=1;function f(){};sloppy=7;this").unwrap() else {
        panic!()
    };
    let object = realm.inspect_object(&global).unwrap();
    for name in ["undefined", "NaN", "Infinity"] {
        let data = object
            .own_property(&spite_core::JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert!(!data.writable && !data.enumerable && !data.configurable);
    }
    for (name, writable, enumerable, configurable) in [
        ("globalThis", true, false, true),
        ("x", true, true, false),
        ("f", true, true, false),
        ("sloppy", true, true, true),
    ] {
        let data = object
            .own_property(&spite_core::JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert_eq!(
            (data.writable, data.enumerable, data.configurable),
            (writable, enumerable, configurable)
        );
    }
    realm
        .eval("this.saved={nested:{}};delete globalThis;f=null")
        .unwrap();
    realm.collect(10000).unwrap();
    assert_eq!(realm.eval("this"), Ok(Value::Object(global)));
    truth("this.saved={value:7};saved.value===7");
    assert_eq!(
        realm.eval("typeof saved.nested"),
        Ok(Value::String("object".into()))
    );
}

#[test]
fn unavailable_standard_global_properties_remain_explicit_gaps() {
    for source in [
        "this.Math",
        "typeof this.Math",
        "this.Math=7",
        "delete this.Math",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    truth("'Math' in this");
    truth("function Math(){}Math===this.Math");
}
