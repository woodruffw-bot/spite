//! Lazy exact-count take/drop closures (27.1.3.3.2/11).

use super::operations::IteratorRecord;
use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    iterator_count::Counter,
    object::{IteratorWrapper, LimitKind},
};
use spite_bigint::Budget;
use spite_core::Span;

impl Realm {
    pub(crate) fn iterator_limit_helper(
        &mut self,
        receiver: Value,
        limit: Value,
        take: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let Value::Object(iterator) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator limit helper requires an object receiver",
            ));
        };
        let uninitialized = IteratorRecord::uninitialized(iterator);
        let number = match self.number(limit, span) {
            Ok(value) => value,
            Err(error) => return Err(self.iterator_close_error(&uninitialized, error, span)),
        };
        let integer = number.trunc();
        // The published edition-17 algorithm rejects NaN and negative integers;
        // it has no safe-integer cap. Negative fractions truncate to signed zero.
        if number.is_nan() || integer < 0.0 {
            let error = Self::exception(
                ExceptionKind::RangeError,
                span,
                "Iterator count must be a nonnegative integer or Infinity",
            );
            return Err(self.iterator_close_error(&uninitialized, error, span));
        }
        let record = self.get_iterator_direct(uninitialized.iterator, span)?;
        let remaining = if integer.is_infinite() {
            None
        } else {
            Some(self.iterator_counter_work(span, |budget| Counter::from_number(integer, budget))?)
        };
        let prototype = self
            .intrinsics
            .as_ref()
            .expect("initialized")
            .iterator
            .helper_prototype
            .clone();
        self.object_work(span, |objects, budget| {
            objects.create_limit_helper(
                &prototype,
                IteratorWrapper {
                    iterator: record.iterator,
                    next: record.next,
                },
                remaining,
                if take {
                    LimitKind::Take
                } else {
                    LimitKind::Drop
                },
                budget,
            )
        })
        .map(Value::Object)
    }

    pub(super) fn iterator_limit_step(
        &mut self,
        helper: &ObjectHandle,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        loop {
            let (iterator, next, kind) = self.object_work(span, |objects, budget| {
                let state = objects
                    .inspect(helper)?
                    .iterator_helper()
                    .expect("helper")
                    .limit()
                    .expect("active limit helper");
                budget.value(&state.iterated.next)?;
                Ok((
                    state.iterated.iterator.clone(),
                    state.iterated.next.clone(),
                    state.kind,
                ))
            })?;
            let remaining = self.iterator_limit_decrement(helper, span)?;
            if kind == LimitKind::Take {
                if !remaining {
                    self.iterator_close_direct(iterator, span)?;
                    return Ok(None);
                }
                return self.iterator_step_value_direct(iterator, next, span);
            }
            if !remaining {
                return self.iterator_step_value_direct(iterator, next, span);
            }
            // IteratorStep deliberately reads done only. A discarded value
            // getter must never run, including when dropping an infinite count.
            if self.iterator_step_direct(iterator, next, span)?.is_none() {
                return Ok(None);
            }
        }
    }

    fn iterator_limit_decrement(
        &mut self,
        helper: &ObjectHandle,
        span: Span,
    ) -> Result<bool, Error> {
        self.tick(span)?;
        let state = self
            .objects
            .limit_mut(helper)
            .expect("validated executing limit helper");
        let mut budget = Budget::with_limits(None, self.remaining_steps);
        let result = if let Some(remaining) = &mut state.remaining {
            if remaining.is_zero() {
                Ok(false)
            } else {
                remaining.decrement(&mut budget).map(|()| true)
            }
        } else {
            Ok(true)
        };
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Self::integer_error(error, span))
    }
}
