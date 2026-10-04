//! CanonicalizeKeyedCollectionKey and ordered, hash-indexed MapData (24.1).

use super::{Budget, Error, Objects, Value};
use spite_bigint::BigInt;
use spite_core::{JsString, JsSymbol};
use spite_heap::{Handle, Trace};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum CollectionKey {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    String(JsString),
    BigInt(BigInt),
    Symbol(JsSymbol),
    Object(Handle),
}

impl CollectionKey {
    pub(crate) fn new(value: &Value) -> Self {
        match value {
            Value::Undefined => Self::Undefined,
            Value::Null => Self::Null,
            Value::Boolean(value) => Self::Boolean(*value),
            Value::Number(value) => Self::Number(if value.is_nan() {
                f64::NAN.to_bits()
            } else if *value == 0.0 {
                0
            } else {
                value.to_bits()
            }),
            Value::String(value) => Self::String(value.clone()),
            Value::BigInt(value) => Self::BigInt(value.clone()),
            Value::Symbol(value) => Self::Symbol(value.clone()),
            Value::Object(value) => Self::Object(value.clone()),
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct MapData {
    // Stable positions preserve paused iterator/forEach cursors. Deletion and
    // clear erase values without shifting positions; reinsertion appends.
    entries: Vec<Option<(Value, Value)>>,
    index: HashMap<CollectionKey, usize>,
}

impl MapData {
    pub(super) fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(None).chain(self.entries.iter().flat_map(|entry| {
            let (key, value) = match entry {
                Some((key, value)) => {
                    (key.trace().next().flatten(), value.trace().next().flatten())
                }
                None => (None, None),
            };
            [key, value]
        }))
    }
}

impl Objects {
    pub(crate) fn create_map(&mut self, prototype: &Handle) -> Result<Handle, Error> {
        let map = self.create(Some(prototype))?;
        self.object_mut(&map)?.map = Some(Box::default());
        Ok(map)
    }

    fn map_data(&self, map: &Handle) -> Result<&MapData, Error> {
        self.inspect(map)?.map.as_deref().ok_or(Error::WrongKind)
    }

    fn map_data_mut(&mut self, map: &Handle) -> Result<&mut MapData, Error> {
        self.object_mut(map)?
            .map
            .as_deref_mut()
            .ok_or(Error::WrongKind)
    }

    fn map_key(&self, value: &Value, budget: &mut Budget) -> Result<CollectionKey, Error> {
        budget.charge(1)?;
        budget.value(value)?;
        if let Value::Object(object) = value {
            self.inspect(object)?;
        }
        Ok(CollectionKey::new(value))
    }

    pub(crate) fn map_has(
        &self,
        map: &Handle,
        key: &Value,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let data = self.map_data(map)?;
        let key = self.map_key(key, budget)?;
        Ok(data.index.contains_key(&key))
    }

    pub(crate) fn map_size(&self, map: &Handle, budget: &mut Budget) -> Result<usize, Error> {
        budget.charge(1)?;
        Ok(self.map_data(map)?.index.len())
    }

    pub(crate) fn map_get(
        &self,
        map: &Handle,
        key: &Value,
        budget: &mut Budget,
    ) -> Result<Option<Value>, Error> {
        let data = self.map_data(map)?;
        let key = self.map_key(key, budget)?;
        let Some(index) = data.index.get(&key) else {
            return Ok(None);
        };
        let (_, value) = data.entries[*index].as_ref().expect("live hash entry");
        budget.value(value)?;
        Ok(Some(value.clone()))
    }

    pub(crate) fn map_set(
        &mut self,
        map: &Handle,
        key: Value,
        value: Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        self.map_data(map)?;
        let index_key = self.map_key(&key, budget)?;
        budget.value(&value)?;
        if let Value::Object(object) = &value {
            self.inspect(object)?;
        }
        let data = self.map_data_mut(map)?;
        if let Some(index) = data.index.get(&index_key) {
            data.entries[*index].as_mut().expect("live hash entry").1 = value;
            return Ok(());
        }
        data.entries
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        data.index
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        let key = if matches!(key, Value::Number(number) if number == 0.0) {
            Value::Number(0.0)
        } else {
            key
        };
        data.index.insert(index_key, data.entries.len());
        data.entries.push(Some((key, value)));
        Ok(())
    }

    pub(crate) fn map_delete(
        &mut self,
        map: &Handle,
        key: &Value,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        self.map_data(map)?;
        let key = self.map_key(key, budget)?;
        let data = self.map_data_mut(map)?;
        let Some(index) = data.index.remove(&key) else {
            return Ok(false);
        };
        data.entries[index] = None;
        Ok(true)
    }

    pub(crate) fn map_clear(&mut self, map: &Handle, budget: &mut Budget) -> Result<(), Error> {
        budget.charge(self.map_data(map)?.entries.len())?;
        let data = self.map_data_mut(map)?;
        data.entries.iter_mut().for_each(|entry| *entry = None);
        data.index.clear();
        Ok(())
    }

    pub(crate) fn map_next(
        &self,
        map: &Handle,
        mut index: usize,
        budget: &mut Budget,
    ) -> Result<Option<(usize, Value, Value)>, Error> {
        let data = self.map_data(map)?;
        while let Some(entry) = data.entries.get(index) {
            budget.charge(1)?;
            index += 1;
            if let Some((key, value)) = entry {
                budget.value(key)?;
                budget.value(value)?;
                return Ok(Some((index, key.clone(), value.clone())));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::ArrayIterationKind;

    #[test]
    fn canonical_number_hashes_and_opted_in_failures_preserve_live_entries() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let map = objects.create_map(&prototype).unwrap();
        let mut budget = Budget::with_work_limit(None);
        for bits in [0x7ff8_0000_0000_0001, 0xfff0_0000_0000_0001] {
            objects
                .map_set(
                    &map,
                    Value::Number(f64::from_bits(bits)),
                    Value::Boolean(true),
                    &mut budget,
                )
                .unwrap();
        }
        objects
            .map_set(&map, Value::Number(-0.0), Value::Null, &mut budget)
            .unwrap();
        assert_eq!(objects.map_size(&map, &mut budget), Ok(2));
        assert_eq!(
            objects.map_get(&map, &Value::Number(f64::NAN), &mut budget),
            Ok(Some(Value::Boolean(true)))
        );
        assert!(matches!(
            objects.map_set(
                &map,
                Value::Number(0.0),
                Value::Boolean(false),
                &mut Budget::new(2)
            ),
            Err(Error::WorkLimit)
        ));
        assert!(matches!(
            objects.map_clear(&map, &mut Budget::new(1)),
            Err(Error::WorkLimit)
        ));
        assert_eq!(
            objects.map_get(&map, &Value::Number(0.0), &mut budget),
            Ok(Some(Value::Null))
        );
        let (_, key, _) = objects.map_next(&map, 1, &mut budget).unwrap().unwrap();
        assert!(matches!(key, Value::Number(number) if number.to_bits()==0));
    }

    #[test]
    fn foreign_or_stale_keys_and_values_fail_before_mutation() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let map = objects.create_map(&prototype).unwrap();
        let stale = objects.create(None).unwrap();
        objects.collect([&map], usize::MAX).unwrap();
        let mut other = Objects::with_limits(None, None);
        let foreign = other.create(None).unwrap();
        for bad in [stale, foreign] {
            for (key, value) in [
                (Value::Object(bad.clone()), Value::Null),
                (Value::Null, Value::Object(bad.clone())),
            ] {
                assert!(matches!(
                    objects.map_set(&map, key, value, &mut Budget::with_work_limit(None)),
                    Err(Error::Heap(_))
                ));
            }
        }
        assert_eq!(
            objects.map_size(&map, &mut Budget::with_work_limit(None)),
            Ok(0)
        );
    }

    #[test]
    fn map_and_iterator_slots_trace_and_release_sources_and_deleted_entries() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let map = objects.create_map(&prototype).unwrap();
        let key = objects.create(None).unwrap();
        let value = objects.create(None).unwrap();
        let mut budget = Budget::with_work_limit(None);
        objects
            .map_set(
                &map,
                Value::Object(key.clone()),
                Value::Object(value.clone()),
                &mut budget,
            )
            .unwrap();
        let iterator = objects
            .create_map_iterator(&prototype, &map, ArrayIterationKind::KeyValue)
            .unwrap();
        assert_eq!(objects.collect([&iterator], usize::MAX).unwrap().live, 5);
        objects
            .map_delete(&map, &Value::Object(key.clone()), &mut budget)
            .unwrap();
        assert_eq!(objects.collect([&iterator], usize::MAX).unwrap().live, 3);
        assert!(objects.inspect(&key).is_err());
        assert!(objects.inspect(&value).is_err());
        objects.update_map_iterator(&iterator, None).unwrap();
        assert_eq!(objects.collect([&iterator], usize::MAX).unwrap().live, 2);
        assert!(objects.inspect(&map).is_err());
    }
}
