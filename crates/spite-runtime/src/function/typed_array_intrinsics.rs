//! TypedArray intrinsic graph, state accessors, and iterator wrappers (23.2).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    binary_data::ElementType,
    object::{ArrayIterationKind, DataDescriptor, DescriptorKind, PropertyDescriptor},
};
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
    Deferred(u8),
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
            Self::Deferred(index) => DEFERRED_METHODS[usize::from(index)].0,
            Self::Uint8Encoding(index) => UINT8_METHODS[usize::from(index)].0,
        }
    }

    pub(super) fn length(self) -> f64 {
        match self {
            Self::Deferred(index) => f64::from(DEFERRED_METHODS[usize::from(index)].1),
            Self::Uint8Encoding(index) => f64::from(UINT8_METHODS[usize::from(index)].1),
            _ => 0.0,
        }
    }
}

// Standard descriptors remain reflectable while invocation reports a host
// Unsupported result, never a catchable JavaScript substitute.
const DEFERRED_METHODS: [(&str, u8); 27] = [
    ("at", 1),
    ("copyWithin", 2),
    ("every", 1),
    ("fill", 1),
    ("filter", 1),
    ("find", 1),
    ("findIndex", 1),
    ("findLast", 1),
    ("findLastIndex", 1),
    ("forEach", 1),
    ("includes", 1),
    ("indexOf", 1),
    ("join", 1),
    ("lastIndexOf", 1),
    ("map", 1),
    ("reduce", 1),
    ("reduceRight", 1),
    ("reverse", 0),
    ("set", 1),
    ("slice", 2),
    ("some", 1),
    ("sort", 1),
    ("subarray", 2),
    ("toLocaleString", 0),
    ("toReversed", 0),
    ("toSorted", 1),
    ("with", 2),
];

// The last column selects a constructor property rather than a prototype one.
const UINT8_METHODS: [(&str, u8, bool); 6] = [
    ("fromBase64", 1, true),
    ("fromHex", 1, true),
    ("setFromBase64", 1, false),
    ("setFromHex", 1, false),
    ("toBase64", 0, false),
    ("toHex", 0, false),
];

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
        for (index, (name, _)) in DEFERRED_METHODS.into_iter().enumerate() {
            let member = Member::Deferred(index as u8);
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
                for (index, (name, _, is_static)) in UINT8_METHODS.into_iter().enumerate() {
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
        span: Span,
    ) -> Result<Value, Error> {
        let unsupported_owner = match member {
            Member::Deferred(_) => Some("TypedArray.prototype"),
            Member::Uint8Encoding(index) => Some(if UINT8_METHODS[usize::from(index)].2 {
                "Uint8Array"
            } else {
                "Uint8Array.prototype"
            }),
            _ => None,
        };
        if let Some(owner) = unsupported_owner {
            let name = member.initial_name();
            return Err(Error::Unsupported {
                span,
                message: format!("{owner}.{name} is not implemented"),
            });
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
                .array_buffer_data()
                .expect("TypedArray's viewed buffer has ArrayBuffer slots");
            Ok(Some((
                data.buffer().clone(),
                data.element_type(),
                data.byte_offset(),
                data.in_bounds_array_length(buffer),
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
            Member::Deferred(_) | Member::Uint8Encoding(_) => {
                unreachable!("deferred methods returned above")
            }
        }
    }
}
