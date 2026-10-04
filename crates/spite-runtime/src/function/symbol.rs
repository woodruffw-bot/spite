//! Symbol primitives, wrappers, and intrinsics (20.4).

use super::Builtin;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{DataDescriptor, DescriptorKind, PropertyDescriptor},
};
use spite_core::{JsString, JsSymbol, Span, WellKnownSymbol};

#[derive(Debug)]
pub(crate) struct SymbolIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    methods: [ObjectHandle; 4],
}

impl SymbolIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.constructor, &self.prototype]
            .into_iter()
            .chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn symbol_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<SymbolIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::Symbol, span)?;
        // Symbol.prototype is ordinary and does not itself have SymbolData.
        let prototype =
            self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        let to_string = self.new_builtin(function_prototype, Builtin::SymbolToString, span)?;
        let value_of = self.new_builtin(function_prototype, Builtin::SymbolValueOf, span)?;
        let description = self.new_builtin(function_prototype, Builtin::SymbolDescription, span)?;
        let to_primitive =
            self.new_builtin(function_prototype, Builtin::SymbolToPrimitive, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &constructor,
                "prototype",
                DataDescriptor {
                    value: Some(Value::Object(prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )?;
            for key in WellKnownSymbol::ALL {
                objects.define(
                    &constructor,
                    key.name(),
                    DataDescriptor {
                        value: Some(Value::Symbol(key.symbol())),
                        writable: Some(false),
                        enumerable: Some(false),
                        configurable: Some(false),
                    },
                    budget,
                )?;
            }
            objects.define(
                &prototype,
                "description",
                PropertyDescriptor {
                    kind: DescriptorKind::Accessor {
                        get: Some(Some(description.clone())),
                        set: Some(None),
                    },
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )?;
            for (key, value) in [
                (
                    WellKnownSymbol::ToPrimitive,
                    Value::Object(to_primitive.clone()),
                ),
                (
                    WellKnownSymbol::ToStringTag,
                    Value::String(JsString::from("Symbol")),
                ),
            ] {
                objects.define(
                    &prototype,
                    key.symbol(),
                    DataDescriptor {
                        value: Some(value),
                        writable: Some(false),
                        enumerable: Some(false),
                        configurable: Some(true),
                    },
                    budget,
                )?;
            }
            Ok(())
        })?;
        for (name, handle) in [
            ("constructor", &constructor),
            ("toString", &to_string),
            ("valueOf", &value_of),
        ] {
            self.define_builtin_property(
                &prototype,
                name,
                Value::Object(handle.clone()),
                true,
                span,
            )?;
        }
        Ok(SymbolIntrinsics {
            constructor,
            prototype,
            methods: [to_string, value_of, description, to_primitive],
        })
    }

    pub(crate) fn symbol_constructor(
        &mut self,
        description: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // Symbol has no [[Construct]]; construction is rejected by the common
        // dispatch before this conversion (20.4.1.1).
        let description = if matches!(description, Value::Undefined) {
            None
        } else {
            let description = self.string(description, span)?;
            if description.len() > self.limits.max_string_units {
                return Err(Error::Limit {
                    span,
                    message: "Symbol description length limit exceeded".into(),
                });
            }
            Some(description)
        };
        self.tick(span)?;
        Ok(Value::Symbol(JsSymbol::new(description)))
    }

    pub(crate) fn this_symbol_value(
        &mut self,
        value: &Value,
        span: Span,
    ) -> Result<JsSymbol, Error> {
        let symbol = match value {
            Value::Symbol(symbol) => Some(symbol.clone()),
            Value::Object(object) => self.object_work(span, |objects, _| {
                Ok(objects.inspect(object)?.symbol_data().cloned())
            })?,
            _ => None,
        };
        symbol.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver does not contain a Symbol value",
            )
        })
    }

    pub(crate) fn symbol_description(&mut self, value: &Value, span: Span) -> Result<Value, Error> {
        let symbol = self.this_symbol_value(value, span)?;
        if symbol.description().is_none() {
            return Ok(Value::Undefined);
        }
        self.symbol_string(&symbol, "", "", span).map(Value::String)
    }

    /// SymbolDescriptiveString (20.4.3.3.1), preserving UTF-16 description units.
    pub(crate) fn symbol_descriptive_string(
        &mut self,
        symbol: &JsSymbol,
        span: Span,
    ) -> Result<JsString, Error> {
        self.symbol_string(symbol, "Symbol(", ")", span)
    }

    /// SetFunctionName's Symbol branch (10.2.9).
    pub(crate) fn symbol_function_name(
        &mut self,
        symbol: &JsSymbol,
        span: Span,
    ) -> Result<JsString, Error> {
        if symbol.description().is_none() {
            return Ok(JsString::from(""));
        }
        self.symbol_string(symbol, "[", "]", span)
    }

    fn symbol_string(
        &mut self,
        symbol: &JsSymbol,
        prefix: &str,
        suffix: &str,
        span: Span,
    ) -> Result<JsString, Error> {
        let limit = || Error::Limit {
            span,
            message: "Symbol description output limit exceeded".into(),
        };
        let description = symbol.description();
        let length = description
            .map_or(0, JsString::len)
            .checked_add(prefix.encode_utf16().count())
            .and_then(|length| length.checked_add(suffix.encode_utf16().count()))
            .ok_or_else(limit)?;
        if length > self.limits.max_string_units {
            return Err(limit());
        }
        self.object_work(span, |_, budget| budget.charge(length))?;
        let mut units = Vec::new();
        units.try_reserve_exact(length).map_err(|_| limit())?;
        units.extend(prefix.encode_utf16());
        if let Some(description) = description {
            units.extend_from_slice(description.code_units());
        }
        units.extend(suffix.encode_utf16());
        Ok(JsString::from_code_units(units))
    }
}

#[cfg(test)]
mod tests;
