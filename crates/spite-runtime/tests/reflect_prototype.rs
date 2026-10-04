//! Reflect prototype and extensibility operations return internal booleans.

mod common;
use common::REALM_ENTRIES;
use spite_runtime::{Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn object_only_targets_and_prototypes_are_never_coerced() {
    for method in [
        "getPrototypeOf",
        "setPrototypeOf",
        "isExtensible",
        "preventExtensions",
    ] {
        for target in ["undefined", "null", "false", "1", "'s'", "1n", "Symbol()"] {
            check(&format!(
                "let caught=false;try{{Reflect.{method}({target},null);}}catch(e){{caught=e instanceof TypeError;}}caught"
            ));
        }
    }
    for prototype in ["undefined", "true", "1", "'s'", "1n", "Symbol()"] {
        check(&format!(
            "let caught=false,o={{}};try{{Reflect.setPrototypeOf(o,{prototype});}}catch(e){{caught=e instanceof TypeError;}}caught && Reflect.getPrototypeOf(o)===Object.prototype"
        ));
    }
    check(
        "let reads=0,o={get prototype(){reads++;throw 7;},toString(){reads++;throw 8;},valueOf(){reads++;throw 9;}};Reflect.getPrototypeOf(o)===Object.prototype && Reflect.isExtensible(o) && Reflect.setPrototypeOf(o,null) && Reflect.preventExtensions(o) && reads===0",
    );
}

#[test]
fn prototype_changes_reject_cycles_and_preserve_same_prototypes_before_extensibility() {
    check(
        "let o={},p={};Reflect.setPrototypeOf(o,p)===true && Reflect.getPrototypeOf(o)===p && Reflect.setPrototypeOf(o,null)===true && Reflect.getPrototypeOf(o)===null && Reflect.setPrototypeOf(o,Object.prototype)===true",
    );
    check(
        "let o={},p=Object.create(o),q=Object.create(p);Reflect.setPrototypeOf(o,o)===false && Reflect.setPrototypeOf(o,q)===false && Reflect.getPrototypeOf(o)===Object.prototype && Reflect.getPrototypeOf(q)===p",
    );
    check(
        "let o={},p={};Reflect.setPrototypeOf(o,p);Reflect.preventExtensions(o);Reflect.setPrototypeOf(o,p)===true && Reflect.setPrototypeOf(o,{})===false && Reflect.setPrototypeOf(o,null)===false && Reflect.getPrototypeOf(o)===p",
    );
    check(
        "let o=Object.create(null);Reflect.preventExtensions(o);Reflect.setPrototypeOf(o,null)===true && Reflect.setPrototypeOf(o,{})===false && Reflect.getPrototypeOf(o)===null",
    );
    check(
        "Reflect.setPrototypeOf(Object.prototype,null)===true && Reflect.setPrototypeOf(Object.prototype,{})===false && Reflect.getPrototypeOf(Object.prototype)===null",
    );
}

#[test]
fn preventing_extensions_retains_existing_properties_and_returns_true_repeatedly() {
    check(
        "let o={x:1};Reflect.isExtensible(o)===true && Reflect.preventExtensions(o)===true && Reflect.preventExtensions(o)===true && Reflect.isExtensible(o)===false && Object.isExtensible(o)===false && (o.x=2)===2 && o.x===2 && delete o.x && !Object.hasOwn(o,'x')",
    );
    check(
        "let o={};Reflect.preventExtensions(o);o.x=1;let caught=false;try{Object.defineProperty(o,'x',{value:1});}catch(e){caught=e instanceof TypeError;}caught && !Object.hasOwn(o,'x')",
    );
    check(
        "let o={};Reflect.preventExtensions(o);let caught=false;try{Object.setPrototypeOf(o,null);}catch(e){caught=e instanceof TypeError;}caught && Reflect.setPrototypeOf(o,null)===false",
    );
}

#[test]
fn operations_handle_exposed_exotic_objects_and_keep_getters_dormant() {
    check(
        "let a=[1,2];Reflect.preventExtensions(a);a.length=4;Reflect.isExtensible(a)===false && a.length===4 && Reflect.setPrototypeOf(a,Array.prototype)===true && Reflect.setPrototypeOf(a,null)===false",
    );
    check(
        "let s=Object('ab');Reflect.getPrototypeOf(s)===String.prototype && Reflect.preventExtensions(s) && Reflect.isExtensible(s)===false && s.length===2 && s[1]==='b'",
    );
    check(
        "function F(){}let a=F.prototype;Reflect.getPrototypeOf(F)===Object.getPrototypeOf(()=>{}) && Reflect.preventExtensions(F) && Reflect.setPrototypeOf(F,Object.getPrototypeOf(F)) && F.prototype===a",
    );
    check(
        "function F(){let target=arguments;Reflect.preventExtensions(target);arguments[0]=2;return Reflect.isExtensible(target)===false && arguments[0]===2;}F(1)",
    );
    check(
        "let reads=0,o={get x(){reads++;throw 7;}};Reflect.getPrototypeOf(o)===Object.prototype && Reflect.preventExtensions(o) && !Reflect.isExtensible(o) && Reflect.setPrototypeOf(o,Object.prototype) && reads===0",
    );
}

#[test]
fn metadata_and_retained_intrinsics_survive_public_deletion() {
    for (method, length) in [
        ("getPrototypeOf", 1),
        ("setPrototypeOf", 2),
        ("isExtensible", 1),
        ("preventExtensions", 1),
    ] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Reflect,'{method}'),n=Object.getOwnPropertyDescriptor(d.value,'name'),l=Object.getOwnPropertyDescriptor(d.value,'length');d.writable && !d.enumerable && d.configurable && n.value==='{method}' && !n.writable && !n.enumerable && n.configurable && l.value==={length} && !l.writable && !l.enumerable && l.configurable && !Object.hasOwn(d.value,'prototype')"
        ));
        check(&format!(
            "let caught=false;try{{new Reflect.{method}();}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let get=Reflect.getPrototypeOf,set=Reflect.setPrototypeOf,isExtensible=Reflect.isExtensible,prevent=Reflect.preventExtensions;delete Reflect.getPrototypeOf;delete Reflect.setPrototypeOf;delete Reflect.isExtensible;delete Reflect.preventExtensions;delete globalThis.Reflect").unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
    assert_eq!(realm.eval("let o={};set(o,null) && get(o)===null && isExtensible(o) && prevent(o) && !isExtensible(o)"),Ok(Value::Boolean(true)));
}
