use super::*;
use crate::object::DataDescriptor;
use spite_core::{JsString, JsSymbol};

fn unlimited() -> Budget {
    Budget::with_work_limit(None)
}

#[test]
fn weak_map_object_keys_use_identity_and_dense_mutation_preserves_the_index() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let map = objects.create_weak_map(&prototype).unwrap();
    assert!(objects.weak_map_has_slot(&map).unwrap());
    assert!(!objects.weak_map_has_slot(&prototype).unwrap());
    let keys: Vec<_> = (0..100).map(|_| objects.create(None).unwrap()).collect();
    for (index, key) in keys.iter().enumerate() {
        objects
            .weak_map_set(&map, key, Value::Number(index as f64), &mut unlimited())
            .unwrap();
    }
    for key in keys.iter().step_by(2) {
        assert!(
            objects
                .weak_map_delete(&map, key, &mut unlimited())
                .unwrap()
        );
        assert!(
            !objects
                .weak_map_delete(&map, key, &mut unlimited())
                .unwrap()
        );
    }
    for (index, key) in keys.iter().enumerate() {
        assert_eq!(
            objects.weak_map_get(&map, key, &mut unlimited()).unwrap(),
            (index % 2 == 1).then_some(Value::Number(index as f64))
        );
        objects
            .weak_map_set(&map, key, Value::Undefined, &mut unlimited())
            .unwrap();
        assert!(objects.weak_map_has(&map, key, &mut unlimited()).unwrap());
        assert_eq!(
            objects.weak_map_get(&map, key, &mut unlimited()).unwrap(),
            Some(Value::Undefined)
        );
    }
    assert_eq!(objects.weak_map_data(&map).unwrap().entries.len(), 100);
    objects
        .weak_map_set(&map, &keys[0], Value::Null, &mut unlimited())
        .unwrap();
    assert_eq!(objects.weak_map_data(&map).unwrap().entries.len(), 100);
}

#[test]
fn conditional_values_require_both_a_rooted_container_and_a_rooted_key() {
    for root_map in [false, true] {
        for root_key in [false, true] {
            let mut objects = Objects::with_limits(None, None);
            let prototype = objects.create(None).unwrap();
            let map = objects.create_weak_map(&prototype).unwrap();
            let key = objects.create(None).unwrap();
            let value = objects.create(None).unwrap();
            objects
                .weak_map_set(&map, &key, Value::Object(value.clone()), &mut unlimited())
                .unwrap();
            let mut roots = Vec::new();
            if root_map {
                roots.push(&map);
            }
            if root_key {
                roots.push(&key);
            }
            objects.collect(roots, usize::MAX).unwrap();
            assert_eq!(objects.inspect(&map).is_ok(), root_map);
            assert_eq!(objects.inspect(&key).is_ok(), root_key);
            assert_eq!(objects.inspect(&value).is_ok(), root_map && root_key);
            if root_map {
                assert_eq!(
                    objects.weak_map_data(&map).unwrap().entries.len(),
                    usize::from(root_key)
                );
            }
        }
    }
}

#[test]
fn value_back_edges_and_conditional_cycles_do_not_keep_weak_keys_alive() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let map = objects.create_weak_map(&prototype).unwrap();
    let key = objects.create(None).unwrap();
    let value = objects.create(None).unwrap();
    objects
        .define(
            &value,
            JsString::from("back"),
            DataDescriptor {
                value: Some(Value::Object(key.clone())),
                ..DataDescriptor::default()
            },
            &mut unlimited(),
        )
        .unwrap();
    objects
        .weak_map_set(&map, &key, Value::Object(value.clone()), &mut unlimited())
        .unwrap();
    objects
        .weak_map_set(&map, &value, Value::Object(key.clone()), &mut unlimited())
        .unwrap();
    assert_eq!(objects.collect([&map], usize::MAX).unwrap().reclaimed, 2);
    assert!(objects.inspect(&key).is_err());
    assert!(objects.inspect(&value).is_err());
    let data = objects.weak_map_data(&map).unwrap();
    assert!(data.entries.is_empty());
    assert!(data.index.is_empty());
}

#[test]
fn activated_weak_maps_can_expose_more_keys_and_conditional_values() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let first = objects.create_weak_map(&prototype).unwrap();
    let second = objects.create_weak_map(&prototype).unwrap();
    let first_key = objects.create(None).unwrap();
    let second_key = objects.create(None).unwrap();
    let value = objects.create(None).unwrap();
    objects
        .define(
            &second,
            JsString::from("key"),
            DataDescriptor {
                value: Some(Value::Object(second_key.clone())),
                ..DataDescriptor::default()
            },
            &mut unlimited(),
        )
        .unwrap();
    objects
        .weak_map_set(
            &first,
            &first_key,
            Value::Object(second.clone()),
            &mut unlimited(),
        )
        .unwrap();
    objects
        .weak_map_set(
            &second,
            &second_key,
            Value::Object(value.clone()),
            &mut unlimited(),
        )
        .unwrap();
    assert_eq!(
        objects
            .collect([&first_key, &first], usize::MAX)
            .unwrap()
            .live,
        6
    );
    assert_eq!(
        objects
            .weak_map_get(&second, &second_key, &mut unlimited())
            .unwrap(),
        Some(Value::Object(value.clone()))
    );
    assert_eq!(objects.collect([&first], usize::MAX).unwrap().reclaimed, 4);
    assert!(objects.inspect(&value).is_err());
    assert!(objects.weak_map_data(&first).unwrap().entries.is_empty());
}

#[test]
fn inactive_symbol_values_are_released_and_host_roots_activate_values() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let map = objects.create_weak_map(&prototype).unwrap();
    let key = objects.create(None).unwrap();
    let root = objects.root(&key, &mut unlimited()).unwrap();
    let symbol = JsSymbol::new(None);
    let weak = symbol.downgrade();
    objects
        .weak_map_set(&map, &key, Value::Symbol(symbol), &mut unlimited())
        .unwrap();
    assert_eq!(objects.collect([&map], usize::MAX).unwrap().live, 3);
    assert!(weak.upgrade().is_some());
    drop(root);
    assert_eq!(objects.collect([&map], usize::MAX).unwrap().reclaimed, 1);
    assert_eq!(weak.upgrade(), None);
    assert!(objects.weak_map_data(&map).unwrap().entries.is_empty());
}

#[test]
fn deleted_and_replaced_symbol_values_release_ownership_immediately() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let map = objects.create_weak_map(&prototype).unwrap();
    let key = objects.create(None).unwrap();
    for delete in [false, true] {
        let symbol = JsSymbol::new(None);
        let weak = symbol.downgrade();
        objects
            .weak_map_set(&map, &key, Value::Symbol(symbol), &mut unlimited())
            .unwrap();
        if delete {
            objects
                .weak_map_delete(&map, &key, &mut unlimited())
                .unwrap();
        } else {
            objects
                .weak_map_set(&map, &key, Value::Null, &mut unlimited())
                .unwrap();
        }
        assert_eq!(weak.upgrade(), None);
    }
}

#[test]
fn cleanup_updates_moved_indices_and_stale_handles_cannot_address_reused_keys() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let map = objects.create_weak_map(&prototype).unwrap();
    let keys: Vec<_> = (0..100).map(|_| objects.create(None).unwrap()).collect();
    for (index, key) in keys.iter().enumerate() {
        objects
            .weak_map_set(&map, key, Value::Number(index as f64), &mut unlimited())
            .unwrap();
    }
    objects
        .collect(
            std::iter::once(&map).chain(keys.iter().skip(1).step_by(2)),
            usize::MAX,
        )
        .unwrap();
    assert_eq!(objects.weak_map_data(&map).unwrap().entries.len(), 50);
    for (index, key) in keys.iter().enumerate().skip(1).step_by(2) {
        assert_eq!(
            objects.weak_map_get(&map, key, &mut unlimited()).unwrap(),
            Some(Value::Number(index as f64))
        );
    }
    let replacement = objects.create(None).unwrap();
    assert_ne!(replacement, keys[0]);
    assert!(
        !objects
            .weak_map_has(&map, &replacement, &mut unlimited())
            .unwrap()
    );
    assert!(matches!(
        objects.weak_map_get(&map, &keys[0], &mut unlimited()),
        Err(Error::Heap(spite_heap::Error::StaleHandle))
    ));
    objects
        .weak_map_set(&map, &replacement, Value::Boolean(true), &mut unlimited())
        .unwrap();
    assert_eq!(
        objects
            .weak_map_get(&map, &replacement, &mut unlimited())
            .unwrap(),
        Some(Value::Boolean(true))
    );
}

#[test]
fn foreign_stale_and_unbranded_handles_fail_before_storage_mutation() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let map = objects.create_weak_map(&prototype).unwrap();
    let key = objects.create(None).unwrap();
    let stale = objects.create(None).unwrap();
    objects.collect([&map, &key], usize::MAX).unwrap();
    let mut other = Objects::with_limits(None, None);
    let foreign = other.create(None).unwrap();
    for bad in [stale, foreign] {
        assert!(matches!(
            objects.weak_map_set(&map, &bad, Value::Null, &mut unlimited()),
            Err(Error::Heap(_))
        ));
        assert!(matches!(
            objects.weak_map_get(&map, &bad, &mut unlimited()),
            Err(Error::Heap(_))
        ));
        assert!(matches!(
            objects.weak_map_has(&map, &bad, &mut unlimited()),
            Err(Error::Heap(_))
        ));
        assert!(matches!(
            objects.weak_map_delete(&map, &bad, &mut unlimited()),
            Err(Error::Heap(_))
        ));
        assert!(matches!(
            objects.weak_map_set(&map, &key, Value::Object(bad.clone()), &mut unlimited()),
            Err(Error::Heap(_))
        ));
        assert!(matches!(
            objects.weak_map_set(&bad, &key, Value::Null, &mut unlimited()),
            Err(Error::Heap(_))
        ));
    }
    assert!(matches!(
        objects.weak_map_set(&prototype, &key, Value::Null, &mut unlimited()),
        Err(Error::WrongKind)
    ));
    assert!(objects.weak_map_data(&map).unwrap().entries.is_empty());
}

#[test]
fn opted_in_operation_and_collection_failures_preserve_entries_and_owned_symbols() {
    let mut objects = Objects::with_limits(None, None);
    let prototype = objects.create(None).unwrap();
    let map = objects.create_weak_map(&prototype).unwrap();
    let key = objects.create(None).unwrap();
    let next = objects.create(None).unwrap();
    let symbol = JsSymbol::new(None);
    let weak = symbol.downgrade();
    objects
        .weak_map_set(&map, &key, Value::Symbol(symbol), &mut unlimited())
        .unwrap();
    for target in [&key, &next] {
        assert!(matches!(
            objects.weak_map_set(
                &map,
                target,
                Value::String(JsString::from("copy")),
                &mut Budget::new(4)
            ),
            Err(Error::WorkLimit)
        ));
    }
    assert!(matches!(
        objects.weak_map_delete(&map, &key, &mut Budget::new(0)),
        Err(Error::WorkLimit)
    ));
    assert!(matches!(
        objects.weak_map_get(&map, &key, &mut Budget::new(1)),
        Err(Error::WorkLimit)
    ));
    assert!(matches!(
        objects.weak_map_has(&map, &key, &mut Budget::new(0)),
        Err(Error::WorkLimit)
    ));
    assert!(weak.upgrade().is_some());
    assert_eq!(objects.weak_map_data(&map).unwrap().entries.len(), 1);
    assert_eq!(objects.collect([&map], 1), Err(spite_heap::Error::Limit));
    assert!(weak.upgrade().is_some());
    assert_eq!(objects.weak_map_data(&map).unwrap().entries.len(), 1);
    objects.collect([&map], usize::MAX).unwrap();
    assert_eq!(weak.upgrade(), None);
    assert!(objects.weak_map_data(&map).unwrap().entries.is_empty());
}
