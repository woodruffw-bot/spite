//! Integer-indexed storage and live TypedArray buffer witnesses (10.4.5).
//!
//! Observable ToNumber/ToBigInt conversions belong to the Realm. Storage returns
//! continuations before those conversions and checks the live buffer again when
//! the normalized value is committed. Constructor algorithms live in the Realm.

use super::{
    ArrayBufferError, Budget, DataProperty, DescriptorKind, Error, Objects, Property,
    PropertyDescriptor,
};
use crate::{Value, binary_data::ElementType, value::number_to_string};
use spite_core::{JsString, PropertyKey, PropertyKeyRef};
use spite_heap::Handle;

/// The buffer, element kind, byte offset, and fixed or auto array-length slots.
/// The byte length and Number/BigInt content type follow from these slots.
#[derive(Debug)]
pub(crate) struct TypedArrayData {
    buffer: Handle,
    element_type: ElementType,
    byte_offset: u64,
    array_length: Option<u64>,
}

impl TypedArrayData {
    pub(crate) fn buffer(&self) -> &Handle {
        &self.buffer
    }

    pub(crate) fn element_type(&self) -> ElementType {
        self.element_type
    }

    pub(crate) fn byte_offset(&self) -> u64 {
        self.byte_offset
    }

    pub(crate) fn array_length(&self) -> Option<u64> {
        self.array_length
    }

    /// IsTypedArrayOutOfBounds and TypedArrayLength from one live buffer state.
    /// Auto lengths round down; an incomplete trailing element is not exposed.
    pub(crate) fn in_bounds_array_length(
        &self,
        buffer: &impl super::array_buffer::BufferBounds,
    ) -> Option<u64> {
        if buffer.is_detached() {
            return None;
        }
        let available = buffer.byte_length().checked_sub(self.byte_offset())?;
        let element_size = self.element_type().size() as u64;
        match self.array_length() {
            Some(length) if length.checked_mul(element_size)? <= available => Some(length),
            Some(_) => None,
            None => Some(available / element_size),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TypedArraySetAction {
    Ordinary,
    Done(bool),
    Element { index: f64 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TypedArrayDefineAction {
    Ordinary,
    Done(bool),
    Element { index: f64 },
}

fn buffer_error(error: ArrayBufferError) -> Error {
    match error {
        ArrayBufferError::Storage(error) => error,
        ArrayBufferError::Capacity => Error::Heap(spite_heap::Error::Capacity),
        ArrayBufferError::Detached
        | ArrayBufferError::NotResizable
        | ArrayBufferError::OutOfBounds
        | ArrayBufferError::SameBuffer => Error::WrongKind,
    }
}

/// CanonicalNumericIndexString (7.1.21), including its special negative zero.
fn canonical_numeric_index(
    key: PropertyKeyRef<'_>,
    budget: &mut Budget,
) -> Result<Option<f64>, Error> {
    let Some(string) = key.as_string() else {
        return Ok(None);
    };
    budget.charge(1)?;
    // Number::toString uses at most 17 significant decimal digits. Its longest
    // fixed form is a minus sign, "0.00000", and 17 digits (25 code units);
    // scientific notation needs at most 24. Longer keys cannot be canonical.
    // This output bound avoids allocating or scanning an arbitrary property key.
    if string.len() > 25 {
        return Ok(None);
    }
    budget.charge(string.len())?;
    let mut ascii = [0; 25];
    for (unit, byte) in string.code_units().iter().zip(&mut ascii) {
        if *unit > 0x7f {
            return Ok(None);
        }
        *byte = *unit as u8;
    }
    let bytes = &ascii[..string.len()];
    if bytes == b"-0" {
        return Ok(Some(-0.0));
    }
    // Reuse the Realm's existing String ToNumber and Number::toString rules.
    // The input is now at most 25 ASCII units, so their temporary allocations
    // are bounded; Rust-specific spellings never determine canonicality.
    let number = Value::String(string.clone())
        .to_number()
        .expect("String ToNumber never requires Realm context");
    Ok((number_to_string(number).as_bytes() == bytes).then_some(number))
}

impl Objects {
    /// Creates a view after the Realm has validated constructor arguments.
    pub(crate) fn create_typed_array(
        &mut self,
        prototype: &Handle,
        buffer: &Handle,
        element_type: ElementType,
        byte_offset: u64,
        array_length: Option<u64>,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        let buffer_data = self
            .inspect(buffer)?
            .buffer_info()?
            .ok_or(Error::WrongKind)?;
        let data = TypedArrayData {
            buffer: buffer.clone(),
            element_type,
            byte_offset,
            array_length,
        };
        if !byte_offset.is_multiple_of(element_type.size() as u64)
            || (array_length.is_none() && !buffer_data.is_resizable())
            || data.in_bounds_array_length(&buffer_data).is_none()
        {
            return Err(Error::WrongKind);
        }
        let view = self.create(Some(prototype))?;
        self.object_mut(&view)?.typed_array = Some(data);
        Ok(view)
    }

    /// Returns every canonical numeric key on a TypedArray, valid or invalid.
    pub(crate) fn typed_array_numeric_index<'key>(
        &self,
        object: &Handle,
        key: impl Into<PropertyKeyRef<'key>>,
        budget: &mut Budget,
    ) -> Result<Option<f64>, Error> {
        if self.inspect(object)?.typed_array_data().is_none() {
            return Ok(None);
        }
        canonical_numeric_index(key.into(), budget)
    }

    /// IsValidIntegerIndex: this does not read an element or allocate a BigInt.
    pub(crate) fn typed_array_has_index(
        &self,
        object: &Handle,
        index: f64,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        budget.charge(1)?;
        let data = self
            .inspect(object)?
            .typed_array_data()
            .ok_or(Error::WrongKind)?;
        let buffer = self
            .inspect(data.buffer())?
            .buffer_info()?
            .ok_or(Error::WrongKind)?;
        Ok(index.is_finite()
            && index.trunc() == index
            && !index.is_sign_negative()
            && data
                .in_bounds_array_length(&buffer)
                .is_some_and(|length| index < length as f64))
    }

    pub(crate) fn typed_array_get_element(
        &self,
        object: &Handle,
        index: f64,
        budget: &mut Budget,
    ) -> Result<Value, Error> {
        if !self.typed_array_has_index(object, index, budget)? {
            return Ok(Value::Undefined);
        }
        let data = self
            .inspect(object)?
            .typed_array_data()
            .ok_or(Error::WrongKind)?;
        let size = data.element_type().size();
        let byte_index = (index as u64)
            .checked_mul(size as u64)
            .and_then(|offset| data.byte_offset().checked_add(offset))
            .ok_or(Error::WrongKind)?;
        let bytes = self
            .read_array_buffer_bytes(data.buffer(), byte_index, size, budget)
            .map_err(buffer_error)?;
        budget.integer(|budget| {
            data.element_type()
                .decode(bytes, cfg!(target_endian = "little"), budget)
        })
    }

    pub(crate) fn typed_array_get_own_element(
        &self,
        object: &Handle,
        index: f64,
        budget: &mut Budget,
    ) -> Result<Option<Property>, Error> {
        let value = self.typed_array_get_element(object, index, budget)?;
        Ok(
            (value != Value::Undefined).then_some(Property::Data(DataProperty {
                value,
                writable: true,
                enumerable: true,
                configurable: true,
            })),
        )
    }

    pub(crate) fn typed_array_set_action<'key>(
        &self,
        object: &Handle,
        key: impl Into<PropertyKeyRef<'key>>,
        receiver: Option<&Handle>,
        budget: &mut Budget,
    ) -> Result<TypedArraySetAction, Error> {
        let Some(index) = self.typed_array_numeric_index(object, key, budget)? else {
            return Ok(TypedArraySetAction::Ordinary);
        };
        if receiver == Some(object) {
            // TypedArraySetElement converts even a NaN, -0, or detached index.
            return Ok(TypedArraySetAction::Element { index });
        }
        if !self.typed_array_has_index(object, index, budget)? {
            return Ok(TypedArraySetAction::Done(true));
        }
        Ok(TypedArraySetAction::Ordinary)
    }

    pub(crate) fn typed_array_define_action<'key>(
        &self,
        object: &Handle,
        key: impl Into<PropertyKeyRef<'key>>,
        descriptor: &PropertyDescriptor,
        budget: &mut Budget,
    ) -> Result<TypedArrayDefineAction, Error> {
        let Some(index) = self.typed_array_numeric_index(object, key, budget)? else {
            return Ok(TypedArrayDefineAction::Ordinary);
        };
        if !self.typed_array_has_index(object, index, budget)?
            || descriptor.configurable == Some(false)
            || descriptor.enumerable == Some(false)
            || matches!(&descriptor.kind, DescriptorKind::Accessor { get, set } if get.is_some() || set.is_some())
            || matches!(
                descriptor.kind,
                DescriptorKind::Data {
                    writable: Some(false),
                    ..
                }
            )
        {
            return Ok(TypedArrayDefineAction::Done(false));
        }
        Ok(
            if matches!(descriptor.kind, DescriptorKind::Data { value: Some(_), .. }) {
                TypedArrayDefineAction::Element { index }
            } else {
                TypedArrayDefineAction::Done(true)
            },
        )
    }

    /// Commits a Realm-normalized value after checking the current buffer again.
    /// Invalid indices are successful no-ops; the caller has already coerced.
    pub(crate) fn typed_array_set_element(
        &mut self,
        object: &Handle,
        index: f64,
        value: Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let data = self
            .inspect(object)?
            .typed_array_data()
            .ok_or(Error::WrongKind)?;
        let element = data.element_type();
        if !matches!(
            (&value, element.is_bigint()),
            (Value::BigInt(_), true) | (Value::Number(_), false)
        ) {
            return Err(Error::UnnormalizedTypedArrayElement);
        }
        if !self.typed_array_has_index(object, index, budget)? {
            return Ok(());
        }
        let data = self
            .inspect(object)?
            .typed_array_data()
            .ok_or(Error::WrongKind)?;
        let buffer = data.buffer().clone();
        let byte_index = (index as u64)
            .checked_mul(element.size() as u64)
            .and_then(|offset| data.byte_offset().checked_add(offset))
            .ok_or(Error::WrongKind)?;
        let bytes = budget
            .integer(|budget| element.encode(value, cfg!(target_endian = "little"), budget))?;
        self.write_array_buffer_bytes(&buffer, byte_index, &bytes[..element.size()], budget)
            .map_err(buffer_error)
    }

    pub(crate) fn typed_array_own_keys(
        &self,
        object: &Handle,
        budget: &mut Budget,
    ) -> Result<Vec<PropertyKey>, Error> {
        let record = self.inspect(object)?;
        let data = record.typed_array_data().ok_or(Error::WrongKind)?;
        let buffer = self
            .inspect(data.buffer())?
            .buffer_info()?
            .ok_or(Error::WrongKind)?;
        let length = data.in_bounds_array_length(&buffer).unwrap_or(0);
        let capacity = usize::try_from(length)
            .ok()
            .and_then(|length| length.checked_add(record.property_count()))
            .ok_or(Error::Heap(spite_heap::Error::Capacity))?;
        budget.charge(capacity.checked_add(1).ok_or(Error::WorkLimit)?)?;
        let mut keys = Vec::new();
        keys.try_reserve_exact(capacity)
            .map_err(|_| Error::Heap(spite_heap::Error::Capacity))?;
        for index in 0..length {
            let key = index.to_string();
            budget.charge(key.len())?;
            keys.push(PropertyKey::from(JsString::from(key.as_str())));
        }
        // All stored string keys are noncanonical: indexed definitions never
        // enter the ordinary property vector. Preserve their creation order.
        for (key, _) in record
            .properties
            .iter()
            .filter(|(key, _)| key.as_string().is_some())
        {
            budget.charge(key.as_string().expect("string key").len())?;
            keys.push(key.clone());
        }
        keys.extend(
            record
                .properties
                .iter()
                .filter_map(|(key, _)| key.as_symbol().map(|_| key.clone())),
        );
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::{DataDescriptor, GetAction, SetAction};
    use spite_bigint::BigInt;
    use spite_core::JsSymbol;

    fn unlimited() -> Budget {
        Budget::with_work_limit(None)
    }

    fn fixture(
        bytes: u64,
        maximum: Option<u64>,
        element: ElementType,
        offset: u64,
        length: Option<u64>,
    ) -> (Objects, Handle, Handle, Handle) {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let buffer = objects
            .create_array_buffer(&prototype, bytes, maximum, &mut unlimited())
            .unwrap();
        let view = objects
            .create_typed_array(&prototype, &buffer, element, offset, length)
            .unwrap();
        (objects, prototype, buffer, view)
    }

    fn data(value: Value) -> DataDescriptor {
        DataDescriptor {
            value: Some(value),
            ..Default::default()
        }
    }

    #[test]
    fn canonical_indices_use_number_spelling_and_a_bounded_ascii_input() {
        for key in [
            "0",
            "1",
            "1.5",
            "-1",
            "NaN",
            "Infinity",
            "-Infinity",
            "1e+21",
            "1e-7",
            "0.000001",
            "100000000000000000000",
            "9007199254740991",
            "9007199254740992",
        ] {
            assert!(
                canonical_numeric_index(
                    PropertyKeyRef::from(&JsString::from(key)),
                    &mut unlimited()
                )
                .unwrap()
                .is_some(),
                "{key}"
            );
        }
        let negative_zero = JsString::from("-0");
        let index = canonical_numeric_index((&negative_zero).into(), &mut unlimited())
            .unwrap()
            .unwrap();
        assert_eq!(index, 0.0);
        assert!(index.is_sign_negative());
        let longest = JsString::from("-0.0000012345678901234567");
        assert_eq!(longest.len(), 25);
        assert!(
            canonical_numeric_index((&longest).into(), &mut unlimited())
                .unwrap()
                .is_some()
        );
        for key in [
            "",
            "01",
            "+0",
            "-00",
            "1.0",
            "1e21",
            "0x1",
            " 1",
            "1 ",
            ".5",
            "nan",
            "+Infinity",
            "1e+20",
            "1e-6",
            "0.0000001",
            "9007199254740993",
        ] {
            assert!(
                canonical_numeric_index(
                    PropertyKeyRef::from(&JsString::from(key)),
                    &mut unlimited()
                )
                .unwrap()
                .is_none(),
                "{key}"
            );
        }
        let long = JsString::from("1".repeat(10_000).as_str());
        assert!(
            canonical_numeric_index((&long).into(), &mut Budget::new(1))
                .unwrap()
                .is_none()
        );
        let (mut objects, _, _, view) = fixture(1, None, ElementType::Uint8, 0, Some(1));
        assert!(
            objects
                .define(
                    &view,
                    long.clone(),
                    data(Value::Number(9.0)),
                    &mut unlimited()
                )
                .unwrap()
        );
        assert_eq!(
            objects.get(&view, &long, &mut unlimited()).unwrap(),
            GetAction::Value(Value::Number(9.0))
        );
        assert!(
            objects
                .own_keys(&view, &mut unlimited())
                .unwrap()
                .contains(&PropertyKey::from(long))
        );
        let surrogate = JsString::from_code_units(vec![0xd800]);
        assert!(
            canonical_numeric_index((&surrogate).into(), &mut unlimited())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn all_element_types_read_and_write_shared_bytes_at_their_native_width() {
        for (element, number, bits) in [
            (ElementType::Int8, -1.0, 0xff),
            (ElementType::Uint8, 255.0, 0xff),
            (ElementType::Uint8Clamped, 2.5, 2),
            (ElementType::Int16, -2.0, 0xfffe),
            (ElementType::Uint16, 258.0, 0x0102),
            (ElementType::Int32, -2.0, 0xffff_fffe),
            (ElementType::Uint32, 0x0102_0304 as f64, 0x0102_0304),
            (ElementType::Float16, 1.0, 0x3c00),
            (ElementType::Float32, 1.0, 0x3f80_0000),
            (ElementType::Float64, 1.0, 0x3ff0_0000_0000_0000),
            (ElementType::BigInt64, -1.0, u64::MAX),
            (ElementType::BigUint64, 258.0, 258),
        ] {
            let (mut objects, _, buffer, view) = fixture(24, None, element, 8, Some(2));
            let value = if element.is_bigint() {
                let integer = if element == ElementType::BigInt64 {
                    BigInt::from_i64(-1, &mut spite_bigint::Budget::with_limits(None, None))
                } else {
                    BigInt::from_u64(258, &mut spite_bigint::Budget::with_limits(None, None))
                };
                Value::BigInt(integer.unwrap())
            } else {
                Value::Number(number)
            };
            assert!(
                objects
                    .define(
                        &view,
                        JsString::from("1"),
                        data(value.clone()),
                        &mut unlimited()
                    )
                    .unwrap()
            );
            let size = element.size();
            let mut expected = bits.to_le_bytes();
            expected[size..].fill(0);
            if !cfg!(target_endian = "little") {
                expected[..size].reverse();
            }
            assert_eq!(
                objects
                    .read_array_buffer_bytes(&buffer, 8 + size as u64, size, &mut unlimited())
                    .unwrap(),
                expected,
                "{element:?}"
            );
            assert_eq!(
                objects
                    .typed_array_get_element(&view, 0.0, &mut unlimited())
                    .unwrap(),
                if element.is_bigint() {
                    Value::BigInt(BigInt::default())
                } else {
                    Value::Number(0.0)
                }
            );
            let expected = if element == ElementType::Uint8Clamped {
                Value::Number(2.0)
            } else {
                value
            };
            assert_eq!(
                objects
                    .get(&view, &JsString::from("1"), &mut unlimited())
                    .unwrap(),
                GetAction::Value(expected)
            );
        }
    }

    #[test]
    fn live_witnesses_floor_auto_lengths_and_recover_fixed_slots_after_resize() {
        let (mut objects, prototype, buffer, fixed) =
            fixture(10, Some(20), ElementType::Uint16, 4, Some(3));
        let auto = objects
            .create_typed_array(&prototype, &buffer, ElementType::Uint16, 4, None)
            .unwrap();
        for (bytes, fixed_length, auto_length) in [
            (9, None, Some(2)),
            (4, None, Some(0)),
            (3, None, None),
            (15, Some(3), Some(5)),
        ] {
            objects
                .resize_array_buffer(&buffer, bytes, &mut unlimited())
                .unwrap();
            let data = objects
                .inspect(&buffer)
                .unwrap()
                .array_buffer_data()
                .unwrap();
            for (view, expected) in [(&fixed, fixed_length), (&auto, auto_length)] {
                assert_eq!(
                    objects
                        .inspect(view)
                        .unwrap()
                        .typed_array_data()
                        .unwrap()
                        .in_bounds_array_length(data),
                    expected
                );
            }
        }
        assert!(!objects.prevent_extensions(&fixed).unwrap());
        assert!(!objects.prevent_extensions(&auto).unwrap());
        assert!(objects.inspect(&fixed).unwrap().is_extensible());
        objects
            .detach_array_buffer(&buffer, &mut unlimited())
            .unwrap();
        assert!(
            !objects
                .has(&fixed, &JsString::from("0"), &mut unlimited())
                .unwrap()
        );
        assert!(
            objects
                .own_keys(&auto, &mut unlimited())
                .unwrap()
                .is_empty()
        );
        let maximum = 9_007_199_254_740_991;
        let high = objects
            .create_array_buffer(&prototype, maximum, None, &mut unlimited())
            .unwrap();
        let tail = objects
            .create_typed_array(&prototype, &high, ElementType::Uint8, maximum - 1, Some(1))
            .unwrap();
        objects
            .typed_array_set_element(&tail, 0.0, Value::Number(23.0), &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .read_array_buffer_bytes(&high, maximum - 1, 1, &mut unlimited())
                .unwrap()[0],
            23
        );
        for (offset, length) in [
            (1, Some(1)),
            (u64::MAX, Some(1)),
            (0, Some(u64::MAX)),
            (0, None),
        ] {
            assert_eq!(
                objects.create_typed_array(&prototype, &high, ElementType::Uint16, offset, length),
                Err(Error::WrongKind)
            );
        }
    }

    #[test]
    fn virtual_elements_have_exotic_descriptors_and_block_numeric_prototypes() {
        let (mut objects, prototype, _, view) = fixture(4, None, ElementType::Uint8, 0, Some(4));
        for key in ["4", "-0", "NaN", "Infinity", "1.5", "01"] {
            objects
                .define(
                    &prototype,
                    JsString::from(key),
                    DataDescriptor {
                        value: Some(Value::Number(99.0)),
                        writable: Some(true),
                        configurable: Some(true),
                        ..Default::default()
                    },
                    &mut unlimited(),
                )
                .unwrap();
        }
        for key in ["4", "-0", "NaN", "Infinity", "1.5"] {
            assert_eq!(
                objects
                    .get(&view, &JsString::from(key), &mut unlimited())
                    .unwrap(),
                GetAction::Value(Value::Undefined)
            );
            assert!(
                !objects
                    .has(&view, &JsString::from(key), &mut unlimited())
                    .unwrap()
            );
            assert!(
                objects
                    .delete(&view, &JsString::from(key), &mut unlimited())
                    .unwrap()
            );
        }
        assert_eq!(
            objects
                .get(&view, &JsString::from("01"), &mut unlimited())
                .unwrap(),
            GetAction::Value(Value::Number(99.0))
        );
        assert!(
            objects
                .has(&view, &JsString::from("01"), &mut unlimited())
                .unwrap()
        );
        let symbol = JsSymbol::new(None);
        for key in [
            PropertyKey::from(JsString::from("01")),
            PropertyKey::from(symbol.clone()),
            PropertyKey::from(JsString::from("name")),
        ] {
            assert!(
                objects
                    .define(
                        &view,
                        key,
                        DataDescriptor {
                            value: Some(Value::Number(7.0)),
                            writable: Some(true),
                            enumerable: Some(true),
                            configurable: Some(true)
                        },
                        &mut unlimited()
                    )
                    .unwrap()
            );
        }
        assert!(
            objects
                .define(
                    &view,
                    JsString::from("0"),
                    data(Value::Number(5.0)),
                    &mut unlimited()
                )
                .unwrap()
        );
        assert_eq!(
            objects
                .get_own(&view, &JsString::from("0"), &mut unlimited())
                .unwrap(),
            Some(Property::Data(DataProperty {
                value: Value::Number(5.0),
                writable: true,
                enumerable: true,
                configurable: true
            }))
        );
        assert!(
            !objects
                .delete(&view, &JsString::from("0"), &mut unlimited())
                .unwrap()
        );
        assert!(
            !objects
                .define(
                    &view,
                    JsString::from("0"),
                    DataDescriptor {
                        configurable: Some(false),
                        ..Default::default()
                    },
                    &mut unlimited()
                )
                .unwrap()
        );
        let keys = objects.own_keys(&view, &mut unlimited()).unwrap();
        let expected: Vec<PropertyKey> = ["0", "1", "2", "3", "01", "name"]
            .map(|key| JsString::from(key).into())
            .into_iter()
            .chain([symbol.into()])
            .collect();
        assert_eq!(keys, expected);
        assert!(objects.prevent_extensions(&view).unwrap());
        assert!(
            !objects
                .define(
                    &view,
                    JsString::from("new"),
                    DataDescriptor::default(),
                    &mut unlimited()
                )
                .unwrap()
        );
        assert!(
            objects
                .define(
                    &view,
                    JsString::from("1"),
                    data(Value::Number(8.0)),
                    &mut unlimited()
                )
                .unwrap()
        );
    }

    #[test]
    fn write_preflight_distinguishes_receivers_and_rechecks_live_bounds() {
        let (mut objects, prototype, buffer, view) =
            fixture(4, Some(8), ElementType::Uint8, 0, Some(4));
        let value = Value::Object(prototype.clone());
        assert!(matches!(
            objects.set(&view, JsString::from("NaN"), value.clone(), Some(&view), &mut unlimited()).unwrap(),
            SetAction::TypedArrayElement { target, index, value: original } if target == view && index.is_nan() && original == value
        ));
        assert_eq!(
            objects
                .typed_array_set_action(
                    &view,
                    &JsString::from("NaN"),
                    Some(&prototype),
                    &mut unlimited()
                )
                .unwrap(),
            TypedArraySetAction::Done(true)
        );
        assert_eq!(
            objects
                .set(
                    &view,
                    JsString::from("NaN"),
                    value.clone(),
                    Some(&prototype),
                    &mut unlimited()
                )
                .unwrap(),
            SetAction::Done(true)
        );
        assert!(
            !objects
                .has_own(&prototype, &JsString::from("NaN"), &mut unlimited())
                .unwrap()
        );
        let descriptor: PropertyDescriptor = data(value.clone()).into();
        assert_eq!(
            objects
                .typed_array_define_action(
                    &view,
                    &JsString::from("0"),
                    &descriptor,
                    &mut unlimited()
                )
                .unwrap(),
            TypedArrayDefineAction::Element { index: 0.0 }
        );
        assert_eq!(
            objects.define(&view, JsString::from("0"), descriptor, &mut unlimited()),
            Err(Error::UnnormalizedTypedArrayElement)
        );
        for descriptor in [
            PropertyDescriptor {
                configurable: Some(false),
                ..Default::default()
            },
            PropertyDescriptor {
                enumerable: Some(false),
                ..Default::default()
            },
            DataDescriptor {
                value: Some(value.clone()),
                writable: Some(false),
                ..Default::default()
            }
            .into(),
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(None),
                    set: None,
                },
                ..Default::default()
            },
        ] {
            assert!(
                !objects
                    .define(&view, JsString::from("0"), descriptor, &mut unlimited())
                    .unwrap()
            );
        }
        objects
            .resize_array_buffer(&buffer, 1, &mut unlimited())
            .unwrap();
        objects
            .typed_array_set_element(&view, 0.0, Value::Number(9.0), &mut unlimited())
            .unwrap();
        objects
            .resize_array_buffer(&buffer, 4, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .typed_array_get_element(&view, 0.0, &mut unlimited())
                .unwrap(),
            Value::Number(0.0)
        );
        let ordinary = objects.create(None).unwrap();
        objects
            .define(
                &ordinary,
                JsString::from("0"),
                DataDescriptor {
                    value: Some(Value::Number(3.0)),
                    writable: Some(true),
                    ..Default::default()
                },
                &mut unlimited(),
            )
            .unwrap();
        assert!(
            matches!(objects.set(&ordinary, JsString::from("0"), value, Some(&view), &mut unlimited()).unwrap(), SetAction::TypedArrayElement { target, index: 0.0, .. } if target == view)
        );
    }

    #[test]
    fn integer_limits_are_shared_without_allocating_values_for_presence_checks() {
        let (mut objects, _, buffer, view) = fixture(8, None, ElementType::BigUint64, 0, Some(1));
        let value = Value::BigInt(
            BigInt::from_u64(u64::MAX, &mut spite_bigint::Budget::with_limits(None, None)).unwrap(),
        );
        objects
            .typed_array_set_element(&view, 0.0, value.clone(), &mut unlimited())
            .unwrap();
        assert!(
            objects
                .has(
                    &view,
                    &JsString::from("0"),
                    &mut Budget::with_limits(Some(8), None)
                )
                .unwrap()
        );
        assert_eq!(
            objects.get(
                &view,
                &JsString::from("0"),
                &mut Budget::with_limits(Some(8), None)
            ),
            Err(Error::Integer(spite_bigint::Error::Limit))
        );
        assert_eq!(
            objects
                .get(&view, &JsString::from("0"), &mut unlimited())
                .unwrap(),
            GetAction::Value(value)
        );
        let mut budget = Budget::with_limits(Some(8), Some(10));
        assert_eq!(
            budget.integer::<()>(|inner| {
                inner.charge(3)?;
                Err(spite_bigint::Error::Limit)
            }),
            Err(Error::Integer(spite_bigint::Error::Limit))
        );
        assert_eq!(budget.remaining_work(), Some(7));
        assert!(
            Budget::new(10)
                .integer(|inner| BigInt::from_u64(u64::MAX, inner))
                .is_ok()
        );
        objects
            .detach_array_buffer(&buffer, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects
                .get(
                    &view,
                    &JsString::from("0"),
                    &mut Budget::with_limits(Some(0), None)
                )
                .unwrap(),
            GetAction::Value(Value::Undefined)
        );
    }

    #[test]
    fn buffer_edges_are_traced_and_invalid_view_edges_never_allocate() {
        let (mut objects, prototype, buffer, view) =
            fixture(8, None, ElementType::Uint16, 0, Some(4));
        let unrelated = objects.create(None).unwrap();
        let mut foreign_heap = Objects::with_limits(None, None);
        let foreign = foreign_heap.create(None).unwrap();
        for (proto, source) in [(&foreign, &buffer), (&prototype, &foreign)] {
            assert_eq!(
                objects.create_typed_array(proto, source, ElementType::Uint8, 0, Some(0)),
                Err(Error::Heap(spite_heap::Error::ForeignHandle))
            );
        }
        assert_eq!(
            objects.create_typed_array(&prototype, &unrelated, ElementType::Uint8, 0, Some(0)),
            Err(Error::WrongKind)
        );
        let collection = objects.collect([&view], 100).unwrap();
        assert_eq!((collection.live, collection.reclaimed), (3, 1));
        assert!(objects.inspect(&buffer).is_ok());
        assert!(matches!(
            objects.inspect(&unrelated),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
        objects
            .detach_array_buffer(&buffer, &mut unlimited())
            .unwrap();
        assert_eq!(
            objects.create_typed_array(&prototype, &buffer, ElementType::Uint8, 0, Some(0)),
            Err(Error::WrongKind)
        );
        assert_eq!(objects.collect([&prototype], 100).unwrap().reclaimed, 2);
    }
}
