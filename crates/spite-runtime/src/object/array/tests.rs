use super::*;
use crate::object::{DataDescriptor, Objects};
use spite_core::JsString;
use spite_heap::Handle;

fn data(value: Value) -> DataDescriptor {
    DataDescriptor {
        value: Some(value),
        writable: Some(true),
        enumerable: Some(true),
        configurable: Some(true),
    }
}

fn number(value: f64) -> DataDescriptor {
    data(Value::Number(value))
}

fn length(objects: &Objects, array: &Handle) -> (u32, bool) {
    objects.inspect(array).unwrap().array_length()
}

fn define_length(
    objects: &mut Objects,
    array: &Handle,
    value: f64,
    writable: Option<bool>,
    budget: &mut Budget,
) -> Result<bool, Error> {
    objects.define(
        array,
        JsString::from("length"),
        DataDescriptor {
            value: Some(Value::Number(value)),
            writable,
            ..Default::default()
        },
        budget,
    )
}

#[test]
fn arrays_are_sparse_and_have_fixed_length_descriptors() {
    let mut objects = Objects::new(10, 10);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, u32::MAX, &mut budget).unwrap();
    let record = objects.inspect(&array).unwrap();
    assert!(record.is_array());
    assert_eq!(record.property_count(), 1);
    assert_eq!(length(&objects, &array), (u32::MAX, true));
    let descriptor = record
        .own_property(&JsString::from("length"))
        .unwrap()
        .as_data()
        .unwrap();
    assert!(!descriptor.enumerable && !descriptor.configurable);
    assert!(
        !objects
            .delete(&array, &JsString::from("length"), &mut budget)
            .unwrap()
    );
    let ordinary = objects.create(Some(&array)).unwrap();
    assert!(!objects.inspect(&ordinary).unwrap().is_array());
    assert_eq!(
        objects
            .create_array(Some(&ordinary), 0, &mut budget)
            .map(|a| objects.inspect(&a).unwrap().prototype().cloned()),
        Ok(Some(ordinary))
    );
}

#[test]
fn canonical_indices_grow_length_and_other_keys_do_not() {
    let mut objects = Objects::new(10, 20);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, 0, &mut budget).unwrap();
    for key in ["01", "-0", "1.0", "4294967295", "4294967296"] {
        assert!(
            objects
                .define(&array, JsString::from(key), number(1.0), &mut budget)
                .unwrap()
        );
        assert_eq!(length(&objects, &array), (0, true));
    }
    assert!(
        objects
            .define(&array, JsString::from("2"), number(2.0), &mut budget)
            .unwrap()
    );
    assert_eq!(length(&objects, &array), (3, true));
    assert!(
        objects
            .define(
                &array,
                JsString::from("4294967294"),
                number(3.0),
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(length(&objects, &array), (u32::MAX, true));
    assert!(
        objects
            .delete(&array, &JsString::from("4294967294"), &mut budget)
            .unwrap()
    );
    assert_eq!(length(&objects, &array), (u32::MAX, true));
    assert!(define_length(&mut objects, &array, 0.0, None, &mut budget).unwrap());
    assert_eq!(
        objects.inspect(&array).unwrap().own_keys(),
        ["length", "01", "-0", "1.0", "4294967295", "4294967296"]
            .map(spite_core::PropertyKey::from)
    );
}

#[test]
fn readonly_length_rejects_growth_but_allows_lower_indices() {
    let mut objects = Objects::new(10, 20);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, 3, &mut budget).unwrap();
    assert!(
        objects
            .define(
                &array,
                JsString::from("length"),
                DataDescriptor {
                    writable: Some(false),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    assert!(
        objects
            .define(&array, JsString::from("1"), number(7.0), &mut budget)
            .unwrap()
    );
    assert!(
        !objects
            .define(&array, JsString::from("3"), number(8.0), &mut budget)
            .unwrap()
    );
    assert!(define_length(&mut objects, &array, 3.0, None, &mut budget).unwrap());
    assert!(!define_length(&mut objects, &array, 2.0, None, &mut budget).unwrap());
    assert!(!define_length(&mut objects, &array, 4.0, None, &mut budget).unwrap());
    assert_eq!(length(&objects, &array), (3, false));
}

#[test]
fn accessor_indices_grow_length_but_length_cannot_become_an_accessor() {
    let mut objects = Objects::new(10, 10);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, 0, &mut budget).unwrap();
    let accessor = PropertyDescriptor {
        kind: DescriptorKind::Accessor {
            get: Some(None),
            set: Some(None),
        },
        enumerable: Some(true),
        configurable: Some(true),
    };
    assert!(
        objects
            .define(&array, JsString::from("2"), accessor.clone(), &mut budget)
            .unwrap()
    );
    assert_eq!(length(&objects, &array), (3, true));
    assert!(
        !objects
            .define(&array, JsString::from("length"), accessor, &mut budget)
            .unwrap()
    );
    assert_eq!(length(&objects, &array), (3, true));
    assert!(define_length(&mut objects, &array, 0.0, None, &mut budget).unwrap());
    assert_eq!(objects.inspect(&array).unwrap().property_count(), 1);
}

#[test]
fn truncation_preserves_partial_deletions_and_requested_readonly_length() {
    for writable in [None, Some(false)] {
        let mut objects = Objects::new(10, 20);
        let mut budget = Budget::new(100_000);
        let array = objects.create_array(None, 10, &mut budget).unwrap();
        // Deliberately insert in nonnumeric order to test descending deletion.
        for index in [7, 2, 9, 5, 3] {
            let mut descriptor = number(f64::from(index));
            if index == 5 {
                descriptor.configurable = Some(false);
            }
            objects
                .define(
                    &array,
                    JsString::from(index.to_string().as_str()),
                    descriptor,
                    &mut budget,
                )
                .unwrap();
        }
        assert!(!define_length(&mut objects, &array, 1.0, writable, &mut budget).unwrap());
        assert_eq!(length(&objects, &array), (6, writable != Some(false)));
        assert_eq!(
            objects.inspect(&array).unwrap().own_keys(),
            ["2", "3", "5", "length"].map(spite_core::PropertyKey::from)
        );
    }
}

#[test]
fn successful_truncation_and_descriptor_rejection_obey_attributes() {
    let mut objects = Objects::new(10, 20);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, 4, &mut budget).unwrap();
    for i in 0..4 {
        objects
            .define(
                &array,
                JsString::from(i.to_string().as_str()),
                number(f64::from(i)),
                &mut budget,
            )
            .unwrap();
    }
    assert!(
        !objects
            .define(
                &array,
                JsString::from("length"),
                DataDescriptor {
                    value: Some(Value::Number(0.0)),
                    configurable: Some(true),
                    writable: Some(false),
                    ..Default::default()
                },
                &mut budget
            )
            .unwrap()
    );
    assert_eq!(length(&objects, &array), (4, true));
    assert_eq!(objects.inspect(&array).unwrap().property_count(), 5);
    objects.prevent_extensions(&array).unwrap();
    assert!(define_length(&mut objects, &array, 8.0, None, &mut budget).unwrap());
    assert!(
        !objects
            .define(&array, JsString::from("7"), number(7.0), &mut budget)
            .unwrap()
    );
    assert!(define_length(&mut objects, &array, 2.0, Some(false), &mut budget).unwrap());
    assert_eq!(length(&objects, &array), (2, false));
    assert_eq!(
        objects.inspect(&array).unwrap().own_keys(),
        ["0", "1", "length"].map(spite_core::PropertyKey::from)
    );
}

#[test]
fn storage_requires_normalized_lengths_and_canonicalizes_negative_zero() {
    let mut objects = Objects::new(10, 10);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, 1, &mut budget).unwrap();
    for value in [
        Value::Undefined,
        Value::Boolean(true),
        Value::String(JsString::from("1")),
        Value::Number(-1.0),
        Value::Number(0.5),
        Value::Number(4294967296.0),
        Value::Number(f64::NAN),
        Value::Number(f64::INFINITY),
    ] {
        assert_eq!(
            objects.define(
                &array,
                JsString::from("length"),
                DataDescriptor {
                    value: Some(value),
                    ..Default::default()
                },
                &mut budget
            ),
            Err(Error::UnnormalizedArrayLength)
        );
        assert_eq!(length(&objects, &array), (1, true));
    }
    assert!(define_length(&mut objects, &array, -0.0, None, &mut budget).unwrap());
    let Value::Number(value) = objects
        .inspect(&array)
        .unwrap()
        .own_property(&JsString::from("length"))
        .unwrap()
        .as_data()
        .unwrap()
        .value
    else {
        panic!("number");
    };
    assert_eq!(value.to_bits(), 0.0f64.to_bits());
}

#[test]
fn work_and_property_limits_leave_array_invariants_intact() {
    let mut objects = Objects::new(10, 3);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, 10, &mut budget).unwrap();
    objects
        .define(&array, JsString::from("8"), number(8.0), &mut budget)
        .unwrap();
    objects
        .define(&array, JsString::from("9"), number(9.0), &mut budget)
        .unwrap();
    assert_eq!(
        objects.define(&array, JsString::from("10"), number(10.0), &mut budget),
        Err(Error::PropertyLimit)
    );
    assert_eq!(length(&objects, &array), (10, true));
    assert_eq!(
        define_length(
            &mut objects,
            &array,
            0.0,
            Some(false),
            &mut Budget::new(100)
        ),
        Err(Error::WorkLimit)
    );
    assert_eq!(length(&objects, &array), (10, true));
    assert_eq!(objects.inspect(&array).unwrap().property_count(), 3);
    let mut no_properties = Objects::new(1, 0);
    assert_eq!(
        no_properties.create_array(None, 0, &mut budget),
        Err(Error::PropertyLimit)
    );
    assert!(no_properties.create(None).is_ok());
}

#[test]
fn array_properties_trace_object_edges_and_release_truncated_elements() {
    let mut objects = Objects::new(10, 10);
    let mut budget = Budget::new(100_000);
    let array = objects.create_array(None, 0, &mut budget).unwrap();
    let element = objects.create(None).unwrap();
    objects
        .define(
            &array,
            JsString::from("0"),
            data(Value::Object(element.clone())),
            &mut budget,
        )
        .unwrap();
    assert_eq!(objects.collect([&array], 1000).unwrap().live, 2);
    assert!(objects.inspect(&element).is_ok());
    assert!(define_length(&mut objects, &array, 0.0, None, &mut budget).unwrap());
    assert_eq!(objects.collect([&array], 1000).unwrap().live, 1);
    assert!(objects.inspect(&element).is_err());
}

#[test]
fn fresh_array_initialization_preserves_descriptors_keys_length_and_traced_edges() {
    let mut objects = Objects::with_limits(None, None);
    let mut budget = Budget::with_work_limit(None);
    let child = objects.create(None).unwrap();
    let array = objects.create_array(None, 3, &mut budget).unwrap();
    objects
        .define(
            &array,
            JsString::from("groups"),
            data(Value::Undefined),
            &mut budget,
        )
        .unwrap();
    objects
        .initialize_array_elements(
            &array,
            vec![
                Value::Undefined,
                Value::Object(child.clone()),
                Value::Number(7.0),
            ],
            &mut budget,
        )
        .unwrap();
    assert_eq!(length(&objects, &array), (3, true));
    assert_eq!(
        objects.inspect(&array).unwrap().own_keys(),
        ["0", "1", "2", "length", "groups"].map(|key| PropertyKey::from(JsString::from(key)))
    );
    let property = objects
        .inspect(&array)
        .unwrap()
        .own_property(&JsString::from("0"))
        .unwrap()
        .as_data()
        .unwrap();
    assert_eq!(property.value, Value::Undefined);
    assert!(property.writable && property.enumerable && property.configurable);
    assert_eq!(objects.collect([&array], usize::MAX).unwrap().live, 2);
    assert!(objects.inspect(&child).is_ok());
    assert!(define_length(&mut objects, &array, 1.0, None, &mut budget).unwrap());
    assert_eq!(objects.collect([&array], usize::MAX).unwrap().live, 1);
    assert!(objects.inspect(&child).is_err());
    assert_eq!(
        objects.inspect(&array).unwrap().own_keys(),
        ["0", "length", "groups"].map(|key| PropertyKey::from(JsString::from(key)))
    );
}

#[test]
fn fresh_array_initialization_rejects_invalid_state_without_changing_properties() {
    let mut objects = Objects::with_limits(None, None);
    let mut budget = Budget::with_work_limit(None);
    for mode in 0..5 {
        let array = if mode == 0 {
            objects.create(None).unwrap()
        } else {
            objects.create_array(None, 1, &mut budget).unwrap()
        };
        match mode {
            1 => {
                objects.prevent_extensions(&array).unwrap();
            }
            2 => {
                assert!(
                    define_length(&mut objects, &array, 1.0, Some(false), &mut budget).unwrap()
                );
            }
            3 => {
                objects
                    .define(&array, JsString::from("0"), number(1.0), &mut budget)
                    .unwrap();
            }
            _ => {}
        }
        let before = objects.inspect(&array).unwrap().properties.clone();
        let values = if mode == 4 {
            vec![]
        } else {
            vec![Value::Undefined]
        };
        assert_eq!(
            objects.initialize_array_elements(&array, values, &mut budget),
            Err(Error::WrongKind)
        );
        assert_eq!(objects.inspect(&array).unwrap().properties, before);
    }
    let mut foreign = Objects::with_limits(None, None);
    let edge = foreign.create(None).unwrap();
    let array = objects.create_array(None, 1, &mut budget).unwrap();
    let before = objects.inspect(&array).unwrap().properties.clone();
    assert!(matches!(
        objects.initialize_array_elements(&array, vec![Value::Object(edge)], &mut budget),
        Err(Error::Heap(spite_heap::Error::ForeignHandle))
    ));
    assert_eq!(objects.inspect(&array).unwrap().properties, before);
    let edge = objects.create(None).unwrap();
    objects.collect([&array], usize::MAX).unwrap();
    assert!(matches!(
        objects.initialize_array_elements(&array, vec![Value::Object(edge)], &mut budget),
        Err(Error::Heap(spite_heap::Error::StaleHandle))
    ));
    assert_eq!(objects.inspect(&array).unwrap().properties, before);
}

#[test]
fn fresh_array_initialization_checks_optional_capacity_and_work_before_mutation() {
    for (properties, work, expected) in [
        (Some(2), None, Error::PropertyLimit),
        (None, Some(1), Error::WorkLimit),
    ] {
        let mut objects = Objects::with_limits(None, properties);
        let mut unlimited = Budget::with_work_limit(None);
        let array = objects.create_array(None, 2, &mut unlimited).unwrap();
        let before = objects.inspect(&array).unwrap().properties.clone();
        let mut budget = Budget::with_work_limit(work);
        assert_eq!(
            objects.initialize_array_elements(
                &array,
                vec![Value::Undefined, Value::Undefined],
                &mut budget
            ),
            Err(expected)
        );
        assert_eq!(objects.inspect(&array).unwrap().properties, before);
        assert_eq!(length(&objects, &array), (2, true));
    }
    let mut objects = Objects::with_limits(None, Some(3));
    let mut budget = Budget::new(100);
    let array = objects.create_array(None, 2, &mut budget).unwrap();
    objects
        .initialize_array_elements(
            &array,
            vec![Value::Undefined, Value::Undefined],
            &mut budget,
        )
        .unwrap();
    assert_eq!(objects.inspect(&array).unwrap().property_count(), 3);
}
