//! String code-point iteration (22.1.3.36, 22.1.5.1, 27.5.3.2–3).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::Span;

impl Realm {
    pub(crate) fn string_iterator(&mut self, receiver: Value, span: Span) -> Result<Value, Error> {
        Self::require_object_coercible(&receiver, span)?;
        let string = self.string(receiver, span)?;
        if self
            .limits
            .max_string_units
            .is_some_and(|limit| string.len() > limit)
        {
            return Err(Error::Limit {
                span,
                message: "string length limit exceeded".into(),
            });
        }
        let prototype = self
            .record
            .intrinsics
            .as_ref()
            .expect("initialized")
            .iterator
            .string_prototype
            .clone();
        self.object_work(span, |objects, _| {
            objects.create_string_iterator(&prototype, string)
        })
        .map(Value::Object)
    }

    pub(crate) fn string_iterator_next(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a String Iterator",
            )
        };
        let Value::Object(iterator) = receiver else {
            return Err(brand_error());
        };
        let owner = self
            .object_work(span, |objects, _| {
                Ok(objects
                    .inspect(&iterator)?
                    .string_iterator()
                    .map(|state| state.realm.clone()))
            })?
            .ok_or_else(brand_error)?;
        let execute = |realm: &mut Self| {
            let result = realm.object_work(span, |objects, budget| {
                objects.next_string_iterator(&iterator, budget)
            })?;
            match result {
                Some(string) => realm
                    .iterator_result(Value::String(string), false, span)
                    .map(Some),
                None => Ok(None),
            }
        };
        let yielded = match owner {
            Some(owner) => self.with_function_realm(&owner, span, execute),
            None => execute(self),
        }?;
        match yielded {
            Some(result) => Ok(result),
            None => self.iterator_result(Value::Undefined, true, span),
        }
    }
}
