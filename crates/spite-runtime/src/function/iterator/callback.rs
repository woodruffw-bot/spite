//! Lazy map/filter closures with mathematical indices (27.1.3.3.4/8).

use super::operations::IteratorRecord;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{CallbackIterator, CallbackKind, IteratorWrapper},
};
use spite_bigint::{Budget, Error as IntegerError};
use spite_core::Span;

impl Realm {
    pub(crate) fn iterator_callback_helper(
        &mut self,
        receiver: Value,
        callback: Value,
        filter: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(iterator) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator callback helper requires an object receiver",
            ));
        };
        if !self.is_callable(&callback, span)? {
            let record = IteratorRecord::uninitialized(iterator);
            let error = Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator callback helper requires a callable callback",
            );
            return Err(self.iterator_close_error(&record, error, span));
        }
        let Value::Object(callback) = callback else {
            unreachable!("callable object")
        };
        let record = self.get_iterator_direct(iterator, span)?;
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .iterator
            .helper_prototype
            .clone();
        self.object_work(span, |objects, budget| {
            objects.create_callback_helper(
                &prototype,
                IteratorWrapper {
                    iterator: record.iterator,
                    next: record.next,
                },
                callback,
                if filter {
                    CallbackKind::Filter
                } else {
                    CallbackKind::Map
                },
                budget,
            )
        })
        .map(Value::Object)
    }

    pub(super) fn iterator_callback_step(
        &mut self,
        helper: &ObjectHandle,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        loop {
            // The increment follows the previous Yield, or a rejected filter
            // value. Return resumed at Yield bypasses this increment entirely.
            self.iterator_callback_counter_work(helper, span, |state, budget| {
                if state.advance {
                    state.counter.advance(budget)?;
                    state.advance = false;
                }
                Ok(())
            })?;
            let (iterator, next, callback, kind) = self.object_work(span, |objects, budget| {
                let state = objects
                    .inspect(helper)?
                    .iterator_helper()
                    .expect("helper")
                    .callback()
                    .expect("active callback helper");
                budget.value(&state.iterated.next)?;
                Ok((
                    state.iterated.iterator.clone(),
                    state.iterated.next.clone(),
                    state.callback.clone(),
                    state.kind,
                ))
            })?;
            let Some(value) = self.iterator_step_value_direct(iterator.clone(), next, span)? else {
                return Ok(None);
            };
            let index = self.iterator_callback_counter_work(helper, span, |state, budget| {
                state.counter.number(budget)
            })?;
            let retained = if kind == CallbackKind::Filter {
                self.object_work(span, |_, budget| budget.value(&value))?;
                Some(value.clone())
            } else {
                None
            };
            let result = self.call(
                Value::Object(callback),
                Value::Undefined,
                vec![value, Value::Number(index)],
                span,
            );
            let result = match result {
                Ok(value) => value,
                Err(error) => {
                    return Err(self.iterator_close_error(
                        &IteratorRecord::uninitialized(iterator),
                        error,
                        span,
                    ));
                }
            };
            self.object_work(span, |objects, _| {
                objects.callback_mut(helper)?.advance = true;
                Ok(())
            })?;
            match kind {
                CallbackKind::Map => return Ok(Some(result)),
                CallbackKind::Filter if result.to_boolean() => return Ok(retained),
                CallbackKind::Filter => {}
            }
        }
    }

    fn iterator_callback_counter_work<T>(
        &mut self,
        helper: &ObjectHandle,
        span: Span,
        work: impl FnOnce(&mut CallbackIterator, &mut Budget) -> Result<T, IntegerError>,
    ) -> Result<T, Error> {
        self.tick(span)?;
        // Native state stays traced in the heap while executing. JavaScript
        // cannot modify its brand/captures, and collection only occurs between
        // host operations, so this executing helper remains valid here.
        let state = self
            .objects
            .callback_mut(helper)
            .expect("validated executing callback helper");
        let mut budget = Budget::with_limits(None, self.remaining_steps);
        let result = work(state, &mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Self::integer_error(error, span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, iterator_count::Counter};

    #[test]
    fn suspended_callback_indices_remain_exact_beyond_u64_without_bigint_value_quota() {
        for kind in ["map", "filter"] {
            let mut realm = Realm::new(Limits {
                max_bigint_bits: Some(0),
                ..Limits::default()
            });
            let Value::Object(helper) = realm.eval(&format!("let seen=[],h=[7,7,7,7].values().{kind}((value,index)=>{{seen.push(index);return true;}});h")).unwrap() else {panic!("helper")};
            let span = Span::new(0, 0);
            realm
                .object_work(span, |objects, budget| {
                    objects.begin_iterator_helper(&helper, budget)?;
                    objects.callback_mut(&helper)?.counter = Counter::Small(u64::MAX - 1);
                    Ok(())
                })
                .unwrap();
            for _ in 0..4 {
                assert!(
                    realm
                        .iterator_callback_step(&helper, span)
                        .unwrap()
                        .is_some()
                );
            }
            assert_eq!(realm.iterator_callback_step(&helper, span), Ok(None));
            realm
                .objects
                .finish_iterator_helper(&helper, false)
                .unwrap();
            assert_eq!(
                realm.eval("seen.length===4 && seen.every(value=>value===18446744073709551616)"),
                Ok(Value::Boolean(true))
            );
        }
    }
}
