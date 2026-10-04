//! JavaScript property descriptors, observable conversion order, and SameValue.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

mod common;

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
fn data_descriptors_default_preserve_and_reject_attributes_correctly() {
    check(
        "let o={};Object.defineProperty(o,'x',{value:7})===o && o.x===7 && !o.propertyIsEnumerable('x') && !delete o.x",
    );
    check(
        "let o={};Object.defineProperty(o,'x',{});let d=Object.getOwnPropertyDescriptor(o,'x');Object.hasOwn(o,'x') && d.value===undefined && !d.writable && !d.enumerable && !d.configurable && !Object.hasOwn(d,'get')",
    );
    check(
        "let o={x:1};Object.defineProperty(o,'x',{value:2});let d=Object.getOwnPropertyDescriptor(o,'x');d.value===2 && d.writable && d.enumerable && d.configurable",
    );
    check(
        "let o={};Object.defineProperty(o,'x',{value:NaN});Object.defineProperty(o,'x',{value:NaN});Object.is(o.x,NaN)",
    );
    type_error(
        "let o={};Object.defineProperty(o,'x',{value:0});Object.defineProperty(o,'x',{value:-0})",
    );
    for descriptor in [
        "{value:2}",
        "{writable:true}",
        "{configurable:true}",
        "{enumerable:true}",
        "{get:undefined}",
    ] {
        let mut realm = Realm::default();
        realm
            .eval("let o={};Object.defineProperty(o,'x',{value:1})")
            .unwrap();
        assert!(
            matches!(
                realm.eval(&format!("Object.defineProperty(o,'x',{descriptor})")),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{descriptor}"
        );
        assert_eq!(
            realm.eval("o.x===1 && !Object.getOwnPropertyDescriptor(o,'x').writable"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn reflected_descriptors_are_fresh_ordinary_objects_with_ordered_mutable_fields() {
    let mut realm = Realm::default();
    realm.eval("let o={x:3};let getter=function(){throw 1;};Object.defineProperty(o,'y',{get:getter,configurable:true});").unwrap();
    for (key, fields) in [
        ("x", ["value", "writable", "enumerable", "configurable"]),
        ("y", ["get", "set", "enumerable", "configurable"]),
    ] {
        let Value::Object(descriptor) = realm
            .eval(&format!("Object.getOwnPropertyDescriptor(o,'{key}')"))
            .unwrap()
        else {
            panic!("descriptor");
        };
        let descriptor = realm.inspect_object(&descriptor).unwrap();
        assert_eq!(
            descriptor.own_keys(),
            fields.map(spite_core::PropertyKey::from)
        );
        for field in fields {
            let property = descriptor
                .own_property(&JsString::from(field))
                .unwrap()
                .as_data()
                .unwrap();
            assert!(property.writable && property.enumerable && property.configurable);
        }
    }
    assert_eq!(realm.eval("let a=Object.getOwnPropertyDescriptor(o,'x');let b=Object.getOwnPropertyDescriptor(o,'x');a.value=99;a!==b && a.constructor===Object && o.x===3 && b.value===3 && Object.getOwnPropertyDescriptor(o,'missing')===undefined && Object.hasOwn(o,'y')"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("Object.getOwnPropertyDescriptor(o,'y').get===getter && Object.getOwnPropertyDescriptor(o,'y').set===undefined"),Ok(Value::Boolean(true)));
}

#[test]
fn accessors_preserve_receiver_identity_and_survive_kind_changes_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let o={};Object.defineProperty(o,'x',{get:function(){return this;},set:function(v){this.last=v;},configurable:true});let c={__proto__:o};").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("c.x===c && (c.x=7)===7 && c.last===7 && !Object.hasOwn(c,'x')"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("Object.defineProperty(o,'x',{value:4});let d=Object.getOwnPropertyDescriptor(o,'x');d.value===4 && !d.writable && !d.enumerable && d.configurable && !Object.hasOwn(d,'get')"),Ok(Value::Boolean(true)));
    assert_eq!(realm.eval("Object.defineProperty(o,'x',{get:undefined});let e=Object.getOwnPropertyDescriptor(o,'x');o.x===undefined && Object.hasOwn(e,'get') && !Object.hasOwn(e,'value') && e.configurable"),Ok(Value::Boolean(true)));
    check(
        "let getter=()=>3,o={};Object.defineProperty(o,'x',{get:getter});Object.defineProperty(o,'x',{get:getter});o.x===3",
    );
    type_error(
        "let o={};Object.defineProperty(o,'x',{get:()=>3});Object.defineProperty(o,'x',{get:()=>3})",
    );
}

#[test]
fn descriptor_fields_use_ordered_inherited_reads_and_boolean_conversion() {
    check(
        "let log='',proto={},d={__proto__:proto};function field(n,v){Object.defineProperty(proto,n,{get:function(){log+=n+':';if(this!==d)throw 1;return v;}});}field('enumerable',{});field('configurable',1);field('value',9);field('writable','');field('get',undefined);field('set',undefined);let o={},caught=false;try{Object.defineProperty(o,'x',d);}catch(e){caught=e instanceof TypeError;}caught && !Object.hasOwn(o,'x') && log==='enumerable:configurable:value:writable:get:set:'",
    );
    check(
        "let d={value:4,writable:true};Object.defineProperty(d,'enumerable',{get:function(){delete this.value;this.configurable=true;return false;}});let o={};Object.defineProperty(o,'x',d);let r=Object.getOwnPropertyDescriptor(o,'x');r.value===undefined && r.writable && r.configurable && !r.enumerable",
    );
    check(
        "let truthy={valueOf:()=>{throw 1;}};let o={};Object.defineProperty(o,'x',{value:1,writable:truthy,enumerable:truthy,configurable:truthy});let d=Object.getOwnPropertyDescriptor(o,'x');d.writable && d.enumerable && d.configurable",
    );
    let mut realm = Realm::default();
    realm.eval("let log='',d={get:null};Object.defineProperty(d,'set',{get:()=>{log+='set';throw 9;}})").unwrap();
    assert!(matches!(
        realm.eval("Object.defineProperty({},'x',d)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    assert_eq!(realm.eval("log"), Ok(Value::String(JsString::from(""))));
    assert_eq!(realm.eval("let d2={};Object.defineProperty(d2,'value',{get:()=>{throw 8;}});Object.defineProperty({},'x',d2)"),Err(Error::Thrown(Value::Number(8.0))));
}

#[test]
fn static_methods_convert_the_target_before_the_key_and_preserve_abrupt_completions() {
    for method in ["defineProperty", "getOwnPropertyDescriptor", "hasOwn"] {
        type_error(&format!(
            "Object.{method}(null,{{toString:()=>{{throw 9;}}}},{{}})"
        ));
        assert_eq!(
            Realm::default().eval(&format!(
                "Object.{method}({{}},{{toString:()=>{{throw 9;}}}},null)"
            )),
            Err(Error::Thrown(Value::Number(9.0)))
        );
    }
    for target in ["true", "1", "1n", "'x'", "undefined"] {
        type_error(&format!(
            "Object.defineProperty({target},{{toString:()=>{{throw 9;}}}},{{}})"
        ));
    }
    for attributes in [
        "undefined",
        "null",
        "1",
        "'x'",
        "true",
        "{get:null}",
        "{set:3}",
        "{get:undefined,value:undefined}",
        "{set:undefined,writable:false}",
    ] {
        type_error(&format!("Object.defineProperty({{}},'x',{attributes})"));
    }
    check(
        "!Object.hasOwn(true,'valueOf') && Object.getOwnPropertyDescriptor(1,'toString')===undefined && Object.hasOwn({__proto__:null,x:1},'x') && !Object.hasOwn({__proto__:{x:1}},'x')",
    );
    check(
        "let log='',key={toString:()=>{log+='k';return 'x';}},d={};Object.defineProperty(d,'value',{get:()=>{log+='d';return 3;}});let o={};Object.defineProperty(o,key,d);log==='kd' && o.x===3",
    );
}

#[test]
fn mapped_arguments_follow_live_values_and_detach_only_after_successful_definitions() {
    check(
        "function f(a){let x=arguments;a=2;if(Object.getOwnPropertyDescriptor(x,'0').value!==2)return false;Object.defineProperty(x,'0',{value:3});if(a!==3)return false;Object.defineProperty(x,'0',{writable:false});a=4;return x[0]===3 && !Object.getOwnPropertyDescriptor(x,'0').writable;}f(1)",
    );
    check(
        "function f(a){Object.defineProperty(arguments,'0',{get:()=>7});a=4;return arguments[0]===7 && a===4;}f(1)",
    );
    check(
        "function f(a){Object.defineProperty(arguments,'0',{configurable:false});try{Object.defineProperty(arguments,'0',{get:()=>7});}catch{}a=4;return arguments[0]===4;}f(1)",
    );
    check(
        "var x=1;Object.defineProperty(globalThis,'x',{value:2});x===2 && Object.getOwnPropertyDescriptor(globalThis,'x').value===2",
    );
}

#[test]
fn same_value_does_not_coerce_and_distinguishes_negative_zero() {
    check(
        "Object.is() && Object.is(NaN,NaN) && !Object.is(-0,0) && Object.is(-0,-0) && !Object.is(1,1n) && !Object.is(false,0) && Object.is(42n,42n) && !Object.is(42n,43n) && Object.is('\\uD800','\\uD800') && !Object.is('a','b')",
    );
    check("let o={valueOf:()=>{throw 1;}};Object.is(o,o) && !Object.is(o,{}) && !Object.is(o,1)");
}

#[test]
fn method_metadata_roots_and_missing_intrinsic_descriptors_remain_correct() {
    for (method, length) in [
        ("defineProperty", 3),
        ("getOwnPropertyDescriptor", 2),
        ("hasOwn", 2),
        ("is", 2),
    ] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Object,'{method}');d.writable && !d.enumerable && d.configurable && d.value.name==='{method}' && d.value.length==={length}"
        ));
        type_error(&format!("new Object.{method}()"));
    }
    let mut realm = Realm::default();
    realm.eval("let C=Object;delete globalThis.Object").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let o={};C.defineProperty(o,'x',{value:3});C.is(C.getOwnPropertyDescriptor(o,'x').value,3) && C.hasOwn(o,'x')"),Ok(Value::Boolean(true)));
    for source in [
        "Object.getOwnPropertyDescriptor(String.prototype,'normalize')",
        "Object.hasOwn(String.prototype,'normalize')",
        "Object.defineProperty(String.prototype,'normalize',{})",
        "Object.defineProperty(globalThis,'Math',{})",
        "Object.getOwnPropertyDescriptor(globalThis,'Math')",
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
fn descriptor_allocation_failure_is_a_host_limit_and_leaves_the_target_intact() {
    let mut realm = Realm::new(Limits {
        max_heap_entries: Some(common::REALM_ENTRIES + 1),
        ..Limits::default()
    });
    realm.eval("let o={x:3}").unwrap();
    assert_eq!(
        realm.eval("Object.getOwnPropertyDescriptor(o,'missing')"),
        Ok(Value::Undefined)
    );
    assert_eq!(realm.eval("Object.hasOwn(o,'x')"), Ok(Value::Boolean(true)));
    assert!(matches!(
        realm.eval("Object.getOwnPropertyDescriptor(o,'x')"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("o.x"), Ok(Value::Number(3.0)));
}
