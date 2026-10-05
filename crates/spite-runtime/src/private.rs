//! Private names, ResolvePrivateIdentifier, PrivateGet/Set, and private-in (6.2.10/9.2/13.10).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{GetAction, SetAction},
};
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

#[derive(Clone, Debug)]
pub(crate) enum PrivateMethodKind {
    Method(ObjectHandle),
    Accessor {
        get: Option<ObjectHandle>,
        set: Option<ObjectHandle>,
    },
}

impl PrivateMethodKind {
    pub(crate) fn trace(&self) -> impl Iterator<Item = Option<&ObjectHandle>> {
        let edges = match self {
            Self::Method(method) => [Some(method), None],
            Self::Accessor { get, set } => [get.as_ref(), set.as_ref()],
        };
        edges.into_iter()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PrivateMethod {
    pub name: PrivateName,
    pub kind: PrivateMethodKind,
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
            if let Some(action) = self.object_work(span, |objects, budget| {
                objects.private_get(object, name, budget)
            })? {
                return match action {
                    GetAction::Value(value) => Ok(value),
                    GetAction::Call(getter) => {
                        self.call(Value::Object(getter), base.clone(), Vec::new(), span)
                    }
                };
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "object lacks private element or getter",
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
            let action = self.object_work(span, |objects, budget| {
                objects.private_set(object, name, &value, budget)
            })?;
            match action {
                SetAction::Done(true) => return Ok(()),
                SetAction::Call(setter) => {
                    self.call(Value::Object(setter), base.clone(), vec![value], span)?;
                    return Ok(());
                }
                SetAction::Done(false) => {}
                SetAction::ArrayLength(_) => {
                    unreachable!("private writes do not target array length")
                }
            }
        }
        Err(Self::exception(
            ExceptionKind::TypeError,
            span,
            "private element is absent or cannot be written",
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
                .private_has(&object, &name, budget)
                .map(Value::Boolean)
        })
    }
}
