//! Native iterator-helper suspension state and traced captures (27.1.2).

use super::{Budget, Error, IteratorState, IteratorWrapper, Objects};
use crate::iterator_count::Counter;
use spite_heap::{Handle, Trace};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HelperStatus {
    SuspendedStart,
    SuspendedYield,
    Executing,
    Completed,
}

#[derive(Debug)]
pub(crate) struct ConcatIterable {
    pub iterable: Handle,
    pub method: Handle,
}

#[derive(Debug)]
pub(crate) struct ConcatIterator {
    pub sources: Vec<ConcatIterable>,
    pub index: usize,
    pub inner: Option<IteratorWrapper>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallbackKind {
    Map,
    Filter,
    FlatMap,
}

#[derive(Debug)]
pub(crate) struct CallbackIterator {
    pub iterated: IteratorWrapper,
    pub callback: Handle,
    pub kind: CallbackKind,
    pub counter: Counter,
    pub advance: bool,
    pub inner: Option<IteratorWrapper>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LimitKind {
    Take,
    Drop,
}

#[derive(Debug)]
pub(crate) struct LimitIterator {
    pub iterated: IteratorWrapper,
    pub kind: LimitKind,
    // None represents positive infinity; finite counts remain mathematical integers.
    pub remaining: Option<Counter>,
}

#[derive(Debug)]
pub(crate) enum HelperClosure {
    Concat(ConcatIterator),
    Callback(CallbackIterator),
    Limit(LimitIterator),
}

#[derive(Debug)]
pub(crate) struct IteratorHelper {
    pub status: HelperStatus,
    closure: Option<HelperClosure>,
}

impl IteratorHelper {
    pub(crate) fn concat(&self) -> Option<&ConcatIterator> {
        match &self.closure {
            Some(HelperClosure::Concat(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn callback(&self) -> Option<&CallbackIterator> {
        match &self.closure {
            Some(HelperClosure::Callback(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn limit(&self) -> Option<&LimitIterator> {
        match &self.closure {
            Some(HelperClosure::Limit(state)) => Some(state),
            _ => None,
        }
    }

    pub(crate) fn underlying(&self) -> Option<&Handle> {
        match &self.closure {
            Some(HelperClosure::Concat(state)) => state.inner.as_ref().map(|inner| &inner.iterator),
            Some(HelperClosure::Callback(state)) => Some(&state.iterated.iterator),
            Some(HelperClosure::Limit(state)) => Some(&state.iterated.iterator),
            None => None,
        }
    }

    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.concat()
            .into_iter()
            .flat_map(|state| {
                state
                    .sources
                    .iter()
                    .flat_map(|source| [Some(&source.iterable), Some(&source.method)])
                    .chain(state.inner.iter().flat_map(|inner| {
                        std::iter::once(Some(&inner.iterator)).chain(inner.next.trace())
                    }))
            })
            .chain(self.callback().into_iter().flat_map(|state| {
                [
                    Some(&state.iterated.iterator),
                    Some(&state.callback),
                    state.iterated.next.trace().next().flatten(),
                ]
                .into_iter()
                .chain(state.inner.iter().flat_map(|inner| {
                    std::iter::once(Some(&inner.iterator)).chain(inner.next.trace())
                }))
            }))
            .chain(self.limit().into_iter().flat_map(|state| {
                [
                    Some(&state.iterated.iterator),
                    state.iterated.next.trace().next().flatten(),
                ]
            }))
    }
}

impl Objects {
    pub(crate) fn create_concat_helper(
        &mut self,
        prototype: &Handle,
        sources: Vec<ConcatIterable>,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        for source in &sources {
            // Two handle validations and their eventual release on completion.
            // Prepay release once so cleanup never depends on a fresh allowance.
            budget.charge(4)?;
            self.inspect(&source.iterable)?;
            if !self.inspect(&source.method)?.is_callable() {
                return Err(Error::NotCallable);
            }
        }
        let helper = self.create(Some(prototype))?;
        self.object_mut(&helper)?.iterator =
            Some(IteratorState::Helper(Box::new(IteratorHelper {
                status: HelperStatus::SuspendedStart,
                closure: Some(HelperClosure::Concat(ConcatIterator {
                    sources,
                    index: 0,
                    inner: None,
                })),
            })));
        Ok(helper)
    }

    pub(crate) fn create_callback_helper(
        &mut self,
        prototype: &Handle,
        iterated: IteratorWrapper,
        callback: Handle,
        kind: CallbackKind,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        // Validate three captures and prepay their release. Callability of next
        // belongs to the first step, whereas callback callability is required now.
        budget.charge(6)?;
        self.inspect(&iterated.iterator)?;
        if let crate::Value::Object(next) = &iterated.next {
            self.inspect(next)?;
        }
        if !self.inspect(&callback)?.is_callable() {
            return Err(Error::NotCallable);
        }
        let helper = self.create(Some(prototype))?;
        self.object_mut(&helper)?.iterator =
            Some(IteratorState::Helper(Box::new(IteratorHelper {
                status: HelperStatus::SuspendedStart,
                closure: Some(HelperClosure::Callback(CallbackIterator {
                    iterated,
                    callback,
                    kind,
                    counter: Counter::Small(0),
                    advance: false,
                    inner: None,
                })),
            })));
        Ok(helper)
    }

    pub(crate) fn create_limit_helper(
        &mut self,
        prototype: &Handle,
        iterated: IteratorWrapper,
        remaining: Option<Counter>,
        kind: LimitKind,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        // Validate captures and prepay their eventual release.
        budget.charge(4)?;
        self.inspect(&iterated.iterator)?;
        if let crate::Value::Object(next) = &iterated.next {
            self.inspect(next)?;
        }
        let helper = self.create(Some(prototype))?;
        self.object_mut(&helper)?.iterator =
            Some(IteratorState::Helper(Box::new(IteratorHelper {
                status: HelperStatus::SuspendedStart,
                closure: Some(HelperClosure::Limit(LimitIterator {
                    iterated,
                    remaining,
                    kind,
                })),
            })));
        Ok(helper)
    }

    pub(crate) fn limit_mut(&mut self, helper: &Handle) -> Result<&mut LimitIterator, Error> {
        let Some(IteratorState::Helper(state)) = &mut self.object_mut(helper)?.iterator else {
            return Err(Error::WrongKind);
        };
        debug_assert_eq!(state.status, HelperStatus::Executing);
        match &mut state.closure {
            Some(HelperClosure::Limit(state)) => Ok(state),
            _ => Err(Error::WrongKind),
        }
    }

    pub(crate) fn begin_iterator_helper(
        &mut self,
        helper: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<HelperStatus>, Error> {
        let Some(IteratorState::Helper(state)) = &mut self.object_mut(helper)?.iterator else {
            return Ok(None);
        };
        let previous = state.status;
        if matches!(
            previous,
            HelperStatus::SuspendedStart | HelperStatus::SuspendedYield
        ) {
            // Finish and active inner-reference release are prepaid per resume;
            // all other capture release was prepaid at creation. Host work
            // failures cannot strand an executing object.
            budget.charge(3)?;
            state.status = HelperStatus::Executing;
        }
        Ok(Some(previous))
    }

    pub(crate) fn finish_iterator_helper(
        &mut self,
        helper: &Handle,
        yielded: bool,
    ) -> Result<(), Error> {
        let Some(IteratorState::Helper(state)) = &mut self.object_mut(helper)?.iterator else {
            return Err(Error::WrongKind);
        };
        debug_assert_eq!(state.status, HelperStatus::Executing);
        if yielded {
            state.status = HelperStatus::SuspendedYield;
        } else {
            state.status = HelperStatus::Completed;
            state.closure = None;
        }
        Ok(())
    }

    pub(crate) fn set_concat_inner(
        &mut self,
        helper: &Handle,
        inner: IteratorWrapper,
    ) -> Result<(), Error> {
        self.inspect(&inner.iterator)?;
        if let crate::Value::Object(next) = &inner.next {
            self.inspect(next)?;
        }
        let state = self.concat_mut(helper)?;
        debug_assert!(state.index < state.sources.len() && state.inner.is_none());
        state.inner = Some(inner);
        Ok(())
    }

    pub(crate) fn finish_concat_inner(&mut self, helper: &Handle) -> Result<(), Error> {
        let state = self.concat_mut(helper)?;
        debug_assert!(state.index < state.sources.len() && state.inner.is_some());
        state.inner = None;
        // index < sources.len(), which is an addressable Vec length.
        state.index += 1;
        Ok(())
    }

    fn concat_mut(&mut self, helper: &Handle) -> Result<&mut ConcatIterator, Error> {
        let Some(IteratorState::Helper(state)) = &mut self.object_mut(helper)?.iterator else {
            return Err(Error::WrongKind);
        };
        debug_assert_eq!(state.status, HelperStatus::Executing);
        match &mut state.closure {
            Some(HelperClosure::Concat(state)) => Ok(state),
            _ => Err(Error::WrongKind),
        }
    }

    pub(crate) fn set_callback_inner(
        &mut self,
        helper: &Handle,
        inner: IteratorWrapper,
    ) -> Result<(), Error> {
        self.inspect(&inner.iterator)?;
        if let crate::Value::Object(next) = &inner.next {
            self.inspect(next)?;
        }
        let state = self.callback_mut(helper)?;
        debug_assert_eq!(state.kind, CallbackKind::FlatMap);
        debug_assert!(state.inner.is_none());
        state.inner = Some(inner);
        Ok(())
    }

    pub(crate) fn finish_callback_inner(
        &mut self,
        helper: &Handle,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        // Each exhausted inner releases its two captures. Completion after a
        // host abort instead uses the release prepaid before this resume.
        budget.charge(2)?;
        let state = self.callback_mut(helper)?;
        debug_assert_eq!(state.kind, CallbackKind::FlatMap);
        debug_assert!(state.inner.is_some());
        state.inner = None;
        state.advance = true;
        Ok(())
    }

    pub(crate) fn callback_mut(&mut self, helper: &Handle) -> Result<&mut CallbackIterator, Error> {
        let Some(IteratorState::Helper(state)) = &mut self.object_mut(helper)?.iterator else {
            return Err(Error::WrongKind);
        };
        debug_assert_eq!(state.status, HelperStatus::Executing);
        match &mut state.closure {
            Some(HelperClosure::Callback(state)) => Ok(state),
            _ => Err(Error::WrongKind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::Builtin;

    #[test]
    fn limit_helper_rejects_foreign_captures_before_allocation() {
        use crate::Value;
        let mut objects = Objects::new(6, 8);
        let prototype = objects.create(None).unwrap();
        let source = objects.create(None).unwrap();
        let next = objects
            .create_builtin(&prototype, Builtin::IteratorIdentity)
            .unwrap();
        let mut other = Objects::new(1, 8);
        let foreign = other.create(None).unwrap();
        for (proto, iterator, step) in [
            (&foreign, &source, &next),
            (&prototype, &foreign, &next),
            (&prototype, &source, &foreign),
        ] {
            assert!(matches!(
                objects.create_limit_helper(
                    proto,
                    IteratorWrapper {
                        iterator: iterator.clone(),
                        next: Value::Object(step.clone())
                    },
                    Some(Counter::Small(1)),
                    LimitKind::Take,
                    &mut Budget::new(40)
                ),
                Err(Error::Heap(spite_heap::Error::ForeignHandle))
            ));
        }
        assert_eq!(
            objects
                .collect([&prototype, &source, &next], 100)
                .unwrap()
                .live,
            3
        );
    }

    #[test]
    fn callback_helper_rejects_foreign_and_noncallable_captures_before_allocation() {
        use crate::Value;
        let mut objects = Objects::new(6, 8);
        let prototype = objects.create(None).unwrap();
        let source = objects.create(None).unwrap();
        let callback = objects
            .create_builtin(&prototype, Builtin::IteratorIdentity)
            .unwrap();
        let mut other = Objects::new(1, 8);
        let foreign = other.create(None).unwrap();
        for (proto, iterator, next, procedure) in [
            (&foreign, &source, &callback, &callback),
            (&prototype, &foreign, &callback, &callback),
            (&prototype, &source, &foreign, &callback),
            (&prototype, &source, &callback, &foreign),
        ] {
            assert!(matches!(
                objects.create_callback_helper(
                    proto,
                    IteratorWrapper {
                        iterator: iterator.clone(),
                        next: Value::Object(next.clone())
                    },
                    procedure.clone(),
                    CallbackKind::Map,
                    &mut Budget::new(40)
                ),
                Err(Error::Heap(spite_heap::Error::ForeignHandle))
            ));
        }
        assert!(matches!(
            objects.create_callback_helper(
                &prototype,
                IteratorWrapper {
                    iterator: source.clone(),
                    next: Value::Undefined
                },
                source.clone(),
                CallbackKind::Filter,
                &mut Budget::new(40)
            ),
            Err(Error::NotCallable)
        ));
        assert_eq!(
            objects
                .collect([&prototype, &source, &callback], 100)
                .unwrap()
                .live,
            3
        );
    }

    #[test]
    fn foreign_or_noncallable_captures_are_rejected_before_allocating_a_helper() {
        let mut objects = Objects::new(5, 8);
        let prototype = objects.create(None).unwrap();
        let iterable = objects.create(None).unwrap();
        let method = objects
            .create_builtin(&prototype, Builtin::IteratorIdentity)
            .unwrap();
        let mut other = Objects::new(1, 8);
        let foreign = other.create(None).unwrap();
        for (proto, source, open) in [
            (&foreign, &iterable, &method),
            (&prototype, &foreign, &method),
            (&prototype, &iterable, &foreign),
        ] {
            assert!(matches!(
                objects.create_concat_helper(
                    proto,
                    vec![ConcatIterable {
                        iterable: source.clone(),
                        method: open.clone()
                    }],
                    &mut Budget::new(10)
                ),
                Err(Error::Heap(spite_heap::Error::ForeignHandle))
            ));
        }
        assert!(matches!(
            objects.create_concat_helper(
                &prototype,
                vec![ConcatIterable {
                    iterable: iterable.clone(),
                    method: iterable.clone()
                }],
                &mut Budget::new(10)
            ),
            Err(Error::NotCallable)
        ));
        assert_eq!(
            objects
                .collect([&prototype, &iterable, &method], 100)
                .unwrap()
                .live,
            3
        );
    }
}
