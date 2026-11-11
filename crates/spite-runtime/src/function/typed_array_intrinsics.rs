//! TypedArray intrinsic graph, state accessors, and iterator wrappers (23.2).

use super::Builtin;
use super::typed_array_methods::Method;
use super::uint8_base64::Method as Base64Method;
use super::uint8_hex::Method as HexMethod;
use crate::binary_data::ElementType;
use crate::object::{ArrayIterationKind, DataDescriptor, DescriptorKind, PropertyDescriptor};
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, PropertyKey, Span, WellKnownSymbol};

pub(crate) const ELEMENT_TYPES: [ElementType; 12] = [
    ElementType::Int8,
    ElementType::Uint8,
    ElementType::Uint8Clamped,
    ElementType::Int16,
    ElementType::Uint16,
    ElementType::Int32,
    ElementType::Uint32,
    ElementType::BigInt64,
    ElementType::BigUint64,
    ElementType::Float16,
    ElementType::Float32,
    ElementType::Float64,
];

pub(crate) fn typed_array_name(element: ElementType) -> &'static str {
    match element {
        ElementType::Int8 => "Int8Array",
        ElementType::Uint8 => "Uint8Array",
        ElementType::Uint8Clamped => "Uint8ClampedArray",
        ElementType::Int16 => "Int16Array",
        ElementType::Uint16 => "Uint16Array",
        ElementType::Int32 => "Int32Array",
        ElementType::Uint32 => "Uint32Array",
        ElementType::BigInt64 => "BigInt64Array",
        ElementType::BigUint64 => "BigUint64Array",
        ElementType::Float16 => "Float16Array",
        ElementType::Float32 => "Float32Array",
        ElementType::Float64 => "Float64Array",
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Member {
    Buffer,
    ByteLength,
    ByteOffset,
    Length,
    ToStringTag,
    Iterator(ArrayIterationKind),
    // Indices come only from enumerating the corresponding fixed tables.
    PrototypeMethod(u8),
    Uint8Encoding(u8),
}

impl Member {
    pub(super) fn initial_name(self) -> &'static str {
        match self {
            Self::Buffer => "get buffer",
            Self::ByteLength => "get byteLength",
            Self::ByteOffset => "get byteOffset",
            Self::Length => "get length",
            Self::ToStringTag => "get [Symbol.toStringTag]",
            Self::Iterator(ArrayIterationKind::Key) => "keys",
            Self::Iterator(ArrayIterationKind::Value) => "values",
            Self::Iterator(ArrayIterationKind::KeyValue) => "entries",
            Self::PrototypeMethod(index) => PROTOTYPE_METHODS[usize::from(index)].0,
            Self::Uint8Encoding(index) => UINT8_METHODS[usize::from(index)].0,
        }
    }

    pub(super) fn length(self) -> f64 {
        match self {
            Self::PrototypeMethod(index) => f64::from(PROTOTYPE_METHODS[usize::from(index)].1),
            Self::Uint8Encoding(index) => f64::from(UINT8_METHODS[usize::from(index)].1),
            _ => 0.0,
        }
    }
}

// Initial names, lengths and invocation methods share one ordered catalogue.
const PROTOTYPE_METHODS: [(&str, u8, Method); 27] = [
    ("at", 1, Method::At),
    ("copyWithin", 2, Method::CopyWithin),
    ("every", 1, Method::Every),
    ("fill", 1, Method::Fill),
    ("filter", 1, Method::Filter),
    ("find", 1, Method::Find),
    ("findIndex", 1, Method::FindIndex),
    ("findLast", 1, Method::FindLast),
    ("findLastIndex", 1, Method::FindLastIndex),
    ("forEach", 1, Method::ForEach),
    ("includes", 1, Method::Includes),
    ("indexOf", 1, Method::IndexOf),
    ("join", 1, Method::Join),
    ("lastIndexOf", 1, Method::LastIndexOf),
    ("map", 1, Method::Map),
    ("reduce", 1, Method::Reduce),
    ("reduceRight", 1, Method::ReduceRight),
    ("reverse", 0, Method::Reverse),
    ("set", 1, Method::Set),
    ("slice", 2, Method::Slice),
    ("some", 1, Method::Some),
    ("sort", 1, Method::Sort),
    ("subarray", 2, Method::Subarray),
    ("toLocaleString", 0, Method::ToLocaleString),
    ("toReversed", 0, Method::ToReversed),
    ("toSorted", 1, Method::ToSorted),
    ("with", 2, Method::With),
];

#[derive(Clone, Copy)]
enum EncodingMethod {
    Base64(Base64Method),
    Hex(HexMethod),
}

// The bool selects a constructor property rather than a prototype one.
const UINT8_METHODS: [(&str, u8, bool, EncodingMethod); 6] = {
    use EncodingMethod::{Base64, Hex};
    [
        ("fromBase64", 1, true, Base64(Base64Method::From)),
        ("fromHex", 1, true, Hex(HexMethod::From)),
        ("setFromBase64", 1, false, Base64(Base64Method::SetFrom)),
        ("setFromHex", 1, false, Hex(HexMethod::SetFrom)),
        ("toBase64", 0, false, Base64(Base64Method::To)),
        ("toHex", 0, false, Hex(HexMethod::To)),
    ]
};

#[derive(Debug)]
struct ConcreteIntrinsics {
    element: ElementType,
    constructor: ObjectHandle,
    prototype: ObjectHandle,
}

#[derive(Debug)]
pub(crate) struct TypedArrayIntrinsics {
    abstract_constructor: ObjectHandle,
    shared_prototype: ObjectHandle,
    concrete: Vec<ConcreteIntrinsics>,
    methods: Vec<ObjectHandle>,
}

impl TypedArrayIntrinsics {
    pub(crate) fn constructor(&self, element: ElementType) -> &ObjectHandle {
        &self.concrete(element).constructor
    }

    pub(crate) fn prototype(&self, element: ElementType) -> &ObjectHandle {
        &self.concrete(element).prototype
    }

    fn concrete(&self, element: ElementType) -> &ConcreteIntrinsics {
        self.concrete
            .iter()
            .find(|entry| entry.element == element)
            .expect("every TypedArray element kind has concrete intrinsics")
    }

    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.abstract_constructor, &self.shared_prototype]
            .into_iter()
            .chain(
                self.concrete
                    .iter()
                    .flat_map(|entry| [&entry.constructor, &entry.prototype]),
            )
            .chain(&self.methods)
    }
}

impl Realm {
    fn typed_array_fixed_property(
        &mut self,
        object: &ObjectHandle,
        key: &str,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        self.define_property_or_throw(
            object,
            JsString::from(key),
            DataDescriptor {
                value: Some(value),
                writable: Some(false),
                enumerable: Some(false),
                configurable: Some(false),
            }
            .into(),
            span,
        )
    }

    pub(super) fn typed_array_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        array_to_string: &ObjectHandle,
        span: Span,
    ) -> Result<TypedArrayIntrinsics, Error> {
        let abstract_constructor =
            self.new_builtin(function_prototype, Builtin::TypedArrayAbstract, span)?;
        let shared_prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        self.typed_array_fixed_property(
            &abstract_constructor,
            "prototype",
            Value::Object(shared_prototype.clone()),
            span,
        )?;
        self.define_builtin_property(
            &shared_prototype,
            "constructor",
            Value::Object(abstract_constructor.clone()),
            true,
            span,
        )?;
        let mut methods = Vec::new();
        for builtin in [Builtin::TypedArrayFrom, Builtin::TypedArrayOf] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            self.define_builtin_property(
                &abstract_constructor,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        for (object, key, builtin) in [
            (
                &abstract_constructor,
                PropertyKey::from(WellKnownSymbol::Species.symbol()),
                Builtin::TypedArraySpecies,
            ),
            (
                &shared_prototype,
                PropertyKey::from(JsString::from("buffer")),
                Builtin::TypedArrayMember(Member::Buffer),
            ),
            (
                &shared_prototype,
                PropertyKey::from(JsString::from("byteLength")),
                Builtin::TypedArrayMember(Member::ByteLength),
            ),
            (
                &shared_prototype,
                PropertyKey::from(JsString::from("byteOffset")),
                Builtin::TypedArrayMember(Member::ByteOffset),
            ),
            (
                &shared_prototype,
                PropertyKey::from(JsString::from("length")),
                Builtin::TypedArrayMember(Member::Length),
            ),
            (
                &shared_prototype,
                PropertyKey::from(WellKnownSymbol::ToStringTag.symbol()),
                Builtin::TypedArrayMember(Member::ToStringTag),
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
        for kind in [
            ArrayIterationKind::Key,
            ArrayIterationKind::Value,
            ArrayIterationKind::KeyValue,
        ] {
            let member = Member::Iterator(kind);
            let method =
                self.new_builtin(function_prototype, Builtin::TypedArrayMember(member), span)?;
            self.define_builtin_property(
                &shared_prototype,
                member.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            if kind == ArrayIterationKind::Value {
                self.define_property_or_throw(
                    &shared_prototype,
                    WellKnownSymbol::Iterator.symbol(),
                    DataDescriptor {
                        value: Some(Value::Object(method.clone())),
                        writable: Some(true),
                        enumerable: Some(false),
                        configurable: Some(true),
                    }
                    .into(),
                    span,
                )?;
            }
            methods.push(method);
        }
        // 23.2.3.34 aliases the initial Array intrinsic, even after programs
        // subsequently replace the public Array.prototype.toString property.
        self.define_builtin_property(
            &shared_prototype,
            "toString",
            Value::Object(array_to_string.clone()),
            true,
            span,
        )?;
        for (index, (name, _, _)) in PROTOTYPE_METHODS.into_iter().enumerate() {
            let member = Member::PrototypeMethod(index as u8);
            let method =
                self.new_builtin(function_prototype, Builtin::TypedArrayMember(member), span)?;
            self.define_builtin_property(
                &shared_prototype,
                name,
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        let mut concrete = Vec::new();
        for element in ELEMENT_TYPES {
            let constructor =
                self.new_builtin(&abstract_constructor, Builtin::TypedArray(element), span)?;
            let prototype =
                self.object_work(span, |objects, _| objects.create(Some(&shared_prototype)))?;
            self.typed_array_fixed_property(
                &constructor,
                "prototype",
                Value::Object(prototype.clone()),
                span,
            )?;
            for target in [&constructor, &prototype] {
                self.typed_array_fixed_property(
                    target,
                    "BYTES_PER_ELEMENT",
                    Value::Number(element.size() as f64),
                    span,
                )?;
            }
            self.define_builtin_property(
                &prototype,
                "constructor",
                Value::Object(constructor.clone()),
                true,
                span,
            )?;
            if element == ElementType::Uint8 {
                for (index, (name, _, is_static, _)) in UINT8_METHODS.into_iter().enumerate() {
                    let target = if is_static { &constructor } else { &prototype };
                    let member = Member::Uint8Encoding(index as u8);
                    let method = self.new_builtin(
                        function_prototype,
                        Builtin::TypedArrayMember(member),
                        span,
                    )?;
                    self.define_builtin_property(
                        target,
                        name,
                        Value::Object(method.clone()),
                        true,
                        span,
                    )?;
                    methods.push(method);
                }
            }
            concrete.push(ConcreteIntrinsics {
                element,
                constructor,
                prototype,
            });
        }
        Ok(TypedArrayIntrinsics {
            abstract_constructor,
            shared_prototype,
            concrete,
            methods,
        })
    }

    pub(crate) fn define_typed_array_globals(
        &mut self,
        global: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        for element in ELEMENT_TYPES {
            let constructor = self
                .record
                .intrinsics
                .as_ref()
                .expect("initialized")
                .typed_array
                .constructor(element)
                .clone();
            self.define_builtin_property(
                global,
                typed_array_name(element),
                Value::Object(constructor),
                true,
                span,
            )?;
        }
        Ok(())
    }

    pub(super) fn typed_array_member(
        &mut self,
        member: Member,
        receiver: Value,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        if let Member::PrototypeMethod(index) = member {
            return self.typed_array_method(
                PROTOTYPE_METHODS[usize::from(index)].2,
                receiver,
                arguments,
                span,
            );
        }
        if let Member::Uint8Encoding(index) = member {
            return match UINT8_METHODS[usize::from(index)].3 {
                EncodingMethod::Hex(method) => {
                    self.uint8_hex_method(method, receiver, arguments, span)
                }
                EncodingMethod::Base64(method) => {
                    self.uint8_base64_method(method, receiver, arguments, span)
                }
            };
        }
        let tag = matches!(member, Member::ToStringTag);
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver has no TypedArray internal slots",
            )
        };
        let Value::Object(object) = receiver else {
            return if tag {
                Ok(Value::Undefined)
            } else {
                Err(brand_error())
            };
        };
        let state = self.object_work(span, |objects, _| {
            let Some(data) = objects.inspect(&object)?.typed_array_data() else {
                return Ok(None);
            };
            let buffer = objects
                .inspect(data.buffer())?
                .buffer_info()?
                .expect("TypedArray's viewed buffer has ArrayBuffer slots");
            Ok(Some((
                data.buffer().clone(),
                data.element_type(),
                data.byte_offset(),
                data.in_bounds_array_length(&buffer),
            )))
        })?;
        let Some((buffer, element, byte_offset, length)) = state else {
            return if tag {
                Ok(Value::Undefined)
            } else {
                Err(brand_error())
            };
        };
        match member {
            Member::Buffer => Ok(Value::Object(buffer)),
            Member::ByteLength => Ok(Value::Number(
                length.map_or(0, |length| length * element.size() as u64) as f64,
            )),
            Member::ByteOffset => Ok(Value::Number(if length.is_some() {
                byte_offset as f64
            } else {
                0.0
            })),
            Member::Length => Ok(Value::Number(length.unwrap_or(0) as f64)),
            Member::ToStringTag => Ok(Value::String(JsString::from(typed_array_name(element)))),
            Member::Iterator(kind) => {
                if length.is_none() {
                    return Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "TypedArray is out of bounds or detached",
                    ));
                }
                self.array_iterator(Value::Object(object), kind, span)
            }
            Member::PrototypeMethod(_) | Member::Uint8Encoding(_) => {
                unreachable!("method dispatch returned above")
            }
        }
    }
}
