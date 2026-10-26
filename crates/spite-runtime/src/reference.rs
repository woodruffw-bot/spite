//! References share ordinary GetValue/PutValue and can own names through suspension.

use crate::{Error, ObjectHandle, Value, environment::EnvironmentHandle, private};
use spite_core::{JsString, JsSymbol, Span};
use std::borrow::Cow;

// Resolve references before evaluating assignment RHS expressions. GetValue and
// PutValue are separate operations, including the unresolvable typeof case.
#[derive(Debug)]
pub(super) enum Reference<'a> {
    Value(Value),
    Lexical(EnvironmentHandle, Cow<'a, str>),
    ObjectBinding {
        object: ObjectHandle,
        name: Cow<'a, str>,
    },
    Global {
        name: Cow<'a, str>,
        key: JsString,
    },
    Unresolvable(Cow<'a, str>),
    UnsupportedGlobal(Cow<'a, str>),
    Property {
        base: Value,
        // Edition 17 converts a computed name at GetValue/PutValue, not when
        // creating the reference. GetValue caches the converted property key.
        key: Value,
    },
    Private {
        base: Value,
        name: private::PrivateName,
    },
    SuperProperty {
        base: Value,
        key: Value,
        this_value: Value,
    },
}

impl Reference<'_> {
    pub(super) fn is_eval_environment_reference(&self) -> bool {
        match self {
            Self::Lexical(_, name)
            | Self::ObjectBinding { name, .. }
            | Self::Global { name, .. } => name.as_ref() == "eval",
            _ => false,
        }
    }
    // ECMA-262 13.3.6.2 / 9.1.1.2.10: environment calls use WithBaseObject.
    pub(super) fn call_receiver(self) -> Value {
        match self {
            Self::Property { base, .. } | Self::Private { base, .. } => base,
            Self::SuperProperty { this_value, .. } => this_value,
            Self::ObjectBinding { object, .. } => Value::Object(object),
            _ => Value::Undefined,
        }
    }
}

impl Reference<'_> {
    pub(super) fn into_owned(self, span: Span) -> Result<Reference<'static>, Error> {
        Ok(match self {
            Self::Value(value) => Reference::Value(value),
            Self::Lexical(environment, name) => {
                Reference::Lexical(environment, own_name(name, span)?)
            }
            Self::ObjectBinding { object, name } => Reference::ObjectBinding {
                object,
                name: own_name(name, span)?,
            },
            Self::Global { name, key } => Reference::Global {
                name: own_name(name, span)?,
                key,
            },
            Self::Unresolvable(name) => Reference::Unresolvable(own_name(name, span)?),
            Self::UnsupportedGlobal(name) => Reference::UnsupportedGlobal(own_name(name, span)?),
            Self::Property { base, key } => Reference::Property { base, key },
            Self::Private { base, name } => Reference::Private { base, name },
            Self::SuperProperty {
                base,
                key,
                this_value,
            } => Reference::SuperProperty {
                base,
                key,
                this_value,
            },
        })
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &Value> {
        let values = match self {
            Self::Value(value) => [Some(value), None, None],
            Self::Property { base, key } => [Some(base), Some(key), None],
            Self::Private { base, .. } => [Some(base), None, None],
            Self::SuperProperty {
                base,
                key,
                this_value,
            } => [Some(base), Some(key), Some(this_value)],
            _ => [None, None, None],
        };
        values.into_iter().flatten()
    }

    pub(crate) fn environment(&self) -> Option<&EnvironmentHandle> {
        match self {
            Self::Lexical(environment, _) => Some(environment),
            _ => None,
        }
    }

    pub(crate) fn symbols(&self) -> impl Iterator<Item = &JsSymbol> {
        match self {
            Self::Private { name, .. } => Some(name.symbol()),
            _ => None,
        }
        .into_iter()
    }

    pub(crate) fn object(&self) -> Option<&ObjectHandle> {
        match self {
            Self::ObjectBinding { object, .. } => Some(object),
            _ => None,
        }
    }
}

fn own_name(name: Cow<'_, str>, span: Span) -> Result<Cow<'static, str>, Error> {
    match name {
        Cow::Owned(name) => Ok(Cow::Owned(name)),
        Cow::Borrowed(name) => {
            let mut owned = String::new();
            owned
                .try_reserve_exact(name.len())
                .map_err(|_| Error::Limit {
                    span,
                    message: "async reference name allocation failed".into(),
                })?;
            owned.push_str(name);
            Ok(Cow::Owned(owned))
        }
    }
}
