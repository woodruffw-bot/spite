//! Lazy concat closure and iterator-helper resumes (27.1.3.2.1, 27.1.2.1).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{CallbackKind, ConcatIterable, HelperStatus, IteratorWrapper},
};
use spite_core::{Span, WellKnownSymbol};

impl Realm {
    pub(crate) fn iterator_concat(
        &mut self,
        arguments: std::vec::IntoIter<Value>,
        span: Span,
    ) -> Result<Value, Error> {
        let mut sources = Vec::new();
        sources
            .try_reserve(arguments.len())
            .map_err(|_| Error::Limit {
                span,
                message: "iterator capture allocation failed".into(),
            })?;
        for value in arguments {
            let Value::Object(iterable) = value else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Iterator.concat requires object iterables",
                ));
            };
            let Some(method) = self.get_method(
                &Value::Object(iterable.clone()),
                &WellKnownSymbol::Iterator.symbol(),
                span,
            )?
            else {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Iterator.concat requires an iterator method",
                ));
            };
            let Value::Object(method) = method else {
                unreachable!("callable object")
            };
            sources.push(ConcatIterable { iterable, method });
        }
        let prototype = self
            .record
            .intrinsics
            .as_ref()
            .expect("initialized")
            .iterator
            .helper_prototype
            .clone();
        self.object_work(span, |objects, budget| {
            objects.create_concat_helper(&prototype, sources, budget)
        })
        .map(Value::Object)
    }

    pub(crate) fn iterator_helper_resume(
        &mut self,
        receiver: Value,
        return_method: bool,
        span: Span,
    ) -> Result<Value, Error> {
        let brand_error = || {
            Self::exception(
                ExceptionKind::TypeError,
                span,
                "receiver is not an Iterator Helper",
            )
        };
        let Value::Object(helper) = receiver else {
            return Err(brand_error());
        };
        let (previous, owner) = self
            .object_work(span, |objects, budget| {
                let Some(previous) = objects.begin_iterator_helper(&helper, budget)? else {
                    return Ok(None);
                };
                let owner = objects
                    .inspect(&helper)?
                    .iterator_helper()
                    .expect("validated helper")
                    .realm
                    .clone();
                Ok(Some((previous, owner)))
            })?
            .ok_or_else(brand_error)?;
        match previous {
            HelperStatus::Completed => return self.iterator_result(Value::Undefined, true, span),
            HelperStatus::Executing => {
                return Err(Self::exception(
                    ExceptionKind::TypeError,
                    span,
                    "Iterator Helper is already executing",
                ));
            }
            HelperStatus::SuspendedStart | HelperStatus::SuspendedYield => {}
        }
        if return_method && previous == HelperStatus::SuspendedStart {
            // 27.1.2.1.2: complete before IteratorCloseAll, so cleanup reentry
            // observes completion. Concat initially has no underlying iterators;
            // callback helpers have their captured direct iterator immediately.
            let iterator = self.object_work(span, |objects, _| {
                Ok(objects
                    .inspect(&helper)?
                    .iterator_helper()
                    .expect("helper")
                    .underlying()
                    .cloned())
            });
            self.objects
                .finish_iterator_helper(&helper, false)
                .expect("validated executing helper");
            if let Some(iterator) = iterator? {
                self.iterator_close_direct(iterator, span)?;
            }
            return self.iterator_result(Value::Undefined, true, span);
        }
        let execute = |realm: &mut Self| {
            let value = if return_method {
                realm.iterator_helper_close_yielded(&helper, span)?;
                None
            } else {
                let (callback, limit) = realm.object_work(span, |objects, _| {
                    let state = objects.inspect(&helper)?.iterator_helper().expect("helper");
                    Ok((
                        state.callback().map(|state| state.kind),
                        state.limit().is_some(),
                    ))
                })?;
                if callback == Some(CallbackKind::FlatMap) {
                    realm.iterator_flat_map_step(&helper, span)?
                } else if callback.is_some() {
                    realm.iterator_callback_step(&helper, span)?
                } else if limit {
                    realm.iterator_limit_step(&helper, span)?
                } else {
                    realm.concat_step(&helper, span)?
                }
            };
            match value {
                Some(value) => realm.iterator_result(value, false, span).map(Some),
                None => Ok(None),
            }
        };
        let result = match owner {
            Some(owner) => self.with_function_realm(&owner, span, execute),
            None => execute(self),
        };
        // State transition and capture release were prepaid before this resume.
        // This non-allocating cleanup must
        // run even after a host quota failure, without calling JavaScript cleanup.
        // Internal handles remain valid throughout a resume; collection occurs
        // only between host operations. Captures stayed traced while executing.
        self.objects
            .finish_iterator_helper(&helper, matches!(&result, Ok(Some(_))))
            .expect("validated executing helper");
        // GeneratorStart creates its completion result after popping the
        // closure context, unlike a yielded result (27.5.3.1).
        match result? {
            Some(result) => Ok(result),
            None => self.iterator_result(Value::Undefined, true, span),
        }
    }

    fn concat_step(&mut self, helper: &ObjectHandle, span: Span) -> Result<Option<Value>, Error> {
        loop {
            let next = self.object_work(span, |objects, budget| {
                let state = objects
                    .inspect(helper)?
                    .iterator_helper()
                    .expect("helper")
                    .concat()
                    .expect("active concat");
                if let Some(inner) = &state.inner {
                    budget.value(&inner.next)?;
                    return Ok(Some((inner.iterator.clone(), inner.next.clone())));
                }
                Ok(None)
            })?;
            if let Some((iterator, next)) = next {
                // IteratorStepValue failures complete this closure and never
                // close the source. Only a return resumed at Yield closes it.
                if let Some(value) = self.iterator_step_value_direct(iterator, next, span)? {
                    return Ok(Some(value));
                }
                self.object_work(span, |objects, _| objects.finish_concat_inner(helper))?;
                continue;
            }
            let source = self.object_work(span, |objects, _| {
                let state = objects
                    .inspect(helper)?
                    .iterator_helper()
                    .expect("helper")
                    .concat()
                    .expect("active concat");
                Ok(state
                    .sources
                    .get(state.index)
                    .map(|source| (source.iterable.clone(), source.method.clone())))
            })?;
            let Some((iterable, method)) = source else {
                return Ok(None);
            };
            let record = self.get_iterator_from_method(
                Value::Object(iterable),
                Value::Object(method),
                span,
            )?;
            self.object_work(span, |objects, _| {
                objects.set_concat_inner(
                    helper,
                    IteratorWrapper {
                        iterator: record.iterator,
                        next: record.next,
                    },
                )
            })?;
        }
    }
}
