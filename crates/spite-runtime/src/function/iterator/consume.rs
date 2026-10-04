//! Eager direct-iterator consumers (27.1.3.3).

use super::{Builtin, operations::IteratorRecord};
use crate::{Error, ExceptionKind, Realm, Value, iterator_count::Counter};
use spite_bigint::{Budget, Error as IntegerError};
use spite_core::Span;

impl Realm {
    pub(crate) fn iterator_for_each(
        &mut self,
        receiver: Value,
        procedure: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.iterator_for_each_counted(receiver, procedure, Counter::Small(0), span)
    }

    fn iterator_for_each_counted(
        &mut self,
        receiver: Value,
        procedure: Value,
        mut counter: Counter,
        span: Span,
    ) -> Result<Value, Error> {
        // 27.1.3.3.7: validate the procedure before GetIteratorDirect, closing
        // on validation failure without ever looking up next.
        let Value::Object(iterator) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator.forEach requires an object receiver",
            ));
        };
        if !self.is_callable(&procedure, span)? {
            let record = IteratorRecord::uninitialized(iterator);
            let error = Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator.forEach requires a callable procedure",
            );
            return Err(self.iterator_close_error(&record, error, span));
        }
        let mut record = self.get_iterator_direct(iterator, span)?;
        while let Some(value) = self.iterator_step_value(&mut record, span)? {
            let index = self.iterator_counter_work(span, |budget| counter.number(budget))?;
            let result = self.call(
                procedure.clone(),
                Value::Undefined,
                vec![value, Value::Number(index)],
                span,
            );
            if let Err(error) = result {
                return Err(self.iterator_close_error(&record, error, span));
            }
            // The mathematical counter has no specification size limit. Keep
            // its integer value exact before converting each callback index.
            self.iterator_counter_work(span, |budget| counter.advance(budget))?;
        }
        Ok(Value::Undefined)
    }

    pub(crate) fn iterator_predicate(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        predicate: Value,
        span: Span,
    ) -> Result<Value, Error> {
        self.iterator_predicate_counted(builtin, receiver, predicate, Counter::Small(0), span)
    }

    fn iterator_predicate_counted(
        &mut self,
        builtin: Builtin,
        receiver: Value,
        predicate: Value,
        mut counter: Counter,
        span: Span,
    ) -> Result<Value, Error> {
        // 27.1.3.3.3/5/10: validate before reading next, closing the receiver
        // on an invalid callback; acquisition/step failures do not close.
        let Value::Object(iterator) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator predicate requires an object receiver",
            ));
        };
        if !self.is_callable(&predicate, span)? {
            let record = IteratorRecord::uninitialized(iterator);
            let error = Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator predicate requires a callable predicate",
            );
            return Err(self.iterator_close_error(&record, error, span));
        }
        let mut record = self.get_iterator_direct(iterator, span)?;
        while let Some(value) = self.iterator_step_value(&mut record, span)? {
            let index = self.iterator_counter_work(span, |budget| counter.number(budget))?;
            // Find returns the original value, even if the predicate changes
            // the iterator's value source. Charge its owned copy before Call.
            let retained = if matches!(builtin, Builtin::IteratorFind) {
                self.object_work(span, |_, budget| budget.value(&value))?;
                Some(value.clone())
            } else {
                None
            };
            let result = self.call(
                predicate.clone(),
                Value::Undefined,
                vec![value, Value::Number(index)],
                span,
            );
            let result = match result {
                Ok(value) => value,
                Err(error) => return Err(self.iterator_close_error(&record, error, span)),
            };
            let matched = result.to_boolean();
            let output = match builtin {
                Builtin::IteratorEvery if !matched => Some(Value::Boolean(false)),
                Builtin::IteratorSome if matched => Some(Value::Boolean(true)),
                Builtin::IteratorFind if matched => retained,
                Builtin::IteratorEvery | Builtin::IteratorSome | Builtin::IteratorFind => None,
                _ => unreachable!("iterator predicate consumer"),
            };
            if let Some(output) = output {
                // A normal short-circuit completion is replaced by any close
                // error, unlike an incoming callback throw.
                self.iterator_close(&record, span)?;
                return Ok(output);
            }
            self.iterator_counter_work(span, |budget| counter.advance(budget))?;
        }
        Ok(match builtin {
            Builtin::IteratorEvery => Value::Boolean(true),
            Builtin::IteratorSome => Value::Boolean(false),
            Builtin::IteratorFind => Value::Undefined,
            _ => unreachable!("iterator predicate consumer"),
        })
    }

    pub(crate) fn iterator_reduce(
        &mut self,
        receiver: Value,
        reducer: Value,
        initial: Option<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        // 27.1.3.3.9: validate the reducer before next lookup, closing only
        // for callback validation/call failures. Presence of initial matters.
        let Value::Object(iterator) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator.reduce requires an object receiver",
            ));
        };
        if !self.is_callable(&reducer, span)? {
            let record = IteratorRecord::uninitialized(iterator);
            let error = Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator.reduce requires a callable reducer",
            );
            return Err(self.iterator_close_error(&record, error, span));
        }
        let mut record = self.get_iterator_direct(iterator, span)?;
        let (accumulator, counter) = if let Some(initial) = initial {
            (initial, Counter::Small(0))
        } else {
            let Some(first) = self.iterator_step_value(&mut record, span)? else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Iterator.reduce has no values or initial value",
                ));
            };
            (first, Counter::Small(1))
        };
        self.iterator_reduce_loop(record, reducer, accumulator, counter, span)
    }

    fn iterator_reduce_loop(
        &mut self,
        mut record: IteratorRecord,
        reducer: Value,
        mut accumulator: Value,
        mut counter: Counter,
        span: Span,
    ) -> Result<Value, Error> {
        while let Some(value) = self.iterator_step_value(&mut record, span)? {
            let index = self.iterator_counter_work(span, |budget| counter.number(budget))?;
            let result = self.call(
                reducer.clone(),
                Value::Undefined,
                vec![accumulator, value, Value::Number(index)],
                span,
            );
            accumulator = match result {
                Ok(value) => value,
                Err(error) => return Err(self.iterator_close_error(&record, error, span)),
            };
            self.iterator_counter_work(span, |budget| counter.advance(budget))?;
        }
        Ok(accumulator)
    }

    pub(super) fn iterator_counter_work<T>(
        &mut self,
        span: Span,
        work: impl FnOnce(&mut Budget) -> Result<T, IntegerError>,
    ) -> Result<T, Error> {
        // An internal mathematical counter produces Numbers, not BigInt values;
        // the host BigInt-value magnitude quota does not apply to this storage.
        let mut budget = Budget::with_limits(None, self.remaining_steps);
        let result = work(&mut budget);
        self.remaining_steps = budget.remaining_work();
        result.map_err(|error| Self::integer_error(error, span))
    }

    pub(crate) fn iterator_to_array(
        &mut self,
        receiver: Value,
        span: Span,
    ) -> Result<Value, Error> {
        // 27.1.3.3.12: use GetIteratorDirect, without consulting @@iterator.
        let Value::Object(iterator) = receiver else {
            return Err(Self::exception(
                ExceptionKind::TypeError,
                span,
                "Iterator.toArray requires an object receiver",
            ));
        };
        let mut record = self.get_iterator_direct(iterator, span)?;
        let mut items = Vec::new();
        while let Some(value) = self.iterator_step_value(&mut record, span)? {
            items.try_reserve(1).map_err(|_| Error::Limit {
                span,
                message: "iterator result list exceeds platform capacity".into(),
            })?;
            items.push(value);
        }
        // Materialize only after exhaustion, using intrinsic own data elements.
        // Neither iterator-step errors nor host failures run IteratorClose.
        self.create_array_from_list(items, span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Limits;
    use spite_bigint::BigInt;

    fn check_indices(counter: Counter, expected: &str) {
        let mut realm = Realm::new(Limits {
            max_bigint_bits: Some(0),
            ..Limits::default()
        });
        let receiver = realm
            .eval("let n=0,seen=[],i={next:()=>({value:7,done:n++===4})};i")
            .unwrap();
        let callback = realm.eval("(value,index)=>seen.push(index)").unwrap();
        assert_eq!(
            realm.iterator_for_each_counted(receiver, callback, counter, Span::new(0, 0)),
            Ok(Value::Undefined)
        );
        assert_eq!(realm.eval(expected), Ok(Value::Boolean(true)));
    }

    #[test]
    fn callback_indices_round_exact_counters_beyond_safe_integers_and_u64() {
        check_indices(
            Counter::Small((1u64 << 53) - 1),
            "seen[0]===9007199254740991 && seen[1]===9007199254740992 && seen[2]===9007199254740992 && seen[3]===9007199254740994",
        );
        check_indices(
            Counter::Small(u64::MAX - 1),
            "seen.every(value=>value===18446744073709551616)",
        );
        let integer = BigInt::parse_digits(
            "18446744073709553663",
            10,
            &mut Budget::with_limits(None, None),
        )
        .unwrap();
        check_indices(
            Counter::Large(integer),
            "seen[0]===18446744073709551616 && seen[1]===18446744073709551616 && seen[2]===18446744073709555712 && seen[3]===18446744073709555712",
        );
    }

    #[test]
    fn predicate_consumers_preserve_mathematical_indices_beyond_u64() {
        for builtin in [
            Builtin::IteratorEvery,
            Builtin::IteratorSome,
            Builtin::IteratorFind,
        ] {
            let mut realm = Realm::new(Limits {
                max_bigint_bits: Some(0),
                ..Limits::default()
            });
            let receiver = realm
                .eval("let n=0,seen=[],i={next:()=>({value:7,done:n++===4})};i")
                .unwrap();
            let continuing = matches!(builtin, Builtin::IteratorEvery);
            let callback = realm
                .eval(&format!(
                    "(value,index)=>{{seen.push(index);return {continuing};}}"
                ))
                .unwrap();
            let expected = match builtin {
                Builtin::IteratorEvery => Value::Boolean(true),
                Builtin::IteratorSome => Value::Boolean(false),
                _ => Value::Undefined,
            };
            assert_eq!(
                realm.iterator_predicate_counted(
                    builtin,
                    receiver,
                    callback,
                    Counter::Small(u64::MAX - 1),
                    Span::new(0, 0)
                ),
                Ok(expected)
            );
            assert_eq!(
                realm.eval("seen.length===4 && seen.every(value=>value===18446744073709551616)"),
                Ok(Value::Boolean(true))
            );
        }
    }

    #[test]
    fn internal_counter_promotion_remains_exact_and_obeys_work_failures() {
        let mut counter = Counter::Small(u64::MAX);
        for _ in 0..3 {
            counter
                .advance(&mut Budget::with_limits(None, None))
                .unwrap();
        }
        let Counter::Large(integer) = counter else {
            panic!("promoted counter")
        };
        assert_eq!(
            integer
                .to_radix(10, &mut Budget::with_limits(None, None))
                .unwrap(),
            "18446744073709551618"
        );
        let mut counter = Counter::Small(7);
        assert_eq!(
            counter.advance(&mut Budget::with_limits(None, Some(0))),
            Err(IntegerError::Limit)
        );
        assert_eq!(
            counter.number(&mut Budget::with_limits(None, None)),
            Ok(7.0)
        );
    }
}
