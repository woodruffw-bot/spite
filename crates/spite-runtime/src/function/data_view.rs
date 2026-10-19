//! Ordinary DataView construction and live buffer witnesses (25.3).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    binary_data::ElementType,
    object::{
        ArrayBufferError, Budget, DataDescriptor, DescriptorKind, Objects, PropertyDescriptor,
    },
};
use spite_core::{JsString, Span, WellKnownSymbol};

const ELEMENT_TYPES: [ElementType; 11] = [
    ElementType::BigInt64,
    ElementType::BigUint64,
    ElementType::Float16,
    ElementType::Float32,
    ElementType::Float64,
    ElementType::Int8,
    ElementType::Int16,
    ElementType::Int32,
    ElementType::Uint8,
    ElementType::Uint16,
    ElementType::Uint32,
];

#[derive(Clone, Copy, Debug)]
pub(crate) enum Member {
    Buffer,
    ByteLength,
    ByteOffset,
    Get(ElementType),
    Set(ElementType),
}

impl Member {
    pub(super) fn initial_name(self) -> &'static str {
        match self {
            Self::Buffer => "get buffer",
            Self::ByteLength => "get byteLength",
            Self::ByteOffset => "get byteOffset",
            Self::Get(ElementType::BigInt64) => "getBigInt64",
            Self::Get(ElementType::BigUint64) => "getBigUint64",
            Self::Get(ElementType::Float16) => "getFloat16",
            Self::Get(ElementType::Float32) => "getFloat32",
            Self::Get(ElementType::Float64) => "getFloat64",
            Self::Get(ElementType::Int8) => "getInt8",
            Self::Get(ElementType::Int16) => "getInt16",
            Self::Get(ElementType::Int32) => "getInt32",
            Self::Get(ElementType::Uint8) => "getUint8",
            Self::Get(ElementType::Uint16) => "getUint16",
            Self::Get(ElementType::Uint32) => "getUint32",
            Self::Set(ElementType::BigInt64) => "setBigInt64",
            Self::Set(ElementType::BigUint64) => "setBigUint64",
            Self::Set(ElementType::Float16) => "setFloat16",
            Self::Set(ElementType::Float32) => "setFloat32",
            Self::Set(ElementType::Float64) => "setFloat64",
            Self::Set(ElementType::Int8) => "setInt8",
            Self::Set(ElementType::Int16) => "setInt16",
            Self::Set(ElementType::Int32) => "setInt32",
            Self::Set(ElementType::Uint8) => "setUint8",
            Self::Set(ElementType::Uint16) => "setUint16",
            Self::Set(ElementType::Uint32) => "setUint32",
            Self::Get(ElementType::Uint8Clamped) | Self::Set(ElementType::Uint8Clamped) => {
                unreachable!("DataView has no clamped byte methods")
            }
        }
    }

    pub(super) fn length(self) -> f64 {
        match self {
            Self::Get(_) => 1.0,
            Self::Set(_) => 2.0,
            _ => 0.0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DataViewIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl DataViewIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

struct ViewWitness {
    buffer: ObjectHandle,
    byte_offset: u64,
    byte_length: u64,
}

#[derive(Clone, Copy)]
struct BufferState {
    byte_length: u64,
    resizable: bool,
    detached: bool,
}

impl Realm {
    pub(super) fn data_view_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<DataViewIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::DataView, span)?;
        // %DataView.prototype% is ordinary and does not have view slots.
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.define_property_or_throw(
            &constructor,
            JsString::from("prototype"),
            DataDescriptor {
                value: Some(Value::Object(prototype.clone())),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )?;
        self.define_builtin_property(
            &prototype,
            "constructor",
            Value::Object(constructor.clone()),
            true,
            span,
        )?;
        let mut methods = Vec::new();
        for element in ELEMENT_TYPES {
            for member in [Member::Get(element), Member::Set(element)] {
                let method =
                    self.new_builtin(function_prototype, Builtin::DataViewMember(member), span)?;
                self.define_builtin_property(
                    &prototype,
                    member.initial_name(),
                    Value::Object(method.clone()),
                    true,
                    span,
                )?;
                methods.push(method);
            }
        }
        for (name, member) in [
            ("buffer", Member::Buffer),
            ("byteLength", Member::ByteLength),
            ("byteOffset", Member::ByteOffset),
        ] {
            let getter =
                self.new_builtin(function_prototype, Builtin::DataViewMember(member), span)?;
            self.define_property_or_throw(
                &prototype,
                JsString::from(name),
                PropertyDescriptor {
                    enumerable: Some(false),
                    configurable: Some(true),
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(getter.clone())),
                        set: Some(None),
                    },
                },
                span,
            )?;
            methods.push(getter);
        }
        self.define_property_or_throw(
            &prototype,
            WellKnownSymbol::ToStringTag.symbol(),
            DataDescriptor {
                value: Some(Value::String(JsString::from("DataView"))),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        Ok(DataViewIntrinsics {
            constructor,
            prototype,
            methods,
        })
    }

    fn data_view_index(&mut self, value: Value, span: Span) -> Result<u64, Error> {
        // ToIndex (7.1.22) truncates before checking the safe-index range.
        let number = self.number(value, span)?;
        let integer = if number.is_nan() { 0.0 } else { number.trunc() };
        if !(0.0..=9_007_199_254_740_991.0).contains(&integer) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "DataView index is outside the safe index range",
            ));
        }
        Ok(integer as u64)
    }

    fn data_view_buffer_state(
        &mut self,
        buffer: &ObjectHandle,
        span: Span,
    ) -> Result<Option<BufferState>, Error> {
        self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(buffer)?
                .array_buffer_data()
                .map(|data| BufferState {
                    byte_length: data.byte_length(),
                    resizable: data.is_resizable(),
                    detached: data.is_detached(),
                }))
        })
    }

    fn data_view_attached(state: BufferState, span: Span) -> Result<(), Error> {
        if state.detached {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "DataView buffer is detached",
            ));
        }
        Ok(())
    }

    fn data_view_range(span: Span) -> Error {
        Self::exception(
            ExceptionKind::RangeError,
            span,
            "DataView access exceeds its byte range",
        )
    }

    pub(super) fn data_view_constructor(
        &mut self,
        new_target: ObjectHandle,
        buffer: Value,
        byte_offset: Value,
        byte_length: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let buffer = if let Value::Object(buffer) = buffer {
            if self.data_view_buffer_state(&buffer, span)?.is_some() {
                buffer
            } else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "DataView requires an ArrayBuffer",
                ));
            }
        } else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "DataView requires an ArrayBuffer",
            ));
        };
        // 25.3.2.1: offset conversion precedes detachment and buffer length.
        let offset = self.data_view_index(byte_offset, span)?;
        let original = self
            .data_view_buffer_state(&buffer, span)?
            .expect("buffer slot");
        Self::data_view_attached(original, span)?;
        if offset > original.byte_length {
            return Err(Self::data_view_range(span));
        }
        let explicit_length = !matches!(byte_length, Value::Undefined);
        let length = if explicit_length {
            let length = self.data_view_index(byte_length, span)?;
            // This check uses the pre-conversion snapshot even if conversion
            // resized or detached the buffer. The prototype lookup comes later.
            if length > original.byte_length - offset {
                return Err(Self::data_view_range(span));
            }
            Some(length)
        } else if original.resizable {
            None
        } else {
            Some(original.byte_length - offset)
        };
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .data_view
                .prototype
                .clone()
        };
        // Both explicit length conversion and newTarget.prototype can run code.
        // Recheck the current buffer after that observable constructor step.
        let current = self
            .data_view_buffer_state(&buffer, span)?
            .expect("buffer slot");
        Self::data_view_attached(current, span)?;
        if offset > current.byte_length
            || explicit_length && length.is_some_and(|length| length > current.byte_length - offset)
        {
            return Err(Self::data_view_range(span));
        }
        self.object_work(span, |objects, _| {
            objects.create_data_view(&prototype, &buffer, offset, length)
        })
        .map(Value::Object)
    }

    fn data_view_receiver(&mut self, receiver: Value, span: Span) -> Result<ObjectHandle, Error> {
        if let Value::Object(view) = receiver {
            if self.object_work(span, |objects, _| {
                Ok(objects.inspect(&view)?.data_view_data().is_some())
            })? {
                return Ok(view);
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no DataView internal slot",
        ))
    }

    fn data_view_witness(&mut self, view: &ObjectHandle, span: Span) -> Result<ViewWitness, Error> {
        // 25.3.1.2–4: snapshot the buffer length after all conversions, then
        // derive fixed or auto length without another observable operation.
        self.object_work(span, |objects, _| {
            let view = objects.inspect(view)?.data_view_data().expect("view slot");
            let buffer = objects
                .inspect(view.buffer())?
                .array_buffer_data()
                .expect("viewed buffer slot");
            Ok(view
                .in_bounds_byte_length(buffer)
                .map(|length| ViewWitness {
                    buffer: view.buffer().clone(),
                    byte_offset: view.byte_offset(),
                    byte_length: length,
                }))
        })?
        .ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "DataView is out of bounds or its buffer is detached",
            )
        })
    }

    fn data_view_buffer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Objects, &mut Budget) -> Result<T, ArrayBufferError>,
    ) -> Result<T, Error> {
        let result = self.object_work(span, |objects, budget| match work(objects, budget) {
            Err(ArrayBufferError::Storage(error)) => Err(error),
            // GetValueFromBuffer/SetValueInBuffer do not allocate a new Data
            // Block in the specification. A lazy backing-page allocation
            // failure is a host failure, not a JavaScript RangeError.
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

    pub(super) fn array_buffer_is_view(
        &mut self,
        argument: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 25.1.5.1 tests the slot without observing any public properties or
        // rejecting a view whose buffer has since been detached.
        let result = if let Value::Object(object) = argument {
            self.object_work(span, |objects, _| {
                let record = objects.inspect(&object)?;
                Ok(record.data_view_data().is_some() || record.typed_array_data().is_some())
            })?
        } else {
            false
        };
        Ok(Value::Boolean(result))
    }

    pub(super) fn data_view_member(
        &mut self,
        member: Member,
        receiver: Value,
        byte_offset: Value,
        value_or_endian: Value,
        little_endian: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let view = self.data_view_receiver(receiver, span)?;
        if matches!(member, Member::Buffer) {
            return self
                .object_work(span, |objects, _| {
                    Ok(objects
                        .inspect(&view)?
                        .data_view_data()
                        .expect("view slot")
                        .buffer()
                        .clone())
                })
                .map(Value::Object);
        }
        let element = match member {
            Member::ByteLength | Member::ByteOffset => {
                let witness = self.data_view_witness(&view, span)?;
                return Ok(Value::Number(if matches!(member, Member::ByteLength) {
                    witness.byte_length as f64
                } else {
                    witness.byte_offset as f64
                }));
            }
            Member::Get(element) | Member::Set(element) => element,
            Member::Buffer => unreachable!("buffer getter returned above"),
        };
        // GetViewValue and SetViewValue (25.3.1.5–6) validate the receiver,
        // convert the offset, then convert setter values before live bounds.
        let index = self.data_view_index(byte_offset, span)?;
        let (value, little_endian) = match member {
            Member::Get(_) => (Value::Undefined, value_or_endian.to_boolean()),
            Member::Set(_) => {
                let value = if element.is_bigint() {
                    Value::BigInt(self.bigint(value_or_endian, span)?)
                } else {
                    Value::Number(self.number(value_or_endian, span)?)
                };
                (value, little_endian.to_boolean())
            }
            _ => unreachable!("getters returned above"),
        };
        let witness = self.data_view_witness(&view, span)?;
        let size = element.size();
        if index
            .checked_add(size as u64)
            .is_none_or(|end| end > witness.byte_length)
        {
            return Err(Self::data_view_range(span));
        }
        let buffer_index = witness
            .byte_offset
            .checked_add(index)
            .ok_or_else(|| Self::data_view_range(span))?;
        if matches!(member, Member::Get(_)) {
            let bytes = self.data_view_buffer_work(span, |objects, budget| {
                objects.read_array_buffer_bytes(&witness.buffer, buffer_index, size, budget)
            })?;
            self.integer_work(span, |budget| element.decode(bytes, little_endian, budget))
        } else {
            let bytes =
                self.integer_work(span, |budget| element.encode(value, little_endian, budget))?;
            self.data_view_buffer_work(span, |objects, budget| {
                objects.write_array_buffer_bytes(
                    &witness.buffer,
                    buffer_index,
                    &bytes[..size],
                    budget,
                )
            })?;
            Ok(Value::Undefined)
        }
    }
}
