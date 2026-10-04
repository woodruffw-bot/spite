//! Object construction and the standard Object.prototype methods.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn construction_distinguishes_nullish_object_and_primitive_arguments() {
    check(
        "let a=Object(),b=new Object(null),c=Object(undefined);a!==b && b!==c && a instanceof Object && Object.prototype.isPrototypeOf(a) && a.constructor===Object",
    );
    check(
        "let o={valueOf:()=>{throw 1;},toString:()=>{throw 2;}};Object(o)===o && new Object(o)===o && Object.call(null,o)===o && new (Object.bind(null,o))()===o",
    );
    check(
        "let b=Object(false),n=new Object(-0);b instanceof Boolean && b.valueOf()===false && n instanceof Number && 1/n.valueOf()===-Infinity && n!==Object(-0)",
    );
    check("let effect=0;Object(null,effect=1);effect===1");
    for source in ["Object('x')", "new Object(1n)"] {
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
fn object_constructor_and_prototype_methods_have_standard_descriptors() {
    let mut realm = Realm::default();
    let Value::Object(constructor) = realm.eval("Object").unwrap() else {
        panic!("Object");
    };
    let object = realm.inspect_object(&constructor).unwrap();
    assert!(object.is_callable() && object.is_constructor());
    let descriptor = object
        .own_property(&JsString::from("prototype"))
        .unwrap()
        .as_data()
        .unwrap();
    assert!(!descriptor.writable && !descriptor.enumerable && !descriptor.configurable);
    assert_eq!(
        realm.eval(
            "Object.name==='Object' && Object.length===1 && Object.prototype.constructor===Object"
        ),
        Ok(Value::Boolean(true))
    );
    let Value::Object(prototype) = realm.eval("Object.prototype").unwrap() else {
        panic!("prototype");
    };
    assert!(
        realm
            .inspect_object(&prototype)
            .unwrap()
            .prototype()
            .is_none()
    );
    for (name, length) in [
        ("hasOwnProperty", 1.0),
        ("propertyIsEnumerable", 1.0),
        ("isPrototypeOf", 1.0),
        ("toLocaleString", 0.0),
    ] {
        let descriptor = realm
            .inspect_object(&prototype)
            .unwrap()
            .own_property(&JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert!(descriptor.writable && !descriptor.enumerable && descriptor.configurable);
        let Value::Object(function) = &descriptor.value else {
            panic!("method");
        };
        let function = realm.inspect_object(function).unwrap();
        assert!(function.is_callable() && !function.is_constructor());
        assert_eq!(
            function
                .own_property(&JsString::from("length"))
                .unwrap()
                .as_data()
                .unwrap()
                .value,
            Value::Number(length)
        );
        assert_eq!(
            function
                .own_property(&JsString::from("name"))
                .unwrap()
                .as_data()
                .unwrap()
                .value,
            Value::String(JsString::from(name))
        );
    }
}

#[test]
fn property_predicates_use_own_descriptors_and_convert_keys_before_receivers() {
    check(
        "let o={__proto__:{inherited:1},own:2,1:3};o.hasOwnProperty('own') && o.propertyIsEnumerable('own') && o.hasOwnProperty(1n) && !o.hasOwnProperty('inherited') && !o.propertyIsEnumerable('inherited')",
    );
    check(
        "Object.prototype.hasOwnProperty('constructor') && !Object.prototype.propertyIsEnumerable('constructor') && !({}).hasOwnProperty('constructor')",
    );
    check(
        "!true.hasOwnProperty('valueOf') && !Object.prototype.propertyIsEnumerable.call(1,'valueOf')",
    );
    for method in ["hasOwnProperty", "propertyIsEnumerable"] {
        let mut realm = Realm::default();
        assert_eq!(realm.eval(&format!("let order='';let key={{toString:()=>{{order+='s';return 'x';}},valueOf:()=>{{throw 1;}}}};Object.prototype.{method}.call({{x:1}},key) && order==='s'")),Ok(Value::Boolean(true)));
        assert_eq!(
            realm.eval(&format!(
                "Object.prototype.{method}.call(null,{{toString:()=>{{throw 7;}}}})"
            )),
            Err(Error::Thrown(Value::Number(7.0)))
        );
        assert!(matches!(
            realm.eval(&format!("Object.prototype.{method}.call(undefined,'x')")),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
        assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
    }
}

#[test]
fn prototype_predicate_checks_argument_before_receiver_and_walks_ancestors() {
    for argument in ["undefined", "null", "true", "1", "1n", "'x'"] {
        check(&format!(
            "Object.prototype.isPrototypeOf.call(null,{argument})===false"
        ));
    }
    assert!(matches!(
        Realm::default().eval("Object.prototype.isPrototypeOf.call(null,{})"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
    check(
        "let p={},child={__proto__:p},grandchild={__proto__:child};p.isPrototypeOf(child) && p.isPrototypeOf(grandchild) && !p.isPrototypeOf(p) && !child.isPrototypeOf(p) && !p.isPrototypeOf({__proto__:null})",
    );
    check(
        "Object.prototype.isPrototypeOf(new TypeError) && Error.prototype.isPrototypeOf(new TypeError) && !TypeError.prototype.isPrototypeOf(new Error)",
    );
}

#[test]
fn locale_string_invokes_the_current_method_without_forwarding_reserved_arguments() {
    check(
        "let count=0;let o={toString:function(){'use strict';count++;return this;}};o.toLocaleString()===o && count===1",
    );
    check(
        "let effect=0;let o={toString:function(){return arguments.length;}};o.toLocaleString(effect=1,effect=2)===0 && effect===2",
    );
    check(
        "Object.prototype.toLocaleString.call(true)==='true' && Object.prototype.toLocaleString.call(12)==='12'",
    );
    for source in [
        "Object.prototype.toLocaleString.call(null)",
        "({toString:3}).toLocaleString()",
    ] {
        assert!(matches!(
            Realm::default().eval(source),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
    assert_eq!(
        Realm::default().eval("({toString:()=>{throw 9;}}).toLocaleString()"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
}

#[test]
fn intrinsic_roots_survive_deleted_bindings_and_missing_statics_remain_gaps() {
    let mut realm = Realm::default();
    realm.eval("let C=Object;delete globalThis.Object").unwrap();
    realm.collect(100_000).unwrap();
    assert_eq!(
        realm.eval(
            "C().constructor===C && C.prototype.hasOwnProperty('constructor') && C(1).valueOf()===1"
        ),
        Ok(Value::Boolean(true))
    );
    for source in [
        "Object.create",
        "Object.getOwnPropertyDescriptors",
        "Object.create=1",
        "delete Object.create",
        "Object.hasOwnProperty('create')",
        "Object.propertyIsEnumerable('freeze')",
        "globalThis.hasOwnProperty('Array')",
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
