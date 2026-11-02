//! Live Map cursors, forEach, and MapIterator.next (24.1.3.5, 24.1.5).

use super::*;
use crate::object::ArrayIterationKind;

impl Realm {
    pub(crate) fn map_for_each(
        &mut self,
        receiver: Value,
        callback: Value,
        this_arg: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let map = self.map_receiver(receiver, span)?;
        if !self.is_callable(&callback, span)? {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Map callback is not callable",
            ));
        }
        let mut index = 0;
        loop {
            self.tick(span)?;
            let Some((next, key, value)) = self.object_work(span, |objects, budget| {
                objects.map_next(&map, index, budget)
            })?
            else {
                return Ok(Value::Undefined);
            };
            index = next;
            self.call(
                callback.clone(),
                this_arg.clone(),
                vec![value, key, Value::Object(map.clone())],
                span,
            )?;
        }
    }

    pub(crate) fn map_iterator(
        &mut self,
        receiver: Value,
        kind: ArrayIterationKind,
        span: Span,
    ) -> Result<Value, Error> {
        let map = self.map_receiver(receiver, span)?;
        let prototype = self
            .record
            .intrinsics
            .as_ref()
            .expect("initialized")
            .map
            .iterator_prototype
            .clone();
        self.object_work(span, |objects, _| {
            objects.create_map_iterator(&prototype, &map, kind)
        })
        .map(Value::Object)
    }

    pub(crate) fn map_iterator_next(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not a Map Iterator",
            )
        };
        let Value::Object(iterator) = receiver else {
            return Err(brand_error());
        };
        let state = self
            .object_work(span, |objects, _| {
                Ok(objects.inspect(&iterator)?.map_iterator().cloned())
            })?
            .ok_or_else(brand_error)?;
        let Some(map) = state.map else {
            return self.iterator_result(Value::Undefined, true, span);
        };
        let owner = state.realm;
        let execute = |realm: &mut Self| {
            let next = realm.object_work(span, |objects, budget| {
                objects.map_next(&map, state.next_index, budget)
            })?;
            let Some((index, key, value)) = next else {
                realm.object_work(span, |objects, _| {
                    objects.update_map_iterator(&iterator, None)
                })?;
                return Ok(None);
            };
            realm.object_work(span, |objects, _| {
                objects.update_map_iterator(&iterator, Some(index))
            })?;
            let result = match state.kind {
                ArrayIterationKind::Key => key,
                ArrayIterationKind::Value => value,
                ArrayIterationKind::KeyValue => realm.create_array_from_list([key, value], span)?,
            };
            realm.iterator_result(result, false, span).map(Some)
        };
        let yielded = match owner {
            Some(owner) => self.with_function_realm(&owner, span, execute),
            None => execute(self),
        }?;
        // GeneratorStart restores the native method context before allocating
        // the final result; yields were allocated in the closure's Realm.
        match yielded {
            Some(result) => Ok(result),
            None => self.iterator_result(Value::Undefined, true, span),
        }
    }
}
