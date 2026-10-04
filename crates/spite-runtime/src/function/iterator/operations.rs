//! Synchronous iterator acquisition, stepping, and closing throw completions (7.4).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, PropertyKeyRef, Span};

pub(crate) struct IteratorRecord {
    iterator: ObjectHandle,
    next: Value,
    done: bool,
}

impl Realm {
    /// GetMethod (7.3.10) preserves the original primitive/object receiver.
    pub(crate) fn get_method<'key>(
        &mut self,
        value: &Value,
        key: impl Into<PropertyKeyRef<'key>>,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        Self::require_object_coercible(value, span)?;
        let method = self.get_property_value(value, key, span)?;
        if matches!(method, Value::Undefined | Value::Null) {
            return Ok(None);
        }
        if !self.is_callable(&method, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "method is not callable",
            ));
        }
        Ok(Some(method))
    }

    pub(crate) fn get_iterator_from_method(
        &mut self,
        value: Value,
        method: Value,
        span: Span,
    ) -> Result<IteratorRecord, Error> {
        let Value::Object(iterator) = self.call(method, value, vec![], span)? else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "iterator is not an object",
            ));
        };
        // Get next exactly once. Its callable check belongs to IteratorNext.
        let next = self.get_property(&iterator, &JsString::from("next"), span)?;
        Ok(IteratorRecord {
            iterator,
            next,
            done: false,
        })
    }

    pub(crate) fn iterator_step_value(
        &mut self,
        record: &mut IteratorRecord,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        debug_assert!(!record.done);
        let result = (|| {
            self.object_work(span, |_, budget| budget.value(&record.next))?;
            let Value::Object(result) = self.call(
                record.next.clone(),
                Value::Object(record.iterator.clone()),
                vec![],
                span,
            )?
            else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "iterator result is not an object",
                ));
            };
            let done = self.get_property(&result, &JsString::from("done"), span)?;
            if done.to_boolean() {
                return Ok(None);
            }
            self.get_property(&result, &JsString::from("value"), span)
                .map(Some)
        })();
        if !matches!(&result, Ok(Some(_))) {
            record.done = true;
        }
        result
    }

    pub(crate) fn iterator_close_error(
        &mut self,
        record: &IteratorRecord,
        error: Error,
        span: Span,
    ) -> Error {
        // IteratorClose preserves an incoming throw over GetMethod/Call errors
        // and over non-object return results. Engine failures are not language
        // completions: never execute return after one, nor hide one while closing.
        if !error.is_language_exception() {
            return error;
        }
        let receiver = Value::Object(record.iterator.clone());
        let close = self
            .get_method(&receiver, &JsString::from("return"), span)
            .and_then(|method| {
                if let Some(method) = method {
                    self.call(method, receiver, vec![], span)?;
                }
                Ok(())
            });
        match close {
            Err(close_error) if !close_error.is_language_exception() => close_error,
            _ => error,
        }
    }
}
