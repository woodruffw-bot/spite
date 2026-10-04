//! JSON object, ParseJSON, and iterative reviver traversal (25.5.1).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span, WellKnownSymbol};
use spite_parser::json::{JsonKind, parse_json_with_work};

mod reviver;

#[derive(Debug)]
pub(crate) struct JsonIntrinsics {
    pub object: ObjectHandle,
    parse: ObjectHandle,
}

impl JsonIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        [&self.object, &self.parse].into_iter()
    }
}

impl Realm {
    pub(super) fn json_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<JsonIntrinsics, Error> {
        let object = self.object_work(span, |objects, _| objects.create(Some(object_prototype)))?;
        let parse = self.new_builtin(function_prototype, Builtin::JsonParse, span)?;
        self.define_builtin_property(&object, "parse", Value::Object(parse.clone()), true, span)?;
        self.object_work(span, |objects, budget| {
            objects.define(
                &object,
                WellKnownSymbol::ToStringTag.symbol(),
                DataDescriptor {
                    value: Some(Value::String(JsString::from("JSON"))),
                    writable: Some(false),
                    enumerable: Some(false),
                    configurable: Some(true),
                },
                budget,
            )
        })?;
        Ok(JsonIntrinsics { object, parse })
    }

    pub(super) fn json_parse(
        &mut self,
        text: Value,
        reviver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let text = self.string(text, span)?;
        let mut abort = None;
        let result = parse_json_with_work(&text, |work| {
            match self.object_work(span, |_, budget| budget.charge(work)) {
                Ok(()) => true,
                Err(error) => {
                    abort = Some(error);
                    false
                }
            }
        });
        if let Some(error) = abort {
            return Err(error);
        }
        let document = result.map_err(|error| {
            if error.limit {
                Error::Limit {
                    span,
                    message: error.to_string(),
                }
            } else {
                Self::exception(ExceptionKind::SyntaxError, span, error.to_string())
            }
        })?;
        let mut values: Vec<Value> = Vec::new();
        for node in &document.nodes {
            self.tick(span)?;
            let value = match &node.kind {
                JsonKind::Null => Value::Null,
                JsonKind::Boolean(value) => Value::Boolean(*value),
                JsonKind::Number(value) => Value::Number(*value),
                JsonKind::String(value) => {
                    self.check_json_string(value, span)?;
                    Value::String(value.clone())
                }
                JsonKind::Array(elements) => {
                    let array = self.create_intrinsic_array(0, span)?;
                    for (index, child) in elements.iter().enumerate() {
                        self.tick(span)?;
                        let value = values[*child].clone();
                        self.create_array_element(&array, index as u64, value, span)?;
                    }
                    Value::Object(array)
                }
                JsonKind::Object(entries) => {
                    let prototype = self.ensure_object_intrinsics(span)?;
                    let object =
                        self.object_work(span, |objects, _| objects.create(Some(&prototype)))?;
                    for (key, child) in entries {
                        self.tick(span)?;
                        self.check_json_string(key, span)?;
                        let value = values[*child].clone();
                        // Create own data properties, including __proto__, without
                        // inherited setters. Later duplicates replace earlier values.
                        self.define_property_or_throw(
                            &object,
                            key.clone(),
                            DataDescriptor {
                                value: Some(value),
                                writable: Some(true),
                                enumerable: Some(true),
                                configurable: Some(true),
                            }
                            .into(),
                            span,
                        )?;
                    }
                    Value::Object(object)
                }
            };
            values.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "JSON value tree exceeds platform capacity".into(),
            })?;
            values.push(value);
        }
        let value = values[document.root].clone();
        if !self.is_callable(&reviver, span)? {
            return Ok(value);
        }
        self.json_revive(&text, &document, &values, reviver, span)
    }

    fn check_json_string(&self, string: &JsString, span: Span) -> Result<(), Error> {
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| string.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "JSON string or property name exceeds host string limit".into(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_decoded_strings_and_keys_respect_opted_in_string_quotas() {
        let mut realm = Realm::new(crate::Limits {
            max_string_units: Some(1),
            ..crate::Limits::default()
        });
        assert_eq!(
            realm.json_parse(
                Value::String(JsString::from(r#""\u0061""#)),
                Value::Undefined,
                Span::new(0, 0)
            ),
            Ok(Value::String(JsString::from("a")))
        );
        for source in [r#""ab""#, r#"{"ab":1}"#] {
            assert!(matches!(
                realm.json_parse(
                    Value::String(JsString::from(source)),
                    Value::Undefined,
                    Span::new(0, 0)
                ),
                Err(Error::Limit { .. })
            ));
        }
    }
}
