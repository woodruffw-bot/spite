//! TypedArray copies and subarray views (23.2.3.27/.30/.32/.36).

use super::typed_array_methods::{Method, TypedArrayWitness};
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    binary_data::ElementType,
    object::{ArrayBufferError, Budget, Objects},
};
use spite_core::{JsString, Span};

struct Layout {
    element: ElementType,
    buffer: ObjectHandle,
    byte_offset: u64,
    auto_length: bool,
}

impl Realm {
    pub(super) fn typed_array_copy_method(
        &mut self,
        method: Method,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if matches!(method, Method::Subarray) {
            // subarray requires the brand, but accepts an initially detached
            // or out-of-bounds source. Its source length is zero in that case.
            return self.typed_array_subarray(
                receiver,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            );
        }
        let witness = self.typed_array_validate(receiver, span)?;
        let result = match method {
            Method::Slice => self.typed_array_slice(
                &witness,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            )?,
            Method::ToReversed => self.typed_array_to_reversed(&witness, span)?,
            Method::With => self.typed_array_with(
                &witness,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            )?,
            _ => unreachable!("other TypedArray methods have separate dispatch"),
        };
        Ok(Value::Object(result))
    }

    fn typed_array_copy_integer(&mut self, value: Value, span: Span) -> Result<f64, Error> {
        let number = self.number(value, span)?;
        let integer = number.trunc();
        // ToIntegerOrInfinity returns mathematical +0 for NaN and either zero,
        // including negative fractions that truncate to negative-zero bits.
        Ok(if number.is_nan() || integer == 0.0 {
            0.0
        } else {
            integer
        })
    }

    fn typed_array_copy_relative(
        &mut self,
        value: Value,
        length: u64,
        span: Span,
    ) -> Result<u64, Error> {
        let integer = self.typed_array_copy_integer(value, span)?;
        // Clamp infinities before narrowing to an exact safe-integer index.
        Ok(if integer < 0.0 {
            (length as f64 + integer).max(0.0) as u64
        } else {
            integer.min(length as f64) as u64
        })
    }

    fn typed_array_copy_layout(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Layout, Error> {
        self.object_work(span, |objects, _| {
            let data = objects
                .inspect(object)?
                .typed_array_data()
                .expect("validated TypedArray");
            Ok(Layout {
                element: data.element_type(),
                buffer: data.buffer().clone(),
                byte_offset: data.byte_offset(),
                auto_length: data.array_length().is_none(),
            })
        })
    }

    fn typed_array_copy_byte_index(layout: &Layout, index: u64, span: Span) -> Result<u64, Error> {
        index
            .checked_mul(layout.element.size() as u64)
            .and_then(|bytes| layout.byte_offset.checked_add(bytes))
            .ok_or_else(|| Self::typed_array_copy_range_error(span))
    }

    fn typed_array_copy_range_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::RangeError,
            span,
            "TypedArray byte range is too large",
        )
    }

    fn typed_array_slice(
        &mut self,
        witness: &TypedArrayWitness,
        start: Value,
        end: Value,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let start = self.typed_array_copy_relative(start, witness.length, span)?;
        let end = if matches!(end, Value::Undefined) {
            witness.length
        } else {
            self.typed_array_copy_relative(end, witness.length, span)?
        };
        let requested = end.saturating_sub(start);
        let result =
            self.typed_array_species_create(&witness.object, witness.element, requested, span)?;
        if requested == 0 {
            // Species may detach the source. An originally empty slice skips
            // the second witness, but still validates the species result.
            return Ok(result);
        }
        let current = self.typed_array_validate(Value::Object(witness.object.clone()), span)?;
        let count = end.min(current.length).saturating_sub(start);
        if count == 0 {
            // A valid auto view can shrink past start. Preserve the result's
            // existing tail and do not form an obsolete source byte range.
            return Ok(result);
        }
        let source = self.typed_array_copy_layout(&witness.object, span)?;
        let target = self.typed_array_copy_layout(&result, span)?;
        if source.element == target.element {
            let source_start = Self::typed_array_copy_byte_index(&source, start, span)?;
            let byte_count = count
                .checked_mul(source.element.size() as u64)
                .ok_or_else(|| Self::typed_array_copy_range_error(span))?;
            if source.buffer != target.buffer {
                self.typed_array_copy_buffer_work(span, |objects, budget| {
                    objects.copy_array_buffer_bytes(
                        &source.buffer,
                        source_start,
                        &target.buffer,
                        target.byte_offset,
                        byte_count,
                        budget,
                    )
                })?;
            } else {
                // 23.2.3.27 requires LIVE FORWARD byte copying, even when the
                // species result overlaps source. Neither memmove nor a source
                // snapshot preserves that aliasing behavior. Each read sees
                // earlier writes; raw bytes retain NaN payload/sign encodings.
                for offset in 0..byte_count {
                    self.tick(span)?;
                    let source_index = source_start
                        .checked_add(offset)
                        .ok_or_else(|| Self::typed_array_copy_range_error(span))?;
                    let target_index = target
                        .byte_offset
                        .checked_add(offset)
                        .ok_or_else(|| Self::typed_array_copy_range_error(span))?;
                    self.typed_array_copy_buffer_work(span, |objects, budget| {
                        let byte = objects.read_array_buffer_bytes(
                            &source.buffer,
                            source_index,
                            1,
                            budget,
                        )?;
                        objects.write_array_buffer_bytes(
                            &target.buffer,
                            target_index,
                            &byte[..1],
                            budget,
                        )
                    })?;
                }
            }
        } else {
            // Different kinds copy numeric values, including live same-buffer
            // alias effects. The species layer already checked content types.
            for index in 0..count {
                self.tick(span)?;
                let value = self.get_property(
                    &witness.object,
                    &JsString::from((start + index).to_string().as_str()),
                    span,
                )?;
                self.set_property_or_throw(
                    &result,
                    JsString::from(index.to_string().as_str()),
                    value,
                    span,
                )?;
            }
        }
        Ok(result)
    }

    fn typed_array_subarray(
        &mut self,
        receiver: Value,
        start: Value,
        end: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let invalid = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver has no TypedArray internal slots",
            )
        };
        let Value::Object(object) = receiver else {
            return Err(invalid());
        };
        let initial = self.object_work(span, |objects, _| {
            let Some(data) = objects.inspect(&object)?.typed_array_data() else {
                return Ok(None);
            };
            let buffer = objects
                .inspect(data.buffer())?
                .buffer_info()?
                .expect("TypedArray buffer");
            Ok(Some((
                data.element_type(),
                data.in_bounds_array_length(&buffer).unwrap_or(0),
            )))
        })?;
        let (element, length) = initial.ok_or_else(invalid)?;
        let start = self.typed_array_copy_relative(start, length, span)?;
        let layout = self.typed_array_copy_layout(&object, span)?;
        let offset = Self::typed_array_copy_byte_index(&layout, start, span)?;
        let arguments = if layout.auto_length && matches!(end, Value::Undefined) {
            vec![Value::Object(layout.buffer), Value::Number(offset as f64)]
        } else {
            let end = if matches!(end, Value::Undefined) {
                length
            } else {
                self.typed_array_copy_relative(end, length, span)?
            };
            vec![
                Value::Object(layout.buffer),
                Value::Number(offset as f64),
                Value::Number(end.saturating_sub(start) as f64),
            ]
        };
        // No second source validation. A custom species can return an unrelated
        // live view, even for a detached/OOB source. Two/three arguments impose
        // no one-Number minimum-length requirement on that result (23.2.4.1).
        self.typed_array_species_create_arguments(&object, element, arguments, span)
            .map(Value::Object)
    }

    fn typed_array_to_reversed(
        &mut self,
        witness: &TypedArrayWitness,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let result = self.typed_array_create_same_type(witness.element, witness.length, span)?;
        for index in 0..witness.length {
            self.tick(span)?;
            let value = self.get_property(
                &witness.object,
                &JsString::from((witness.length - index - 1).to_string().as_str()),
                span,
            )?;
            self.set_property_or_throw(
                &result,
                JsString::from(index.to_string().as_str()),
                value,
                span,
            )?;
        }
        Ok(result)
    }

    fn typed_array_with(
        &mut self,
        witness: &TypedArrayWitness,
        index: Value,
        value: Value,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let relative = self.typed_array_copy_integer(index, span)?;
        let actual = if relative >= 0.0 {
            relative
        } else {
            witness.length as f64 + relative
        };
        // Conversion precedes IsValidIntegerIndex even for infinite indices,
        // empty views, and values already outside the captured length. It may
        // resize/detach the source or make a formerly invalid index valid.
        let numeric = if witness.element.is_bigint() {
            Value::BigInt(self.bigint(value, span)?)
        } else {
            Value::Number(self.number(value, span)?)
        };
        if !self.object_work(span, |objects, budget| {
            objects.typed_array_has_index(&witness.object, actual, budget)
        })? {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "TypedArray replacement index is invalid",
            ));
        }
        let result = self.typed_array_create_same_type(witness.element, witness.length, span)?;
        let mut numeric = Some(numeric);
        for index in 0..witness.length {
            self.tick(span)?;
            let key = JsString::from(index.to_string().as_str());
            let value = if index as f64 == actual {
                // Move the already converted replacement exactly once. Skip
                // source Get at that index and do not clone a large BigInt.
                numeric.take().expect("one replacement index")
            } else {
                self.get_property(&witness.object, &key, span)?
            };
            self.set_property_or_throw(&result, key, value, span)?;
        }
        // Auto-growth may validate actual >= captured length: in that case the
        // result still has the captured length and no replacement is copied.
        Ok(result)
    }

    fn typed_array_copy_buffer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Objects, &mut Budget) -> Result<T, ArrayBufferError>,
    ) -> Result<T, Error> {
        let result = self.object_work(span, |objects, budget| match work(objects, budget) {
            Err(ArrayBufferError::Storage(error)) => Err(error),
            // Raw-copy staging and bounded byte I/O do not normatively allocate
            // a new Data Block. Native allocation failure stays a host failure.
            Err(ArrayBufferError::Capacity) => {
                Err(crate::object::Error::Heap(spite_heap::Error::Capacity))
            }
            result => Ok(result),
        })?;
        result.map_err(|error| {
            let kind = match error {
                ArrayBufferError::OutOfBounds => ExceptionKind::RangeError,
                ArrayBufferError::Detached
                | ArrayBufferError::NotResizable
                | ArrayBufferError::SameBuffer => ExceptionKind::TypeError,
                ArrayBufferError::Storage(_) | ArrayBufferError::Capacity => {
                    unreachable!("host errors returned above")
                }
            };
            Self::exception(kind, span, error.to_string())
        })
    }
}
