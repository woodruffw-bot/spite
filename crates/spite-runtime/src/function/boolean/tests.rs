use super::*;
use crate::object::{Budget, DescriptorKind, PropertyDescriptor};

#[test]
fn primitive_accessor_receivers_are_preserved_and_non_strict_accessors_box_them() {
    let mut realm = Realm::default();
    realm.eval("let received, assigned;").unwrap();
    let prototype = realm.intrinsics.as_ref().unwrap().boolean.prototype.clone();
    for strict in [false, true] {
        let directive = if strict { "'use strict';" } else { "" };
        let Value::Object(get) = realm
            .eval(&format!("(function(){{{directive}return this;}})"))
            .unwrap()
        else {
            panic!("getter")
        };
        let Value::Object(set) = realm
            .eval(&format!(
                "(function(value){{{directive}received=this;assigned=value;}})"
            ))
            .unwrap()
        else {
            panic!("setter")
        };
        assert!(
            realm
                .objects
                .define(
                    &prototype,
                    JsString::from("access"),
                    PropertyDescriptor {
                        kind: DescriptorKind::Accessor {
                            get: Some(Some(get)),
                            set: Some(Some(set))
                        },
                        enumerable: Some(false),
                        configurable: Some(true),
                    },
                    &mut Budget::new(1000)
                )
                .unwrap()
        );
        let getter_test = if strict {
            "true.access===true && false.access===false"
        } else {
            "let a=true.access,b=false.access; a instanceof Boolean && a.valueOf()===true && b.valueOf()===false && a!==true.access"
        };
        assert_eq!(realm.eval(getter_test), Ok(Value::Boolean(true)));
        for prefix in ["", "'use strict';"] {
            realm.eval(&format!("{prefix}false.access=7")).unwrap();
            let receiver_test = if strict {
                "received===false && assigned===7"
            } else {
                "received instanceof Boolean && received.valueOf()===false && assigned===7"
            };
            assert_eq!(realm.eval(receiver_test), Ok(Value::Boolean(true)));
        }
        assert_eq!(
            realm.eval("var boxed=new Boolean; boxed.access===boxed"),
            Ok(Value::Boolean(true))
        );
        realm.eval("boxed.access=9").unwrap();
        assert_eq!(
            realm.eval("received===boxed && assigned===9"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn boolean_slot_is_independent_of_prototype_identity() {
    let mut realm = Realm::default();
    let Value::Object(wrapper) = realm
        .eval("let wrapped=new Boolean(true); wrapped")
        .unwrap()
    else {
        panic!("wrapper")
    };
    assert!(
        realm
            .objects
            .set_prototype(&wrapper, None, &mut Budget::new(100))
            .unwrap()
    );
    assert_eq!(
        realm.eval("Boolean.prototype.valueOf.call(wrapped)"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm.eval("({}).toString.call(wrapped)"),
        Ok(Value::String(JsString::from("[object Boolean]")))
    );
}

#[test]
fn generated_strings_obey_limits_and_restore_call_state_after_failure() {
    let mut realm = Realm::default();
    realm
        .eval("Boolean.prototype.s=Boolean.prototype.toString; let flag=0")
        .unwrap();
    realm.limits.max_string_units = 4;
    assert_eq!(
        realm.eval("true.s()"),
        Ok(Value::String(JsString::from("true")))
    );
    assert!(matches!(
        realm.eval("try{false.s();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(
        realm.eval("true.s()"),
        Ok(Value::String(JsString::from("true")))
    );
}
