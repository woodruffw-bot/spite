use super::*;
use crate::{Value, function::Builtin};

fn accessor(
    get: Option<Option<Handle>>,
    set: Option<Option<Handle>>,
    configurable: Option<bool>,
) -> PropertyDescriptor {
    PropertyDescriptor {
        kind: DescriptorKind::Accessor { get, set },
        configurable,
        enumerable: None,
    }
}

fn getter(object: &OrdinaryObject, key: &JsString) -> Option<Handle> {
    match object.own_property(key).unwrap() {
        Property::Accessor(property) => property.get.clone(),
        _ => panic!("accessor"),
    }
}

#[test]
fn kinds_switch_only_when_configurable_and_preserve_order_and_common_attributes() {
    let mut objects = Objects::new(3, 3);
    let object = objects.create(None).unwrap();
    let function = objects
        .create_builtin(&object, Builtin::ObjectValueOf)
        .unwrap();
    let mut budget = Budget::new(1000);
    let x = JsString::from("x");
    objects
        .define(
            &object,
            x.clone(),
            DataDescriptor {
                value: Some(Value::Number(7.0)),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            },
            &mut budget,
        )
        .unwrap();
    objects
        .define(
            &object,
            JsString::from("y"),
            DataDescriptor::default(),
            &mut budget,
        )
        .unwrap();
    assert!(
        objects
            .define(
                &object,
                x.clone(),
                accessor(Some(Some(function.clone())), None, None),
                &mut budget
            )
            .unwrap()
    );
    let record = objects.inspect(&object).unwrap();
    assert_eq!(getter(record, &x), Some(function.clone()));
    assert!(record.own_property(&x).unwrap().enumerable());
    assert!(record.own_property(&x).unwrap().configurable());
    assert_eq!(
        record.own_keys(),
        [JsString::from("x"), JsString::from("y")]
    );
    // An empty data/accessor kind is generic, not a request to change the kind.
    for kind in [
        DescriptorKind::Generic,
        DescriptorKind::Data {
            value: None,
            writable: None,
        },
        DescriptorKind::Accessor {
            get: None,
            set: None,
        },
    ] {
        assert!(
            objects
                .define(
                    &object,
                    x.clone(),
                    PropertyDescriptor {
                        kind,
                        ..Default::default()
                    },
                    &mut budget
                )
                .unwrap()
        );
        assert_eq!(
            getter(objects.inspect(&object).unwrap(), &x),
            Some(function.clone())
        );
    }
    assert!(
        objects
            .define(
                &object,
                x.clone(),
                DataDescriptor {
                    value: Some(Value::Null),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    let data = objects
        .inspect(&object)
        .unwrap()
        .own_property(&x)
        .unwrap()
        .as_data()
        .unwrap();
    assert_eq!(data.value, Value::Null);
    assert!(!data.writable && data.enumerable && data.configurable);
    assert!(
        objects
            .define(
                &object,
                x.clone(),
                accessor(None, Some(None), Some(false)),
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(getter(objects.inspect(&object).unwrap(), &x), None);
    assert!(
        !objects
            .define(
                &object,
                x.clone(),
                DataDescriptor {
                    value: Some(Value::Undefined),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    assert!(!objects.delete(&object, &x, &mut budget).unwrap());
}

#[test]
fn frozen_accessors_require_same_getter_and_setter_and_reject_changes_atomically() {
    let mut objects = Objects::new(3, 1);
    let object = objects.create(None).unwrap();
    let first = objects
        .create_builtin(&object, Builtin::ObjectValueOf)
        .unwrap();
    let second = objects
        .create_builtin(&object, Builtin::ObjectToString)
        .unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("x");
    assert!(
        objects
            .define(
                &object,
                key.clone(),
                accessor(Some(Some(first.clone())), Some(Some(second.clone())), None),
                &mut budget
            )
            .unwrap()
    );
    let before = objects.get_own(&object, &key, &mut budget).unwrap();
    assert!(
        objects
            .define(
                &object,
                key.clone(),
                accessor(Some(Some(first.clone())), Some(Some(second.clone())), None),
                &mut budget
            )
            .unwrap()
    );
    for descriptor in [
        accessor(Some(None), None, None),
        accessor(Some(Some(second)), None, None),
        accessor(None, Some(None), None),
        accessor(None, Some(Some(first)), None),
        PropertyDescriptor {
            configurable: Some(true),
            ..Default::default()
        },
        PropertyDescriptor {
            enumerable: Some(true),
            ..Default::default()
        },
    ] {
        assert!(
            !objects
                .define(&object, key.clone(), descriptor, &mut budget)
                .unwrap()
        );
        assert_eq!(objects.get_own(&object, &key, &mut budget).unwrap(), before);
    }
}

#[test]
fn accessor_actions_preserve_receiver_rules_and_do_not_invoke_during_presence_checks() {
    let mut objects = Objects::new(4, 2);
    let base = objects.create(None).unwrap();
    let child = objects.create(Some(&base)).unwrap();
    let function = objects
        .create_builtin(&base, Builtin::ObjectValueOf)
        .unwrap();
    let receiver = objects.create(None).unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("x");
    objects
        .define(
            &base,
            key.clone(),
            accessor(
                Some(Some(function.clone())),
                Some(Some(function.clone())),
                Some(true),
            ),
            &mut budget,
        )
        .unwrap();
    assert!(objects.has(&child, &key, &mut budget).unwrap());
    assert_eq!(
        objects.get(&child, &key, &mut budget).unwrap(),
        GetAction::Call(function.clone())
    );
    for target in [Some(&child), Some(&receiver), None] {
        assert_eq!(
            objects
                .set(&child, key.clone(), Value::Number(9.0), target, &mut budget)
                .unwrap(),
            SetAction::Call(function.clone())
        );
    }
    assert!(!objects.has_own(&child, &key, &mut budget).unwrap());
    objects
        .define(
            &base,
            key.clone(),
            accessor(Some(None), Some(None), None),
            &mut budget,
        )
        .unwrap();
    assert_eq!(
        objects.get(&child, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Undefined)
    );
    assert_eq!(
        objects
            .set(&child, key.clone(), Value::Null, Some(&child), &mut budget)
            .unwrap(),
        SetAction::Done(false)
    );
    // A writable inherited data property must not invoke an accessor on Receiver.
    objects
        .define(
            &receiver,
            key.clone(),
            accessor(None, Some(Some(function)), None),
            &mut budget,
        )
        .unwrap();
    objects
        .define(
            &base,
            key.clone(),
            DataDescriptor {
                writable: Some(true),
                ..Default::default()
            },
            &mut budget,
        )
        .unwrap();
    assert_eq!(
        objects
            .set(&base, key, Value::Null, Some(&receiver), &mut budget)
            .unwrap(),
        SetAction::Done(false)
    );
}

#[test]
fn accessor_edges_are_validated_traced_and_removed_on_kind_change() {
    let mut objects = Objects::new(4, 1);
    let object = objects.create(None).unwrap();
    let first = objects
        .create_builtin(&object, Builtin::ObjectValueOf)
        .unwrap();
    let second = objects
        .create_builtin(&object, Builtin::ObjectToString)
        .unwrap();
    let stale = objects.create(None).unwrap();
    let foreign = Objects::new(1, 0).create(None).unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("x");
    objects
        .define(
            &object,
            key.clone(),
            accessor(Some(Some(first)), Some(Some(second)), Some(true)),
            &mut budget,
        )
        .unwrap();
    assert_eq!(objects.collect([&object], 100).unwrap().live, 3);
    let before = objects.get_own(&object, &key, &mut budget).unwrap();
    for (handle, error) in [
        (foreign, Error::Heap(spite_heap::Error::ForeignHandle)),
        (stale, Error::Heap(spite_heap::Error::StaleHandle)),
        (object.clone(), Error::InvalidAccessor),
    ] {
        assert_eq!(
            objects.define(
                &object,
                key.clone(),
                accessor(Some(Some(handle)), None, None),
                &mut budget
            ),
            Err(error)
        );
        assert_eq!(objects.get_own(&object, &key, &mut budget).unwrap(), before);
    }
    let mut succeeded = false;
    for work in 0..100 {
        match objects.define(
            &object,
            key.clone(),
            accessor(Some(None), None, None),
            &mut Budget::new(work),
        ) {
            Err(Error::WorkLimit) => {
                assert_eq!(objects.get_own(&object, &key, &mut budget).unwrap(), before)
            }
            Ok(true) => {
                succeeded = true;
                break;
            }
            result => panic!("unexpected {result:?}"),
        }
    }
    assert!(succeeded);
    objects
        .define(
            &object,
            key,
            DataDescriptor {
                value: Some(Value::Null),
                ..Default::default()
            },
            &mut budget,
        )
        .unwrap();
    assert_eq!(objects.collect([&object], 100).unwrap().reclaimed, 2);
}
