//! Ordinary ArrayBuffer construction, resizing, species copies, and transfer (25.1).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{
        ArrayBufferError, Budget, DataDescriptor, DescriptorKind, Objects, PropertyDescriptor,
    },
};
use spite_core::{JsString, PropertyKey, Span, WellKnownSymbol};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Member {
    ByteLength,
    Detached,
    MaxByteLength,
    Resizable,
    Resize,
    Slice,
    Transfer,
    TransferToFixedLength,
}

impl Member {
    pub(super) fn initial_name(self) -> &'static str {
        match self {
            Self::ByteLength => "get byteLength",
            Self::Detached => "get detached",
            Self::MaxByteLength => "get maxByteLength",
            Self::Resizable => "get resizable",
            Self::Resize => "resize",
            Self::Slice => "slice",
            Self::Transfer => "transfer",
            Self::TransferToFixedLength => "transferToFixedLength",
        }
    }

    fn property_name(self) -> &'static str {
        match self {
            Self::ByteLength => "byteLength",
            Self::Detached => "detached",
            Self::MaxByteLength => "maxByteLength",
            Self::Resizable => "resizable",
            _ => self.initial_name(),
        }
    }

    pub(super) fn length(self) -> f64 {
        match self {
            Self::Resize => 1.0,
            Self::Slice => 2.0,
            _ => 0.0,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ArrayBufferIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl ArrayBufferIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

#[derive(Clone, Copy)]
struct BufferState {
    byte_length: u64,
    max_byte_length: Option<u64>,
    detached: bool,
}

impl Realm {
    pub(super) fn array_buffer_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ArrayBufferIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::ArrayBuffer, span)?;
        // %ArrayBuffer.prototype% is ordinary and has no buffer internal slots.
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
        let is_view = self.new_builtin(function_prototype, Builtin::ArrayBufferIsView, span)?;
        self.define_builtin_property(
            &constructor,
            "isView",
            Value::Object(is_view.clone()),
            true,
            span,
        )?;
        methods.push(is_view);
        for member in [
            Member::Resize,
            Member::Slice,
            Member::Transfer,
            Member::TransferToFixedLength,
        ] {
            let method =
                self.new_builtin(function_prototype, Builtin::ArrayBufferMember(member), span)?;
            self.define_builtin_property(
                &prototype,
                member.property_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        for (object, key, builtin) in [
            (
                &constructor,
                WellKnownSymbol::Species.symbol().into(),
                Builtin::ArrayBufferSpecies,
            ),
            (
                &prototype,
                PropertyKey::from(JsString::from("byteLength")),
                Builtin::ArrayBufferMember(Member::ByteLength),
            ),
            (
                &prototype,
                PropertyKey::from(JsString::from("detached")),
                Builtin::ArrayBufferMember(Member::Detached),
            ),
            (
                &prototype,
                PropertyKey::from(JsString::from("maxByteLength")),
                Builtin::ArrayBufferMember(Member::MaxByteLength),
            ),
            (
                &prototype,
                PropertyKey::from(JsString::from("resizable")),
                Builtin::ArrayBufferMember(Member::Resizable),
            ),
        ] {
            let getter = self.new_builtin(function_prototype, builtin, span)?;
            self.define_property_or_throw(
                object,
                key,
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
                value: Some(Value::String(JsString::from("ArrayBuffer"))),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(true),
            }
            .into(),
            span,
        )?;
        Ok(ArrayBufferIntrinsics {
            constructor,
            prototype,
            methods,
        })
    }

    // ToIndex (7.1.22) truncates before checking the inclusive safe-index range.
    pub(super) fn array_buffer_index(&mut self, value: Value, span: Span) -> Result<u64, Error> {
        let number = self.number(value, span)?;
        let integer = if number.is_nan() { 0.0 } else { number.trunc() };
        if !(0.0..=9_007_199_254_740_991.0).contains(&integer) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "ArrayBuffer length is outside the safe index range",
            ));
        }
        Ok(integer as u64)
    }

    // GetArrayBufferMaxByteLengthOption (25.1.3.7), shared by both buffer kinds.
    // Primitive options are ignored rather than boxed.
    pub(super) fn array_buffer_max_byte_length_option(
        &mut self,
        options: Value,
        span: Span,
    ) -> Result<Option<u64>, Error> {
        Ok(if let Value::Object(options) = options {
            let maximum = self.get_property(&options, &JsString::from("maxByteLength"), span)?;
            if matches!(maximum, Value::Undefined) {
                None
            } else {
                Some(self.array_buffer_index(maximum, span)?)
            }
        } else {
            None
        })
    }

    pub(super) fn array_buffer_constructor(
        &mut self,
        new_target: ObjectHandle,
        length: Value,
        options: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let byte_length = self.array_buffer_index(length, span)?;
        let max_byte_length = self.array_buffer_max_byte_length_option(options, span)?;
        // AllocateArrayBuffer rejects this range before newTarget.prototype.
        if max_byte_length.is_some_and(|maximum| byte_length > maximum) {
            return Err(Self::exception(
                ExceptionKind::RangeError,
                span,
                "ArrayBuffer length exceeds maxByteLength",
            ));
        }
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = if let Value::Object(prototype) = prototype {
            prototype
        } else {
            self.check_function_realm(&new_target, span)?;
            self.intrinsics
                .as_ref()
                .expect("initialized")
                .array_buffer
                .prototype
                .clone()
        };
        self.array_buffer_work(span, |objects, budget| {
            objects.create_array_buffer(&prototype, byte_length, max_byte_length, budget)
        })
        .map(Value::Object)
    }

    pub(super) fn array_buffer_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Objects, &mut Budget) -> Result<T, ArrayBufferError>,
    ) -> Result<T, Error> {
        // Preserve host heap/work failures as host failures. Backing-store
        // allocation and invalid buffer states have specification exceptions.
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

    fn array_buffer_state(
        &mut self,
        object: &ObjectHandle,
        span: Span,
    ) -> Result<Option<BufferState>, Error> {
        self.object_work(span, |objects, _| {
            Ok(objects
                .inspect(object)?
                .array_buffer_data()
                .map(|data| BufferState {
                    byte_length: data.byte_length(),
                    max_byte_length: data.is_resizable().then(|| data.max_byte_length()),
                    detached: data.is_detached(),
                }))
        })
    }

    fn array_buffer_receiver(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<(ObjectHandle, BufferState), Error> {
        if let Value::Object(object) = receiver
            && let Some(state) = self.array_buffer_state(&object, span)?
        {
            return Ok((object, state));
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "receiver has no ArrayBufferData internal slot",
        ))
    }

    fn array_buffer_attached(state: BufferState, span: Span) -> Result<(), Error> {
        if state.detached {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "ArrayBuffer is detached",
            ));
        }
        Ok(())
    }

    pub(super) fn array_buffer_member(
        &mut self,
        member: Member,
        receiver: Value,
        argument: Value,
        end: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let (buffer, state) = self.array_buffer_receiver(receiver, span)?;
        match member {
            Member::ByteLength => Ok(Value::Number(state.byte_length as f64)),
            Member::Detached => Ok(Value::Boolean(state.detached)),
            Member::MaxByteLength => Ok(Value::Number(
                state.max_byte_length.unwrap_or(state.byte_length) as f64,
            )),
            Member::Resizable => Ok(Value::Boolean(state.max_byte_length.is_some())),
            Member::Resize => {
                // A missing max-length slot preempts argument conversion. A
                // detached resizable buffer still converts before rejecting.
                if state.max_byte_length.is_none() {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "ArrayBuffer is not resizable",
                    ));
                }
                let length = self.array_buffer_index(argument, span)?;
                self.array_buffer_work(span, |objects, budget| {
                    objects.resize_array_buffer(&buffer, length, budget)
                })?;
                Ok(Value::Undefined)
            }
            Member::Slice => self.array_buffer_slice(buffer, state, argument, end, span),
            Member::Transfer | Member::TransferToFixedLength => self.array_buffer_transfer(
                buffer,
                state,
                argument,
                matches!(member, Member::Transfer),
                span,
            ),
        }
    }

    pub(super) fn buffer_species_constructor(
        &mut self,
        buffer: &ObjectHandle,
        default: ObjectHandle,
        span: Span,
    ) -> Result<Value, Error> {
        // SpeciesConstructor (7.3.22) rejects primitive constructors before
        // reading species, and defaults to the intrinsic rather than a binding.
        let constructor = self.get_property(buffer, &JsString::from("constructor"), span)?;
        let species = match constructor {
            Value::Undefined => Value::Undefined,
            Value::Object(constructor) => {
                self.get_property(&constructor, &WellKnownSymbol::Species.symbol(), span)?
            }
            _ => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "buffer constructor property must be an Object or undefined",
                ));
            }
        };
        if matches!(species, Value::Undefined | Value::Null) {
            return Ok(Value::Object(default));
        }
        if !self.is_constructor(&species, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "buffer species must be a constructor",
            ));
        }
        Ok(species)
    }

    pub(super) fn array_buffer_relative_index(
        &mut self,
        value: Value,
        length: u64,
        span: Span,
    ) -> Result<u64, Error> {
        let number = self.number(value, span)?;
        let integer = if number.is_nan() { 0.0 } else { number.trunc() };
        Ok(if integer < 0.0 {
            (length as f64 + integer).max(0.0)
        } else {
            integer.min(length as f64)
        } as u64)
    }

    fn array_buffer_slice(
        &mut self,
        source: ObjectHandle,
        original: BufferState,
        start: Value,
        end: Value,
        span: Span,
    ) -> Result<Value, Error> {
        Self::array_buffer_attached(original, span)?;
        let first = self.array_buffer_relative_index(start, original.byte_length, span)?;
        let end = if matches!(end, Value::Undefined) {
            original.byte_length
        } else {
            self.array_buffer_relative_index(end, original.byte_length, span)?
        };
        let length = end.saturating_sub(first);
        let default = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .array_buffer
            .constructor
            .clone();
        let constructor = self.buffer_species_constructor(&source, default, span)?;
        let result = self.construct(constructor, vec![Value::Number(length as f64)], span)?;
        let (result, result_state) = self.array_buffer_receiver(result, span)?;
        Self::array_buffer_attached(result_state, span)?;
        if result == source {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "ArrayBuffer slice species returned its source",
            ));
        }
        if result_state.byte_length < length {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "ArrayBuffer slice species returned a short buffer",
            ));
        }
        // Conversions and species construction can resize or detach the source.
        // Recheck it only after all species-result checks (25.1.6.7).
        let current = self
            .array_buffer_state(&source, span)?
            .expect("buffer slot");
        Self::array_buffer_attached(current, span)?;
        if first < current.byte_length {
            let count = length.min(current.byte_length - first);
            self.array_buffer_work(span, |objects, budget| {
                objects.copy_array_buffer_bytes(&source, first, &result, 0, count, budget)
            })?;
        }
        Ok(Value::Object(result))
    }

    fn array_buffer_transfer(
        &mut self,
        source: ObjectHandle,
        original: BufferState,
        length: Value,
        preserve_resizability: bool,
        span: Span,
    ) -> Result<Value, Error> {
        // ArrayBufferCopyAndDetach (25.1.3.3) converts a supplied newLength
        // before checking detachment, then uses the post-conversion source state.
        let length = if matches!(length, Value::Undefined) {
            original.byte_length
        } else {
            self.array_buffer_index(length, span)?
        };
        let current = self
            .array_buffer_state(&source, span)?
            .expect("buffer slot");
        Self::array_buffer_attached(current, span)?;
        let maximum = if preserve_resizability {
            current.max_byte_length
        } else {
            None
        };
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .array_buffer
            .prototype
            .clone();
        // The intrinsic allocation bypasses public constructor/species hooks.
        // Allocate and copy before detaching, preserving the source on failure.
        let result = self.array_buffer_work(span, |objects, budget| {
            objects.create_array_buffer(&prototype, length, maximum, budget)
        })?;
        self.array_buffer_work(span, |objects, budget| {
            objects.copy_array_buffer_bytes(
                &source,
                0,
                &result,
                0,
                length.min(current.byte_length),
                budget,
            )
        })?;
        self.array_buffer_work(span, |objects, budget| {
            objects.detach_array_buffer(&source, budget)
        })?;
        Ok(Value::Object(result))
    }
}
