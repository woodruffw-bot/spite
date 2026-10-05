//! WithStatement and its Object Environment Record (14.11, 9.1.1.2).

use crate::{Completion, Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};
use spite_parser::ast::{Expr, Statement};

impl Realm {
    pub(super) fn with_statement(
        &mut self,
        expression: &Expr,
        body: &Statement,
        span: Span,
    ) -> Result<Completion, Error> {
        let value = self.expression(expression)?;
        let Value::Object(object) = self.box_primitive(value, expression.span)? else {
            unreachable!("ToObject returns an object");
        };
        let outer = self.scopes.last().cloned();
        let environment = self.object_work(span, |objects, budget| {
            objects.create_with_environment(outer, object, budget)
        })?;
        self.scopes.push(environment);
        // 14.11.2 restores the old environment on every completion, including
        // host failures that bypass JavaScript catch/finally.
        let result = self.statement(body);
        self.scopes.pop();
        result.map(|completion| completion.update_empty(Some(Value::Undefined)))
    }

    pub(super) fn with_has_binding(
        &mut self,
        object: &ObjectHandle,
        name: &str,
        span: Span,
    ) -> Result<bool, Error> {
        // 9.1.1.2.1: absent properties do not consult unscopables. Ordinary
        // property reads retain the binding object / unscopables receiver.
        if !self.has_property(object, &JsString::from(name), span)? {
            return Ok(false);
        }
        let unscopables =
            self.get_property(object, &WellKnownSymbol::Unscopables.symbol(), span)?;
        if let Value::Object(unscopables) = unscopables {
            if self
                .get_property(&unscopables, &JsString::from(name), span)?
                .to_boolean()
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn with_get_binding(
        &mut self,
        object: &ObjectHandle,
        name: &str,
        span: Span,
    ) -> Result<Value, Error> {
        // 9.1.1.2.6 rechecks HasProperty after resolving the reference: a
        // getter for unscopables can have removed the binding in the meantime.
        let key = JsString::from(name);
        if !self.has_property(object, &key, span)? {
            if self.strict {
                return Err(Self::exception(
                    ExceptionKind::ReferenceError,
                    span,
                    format!("{name} is not defined"),
                ));
            }
            return Ok(Value::Undefined);
        }
        self.get_property(object, &key, span)
    }

    pub(super) fn with_set_binding(
        &mut self,
        object: &ObjectHandle,
        name: &str,
        value: Value,
        span: Span,
    ) -> Result<(), Error> {
        // 9.1.1.2.5: a strict write to a disappeared binding is ReferenceError;
        // an existing binding that rejects Set is TypeError.
        let key = JsString::from(name);
        let exists = self.has_property(object, &key, span)?;
        if !exists && self.strict {
            return Err(Self::exception(
                ExceptionKind::ReferenceError,
                span,
                format!("{name} is not defined"),
            ));
        }
        let written = self.set_property_value(&Value::Object(object.clone()), key, value, span)?;
        if !written && self.strict {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "property is not writable",
            ));
        }
        Ok(())
    }
}
