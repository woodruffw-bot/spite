use super::*;
use crate::{
    Limits,
    object::{Budget, DescriptorKind, PropertyDescriptor},
};

fn getter(realm: &mut Realm, object: &ObjectHandle, name: &str, source: &str) {
    let Value::Object(get) = realm.eval(source).unwrap() else {
        panic!("getter");
    };
    assert!(
        realm
            .objects
            .define(
                object,
                JsString::from(name),
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(get)),
                        set: Some(None)
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                &mut Budget::new(1_000)
            )
            .unwrap()
    );
}

#[test]
fn constructor_gets_prototype_before_message_and_cause_with_original_receivers() {
    let mut realm = Realm::default();
    let Value::Object(target)=realm.eval("let order='';let prototype={};function F(){} let target=F.bind(null);let message={toString:()=>{order+='m';return 'text';}};let options={marker:23};target").unwrap() else {panic!("target");};
    getter(
        &mut realm,
        &target,
        "prototype",
        "(function(){order+='p';if(this!==target)throw 1;return prototype;})",
    );
    let Value::Object(options) = realm.eval("options").unwrap() else {
        panic!("options");
    };
    getter(
        &mut realm,
        &options,
        "cause",
        "(function(){order+='c';return this.marker;})",
    );
    let message = realm.eval("message").unwrap();
    let Value::Object(error) = realm
        .error_constructor(
            ErrorConstructor::TypeError,
            Some(target),
            vec![message, Value::Object(options)].into_iter(),
            Span::new(0, 0),
        )
        .unwrap()
    else {
        panic!("error");
    };
    assert_eq!(
        realm.eval("order"),
        Ok(Value::String(JsString::from("pmc")))
    );
    let Value::Object(prototype) = realm.eval("prototype").unwrap() else {
        panic!("prototype");
    };
    let object = realm.inspect_object(&error).unwrap();
    assert_eq!(object.prototype(), Some(&prototype));
    assert!(object.is_error());
    assert_eq!(
        object
            .own_property(&JsString::from("cause"))
            .unwrap()
            .as_data()
            .unwrap()
            .value,
        Value::Number(23.0)
    );
}

#[test]
fn message_conversion_precedes_cause_access_and_abrupt_results_skip_later_steps() {
    let mut realm = Realm::default();
    let Value::Object(options) = realm.eval("let order='';let options={};options").unwrap() else {
        panic!("options");
    };
    getter(&mut realm, &options, "cause", "(()=>{order+='c';throw 9;})");
    assert_eq!(
        realm.eval("Error({toString:()=>{order+='m';throw 8;}},options)"),
        Err(Error::Thrown(Value::Number(8.0)))
    );
    assert_eq!(realm.eval("order"), Ok(Value::String(JsString::from("m"))));
    assert_eq!(
        realm.eval("order='';Error({toString:()=>{order+='m';return 'text';}},options)"),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    assert_eq!(realm.eval("order"), Ok(Value::String(JsString::from("mc"))));
    assert_eq!(realm.eval("after=7"), Ok(Value::Number(7.0)));
}

#[test]
fn generic_stringification_orders_gets_and_conversions_even_with_an_empty_name() {
    let mut realm = Realm::default();
    let Value::Object(object) = realm.eval("let order='';let object={};object").unwrap() else {
        panic!("object");
    };
    getter(
        &mut realm,
        &object,
        "name",
        "(()=>{order+='n';return {toString:()=>{order+='N';return 'Kind';}};})",
    );
    getter(
        &mut realm,
        &object,
        "message",
        "(()=>{order+='m';return {toString:()=>{order+='M';return 'detail';}};})",
    );
    assert_eq!(
        realm.eval("Error.prototype.toString.call(object)"),
        Ok(Value::String(JsString::from("Kind: detail")))
    );
    assert_eq!(
        realm.eval("order"),
        Ok(Value::String(JsString::from("nNmM")))
    );
    getter(
        &mut realm,
        &object,
        "name",
        "(()=>{order+='n';return {toString:()=>{throw 8;}};})",
    );
    assert_eq!(
        realm.eval("order='';Error.prototype.toString.call(object)"),
        Err(Error::Thrown(Value::Number(8.0)))
    );
    assert_eq!(realm.eval("order"), Ok(Value::String(JsString::from("n"))));
    getter(&mut realm, &object, "name", "(()=>{order+='n';return '';})");
    assert_eq!(
        realm.eval("order='';Error.prototype.toString.call(object)"),
        Ok(Value::String(JsString::from("detail")))
    );
    assert_eq!(
        realm.eval("order"),
        Ok(Value::String(JsString::from("nmM")))
    );
}

#[test]
fn error_data_survives_prototype_replacement_and_is_not_inherited() {
    let mut realm = Realm::default();
    let Value::Object(object) = realm.eval("let e=TypeError('text');e").unwrap() else {
        panic!("error");
    };
    assert!(
        realm
            .objects
            .set_prototype(&object, None, &mut Budget::new(1_000))
            .unwrap()
    );
    assert_eq!(realm.eval("Error.isError(e) && ({}).toString.call(e)==='[object Error]' && !Error.isError({__proto__:e})"),Ok(Value::Boolean(true)));
}

#[test]
fn message_output_and_allocation_limits_remain_host_aborts() {
    let mut realm = Realm::default();
    realm
        .eval("let flag=0;let e=Error('abc');let stringify=Error.prototype.toString;")
        .unwrap();
    realm.limits.max_string_units = 4;
    for source in ["Error(123456n)", "stringify.call(e)"] {
        assert!(matches!(
            realm.eval(&format!(
                "try{{{source};}}catch{{flag=1;}}finally{{flag=2;}}"
            )),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
    assert_eq!(
        realm.eval("Error('ok') instanceof Error"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_heap_entries: crate::test_support::REALM_ENTRIES,
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{Error();}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn materializing_a_builtin_exception_can_abort_without_entering_pending_handlers() {
    // The try block consumes the last slot; creating the catch value must fail.
    let mut realm = Realm::new(Limits {
        max_heap_entries: crate::test_support::REALM_ENTRIES + 1,
        ..Limits::default()
    });
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("try{+1n;}catch(e){flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));

    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    realm.limits.max_string_units = 16;
    assert!(matches!(
        realm.eval("try{+1n;}catch(e){flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("1+2"), Ok(Value::Number(3.0)));
}

#[test]
fn host_property_reads_run_inherited_getters_with_bounded_work_and_restore_state() {
    let mut realm = Realm::default();
    let Value::Object(prototype) = realm
        .eval("let flag=0;let parent={};let child={__proto__:parent,marker:7};parent")
        .unwrap()
    else {
        panic!("prototype");
    };
    let child = realm.eval("child").unwrap();
    getter(
        &mut realm,
        &prototype,
        "x",
        "(function(){flag=1;return this.marker;})",
    );
    assert_eq!(
        realm.read_property(&child, &JsString::from("x")),
        Ok(Value::Number(7.0))
    );
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    getter(
        &mut realm,
        &prototype,
        "x",
        "(function(){'use strict';throw 9;})",
    );
    assert_eq!(
        realm.read_property(&child, &JsString::from("x")),
        Err(Error::Thrown(Value::Number(9.0)))
    );
    assert_eq!(realm.eval("after=3"), Ok(Value::Number(3.0)));
    getter(
        &mut realm,
        &prototype,
        "x",
        "(function(){try{while(true){}}finally{flag=2;}})",
    );
    realm.limits.max_steps = Some(1_000);
    assert!(matches!(
        realm.read_property(&child, &JsString::from("x")),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(1.0)));
    assert_eq!(
        realm.read_property(&child, &JsString::from("marker")),
        Ok(Value::Number(7.0))
    );
}
