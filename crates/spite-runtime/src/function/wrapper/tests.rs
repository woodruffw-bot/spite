use super::*;
use crate::object::{Budget, DescriptorKind, PropertyDescriptor};
use spite_core::JsString;

#[test]
fn number_accessor_receivers_preserve_negative_zero_and_box_only_for_non_strict_calls() {
    let mut realm = Realm::default();
    realm.eval("let received, assigned").unwrap();
    let prototype = realm.intrinsics.as_ref().unwrap().number.prototype.clone();
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
        realm
            .objects
            .define(
                &prototype,
                JsString::from("access"),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(get)),
                        set: Some(Some(set)),
                    },
                    configurable: Some(true),
                    ..Default::default()
                },
                &mut Budget::new(1000),
            )
            .unwrap();
        let source = if strict {
            "1/(-0).access===-Infinity"
        } else {
            "(-0).access instanceof Number && 1/(-0).access.valueOf()===-Infinity"
        };
        assert_eq!(realm.eval(source), Ok(Value::Boolean(true)));
        realm.eval("'use strict'; (-0).access=7").unwrap();
        let source = if strict {
            "1/received===-Infinity && typeof received==='number' && assigned===7"
        } else {
            "received instanceof Number && 1/received.valueOf()===-Infinity && assigned===7"
        };
        assert_eq!(realm.eval(source), Ok(Value::Boolean(true)));
    }
}

#[test]
fn number_slots_and_tags_do_not_depend_on_the_current_prototype() {
    let mut realm = Realm::default();
    let Value::Object(handle) = realm.eval("let n=new Number(-0); n").unwrap() else {
        panic!("wrapper")
    };
    realm
        .objects
        .set_prototype(&handle, None, &mut Budget::new(100))
        .unwrap();
    assert_eq!(
        realm.eval("1/Number.prototype.valueOf.call(n)"),
        Ok(Value::Number(f64::NEG_INFINITY))
    );
    assert_eq!(
        realm.eval("({}).toString.call(n)"),
        Ok(Value::String(JsString::from("[object Number]")))
    );
}

#[test]
fn number_string_limits_abort_without_running_catch_or_finally() {
    let mut realm = Realm::default();
    realm
        .eval("Number.prototype.s=Number.prototype.toString; let flag=0;")
        .unwrap();
    realm.limits.max_string_units = Some(3);
    assert_eq!(
        realm.eval("(100).s()"),
        Ok(Value::String(JsString::from("100")))
    );
    assert!(matches!(
        realm.eval("try{(1000).s();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
