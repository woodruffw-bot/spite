//! Ordinary data descriptors, string-key ordering, and heap integration.

use spite_core::JsString;
use spite_heap::{Error as HeapError, Heap};
use spite_runtime::{
    Value,
    object::{DataDescriptor, OrdinaryObject, PropertyLimit},
};

fn data(value: Value) -> DataDescriptor {
    DataDescriptor {
        value: Some(value),
        writable: Some(true),
        enumerable: Some(true),
        configurable: Some(true),
    }
}

fn define(object: &mut OrdinaryObject, name: &str, descriptor: DataDescriptor) -> bool {
    object
        .define_own_property(JsString::from(name), descriptor)
        .unwrap()
}

fn value(object: &OrdinaryObject, name: &str) -> Value {
    object
        .own_property(&JsString::from(name))
        .unwrap()
        .as_data()
        .unwrap()
        .value
        .clone()
}

#[test]
fn omitted_fields_default_on_creation_and_preserve_existing_properties() {
    let mut object = OrdinaryObject::new(None, 4);
    assert!(object.is_extensible());
    assert!(object.prototype().is_none());
    assert!(define(&mut object, "empty", DataDescriptor::default()));
    let property = object
        .own_property(&JsString::from("empty"))
        .unwrap()
        .as_data()
        .unwrap();
    assert_eq!(property.value, Value::Undefined);
    assert!(!property.writable && !property.enumerable && !property.configurable);
    assert!(define(&mut object, "full", data(Value::Number(3.0))));
    assert!(define(
        &mut object,
        "full",
        DataDescriptor {
            value: Some(Value::Null),
            ..Default::default()
        }
    ));
    assert!(define(&mut object, "full", DataDescriptor::default()));
    let property = object
        .own_property(&JsString::from("full"))
        .unwrap()
        .as_data()
        .unwrap();
    assert_eq!(property.value, Value::Null);
    assert!(property.writable && property.enumerable && property.configurable);
    assert_eq!(object.property_count(), 2);
}

#[test]
fn nonconfigurable_writable_properties_can_change_value_and_be_frozen() {
    let mut object = OrdinaryObject::new(None, 1);
    assert!(define(
        &mut object,
        "x",
        DataDescriptor {
            configurable: Some(false),
            ..data(Value::Number(1.0))
        }
    ));
    for descriptor in [
        DataDescriptor {
            configurable: Some(true),
            ..Default::default()
        },
        DataDescriptor {
            enumerable: Some(false),
            ..Default::default()
        },
        DataDescriptor {
            value: Some(Value::Null),
            enumerable: Some(false),
            ..Default::default()
        },
    ] {
        assert!(!define(&mut object, "x", descriptor));
        assert_eq!(value(&object, "x"), Value::Number(1.0));
    }
    assert!(define(
        &mut object,
        "x",
        DataDescriptor {
            value: Some(Value::Number(2.0)),
            writable: Some(false),
            ..Default::default()
        }
    ));
    assert_eq!(value(&object, "x"), Value::Number(2.0));
    assert!(!define(
        &mut object,
        "x",
        DataDescriptor {
            writable: Some(true),
            ..Default::default()
        }
    ));
    assert!(!define(
        &mut object,
        "x",
        DataDescriptor {
            value: Some(Value::Number(3.0)),
            ..Default::default()
        }
    ));
    assert!(define(
        &mut object,
        "x",
        DataDescriptor {
            value: Some(Value::Number(2.0)),
            writable: Some(false),
            ..Default::default()
        }
    ));
    assert!(!object.delete(&JsString::from("x")));
    assert!(object.delete(&JsString::from("absent")));
}

#[test]
fn frozen_values_use_same_value_and_preserve_nan_payloads() {
    let mut object = OrdinaryObject::new(None, 2);
    let original = f64::from_bits(0x7ff8_0000_0000_0001);
    let replacement = f64::from_bits(0xfff8_0000_0000_0002);
    assert!(define(
        &mut object,
        "nan",
        DataDescriptor {
            value: Some(Value::Number(original)),
            ..Default::default()
        }
    ));
    assert!(define(
        &mut object,
        "nan",
        DataDescriptor {
            value: Some(Value::Number(replacement)),
            ..Default::default()
        }
    ));
    let Value::Number(stored) = value(&object, "nan") else {
        panic!("expected Number")
    };
    assert_eq!(stored.to_bits(), original.to_bits());
    assert!(define(
        &mut object,
        "zero",
        DataDescriptor {
            value: Some(Value::Number(-0.0)),
            ..Default::default()
        }
    ));
    assert!(!define(
        &mut object,
        "zero",
        DataDescriptor {
            value: Some(Value::Number(0.0)),
            ..Default::default()
        }
    ));
    assert!(define(
        &mut object,
        "zero",
        DataDescriptor {
            value: Some(Value::Number(-0.0)),
            ..Default::default()
        }
    ));
}

#[test]
fn capacity_and_nonextensibility_are_distinct_and_do_not_mutate_properties() {
    let mut object = OrdinaryObject::new(None, 1);
    assert!(define(&mut object, "x", data(Value::Number(1.0))));
    assert_eq!(
        object.define_own_property(JsString::from("y"), data(Value::Null)),
        Err(PropertyLimit)
    );
    assert_eq!(object.property_count(), 1);
    object.prevent_extensions();
    object.prevent_extensions();
    assert!(!object.is_extensible());
    assert!(!define(&mut object, "y", data(Value::Null)));
    assert!(define(&mut object, "x", data(Value::Number(2.0))));
    assert!(object.delete(&JsString::from("x")));
    assert!(!define(&mut object, "x", data(Value::Number(3.0))));
    assert_eq!(object.property_count(), 0);
    let mut zero = OrdinaryObject::new(None, 0);
    assert_eq!(
        zero.define_own_property(JsString::from("x"), DataDescriptor::default()),
        Err(PropertyLimit)
    );
}

#[test]
fn configurable_properties_can_be_reconfigured_and_deleted_capacity_reused() {
    let mut object = OrdinaryObject::new(None, 1);
    assert!(define(
        &mut object,
        "x",
        DataDescriptor {
            writable: Some(false),
            ..data(Value::Null)
        }
    ));
    assert!(define(
        &mut object,
        "x",
        DataDescriptor {
            writable: Some(true),
            enumerable: Some(false),
            value: Some(Value::Boolean(true)),
            ..Default::default()
        }
    ));
    assert_eq!(value(&object, "x"), Value::Boolean(true));
    assert!(object.delete(&JsString::from("x")));
    assert!(define(&mut object, "y", DataDescriptor::default()));
    assert_eq!(object.own_keys(), [JsString::from("y")]);
}

#[test]
fn keys_use_array_index_order_then_exact_string_creation_order() {
    let mut object = OrdinaryObject::new(None, 30);
    let strings = [
        "b",
        "4294967295",
        "00",
        "-0",
        "1.0",
        "1e0",
        "",
        "٠",
        "a",
        "4294967296",
        "999999999999999999999",
    ];
    for name in strings {
        assert!(define(&mut object, name, data(Value::Null)));
    }
    for name in ["4294967294", "20", "3", "0", "1"] {
        assert!(define(&mut object, name, DataDescriptor::default()));
    }
    let surrogate = JsString::from_code_units(vec![0xd800]);
    assert!(
        object
            .define_own_property(surrogate.clone(), DataDescriptor::default())
            .unwrap()
    );
    assert!(define(&mut object, "b", data(Value::Boolean(true))));
    let expected = ["0", "1", "3", "20", "4294967294"]
        .into_iter()
        .chain(strings)
        .map(JsString::from)
        .chain([surrogate.clone()])
        .collect::<Vec<_>>();
    assert_eq!(object.own_keys(), expected);
    assert!(object.delete(&JsString::from("b")));
    assert!(define(&mut object, "b", data(Value::Undefined)));
    let mut expected = expected;
    expected.remove(5);
    expected.push(JsString::from("b"));
    assert_eq!(object.own_keys(), expected);
    assert_eq!(
        object
            .own_property(&surrogate)
            .unwrap()
            .as_data()
            .unwrap()
            .value,
        Value::Undefined
    );
}

#[test]
fn collection_traces_prototypes_and_charges_for_primitive_properties() {
    let mut heap = Heap::new(3);
    let prototype = heap.insert(OrdinaryObject::new(None, 0)).unwrap();
    let mut object = OrdinaryObject::new(Some(prototype.clone()), 100);
    for index in 0..100 {
        assert!(define(
            &mut object,
            &format!("p{index}"),
            data(Value::Number(index.into()))
        ));
    }
    let root = heap.insert(object).unwrap();
    let garbage = heap.insert(OrdinaryObject::new(None, 0)).unwrap();
    assert_eq!(heap.get(&root).unwrap().prototype(), Some(&prototype));
    assert_eq!(heap.collect([&root], 100), Err(HeapError::Limit));
    assert_eq!(heap.len(), 3);
    let result = heap.collect([&root], 200).unwrap();
    assert_eq!(result.live, 2);
    assert_eq!(result.reclaimed, 1);
    assert!(result.work_used > 100);
    assert!(heap.get(&prototype).is_ok());
    assert!(matches!(heap.get(&garbage), Err(HeapError::StaleHandle)));
    assert_eq!(heap.collect([], 20).unwrap().reclaimed, 2);
}
