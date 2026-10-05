//! Synchronous iterator acquisition, stepping, and completion closing (7.4).

use crate::{Error, ExceptionKind, ObjectHandle, Realm, Value};
use spite_core::{JsString, PropertyKeyRef, Span, WellKnownSymbol};

pub(crate) struct IteratorRecord {
    pub(super) iterator: ObjectHandle,
    pub(super) next: Value,
    done: bool,
}

impl IteratorRecord {
    pub(crate) fn is_done(&self) -> bool {
        self.done
    }

    pub(super) fn uninitialized(iterator: ObjectHandle) -> Self {
        Self {
            iterator,
            next: Value::Undefined,
            done: false,
        }
    }
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
        self.get_iterator_direct(iterator, span)
    }

    /// GetIteratorFlattenable (7.4): reject disallowed primitives before lookup.
    pub(super) fn get_iterator_flattenable(
        &mut self,
        value: Value,
        iterate_string_primitives: bool,
        span: Span,
    ) -> Result<IteratorRecord, Error> {
        if !matches!(&value, Value::Object(_))
            && !(iterate_string_primitives && matches!(&value, Value::String(_)))
        {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "value cannot be flattened as an iterator",
            ));
        }
        let iterator = if let Some(method) =
            self.get_method(&value, &WellKnownSymbol::Iterator.symbol(), span)?
        {
            self.call(method, value, vec![], span)?
        } else {
            value
        };
        let Value::Object(iterator) = iterator else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "iterator is not an object",
            ));
        };
        self.get_iterator_direct(iterator, span)
    }

    /// GetIteratorDirect (sec-getiteratordirect) captures next without checking callability.
    pub(crate) fn get_iterator_direct(
        &mut self,
        iterator: ObjectHandle,
        span: Span,
    ) -> Result<IteratorRecord, Error> {
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
            self.iterator_step_value_direct(record.iterator.clone(), record.next.clone(), span)
        })();
        if !matches!(&result, Ok(Some(_))) {
            record.done = true;
        }
        result
    }

    /// IteratorStep for a binding elision: done is read but value is skipped.
    pub(crate) fn iterator_skip_value(
        &mut self,
        record: &mut IteratorRecord,
        span: Span,
    ) -> Result<(), Error> {
        debug_assert!(!record.done);
        let result = (|| {
            self.object_work(span, |_, budget| budget.value(&record.next))?;
            self.iterator_step_direct(record.iterator.clone(), record.next.clone(), span)
        })();
        if !matches!(&result, Ok(Some(_))) {
            record.done = true;
        }
        result.map(|_| ())
    }

    pub(crate) fn iterator_step_value_direct(
        &mut self,
        iterator: ObjectHandle,
        next: Value,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        let Some(result) = self.iterator_step_direct(iterator, next, span)? else {
            return Ok(None);
        };
        self.get_property(&result, &JsString::from("value"), span)
            .map(Some)
    }

    pub(crate) fn iterator_step_direct(
        &mut self,
        iterator: ObjectHandle,
        next: Value,
        span: Span,
    ) -> Result<Option<ObjectHandle>, Error> {
        let Value::Object(result) = self.call(next, Value::Object(iterator), vec![], span)? else {
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
        Ok(Some(result))
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

    /// IteratorClose for a non-throw completion: cleanup errors replace it.
    pub(crate) fn iterator_close(
        &mut self,
        record: &IteratorRecord,
        span: Span,
    ) -> Result<(), Error> {
        self.iterator_close_direct(record.iterator.clone(), span)
    }

    pub(crate) fn iterator_close_direct(
        &mut self,
        iterator: ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        let receiver = Value::Object(iterator);
        if let Some(method) = self.get_method(&receiver, &JsString::from("return"), span)? {
            let result = self.call(method, receiver, vec![], span)?;
            if !matches!(result, Value::Object(_)) {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "iterator return result is not an object",
                ));
            }
        }
        Ok(())
    }
}
