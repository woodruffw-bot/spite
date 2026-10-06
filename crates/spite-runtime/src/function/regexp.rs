//! RegExp metadata, String escape encoding and generic operations (22.2.5–6).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, Span, WellKnownSymbol, regexp_escape_units};

#[derive(Clone, Copy, Debug)]
pub(crate) enum Member {
    Exec,
    DotAll,
    Flags,
    Global,
    HasIndices,
    IgnoreCase,
    Match,
    MatchAll,
    Multiline,
    Replace,
    Search,
    Source,
    Split,
    Sticky,
    Test,
    ToString,
    Unicode,
    UnicodeSets,
}

impl Member {
    const ALL: [Self; 18] = [
        Self::Exec,
        Self::DotAll,
        Self::Flags,
        Self::Global,
        Self::HasIndices,
        Self::IgnoreCase,
        Self::Match,
        Self::MatchAll,
        Self::Multiline,
        Self::Replace,
        Self::Search,
        Self::Source,
        Self::Split,
        Self::Sticky,
        Self::Test,
        Self::ToString,
        Self::Unicode,
        Self::UnicodeSets,
    ];
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::Exec => "exec",
            Self::DotAll => "get dotAll",
            Self::Flags => "get flags",
            Self::Global => "get global",
            Self::HasIndices => "get hasIndices",
            Self::IgnoreCase => "get ignoreCase",
            Self::Match => "[Symbol.match]",
            Self::MatchAll => "[Symbol.matchAll]",
            Self::Multiline => "get multiline",
            Self::Replace => "[Symbol.replace]",
            Self::Search => "[Symbol.search]",
            Self::Source => "get source",
            Self::Split => "[Symbol.split]",
            Self::Sticky => "get sticky",
            Self::Test => "test",
            Self::ToString => "toString",
            Self::Unicode => "get unicode",
            Self::UnicodeSets => "get unicodeSets",
        }
    }
    pub(super) fn length(self) -> f64 {
        match self {
            Self::Replace | Self::Split => 2.0,
            Self::Exec | Self::Match | Self::MatchAll | Self::Search | Self::Test => 1.0,
            _ => 0.0,
        }
    }
    fn symbol(self) -> Option<WellKnownSymbol> {
        match self {
            Self::Match => Some(WellKnownSymbol::Match),
            Self::MatchAll => Some(WellKnownSymbol::MatchAll),
            Self::Replace => Some(WellKnownSymbol::Replace),
            Self::Search => Some(WellKnownSymbol::Search),
            Self::Split => Some(WellKnownSymbol::Split),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RegExpIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: Vec<ObjectHandle>,
}
impl RegExpIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(&self.methods)
    }
}

impl Realm {
    pub(super) fn regexp_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<RegExpIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::RegExp, span)?;
        // 22.2.6: the prototype is ordinary and owns no RegExp internal slots.
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
        methods.try_reserve_exact(20).map_err(|_| Error::Limit {
            span,
            message: "RegExp intrinsic allocation failed".into(),
        })?;
        let escape = self.new_builtin(function_prototype, Builtin::RegExpEscape, span)?;
        self.define_builtin_property(
            &constructor,
            "escape",
            Value::Object(escape.clone()),
            true,
            span,
        )?;
        methods.push(escape);
        let species = self.new_builtin(function_prototype, Builtin::RegExpSpecies, span)?;
        self.define_property_or_throw(
            &constructor,
            WellKnownSymbol::Species.symbol(),
            PropertyDescriptor {
                kind: DescriptorKind::Accessor {
                    get: Some(Some(species.clone())),
                    set: Some(None),
                },
                enumerable: Some(false),
                configurable: Some(true),
            },
            span,
        )?;
        methods.push(species);
        for member in Member::ALL {
            let function =
                self.new_builtin(function_prototype, Builtin::RegExpMember(member), span)?;
            let key: spite_core::PropertyKey = member
                .symbol()
                .map(|symbol| symbol.symbol().into())
                .unwrap_or_else(|| {
                    JsString::from(member.name().strip_prefix("get ").unwrap_or(member.name()))
                        .into()
                });
            let descriptor = if member.name().starts_with("get ") {
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(function.clone())),
                        set: Some(None),
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                }
            } else {
                DataDescriptor {
                    value: Some(Value::Object(function.clone())),
                    writable: Some(true),
                    enumerable: Some(false),
                    configurable: Some(true),
                }
                .into()
            };
            self.define_property_or_throw(&prototype, key, descriptor, span)?;
            methods.push(function);
        }
        Ok(RegExpIntrinsics {
            constructor,
            prototype,
            methods,
        })
    }

    pub(super) fn regexp_escape(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        let Value::String(string) = value else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "RegExp.escape requires a String",
            ));
        };
        self.object_work(span, |_, budget| budget.charge(string.len()))?;
        let units = regexp_escape_units(&string);
        let length = units
            .clone()
            .try_fold(0usize, |n, _| n.checked_add(1))
            .ok_or_else(|| regexp_output_limit(span))?;
        let mut result = self.regexp_string_buffer(length, span)?;
        result.extend(units);
        Ok(Value::String(JsString::from_code_units(result)))
    }

    fn regexp_string_buffer(&mut self, length: usize, span: Span) -> Result<Vec<u16>, Error> {
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| length > limit)
        {
            return Err(regexp_output_limit(span));
        }
        self.object_work(span, |_, budget| budget.charge(length))?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(length)
            .map_err(|_| regexp_output_limit(span))?;
        Ok(result)
    }
}

fn regexp_output_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "RegExp output exceeds string limit or platform capacity".into(),
    }
}

impl Realm {
    #[inline(never)]
    pub(super) fn regexp_member(
        &mut self,
        member: Member,
        receiver: Value,
        argument: Value,
        span: Span,
    ) -> Result<Value, Error> {
        match member {
            Member::Flags => self.regexp_flags(receiver, span),
            Member::ToString => self.regexp_to_string(receiver, span),
            Member::Test => self.regexp_test(receiver, argument, span),
            Member::DotAll
            | Member::Global
            | Member::HasIndices
            | Member::IgnoreCase
            | Member::Multiline
            | Member::Source
            | Member::Sticky
            | Member::Unicode
            | Member::UnicodeSets => {
                let object = Self::regexp_object_receiver(receiver, span)?;
                if object
                    == self
                        .intrinsics
                        .as_ref()
                        .expect("initialized")
                        .regexp
                        .prototype
                {
                    if matches!(member, Member::Source) {
                        let mut units = self.regexp_string_buffer(4, span)?;
                        units.extend("(?:)".encode_utf16());
                        Ok(Value::String(JsString::from_code_units(units)))
                    } else {
                        Ok(Value::Undefined)
                    }
                } else {
                    // RegExp instances and their OriginalSource/Flags slots are
                    // still pending. Ordinary objects must never acquire them
                    // from their prototype or public source/flags properties.
                    Err(Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "receiver has no RegExp internal slots",
                    ))
                }
            }
            _ => Err(Self::unsupported(
                span,
                "native RegExp matching and symbol operations",
            )),
        }
    }
    fn regexp_object_receiver(value: Value, span: Span) -> Result<ObjectHandle, Error> {
        if let Value::Object(object) = value {
            Ok(object)
        } else {
            Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "RegExp operation requires an Object receiver",
            ))
        }
    }
    fn regexp_flags(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        let mut units = [0u16; 8];
        let mut length = 0;
        // 22.2.6.4 specifies this observable Get/ToBoolean order. Both Unicode
        // flags can appear here because this getter is generic, not a parser.
        for (name, flag) in [
            ("hasIndices", b'd'),
            ("global", b'g'),
            ("ignoreCase", b'i'),
            ("multiline", b'm'),
            ("dotAll", b's'),
            ("unicode", b'u'),
            ("unicodeSets", b'v'),
            ("sticky", b'y'),
        ] {
            if self
                .get_property(&object, &JsString::from(name), span)?
                .to_boolean()
            {
                units[length] = u16::from(flag);
                length += 1;
            }
        }
        let mut output = self.regexp_string_buffer(length, span)?;
        output.extend_from_slice(&units[..length]);
        Ok(Value::String(JsString::from_code_units(output)))
    }
    fn regexp_to_string(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        let source = self.get_property(&object, &JsString::from("source"), span)?;
        let source = self.string(source, span)?;
        let flags = self.get_property(&object, &JsString::from("flags"), span)?;
        let flags = self.string(flags, span)?;
        let length = source
            .len()
            .checked_add(flags.len())
            .and_then(|n| n.checked_add(2))
            .ok_or_else(|| regexp_output_limit(span))?;
        let mut units = self.regexp_string_buffer(length, span)?;
        units.push(u16::from(b'/'));
        units.extend_from_slice(source.code_units());
        units.push(u16::from(b'/'));
        units.extend_from_slice(flags.code_units());
        Ok(Value::String(JsString::from_code_units(units)))
    }
    fn regexp_test(
        &mut self,
        receiver: Value,
        argument: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        let string = self.string(argument, span)?;
        let exec = self.get_property(&object, &JsString::from("exec"), span)?;
        if !self.is_callable(&exec, span)? {
            // RequireInternalSlot in RegExpExec precedes its native fallback.
            // No native RegExpMatcher objects are exposed yet.
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver has no RegExpMatcher internal slot",
            ));
        }
        let result = self.call(
            exec,
            Value::Object(object),
            vec![Value::String(string)],
            span,
        )?;
        match result {
            Value::Null => Ok(Value::Boolean(false)),
            Value::Object(_) => Ok(Value::Boolean(true)),
            _ => Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "RegExp exec must return an Object or null",
            )),
        }
    }
}
