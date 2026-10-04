use super::*;
use crate::object::Budget;

#[test]
fn object_prototype_is_immutable_and_function_prototype_is_callable() {
    let mut realm = Realm::default();
    realm.eval("({})").unwrap();
    let intrinsics = realm.intrinsics.as_ref().unwrap();
    let object = intrinsics.object_prototype.clone();
    let function = intrinsics.function_prototype.clone();
    assert!(realm.objects.inspect(&object).unwrap().is_extensible());
    assert!(
        realm
            .objects
            .set_prototype(&object, None, &mut Budget::new(100))
            .unwrap()
    );
    assert!(
        !realm
            .objects
            .set_prototype(&object, Some(&function), &mut Budget::new(100))
            .unwrap()
    );
    let independent = realm.objects.create(None).unwrap();
    assert!(
        !realm
            .objects
            .set_prototype(&object, Some(&independent), &mut Budget::new(100))
            .unwrap()
    );
    assert_eq!(
        realm.call(
            Value::Object(function),
            Value::Null,
            vec![Value::Number(7.0)],
            Span::new(0, 0)
        ),
        Ok(Value::Undefined)
    );
}

#[test]
fn restricted_descriptors_share_a_frozen_nonextensible_thrower() {
    let mut realm = Realm::default();
    realm.eval("let f = ({}).toString").unwrap();
    let intrinsics = realm.intrinsics.as_ref().unwrap();
    let function = intrinsics.function_prototype.clone();
    let thrower = intrinsics.throw_type_error.clone();
    let object = realm.objects.inspect(&thrower).unwrap();
    assert!(object.is_callable());
    assert!(!object.is_extensible());
    assert_eq!(object.prototype(), Some(&function));
    for (name, expected) in [
        ("name", Value::String(JsString::from(""))),
        ("length", Value::Number(0.0)),
    ] {
        let property = object
            .own_property(&JsString::from(name))
            .unwrap()
            .as_data()
            .unwrap();
        assert_eq!(property.value, expected);
        assert!(!property.writable && !property.enumerable && !property.configurable);
    }
    for name in ["caller", "arguments"] {
        let crate::object::Property::Accessor(property) = realm
            .objects
            .inspect(&function)
            .unwrap()
            .own_property(&JsString::from(name))
            .unwrap()
        else {
            panic!("accessor")
        };
        assert_eq!(property.get.as_ref(), Some(&thrower));
        assert_eq!(property.set.as_ref(), Some(&thrower));
        assert!(!property.enumerable && property.configurable);
        assert!(
            realm
                .objects
                .delete(&function, &JsString::from(name), &mut Budget::new(1000))
                .unwrap()
        );
        assert_eq!(realm.eval(&format!("f.{name}")), Ok(Value::Undefined));
        assert_eq!(
            realm.eval(&format!("'{name}' in f")),
            Ok(Value::Boolean(false))
        );
    }
    // The intrinsic identity is retained even after deleting all accessor edges.
    assert_eq!(
        realm.collect(usize::MAX).unwrap().live,
        crate::test_support::REALM_ENTRIES
    );
    assert!(realm.objects.inspect(&thrower).is_ok());
}

#[test]
fn call_forwarding_is_iterative_and_budgeted() {
    let mut realm = Realm::default();
    realm.eval("({})").unwrap();
    let intrinsics = realm.intrinsics.as_ref().unwrap();
    let call = Value::Object(intrinsics.function_call.clone());
    let tag = Value::Object(intrinsics.object_to_string.clone());
    let mut arguments = vec![call.clone(); 10_000];
    arguments.extend([tag.clone(), Value::Null]);
    assert_eq!(
        realm.call(call.clone(), call.clone(), arguments, Span::new(0, 0)),
        Ok(Value::String(JsString::from("[object Null]")))
    );
    realm.remaining_steps = Some(3);
    assert!(matches!(
        realm.call(
            call.clone(),
            call.clone(),
            vec![call.clone(); 10],
            Span::new(0, 0)
        ),
        Err(Error::Limit { .. })
    ));
    // An abrupt host exit leaves no pending call state behind.
    assert_eq!(
        realm.eval("({}).toString.call(null)"),
        Ok(Value::String(JsString::from("[object Null]")))
    );
}

#[test]
fn native_source_does_not_read_the_name_property() {
    let mut realm = Realm::default();
    realm.eval("let f = ({}).valueOf").unwrap();
    let intrinsics = realm.intrinsics.as_ref().unwrap();
    let function = intrinsics.object_value_of.clone();
    let thrower = intrinsics.throw_type_error.clone();
    realm
        .objects
        .define(
            &function,
            JsString::from("name"),
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(thrower)),
                    set: Some(None),
                },
                ..Default::default()
            },
            &mut Budget::new(1000),
        )
        .unwrap();
    assert_eq!(
        realm.eval("f.toString()"),
        Ok(Value::String(JsString::from(
            "function valueOf() { [native code] }"
        )))
    );
    assert!(matches!(
        realm.eval("f.name"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn apply_reads_every_index_with_the_list_receiver_before_calling_the_target() {
    let mut realm = Realm::default();
    let Value::Object(base) = realm
        .eval("let base = {}; let args = {__proto__: base, length: 1}; base")
        .unwrap()
    else {
        panic!("object")
    };
    let intrinsics = realm.intrinsics.as_ref().unwrap();
    let value_of = intrinsics.object_value_of.clone();
    let thrower = intrinsics.throw_type_error.clone();
    realm
        .objects
        .define(
            &base,
            JsString::from("0"),
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(value_of)),
                    set: Some(None),
                },
                ..Default::default()
            },
            &mut Budget::new(1000),
        )
        .unwrap();
    assert_eq!(
        realm.eval("({}).valueOf.call.apply(({}).valueOf, args) === args"),
        Ok(Value::Boolean(true))
    );
    realm
        .objects
        .define(
            &base,
            JsString::from("1"),
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(thrower)),
                    set: Some(None),
                },
                ..Default::default()
            },
            &mut Budget::new(1000),
        )
        .unwrap();
    // Object.prototype.toString ignores its arguments, but apply must still
    // read the full list and propagate a later getter's exception first.
    assert!(matches!(
        realm.eval("args.length = 2; ({}).toString.apply(null, args)"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}
