//! Object creation and the two phases of ObjectDefineProperties.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

fn type_error(source: &str) {
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

#[test]
fn create_uses_exact_prototype_identity_without_calling_or_coercing_it() {
    check(
        "let p={x:1,valueOf:()=>{throw 1;}};let a=Object.create(p),b=Object.create(p);a!==b && a.x===1 && !Object.hasOwn(a,'x') && Object.getPrototypeOf(a)===p && Object.isExtensible(a)",
    );
    check(
        "let o=Object.create(null);Object.getPrototypeOf(o)===null && !('constructor' in o) && !('toString' in o)",
    );
    check(
        "let F=function(){throw 1;};let o=Object.create(F);Object.getPrototypeOf(o)===F && typeof o==='object'",
    );
    check(
        "let p={},o=Object.create(p,{x:{value:3},y:{get:function(){return this;}}});o.x===3 && o.y===o && !Object.getOwnPropertyDescriptor(o,'x').writable && Object.getPrototypeOf(o)===p",
    );
    check(
        "let o=Object.create(null,{['__proto__']:{value:7}});o.__proto__===7 && Object.getPrototypeOf(o)===null",
    );
    for prototype in ["undefined", "1", "true", "1n", "'x'"] {
        type_error(&format!("Object.create({prototype},null)"));
    }
    type_error("Object.create(null,null)");
    check(
        "Object.getPrototypeOf(Object.create(null,undefined))===null && Object.getPrototypeOf(Object.create(null,true))===null",
    );
}

#[test]
fn definitions_snapshot_numeric_then_string_keys_and_ignore_inherited_or_hidden_fields() {
    let mut realm = Realm::default();
    let Value::Object(object)=realm.eval("let log='',props={__proto__:{inherited:null}};function add(k){Object.defineProperty(props,k,{get:()=>{log+=k+':';return {value:k};},enumerable:true});}add('b');add('10');add('2');add('a');add('0');Object.defineProperty(props,'hidden',{get:()=>{throw 1;}});let o={};Object.defineProperties(o,props)").unwrap() else {panic!("target");};
    assert_eq!(
        realm.inspect_object(&object).unwrap().own_keys(),
        ["0", "2", "10", "b", "a"].map(JsString::from)
    );
    assert_eq!(
        realm.eval(
            "log==='0:2:10:b:a:' && !Object.hasOwn(o,'inherited') && !Object.hasOwn(o,'hidden')"
        ),
        Ok(Value::Boolean(true))
    );
    check(
        "let props={};Object.defineProperty(props,'a',{get:()=>{delete props.b;props.c={value:3};return {value:1};},enumerable:true});props.b={value:2};let o={};Object.defineProperties(o,props);o.a===1 && !Object.hasOwn(o,'b') && !Object.hasOwn(o,'c')",
    );
    check(
        "let props={};Object.defineProperty(props,'a',{get:()=>{Object.defineProperty(props,'b',{enumerable:false});return {value:1};},enumerable:true});props.b=null;let o={};Object.defineProperties(o,props);o.a===1 && !Object.hasOwn(o,'b')",
    );
}

#[test]
fn every_conversion_precedes_definitions_and_earlier_records_are_independent() {
    check(
        "let target={},observed=false,first={value:1};let props={a:first};Object.defineProperty(props,'b',{get:()=>{observed=!Object.hasOwn(target,'a');first.value=99;return {value:2};},enumerable:true});Object.defineProperties(target,props)===target && observed && target.a===1 && target.b===2",
    );
    let mut realm = Realm::default();
    realm
        .eval("let target={},props={a:{value:1},b:{get:null}}")
        .unwrap();
    assert!(matches!(
        realm.eval("Object.defineProperties(target,props)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(
        realm.eval("!Object.hasOwn(target,'a') && !Object.hasOwn(target,'b')"),
        Ok(Value::Boolean(true))
    );
    realm.eval("let props2={a:{value:1}};Object.defineProperty(props2,'b',{get:()=>{target.side=7;throw 9;},enumerable:true})").unwrap();
    assert_eq!(
        realm.eval("Object.defineProperties(target,props2)"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    assert_eq!(
        realm.eval("!Object.hasOwn(target,'a') && target.side===7"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn later_definition_rejection_keeps_earlier_definitions_after_all_conversions() {
    let mut realm = Realm::default();
    realm.eval("let o={},seen=false;Object.defineProperty(o,'b',{value:0});let props={a:{value:1},b:{value:2}};Object.defineProperty(props,'c',{get:()=>{seen=!Object.hasOwn(o,'a');return {value:3};},enumerable:true})").unwrap();
    assert!(matches!(
        realm.eval("Object.defineProperties(o,props)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(
        realm.eval("seen && o.a===1 && o.b===0 && !Object.hasOwn(o,'c')"),
        Ok(Value::Boolean(true))
    );
    check(
        "function f(a){Object.defineProperties(arguments,{0:{value:3,writable:false},x:{get:()=>9}});a=4;return arguments[0]===3 && arguments.x===9 && a===4;}f(1)",
    );
}

#[test]
fn primitive_properties_and_invalid_targets_follow_distinct_conversion_rules() {
    for target in ["null", "undefined", "true", "1", "'x'", "1n"] {
        type_error(&format!("Object.defineProperties({target},{{}})"));
    }
    for properties in ["null", "undefined"] {
        type_error(&format!("Object.defineProperties({{}},{properties})"));
    }
    for properties in ["true", "false", "1", "NaN"] {
        check(&format!(
            "let o={{}};Object.defineProperties(o,{properties})===o && Object.isExtensible(o)"
        ));
    }
    for source in [
        "Object.create(null,1n)",
        "Object.defineProperties({},Object)",
        "Object.defineProperties({},globalThis)",
        "Object.defineProperties({},Object.getPrototypeOf(Object))",
    ] {
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
fn metadata_and_created_object_edges_survive_global_replacement_and_collection() {
    for name in ["create", "defineProperties"] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Object,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===2"
        ));
        type_error(&format!("new Object.{name}()"));
    }
    let mut realm = Realm::default();
    realm
        .eval("let C=Object;let o=C.create({x:7},{y:{get:()=>8}});delete globalThis.Object")
        .unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.eval("o.x===7 && o.y===8 && C.defineProperties(o,{z:{value:9}})===o && o.z===9"),
        Ok(Value::Boolean(true))
    );
}
