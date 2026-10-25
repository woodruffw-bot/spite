//! TypedArray copyWithin/fill/reverse/set with explicit live witnesses (23.2.3).

use super::typed_array_methods::Method;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    binary_data::ElementType,
    object::{ArrayBufferError, Budget, Objects},
};
use spite_core::{JsString, Span};

struct View {
    object: ObjectHandle,
    element: ElementType,
    buffer: ObjectHandle,
    byte_offset: u64,
}

impl Realm {
    pub(super) fn typed_array_mutation_method(
        &mut self,
        method: Method,
        receiver: Value,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = receiver else {
            return Err(Self::typed_array_mutation_brand_error(span));
        };
        let view = self
            .typed_array_mutation_view(&object, span)?
            .ok_or_else(|| Self::typed_array_mutation_brand_error(span))?;
        // set requires only the brand before offset conversion. The other
        // three methods validate the live view before any argument conversion.
        if matches!(method, Method::Set) {
            return self.typed_array_set_values(
                &view,
                arguments.next().unwrap_or(Value::Undefined),
                arguments.next().unwrap_or(Value::Undefined),
                span,
            );
        }
        let length = self.typed_array_mutation_length(&view, span)?;
        match method {
            Method::CopyWithin => self.typed_array_copy_within(
                &view,
                length,
                [
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next().unwrap_or(Value::Undefined),
                ],
                span,
            )?,
            Method::Fill => self.typed_array_fill_values(
                &view,
                length,
                [
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next().unwrap_or(Value::Undefined),
                    arguments.next().unwrap_or(Value::Undefined),
                ],
                span,
            )?,
            Method::Reverse => self.typed_array_reverse_values(&view, length, span)?,
            _ => unreachable!("other TypedArray methods have separate dispatch"),
        }
        Ok(Value::Object(view.object))
    }

    fn typed_array_mutation_brand_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no TypedArray internal slots",
        )
    }

    // This reads immutable layout slots, without a buffer witness. In
    // particular it must not reject a detached set receiver before offset.
    fn typed_array_mutation_view(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Option<View>, Error> {
        self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(object)?
                .typed_array_data()
                .map(|data| View {
                    object: object.clone(),
                    element: data.element_type(),
                    buffer: data.buffer().clone(),
                    byte_offset: data.byte_offset(),
                }))
        })
    }

    fn typed_array_mutation_length(&mut self, view: &View, span: Span) -> Result<u64, Error> {
        let length = self.object_work(span, |objects, _| {
            let data = objects
                .inspect(&view.object)?
                .typed_array_data()
                .expect("branded TypedArray");
            let buffer = objects
                .inspect(&view.buffer)?
                .array_buffer_data()
                .expect("TypedArray buffer");
            Ok(data.in_bounds_array_length(buffer))
        })?;
        length.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray is out of bounds or detached",
            )
        })
    }

    fn typed_array_mutation_integer(&mut self, value: Value, span: Span) -> Result<f64, Error> {
        let number = self.number(value, span)?;
        let integer = number.trunc();
        // ToIntegerOrInfinity returns mathematical zero, including for a
        // negative fraction whose truncation has Rust's negative-zero bits.
        Ok(if number.is_nan() || integer == 0.0 {
            0.0
        } else {
            integer
        })
    }

    fn typed_array_mutation_relative(
        &mut self,
        value: Value,
        length: u64,
        span: Span,
    ) -> Result<u64, Error> {
        let integer = self.typed_array_mutation_integer(value, span)?;
        // Clamp infinities and large finite values before narrowing. The
        // result is an exact integral index within the safe captured length.
        Ok(if integer < 0.0 {
            (length as f64 + integer).max(0.0) as u64
        } else {
            integer.min(length as f64) as u64
        })
    }

    fn typed_array_mutation_bytes(
        element: ElementType,
        count: u64,
        span: Span,
    ) -> Result<u64, Error> {
        count.checked_mul(element.size() as u64).ok_or_else(|| {
            Self::exception(
                ExceptionKind::RangeError,
                span,
                "TypedArray byte range is too large",
            )
        })
    }

    fn typed_array_mutation_byte_index(view: &View, index: u64, span: Span) -> Result<u64, Error> {
        Self::typed_array_mutation_bytes(view.element, index, span)?
            .checked_add(view.byte_offset)
            .ok_or_else(|| {
                Self::exception(
                    ExceptionKind::RangeError,
                    span,
                    "TypedArray byte range is too large",
                )
            })
    }

    fn typed_array_copy_within(
        &mut self,
        view: &View,
        length: u64,
        [target, start, end]: [Value; 3],
        span: Span,
    ) -> Result<(), Error> {
        let target = self.typed_array_mutation_relative(target, length, span)?;
        let start = self.typed_array_mutation_relative(start, length, span)?;
        let end = if matches!(end, Value::Undefined) {
            length
        } else {
            self.typed_array_mutation_relative(end, length, span)?
        };
        let count = end.saturating_sub(start).min(length - target);
        if count == 0 {
            // 23.2.3.6 skips the second witness for an originally empty copy,
            // even if conversion has detached or invalidated the receiver.
            return Ok(());
        }
        let current_length = self.typed_array_mutation_length(view, span)?;
        let count = count
            .min(current_length.saturating_sub(start))
            .min(current_length.saturating_sub(target));
        if count == 0 {
            // A valid auto view may now have no applicable prefix. Do not
            // validate or calculate byte ranges for its obsolete positions.
            return Ok(());
        }
        let source_start = Self::typed_array_mutation_byte_index(view, start, span)?;
        let target_start = Self::typed_array_mutation_byte_index(view, target, span)?;
        let bytes = Self::typed_array_mutation_bytes(view.element, count, span)?;
        self.typed_array_mutation_buffer_work(span, |objects, budget| {
            objects.move_array_buffer_bytes(&view.buffer, source_start, target_start, bytes, budget)
        })
    }

    fn typed_array_fill_values(
        &mut self,
        view: &View,
        length: u64,
        [value, start, end]: [Value; 3],
        span: Span,
    ) -> Result<(), Error> {
        // 23.2.3.9 converts value exactly once, before either bound and even
        // for an empty view or requested range.
        let value = if view.element.is_bigint() {
            Value::BigInt(self.bigint(value, span)?)
        } else {
            Value::Number(self.number(value, span)?)
        };
        let start = self.typed_array_mutation_relative(start, length, span)?;
        let end = if matches!(end, Value::Undefined) {
            length
        } else {
            self.typed_array_mutation_relative(end, length, span)?
        };
        // Unlike copyWithin, this witness is unconditional, including for an
        // empty old range. Growth does not extend the original end.
        let end = end.min(self.typed_array_mutation_length(view, span)?);
        for index in start..end {
            self.tick(span)?;
            self.object_work(span, |objects, budget| {
                budget.value(&value)?;
                objects.typed_array_set_element(&view.object, index as f64, value.clone(), budget)
            })?;
        }
        Ok(())
    }

    fn typed_array_reverse_values(
        &mut self,
        view: &View,
        length: u64,
        span: Span,
    ) -> Result<(), Error> {
        for lower in 0..length / 2 {
            self.tick(span)?;
            let upper = length - lower - 1;
            let lower_key = JsString::from(lower.to_string().as_str());
            let upper_key = JsString::from(upper.to_string().as_str());
            // 23.2.3.25 is numeric Get/Get/Set/Set, not raw-byte swapping.
            // Both reads precede either write, including on host failure.
            let lower_value = self.get_property(&view.object, &lower_key, span)?;
            let upper_value = self.get_property(&view.object, &upper_key, span)?;
            self.set_property_or_throw(&view.object, lower_key, upper_value, span)?;
            self.set_property_or_throw(&view.object, upper_key, lower_value, span)?;
        }
        Ok(())
    }

    fn typed_array_set_values(
        &mut self,
        target: &View,
        source: Value,
        offset: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 23.2.3.26 converts offset after the brand, but before either live
        // witness. Negative offsets reject even detached targets immediately.
        let offset = self.typed_array_mutation_integer(offset, span)?;
        if offset < 0.0 {
            return Err(Self::typed_array_set_range_error(span));
        }
        if let Value::Object(object) = &source
            && let Some(source) = self.typed_array_mutation_view(object, span)?
        {
            self.typed_array_set_from_typed_array(target, &source, offset, span)?;
            return Ok(Value::Undefined);
        }
        // Target validation precedes ToObject/LengthOfArrayLike. The length
        // getter precedes positive-infinite or excessive offset rejection.
        let target_length = self.typed_array_mutation_length(target, span)?;
        let Value::Object(source) = self.box_primitive(source, span)? else {
            unreachable!("ToObject");
        };
        let source_length = self.length_of_array_like(&source, span)?;
        let offset = Self::typed_array_set_offset(offset, source_length, target_length, span)?;
        for index in 0..source_length {
            self.tick(span)?;
            let value =
                self.get_property(&source, &JsString::from(index.to_string().as_str()), span)?;
            // Reentrant getters/conversions can invalidate target. Conversion
            // still runs before its current bounds check; old writes remain.
            self.set_typed_array_element(&target.object, (offset + index) as f64, value, span)?;
        }
        Ok(Value::Undefined)
    }

    fn typed_array_set_offset(
        offset: f64,
        source_length: u64,
        target_length: u64,
        span: Span,
    ) -> Result<u64, Error> {
        if offset == f64::INFINITY || offset > target_length as f64 {
            return Err(Self::typed_array_set_range_error(span));
        }
        let offset = offset as u64;
        if source_length > target_length - offset {
            return Err(Self::typed_array_set_range_error(span));
        }
        Ok(offset)
    }

    fn typed_array_set_range_error(span: Span) -> Error {
        Self::exception(
            ExceptionKind::RangeError,
            span,
            "TypedArray source and offset exceed the target range",
        )
    }

    fn typed_array_set_from_typed_array(
        &mut self,
        target: &View,
        source: &View,
        offset: f64,
        span: Span,
    ) -> Result<(), Error> {
        let target_length = self.typed_array_mutation_length(target, span)?;
        let source_length = self.typed_array_mutation_length(source, span)?;
        let offset = Self::typed_array_set_offset(offset, source_length, target_length, span)?;
        if source.element.is_bigint() != target.element.is_bigint() {
            // Range checks precede content-type rejection, including for an
            // empty source. Public iterator/length/constructor are ignored.
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "TypedArray source and target have different content types",
            ));
        }
        let source_bytes = Self::typed_array_mutation_bytes(source.element, source_length, span)?;
        let (source_buffer, source_start) = if source.buffer == target.buffer {
            // Snapshot the entire source view, for all same-buffer copies,
            // even nonoverlapping ranges or different element widths.
            let prototype = self
                .intrinsics
                .as_ref()
                .expect("initialized realm")
                .array_buffer
                .prototype
                .clone();
            let snapshot = self.typed_array_backing_work(span, |objects, budget| {
                objects.clone_array_buffer(
                    &prototype,
                    &source.buffer,
                    source.byte_offset,
                    source_bytes,
                    budget,
                )
            })?;
            (snapshot, 0)
        } else {
            (source.buffer.clone(), source.byte_offset)
        };
        let target_start = Self::typed_array_mutation_byte_index(target, offset, span)?;
        if source.element == target.element {
            return self.typed_array_mutation_buffer_work(span, |objects, budget| {
                objects.copy_array_buffer_bytes(
                    &source_buffer,
                    source_start,
                    &target.buffer,
                    target_start,
                    source_bytes,
                    budget,
                )
            });
        }
        let mut source_index = source_start;
        for index in 0..source_length {
            self.tick(span)?;
            let bytes = self.typed_array_mutation_buffer_work(span, |objects, budget| {
                objects.read_array_buffer_bytes(
                    &source_buffer,
                    source_index,
                    source.element.size(),
                    budget,
                )
            })?;
            // Decode the chosen original/snapshot buffer directly; no
            // synthetic TypedArray or extra heap entry is needed.
            let value = self.integer_work(span, |budget| {
                source
                    .element
                    .decode(bytes, cfg!(target_endian = "little"), budget)
            })?;
            self.object_work(span, |objects, budget| {
                objects.typed_array_set_element(
                    &target.object,
                    (offset + index) as f64,
                    value,
                    budget,
                )
            })?;
            source_index = source_index
                .checked_add(source.element.size() as u64)
                .ok_or_else(|| Self::typed_array_set_range_error(span))?;
        }
        Ok(())
    }

    fn typed_array_mutation_buffer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Objects, &mut Budget) -> Result<T, ArrayBufferError>,
    ) -> Result<T, Error> {
        let result = self.object_work(span, |objects, budget| match work(objects, budget) {
            Err(ArrayBufferError::Storage(error)) => Err(error),
            // Move/distinct-copy staging and bounded I/O do not normatively
            // allocate a new Data Block. Native allocation failure stays host.
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
