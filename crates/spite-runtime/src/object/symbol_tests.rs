use super::*;
use crate::{Realm, function::Builtin};
use spite_core::JsSymbol;

fn symbol(description: &str) -> JsSymbol {
    JsSymbol::new(Some(JsString::from(description)))
}

fn data(value: Value) -> DataDescriptor {
    DataDescriptor {
        value: Some(value),
        writable: Some(true),
        enumerable: Some(true),
        configurable: Some(true),
    }
}

#[test]
fn symbol_keys_follow_string_keys_and_keep_their_own_creation_order() {
    let mut record = OrdinaryObject::new(None, 10);
    let first = symbol("0");
    let second = symbol("0");
    for key in [
        first.clone().into(),
        PropertyKey::from("b"),
        PropertyKey::from("2"),
        second.clone().into(),
        PropertyKey::from("a"),
        PropertyKey::from("0"),
    ] {
        assert!(record.define_own_property(key, data(Value::Null)).unwrap());
    }
    assert_eq!(
        record.own_keys(),
        vec![
            "0".into(),
            "2".into(),
            "b".into(),
            "a".into(),
            first.clone().into(),
            second.clone().into(),
        ]
    );
    assert!(
        record
            .define_own_property(first.clone(), data(Value::Boolean(true)))
            .unwrap()
    );
    assert_eq!(
        record.own_keys().last(),
        Some(&PropertyKey::from(second.clone()))
    );
    assert!(record.delete(&first));
    assert!(
        record
            .define_own_property(first.clone(), data(Value::Undefined))
            .unwrap()
    );
    assert_eq!(
        record.own_keys(),
        vec![
            "0".into(),
            "2".into(),
            "b".into(),
            "a".into(),
            second.into(),
            first.into(),
        ]
    );
}

#[test]
fn symbol_prototype_reads_and_receiver_writes_use_identity() {
    let mut objects = Objects::new(4, 10);
    let mut budget = Budget::new(1000);
    let parent = objects.create(None).unwrap();
    let child = objects.create(Some(&parent)).unwrap();
    let key = symbol("x");
    objects
        .define(&parent, key.clone(), data(Value::Number(1.0)), &mut budget)
        .unwrap();
    objects
        .define(&parent, "x", data(Value::Number(9.0)), &mut budget)
        .unwrap();
    assert_eq!(
        objects.get(&child, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Number(1.0))
    );
    assert!(objects.has(&child, &key, &mut budget).unwrap());
    assert!(!objects.has_own(&child, &key, &mut budget).unwrap());
    assert!(!objects.has(&child, &symbol("x"), &mut budget).unwrap());
    assert_eq!(
        objects
            .set(
                &parent,
                key.clone(),
                Value::Number(2.0),
                Some(&child),
                &mut budget
            )
            .unwrap(),
        SetAction::Done(true)
    );
    assert_eq!(
        objects.get(&parent, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Number(1.0))
    );
    assert_eq!(
        objects.get(&child, &key, &mut budget).unwrap(),
        GetAction::Value(Value::Number(2.0))
    );
    assert_eq!(
        objects
            .get(&child, &JsString::from("x"), &mut budget)
            .unwrap(),
        GetAction::Value(Value::Number(9.0))
    );
}

#[test]
fn symbol_accessors_reuse_call_actions_and_descriptor_invariants() {
    let mut objects = Objects::new(4, 10);
    let mut budget = Budget::new(1000);
    let parent = objects.create(None).unwrap();
    let child = objects.create(Some(&parent)).unwrap();
    let function = objects
        .create_builtin(&parent, Builtin::ObjectValueOf)
        .unwrap();
    let key = symbol("getter");
    let descriptor = PropertyDescriptor {
        kind: DescriptorKind::Accessor {
            get: Some(Some(function.clone())),
            set: Some(Some(function.clone())),
        },
        enumerable: Some(false),
        configurable: Some(false),
    };
    assert!(
        objects
            .define(&parent, key.clone(), descriptor.clone(), &mut budget)
            .unwrap()
    );
    assert_eq!(
        objects.get(&child, &key, &mut budget).unwrap(),
        GetAction::Call(function.clone())
    );
    assert_eq!(
        objects
            .set(&parent, key.clone(), Value::Null, Some(&child), &mut budget)
            .unwrap(),
        SetAction::Call(function)
    );
    assert!(!objects.has_own(&child, &key, &mut budget).unwrap());
    assert!(
        objects
            .define(&parent, key.clone(), descriptor, &mut budget)
            .unwrap()
    );
    assert!(
        !objects
            .define(&parent, key.clone(), data(Value::Null), &mut budget)
            .unwrap()
    );
    assert!(!objects.delete(&parent, &key, &mut budget).unwrap());
}

#[test]
fn symbol_descriptions_do_not_trigger_array_indices_or_length() {
    let mut objects = Objects::new(3, 10);
    let mut budget = Budget::new(10_000);
    let array = objects.create_array(None, 3, &mut budget).unwrap();
    let zero = symbol("0");
    let length = symbol("length");
    objects
        .define(&array, "2", data(Value::Number(7.0)), &mut budget)
        .unwrap();
    for key in [zero.clone(), length.clone()] {
        assert!(
            objects
                .define(
                    &array,
                    key,
                    DataDescriptor {
                        configurable: Some(false),
                        ..data(Value::String(JsString::from("not a length")))
                    },
                    &mut budget
                )
                .unwrap()
        );
    }
    assert!(
        objects
            .define(
                &array,
                "length",
                DataDescriptor {
                    value: Some(Value::Number(0.0)),
                    writable: Some(false),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    assert!(
        !objects
            .has_own(&array, &JsString::from("2"), &mut budget)
            .unwrap()
    );
    assert!(objects.has_own(&array, &zero, &mut budget).unwrap());
    assert!(objects.has_own(&array, &length, &mut budget).unwrap());
    assert_eq!(
        objects
            .get(&array, &JsString::from("length"), &mut budget)
            .unwrap(),
        GetAction::Value(Value::Number(0.0))
    );
    assert!(
        objects
            .define(&array, symbol("4294967294"), data(Value::Null), &mut budget)
            .unwrap()
    );
    assert!(
        !objects
            .define(&array, "0", data(Value::Null), &mut budget)
            .unwrap()
    );
}

#[test]
fn string_wrapper_symbol_keys_are_distinct_from_fixed_index_and_length_properties() {
    let mut objects = Objects::new(3, 10);
    let mut budget = Budget::new(1000);
    let prototype = objects.create(None).unwrap();
    let string = objects
        .create_string(&prototype, JsString::from("a"), &mut budget)
        .unwrap();
    let zero = symbol("0");
    let length = symbol("length");
    objects
        .define(&string, zero.clone(), data(Value::Number(7.0)), &mut budget)
        .unwrap();
    objects
        .define(
            &string,
            length.clone(),
            data(Value::Number(8.0)),
            &mut budget,
        )
        .unwrap();
    assert_eq!(
        objects.get(&string, &zero, &mut budget).unwrap(),
        GetAction::Value(Value::Number(7.0))
    );
    assert_eq!(
        objects
            .get(&string, &JsString::from("0"), &mut budget)
            .unwrap(),
        GetAction::Value(Value::String(JsString::from("a")))
    );
    assert_eq!(
        objects
            .get(&string, &JsString::from("length"), &mut budget)
            .unwrap(),
        GetAction::Value(Value::Number(1.0))
    );
    assert!(objects.delete(&string, &zero, &mut budget).unwrap());
    assert!(
        !objects
            .delete(&string, &JsString::from("0"), &mut budget)
            .unwrap()
    );
}

#[test]
fn symbol_zero_does_not_alias_or_detach_a_mapped_argument() {
    let mut realm = Realm::default();
    let Value::Object(arguments) = realm
        .eval("let get,set,args;function f(a){get=()=>a;set=x=>a=x;args=arguments;}f(1);args")
        .unwrap()
    else {
        panic!("arguments")
    };
    let key = symbol("0");
    realm
        .objects
        .define(
            &arguments,
            key.clone(),
            DataDescriptor {
                writable: Some(false),
                ..data(Value::Number(7.0))
            },
            &mut Budget::new(1000),
        )
        .unwrap();
    assert_eq!(
        realm.eval("set(2);args[0]===2 && get()===2"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        realm
            .objects
            .get(&arguments, &key, &mut Budget::new(1000))
            .unwrap(),
        GetAction::Value(Value::Number(7.0))
    );
    assert!(
        realm
            .objects
            .delete(&arguments, &key, &mut Budget::new(1000))
            .unwrap()
    );
    assert_eq!(realm.eval("args[0]=3;get()===3"), Ok(Value::Boolean(true)));
}

#[test]
fn symbol_key_work_is_constant_but_properties_still_obey_capacity_and_value_budgets() {
    let mut objects = Objects::new(2, 2);
    let object = objects.create(None).unwrap();
    let key = JsSymbol::new(Some(JsString::from_code_units(vec![0x61; 100_000])));
    assert!(
        objects
            .define(
                &object,
                key.clone(),
                data(Value::Null),
                &mut Budget::new(10)
            )
            .unwrap()
    );
    assert!(
        objects
            .get_own(&object, &key, &mut Budget::new(10))
            .unwrap()
            .is_some()
    );
    assert_eq!(
        objects.own_keys(&object, &mut Budget::new(10)).unwrap(),
        [PropertyKey::from(key.clone())]
    );
    assert_eq!(
        objects.define(
            &object,
            key.clone(),
            data(Value::String(JsString::from_code_units(vec![0x61; 100]))),
            &mut Budget::new(10)
        ),
        Err(Error::WorkLimit)
    );
    assert_eq!(
        objects.get(&object, &key, &mut Budget::new(10)).unwrap(),
        GetAction::Value(Value::Null)
    );
    objects
        .define(&object, "x", data(Value::Null), &mut Budget::new(100))
        .unwrap();
    assert_eq!(
        objects.define(
            &object,
            symbol("x"),
            data(Value::Null),
            &mut Budget::new(100)
        ),
        Err(Error::PropertyLimit)
    );
    objects.prevent_extensions(&object).unwrap();
    assert!(
        objects
            .define(
                &object,
                key.clone(),
                data(Value::Boolean(true)),
                &mut Budget::new(100)
            )
            .unwrap()
    );
    assert!(
        !objects
            .define(
                &object,
                symbol("x"),
                data(Value::Null),
                &mut Budget::new(100)
            )
            .unwrap()
    );
    assert_eq!(
        objects.delete(&object, &key, &mut Budget::new(0)),
        Err(Error::WorkLimit)
    );
    assert!(
        objects
            .has_own(&object, &key, &mut Budget::new(10))
            .unwrap()
    );
}

#[test]
fn symbol_keyed_values_and_accessors_remain_traced_until_their_properties_are_deleted() {
    let mut objects = Objects::new(5, 10);
    let mut budget = Budget::new(1000);
    let object = objects.create(None).unwrap();
    let value = objects.create(None).unwrap();
    let function = objects
        .create_builtin(&object, Builtin::ObjectValueOf)
        .unwrap();
    let _garbage = objects.create(None).unwrap();
    let value_key = symbol("value");
    let accessor_key = symbol("accessor");
    objects
        .define(
            &object,
            value_key.clone(),
            data(Value::Object(value)),
            &mut budget,
        )
        .unwrap();
    objects
        .define(
            &object,
            accessor_key.clone(),
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(function.clone())),
                    set: Some(Some(function)),
                },
                configurable: Some(true),
                enumerable: Some(true),
            },
            &mut budget,
        )
        .unwrap();
    let collected = objects.collect([&object], 100).unwrap();
    assert_eq!((collected.live, collected.reclaimed), (3, 1));
    objects.delete(&object, &value_key, &mut budget).unwrap();
    let collected = objects.collect([&object], 100).unwrap();
    assert_eq!((collected.live, collected.reclaimed), (2, 1));
    objects.delete(&object, &accessor_key, &mut budget).unwrap();
    let collected = objects.collect([&object], 100).unwrap();
    assert_eq!((collected.live, collected.reclaimed), (1, 1));
}

#[test]
fn realm_enumeration_reports_the_pending_symbol_boundary_without_omitting_keys() {
    let mut realm = Realm::default();
    let Value::Object(object) = realm.eval("let source={};source").unwrap() else {
        panic!("object")
    };
    realm
        .objects
        .define(
            &object,
            symbol("x"),
            data(Value::Number(7.0)),
            &mut Budget::new(100),
        )
        .unwrap();
    assert!(matches!(
        realm.eval("Object.assign({},source)"),
        Err(crate::Error::Unsupported { .. })
    ));
    assert!(matches!(
        realm.eval("Object.getOwnPropertyDescriptors(source)"),
        Err(crate::Error::Unsupported { .. })
    ));
    assert_eq!(
        realm.eval("Object.hasOwn(source,'x')"),
        Ok(Value::Boolean(false))
    );
}
