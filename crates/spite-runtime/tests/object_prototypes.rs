//! Object prototype mutation and extensibility through standard library APIs.

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
fn inspection_and_mutation_use_internal_prototypes_without_user_conversion() {
    check(
        "Object.getPrototypeOf({})===Object.prototype && Object.getPrototypeOf(Object.prototype)===null && Object.getPrototypeOf(true)===Boolean.prototype && Object.getPrototypeOf(1)===Number.prototype",
    );
    check(
        "let p={x:3,valueOf:()=>{throw 1;}},o={valueOf:()=>{throw 2;}};Object.setPrototypeOf(o,p)===o && o.x===3 && Object.getPrototypeOf(o)===p && p.isPrototypeOf(o)",
    );
    check(
        "let o={};Object.defineProperty(o,'__proto__',{get:()=>{throw 1;},set:()=>{throw 2;}});Object.setPrototypeOf(o,null);Object.getPrototypeOf(o)===null",
    );
    check(
        "let F=function(){},p={};Object.setPrototypeOf(F,p);Object.getPrototypeOf(F)===p && typeof F==='function' && new F instanceof F",
    );
    for target in ["null", "undefined"] {
        type_error(&format!("Object.getPrototypeOf({target})"));
        type_error(&format!("Object.setPrototypeOf({target},null)"));
    }
    for primitive in ["true", "false", "0", "-0", "NaN", "1n", "'x'"] {
        check(&format!(
            "Object.is(Object.setPrototypeOf({primitive},null),{primitive}) && Object.is(Object.setPrototypeOf({primitive},{{}}),{primitive})"
        ));
        type_error(&format!("Object.setPrototypeOf({primitive},1)"));
    }
    for prototype in ["undefined", "true", "1", "1n", "'x'"] {
        type_error(&format!("Object.setPrototypeOf({{}},{prototype})"));
    }
}

#[test]
fn cycle_and_immutable_prototype_rejections_leave_existing_chains_intact() {
    let mut realm = Realm::default();
    realm
        .eval("let a={},b={__proto__:a},c={__proto__:b}")
        .unwrap();
    for source in [
        "Object.setPrototypeOf(a,a)",
        "Object.setPrototypeOf(a,c)",
        "Object.setPrototypeOf(Object.prototype,{})",
    ] {
        assert!(
            matches!(
                realm.eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    assert_eq!(realm.eval("Object.getPrototypeOf(a)===Object.prototype && Object.getPrototypeOf(b)===a && Object.getPrototypeOf(c)===b && Object.getPrototypeOf(Object.prototype)===null"),Ok(Value::Boolean(true)));
    check("Object.setPrototypeOf(Object.prototype,null)===Object.prototype");
    check(
        "let a={},b={__proto__:a};Object.setPrototypeOf(b,null);Object.setPrototypeOf(a,b);Object.getPrototypeOf(a)===b && Object.getPrototypeOf(b)===null",
    );
}

#[test]
fn non_extensible_objects_keep_mutable_existing_properties_and_prototype_identity() {
    check(
        "let p={},o={__proto__:p,x:1};Object.isExtensible(o) && Object.preventExtensions(o)===o && !Object.isExtensible(o) && Object.preventExtensions(o)===o && Object.setPrototypeOf(o,p)===o",
    );
    type_error("let o={};Object.preventExtensions(o);Object.setPrototypeOf(o,null)");
    type_error("let o={};Object.preventExtensions(o);Object.defineProperty(o,'x',{value:1})");
    type_error("'use strict';let o={};Object.preventExtensions(o);o.x=1");
    check(
        "let o={x:1};Object.preventExtensions(o);o.x=2;o.y=3;Object.defineProperty(o,'x',{enumerable:false});o.x===2 && !Object.hasOwn(o,'y') && !o.propertyIsEnumerable('x') && delete o.x && !Object.hasOwn(o,'x') && !Object.isExtensible(o)",
    );
    check(
        "let stored=0,p={};Object.defineProperty(p,'x',{set:function(v){stored=v;}});let o={__proto__:p};Object.preventExtensions(o);o.x=7;stored===7 && !Object.hasOwn(o,'x')",
    );
    for primitive in ["undefined", "null", "true", "1", "-0", "NaN", "1n", "'x'"] {
        check(&format!(
            "!Object.isExtensible({primitive}) && Object.is(Object.preventExtensions({primitive}),{primitive})"
        ));
    }
}

#[test]
fn changing_callable_prototypes_and_descriptor_results_preserves_internal_brands() {
    check(
        "let n=new Number(3);Object.setPrototypeOf(n,null);Number.prototype.valueOf.call(n)===3 && Object.getPrototypeOf(n)===null",
    );
    check(
        "let e=TypeError();Object.setPrototypeOf(e,null);Error.isError(e) && Object.prototype.toString.call(e)==='[object Error]'",
    );
    check(
        "let o={x:1};Object.preventExtensions(o);let d=Object.getOwnPropertyDescriptor(o,'x');Object.isExtensible(d) && d.writable && d.configurable",
    );
    let mut realm = Realm::default();
    realm
        .eval(
            "let c={};Object.setPrototypeOf(c,{x:{value:7}});let C=Object;delete globalThis.Object",
        )
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("C.getPrototypeOf(c).x.value===7 && C.isExtensible(c) && C.preventExtensions(c)===c && !C.isExtensible(c)"),Ok(Value::Boolean(true)));
}

#[test]
fn prototype_and_extensibility_method_metadata_is_standard() {
    for (name, length) in [
        ("getPrototypeOf", 1),
        ("setPrototypeOf", 2),
        ("isExtensible", 1),
        ("preventExtensions", 1),
    ] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Object,'{name}');d.value.name==='{name}' && d.value.length==={length} && d.writable && !d.enumerable && d.configurable"
        ));
        type_error(&format!("new Object.{name}()"));
    }
}
