//! Ordinary prototype operations and receiver-sensitive data writes.

use spite_core::JsString;
use spite_heap::Error as HeapError;
use spite_runtime::{
    Value,
    object::{Budget, DataDescriptor, Error, GetAction, Objects, SetAction},
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
fn inherited_get_and_has_distinguish_absence_from_undefined() {
    let mut objects = Objects::new(3, 5);
    let base = objects.create(None).unwrap();
    let middle = objects.create(Some(&base)).unwrap();
    let leaf = objects.create(Some(&middle)).unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("x");
    assert!(!objects.has(&leaf, &key, &mut budget).unwrap());
    assert_eq!(
        objects.get(&leaf, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Undefined)
    );
    assert!(
        objects
            .define(&base, key.clone(), data(Value::Undefined), &mut budget)
            .unwrap()
    );
    assert!(objects.has(&leaf, &key, &mut budget).unwrap());
    assert_eq!(
        objects.get(&leaf, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Undefined)
    );
    assert!(
        objects
            .define(&middle, key.clone(), data(Value::Number(7.0)), &mut budget)
            .unwrap()
    );
    assert_eq!(
        objects.get(&leaf, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Number(7.0))
    );
    assert!(objects.delete(&leaf, &key, &mut budget).unwrap());
    assert!(objects.has(&leaf, &key, &mut budget).unwrap());
    assert!(objects.delete(&middle, &key, &mut budget).unwrap());
    assert_eq!(
        objects.get(&leaf, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Undefined)
    );
}

#[test]
fn prototype_mutation_rejects_cycles_and_respects_extensibility() {
    let mut objects = Objects::new(3, 0);
    let a = objects.create(None).unwrap();
    let b = objects.create(Some(&a)).unwrap();
    let c = objects.create(Some(&b)).unwrap();
    let mut budget = Budget::new(100);
    assert!(!objects.set_prototype(&a, Some(&a), &mut budget).unwrap());
    assert!(!objects.set_prototype(&a, Some(&c), &mut budget).unwrap());
    assert!(objects.inspect(&a).unwrap().prototype().is_none());
    objects.prevent_extensions(&c).unwrap();
    assert!(objects.set_prototype(&c, Some(&b), &mut budget).unwrap());
    assert!(!objects.set_prototype(&c, Some(&a), &mut budget).unwrap());
    assert!(!objects.set_prototype(&c, None, &mut budget).unwrap());
    assert_eq!(objects.inspect(&c).unwrap().prototype(), Some(&b));
    assert!(objects.set_prototype(&b, None, &mut budget).unwrap());
    assert!(objects.set_prototype(&a, Some(&c), &mut budget).unwrap());
}

#[test]
fn inherited_writes_create_receiver_properties_and_preserve_existing_attributes() {
    let mut objects = Objects::new(3, 3);
    let base = objects.create(None).unwrap();
    let receiver = objects.create(Some(&base)).unwrap();
    let unrelated = objects.create(None).unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("x");
    assert!(
        objects
            .define(&base, key.clone(), data(Value::Number(1.0)), &mut budget)
            .unwrap()
    );
    assert_eq!(
        objects
            .set(
                &receiver,
                key.clone(),
                Value::Number(2.0),
                Some(&receiver),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(true)
    );
    assert_eq!(
        objects.get(&base, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Number(1.0))
    );
    let property = objects
        .inspect(&receiver)
        .unwrap()
        .own_property(&key)
        .unwrap()
        .as_data()
        .unwrap();
    assert_eq!(property.value, Value::Number(2.0));
    assert!(property.writable && property.enumerable && property.configurable);
    assert!(
        objects
            .define(
                &receiver,
                key.clone(),
                DataDescriptor {
                    enumerable: Some(false),
                    configurable: Some(false),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(
        objects
            .set(
                &base,
                key.clone(),
                Value::Number(3.0),
                Some(&receiver),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(true)
    );
    let property = objects
        .inspect(&receiver)
        .unwrap()
        .own_property(&key)
        .unwrap()
        .as_data()
        .unwrap();
    assert_eq!(property.value, Value::Number(3.0));
    assert!(property.writable && !property.enumerable && !property.configurable);
    assert_eq!(
        objects
            .set(
                &base,
                key.clone(),
                Value::Number(4.0),
                Some(&unrelated),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(true)
    );
    assert_eq!(
        objects.get(&unrelated, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Number(4.0))
    );
    let missing = JsString::from("missing");
    assert_eq!(
        objects
            .set(
                &base,
                missing.clone(),
                Value::Null,
                Some(&unrelated),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(true)
    );
    assert!(!objects.has(&base, &missing, &mut budget).unwrap());
    assert_eq!(
        objects.get(&unrelated, &missing, &mut budget).unwrap(),
        GetAction::Value(Value::Null)
    );
}

#[test]
fn rejected_writes_leave_receiver_and_prototype_values_unchanged() {
    let mut objects = Objects::new(3, 3);
    let base = objects.create(None).unwrap();
    let receiver = objects.create(Some(&base)).unwrap();
    let unrelated = objects.create(None).unwrap();
    let mut budget = Budget::new(1000);
    let key = JsString::from("frozen");
    assert!(
        objects
            .define(
                &base,
                key.clone(),
                DataDescriptor {
                    value: Some(Value::Number(1.0)),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(
        objects
            .set(
                &receiver,
                key.clone(),
                Value::Number(1.0),
                Some(&receiver),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(false)
    );
    assert!(
        objects
            .inspect(&receiver)
            .unwrap()
            .own_property(&key)
            .is_none()
    );
    assert!(
        objects
            .define(
                &receiver,
                key.clone(),
                data(Value::Number(2.0)),
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(
        objects
            .set(
                &base,
                key.clone(),
                Value::Number(3.0),
                Some(&receiver),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(false)
    );
    assert_eq!(
        objects.get(&receiver, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Number(2.0))
    );
    assert_eq!(
        objects
            .set(
                &unrelated,
                key.clone(),
                Value::Number(3.0),
                Some(&base),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(false)
    );
    assert_eq!(
        objects
            .set(
                &unrelated,
                key.clone(),
                Value::Number(3.0),
                None,
                &mut budget
            )
            .unwrap(),
        SetAction::Done(false)
    );
    objects.prevent_extensions(&unrelated).unwrap();
    assert_eq!(
        objects
            .set(
                &receiver,
                key.clone(),
                Value::Number(4.0),
                Some(&unrelated),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(false)
    );
    assert!(
        objects
            .inspect(&unrelated)
            .unwrap()
            .own_property(&key)
            .is_none()
    );
}

#[test]
fn deep_chains_are_iterative_and_exhaustion_never_changes_the_graph() {
    let mut objects = Objects::new(20_002, 1);
    let base = objects.create(None).unwrap();
    let mut leaf = base.clone();
    for _ in 0..20_000 {
        leaf = objects.create(Some(&leaf)).unwrap();
    }
    let other = objects.create(None).unwrap();
    let key = JsString::from("x");
    let mut budget = Budget::new(100_000);
    assert!(
        objects
            .define(&base, key.clone(), data(Value::Boolean(true)), &mut budget)
            .unwrap()
    );
    assert_eq!(
        objects.get(&leaf, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Boolean(true))
    );
    assert!(objects.has(&leaf, &key, &mut budget).unwrap());
    for work in [0, 1, 10, 100, 20_000] {
        assert_eq!(
            objects.set_prototype(&other, Some(&leaf), &mut Budget::new(work)),
            Err(Error::WorkLimit)
        );
        assert!(objects.inspect(&other).unwrap().prototype().is_none());
    }
    assert_eq!(
        objects.get(&leaf, &key, &mut Budget::new(100)),
        Err(Error::WorkLimit)
    );
    assert!(
        objects
            .set_prototype(&other, Some(&leaf), &mut budget)
            .unwrap()
    );
    assert!(
        !objects
            .set_prototype(&base, Some(&other), &mut budget)
            .unwrap()
    );
}

#[test]
fn every_failing_write_budget_preserves_state_and_capacity_is_separate() {
    let mut objects = Objects::new(1, 1);
    let object = objects.create(None).unwrap();
    let key = JsString::from("x");
    let mut budget = Budget::new(1000);
    assert!(
        objects
            .define(
                &object,
                key.clone(),
                data(Value::String(JsString::from("old"))),
                &mut budget
            )
            .unwrap()
    );
    let mut succeeded = false;
    for work in 0..100 {
        match objects.set(
            &object,
            key.clone(),
            Value::String(JsString::from("replacement")),
            Some(&object),
            &mut Budget::new(work),
        ) {
            Err(Error::WorkLimit) => assert_eq!(
                objects
                    .inspect(&object)
                    .unwrap()
                    .own_property(&key)
                    .unwrap()
                    .as_data()
                    .unwrap()
                    .value,
                Value::String(JsString::from("old"))
            ),
            Ok(SetAction::Done(true)) => {
                succeeded = true;
                break;
            }
            result => panic!("unexpected {result:?}"),
        }
    }
    assert!(succeeded);
    assert_eq!(
        objects.define(&object, JsString::from("y"), data(Value::Null), &mut budget),
        Err(Error::PropertyLimit)
    );
    assert_eq!(objects.create(None), Err(Error::Heap(HeapError::Capacity)));
}

#[test]
fn foreign_and_collected_handles_cannot_enter_prototype_graphs() {
    let mut objects = Objects::new(3, 0);
    let root = objects.create(None).unwrap();
    let garbage = objects.create(None).unwrap();
    let foreign = Objects::new(1, 0).create(None).unwrap();
    let mut budget = Budget::new(100);
    assert_eq!(
        objects.create(Some(&foreign)),
        Err(Error::Heap(HeapError::ForeignHandle))
    );
    assert_eq!(
        objects.set_prototype(&root, Some(&foreign), &mut budget),
        Err(Error::Heap(HeapError::ForeignHandle))
    );
    assert_eq!(objects.collect([&root], 100).unwrap().reclaimed, 1);
    assert_eq!(
        objects.create(Some(&garbage)),
        Err(Error::Heap(HeapError::StaleHandle))
    );
    assert_eq!(
        objects.set_prototype(&root, Some(&garbage), &mut budget),
        Err(Error::Heap(HeapError::StaleHandle))
    );
    let leaf = objects.create(Some(&root)).unwrap();
    assert_eq!(objects.collect([&leaf], 100).unwrap().live, 2);
    assert!(objects.inspect(&root).is_ok());
}
