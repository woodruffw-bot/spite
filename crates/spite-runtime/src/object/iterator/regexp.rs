//! RegExp String Iterator slots and matcher tracing (22.2.9).

use super::{Error, IteratorState, Objects};
use spite_core::JsString;
use spite_heap::Handle;

#[derive(Clone, Debug)]
pub(crate) struct RegExpStringIterator {
    pub matcher: Handle,
    pub string: JsString,
    pub global: bool,
    pub unicode: bool,
    pub done: bool,
}

impl Objects {
    pub(crate) fn create_regexp_string_iterator(
        &mut self,
        prototype: &Handle,
        state: RegExpStringIterator,
    ) -> Result<Handle, Error> {
        self.inspect(&state.matcher)?;
        let iterator = self.create(Some(prototype))?;
        self.object_mut(&iterator)?.iterator = Some(IteratorState::RegExp(state));
        Ok(iterator)
    }

    pub(crate) fn finish_regexp_string_iterator(&mut self, iterator: &Handle) -> Result<(), Error> {
        let Some(IteratorState::RegExp(state)) = &mut self.object_mut(iterator)?.iterator else {
            return Err(Error::WrongKind);
        };
        state.done = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iterator_slots_trace_the_matcher_even_after_completion_and_are_not_inherited() {
        let mut objects = Objects::new(5, 8);
        let prototype = objects.create(None).unwrap();
        let matcher = objects.create(None).unwrap();
        let iterator = objects
            .create_regexp_string_iterator(
                &prototype,
                RegExpStringIterator {
                    matcher: matcher.clone(),
                    string: JsString::from("x"),
                    global: true,
                    unicode: false,
                    done: false,
                },
            )
            .unwrap();
        let lookalike = objects.create(Some(&iterator)).unwrap();
        assert!(
            objects
                .inspect(&lookalike)
                .unwrap()
                .regexp_string_iterator()
                .is_none()
        );
        assert!(matches!(
            objects.finish_regexp_string_iterator(&lookalike),
            Err(Error::WrongKind)
        ));
        assert!(
            !objects
                .inspect(&iterator)
                .unwrap()
                .regexp_string_iterator()
                .unwrap()
                .done
        );
        let collected = objects.collect([&iterator], 100).unwrap();
        assert_eq!((collected.live, collected.reclaimed), (3, 1));
        objects.finish_regexp_string_iterator(&iterator).unwrap();
        assert!(
            objects
                .inspect(&iterator)
                .unwrap()
                .regexp_string_iterator()
                .unwrap()
                .done
        );
        assert_eq!(objects.collect([&iterator], 100).unwrap().live, 3);
        assert!(objects.inspect(&matcher).is_ok());
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
        assert!(matches!(
            objects.inspect(&matcher),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
    }

    #[test]
    fn foreign_and_stale_matchers_are_rejected_before_allocating_an_iterator() {
        let mut objects = Objects::new(4, 8);
        let prototype = objects.create(None).unwrap();
        let matcher = objects.create(None).unwrap();
        let mut other = Objects::new(1, 8);
        let foreign = other.create(None).unwrap();
        for (proto, source) in [(&foreign, &matcher), (&prototype, &foreign)] {
            assert!(matches!(
                objects.create_regexp_string_iterator(
                    proto,
                    RegExpStringIterator {
                        matcher: source.clone(),
                        string: JsString::default(),
                        global: false,
                        unicode: true,
                        done: false,
                    }
                ),
                Err(Error::Heap(spite_heap::Error::ForeignHandle))
            ));
        }
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
        assert!(matches!(
            objects.create_regexp_string_iterator(
                &prototype,
                RegExpStringIterator {
                    matcher,
                    string: JsString::default(),
                    global: false,
                    unicode: false,
                    done: false,
                }
            ),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
        assert_eq!(objects.collect([&prototype], 100).unwrap().live, 1);
    }
}
