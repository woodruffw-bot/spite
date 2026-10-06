//! Generic RegExp replacement via custom exec (22.2.6.11).

use super::advance_string_index;
use crate::{Error, ObjectHandle, Realm, Value, function::string::Substitution};
use spite_core::{JsString, Span};

enum Replacement {
    Function(Value),
    Text(JsString),
}

impl Realm {
    pub(super) fn regexp_replace(
        &mut self,
        receiver: Value,
        argument: Value,
        replacement: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let object = Self::regexp_object_receiver(receiver, span)?;
        let string = self.string(argument, span)?;
        let replacement = if self.is_callable(&replacement, span)? {
            Replacement::Function(replacement)
        } else {
            Replacement::Text(self.string(replacement, span)?)
        };
        let flags = self.get_property(&object, &JsString::from("flags"), span)?;
        let flags = self.string(flags, span)?;
        self.object_work(span, |_, budget| budget.charge(flags.len()))?;
        let mut global = false;
        let mut full_unicode = false;
        for &unit in flags.code_units() {
            global |= unit == u16::from(b'g');
            full_unicode |= unit == u16::from(b'u') || unit == u16::from(b'v');
        }
        let last_index = JsString::from("lastIndex");
        if global {
            self.set_property_or_throw(&object, last_index.clone(), Value::Number(0.0), span)?;
        }
        // All exec results are collected before any replacement callback or
        // capture reads. Keep the actual objects so later accesses stay live.
        let mut results: Vec<ObjectHandle> = Vec::new();
        loop {
            self.tick(span)?;
            let result = self.regexp_exec(&object, &string, span)?;
            let Value::Object(result) = result else {
                break;
            };
            results.try_reserve(1).map_err(|_| storage_limit(span))?;
            results.push(result.clone());
            if !global {
                break;
            }
            let value = self.get_property(&result, &JsString::from("0"), span)?;
            let matched = self.string(value, span)?;
            if matched.is_empty() {
                let value = self.get_property(&object, &last_index, span)?;
                let index = self.length_from_value(value, span)?;
                let next = advance_string_index(&string, index, full_unicode);
                self.set_property_or_throw(
                    &object,
                    last_index.clone(),
                    Value::Number(next as f64),
                    span,
                )?;
            }
        }
        if results.is_empty() {
            return Ok(Value::String(string));
        }
        let mut output = Vec::new();
        let mut next_source = 0usize;
        for result in results {
            self.tick(span)?;
            let length = self.length_of_array_like(&result, span)?;
            let value = self.get_property(&result, &JsString::from("0"), span)?;
            let matched = self.string(value, span)?;
            let value = self.get_property(&result, &JsString::from("index"), span)?;
            let number = self.number(value, span)?;
            let position = if number.is_nan() || number <= 0.0 {
                0
            } else {
                (number.trunc().min(string.len() as f64) as usize).min(string.len())
            };
            let mut captures = Vec::new();
            for index in 1..length {
                self.tick(span)?;
                let value =
                    self.get_property(&result, &JsString::from(index.to_string().as_str()), span)?;
                let value = if matches!(value, Value::Undefined) {
                    value
                } else {
                    Value::String(self.string(value, span)?)
                };
                captures.try_reserve(1).map_err(|_| storage_limit(span))?;
                captures.push(value);
            }
            let named = self.get_property(&result, &JsString::from("groups"), span)?;
            let replacement = match &replacement {
                Replacement::Function(function) => {
                    let extra = if matches!(named, Value::Undefined) {
                        3
                    } else {
                        4
                    };
                    captures
                        .try_reserve(extra)
                        .map_err(|_| storage_limit(span))?;
                    captures.insert(0, Value::String(matched.clone()));
                    captures.push(Value::Number(position as f64));
                    captures.push(Value::String(string.clone()));
                    if !matches!(named, Value::Undefined) {
                        captures.push(named);
                    }
                    let value = self.call(function.clone(), Value::Undefined, captures, span)?;
                    self.string(value, span)?
                }
                Replacement::Text(template) => {
                    let named = if matches!(named, Value::Undefined) {
                        None
                    } else {
                        let Value::Object(object) = self.box_primitive(named, span)? else {
                            unreachable!("ToObject");
                        };
                        Some(object)
                    };
                    let mut units = Vec::new();
                    self.append_substitution(
                        &mut units,
                        Substitution {
                            string: &string,
                            matched: &matched,
                            position,
                            captures: &captures,
                            named: named.as_ref(),
                        },
                        template,
                        span,
                    )?;
                    JsString::from_code_units(units)
                }
            };
            // Even an overlapping/backward result performs all capture reads,
            // replacement calls and conversions before its output is ignored.
            if position >= next_source {
                self.append_replacement_units(
                    &mut output,
                    &string.code_units()[next_source..position],
                    span,
                )?;
                self.append_replacement_units(&mut output, replacement.code_units(), span)?;
                next_source = position
                    .checked_add(matched.len())
                    .ok_or_else(|| storage_limit(span))?;
            }
        }
        if next_source < string.len() {
            self.append_replacement_units(&mut output, &string.code_units()[next_source..], span)?;
        }
        Ok(Value::String(JsString::from_code_units(output)))
    }
}

fn storage_limit(span: Span) -> Error {
    Error::Limit {
        span,
        message: "RegExp replacement storage exceeds platform capacity".into(),
    }
}
