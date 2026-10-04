//! Array iterator internal slots and source retention (23.1.5).

use super::{Error, Objects};
use spite_heap::Handle;

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

impl Objects {
    pub(crate) fn create_array_iterator(
        &mut self,
        prototype: &Handle,
        array: &Handle,
        kind: ArrayIterationKind,
    ) -> Result<Handle, Error> {
        self.inspect(array)?;
        let iterator = self.create(Some(prototype))?;
        self.object_mut(&iterator)?.array_iterator = Some(ArrayIterator {
            array: Some(array.clone()),
            next_index: 0,
            kind,
        });
        Ok(iterator)
    }

    pub(crate) fn set_array_iterator_index(
        &mut self,
        iterator: &Handle,
        index: u64,
    ) -> Result<(), Error> {
        self.object_mut(iterator)?
            .array_iterator
            .as_mut()
            .ok_or(Error::WrongKind)?
            .next_index = index;
        Ok(())
    }

    pub(crate) fn finish_array_iterator(&mut self, iterator: &Handle) -> Result<(), Error> {
        self.object_mut(iterator)?
            .array_iterator
            .as_mut()
            .ok_or(Error::WrongKind)?
            .array = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
