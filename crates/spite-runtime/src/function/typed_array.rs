//! TypedArray construction and shared construction factories (23.2.2, 23.2.5.1).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    binary_data::ElementType,
    object::{ArrayBufferError, Budget, Objects},
};
use spite_core::{JsString, Span, WellKnownSymbol};

const MAX_BYTE_LENGTH: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy)]
enum SourceKind {
    TypedArray,
    ArrayBuffer,
    Ordinary,
}

impl Realm {
    pub(super) fn typed_array_constructor(
        &mut self,
        element: ElementType,
        new_target: ObjectHandle,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let mut arguments = arguments.into_iter();
        let Some(first) = arguments.next() else {
            let prototype = self.typed_array_constructor_prototype(element, &new_target, span)?;
            return self
                .allocate_typed_array(element, &prototype, 0, span)
                .map(Value::Object);
        };
        let Value::Object(source) = first else {
            // Primitive ToIndex precedes GetPrototypeFromConstructor; the
            // Object overloads have the opposite order (23.2.5.1).
            let length = self.typed_array_to_index(first, span)?;
            let prototype = self.typed_array_constructor_prototype(element, &new_target, span)?;
            return self
                .allocate_typed_array(element, &prototype, length, span)
                .map(Value::Object);
        };
        let prototype = self.typed_array_constructor_prototype(element, &new_target, span)?;
        let source_kind = self.object_work(span, |objects, _| {
            let record = objects.inspect(&source)?;
            Ok(if record.typed_array_data().is_some() {
                SourceKind::TypedArray
            } else if record.array_buffer_data().is_some() {
                SourceKind::ArrayBuffer
            } else {
                SourceKind::Ordinary
            })
        })?;
        let result = match source_kind {
            SourceKind::TypedArray => {
                self.typed_array_from_typed_array(element, &prototype, &source, span)?
            }
            SourceKind::ArrayBuffer => self.typed_array_from_buffer(
                element,
                &prototype,
                &source,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            )?,
            SourceKind::Ordinary => {
                let receiver = Value::Object(source.clone());
                if let Some(method) =
                    self.get_method(&receiver, &WellKnownSymbol::Iterator.symbol(), span)?
                {
                    let values = self.typed_array_iterator_list(receiver, method, span)?;
                    let length = Self::typed_array_list_length(values.len(), span)?;
                    let result = self.allocate_typed_array(element, &prototype, length, span)?;
                    for (index, value) in values.into_iter().enumerate() {
                        self.tick(span)?;
                        self.set_property_or_throw(
                            &result,
                            JsString::from(index.to_string().as_str()),
                            value,
                            span,
                        )?;
                    }
                    result
                } else {
                    let length = self.length_of_array_like(&source, span)?;
                    let result = self.allocate_typed_array(element, &prototype, length, span)?;
                    for index in 0..length {
                        self.tick(span)?;
                        let key = JsString::from(index.to_string().as_str());
                        let value = self.get_property(&source, &key, span)?;
                        self.set_property_or_throw(&result, key, value, span)?;
                    }
                    result
                }
            }
        };
        Ok(Value::Object(result))
    }

    fn typed_array_constructor_prototype(
        &mut self,
        element: ElementType,
        new_target: &ObjectHandle,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let prototype = self.get_property(new_target, &JsString::from("prototype"), span)?;
        Ok(if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.intrinsics
                .as_ref()
                .expect("initialized realm")
                .typed_array
                .prototype(element)
                .clone()
        })
    }

    fn typed_array_to_index(&mut self, value: Value, span: Span) -> Result<u64, Error> {
        let number = self.number(value, span)?;
        let integer = if number.is_nan() { 0.0 } else { number.trunc() };
        if !(0.0..=MAX_BYTE_LENGTH as f64).contains(&integer) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "TypedArray index is outside the safe index range",
            ));
        }
        Ok(integer as u64)
    }

    fn typed_array_byte_length(
        element: ElementType,
        length: u64,
        span: Span,
    ) -> Result<u64, Error> {
        length
            .checked_mul(element.size() as u64)
            .filter(|bytes| *bytes <= MAX_BYTE_LENGTH)
            .ok_or_else(|| {
                // CreateByteDataBlock (6.2.9.1) limits bytes, independently of
                // ToIndex on the number of elements.
                Self::exception(
                    ExceptionKind::RangeError,
                    span,
                    "TypedArray byte length exceeds the maximum safe integer",
                )
            })
    }

    fn allocate_typed_array(
        &mut self,
        element: ElementType,
        prototype: &ObjectHandle,
        length: u64,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let byte_length = Self::typed_array_byte_length(element, length, span)?;
        let buffer_prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized realm")
            .array_buffer
            .prototype
            .clone();
        let buffer = self.typed_array_backing_work(span, |objects, budget| {
            objects.create_array_buffer(&buffer_prototype, byte_length, None, budget)
        })?;
        // The fresh object is unobservable until returned. Install only complete
        // final slots instead of exposing an uninitialized buffer state.
        self.object_work(span, |objects, _| {
            objects.create_typed_array(prototype, &buffer, element, 0, Some(length))
        })
    }

    fn typed_array_from_buffer(
        &mut self,
        element: ElementType,
        prototype: &ObjectHandle,
        buffer: &ObjectHandle,
        offset: Value,
        length: Value,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        // InitializeTypedArrayFromArrayBuffer (23.2.5.1.3) checks alignment
        // before converting length or checking detachment.
        let offset = self.typed_array_to_index(offset, span)?;
        let size = element.size() as u64;
        if offset % size != 0 {
            return Err(Self::typed_array_buffer_range_error(span));
        }
        let resizable = self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(buffer)?
                .array_buffer_data()
                .expect("ArrayBuffer source")
                .is_resizable())
        })?;
        let length = if matches!(length, Value::Undefined) {
            None
        } else {
            Some(self.typed_array_to_index(length, span)?)
        };
        let (detached, buffer_length) = self.object_work(span, |objects, _| {
            let data = objects
                .inspect(buffer)?
                .array_buffer_data()
                .expect("ArrayBuffer source");
            Ok((data.is_detached(), data.byte_length()))
        })?;
        if detached {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray buffer is detached",
            ));
        }
        let length = if let Some(length) = length {
            let bytes = Self::typed_array_byte_length(element, length, span)?;
            if offset
                .checked_add(bytes)
                .is_none_or(|end| end > buffer_length)
            {
                return Err(Self::typed_array_buffer_range_error(span));
            }
            Some(length)
        } else if resizable {
            if offset > buffer_length {
                return Err(Self::typed_array_buffer_range_error(span));
            }
            None
        } else {
            if buffer_length % size != 0 || offset > buffer_length {
                return Err(Self::typed_array_buffer_range_error(span));
            }
            Some((buffer_length - offset) / size)
        };
        self.object_work(span, |objects, _| {
            objects.create_typed_array(prototype, buffer, element, offset, length)
        })
    }

    fn typed_array_buffer_range_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::RangeError,
            span,
            "TypedArray view exceeds its buffer or is misaligned",
        )
    }

    fn typed_array_from_typed_array(
        &mut self,
        element: ElementType,
        prototype: &ObjectHandle,
        source: &ObjectHandle,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let (source_element, source_buffer, source_offset, length) =
            self.object_work(span, |objects, _| {
                let source = objects
                    .inspect(source)?
                    .typed_array_data()
                    .expect("TypedArray source");
                let buffer = objects
                    .inspect(source.buffer())?
                    .array_buffer_data()
                    .expect("TypedArray buffer");
                Ok((
                    source.element_type(),
                    source.buffer().clone(),
                    source.byte_offset(),
                    source.in_bounds_array_length(buffer),
                ))
            })?;
        let length = length.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray source is out of bounds or detached",
            )
        })?;
        if element == source_element {
            // CloneArrayBuffer is a raw-byte operation: decoding and encoding
            // would lose the specified NaN payload and sign preservation.
            let byte_length = Self::typed_array_byte_length(element, length, span)?;
            let buffer_prototype = self
                .intrinsics
                .as_ref()
                .expect("initialized realm")
                .array_buffer
                .prototype
                .clone();
            let buffer = self.typed_array_backing_work(span, |objects, budget| {
                objects.clone_array_buffer(
                    &buffer_prototype,
                    &source_buffer,
                    source_offset,
                    byte_length,
                    budget,
                )
            })?;
            return self.object_work(span, |objects, _| {
                objects.create_typed_array(prototype, &buffer, element, 0, Some(length))
            });
        }
        // The destination allocation precedes the content-type mismatch check,
        // even for an empty source (23.2.5.1.2).
        let result = self.allocate_typed_array(element, prototype, length, span)?;
        if element.is_bigint() != source_element.is_bigint() {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray source and destination have different content types",
            ));
        }
        for index in 0..length {
            self.tick(span)?;
            self.object_work(span, |objects, budget| {
                let value = objects.typed_array_get_element(source, index as f64, budget)?;
                objects.typed_array_set_element(&result, index as f64, value, budget)
            })?;
        }
        Ok(result)
    }

    pub(super) fn typed_array_backing_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Objects, &mut Budget) -> Result<T, ArrayBufferError>,
    ) -> Result<T, Error> {
        let result = self.object_work(span, |objects, budget| match work(objects, budget) {
            Err(ArrayBufferError::Storage(error)) => Err(error),
            result => Ok(result),
        })?;
        result.map_err(|error| {
            let kind = match error {
                ArrayBufferError::Capacity | ArrayBufferError::OutOfBounds => {
                    ExceptionKind::RangeError
                }
                ArrayBufferError::Detached
                | ArrayBufferError::NotResizable
                | ArrayBufferError::SameBuffer => ExceptionKind::TypeError,
                ArrayBufferError::Storage(_) => unreachable!("host errors returned above"),
            };
            Self::exception(kind, span, error.to_string())
        })
    }

    fn typed_array_iterator_list(
        &mut self,
        source: Value,
        method: Value,
        span: Span,
    ) -> Result<Vec<Value>, Error> {
        let mut iterator = self.get_iterator_from_method(source, method, span)?;
        let mut values = Vec::new();
        loop {
            self.tick(span)?;
            // IteratorToList (7.4.18) exhausts the iterator before conversion or
            // construction. Neither step failures nor later conversion errors
            // trigger IteratorClose here.
            let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                return Ok(values);
            };
            let movement = if values.len() == values.capacity() {
                values.len()
            } else {
                0
            };
            self.object_work(span, |_, budget| {
                budget.charge(movement)?;
                budget.charge(1)
            })?;
            // This is a container's scratch List, not a call argument List;
            // the optional max_call_arguments limit does not apply.
            values.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "TypedArray input list exceeds platform capacity".into(),
            })?;
            values.push(value);
        }
    }

    fn typed_array_list_length(length: usize, span: Span) -> Result<u64, Error> {
        u64::try_from(length).map_err(|_| Error::Limit {
            span,
            message: "TypedArray input list exceeds platform capacity".into(),
        })
    }

    pub(super) fn typed_array_from(
        &mut self,
        receiver: Value,
        source: Value,
        mapper: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.require_typed_array_factory_constructor(&receiver, span)?;
        let mapping = !matches!(mapper, Value::Undefined);
        if mapping && !self.is_callable(&mapper, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray mapper is not callable",
            ));
        }
        let method = self.get_method(&source, &WellKnownSymbol::Iterator.symbol(), span)?;
        let result = if let Some(method) = method {
            let values = self.typed_array_iterator_list(source, method, span)?;
            let length = Self::typed_array_list_length(values.len(), span)?;
            let result = self.typed_array_factory_result(receiver, length, span)?;
            for (index, value) in values.into_iter().enumerate() {
                self.tick(span)?;
                let value = self.typed_array_map_value(
                    value,
                    index as u64,
                    mapping.then_some(&mapper),
                    &this_arg,
                    span,
                )?;
                self.set_property_or_throw(
                    &result,
                    JsString::from(index.to_string().as_str()),
                    value,
                    span,
                )?;
            }
            result
        } else {
            let Value::Object(source) = self.box_primitive(source, span)? else {
                unreachable!("ToObject");
            };
            let length = self.length_of_array_like(&source, span)?;
            let result = self.typed_array_factory_result(receiver, length, span)?;
            for index in 0..length {
                self.tick(span)?;
                let key = JsString::from(index.to_string().as_str());
                let value = self.get_property(&source, &key, span)?;
                let value = self.typed_array_map_value(
                    value,
                    index,
                    mapping.then_some(&mapper),
                    &this_arg,
                    span,
                )?;
                self.set_property_or_throw(&result, key, value, span)?;
            }
            result
        };
        Ok(Value::Object(result))
    }

    pub(super) fn typed_array_of(
        &mut self,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let length = Self::typed_array_list_length(arguments.len(), span)?;
        self.require_typed_array_factory_constructor(&receiver, span)?;
        let result = self.typed_array_factory_result(receiver, length, span)?;
        for (index, value) in arguments.enumerate() {
            self.tick(span)?;
            self.set_property_or_throw(
                &result,
                JsString::from(index.to_string().as_str()),
                value,
                span,
            )?;
        }
        Ok(Value::Object(result))
    }

    fn require_typed_array_factory_constructor(
        &mut self,
        receiver: &Value,
        span: Span,
    ) -> Result<(), Error> {
        if self.is_constructor(receiver, span)? {
            Ok(())
        } else {
            Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray factory receiver is not a constructor",
            ))
        }
    }

    pub(super) fn typed_array_factory_result(
        &mut self,
        constructor: Value,
        length: u64,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        self.typed_array_factory_result_arguments(
            constructor,
            vec![Value::Number(length as f64)],
            span,
        )
    }

    /// TypedArrayCreateFromConstructor (23.2.4.1) validates the constructed
    /// result for every argument list. Only one Number argument imposes a
    /// minimum length; subarray's buffer/offset[/length] forms do not.
    pub(super) fn typed_array_factory_result_arguments(
        &mut self,
        constructor: Value,
        arguments: Vec<Value>,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let minimum_length = match arguments.as_slice() {
            [Value::Number(length)] => Some(*length),
            _ => None,
        };
        let Value::Object(result) = self.construct(constructor, arguments, span)? else {
            unreachable!("Construct returns an object");
        };
        // TypedArrayCreateFromConstructor (23.2.4.1) validates only here.
        // Later getter/mapper effects use ordinary Set, including its live
        // coercion-before-bounds behavior after detachment or resizing.
        let valid_length = self.object_work(span, |objects, _| {
            let record = objects.inspect(&result)?;
            let Some(data) = record.typed_array_data() else {
                return Ok(None);
            };
            let buffer = objects
                .inspect(data.buffer())?
                .array_buffer_data()
                .expect("TypedArray buffer");
            Ok(data.in_bounds_array_length(buffer))
        })?;
        if valid_length
            .is_none_or(|actual| minimum_length.is_some_and(|minimum| (actual as f64) < minimum))
        {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray factory constructor returned an invalid or short result",
            ));
        }
        Ok(result)
    }

    fn typed_array_map_value(
        &mut self,
        value: Value,
        index: u64,
        mapper: Option<&Value>,
        this_arg: &Value,
        span: Span,
    ) -> Result<Value, Error> {
        if let Some(mapper) = mapper {
            self.object_work(span, |_, budget| {
                budget.value(mapper)?;
                budget.value(this_arg)
            })?;
            self.call(
                mapper.clone(),
                this_arg.clone(),
                vec![value, Value::Number(index as f64)],
                span,
            )
        } else {
            Ok(value)
        }
    }
}
