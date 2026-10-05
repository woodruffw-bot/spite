//! Set live iteration and exact callback arguments (24.2.4.7, 24.2.6).

use super::*;
use crate::object::ArrayIterationKind;

impl Realm {
    pub(crate) fn set_for_each(
        &mut self,
        receiver: Value,
        callback: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let set = self.set_receiver(receiver, span)?;
        if !self.is_callable(&callback, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Set callback is not callable",
            ));
        }
        let mut index = 0;
        loop {
            self.tick(span)?;
            let Some((next, value)) = self.object_work(span, |objects, budget| {
                objects.set_data(&set)?.next(index, budget)
            })?
            else {
                return Ok(Value::Undefined);
            };
            index = next;
            self.call(
                callback.clone(),
                this_arg.clone(),
                vec![value.clone(), value, Value::Object(set.clone())],
                span,
            )?;
        }
    }

    pub(crate) fn set_iterator(
        &mut self,
        receiver: Value,
        kind: ArrayIterationKind,
        span: Span,
    ) -> Result<Value, Error> {
        let set = self.set_receiver(receiver, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .set
            .iterator_prototype
            .clone();
        self.object_work(span, |objects, _| {
            objects.create_set_iterator(&prototype, &set, kind)
        })
        .map(Value::Object)
    }

    pub(crate) fn set_iterator_next(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a Set Iterator",
            )
        };
        let Value::Object(iterator) = receiver else {
            return Err(brand_error());
        };
        let state = self
            .object_work(span, |objects, _| {
                Ok(objects.inspect(&iterator)?.set_iterator().cloned())
            })?
            .ok_or_else(brand_error)?;
        let Some(set) = state.set else {
            return self.iterator_result(Value::Undefined, true, span);
        };
        let next = self.object_work(span, |objects, budget| {
            objects.set_data(&set)?.next(state.next_index, budget)
        })?;
        let Some((index, value)) = next else {
            self.object_work(span, |objects, _| {
                objects.update_set_iterator(&iterator, None)
            })?;
            return self.iterator_result(Value::Undefined, true, span);
        };
        self.object_work(span, |objects, _| {
            objects.update_set_iterator(&iterator, Some(index))
        })?;
        let value = if state.kind == ArrayIterationKind::KeyValue {
            self.create_array_from_list([value.clone(), value], span)?
        } else {
            value
        };
        self.iterator_result(value, false, span)
    }
}
