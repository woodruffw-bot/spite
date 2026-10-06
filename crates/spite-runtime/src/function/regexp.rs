//! RegExp intrinsic metadata and String escape encoding (22.2.5–6).

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
