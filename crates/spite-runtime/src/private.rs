//! Private names, ResolvePrivateIdentifier, PrivateGet/Set, and private-in (6.2.10/9.2/13.10).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, JsSymbol, Span};
use std::collections::BTreeSet;

// Reuse opaque allocation identity without exposing a Symbol or property key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PrivateName(JsSymbol);

impl PrivateName {
    pub(crate) fn new(description: &str) -> Self {
        Self(JsSymbol::new(Some(JsString::from(description))))
    }

    pub(crate) fn description(&self) -> &JsString {
        self.0.description().expect("private name description")
    }
}

impl Realm {
    pub(crate) fn resolve_private_name(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<PrivateName, Error> {
        let mut next = self.scopes.last().cloned();
        while let Some(environment) = next {
            self.object_work(span, |_, budget| budget.charge(name.len() + 1))?;
            let record = self
                .objects
                .environment(&environment)
                .expect("active environment");
            if let Some(name) = record.private_names.get(name) {
                return Ok(name.clone());
            }
            next = record.outer.clone();
        }
        unreachable!("parser validates private lexical scope")
    }

    pub(super) fn eval_private_names(&mut self, span: Span) -> Result<BTreeSet<String>, Error> {
        let mut names = BTreeSet::new();
        let mut next = self.scopes.last().cloned();
        while let Some(environment) = next {
            next = self.object_work(span, |objects, budget| {
                budget.charge(1)?;
                let record = objects.environment(&environment)?;
                for name in record.private_names.keys() {
                    budget.charge(name.len() + 1)?;
                    names.insert(name.clone());
                }
                Ok(record.outer.clone())
            })?;
        }
        Ok(names)
    }

    pub(super) fn private_get(
        &mut self,
        base: &Value,
        name: &PrivateName,
        span: Span,
    ) -> Result<Value, Error> {
        // GetValue permits omitting an unobservable fresh primitive wrapper
        // (6.2.5.5). Such a wrapper cannot contain a private element.
        if let Value::Object(object) = base {
            if let Some(value) = self.object_work(span, |objects, budget| {
                objects.private_field_get(object, name, budget)
            })? {
                return Ok(value);
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "object lacks private field",
        ))
    }

    pub(super) fn private_set(
        &mut self,
        base: &Value,
        name: &PrivateName,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        if let Value::Object(object) = base {
            if self.object_work(span, |objects, budget| {
                objects.private_field_set(object, name, value, budget)
            })? {
                return Ok(());
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "object lacks private field",
        ))
    }

    pub(super) fn private_in(
        &mut self,
        name: &str,
        value: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(object) = value else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "private-in requires an object",
            ));
        };
        let name = self.resolve_private_name(name, span)?;
        self.object_work(span, |objects, budget| {
            objects
                .private_field_has(&object, &name, budget)
                .map(Value::Boolean)
        })
    }
}
