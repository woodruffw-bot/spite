use super::*;
use crate::object::{Budget, DescriptorKind, PropertyDescriptor};

#[test]
fn distinct_new_target_selects_a_prototype_and_ignores_the_value_argument() {
    let mut realm = Realm::default();
    let Value::Object(target) = realm
        .eval("function F(){};let p={};F.prototype=p;let input={toString:()=>{throw 1;}};F")
        .unwrap()
    else {
        panic!("target");
    };
    let input = realm.eval("input").unwrap();
    let value = realm
        .object_constructor(Some(target.clone()), input.clone(), Span::new(0, 0))
        .unwrap();
    assert_ne!(value, input);
    let Value::Object(object) = value else {
        panic!("object");
    };
    let Value::Object(prototype) = realm.eval("p").unwrap() else {
        panic!("prototype");
    };
    assert_eq!(
        realm.inspect_object(&object).unwrap().prototype(),
        Some(&prototype)
    );
    realm.eval("F.prototype=null").unwrap();
    let Value::Object(object) = realm
        .object_constructor(Some(target), Value::Null, Span::new(0, 0))
        .unwrap()
    else {
        panic!("object");
    };
    assert_eq!(
        realm.inspect_object(&object).unwrap().prototype(),
        Some(&realm.intrinsics.as_ref().unwrap().object_prototype)
    );
}

#[test]
fn own_property_predicates_do_not_invoke_accessors() {
    let mut realm = Realm::default();
    let Value::Object(object) = realm.eval("let calls=0;let o={};o").unwrap() else {
        panic!("object");
    };
    let Value::Object(getter) = realm.eval("(()=>{calls++;throw 9;})").unwrap() else {
        panic!("getter");
    };
    realm
        .objects
        .define(
            &object,
            JsString::from("x"),
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(getter)),
                    set: Some(None),
                },
                enumerable: Some(false),
                configurable: Some(true),
            },
            &mut Budget::new(100),
        )
        .unwrap();
    assert_eq!(
        realm.eval("o.hasOwnProperty('x') && !o.propertyIsEnumerable('x') && calls===0"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("o.x"), Err(Error::Thrown(Value::Number(9.0))));
    assert_eq!(realm.eval("calls"), Ok(Value::Number(1.0)));
}
