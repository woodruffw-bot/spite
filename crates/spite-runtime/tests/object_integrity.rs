//! Sealed and frozen own-property invariants through Object's standard APIs.

use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn primitive_and_empty_object_integrity_have_standard_special_cases() {
    for primitive in [
        "undefined",
        "null",
        "true",
        "false",
        "0",
        "-0",
        "NaN",
        "Infinity",
        "1n",
        "'x'",
    ] {
        check(&format!(
            "Object.is(Object.freeze({primitive}),{primitive}) && Object.is(Object.seal({primitive}),{primitive}) && Object.isFrozen({primitive}) && Object.isSealed({primitive})"
        ));
    }
    check(
        "let o={};!Object.isFrozen(o) && !Object.isSealed(o) && Object.preventExtensions(o)===o && Object.isFrozen(o) && Object.isSealed(o)",
    );
    check(
        "let o=Object.create(null);Object.seal(o)===o && Object.freeze(o)===o && Object.isFrozen(o) && Object.isSealed(o)",
    );
}

#[test]
fn sealing_preserves_writable_data_and_freezing_is_shallow() {
    check(
        "let child={x:1},o={child:child,x:2};Object.seal(o)===o && Object.isSealed(o) && !Object.isFrozen(o) && !Object.isExtensible(o) && !Object.getOwnPropertyDescriptor(o,'x').configurable && Object.getOwnPropertyDescriptor(o,'x').writable",
    );
    check(
        "let o={x:1};Object.seal(o);o.x=3;o.y=4;o.x===3 && !Object.hasOwn(o,'y') && !delete o.x && Object.seal(o)===o",
    );
    check(
        "let child={x:1},o={child:child,x:2};Object.freeze(o);o.x=3;child.x=4;Object.isFrozen(o) && Object.isSealed(o) && o.x===2 && child.x===4 && Object.isExtensible(child) && Object.freeze(o)===o",
    );
    check(
        "let o={x:1};Object.seal(o);Object.defineProperty(o,'x',{writable:false});Object.isFrozen(o)",
    );
    check(
        "let o={};Object.defineProperty(o,'hidden',{value:1,configurable:true});Object.freeze(o);Object.isFrozen(o) && !Object.getOwnPropertyDescriptor(o,'hidden').configurable",
    );
    for source in [
        "'use strict';let o=Object.freeze({x:1});o.x=2",
        "let o=Object.seal({});Object.defineProperty(o,'x',{value:1})",
        "let o=Object.freeze({x:1});Object.defineProperty(o,'x',{writable:true})",
        "let o=Object.seal({x:1});Object.defineProperty(o,'x',{configurable:true})",
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
}

#[test]
fn accessors_are_never_invoked_or_disabled_by_integrity_operations() {
    check(
        "let count=0,o={};let get=()=>{throw 9;},set=v=>{count=v;};Object.defineProperty(o,'x',{get:get,set:set,configurable:true});Object.freeze(o);o.x=7;let d=Object.getOwnPropertyDescriptor(o,'x');count===7 && Object.isFrozen(o) && Object.isSealed(o) && d.get===get && d.set===set && !d.configurable && !Object.hasOwn(d,'writable')",
    );
    check(
        "let o={};Object.defineProperty(o,'x',{get:()=>{throw 9;},configurable:true});Object.seal(o);Object.isFrozen(o) && Object.isSealed(o)",
    );
    check(
        "let p={x:1},o=Object.create(p);Object.freeze(o);p.x=2;o.x===2 && Object.isFrozen(o) && !Object.isFrozen(p) && Object.setPrototypeOf(o,p)===o",
    );
}

#[test]
fn mapped_arguments_freeze_captures_live_values_and_sealing_keeps_aliases() {
    check(
        "function f(a){a=2;Object.freeze(arguments);a=3;return arguments[0]===2 && Object.isFrozen(arguments) && !Object.getOwnPropertyDescriptor(arguments,'0').writable;}f(1)",
    );
    check(
        "function f(a){Object.seal(arguments);a=2;arguments[0]=3;return a===3 && Object.isSealed(arguments) && !Object.isFrozen(arguments);}f(1)",
    );
    check(
        "function f(a){'use strict';Object.freeze(arguments);a=2;return arguments[0]===1 && Object.isFrozen(arguments);}f(1)",
    );
}

#[test]
fn incomplete_intrinsics_cannot_be_reported_as_frozen_or_sealed() {
    check("!Object.isFrozen(Object) && !Object.isSealed(Object) && !Object.isFrozen(globalThis)");
    for operation in ["freeze", "seal"] {
        let mut realm = Realm::default();
        realm.eval("let o=String.prototype,flag=0").unwrap();
        assert!(matches!(
            realm.eval(&format!(
                "try{{Object.{operation}(o);}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(
            realm.eval("flag===0 && !Object.isExtensible(o)"),
            Ok(Value::Boolean(true))
        );
        assert!(matches!(
            realm.eval("Object.isFrozen(o)"),
            Err(Error::Unsupported { .. })
        ));
        assert!(matches!(
            realm.eval("Object.isSealed(o)"),
            Err(Error::Unsupported { .. })
        ));
    }
}

#[test]
fn intrinsic_functions_and_frozen_accessor_edges_survive_collection() {
    for name in ["freeze", "seal", "isFrozen", "isSealed"] {
        check(&format!(
            "let d=Object.getOwnPropertyDescriptor(Object,'{name}');d.writable && !d.enumerable && d.configurable && d.value.name==='{name}' && d.value.length===1"
        ));
        assert!(matches!(
            Realm::default().eval(&format!("new Object.{name}()")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    let mut realm = Realm::default();
    realm.eval("let C=Object;let o=C.create(null,{x:{get:()=>7}});C.freeze(o);delete globalThis.Object").unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.eval("o.x===7 && C.isFrozen(o) && C.isSealed(o) && C.seal(o)===o"),
        Ok(Value::Boolean(true))
    );
}
