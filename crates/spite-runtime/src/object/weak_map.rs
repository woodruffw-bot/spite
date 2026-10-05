//! Object-key WeakMapData with conditional values and dense cleanup (24.3).
//!
//! JavaScript intrinsics and Symbol-key reachability are separate increments.

use super::{Budget, Error, Objects};
use crate::Value;
use spite_heap::{Handle, Trace};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(super) struct WeakMapData {
    // WeakMap has no observable iteration order. Dense entries keep each trace
    // step bounded and let cleanup release values without allocating or leaving
    // tombstones that an iterator would have to skip.
    entries: Vec<(Handle, Value)>,
    index: HashMap<Handle, usize>,
}

impl WeakMapData {
    pub(super) fn ephemerons(&self) -> impl Iterator<Item = (&Handle, Option<&Handle>)> {
        self.entries
            .iter()
            .map(|(key, value)| (key, value.trace().next().flatten()))
    }

    fn remove_at(&mut self, index: usize) {
        let (key, value) = self.entries.swap_remove(index);
        self.index.remove(&key);
        if let Some((moved, _)) = self.entries.get(index) {
            *self.index.get_mut(moved).expect("moved entry is indexed") = index;
        }
        drop((key, value));
    }

    pub(super) fn retain(&mut self, retain: impl Fn(&Handle) -> bool) {
        let mut index = 0;
        while let Some((key, _)) = self.entries.get(index) {
            if retain(key) {
                index += 1;
            } else {
                self.remove_at(index);
            }
        }
    }
}

// Keep the checked storage private until Symbol-key tracing permits exposing
// WeakMap's JavaScript intrinsics with the complete CanBeHeldWeakly domain.
#[allow(dead_code, reason = "WeakMap intrinsics await Symbol-key reachability")]
impl Objects {
    pub(crate) fn create_weak_map(&mut self, prototype: &Handle) -> Result<Handle, Error> {
        let map = self.create(Some(prototype))?;
        self.object_mut(&map)?.weak_map = Some(Box::default());
        Ok(map)
    }

    pub(crate) fn weak_map_has_slot(&self, map: &Handle) -> Result<bool, Error> {
        Ok(self.inspect(map)?.weak_map.is_some())
    }

    fn weak_map_data(&self, map: &Handle) -> Result<&WeakMapData, Error> {
        self.inspect(map)?
            .weak_map
            .as_deref()
            .ok_or(Error::WrongKind)
    }

    fn weak_map_data_mut(&mut self, map: &Handle) -> Result<&mut WeakMapData, Error> {
        self.object_mut(map)?
            .weak_map
            .as_deref_mut()
            .ok_or(Error::WrongKind)
    }

    pub(crate) fn weak_map_has(
        &self,
        map: &Handle,
        key: &Handle,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let data = self.weak_map_data(map)?;
        self.inspect(key)?;
        budget.charge(1)?;
        Ok(data.index.contains_key(key))
    }

    pub(crate) fn weak_map_get(
        &self,
        map: &Handle,
        key: &Handle,
        budget: &mut Budget,
    ) -> Result<Option<Value>, Error> {
        let data = self.weak_map_data(map)?;
        self.inspect(key)?;
        budget.charge(1)?;
        let Some(index) = data.index.get(key) else {
            return Ok(None);
        };
        let value = &data.entries[*index].1;
        budget.value(value)?;
        Ok(Some(value.clone()))
    }

    pub(crate) fn weak_map_set(
        &mut self,
        map: &Handle,
        key: &Handle,
        value: Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        self.weak_map_data(map)?;
        self.inspect(key)?;
        if let Value::Object(object) = &value {
            self.inspect(object)?;
        }
        budget.charge(1)?;
        budget.value(&value)?;
        let data = self.weak_map_data_mut(map)?;
        if let Some(index) = data.index.get(key) {
            data.entries[*index].1 = value;
            return Ok(());
        }
        data.entries
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        data.index
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        data.index.insert(key.clone(), data.entries.len());
        data.entries.push((key.clone(), value));
        Ok(())
    }

    pub(crate) fn weak_map_delete(
        &mut self,
        map: &Handle,
        key: &Handle,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        self.weak_map_data(map)?;
        self.inspect(key)?;
        budget.charge(1)?;
        let data = self.weak_map_data_mut(map)?;
        let Some(index) = data.index.get(key).copied() else {
            return Ok(false);
        };
        data.remove_at(index);
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
