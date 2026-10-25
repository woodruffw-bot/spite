//! Promise.race and ordered iterable subscription (27.2.4.5).

use crate::{Error, ExceptionKind, Realm, Value};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(in crate::function) fn promise_race(
        &mut self,
        constructor: Value,
        iterable: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // Capability construction throws directly. Every later language failure
        // rejects that capability; host failures bypass JavaScript callbacks.
        let capability = self.new_promise_capability(constructor.clone(), span)?;
        let result = (|| -> Result<(), Error> {
            let resolve =
                self.get_property_value(&constructor, &JsString::from("resolve"), span)?;
            if !self.is_callable(&resolve, span)? {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Promise constructor resolve is not callable",
                ));
            }
            let method = self
                .get_method(&iterable, &WellKnownSymbol::Iterator.symbol(), span)?
                .ok_or_else(|| {
                    Self::exception(
                        ExceptionKind::TypeError,
                        span,
                        "Promise.race input is not iterable",
                    )
                })?;
            let mut iterator = self.get_iterator_from_method(iterable, method, span)?;
            loop {
                self.tick(span)?;
                // IteratorStepValue marks stepping/done/value errors as done.
                // Only failures after a value is yielded close the iterator.
                let Some(value) = self.iterator_step_value(&mut iterator, span)? else {
                    return Ok(());
                };
                let subscribe = (|| {
                    let promise =
                        self.call(resolve.clone(), constructor.clone(), vec![value], span)?;
                    let then = self.get_property_value(&promise, &JsString::from("then"), span)?;
                    self.call(
                        then,
                        promise,
                        vec![capability.resolve.clone(), capability.reject.clone()],
                        span,
                    )
                })();
                if let Err(error) = subscribe {
                    return Err(self.iterator_close_error(&iterator, error, span));
                }
            }
        })();
        if let Err(error) = result {
            let reason = self.language_exception_value(error)?;
            self.call(capability.reject, Value::Undefined, vec![reason], span)?;
        }
        Ok(capability.promise)
    }
}
