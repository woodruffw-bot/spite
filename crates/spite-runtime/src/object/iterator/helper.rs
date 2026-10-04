//! Native iterator-helper suspension state and traced concat captures (27.1.2).

use super::{Budget, Error, IteratorState, IteratorWrapper, Objects};
use spite_heap::{Handle, Trace};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HelperStatus {
    SuspendedStart,
    SuspendedYield,
    Executing,
    Completed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::Builtin;

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

#[derive(Debug)]
pub(crate) struct IteratorHelper {
    pub status: HelperStatus,
    pub concat: Option<ConcatIterator>,
}

impl IteratorHelper {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.concat.iter().flat_map(|state| {
            state
                .sources
                .iter()
                .flat_map(|source| [Some(&source.iterable), Some(&source.method)])
                .chain(state.inner.iter().flat_map(|inner| {
                    std::iter::once(Some(&inner.iterator)).chain(inner.next.trace())
                }))
        })
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
                concat: Some(ConcatIterator {
                    sources,
                    index: 0,
                    inner: None,
                }),
            })));
        Ok(helper)
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
            // Prepay the finish transition and release of the two active
            // iterator/next references. Captured source release was paid at
            // creation. Host work failures cannot strand an executing object.
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
            state.concat = None;
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
        state.concat.as_mut().ok_or(Error::WrongKind)
    }
}
