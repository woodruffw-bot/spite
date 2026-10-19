//! Object-key WeakMapData with conditional values and dense cleanup (24.3).
//!
//! Object keys use existing ephemerons. Valid weak Symbol keys remain an explicit
//! Realm Unsupported boundary; the Symbol surface is a separate increment.

use super::{Budget, Error, Objects};
use crate::Value;
use spite_core::{JsSymbol, WeakJsSymbol};
use spite_heap::{Handle, Trace};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(super) struct WeakMapData {
    // WeakMap has no observable iteration order. Dense entries keep each trace
    // step bounded and let cleanup release values without allocating or leaving
    // tombstones that an iterator would have to skip.
    pub(super) entries: Vec<(Handle, Value)>,
    pub(super) index: HashMap<Handle, usize>,
    // Keep the existing object-key API and GC tests independent. Both domains
    // participate in the Runtime collector's one joint conditional fixpoint.
    pub(super) symbol_entries: Vec<(WeakJsSymbol, Value)>,
    pub(super) symbol_index: HashMap<WeakJsSymbol, usize>,
}

impl WeakMapData {
    pub(super) fn ephemerons(&self) -> impl Iterator<Item = (&Handle, Option<&Handle>)> {
        self.entries
            .iter()
            .map(|(key, value)| (key, value.trace().next().flatten()))
    }

    pub(super) fn remove_at(&mut self, index: usize) {
        let (key, value) = self.entries.swap_remove(index);
        self.index.remove(&key);
        if let Some((moved, _)) = self.entries.get(index) {
            *self.index.get_mut(moved).expect("moved entry is indexed") = index;
        }
        drop((key, value));
    }

    pub(super) fn remove_symbol_at(&mut self, index: usize) {
        let (key, value) = self.symbol_entries.swap_remove(index);
        self.symbol_index.remove(&key);
        if let Some((moved, _)) = self.symbol_entries.get(index) {
            *self
                .symbol_index
                .get_mut(moved)
                .expect("moved entry is indexed") = index;
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

impl Objects {
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Symbol surface is a separate increment")
    )]
    pub(crate) fn weak_map_has_symbol(
        &self,
        map: &Handle,
        key: &JsSymbol,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let data = self.weak_map_data(map)?;
        budget.charge(1)?;
        Ok(data.symbol_index.contains_key(&key.downgrade()))
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Symbol surface is a separate increment")
    )]
    pub(crate) fn weak_map_get_symbol(
        &self,
        map: &Handle,
        key: &JsSymbol,
        budget: &mut Budget,
    ) -> Result<Option<Value>, Error> {
        let data = self.weak_map_data(map)?;
        budget.charge(1)?;
        let Some(index) = data.symbol_index.get(&key.downgrade()) else {
            return Ok(None);
        };
        let value = &data.symbol_entries[*index].1;
        budget.value(value)?;
        Ok(Some(value.clone()))
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Symbol surface is a separate increment")
    )]
    pub(crate) fn weak_map_set_symbol(
        &mut self,
        map: &Handle,
        key: &JsSymbol,
        value: Value,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        self.weak_map_data(map)?;
        if let Value::Object(object) = &value {
            self.inspect(object)?;
        }
        budget.charge(1)?;
        budget.value(&value)?;
        let key = key.downgrade();
        let data = self.weak_map_data_mut(map)?;
        if let Some(index) = data.symbol_index.get(&key) {
            data.symbol_entries[*index].1 = value;
            self.symbol_keys_ever_stored = true;
            return Ok(());
        }
        if data.symbol_entries.len() == data.symbol_entries.capacity() {
            budget.charge(data.symbol_entries.len())?;
        }
        if data.symbol_index.len() == data.symbol_index.capacity() {
            budget.charge(data.symbol_index.len())?;
        }
        data.symbol_entries
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        data.symbol_index
            .try_reserve(1)
            .map_err(|_| spite_heap::Error::Capacity)?;
        data.symbol_index
            .insert(key.clone(), data.symbol_entries.len());
        data.symbol_entries.push((key, value));
        self.symbol_keys_ever_stored = true;
        Ok(())
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "Symbol surface is a separate increment")
    )]
    pub(crate) fn weak_map_delete_symbol(
        &mut self,
        map: &Handle,
        key: &JsSymbol,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        self.weak_map_data(map)?;
        budget.charge(1)?;
        let data = self.weak_map_data_mut(map)?;
        let Some(index) = data.symbol_index.get(&key.downgrade()).copied() else {
            return Ok(false);
        };
        data.remove_symbol_at(index);
        Ok(true)
    }

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
        // Reserve/copy work must finish before entries or retained values
        // change. An overwrite above performs no growth or index movement.
        if data.entries.len() == data.entries.capacity() {
            budget.charge(data.entries.len())?;
        }
        if data.index.len() == data.index.capacity() {
            budget.charge(data.index.len())?;
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
