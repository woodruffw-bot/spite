//! Ordered hash-indexed SetData with stable positions and canonical keys (24.2).

use super::{Budget, CollectionKey, Error, Objects, Value};
use spite_heap::{Handle, Trace};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(crate) struct SetData {
    entries: Vec<Option<Value>>,
    index: HashMap<CollectionKey, usize>,
}

impl SetData {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(None).chain(self.entries.iter().map(|entry| {
            entry
                .as_ref()
                .and_then(|value| value.trace().next().flatten())
        }))
    }

    pub(crate) fn size(&self) -> usize {
        self.index.len()
    }

    pub(crate) fn has(&self, value: &Value, budget: &mut Budget) -> Result<bool, Error> {
        budget.charge(1)?;
        budget.value(value)?;
        Ok(self.index.contains_key(&CollectionKey::new(value)))
    }

    pub(crate) fn insert(&mut self, value: Value, budget: &mut Budget) -> Result<(), Error> {
        budget.charge(1)?;
        budget.value(&value)?;
        let key = CollectionKey::new(&value);
        if self.index.contains_key(&key) {
            return Ok(());
        }
        self.entries
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        self.index
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        let value = if matches!(value, Value::Number(number) if number == 0.0) {
            Value::Number(0.0)
        } else {
            value
        };
        self.index.insert(key, self.entries.len());
        self.entries.push(Some(value));
        Ok(())
    }

    pub(crate) fn delete(&mut self, value: &Value, budget: &mut Budget) -> Result<bool, Error> {
        budget.charge(1)?;
        budget.value(value)?;
        let Some(index) = self.index.remove(&CollectionKey::new(value)) else {
            return Ok(false);
        };
        self.entries[index] = None;
        Ok(true)
    }

    pub(crate) fn clear(&mut self, budget: &mut Budget) -> Result<(), Error> {
        budget.charge(self.entries.len())?;
        self.entries.iter_mut().for_each(|entry| *entry = None);
        self.index.clear();
        Ok(())
    }

    pub(crate) fn next(
        &self,
        mut index: usize,
        budget: &mut Budget,
    ) -> Result<Option<(usize, Value)>, Error> {
        while let Some(entry) = self.entries.get(index) {
            budget.charge(1)?;
            index += 1;
            if let Some(value) = entry {
                budget.value(value)?;
                return Ok(Some((index, value.clone())));
            }
        }
        Ok(None)
    }

    pub(crate) fn copy(&self, budget: &mut Budget) -> Result<Self, Error> {
        let mut result = Self::default();
        result
            .entries
            .try_reserve(self.entries.len())
            .map_err(|_| spite_heap::Error::Capacity)?;
        result
            .index
            .try_reserve(self.index.len())
            .map_err(|_| spite_heap::Error::Capacity)?;
        for entry in &self.entries {
            budget.charge(1)?;
            if let Some(value) = entry {
                budget.value(value)?;
                result
                    .index
                    .insert(CollectionKey::new(value), result.entries.len());
            }
            result.entries.push(entry.clone());
        }
        Ok(result)
    }
}

impl Objects {
    pub(crate) fn create_set(
        &mut self,
        prototype: &Handle,
        data: SetData,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        budget.charge(data.entries.len())?;
        for value in data.entries.iter().flatten() {
            if let Value::Object(object) = value {
                self.inspect(object)?;
            }
        }
        let set = self.create(Some(prototype))?;
        self.object_mut(&set)?.set = Some(Box::new(data));
        Ok(set)
    }

    pub(crate) fn set_data(&self, set: &Handle) -> Result<&SetData, Error> {
        self.inspect(set)?.set.as_deref().ok_or(Error::WrongKind)
    }

    pub(crate) fn set_data_mut(&mut self, set: &Handle) -> Result<&mut SetData, Error> {
        self.object_mut(set)?
            .set
            .as_deref_mut()
            .ok_or(Error::WrongKind)
    }

    pub(crate) fn set_insert(
        &mut self,
        set: &Handle,
        value: Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        self.set_data(set)?;
        if let Value::Object(object) = &value {
            self.inspect(object)?;
        }
        self.set_data_mut(set)?.insert(value, budget)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ArrayIterationKind;

    #[test]
    fn snapshots_preserve_holes_and_identity_without_retaining_deleted_values() {
        let mut budget = Budget::with_work_limit(None);
        let mut data = SetData::default();
        data.insert(Value::Number(1.0), &mut budget).unwrap();
        data.insert(Value::Number(2.0), &mut budget).unwrap();
        data.delete(&Value::Number(1.0), &mut budget).unwrap();
        data.insert(Value::Number(1.0), &mut budget).unwrap();
        let copy = data.copy(&mut budget).unwrap();
        assert_eq!(copy.next(0, &mut budget), Ok(Some((2, Value::Number(2.0)))));
        assert_eq!(copy.next(2, &mut budget), Ok(Some((3, Value::Number(1.0)))));
        assert_eq!(copy.size(), 2);
        assert!(matches!(
            data.clear(&mut Budget::new(2)),
            Err(Error::WorkLimit)
        ));
        assert!(matches!(
            data.insert(Value::Number(3.0), &mut Budget::new(1)),
            Err(Error::WorkLimit)
        ));
        assert_eq!(data.size(), 2);
    }

    #[test]
    fn foreign_and_stale_values_fail_before_set_mutation() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let mut budget = Budget::with_work_limit(None);
        let set = objects
            .create_set(&prototype, SetData::default(), &mut budget)
            .unwrap();
        let stale = objects.create(None).unwrap();
        objects.collect([&set], usize::MAX).unwrap();
        let mut other = Objects::with_limits(None, None);
        let foreign = other.create(None).unwrap();
        for bad in [stale, foreign] {
            assert!(matches!(
                objects.set_insert(&set, Value::Object(bad), &mut budget),
                Err(Error::Heap(_))
            ));
        }
        assert_eq!(objects.set_data(&set).unwrap().size(), 0);
    }

    #[test]
    fn set_and_iterator_slots_trace_and_release_sources_and_erased_values() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let mut budget = Budget::with_work_limit(None);
        let set = objects
            .create_set(&prototype, SetData::default(), &mut budget)
            .unwrap();
        let value = objects.create(None).unwrap();
        objects
            .set_insert(&set, Value::Object(value.clone()), &mut budget)
            .unwrap();
        let iterator = objects
            .create_set_iterator(&prototype, &set, ArrayIterationKind::Value)
            .unwrap();
        assert_eq!(objects.collect([&iterator], usize::MAX).unwrap().live, 4);
        objects
            .set_data_mut(&set)
            .unwrap()
            .clear(&mut budget)
            .unwrap();
        assert_eq!(objects.collect([&iterator], usize::MAX).unwrap().live, 3);
        assert!(objects.inspect(&value).is_err());
        objects.update_set_iterator(&iterator, None).unwrap();
        assert_eq!(objects.collect([&iterator], usize::MAX).unwrap().live, 2);
        assert!(objects.inspect(&set).is_err());
    }
}
