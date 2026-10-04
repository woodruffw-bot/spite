//! One-level lazy flattening and nested iterator closing (27.1.3.3.6).

use super::operations::IteratorRecord;
use crate::{Error, ObjectHandle, Realm, Value, object::IteratorWrapper};
use spite_core::Span;

impl Realm {
    pub(super) fn iterator_flat_map_step(
        &mut self,
        helper: &ObjectHandle,
        span: Span,
    ) -> Result<Option<Value>, Error> {
        loop {
            let inner = self.object_work(span, |objects, budget| {
                let state = objects
                    .inspect(helper)?
                    .iterator_helper()
                    .expect("helper")
                    .callback()
                    .expect("active flatMap");
                if let Some(inner) = &state.inner {
                    budget.value(&inner.next)?;
                    Ok(Some((
                        inner.iterator.clone(),
                        inner.next.clone(),
                        state.iterated.iterator.clone(),
                    )))
                } else {
                    Ok(None)
                }
            })?;
            if let Some((iterator, next, outer)) = inner {
                // IteratorStepValue marks the failing inner done. Only the
                // outer source is closed after a language exception here.
                match self.iterator_step_value_direct(iterator, next, span) {
                    Ok(Some(value)) => return Ok(Some(value)),
                    Ok(None) => self.object_work(span, |objects, budget| {
                        objects.finish_callback_inner(helper, budget)
                    })?,
                    Err(error) => {
                        return Err(self.iterator_close_error(
                            &IteratorRecord::uninitialized(outer),
                            error,
                            span,
                        ));
                    }
                }
                continue;
            }
            // Advance once per exhausted mapped iterator, including empty
            // ones. Yielding another inner value never changes the index.
            self.iterator_callback_counter_work(helper, span, |state, budget| {
                if state.advance {
                    state.counter.advance(budget)?;
                    state.advance = false;
                }
                Ok(())
            })?;
            let (iterator, next, mapper) = self.object_work(span, |objects, budget| {
                let state = objects
                    .inspect(helper)?
                    .iterator_helper()
                    .expect("helper")
                    .callback()
                    .expect("active flatMap");
                budget.value(&state.iterated.next)?;
                Ok((
                    state.iterated.iterator.clone(),
                    state.iterated.next.clone(),
                    state.callback.clone(),
                ))
            })?;
            // Outer stepping failures and host failures never run cleanup.
            let Some(value) = self.iterator_step_value_direct(iterator.clone(), next, span)? else {
                return Ok(None);
            };
            let index = self.iterator_callback_counter_work(helper, span, |state, budget| {
                state.counter.number(budget)
            })?;
            let inner = self
                .call(
                    Value::Object(mapper),
                    Value::Undefined,
                    vec![value, Value::Number(index)],
                    span,
                )
                .and_then(|mapped| self.get_iterator_flattenable(mapped, false, span));
            let record = match inner {
                Ok(record) => record,
                Err(error) => {
                    return Err(self.iterator_close_error(
                        &IteratorRecord::uninitialized(iterator),
                        error,
                        span,
                    ));
                }
            };
            self.object_work(span, |objects, _| {
                objects.set_callback_inner(
                    helper,
                    IteratorWrapper {
                        iterator: record.iterator,
                        next: record.next,
                    },
                )
            })?;
        }
    }

    pub(super) fn iterator_helper_close_yielded(
        &mut self,
        helper: &ObjectHandle,
        span: Span,
    ) -> Result<(), Error> {
        let (outer, inner) = self.object_work(span, |objects, _| {
            let state = objects.inspect(helper)?.iterator_helper().expect("helper");
            Ok((
                state.underlying().cloned(),
                state
                    .callback()
                    .and_then(|state| state.inner.as_ref())
                    .map(|inner| inner.iterator.clone()),
            ))
        })?;
        if let Some(inner) = inner {
            if let Err(error) = self.iterator_close_direct(inner, span) {
                // A failed inner close becomes the incoming throw for the
                // outer close. Host failures bypass further JavaScript cleanup.
                return Err(self.iterator_close_error(
                    &IteratorRecord::uninitialized(outer.expect("flatMap outer")),
                    error,
                    span,
                ));
            }
        }
        if let Some(outer) = outer {
            self.iterator_close_direct(outer, span)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Limits, iterator_count::Counter};

    #[test]
    fn outer_indices_remain_exact_beyond_u64_without_bigint_value_quota() {
        let mut realm = Realm::new(Limits {
            max_bigint_bits: Some(0),
            ..Limits::default()
        });
        let Value::Object(helper) = realm.eval("let seen=[],h=[7,7,7,7].values().flatMap((value,index)=>{seen.push(index);return [value,value];});h").unwrap() else {
            panic!("helper")
        };
        let span = Span::new(0, 0);
        realm
            .object_work(span, |objects, budget| {
                objects.begin_iterator_helper(&helper, budget)?;
                objects.callback_mut(&helper)?.counter = Counter::Small(u64::MAX - 1);
                Ok(())
            })
            .unwrap();
        for _ in 0..8 {
            assert!(
                realm
                    .iterator_flat_map_step(&helper, span)
                    .unwrap()
                    .is_some()
            );
        }
        assert_eq!(realm.iterator_flat_map_step(&helper, span), Ok(None));
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
