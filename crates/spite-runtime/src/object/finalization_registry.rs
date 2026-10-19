//! FinalizationRegistry cells and prepared cleanup captures (26.2, 9.12).
//!
//! Storage never calls JavaScript. The Realm validates language-level key and
//! SameValue constraints, and prepares its real call argument vector before
//! committing a cell. Queue captures are explicit Objects collection roots.

use super::{Budget, Error, Objects};
use crate::Value;
use spite_core::WeakJsSymbol;
use spite_heap::{Handle, Trace};

#[derive(Debug)]
pub(super) enum WeakTarget {
    Object(Handle),
    Symbol(WeakJsSymbol),
}

impl WeakTarget {
    fn from_value(objects: &Objects, value: &Value) -> Result<Self, Error> {
        match value {
            Value::Object(value) => {
                objects.inspect(value)?;
                Ok(Self::Object(value.clone()))
            }
            Value::Symbol(value) => Ok(Self::Symbol(value.downgrade())),
            _ => Err(Error::WrongKind),
        }
    }

    fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Object(left), Self::Object(right)) => left == right,
            (Self::Symbol(left), Self::Symbol(right)) => left == right,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub(super) struct Cell {
    // None is a cleared target. The unregister token remains weak and distinct.
    pub(super) target: Option<WeakTarget>,
    pub(super) held: Value,
    pub(super) token: Option<WeakTarget>,
}

#[derive(Debug)]
pub(super) struct FinalizationRegistryData {
    pub(super) callback: Handle,
    pub(super) cells: Vec<Cell>,
    // Maintained with cell mutation, allowing infallible post-abort completion.
    pub(super) cleared: usize,
    pub(super) pending: bool,
}

impl Trace for FinalizationRegistryData {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        std::iter::once(Some(&self.callback)).chain(self.cells.iter().map(
            |cell| match &cell.held {
                Value::Object(value) => Some(value),
                _ => None,
            },
        ))
    }
}

/// An opaque checked queue identity. Its capture stays queued across Call.
#[derive(Debug)]
pub(crate) struct FinalizationCleanupJob {
    index: usize,
    registry: Handle,
}

/// All fallible cell selection is complete; no JavaScript may run before commit.
#[derive(Debug)]
pub(crate) struct FinalizationCleanupCell {
    job_index: usize,
    registry: Handle,
    index: usize,
    callback: Handle,
    held_root: Option<Handle>,
}

#[derive(Debug)]
pub(crate) struct FinalizationCleanupCall {
    pub(crate) callback: Handle,
    pub(crate) held: Value,
}

impl Objects {
    pub(crate) fn create_finalization_registry(
        &mut self,
        prototype: &Handle,
        callback: &Handle,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        self.inspect(prototype)?;
        if !self.inspect(callback)?.is_callable() {
            return Err(Error::NotCallable);
        }
        budget.charge(1)?;
        // Like the existing WeakRef slot, box the optional record so ordinary
        // objects do not carry an inline cells Vec and cleanup counters. This
        // allocation precedes heap insertion and any collector-state mutation.
        let data = Box::new(FinalizationRegistryData {
            callback: callback.clone(),
            cells: Vec::new(),
            cleared: 0,
            pending: false,
        });
        let registry = self.create(Some(prototype))?;
        self.object_mut(&registry)?.finalization_registry = Some(data);
        // Only a successfully installed registry changes collector selection.
        self.finalization_registries_ever_stored = true;
        Ok(registry)
    }

    pub(crate) fn finalization_registry_has_slot(&self, registry: &Handle) -> Result<bool, Error> {
        Ok(self.inspect(registry)?.finalization_registry.is_some())
    }

    pub(crate) fn finalization_registry_register(
        &mut self,
        registry: &Handle,
        target: &Value,
        held: Value,
        token: Option<&Value>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let data = self
            .inspect(registry)?
            .finalization_registry
            .as_ref()
            .ok_or(Error::WrongKind)?;
        // Charge identity copies and possible existing-vector movement before
        // making even weak identity temporaries or changing the record.
        budget.charge(2 + usize::from(token.is_some()))?;
        if data.cells.len() == data.cells.capacity() {
            budget.charge(data.cells.len())?;
        }
        let target = WeakTarget::from_value(self, target)?;
        let token = token
            .map(|token| WeakTarget::from_value(self, token))
            .transpose()?;
        if let Value::Object(held) = &held {
            self.inspect(held)?;
        }
        let data = self
            .object_mut(registry)?
            .finalization_registry
            .as_mut()
            .expect("validated registry");
        data.cells
            .try_reserve(1)
            .map_err(|_| Error::Heap(spite_heap::Error::Capacity))?;
        data.cells.push(Cell {
            target: Some(target),
            held,
            token,
        });
        Ok(())
    }

    pub(crate) fn finalization_registry_unregister(
        &mut self,
        registry: &Handle,
        token: &Value,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let data = self
            .inspect(registry)?
            .finalization_registry
            .as_ref()
            .ok_or(Error::WrongKind)?;
        budget.charge(data.cells.len().checked_mul(2).ok_or(Error::WorkLimit)?)?;
        budget.charge(1)?;
        let token = WeakTarget::from_value(self, token)?;
        let count = data
            .cells
            .iter()
            .filter(|cell| {
                cell.token
                    .as_ref()
                    .is_some_and(|stored| stored.same(&token))
            })
            .count();
        if count == 0 {
            return Ok(false);
        }
        let mut removed = Vec::new();
        removed
            .try_reserve_exact(count)
            .map_err(|_| Error::Heap(spite_heap::Error::Capacity))?;
        {
            let data = self
                .object_mut(registry)?
                .finalization_registry
                .as_mut()
                .expect("validated registry");
            for index in (0..data.cells.len()).rev() {
                if data.cells[index]
                    .token
                    .as_ref()
                    .is_some_and(|stored| stored.same(&token))
                {
                    let cell = data.cells.swap_remove(index);
                    if cell.target.is_none() {
                        data.cleared -= 1;
                    }
                    removed.push(cell);
                }
            }
        }
        // Held Values and weak allocations can run destructors: release the
        // record borrow first, and never perform this operation under a gate.
        drop(removed);
        Ok(true)
    }

    pub(crate) fn has_finalization_cleanup_job(&self) -> bool {
        !self.finalization_jobs.is_empty()
    }

    pub(crate) fn prepare_finalization_cleanup_job(
        &self,
        budget: &mut Budget,
    ) -> Result<Option<FinalizationCleanupJob>, Error> {
        let Some((index, registry)) = self.finalization_jobs.iter().enumerate().next_back() else {
            return Ok(None);
        };
        budget.charge(1)?;
        let data = self
            .inspect(registry)?
            .finalization_registry
            .as_ref()
            .ok_or(Error::WrongKind)?;
        if !data.pending {
            return Err(Error::Heap(spite_heap::Error::CollectionState));
        }
        Ok(Some(FinalizationCleanupJob {
            index,
            registry: registry.clone(),
        }))
    }

    pub(crate) fn prepare_finalization_cleanup_cell(
        &self,
        job: &FinalizationCleanupJob,
        budget: &mut Budget,
    ) -> Result<Option<FinalizationCleanupCell>, Error> {
        if self.finalization_jobs.get(job.index) != Some(&job.registry) {
            return Err(Error::Heap(spite_heap::Error::CollectionState));
        }
        let data = self
            .inspect(&job.registry)?
            .finalization_registry
            .as_ref()
            .ok_or(Error::WrongKind)?;
        if data.cleared == 0 {
            return Ok(None);
        }
        let mut selected = None;
        for (index, cell) in data.cells.iter().enumerate().rev() {
            budget.charge(1)?;
            if cell.target.is_none() {
                selected = Some((index, cell));
                break;
            }
        }
        let (index, cell) = selected.ok_or(Error::Heap(spite_heap::Error::CollectionState))?;
        // No held Value copy is needed. These checked Handles need no new Rc
        // root registration and cannot keep an otherwise dead registry alive.
        budget.charge(4)?;
        let held_root = match &cell.held {
            Value::Object(value) => Some(value.clone()),
            _ => None,
        };
        Ok(Some(FinalizationCleanupCell {
            job_index: job.index,
            registry: job.registry.clone(),
            index,
            callback: data.callback.clone(),
            held_root,
        }))
    }

    /// No JavaScript or cell mutation may intervene after prepare. The Realm
    /// has already reserved its argument vector and checked argument limits.
    pub(crate) fn commit_finalization_cleanup_cell(
        &mut self,
        prepared: FinalizationCleanupCell,
    ) -> super::FinalizationCleanupCall {
        assert_eq!(
            self.finalization_jobs.get(prepared.job_index),
            Some(&prepared.registry)
        );
        assert!(self.finalization_active_held.is_none());
        // Establish the strong argument edge before removing its old cell edge.
        self.finalization_active_held = prepared.held_root;
        let cell = {
            let data = self
                .object_mut(&prepared.registry)
                .expect("queued registry")
                .finalization_registry
                .as_mut()
                .expect("registry slot");
            assert!(data.cells[prepared.index].target.is_none());
            data.cleared -= 1;
            data.cells.swap_remove(prepared.index)
        };
        let Cell {
            target,
            held,
            token,
        } = cell;
        drop((target, token));
        FinalizationCleanupCall {
            callback: prepared.callback,
            held,
        }
    }

    pub(crate) fn finish_finalization_cleanup_argument(&mut self) {
        let root = self.finalization_active_held.take();
        drop(root);
    }

    /// Called after releasing the active argument, on success or any abort.
    /// Nested job draining is forbidden; collection only appends queue entries.
    pub(crate) fn finish_finalization_cleanup_job(&mut self, job: FinalizationCleanupJob) {
        assert_eq!(self.finalization_jobs.get(job.index), Some(&job.registry));
        assert!(self.finalization_active_held.is_none());
        let data = self
            .object_mut(&job.registry)
            .expect("queued registry")
            .finalization_registry
            .as_mut()
            .expect("registry slot");
        if data.cleared != 0 {
            return;
        }
        data.pending = false;
        let capture = self.finalization_jobs.swap_remove(job.index);
        drop(capture);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::function::Builtin;
    use crate::object::DataDescriptor;
    use spite_core::{JsSymbol, WeakJsSymbol};
    use std::{cell::RefCell, rc::Rc};

    fn unlimited() -> Budget {
        Budget::with_work_limit(None)
    }

    fn fixture() -> (Objects, Handle, Handle, Handle) {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let callback = objects.create_builtin(&prototype, Builtin::Object).unwrap();
        let registry = objects
            .create_finalization_registry(&prototype, &callback, &mut unlimited())
            .unwrap();
        (objects, prototype, callback, registry)
    }

    fn data(value: Value) -> DataDescriptor {
        DataDescriptor {
            value: Some(value),
            writable: Some(true),
            enumerable: Some(true),
            configurable: Some(true),
        }
    }

    fn state(objects: &Objects, registry: &Handle) -> (usize, usize, bool) {
        let data = objects
            .inspect(registry)
            .unwrap()
            .finalization_registry
            .as_ref()
            .unwrap();
        (data.cells.len(), data.cleared, data.pending)
    }

    fn stale(objects: &Objects, handle: &Handle) {
        assert!(matches!(
            objects.inspect(handle),
            Err(Error::Heap(spite_heap::Error::StaleHandle))
        ));
    }

    #[test]
    fn held_symbol_and_callback_object_graphs_trace_without_symbol_weakmap_history() {
        let (mut objects, _, callback, registry) = fixture();
        let held_symbol = JsSymbol::new(None);
        let held_weak = held_symbol.downgrade();
        let target = objects.create(None).unwrap();
        objects
            .define(
                &callback,
                "target",
                data(Value::Object(target.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Object(target.clone()),
                Value::Symbol(held_symbol),
                None,
                &mut unlimited(),
            )
            .unwrap();
        assert!(!objects.symbol_keys_ever_stored);
        objects.collect([&registry], usize::MAX).unwrap();
        assert_eq!(state(&objects, &registry), (1, 0, false));
        assert!(held_weak.upgrade().is_some());
        assert!(objects.inspect(&target).is_ok());
        assert!(!objects.has_finalization_cleanup_job());
        objects.collect([], usize::MAX).unwrap();
        stale(&objects, &registry);
        stale(&objects, &callback);
        stale(&objects, &target);
        assert!(held_weak.upgrade().is_none());
        assert!(!objects.has_finalization_cleanup_job());
    }

    #[test]
    fn reached_held_symbols_activate_weakmaps_but_dead_registry_cycles_do_not_root_themselves() {
        for root_registry in [false, true] {
            let (mut objects, prototype, _, registry) = fixture();
            let map = objects.create_weak_map(&prototype).unwrap();
            let key = JsSymbol::new(None);
            let weak = key.downgrade();
            let target = objects.create(None).unwrap();
            objects
                .finalization_registry_register(
                    &registry,
                    &Value::Object(target.clone()),
                    Value::Symbol(key.clone()),
                    None,
                    &mut unlimited(),
                )
                .unwrap();
            // The only owning key is held in the registry. The conditional
            // value points back to it and the target; this cycle cannot root
            // itself, but becomes live from a real registry root.
            let value = objects.create(None).unwrap();
            objects
                .define(
                    &value,
                    "registry",
                    data(Value::Object(registry.clone())),
                    &mut unlimited(),
                )
                .unwrap();
            objects
                .define(
                    &value,
                    "target",
                    data(Value::Object(target.clone())),
                    &mut unlimited(),
                )
                .unwrap();
            objects
                .weak_map_set_symbol(&map, &key, Value::Object(value), &mut unlimited())
                .unwrap();
            drop(key);
            if root_registry {
                objects.collect([&map, &registry], usize::MAX).unwrap();
                assert!(weak.upgrade().is_some());
                assert!(objects.inspect(&target).is_ok());
                assert_eq!(state(&objects, &registry), (1, 0, false));
            } else {
                objects.collect([&map], usize::MAX).unwrap();
                assert!(weak.upgrade().is_none());
                stale(&objects, &registry);
                stale(&objects, &target);
                assert!(!objects.has_finalization_cleanup_job());
            }
        }
    }

    #[test]
    fn queued_capture_and_removed_held_argument_are_roots_in_objects_only_collection() {
        let (mut objects, _, callback, registry) = fixture();
        let target = objects.create(None).unwrap();
        let held = objects.create(None).unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Object(target.clone()),
                Value::Object(held.clone()),
                None,
                &mut unlimited(),
            )
            .unwrap();
        objects.collect([&registry], usize::MAX).unwrap();
        stale(&objects, &target);
        assert_eq!(state(&objects, &registry), (1, 1, true));
        objects.collect([], usize::MAX).unwrap();
        let job = objects
            .prepare_finalization_cleanup_job(&mut unlimited())
            .unwrap()
            .unwrap();
        let cell = objects
            .prepare_finalization_cleanup_cell(&job, &mut unlimited())
            .unwrap()
            .unwrap();
        let call = objects.commit_finalization_cleanup_cell(cell);
        assert_eq!(call.callback, callback);
        assert_eq!(call.held, Value::Object(held.clone()));
        assert_eq!(state(&objects, &registry), (0, 0, true));
        // Deterministic private phase: there is no stored cell or Rc root token
        // for held, and no public JavaScript collector/callback hook is needed.
        objects.collect([], usize::MAX).unwrap();
        assert!(objects.inspect(&held).is_ok());
        assert!(objects.inspect(&callback).is_ok());
        drop(call);
        objects.finish_finalization_cleanup_argument();
        objects.finish_finalization_cleanup_job(job);
        assert!(!objects.has_finalization_cleanup_job());
        objects.collect([], usize::MAX).unwrap();
        stale(&objects, &held);
        stale(&objects, &registry);
        stale(&objects, &callback);
    }

    #[test]
    fn unregister_removes_all_matching_cleared_cells_and_token_death_is_not_unregister() {
        let (mut objects, _, _, registry) = fixture();
        let token = JsSymbol::new(None);
        let different = JsSymbol::new(None);
        for held in [1.0, 2.0, 3.0] {
            let target = objects.create(None).unwrap();
            objects
                .finalization_registry_register(
                    &registry,
                    &Value::Object(target),
                    Value::Number(held),
                    Some(&Value::Symbol(token.clone())),
                    &mut unlimited(),
                )
                .unwrap();
        }
        let target = objects.create(None).unwrap();
        let dead_token = objects.create(None).unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Object(target),
                Value::Number(4.0),
                Some(&Value::Object(dead_token.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects.collect([&registry], usize::MAX).unwrap();
        stale(&objects, &dead_token);
        assert_eq!(state(&objects, &registry), (4, 4, true));
        assert!(
            !objects
                .finalization_registry_unregister(
                    &registry,
                    &Value::Symbol(different),
                    &mut unlimited()
                )
                .unwrap()
        );
        assert!(
            objects
                .finalization_registry_unregister(
                    &registry,
                    &Value::Symbol(token.clone()),
                    &mut unlimited()
                )
                .unwrap()
        );
        assert!(
            !objects
                .finalization_registry_unregister(
                    &registry,
                    &Value::Symbol(token),
                    &mut unlimited()
                )
                .unwrap()
        );
        assert_eq!(state(&objects, &registry), (1, 1, true));
        let job = objects
            .prepare_finalization_cleanup_job(&mut unlimited())
            .unwrap()
            .unwrap();
        let cell = objects
            .prepare_finalization_cleanup_cell(&job, &mut unlimited())
            .unwrap()
            .unwrap();
        let call = objects.commit_finalization_cleanup_cell(cell);
        assert_eq!(call.held, Value::Number(4.0));
        objects.finish_finalization_cleanup_argument();
        objects.finish_finalization_cleanup_job(job);
        assert!(!objects.has_finalization_cleanup_job());
    }

    #[test]
    fn target_as_token_is_weak_and_allocated_replacement_does_not_revive_a_cell() {
        let (mut objects, _, _, registry) = fixture();
        let target = objects.create(None).unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Object(target.clone()),
                Value::Number(1.0),
                Some(&Value::Object(target.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects.collect([&registry], usize::MAX).unwrap();
        stale(&objects, &target);
        let replacement = objects.create(None).unwrap();
        assert_ne!(replacement, target);
        objects
            .collect([&registry, &replacement], usize::MAX)
            .unwrap();
        assert_eq!(state(&objects, &registry), (1, 1, true));
        assert_eq!(objects.finalization_jobs.len(), 1);
        assert!(objects.inspect(&replacement).is_ok());
    }

    #[test]
    fn preparing_a_cell_can_fail_without_removal_and_finishing_an_aborted_call_keeps_the_same_job()
    {
        let (mut objects, _, _, registry) = fixture();
        for held in [1.0, 2.0, 3.0] {
            let target = objects.create(None).unwrap();
            objects
                .finalization_registry_register(
                    &registry,
                    &Value::Object(target),
                    Value::Number(held),
                    None,
                    &mut unlimited(),
                )
                .unwrap();
        }
        objects.collect([&registry], usize::MAX).unwrap();
        let job = objects
            .prepare_finalization_cleanup_job(&mut unlimited())
            .unwrap()
            .unwrap();
        assert!(matches!(
            objects.prepare_finalization_cleanup_cell(&job, &mut Budget::new(0)),
            Err(Error::WorkLimit)
        ));
        assert_eq!(state(&objects, &registry), (3, 3, true));
        let cell = objects
            .prepare_finalization_cleanup_cell(&job, &mut unlimited())
            .unwrap()
            .unwrap();
        let failed_call = objects.commit_finalization_cleanup_cell(cell);
        let failed_held = failed_call.held;
        objects.finish_finalization_cleanup_argument();
        objects.finish_finalization_cleanup_job(job);
        assert_eq!(state(&objects, &registry), (2, 2, true));
        assert_eq!(objects.finalization_jobs.len(), 1);
        let mut later = Vec::new();
        let job = objects
            .prepare_finalization_cleanup_job(&mut unlimited())
            .unwrap()
            .unwrap();
        while let Some(cell) = objects
            .prepare_finalization_cleanup_cell(&job, &mut unlimited())
            .unwrap()
        {
            later.push(objects.commit_finalization_cleanup_cell(cell).held);
            objects.finish_finalization_cleanup_argument();
        }
        assert_eq!(later.len(), 2);
        assert!(!later.contains(&failed_held));
        objects.finish_finalization_cleanup_job(job);
        assert!(!objects.has_finalization_cleanup_job());
    }

    fn symbol_target_fixture() -> (Objects, Handle, Handle, WeakJsSymbol) {
        let (mut objects, _, _, registry) = fixture();
        let target = JsSymbol::new(None);
        let weak = target.downgrade();
        let owner = objects.create(None).unwrap();
        objects
            .define(
                &owner,
                "symbol",
                data(Value::Symbol(target.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Symbol(target),
                Value::Number(1.0),
                None,
                &mut unlimited(),
            )
            .unwrap();
        (objects, registry, owner, weak)
    }

    #[test]
    fn failing_collection_prefixes_preserve_targets_cells_queue_and_generations() {
        let (mut calibration, registry, _, _) = symbol_target_fixture();
        let complete = calibration
            .collect([&registry], usize::MAX)
            .unwrap()
            .work_used;
        for allowance in [0, 1, complete / 2, complete - 1] {
            let (mut objects, registry, owner, weak) = symbol_target_fixture();
            assert_eq!(
                objects.collect([&registry], allowance),
                Err(spite_heap::Error::Limit)
            );
            assert!(weak.upgrade().is_some());
            assert!(objects.inspect(&owner).is_ok());
            assert_eq!(state(&objects, &registry), (1, 0, false));
            assert!(!objects.has_finalization_cleanup_job());
            objects.collect([&registry], usize::MAX).unwrap();
            assert!(weak.upgrade().is_none());
            stale(&objects, &owner);
            assert_eq!(state(&objects, &registry), (1, 1, true));
        }
    }

    #[test]
    fn logical_symbol_death_is_visible_after_unlock_before_cell_and_owner_drops() {
        let (mut objects, registry, owner, weak) = symbol_target_fixture();
        let observed = Rc::new(RefCell::new(None));
        let sink = observed.clone();
        let key = weak.clone();
        Objects::with_symbol_gc_test_hooks(
            None,
            Some(Box::new(move || {
                *sink.borrow_mut() = Some(key.upgrade().is_none());
            })),
            || objects.collect([&registry], usize::MAX),
        )
        .unwrap();
        assert_eq!(*observed.borrow(), Some(true));
        assert!(weak.upgrade().is_none());
        stale(&objects, &owner);
        assert_eq!(state(&objects, &registry), (1, 1, true));
    }

    #[test]
    fn registry_weakref_and_weakmap_share_symbol_death_while_a_real_host_owner_prevents_it() {
        let (mut objects, prototype, _, registry) = fixture();
        let target = JsSymbol::new(None);
        let weak = target.downgrade();
        let reference = objects
            .create_weak_ref(&prototype, &Value::Symbol(target.clone()), &mut unlimited())
            .unwrap();
        let map = objects.create_weak_map(&prototype).unwrap();
        let value = objects.create(None).unwrap();
        objects
            .define(
                &value,
                "back",
                data(Value::Symbol(target.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .weak_map_set_symbol(
                &map,
                &target,
                Value::Object(value.clone()),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Symbol(target.clone()),
                Value::Number(1.0),
                Some(&Value::Symbol(target.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .collect([&registry, &reference, &map], usize::MAX)
            .unwrap();
        assert_eq!(
            objects
                .weak_ref_target(&reference, &mut unlimited())
                .unwrap(),
            Some(Value::Symbol(target.clone()))
        );
        assert_eq!(state(&objects, &registry), (1, 0, false));
        assert!(objects.inspect(&value).is_ok());
        drop(target);
        objects
            .collect([&registry, &reference, &map], usize::MAX)
            .unwrap();
        assert!(weak.upgrade().is_none());
        assert_eq!(
            objects
                .weak_ref_target(&reference, &mut unlimited())
                .unwrap(),
            None
        );
        assert_eq!(
            objects
                .inspect(&map)
                .unwrap()
                .weak_map
                .as_ref()
                .unwrap()
                .symbol_entries
                .len(),
            0
        );
        assert_eq!(state(&objects, &registry), (1, 1, true));
        stale(&objects, &value);
    }

    #[test]
    fn a_late_symbol_owner_cancels_a_prepared_clear_and_a_second_collection_can_schedule_it() {
        let (mut objects, prototype, _, registry) = fixture();
        let map = objects.create_weak_map(&prototype).unwrap();
        let target = objects.create(None).unwrap();
        let key = JsSymbol::new(None);
        let weak = key.downgrade();
        objects
            .define(
                &target,
                "key",
                data(Value::Symbol(key.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .weak_map_set_symbol(&map, &key, Value::Object(target.clone()), &mut unlimited())
            .unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Object(target.clone()),
                Value::Number(1.0),
                None,
                &mut unlimited(),
            )
            .unwrap();
        drop(key);
        let promoted = Rc::new(RefCell::new(None));
        let sink = promoted.clone();
        let key = weak.clone();
        Objects::with_symbol_gc_test_hooks(
            Some(Box::new(move || {
                *sink.borrow_mut() = key.upgrade();
            })),
            None,
            || objects.collect([&registry, &map], usize::MAX),
        )
        .unwrap();
        assert!(promoted.borrow().is_some());
        assert!(objects.inspect(&target).is_ok());
        assert_eq!(state(&objects, &registry), (1, 0, false));
        assert!(!objects.has_finalization_cleanup_job());
        drop(promoted.borrow_mut().take());
        objects.collect([&registry, &map], usize::MAX).unwrap();
        assert!(weak.upgrade().is_none());
        stale(&objects, &target);
        assert_eq!(state(&objects, &registry), (1, 1, true));
    }

    #[test]
    fn late_symbol_marking_can_reach_a_new_registry_and_queue_only_its_already_marked_capture() {
        let (mut objects, prototype, _, registry) = fixture();
        let map = objects.create_weak_map(&prototype).unwrap();
        let target = objects.create(None).unwrap();
        let key = JsSymbol::new(None);
        let weak = key.downgrade();
        objects
            .define(
                &target,
                "key",
                data(Value::Symbol(key.clone())),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .weak_map_set_symbol(
                &map,
                &key,
                Value::Object(registry.clone()),
                &mut unlimited(),
            )
            .unwrap();
        objects
            .finalization_registry_register(
                &registry,
                &Value::Object(target.clone()),
                Value::Number(1.0),
                None,
                &mut unlimited(),
            )
            .unwrap();
        drop(key);
        let promoted = Rc::new(RefCell::new(None));
        let sink = promoted.clone();
        Objects::with_symbol_gc_test_hooks(
            Some(Box::new(move || {
                *sink.borrow_mut() = weak.upgrade();
            })),
            None,
            || objects.collect([&map], usize::MAX),
        )
        .unwrap();
        assert!(promoted.borrow().is_some());
        stale(&objects, &target);
        assert_eq!(state(&objects, &registry), (1, 1, true));
        assert_eq!(
            objects.finalization_jobs.as_slice(),
            std::slice::from_ref(&registry)
        );
        drop(promoted.borrow_mut().take());
        objects.collect([], usize::MAX).unwrap();
        assert!(objects.inspect(&registry).is_ok());
    }

    #[test]
    fn invalid_factory_inputs_full_heap_and_opted_work_leave_old_storage_and_dispatch_unchanged() {
        let mut objects = Objects::with_limits(Some(2), None);
        let prototype = objects.create(None).unwrap();
        let callback = objects.create_builtin(&prototype, Builtin::Object).unwrap();
        assert!(matches!(
            objects.create_finalization_registry(&prototype, &prototype, &mut unlimited()),
            Err(Error::NotCallable)
        ));
        assert!(matches!(
            objects.create_finalization_registry(&prototype, &callback, &mut Budget::new(0)),
            Err(Error::WorkLimit)
        ));
        assert!(matches!(
            objects.create_finalization_registry(&prototype, &callback, &mut unlimited()),
            Err(Error::Heap(spite_heap::Error::Capacity))
        ));
        assert!(!objects.finalization_registries_ever_stored);
        assert!(!objects.has_finalization_cleanup_job());
        assert!(!objects.finalization_registry_has_slot(&prototype).unwrap());
        let (mut objects, _, _, registry) = fixture();
        let target = objects.create(None).unwrap();
        let token = JsSymbol::new(None);
        assert!(matches!(
            objects.finalization_registry_register(
                &registry,
                &Value::Object(target.clone()),
                Value::Number(1.0),
                Some(&Value::Symbol(token.clone())),
                &mut Budget::new(0)
            ),
            Err(Error::WorkLimit)
        ));
        assert_eq!(state(&objects, &registry), (0, 0, false));
        objects
            .finalization_registry_register(
                &registry,
                &Value::Object(target),
                Value::Number(2.0),
                Some(&Value::Symbol(token.clone())),
                &mut unlimited(),
            )
            .unwrap();
        assert!(matches!(
            objects.finalization_registry_unregister(
                &registry,
                &Value::Symbol(token),
                &mut Budget::new(0)
            ),
            Err(Error::WorkLimit)
        ));
        assert_eq!(state(&objects, &registry), (1, 0, false));
    }
}
