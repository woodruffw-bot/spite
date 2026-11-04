use super::*;

fn define(object: &mut OrdinaryObject, key: impl Into<PropertyKey>, value: f64) {
    assert_eq!(
        object.define_own_property(
            key,
            DataDescriptor {
                value: Some(Value::Number(value)),
                writable: Some(true),
                enumerable: Some(true),
                configurable: Some(true),
            },
        ),
        Ok(true)
    );
}

fn value(object: &OrdinaryObject, key: &str) -> Option<Value> {
    match object.own_property(&JsString::from(key)) {
        Some(Property::Data(property)) => Some(property.value.clone()),
        None => None,
        _ => panic!("data property"),
    }
}

#[test]
fn contiguous_indices_survive_prefix_and_tail_deletion_without_duplicate_keys() {
    let mut object = OrdinaryObject::with_property_limit(None, None);
    define(&mut object, "length", 32.0);
    for index in 0..32 {
        define(&mut object, index.to_string().as_str(), f64::from(index));
    }
    assert!(object.delete(&JsString::from("length")));
    assert!(object.delete(&JsString::from("31")));
    define(&mut object, "31", 99.0);
    define(&mut object, "10", 100.0);
    assert_eq!(object.property_count(), 32);
    assert_eq!(value(&object, "10"), Some(Value::Number(100.0)));
    assert_eq!(value(&object, "31"), Some(Value::Number(99.0)));
    assert_eq!(value(&object, "32"), None);
    for index in 0..31 {
        if index != 10 {
            assert_eq!(
                value(&object, &index.to_string()),
                Some(Value::Number(f64::from(index)))
            );
        }
    }
    assert!(object.delete(&JsString::from("15")));
    define(&mut object, "16", 160.0);
    assert_eq!(object.property_count(), 31);
    assert_eq!(value(&object, "16"), Some(Value::Number(160.0)));
    assert_eq!(value(&object, "31"), Some(Value::Number(99.0)));
}

#[test]
fn gaps_named_suffixes_and_failed_insertions_preserve_lookup_and_key_order() {
    let mut object = OrdinaryObject::with_property_limit(None, Some(5));
    define(&mut object, "0", 0.0);
    define(&mut object, "1", 1.0);
    define(&mut object, "2", 2.0);
    define(&mut object, "tail", 7.0);
    assert!(object.delete(&JsString::from("1")));
    define(&mut object, "2", 20.0);
    define(&mut object, "1", 10.0);
    define(&mut object, "4294967295", 8.0);
    assert_eq!(
        object.define_own_property("3", DataDescriptor::default()),
        Err(PropertyLimit)
    );
    assert_eq!(value(&object, "3"), None);
    assert_eq!(value(&object, "2"), Some(Value::Number(20.0)));
    assert_eq!(value(&object, "1"), Some(Value::Number(10.0)));
    assert_eq!(
        object.own_keys(),
        ["0", "1", "2", "tail", "4294967295"]
            .map(PropertyKey::from)
            .to_vec()
    );
    object.prevent_extensions();
    assert!(object.delete(&JsString::from("tail")));
    assert_eq!(
        object.define_own_property("3", DataDescriptor::default()),
        Ok(false)
    );
    assert_eq!(value(&object, "2"), Some(Value::Number(20.0)));
}
