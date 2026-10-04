//! Ordinary method closures, MakeMethod, and SetFunctionName (15.4.4–5, 10.2.7/9).

use super::ScriptFunction;
use crate::{Error, ObjectHandle, Realm, Value};
use spite_core::{JsString, PropertyKey, Span};
use spite_parser::ast::{ArrowBody, Function, PropertyKind};

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub(crate) struct MethodFunction {
    pub code: ScriptFunction,
    pub home_object: ObjectHandle,
}

impl Realm {
    pub(crate) fn method_function(
        &mut self,
        syntax: &Function,
        home_object: &ObjectHandle,
        key: PropertyKey,
        kind: PropertyKind,
        span: Span,
    ) -> Result<ObjectHandle, Error> {
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .function_prototype
            .clone();
        let method = MethodFunction {
            code: ScriptFunction {
                environment: self.scopes.last().expect("active environment").clone(),
                parameters: syntax.parameters.clone(),
                body: ArrowBody::Block(syntax.body.clone()),
                source: syntax.source.clone(),
                strict: self.strict || syntax.body.is_strict(),
            },
            home_object: home_object.clone(),
        };
        let function =
            self.object_work(span, |objects, _| objects.create_method(&prototype, method))?;
        let length = syntax
            .parameters
            .iter()
            .take_while(|parameter| parameter.initializer.is_none())
            .count();
        self.define_builtin_property(
            &function,
            "length",
            Value::Number(length as f64),
            false,
            span,
        )?;
        let prefix = match kind {
            PropertyKind::Getter => Some("get"),
            PropertyKind::Setter => Some("set"),
            PropertyKind::Method => None,
            _ => unreachable!("method property kind"),
        };
        self.set_function_name(&function, key, prefix, span)?;
        // OrdinaryFunctionCreate without MakeConstructor: no own prototype and
        // no [[Construct]], independently of the method's public properties.
        Ok(function)
    }

    pub(crate) fn set_function_name(
        &mut self,
        function: &ObjectHandle,
        key: PropertyKey,
        prefix: Option<&str>,
        span: Span,
    ) -> Result<(), Error> {
        let mut name = match key {
            PropertyKey::String(name) => name,
            PropertyKey::Symbol(symbol) => self.symbol_function_name(&symbol, span)?,
        };
        if let Some(prefix) = prefix {
            let prefix_len = prefix.encode_utf16().count() + 1;
            let length = name
                .len()
                .checked_add(prefix_len)
                .filter(|length| {
                    self.limits
                        .max_string_units
                        .is_none_or(|limit| *length <= limit)
                })
                .ok_or_else(|| Error::Limit {
                    span,
                    message: "function name length limit exceeded".into(),
                })?;
            self.object_work(span, |_, budget| budget.charge(length))?;
            let mut units = Vec::new();
            units.try_reserve_exact(length).map_err(|_| Error::Limit {
                span,
                message: "function name allocation limit exceeded".into(),
            })?;
            units.extend(prefix.encode_utf16());
            units.push(u16::from(b' '));
            units.extend_from_slice(name.code_units());
            name = JsString::from_code_units(units);
        }
        let value = Value::String(name);
        self.check_string(&value, span)?;
        self.define_builtin_property(function, "name", value, false, span)
    }
}
