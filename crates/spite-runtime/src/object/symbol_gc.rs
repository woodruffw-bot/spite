//! Closed Runtime object/Symbol collection with an allocation-free death commit.

#[cfg(test)]
mod hooks;
#[cfg(test)]
mod tests;
mod walk;

use super::{entry::Entry, finalization_registry::WeakTarget, weak_map::WeakMapData};
use crate::Value;
use spite_core::{JsSymbol, SymbolCollectionGuard, SymbolCollectionPin, WeakJsSymbol};
use spite_heap::{Collection, Error, Handle, Heap, SweepSession, Trace};
use std::collections::{HashMap, HashSet};
use walk::{Mode, SharedKind};

struct Work {
    remaining: usize,
    maximum: usize,
}

impl Work {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.remaining = self.remaining.checked_sub(count).ok_or(Error::Limit)?;
        Ok(())
    }

    fn push<T>(&mut self, values: &mut Vec<T>, value: T) -> Result<(), Error> {
        if values.len() == values.capacity() {
            self.charge(values.len())?;
        }
        values.try_reserve(1).map_err(|_| Error::Capacity)?;
        values.push(value);
        Ok(())
    }
}

struct Candidate {
    pin: Option<SymbolCollectionPin>,
    local: usize,
    marked: bool,
    waiting: Vec<Edge>,
}

struct SharedGroup {
    identity: *const (),
    kind: SharedKind,
    representative: usize,
    local_owners: usize,
    symbols: Vec<usize>,
}

struct Universe {
    ids: HashMap<WeakJsSymbol, usize>,
    cells: HashSet<*const JsSymbol>,
    candidates: Vec<Candidate>,
    shared_ids: HashMap<(SharedKind, *const ()), usize>,
    shared: Vec<SharedGroup>,
}

impl Universe {
    fn new() -> Self {
        Self {
            ids: HashMap::new(),
            cells: HashSet::new(),
            candidates: Vec::new(),
            shared_ids: HashMap::new(),
            shared: Vec::new(),
        }
    }

    fn candidate(&mut self, key: &WeakJsSymbol, work: &mut Work) -> Result<usize, Error> {
        if let Some(index) = self.ids.get(key) {
            return Ok(*index);
        }
        if self.ids.len() == self.ids.capacity() {
            work.charge(self.ids.len())?;
        }
        self.ids.try_reserve(1).map_err(|_| Error::Capacity)?;
        let index = self.candidates.len();
        // Upgrade only outside the commit gate. This is the sole separately
        // counted collector strong owner; weak IDs/queues never clone it.
        let pin = SymbolCollectionPin::acquire(key);
        work.push(
            &mut self.candidates,
            Candidate {
                pin,
                local: 0,
                marked: false,
                waiting: Vec::new(),
            },
        )?;
        self.ids.insert(key.clone(), index);
        Ok(index)
    }

    fn field(&mut self, value: Option<&JsSymbol>, work: &mut Work) -> Result<(), Error> {
        work.charge(1)?;
        let Some(value) = value else {
            return Ok(());
        };
        let pointer = std::ptr::from_ref(value);
        if self.cells.contains(&pointer) {
            return Ok(());
        }
        if self.cells.len() == self.cells.capacity() {
            work.charge(self.cells.len())?;
        }
        self.cells.try_reserve(1).map_err(|_| Error::Capacity)?;
        let index = self.candidate(&value.downgrade(), work)?;
        self.candidates[index].local = self.candidates[index]
            .local
            .checked_add(1)
            .ok_or(Error::CollectionState)?;
        self.cells.insert(pointer);
        Ok(())
    }

    fn id(&self, value: &WeakJsSymbol) -> Result<usize, Error> {
        self.ids.get(value).copied().ok_or(Error::CollectionState)
    }

    fn edge(&self, value: &Value) -> Result<Edge, Error> {
        Ok(match value {
            Value::Object(value) => Edge::Object(value.clone()),
            Value::Symbol(value) => Edge::Symbol(self.id(&value.downgrade())?),
            Value::Undefined
            | Value::Null
            | Value::Boolean(_)
            | Value::Number(_)
            | Value::BigInt(_)
            | Value::String(_) => Edge::None,
        })
    }
}

enum Edge {
    None,
    Object(Handle),
    Symbol(usize),
}

#[derive(Clone, Copy)]
enum Key {
    Object(usize),
    Symbol(usize),
}
enum RawKey {
    Object(Handle),
    Symbol(usize),
}
#[derive(Clone, Copy)]
enum Pending {
    Object(usize),
    Symbol(usize),
}

struct Cleanup {
    slot: usize,
    object_keep: Vec<bool>,
    symbol_keep: Vec<bool>,
    symbol_ids: Vec<usize>,
}

struct RegistryCleanup {
    slot: usize,
    clear: Vec<usize>,
    cleared: usize,
    enqueue: Option<Handle>,
    discarded_targets: Vec<WeakTarget>,
}

fn census(session: &SweepSession<'_, Entry>, work: &mut Work) -> Result<Universe, Error> {
    let mut universe = Universe::new();
    for slot in 0..session.slot_count() {
        work.charge(1)?;
        let Some(entry) = session.entry(slot) else {
            continue;
        };
        work.charge(walk::physical_index_work(entry)?)?;
        walk::symbols(entry, Mode::Physical, &mut |value| {
            universe.field(value, work)
        })?;
        // Include all WeakMap weak-only candidates, even without a local Arc.
        // WeakSet keys have no conditional values and need no census entry.
        walk::weak_symbols(entry, &mut |value| {
            work.charge(1)?;
            if let Some(value) = value {
                universe.candidate(value, work)?;
            }
            Ok(())
        })?;
        for kind in [SharedKind::Fields, SharedKind::PrivateMethods] {
            work.charge(1)?;
            let Some(identity) = walk::shared_identity(entry, kind) else {
                continue;
            };
            if let Some(index) = universe.shared_ids.get(&(kind, identity)).copied() {
                universe.shared[index].local_owners = universe.shared[index]
                    .local_owners
                    .checked_add(1)
                    .ok_or(Error::CollectionState)?;
                continue;
            }
            let mut symbols = Vec::new();
            walk::shared_symbols(entry, kind, &mut |value| {
                work.charge(1)?;
                if let Some(value) = value {
                    work.push(&mut symbols, universe.id(&value.downgrade())?)?;
                }
                Ok(())
            })?;
            if universe.shared_ids.len() == universe.shared_ids.capacity() {
                work.charge(universe.shared_ids.len())?;
            }
            universe
                .shared_ids
                .try_reserve(1)
                .map_err(|_| Error::Capacity)?;
            let index = universe.shared.len();
            work.push(
                &mut universe.shared,
                SharedGroup {
                    identity,
                    kind,
                    representative: slot,
                    local_owners: 1,
                    symbols,
                },
            )?;
            universe.shared_ids.insert((kind, identity), index);
        }
    }
    Ok(universe)
}

struct Collector<'heap> {
    session: SweepSession<'heap, Entry>,
    universe: Universe,
    work: Work,
    pending: Vec<Pending>,
    waiting: Vec<Vec<Edge>>,
    reached_registries: Vec<(usize, Handle)>,
}

impl Collector<'_> {
    fn mark_edge(&mut self, edge: Edge) -> Result<(), Error> {
        match edge {
            Edge::None => {}
            Edge::Object(value) => {
                let index = self.session.index(&value)?;
                if !self.session.is_marked(index) {
                    if matches!(self.session.entry(index), Some(Entry::Object(object))
                        if object.finalization_registry.is_some())
                    {
                        self.work.charge(1)?;
                        // Checked, unrooted metadata: only first-marked registries
                        // reach the planner, including late ephemeron activations.
                        self.work
                            .push(&mut self.reached_registries, (index, value))?;
                    }
                    self.work.push(&mut self.pending, Pending::Object(index))?;
                    self.session.mark(index)?;
                }
            }
            Edge::Symbol(index) => {
                let candidate = self
                    .universe
                    .candidates
                    .get(index)
                    .ok_or(Error::CollectionState)?;
                if candidate.pin.is_some() && !candidate.marked {
                    self.work.push(&mut self.pending, Pending::Symbol(index))?;
                    self.universe.candidates[index].marked = true;
                }
            }
        }
        Ok(())
    }

    fn condition(&mut self, key: Key, edge: Edge) -> Result<(), Error> {
        if matches!(edge, Edge::None) {
            return Ok(());
        }
        let active = match key {
            Key::Object(index) => self.session.is_marked(index),
            Key::Symbol(index) => self.universe.candidates[index].marked,
        };
        if active {
            return self.mark_edge(edge);
        }
        match key {
            Key::Object(index) => self.work.push(&mut self.waiting[index], edge)?,
            Key::Symbol(index) => self
                .work
                .push(&mut self.universe.candidates[index].waiting, edge)?,
        }
        Ok(())
    }

    fn exhaust(&mut self) -> Result<(), Error> {
        while let Some(pending) = self.pending.pop() {
            self.work.charge(1)?;
            let waiting = match pending {
                Pending::Object(index) => std::mem::take(&mut self.waiting[index]),
                Pending::Symbol(index) => {
                    std::mem::take(&mut self.universe.candidates[index].waiting)
                }
            };
            for edge in waiting {
                self.work.charge(1)?;
                self.mark_edge(edge)?;
            }
            let Pending::Object(index) = pending else {
                continue;
            };
            let entry = self.session.entry(index).ok_or(Error::CollectionState)?;
            let mut edges = Vec::new();
            for edge in entry.trace() {
                self.work.charge(1)?;
                if let Some(edge) = edge {
                    self.work.push(&mut edges, Edge::Object(edge.clone()))?;
                }
            }
            walk::symbols(entry, Mode::Logical, &mut |value| {
                self.work.charge(1)?;
                if let Some(value) = value {
                    let id = self.universe.id(&value.downgrade())?;
                    self.work.push(&mut edges, Edge::Symbol(id))?;
                }
                Ok(())
            })?;
            for edge in edges {
                self.mark_edge(edge)?;
            }
            let entry = self.session.entry(index).ok_or(Error::CollectionState)?;
            let mut conditions = Vec::new();
            if let Entry::Object(object) = entry
                && let Some(map) = &object.weak_map
            {
                for (key, value) in &map.entries {
                    self.work.charge(2)?;
                    self.work.push(
                        &mut conditions,
                        (RawKey::Object(key.clone()), self.universe.edge(value)?),
                    )?;
                }
                for (key, value) in &map.symbol_entries {
                    self.work.charge(2)?;
                    self.work.push(
                        &mut conditions,
                        (
                            RawKey::Symbol(self.universe.id(key)?),
                            self.universe.edge(value)?,
                        ),
                    )?;
                }
            }
            for (key, edge) in conditions {
                let key = match key {
                    RawKey::Object(key) => match self.session.index(&key) {
                        Ok(index) => Key::Object(index),
                        Err(Error::StaleHandle) => continue,
                        Err(error) => return Err(error),
                    },
                    RawKey::Symbol(index) => Key::Symbol(index),
                };
                self.condition(key, edge)?;
            }
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<Vec<Cleanup>, Error> {
        let mut cleanup = Vec::new();
        for slot in 0..self.session.slot_count() {
            self.work.charge(1)?;
            if !self.session.is_marked(slot) {
                continue;
            }
            let Some(Entry::Object(object)) = self.session.entry(slot) else {
                continue;
            };
            let Some(map) = &object.weak_map else {
                continue;
            };
            validate_index(map, &mut self.work)?;
            let mut object_keep = Vec::new();
            let mut symbol_keep = Vec::new();
            let mut symbol_ids = Vec::new();
            for (key, _) in &map.entries {
                self.work.charge(1)?;
                let keep = match self.session.index(key) {
                    Ok(index) => self.session.is_marked(index),
                    Err(Error::StaleHandle) => false,
                    Err(error) => return Err(error),
                };
                self.work.push(&mut object_keep, keep)?;
            }
            for (key, _) in &map.symbol_entries {
                self.work.charge(1)?;
                let id = self.universe.id(key)?;
                self.work.push(&mut symbol_ids, id)?;
                self.work
                    .push(&mut symbol_keep, self.universe.candidates[id].marked)?;
            }
            self.work.push(
                &mut cleanup,
                Cleanup {
                    slot,
                    object_keep,
                    symbol_keep,
                    symbol_ids,
                },
            )?;
        }
        Ok(cleanup)
    }

    fn registry_cleanup(&mut self, jobs: &mut Vec<Handle>) -> Result<Vec<RegistryCleanup>, Error> {
        let mut plans = Vec::new();
        let mut additions = 0usize;
        for (slot, handle) in &self.reached_registries {
            self.work.charge(1)?;
            let Some(Entry::Object(object)) = self.session.entry(*slot) else {
                return Err(Error::CollectionState);
            };
            let data = object
                .finalization_registry
                .as_ref()
                .ok_or(Error::CollectionState)?;
            let mut clear = Vec::new();
            let mut already_cleared = 0usize;
            for (index, cell) in data.cells.iter().enumerate() {
                self.work.charge(1)?;
                let keep = match &cell.target {
                    None => {
                        already_cleared = already_cleared
                            .checked_add(1)
                            .ok_or(Error::CollectionState)?;
                        continue;
                    }
                    Some(WeakTarget::Object(key)) => match self.session.index(key) {
                        Ok(index) => self.session.is_marked(index),
                        Err(Error::StaleHandle) => false,
                        Err(error) => return Err(error),
                    },
                    Some(WeakTarget::Symbol(key)) => {
                        // Every marked candidate has a live pin. The final gate
                        // rechecks owners before retirement; a concurrent closed
                        // collector counts our pin as an external owner. No weak
                        // upgrade or owning Symbol clone enters this plan.
                        self.universe.candidates[self.universe.id(key)?].marked
                    }
                };
                if !keep {
                    self.work.push(&mut clear, index)?;
                }
            }
            if already_cleared != data.cleared {
                return Err(Error::CollectionState);
            }
            let cleared = already_cleared
                .checked_add(clear.len())
                .ok_or(Error::CollectionState)?;
            let enqueue = if cleared != 0 && !data.pending {
                self.work.charge(1)?;
                additions = additions.checked_add(1).ok_or(Error::Capacity)?;
                Some(handle.clone())
            } else {
                None
            };
            if !clear.is_empty() || enqueue.is_some() {
                let mut discarded_targets = Vec::new();
                self.work.charge(clear.len())?;
                discarded_targets
                    .try_reserve_exact(clear.len())
                    .map_err(|_| Error::Capacity)?;
                self.work.push(
                    &mut plans,
                    RegistryCleanup {
                        slot: *slot,
                        clear,
                        cleared,
                        enqueue,
                        discarded_targets,
                    },
                )?;
            }
        }
        // Retry after late external owners can reach further registries. Every
        // attempt recomputes masks and reserves its full actual append count
        // before the next irreversible gate, never after a partial commit.
        if additions > jobs.capacity() - jobs.len() {
            self.work.charge(jobs.len())?;
        }
        jobs.try_reserve(additions).map_err(|_| Error::Capacity)?;
        Ok(plans)
    }
}

fn validate_index(map: &WeakMapData, work: &mut Work) -> Result<(), Error> {
    if map.entries.len() != map.index.len() || map.symbol_entries.len() != map.symbol_index.len() {
        return Err(Error::CollectionState);
    }
    for (index, (key, _)) in map.entries.iter().enumerate() {
        work.charge(1)?;
        if map.index.get(key) != Some(&index) {
            return Err(Error::CollectionState);
        }
    }
    for (index, (key, _)) in map.symbol_entries.iter().enumerate() {
        work.charge(1)?;
        if map.symbol_index.get(key) != Some(&index) {
            return Err(Error::CollectionState);
        }
    }
    Ok(())
}

fn final_attempt(
    collector: &Collector<'_>,
    found: &mut [bool],
    live: &mut [bool],
    roots: &mut Vec<usize>,
) -> Result<bool, Error> {
    // No allocating/dropping owner enters the lexical gate scope. Every push is
    // bounded by the full pre-reserved candidate count and the dedup bit vector.
    let mut fault = false;
    let committed = {
        let mut guard = SymbolCollectionGuard::acquire();
        for (id, candidate) in collector.universe.candidates.iter().enumerate() {
            let Some(pin) = &candidate.pin else {
                fault |= candidate.local != 0;
                continue;
            };
            live[id] = guard.is_live(pin);
            fault |= !live[id] && candidate.local != 0;
            let accounted = candidate.local.checked_add(1);
            let residual = accounted.and_then(|local| guard.strong_count(pin).checked_sub(local));
            match residual {
                None => fault = true,
                Some(external) if live[id] && external != 0 && !candidate.marked => {
                    found[id] = true;
                    roots.push(id);
                }
                Some(_) => {}
            }
        }
        for group in &collector.universe.shared {
            let counts = collector
                .session
                .entry(group.representative)
                .and_then(|entry| {
                    (walk::shared_identity(entry, group.kind) == Some(group.identity))
                        .then(|| walk::shared_counts(entry, group.kind))
                        .flatten()
                });
            match counts {
                Some((strong, weak)) if strong >= group.local_owners => {
                    if strong > group.local_owners || weak != 0 {
                        for &id in &group.symbols {
                            if live[id] && !collector.universe.candidates[id].marked && !found[id] {
                                found[id] = true;
                                roots.push(id);
                            }
                        }
                    }
                }
                _ => fault = true,
            }
        }
        if fault || !roots.is_empty() {
            false
        } else {
            // Separate pass: all roots/faults were found before any irreversible
            // store. Never reset a bit after another thread could observe None.
            for (id, candidate) in collector.universe.candidates.iter().enumerate() {
                if !candidate.marked
                    && live[id]
                    && let Some(pin) = &candidate.pin
                {
                    guard.retire(pin);
                }
            }
            true
        }
    };
    if fault {
        Err(Error::CollectionState)
    } else {
        Ok(committed)
    }
}

pub(super) fn collect(
    heap: &mut Heap<Entry>,
    jobs: &mut Vec<Handle>,
    roots: Vec<Handle>,
    maximum: usize,
) -> Result<Collection, Error> {
    let mut work = Work {
        remaining: maximum,
        maximum,
    };
    work.charge(heap.allocated_slots().checked_mul(2).ok_or(Error::Limit)?)?;
    let session = heap.prepare_sweep()?;
    let universe = census(&session, &mut work)?;
    let mut waiting = Vec::new();
    work.charge(session.slot_count())?;
    waiting
        .try_reserve_exact(session.slot_count())
        .map_err(|_| Error::Capacity)?;
    waiting.resize_with(session.slot_count(), Vec::new);
    let mut collector = Collector {
        session,
        universe,
        work,
        pending: Vec::new(),
        waiting,
        reached_registries: Vec::new(),
    };
    for root in roots {
        collector.work.charge(1)?;
        collector.mark_edge(Edge::Object(root))?;
    }
    let count = collector.universe.candidates.len();
    let mut new_roots = Vec::new();
    new_roots
        .try_reserve_exact(count)
        .map_err(|_| Error::Capacity)?;
    let mut found = Vec::new();
    collector
        .work
        .charge(count.checked_mul(2).ok_or(Error::Limit)?)?;
    found
        .try_reserve_exact(count)
        .map_err(|_| Error::Capacity)?;
    found.resize(count, false);
    let mut live = Vec::new();
    live.try_reserve_exact(count).map_err(|_| Error::Capacity)?;
    live.resize(count, false);
    loop {
        collector.exhaust()?;
        let cleanup = collector.cleanup()?;
        let registries = collector.registry_cleanup(jobs)?;
        #[cfg(test)]
        hooks::before_recheck();
        let recheck = collector
            .universe
            .shared
            .iter()
            .try_fold(count, |cost, group| {
                cost.checked_add(group.symbols.len())
                    .and_then(|cost| cost.checked_add(1))
                    .ok_or(Error::Limit)
            })?;
        collector.work.charge(
            recheck
                .checked_add(count.checked_mul(3).ok_or(Error::Limit)?)
                .ok_or(Error::Limit)?,
        )?;
        found.fill(false);
        live.fill(false);
        new_roots.clear();
        if final_attempt(&collector, &mut found, &mut live, &mut new_roots)? {
            // COMMITTED: no fallible operations remain. All drops/allocators run
            // unlocked. The prepared masks/index invariant was validated above.
            #[cfg(test)]
            hooks::after_commit();
            for plan in registries {
                let RegistryCleanup {
                    slot,
                    clear,
                    cleared,
                    enqueue,
                    mut discarded_targets,
                } = plan;
                {
                    let Some(Entry::Object(object)) = collector.session.entry_mut(slot) else {
                        unreachable!("prepared reached registry");
                    };
                    let data = object
                        .finalization_registry
                        .as_mut()
                        .expect("prepared registry slot");
                    for index in clear {
                        if let Some(target) = data.cells[index].target.take() {
                            discarded_targets.push(target);
                        }
                    }
                    data.cleared = cleared;
                    if let Some(handle) = enqueue {
                        data.pending = true;
                        jobs.push(handle);
                    }
                }
                // Weak allocation destructors run outside the record borrow,
                // with reserved capacity and no Symbol gate held.
                drop(discarded_targets);
            }
            for plan in cleanup {
                if let Some(Entry::Object(object)) = collector.session.entry_mut(plan.slot)
                    && let Some(map) = &mut object.weak_map
                {
                    for index in (0..plan.object_keep.len()).rev() {
                        if !plan.object_keep[index] {
                            map.remove_at(index);
                        }
                    }
                    for index in (0..plan.symbol_keep.len()).rev() {
                        if !plan.symbol_keep[index] || !live[plan.symbol_ids[index]] {
                            map.remove_symbol_at(index);
                        }
                    }
                }
            }
            let used = collector.work.maximum - collector.work.remaining;
            return Ok(collector.session.finish(used));
        }
        for id in new_roots.drain(..) {
            collector.work.charge(1)?;
            collector.mark_edge(Edge::Symbol(id))?;
        }
        // Each retry marks at least one new candidate. The finite universe and
        // monotonic marks bound retries; only the caller's explicit work budget
        // can stop planning. Failed planning has not retired any identity.
    }
}
