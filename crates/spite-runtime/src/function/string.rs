//! String construction, StringData wrappers, and branded methods (22.1.1/3).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};

mod case;
mod character;
mod iteration;
mod raw;
mod repeat;
mod replace;
mod search;
mod sequence;
mod split;
#[cfg(test)]
mod tests;
mod trim;
mod well_formed;

#[derive(Debug)]
pub(crate) struct StringIntrinsics {
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
    to_string: ObjectHandle,
    value_of: ObjectHandle,
    methods: Vec<ObjectHandle>,
}

impl StringIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [
            &self.constructor,
            &self.prototype,
            &self.to_string,
            &self.value_of,
        ]
        .into_iter()
        .chain(self.methods.iter())
    }
}

impl Realm {
    pub(super) fn string_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<StringIntrinsics, Error> {
        let constructor = self.new_builtin(function_prototype, Builtin::String, span)?;
        let prototype = self.object_work(span, |objects, budget| {
            objects.create_string(object_prototype, JsString::from(""), budget)
        })?;
        let to_string = self.new_builtin(function_prototype, Builtin::StringToString, span)?;
        let value_of = self.new_builtin(function_prototype, Builtin::StringValueOf, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &constructor,
                JsString::from("prototype"),
                DataDescriptor {
                    value: Some(Value::Object(prototype.clone())),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(false),
                },
                budget,
            )
        })?;
        for (name, method) in [
            ("constructor", &constructor),
            ("toString", &to_string),
            ("valueOf", &value_of),
        ] {
            self.define_builtin_property(
                &prototype,
                name,
                Value::Object(method.clone()),
                true,
                span,
            )?;
        }
        let mut methods = Vec::new();
        for builtin in [
            Builtin::StringFromCharCode,
            Builtin::StringFromCodePoint,
            Builtin::StringRaw,
            Builtin::StringAt,
            Builtin::StringCharAt,
            Builtin::StringCharCodeAt,
            Builtin::StringCodePointAt,
            Builtin::StringIsWellFormed,
            Builtin::StringToWellFormed,
            Builtin::StringToLowerCase,
            Builtin::StringToUpperCase,
            Builtin::StringToLocaleLowerCase,
            Builtin::StringToLocaleUpperCase,
            Builtin::StringConcat,
            Builtin::StringSlice,
            Builtin::StringSubstring,
            Builtin::StringTrim,
            Builtin::StringTrimStart,
            Builtin::StringTrimEnd,
            Builtin::StringRepeat,
            Builtin::StringPadStart,
            Builtin::StringPadEnd,
            Builtin::StringIndexOf,
            Builtin::StringLastIndexOf,
            Builtin::StringIncludes,
            Builtin::StringStartsWith,
            Builtin::StringEndsWith,
            Builtin::StringSplit,
            Builtin::StringReplace,
            Builtin::StringReplaceAll,
        ] {
            let method = self.new_builtin(function_prototype, builtin, span)?;
            let target = if matches!(
                builtin,
                Builtin::StringFromCharCode | Builtin::StringFromCodePoint | Builtin::StringRaw
            ) {
                &constructor
            } else {
                &prototype
            };
            self.define_builtin_property(
                target,
                builtin.initial_name(),
                Value::Object(method.clone()),
                true,
                span,
            )?;
            methods.push(method);
        }
        let iterator = self.new_builtin(function_prototype, Builtin::StringIterator, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &prototype,
                WellKnownSymbol::Iterator.symbol(),
                DataDescriptor {
                    value: Some(Value::Object(iterator.clone())),
                    writable: Some(true),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        methods.push(iterator);
        Ok(StringIntrinsics {
            constructor,
            prototype,
            to_string,
            value_of,
            methods,
        })
    }

    pub(super) fn string_constructor(
        &mut self,
        new_target: Option<ObjectHandle>,
        value: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 22.1.1.1 converts the value before observing newTarget.prototype.
        let string = match value {
            Some(Value::Symbol(symbol)) if new_target.is_none() => {
                self.symbol_descriptive_string(&symbol, span)?
            }
            Some(value) => self.string(value, span)?,
            None => JsString::from(""),
        };
        let value = Value::String(string);
        self.check_string(&value, span)?;
        let Some(target) = new_target else {
            return Ok(value);
        };
        let prototype = self.get_property(&target, &JsString::from("prototype"), span)?;
        let prototype = match prototype {
            Value::Object(prototype) => prototype,
            _ => self
                .intrinsics
                .as_ref()
                .expect("initialized")
                .string
                .prototype
                .clone(),
        };
        let Value::String(string) = value else {
            unreachable!("String result");
        };
        self.object_work(span, |objects, budget| {
            objects.create_string(&prototype, string, budget)
        })
        .map(Value::Object)
    }

    pub(super) fn this_string_value(
        &mut self,
        value: &Value,
        span: Span,
    ) -> Result<JsString, Error> {
        let string = match value {
            Value::String(string) => self.object_work(span, |_, budget| {
                budget.charge(string.len())?;
                Ok(Some(string.clone()))
            })?,
            Value::Object(object) => self.object_work(span, |objects, budget| {
                let string = objects.inspect(object)?.string_data();
                if let Some(string) = string {
                    budget.charge(string.len())?;
                }
                Ok(string.cloned())
            })?,
            _ => None,
        };
        string.ok_or_else(|| {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver does not contain a String value",
            )
        })
    }

    fn copy_string_units(&mut self, units: &[u16], span: Span) -> Result<Value, Error> {
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| units.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        self.object_work(span, |_, budget| budget.charge(units.len()))?;
        Ok(Value::String(JsString::from_code_units(units.to_vec())))
    }
}
