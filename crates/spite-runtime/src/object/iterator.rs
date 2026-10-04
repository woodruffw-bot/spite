//! Built-in iterator state and source retention (22.1.3.36, 23.1.5, 27.1.3.2.2).

use super::{Budget, Error, Objects, Value};
use spite_core::JsString;
use spite_heap::{Handle, Trace};

mod helper;
pub(crate) use helper::{
    CallbackIterator, CallbackKind, ConcatIterable, HelperStatus, IteratorHelper, LimitKind,
};

#[derive(Debug)]
pub(super) enum IteratorState {
    Array(ArrayIterator),
    String(StringIterator),
    Wrapper(Box<IteratorWrapper>),
    Helper(Box<IteratorHelper>),
}

impl IteratorState {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        let (first, second) = match self {
            Self::Array(state) => (state.array.as_ref(), None),
            Self::String(_) | Self::Helper(_) => (None, None),
            Self::Wrapper(state) => (
                Some(&state.iterator),
                Some(state.next.trace().next().flatten()),
            ),
        };
        let helper = match self {
            Self::Helper(state) => Some(state),
            _ => None,
        };
        std::iter::once(first)
            .chain(second)
            .chain(helper.into_iter().flat_map(|state| state.trace()))
    }
}

#[derive(Debug)]
pub(crate) struct IteratorWrapper {
    pub iterator: Handle,
    pub next: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArrayIterationKind {
    Key,
    Value,
    KeyValue,
}

#[derive(Clone, Debug)]
pub(crate) struct ArrayIterator {
    pub array: Option<Handle>,
    pub next_index: u64,
    pub kind: ArrayIterationKind,
}

#[derive(Debug)]
pub(crate) struct StringIterator {
    string: Option<JsString>,
    next_index: usize,
}

impl Objects {
    pub(crate) fn create_iterator_wrapper(
        &mut self,
        prototype: &Handle,
        state: IteratorWrapper,
    ) -> Result<Handle, Error> {
        self.inspect(&state.iterator)?;
        if let Value::Object(next) = &state.next {
            self.inspect(next)?;
        }
        let wrapper = self.create(Some(prototype))?;
        self.object_mut(&wrapper)?.iterator = Some(IteratorState::Wrapper(Box::new(state)));
        Ok(wrapper)
    }

    pub(crate) fn create_string_iterator(
        &mut self,
        prototype: &Handle,
        string: JsString,
    ) -> Result<Handle, Error> {
        let iterator = self.create(Some(prototype))?;
        self.object_mut(&iterator)?.iterator = Some(IteratorState::String(StringIterator {
            string: Some(string),
            next_index: 0,
        }));
        Ok(iterator)
    }

    pub(crate) fn next_string_iterator(
        &mut self,
        iterator: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<JsString>, Error> {
        // The String iterator closure performs no user calls between resumes.
        // Its suspended position and completed state suffice; general-purpose
        // Generator objects will require a separate execution-state machine.
        budget.charge(2)?;
        let Some(IteratorState::String(state)) = &mut self.object_mut(iterator)?.iterator else {
            return Err(Error::WrongKind);
        };
        let Some(string) = &state.string else {
            return Ok(None);
        };
        if state.next_index >= string.len() {
            state.string = None;
            return Ok(None);
        }
        let index = state.next_index;
        let units = string.code_units();
        // CodePointAt (11.1.4): pair only a leading and immediately following
        // trailing surrogate. Unpaired surrogates remain individual strings.
        let count = if (0xd800..=0xdbff).contains(&units[index])
            && units
                .get(index + 1)
                .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
        {
            2
        } else {
            1
        };
        budget.charge(count)?;
        let result = JsString::from_code_units(units[index..index + count].to_vec());
        state.next_index += count;
        Ok(Some(result))
    }

    pub(crate) fn create_array_iterator(
        &mut self,
        prototype: &Handle,
        array: &Handle,
        kind: ArrayIterationKind,
    ) -> Result<Handle, Error> {
        self.inspect(array)?;
        let iterator = self.create(Some(prototype))?;
        self.object_mut(&iterator)?.iterator = Some(IteratorState::Array(ArrayIterator {
            array: Some(array.clone()),
            next_index: 0,
            kind,
        }));
        Ok(iterator)
    }

    pub(crate) fn set_array_iterator_index(
        &mut self,
        iterator: &Handle,
        index: u64,
    ) -> Result<(), Error> {
        let Some(IteratorState::Array(state)) = &mut self.object_mut(iterator)?.iterator else {
            return Err(Error::WrongKind);
        };
        state.next_index = index;
        Ok(())
    }

    pub(crate) fn finish_array_iterator(&mut self, iterator: &Handle) -> Result<(), Error> {
        let Some(IteratorState::Array(state)) = &mut self.object_mut(iterator)?.iterator else {
            return Err(Error::WrongKind);
        };
        state.array = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_iterator_steps_copy_only_one_code_point_and_release_completed_input() {
        let mut objects = Objects::new(3, 8);
        let prototype = objects.create(None).unwrap();
        let mut units = vec![0xd800, 0xdc00, 0xd800, 0, 0xdc00];
        units.extend(std::iter::repeat_n(65, 10_000));
        let iterator = objects
            .create_string_iterator(&prototype, JsString::from_code_units(units))
            .unwrap();
        assert!(matches!(
            objects.next_string_iterator(&iterator, &mut Budget::new(2)),
            Err(Error::WorkLimit)
        ));
        for expected in [&[0xd800, 0xdc00][..], &[0xd800], &[0], &[0xdc00]] {
            let next = objects
                .next_string_iterator(&iterator, &mut Budget::new(4))
                .unwrap()
                .unwrap();
            assert_eq!(next.code_units(), expected);
        }
        for _ in 0..10_000 {
            assert_eq!(
                objects
                    .next_string_iterator(&iterator, &mut Budget::new(3))
                    .unwrap(),
                Some(JsString::from("A"))
            );
        }
        for _ in 0..2 {
            assert_eq!(
                objects.next_string_iterator(&iterator, &mut Budget::new(2)),
                Ok(None)
            );
            assert!(
                objects
                    .inspect(&iterator)
                    .unwrap()
                    .string_iterator()
                    .unwrap()
                    .string
                    .is_none()
            );
        }
        assert!(matches!(
            objects.next_string_iterator(&prototype, &mut Budget::new(4)),
            Err(Error::WrongKind)
        ));
    }

    #[test]
    fn iterator_slots_retain_source_until_exhaustion_and_are_not_inherited() {
        let mut objects = Objects::new(6, 8);
        let prototype = objects.create(None).unwrap();
        let source = objects.create(None).unwrap();
        let garbage = objects.create(None).unwrap();
        let iterator = objects
            .create_array_iterator(&prototype, &source, ArrayIterationKind::Value)
            .unwrap();
        let lookalike = objects.create(Some(&iterator)).unwrap();
        assert!(
            objects
                .inspect(&lookalike)
                .unwrap()
                .array_iterator()
                .is_none()
        );
        let collected = objects.collect([&iterator], 100).unwrap();
        assert_eq!((collected.live, collected.reclaimed), (3, 2));
        assert!(objects.inspect(&source).is_ok());
        assert!(objects.inspect(&garbage).is_err());
        objects
            .set_array_iterator_index(&iterator, 9_007_199_254_740_990)
            .unwrap();
        assert_eq!(
            objects
                .inspect(&iterator)
                .unwrap()
                .array_iterator()
                .unwrap()
                .next_index,
            9_007_199_254_740_990
        );
        objects.finish_array_iterator(&iterator).unwrap();
        objects.set_array_iterator_index(&iterator, 1).unwrap();
        assert!(
            objects
                .inspect(&iterator)
                .unwrap()
                .array_iterator()
                .unwrap()
                .array
                .is_none()
        );
        let collected = objects.collect([&iterator], 100).unwrap();
        assert_eq!((collected.live, collected.reclaimed), (2, 1));
        assert!(matches!(
            objects.inspect(&source),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
    }

    #[test]
    fn wrapper_sources_and_cached_next_handles_are_validated_before_allocation() {
        let mut objects = Objects::new(4, 8);
        let prototype = objects.create(None).unwrap();
        let source = objects.create(None).unwrap();
        let next = objects.create(None).unwrap();
        let mut other = Objects::new(1, 8);
        let foreign = other.create(None).unwrap();
        for (proto, iterator, next_method) in [
            (&foreign, &source, &next),
            (&prototype, &foreign, &next),
            (&prototype, &source, &foreign),
        ] {
            assert!(matches!(
                objects.create_iterator_wrapper(
                    proto,
                    IteratorWrapper {
                        iterator: iterator.clone(),
                        next: Value::Object(next_method.clone()),
                    }
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
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
        for (iterator, next_method) in [(&source, &prototype), (&prototype, &next)] {
            assert!(matches!(
                objects.create_iterator_wrapper(
                    &prototype,
                    IteratorWrapper {
                        iterator: iterator.clone(),
                        next: Value::Object(next_method.clone()),
                    }
                ),
                Err(Error::Heap(spite_heap::Error::StaleHandle))
            ));
        }
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
    }

    #[test]
    fn foreign_or_stale_source_handles_are_rejected_before_allocation() {
        let mut objects = Objects::new(4, 8);
        let prototype = objects.create(None).unwrap();
        let source = objects.create(None).unwrap();
        let mut other = Objects::new(1, 8);
        let foreign = other.create(None).unwrap();
        for (proto, array) in [(&prototype, &foreign), (&foreign, &source)] {
            assert!(matches!(
                objects.create_array_iterator(proto, array, ArrayIterationKind::Key),
                Err(Error::Heap(spite_heap::Error::ForeignHandle))
            ));
        }
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
        assert!(matches!(
            objects.create_array_iterator(&prototype, &source, ArrayIterationKind::Key),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
        assert!(matches!(
            objects.finish_array_iterator(&prototype),
            Err(Error::WrongKind)
        ));
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
    }
}
