//! Object identity, conversion boundaries, and tracing of property value edges.

use spite_bigint::Budget as IntegerBudget;
use spite_core::JsString;
use spite_heap::Error as HeapError;
use spite_runtime::{
    ConversionError, Value,
    object::{Budget, DataDescriptor, Error, Objects},
};

fn data(value: Value) -> DataDescriptor {
    DataDescriptor {
        value: Some(value),
        writable: Some(true),
        enumerable: Some(true),
        configurable: Some(true),
    }
}

#[test]
fn object_identity_and_truthiness_do_not_coerce() {
    let mut objects = Objects::new(2, 0);
    let a = Value::Object(objects.create(None).unwrap());
    let b = Value::Object(objects.create(None).unwrap());
    let foreign = Value::Object(Objects::new(1, 0).create(None).unwrap());
    assert!(a.to_boolean());
    assert!(a.strictly_equal(&a.clone()));
    assert!(a.same_value(&a.clone()));
    for other in [
        b,
        foreign,
        Value::Null,
        Value::Undefined,
        Value::Number(0.0),
    ] {
        assert!(!a.strictly_equal(&other));
        assert!(!a.same_value(&other));
    }
    assert_eq!(a.to_number(), Err(ConversionError::ObjectNeedsContext));
    assert_eq!(
        a.to_js_string(&mut IntegerBudget::new(100, 100)),
        Err(ConversionError::ObjectNeedsContext)
    );
}

#[test]
fn object_valued_properties_trace_cycles_without_retaining_unreachable_objects() {
    let mut objects = Objects::new(4, 2);
    let a = objects.create(None).unwrap();
    let b = objects.create(None).unwrap();
    let prototype = objects.create(None).unwrap();
    let child = objects.create(Some(&prototype)).unwrap();
    let mut budget = Budget::new(1000);
    for (target, key, value) in [
        (&a, "self", &a),
        (&a, "b", &b),
        (&b, "a", &a),
        (&b, "child", &child),
    ] {
        assert!(
            objects
                .define(
                    target,
                    JsString::from(key),
                    data(Value::Object(value.clone())),
                    &mut budget
                )
                .unwrap()
        );
    }
    let root = objects.root(&a, &mut budget).unwrap();
    assert_eq!(objects.collect([], 100).unwrap().live, 4);
    assert_eq!(
        objects.get(&a, &JsString::from("b"), &mut budget).unwrap(),
        Value::Object(b.clone())
    );
    drop(root);
    assert_eq!(objects.collect([], 100).unwrap().reclaimed, 4);
    for handle in [&a, &b, &prototype, &child] {
        assert!(matches!(
            objects.inspect(handle),
            Err(Error::Heap(HeapError::StaleHandle))
        ));
    }
}

#[test]
fn frozen_object_values_require_the_same_identity() {
    let mut objects = Objects::new(2, 1);
    let a = objects.create(None).unwrap();
    let b = objects.create(None).unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("x");
    let descriptor = DataDescriptor {
        value: Some(Value::Object(a.clone())),
        ..Default::default()
    };
    assert!(
        objects
            .define(&a, key.clone(), descriptor.clone(), &mut budget)
            .unwrap()
    );
    assert!(
        objects
            .define(&a, key.clone(), descriptor, &mut budget)
            .unwrap()
    );
    assert!(
        !objects
            .define(
                &a,
                key.clone(),
                DataDescriptor {
                    value: Some(Value::Object(b)),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(
        objects.get(&a, &key, &mut budget).unwrap(),
        Value::Object(a.clone())
    );
    assert_eq!(objects.collect([&a], 100).unwrap().reclaimed, 1);
}

#[test]
fn foreign_and_stale_value_edges_are_rejected_before_mutation() {
    let mut objects = Objects::new(2, 1);
    let a = objects.create(None).unwrap();
    let stale = objects.create(None).unwrap();
    objects.collect([&a], 100).unwrap();
    let foreign = Objects::new(1, 0).create(None).unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("x");
    assert!(
        objects
            .define(&a, key.clone(), data(Value::Null), &mut budget)
            .unwrap()
    );
    for (handle, error) in [
        (stale, HeapError::StaleHandle),
        (foreign, HeapError::ForeignHandle),
    ] {
        assert_eq!(
            objects.define(
                &a,
                key.clone(),
                data(Value::Object(handle.clone())),
                &mut budget
            ),
            Err(Error::Heap(error))
        );
        assert_eq!(
            objects.set(
                &a,
                key.clone(),
                Value::Object(handle),
                Some(&a),
                &mut budget
            ),
            Err(Error::Heap(error))
        );
        assert_eq!(objects.get(&a, &key, &mut budget).unwrap(), Value::Null);
    }
    assert_eq!(objects.collect([&a], 100).unwrap().live, 1);
}
