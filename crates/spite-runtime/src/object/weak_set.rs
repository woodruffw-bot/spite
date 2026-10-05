//! Hash-indexed weak identities; neither object nor Symbol keys are traced (24.4).

use super::{Budget, Error, Objects};
use crate::Value;
use spite_core::WeakJsSymbol;
use spite_heap::Handle;
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum WeakKey {
    Object(Handle),
    Symbol(WeakJsSymbol),
}

impl WeakKey {
    fn new(objects: &Objects, value: &Value) -> Result<Self, Error> {
        match value {
            Value::Object(object) => {
                objects.inspect(object)?;
                Ok(Self::Object(object.clone()))
            }
            Value::Symbol(symbol) => Ok(Self::Symbol(symbol.downgrade())),
            _ => Err(Error::WrongKind),
        }
    }

    fn is_live(&self, objects: &Objects) -> bool {
        match self {
            Self::Object(object) => objects.inspect(object).is_ok(),
            Self::Symbol(symbol) => symbol.upgrade().is_some(),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct WeakSetData {
    keys: HashSet<WeakKey>,
    // Pruning after n new insertions costs O(n), preserving amortized constant
    // access instead of scanning every key on each add. Cleanup latency is not
    // observable: querying an element requires presenting that live identity.
    insertions_before_prune: usize,
}

impl WeakSetData {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        // Charge collection for weak storage without marking any key.
        std::iter::once(None).chain(self.keys.iter().map(|_| None))
    }
}

impl Objects {
    pub(crate) fn create_weak_set(&mut self, prototype: &Handle) -> Result<Handle, Error> {
        let set = self.create(Some(prototype))?;
        self.object_mut(&set)?.weak_set = Some(Box::default());
        Ok(set)
    }

    pub(crate) fn weak_set_has_slot(&self, set: &Handle) -> Result<bool, Error> {
        Ok(self.inspect(set)?.weak_set.is_some())
    }

    fn weak_set_data(&self, set: &Handle) -> Result<&WeakSetData, Error> {
        self.inspect(set)?
            .weak_set
            .as_deref()
            .ok_or(Error::WrongKind)
    }

    pub(crate) fn weak_set_has(
        &self,
        set: &Handle,
        value: &Value,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let data = self.weak_set_data(set)?;
        let key = WeakKey::new(self, value)?;
        budget.charge(1)?;
        Ok(data.keys.contains(&key))
    }

    pub(crate) fn weak_set_delete(
        &mut self,
        set: &Handle,
        value: &Value,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        self.weak_set_data(set)?;
        let key = WeakKey::new(self, value)?;
        budget.charge(1)?;
        Ok(self
            .object_mut(set)?
            .weak_set
            .as_deref_mut()
            .expect("checked slot")
            .keys
            .remove(&key))
    }

    pub(crate) fn weak_set_add(
        &mut self,
        set: &Handle,
        value: &Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let data = self.weak_set_data(set)?;
        let key = WeakKey::new(self, value)?;
        budget.charge(1)?;
        if data.keys.contains(&key) {
            return Ok(());
        }
        let pruned = if data.insertions_before_prune == 0 {
            budget.charge(data.keys.len())?;
            let mut keys = HashSet::new();
            keys.try_reserve(data.keys.len())
                .map_err(|_| spite_heap::Error::Capacity)?;
            keys.extend(data.keys.iter().filter(|key| key.is_live(self)).cloned());
            Some(keys)
        } else {
            None
        };
        let data = self
            .object_mut(set)?
            .weak_set
            .as_deref_mut()
            .expect("checked slot");
        if let Some(mut keys) = pruned {
            keys.try_reserve(1)
                .map_err(|_| spite_heap::Error::Capacity)?;
            data.insertions_before_prune = keys.len().max(1);
            data.keys = keys;
        } else {
            data.keys
                .try_reserve(1)
                .map_err(|_| spite_heap::Error::Capacity)?;
        }
        data.keys.insert(key);
        data.insertions_before_prune -= 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spite_core::{JsString, JsSymbol};

    #[test]
    fn invalid_handles_and_exhausted_work_fail_before_storage_mutation() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let set = objects.create_weak_set(&prototype).unwrap();
        let stale = objects.create(None).unwrap();
        objects.collect([&set], usize::MAX).unwrap();
        let mut other = Objects::with_limits(None, None);
        let foreign = other.create(None).unwrap();
        let mut unlimited = Budget::with_work_limit(None);
        for bad in [stale, foreign] {
            assert!(matches!(
                objects.weak_set_add(&set, &Value::Object(bad.clone()), &mut unlimited),
                Err(Error::Heap(_))
            ));
            assert!(matches!(
                objects.weak_set_has(&set, &Value::Object(bad.clone()), &mut unlimited),
                Err(Error::Heap(_))
            ));
            assert!(matches!(
                objects.weak_set_delete(&set, &Value::Object(bad), &mut unlimited),
                Err(Error::Heap(_))
            ));
        }
        assert!(matches!(
            objects.weak_set_add(&prototype, &Value::Object(set.clone()), &mut unlimited),
            Err(Error::WrongKind)
        ));
        let first = objects.create(None).unwrap();
        let second = objects.create(None).unwrap();
        objects
            .weak_set_add(&set, &Value::Object(first.clone()), &mut unlimited)
            .unwrap();
        // The next insertion prunes; exhaustion must leave the old index and
        // cleanup countdown intact, including when the old key is still live.
        assert!(matches!(
            objects.weak_set_add(&set, &Value::Object(second.clone()), &mut Budget::new(1)),
            Err(Error::WorkLimit)
        ));
        assert_eq!(objects.weak_set_data(&set).unwrap().keys.len(), 1);
        assert!(
            objects
                .weak_set_has(&set, &Value::Object(first.clone()), &mut unlimited)
                .unwrap()
        );
        assert!(matches!(
            objects.weak_set_delete(&set, &Value::Object(first.clone()), &mut Budget::new(0)),
            Err(Error::WorkLimit)
        ));
        assert!(
            objects
                .weak_set_has(&set, &Value::Object(first), &mut unlimited)
                .unwrap()
        );
        assert!(
            !objects
                .weak_set_has(&set, &Value::Object(second), &mut unlimited)
                .unwrap()
        );
    }

    #[test]
    fn weak_keys_do_not_mark_objects_or_retain_symbols_and_pruning_reuses_storage() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let set = objects.create_weak_set(&prototype).unwrap();
        let key = objects.create(None).unwrap();
        let symbol = JsSymbol::new(Some(JsString::from("key")));
        let weak = symbol.downgrade();
        let mut unlimited = Budget::with_work_limit(None);
        objects
            .weak_set_add(&set, &Value::Object(key.clone()), &mut unlimited)
            .unwrap();
        objects
            .weak_set_add(&set, &Value::Symbol(symbol.clone()), &mut unlimited)
            .unwrap();
        assert!(objects.collect([&set], 1).is_err());
        assert!(objects.inspect(&key).is_ok());
        assert_eq!(objects.collect([&set], usize::MAX).unwrap().reclaimed, 1);
        drop(symbol);
        assert_eq!(weak.upgrade(), None);
        assert_eq!(objects.weak_set_data(&set).unwrap().keys.len(), 2);
        let next = objects.create(None).unwrap();
        assert_ne!(key, next);
        objects
            .weak_set_add(&set, &Value::Object(next.clone()), &mut unlimited)
            .unwrap();
        assert_eq!(objects.weak_set_data(&set).unwrap().keys.len(), 1);
        assert!(
            objects
                .weak_set_has(&set, &Value::Object(next), &mut unlimited)
                .unwrap()
        );
        assert_eq!(objects.collect([&set], usize::MAX).unwrap().reclaimed, 1);
        assert_eq!(objects.collect([], usize::MAX).unwrap().reclaimed, 2);
    }
}
