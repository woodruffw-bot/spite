//! TypedArray traversal, callbacks, and species-backed results (23.2.3).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, binary_data::ElementType};
use spite_core::{JsString, Span, WellKnownSymbol};

#[derive(Clone, Copy, Debug)]
pub(super) enum Method {
    At,
    Every,
    Some,
    ForEach,
    Find,
    FindIndex,
    FindLast,
    FindLastIndex,
    Includes,
    IndexOf,
    LastIndexOf,
    Join,
    ToLocaleString,
    Reduce,
    ReduceRight,
    Map,
    Filter,
    CopyWithin,
    Fill,
    Reverse,
    Set,
    Slice,
    Subarray,
    ToReversed,
    With,
    Sort,
    ToSorted,
}

impl Method {
    // The intrinsic module retains the existing metadata table and function
    // objects. Only invocation of implemented entries changes.
    pub(super) fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "at" => Self::At,
            "every" => Self::Every,
            "some" => Self::Some,
            "forEach" => Self::ForEach,
            "find" => Self::Find,
            "findIndex" => Self::FindIndex,
            "findLast" => Self::FindLast,
            "findLastIndex" => Self::FindLastIndex,
            "includes" => Self::Includes,
            "indexOf" => Self::IndexOf,
            "lastIndexOf" => Self::LastIndexOf,
            "join" => Self::Join,
            "toLocaleString" => Self::ToLocaleString,
            "reduce" => Self::Reduce,
            "reduceRight" => Self::ReduceRight,
            "map" => Self::Map,
            "filter" => Self::Filter,
            "copyWithin" => Self::CopyWithin,
            "fill" => Self::Fill,
            "reverse" => Self::Reverse,
            "set" => Self::Set,
            "slice" => Self::Slice,
            "subarray" => Self::Subarray,
            "toReversed" => Self::ToReversed,
            "with" => Self::With,
            "sort" => Self::Sort,
            "toSorted" => Self::ToSorted,
            _ => return None,
        })
    }
}

pub(super) struct TypedArrayWitness {
    pub(super) object: ObjectHandle,
    pub(super) element: ElementType,
    pub(super) length: u64,
}

impl Realm {
    pub(super) fn typed_array_method(
        &mut self,
        method: Method,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        match method {
            Method::At
            | Method::Includes
            | Method::IndexOf
            | Method::LastIndexOf
            | Method::Join
            | Method::ToLocaleString
            | Method::Reduce
            | Method::ReduceRight => {
                self.typed_array_read_method(method, receiver, arguments, span)
            }
            Method::CopyWithin | Method::Fill | Method::Reverse | Method::Set => {
                self.typed_array_mutation_method(method, receiver, arguments, span)
            }
            Method::Slice | Method::Subarray | Method::ToReversed | Method::With => {
                self.typed_array_copy_method(method, receiver, arguments, span)
            }
            Method::Sort | Method::ToSorted => {
                self.typed_array_sort_method(method, receiver, arguments, span)
            }
            _ => self.typed_array_callback_method(method, receiver, arguments, span),
        }
    }

    /// ValidateTypedArray (23.2.4.4), capturing internal length before any
    /// argument conversion. Later traversal uses live indexed Get/Has/Set.
    pub(super) fn typed_array_validate(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<TypedArrayWitness, Error> {
        let invalid = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a live, in-bounds TypedArray",
            )
        };
        let Value::Object(object) = receiver else {
            return Err(invalid());
        };
        let state = self.object_work(span, |objects, _| {
            let Some(data) = objects.inspect(&object)?.typed_array_data() else {
                return Ok(None);
            };
            let buffer = objects
                .inspect(data.buffer())?
                .array_buffer_data()
                .expect("TypedArray buffer");
            Ok(data
                .in_bounds_array_length(buffer)
                .map(|length| (data.element_type(), length)))
        })?;
        let (element, length) = state.ok_or_else(invalid)?;
        Ok(TypedArrayWitness {
            object,
            element,
            length,
        })
    }

    /// TypedArraySpeciesCreate (23.2.4.3) with one numeric count. The wrapper
    /// preserves map/filter/slice's checked minimum-length result contract.
    pub(super) fn typed_array_species_create(
        &mut self,
        exemplar: &ObjectHandle,
        element: ElementType,
        length: u64,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        self.typed_array_species_create_arguments(
            exemplar,
            element,
            vec![Value::Number(length as f64)],
            span,
        )
    }

    /// TypedArraySpeciesCreate with the exact constructor argument list.
    /// subarray supplies two or three arguments and has no count minimum.
    pub(super) fn typed_array_species_create_arguments(
        &mut self,
        exemplar: &ObjectHandle,
        element: ElementType,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let constructor = self.get_property(exemplar, &JsString::from("constructor"), span)?;
        let species = match constructor {
            Value::Undefined => Value::Undefined,
            Value::Object(constructor) => {
                self.get_property(&constructor, &WellKnownSymbol::Species.symbol(), span)?
            }
            _ => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "TypedArray constructor property must be an Object or undefined",
                ));
            }
        };
        let constructor = if matches!(species, Value::Null | Value::Undefined) {
            Value::Object(
                self.intrinsics
                    .as_ref()
                    .expect("initialized realm")
                    .typed_array
                    .constructor(element)
                    .clone(),
            )
        } else {
            if !self.is_constructor(&species, span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "TypedArray species is not a constructor",
                ));
            }
            species
        };
        let result = self.typed_array_factory_result_arguments(constructor, arguments, span)?;
        let result_is_bigint = self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(&result)?
                .typed_array_data()
                .expect("validated TypedArray result")
                .element_type()
                .is_bigint())
        })?;
        if result_is_bigint != element.is_bigint() {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray species result has a different content type",
            ));
        }
        Ok(result)
    }

    /// TypedArrayCreateSameType (23.2.4.2). Only the retained intrinsic
    /// constructor is used; public constructor/species hooks are ignored.
    pub(super) fn typed_array_create_same_type(
        &mut self,
        element: ElementType,
        length: u64,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let constructor = self
            .intrinsics
            .as_ref()
            .expect("initialized realm")
            .typed_array
            .constructor(element)
            .clone();
        self.typed_array_factory_result(Value::Object(constructor), length, span)
    }

    fn typed_array_callback_method(
        &mut self,
        method: Method,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let witness = self.typed_array_validate(receiver, span)?;
        let callback = arguments.next().unwrap_or(Value::Undefined);
        let this_arg = arguments.next().unwrap_or(Value::Undefined);
        if !self.is_callable(&callback, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray callback is not callable",
            ));
        }
        let Value::Object(callback) = callback else {
            unreachable!("callable object");
        };
        // Map creates its result before the first callback. Filter selects all
        // values before looking up constructor/species (23.2.3.10/.22).
        let output = if matches!(method, Method::Map) {
            Some(self.typed_array_species_create(
                &witness.object,
                witness.element,
                witness.length,
                span,
            )?)
        } else {
            None
        };
        let descending = matches!(method, Method::FindLast | Method::FindLastIndex);
        let retain_value = matches!(method, Method::Find | Method::FindLast | Method::Filter);
        let mut kept = Vec::new();
        let mut captured = 0u64;
        for offset in 0..witness.length {
            self.tick(span)?;
            let index = if descending {
                witness.length - 1 - offset
            } else {
                offset
            };
            let key = JsString::from(index.to_string().as_str());
            // No HasProperty or additional ValidateTypedArray: callbacks still
            // visit captured indices after source shrinkage or detachment.
            let value = self.get_property(&witness.object, &key, span)?;
            let retained = if retain_value {
                self.object_work(span, |_, budget| budget.value(&value))?;
                Some(value.clone())
            } else {
                None
            };
            self.object_work(span, |_, budget| budget.value(&this_arg))?;
            let result = self.call(
                Value::Object(callback.clone()),
                this_arg.clone(),
                vec![
                    value,
                    Value::Number(index as f64),
                    Value::Object(witness.object.clone()),
                ],
                span,
            )?;
            match method {
                Method::Every if !result.to_boolean() => return Ok(Value::Boolean(false)),
                Method::Some if result.to_boolean() => return Ok(Value::Boolean(true)),
                Method::Find | Method::FindLast if result.to_boolean() => {
                    return Ok(retained.expect("retained find value"));
                }
                Method::FindIndex | Method::FindLastIndex if result.to_boolean() => {
                    return Ok(Value::Number(index as f64));
                }
                Method::Map => self.set_property_or_throw(
                    output.as_ref().expect("map output"),
                    key,
                    result,
                    span,
                )?,
                Method::Filter if result.to_boolean() => {
                    let movement = if kept.len() == kept.capacity() {
                        kept.len()
                    } else {
                        0
                    };
                    self.object_work(span, |_, budget| {
                        budget.charge(movement)?;
                        budget.charge(1)
                    })?;
                    kept.try_reserve(1).map_err(|_| Error::Limit {
                        span,
                        message: "TypedArray filter list exceeds platform capacity".into(),
                    })?;
                    kept.push(retained.expect("retained filter value"));
                    // The selected count cannot exceed the safe captured length.
                    captured += 1;
                }
                _ => {}
            }
        }
        Ok(match method {
            Method::Every => Value::Boolean(true),
            Method::Some => Value::Boolean(false),
            Method::ForEach => Value::Undefined,
            Method::Find | Method::FindLast => Value::Undefined,
            Method::FindIndex | Method::FindLastIndex => Value::Number(-1.0),
            Method::Map => Value::Object(output.expect("map output")),
            Method::Filter => {
                let output = self.typed_array_species_create(
                    &witness.object,
                    witness.element,
                    captured,
                    span,
                )?;
                for (index, value) in kept.into_iter().enumerate() {
                    self.tick(span)?;
                    self.set_property_or_throw(
                        &output,
                        JsString::from(index.to_string().as_str()),
                        value,
                        span,
                    )?;
                }
                Value::Object(output)
            }
            _ => unreachable!("read methods dispatched separately"),
        })
    }
}
