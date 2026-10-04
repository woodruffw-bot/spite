//! Standard Error objects, native constructors, and methods (20.5).

use super::Builtin;
use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value, object::DataDescriptor};
use spite_core::{JsString, Span};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ErrorConstructor {
    Error,
    EvalError,
    RangeError,
    ReferenceError,
    SyntaxError,
    TypeError,
    URIError,
}

impl ErrorConstructor {
    const ALL: [Self; 7] = [
        Self::Error,
        Self::EvalError,
        Self::RangeError,
        Self::ReferenceError,
        Self::SyntaxError,
        Self::TypeError,
        Self::URIError,
    ];

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::EvalError => "EvalError",
            Self::RangeError => "RangeError",
            Self::ReferenceError => "ReferenceError",
            Self::SyntaxError => "SyntaxError",
            Self::TypeError => "TypeError",
            Self::URIError => "URIError",
        }
    }
}

#[derive(Debug)]
pub(crate) struct ErrorIntrinsic {
    pub kind: ErrorConstructor,
    pub constructor: ObjectHandle,
    pub prototype: ObjectHandle,
}

#[derive(Debug)]
pub(crate) struct ErrorIntrinsics {
    pub entries: Vec<ErrorIntrinsic>,
    to_string: ObjectHandle,
    is_error: ObjectHandle,
}

impl ErrorIntrinsics {
    pub(super) fn roots(&self) -> impl Iterator<Item = &ObjectHandle> {
        self.entries
            .iter()
            .flat_map(|entry| [&entry.constructor, &entry.prototype])
            .chain([&self.to_string, &self.is_error])
    }

    fn get(&self, kind: ErrorConstructor) -> &ErrorIntrinsic {
        &self.entries[kind as usize]
    }
}

impl Realm {
    pub(super) fn error_intrinsics(
        &mut self,
        object_prototype: &ObjectHandle,
        function_prototype: &ObjectHandle,
        span: Span,
    ) -> Result<ErrorIntrinsics, Error> {
        let mut entries: Vec<ErrorIntrinsic> = Vec::new();
        for kind in ErrorConstructor::ALL {
            let (parent_constructor, parent_prototype) = match entries.first() {
                None => (function_prototype, object_prototype),
                Some(error) => (&error.constructor, &error.prototype),
            };
            let constructor = self.new_builtin(parent_constructor, Builtin::Error(kind), span)?;
            // Error.prototype and NativeError prototypes are ordinary objects
            // without ErrorData (20.5.3, 20.5.6.3).
            let prototype =
                self.object_work(span, |objects, _| objects.create(Some(parent_prototype)))?;
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
            for (name, value) in [
                ("constructor", Value::Object(constructor.clone())),
                ("name", Value::String(JsString::from(kind.name()))),
                ("message", Value::String(JsString::from(""))),
            ] {
                self.define_builtin_property(&prototype, name, value, true, span)?;
            }
            entries.push(ErrorIntrinsic {
                kind,
                constructor,
                prototype,
            });
        }
        let to_string = self.new_builtin(function_prototype, Builtin::ErrorToString, span)?;
        let is_error = self.new_builtin(function_prototype, Builtin::ErrorIsError, span)?;
        self.define_builtin_property(
            &entries[0].prototype,
            "toString",
            Value::Object(to_string.clone()),
            true,
            span,
        )?;
        self.define_builtin_property(
            &entries[0].constructor,
            "isError",
            Value::Object(is_error.clone()),
            true,
            span,
        )?;
        Ok(ErrorIntrinsics {
            entries,
            to_string,
            is_error,
        })
    }

    pub(super) fn error_constructor(
        &mut self,
        kind: ErrorConstructor,
        new_target: Option<ObjectHandle>,
        mut arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let intrinsic = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .errors
            .get(kind);
        let new_target = new_target.unwrap_or_else(|| intrinsic.constructor.clone());
        let fallback = intrinsic.prototype.clone();
        // 20.5.1.1 / 20.5.6.1.1: select the prototype and allocate before
        // converting message, then inspect cause only on object-valued options.
        let prototype = self.get_property(&new_target, &JsString::from("prototype"), span)?;
        let prototype = match prototype {
            Value::Object(prototype) => prototype,
            _ => fallback,
        };
        let object = self.object_work(span, |objects, _| objects.create_error(&prototype))?;
        if let Some(message) = arguments.next().filter(|v| !matches!(v, Value::Undefined)) {
            let message = Value::String(self.string(message, span)?);
            self.check_string(&message, span)?;
            self.define_builtin_property(&object, "message", message, true, span)?;
        }
        if let Some(Value::Object(options)) = arguments.next() {
            let key = JsString::from("cause");
            if self.has_property(&options, &key, span)? {
                let cause = self.get_property(&options, &key, span)?;
                self.define_builtin_property(&object, "cause", cause, true, span)?;
            }
        }
        Ok(Value::Object(object))
    }

    pub(super) fn error_is_error(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        let result = if let Value::Object(object) = value {
            self.object_work(span, |objects, _| Ok(objects.inspect(&object)?.is_error()))?
        } else {
            false
        };
        Ok(Value::Boolean(result))
    }

    pub(super) fn error_to_string(&mut self, this: Value, span: Span) -> Result<Value, Error> {
        let Value::Object(object) = this else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Error.toString requires an object",
            ));
        };
        let name = self.get_property(&object, &JsString::from("name"), span)?;
        let name = if matches!(name, Value::Undefined) {
            JsString::from("Error")
        } else {
            self.string(name, span)?
        };
        let message = self.get_property(&object, &JsString::from("message"), span)?;
        let message = if matches!(message, Value::Undefined) {
            JsString::from("")
        } else {
            self.string(message, span)?
        };
        let result = if name.is_empty() {
            message
        } else if message.is_empty() {
            name
        } else {
            let mut units = Vec::new();
            self.append_string(&mut units, &name, span)?;
            self.append_string(&mut units, &JsString::from(": "), span)?;
            self.append_string(&mut units, &message, span)?;
            self.object_work(span, |_, budget| budget.charge(units.len()))?;
            JsString::from_code_units(units)
        };
        Ok(Value::String(result))
    }
}
