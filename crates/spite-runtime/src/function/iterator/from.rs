//! Iterator.from and %WrapForValidIteratorPrototype% (27.1.3.2.2).

use crate::{Error, ExceptionKind, Realm, Value, object::IteratorWrapper};
use spite_core::{JsString, Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn iterator_from(&mut self, value: Value, span: Span) -> Result<Value, Error> {
        // GetIteratorFlattenable (sec-getiteratorflattenable), iterate-string-primitives mode:
        // reject every other primitive before any property lookup or coercion.
        if !matches!(&value, Value::Object(_) | Value::String(_)) {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator.from requires an object or String",
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
        let record = self.get_iterator_direct(iterator, span)?;
        let intrinsics = &self.intrinsics.as_ref().expect("initialized").iterator;
        let constructor = intrinsics.constructor.clone();
        let prototype = intrinsics.wrapper_prototype.clone();
        // Use OrdinaryHasInstance on the intrinsic, never an observable custom
        // @@hasInstance hook or a replaced public Iterator binding.
        if self.ordinary_has_instance(
            Value::Object(constructor),
            Value::Object(record.iterator.clone()),
            span,
        )? {
            return Ok(Value::Object(record.iterator));
        }
        self.object_work(span, |objects, _| {
            objects.create_iterator_wrapper(
                &prototype,
                IteratorWrapper {
                    iterator: record.iterator,
                    next: record.next,
                },
            )
        })
        .map(Value::Object)
    }

    pub(crate) fn iterator_wrapper_method(
        &mut self,
        receiver: Value,
        return_method: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not an Iterator.from wrapper",
            )
        };
        let Value::Object(wrapper) = receiver else {
            return Err(brand_error());
        };
        let (iterator, next) = self
            .object_work(span, |objects, budget| {
                let Some(state) = objects.inspect(&wrapper)?.iterator_wrapper() else {
                    return Ok(None);
                };
                let next = if return_method {
                    None
                } else {
                    budget.value(&state.next)?;
                    Some(state.next.clone())
                };
                Ok(Some((state.iterator.clone(), next)))
            })?
            .ok_or_else(brand_error)?;
        let receiver = Value::Object(iterator);
        let method = if return_method {
            let Some(method) = self.get_method(&receiver, &JsString::from("return"), span)? else {
                return self.iterator_result(Value::Undefined, true, span);
            };
            method
        } else {
            next.expect("cached next")
        };
        // These methods forward the result unchanged, including primitives,
        // without inspecting done/value or tracking generator completion.
        self.call(method, receiver, vec![], span)
    }
}
