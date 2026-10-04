//! Lazy concat closure and iterator-helper resumes (27.1.3.2.1, 27.1.2.1).

use crate::{
    Error, ExceptionKind, ObjectHandle, Realm, Value,
    object::{ConcatIterable, HelperStatus, IteratorWrapper},
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
        let previous = self
            .object_work(span, |objects, budget| {
                objects.begin_iterator_helper(&helper, budget)
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
        let result = (|| {
            let value = if return_method {
                // Concat's [[UnderlyingIterators]] starts empty. Before its
                // first next, return closes nothing and never opens a source.
                if previous == HelperStatus::SuspendedYield {
                    let iterator = self.object_work(span, |objects, _| {
                        let state = objects.inspect(&helper)?.iterator_helper().expect("helper");
                        Ok(state
                            .concat
                            .as_ref()
                            .expect("active concat")
                            .inner
                            .as_ref()
                            .map(|inner| inner.iterator.clone()))
                    })?;
                    if let Some(iterator) = iterator {
                        self.iterator_close_direct(iterator, span)?;
                    }
                }
                None
            } else {
                self.concat_step(&helper, span)?
            };
            let yielded = value.is_some();
            self.iterator_result(value.unwrap_or(Value::Undefined), !yielded, span)
                .map(|result| (result, yielded))
        })();
        // State transition and capture release were prepaid before this resume.
        // This non-allocating cleanup must
        // run even after a host quota failure, without calling JavaScript cleanup.
        // Internal handles remain valid throughout a resume; collection occurs
        // only between host operations. Captures stayed traced while executing.
        self.objects
            .finish_iterator_helper(&helper, matches!(&result, Ok((_, true))))
            .expect("validated executing helper");
        result.map(|(value, _)| value)
    }

    fn concat_step(&mut self, helper: &ObjectHandle, span: Span) -> Result<Option<Value>, Error> {
        loop {
            let next = self.object_work(span, |objects, budget| {
                let state = objects
                    .inspect(helper)?
                    .iterator_helper()
                    .expect("helper")
                    .concat
                    .as_ref()
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
                    .concat
                    .as_ref()
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
