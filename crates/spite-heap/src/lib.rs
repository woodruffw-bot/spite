//! Safe generational storage with optional capacity limits for object graphs.
//!
//! Handles validate both heap identity and slot generation. Cloning a handle does
//! not keep its stored value alive; collection roots are supplied explicitly.
//! Allocation never performs implicit collection.

use std::{
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
};

#[derive(Debug)]
struct Identity;

/// An opaque, unrooted reference to one generation of one heap slot.
///
/// The identity token contains no object data. Handles from another heap, or from
/// a removed value, are rejected even when their slot numbers happen to match.
#[derive(Clone)]
pub struct Handle {
    owner: Rc<Identity>,
    slot: usize,
    generation: u64,
}

impl PartialEq for Handle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.owner, &other.owner)
            && self.slot == other.slot
            && self.generation == other.generation
    }
}
impl Eq for Handle {}

impl Hash for Handle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.owner).hash(state);
        self.slot.hash(state);
        self.generation.hash(state);
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Handle")
            .field("slot", &self.slot)
            .field("generation", &self.generation)
            .finish_non_exhaustive()
    }
}

/// An invalid reference or exhausted storage limit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// The configured slot quota or allocator capacity has been exhausted.
    Capacity,
    /// The handle belongs to a different heap.
    ForeignHandle,
    /// The handle's slot is empty or belongs to a later generation.
    StaleHandle,
    /// Collection exhausted its work budget before sweeping any values.
    Limit,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Capacity => "heap storage capacity exceeded",
            Self::ForeignHandle => "handle belongs to another heap",
            Self::StaleHandle => "handle refers to a removed value",
            Self::Limit => "heap collection work limit exceeded",
        })
    }
}
impl std::error::Error for Error {}

/// Enumerates the fields a collector must inspect to find outgoing references.
///
/// Yield `Some(handle)` for an outgoing edge and `None` for an inspected value
/// that contains no edge. Yielding non-reference fields lets the collector charge
/// for scanning primitive-valued properties too. Each iterator step must perform
/// bounded work; do not hide an unbounded filter or scan inside `next`.
pub trait Trace {
    /// Returns each inspected field, without cloning handles or mutating the graph.
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>>;

    /// Returns weak keys and their conditionally retained values.
    ///
    /// These entries do not retain their keys. A reachable container retains a
    /// value only when its key is independently reachable; `None` denotes a
    /// primitive value. Stale keys are ignored, while foreign keys and active
    /// invalid value handles fail collection before sweeping. Each iterator
    /// step must perform bounded work. Do not also yield these entries from
    /// [`Self::trace`].
    fn ephemerons(&self) -> impl Iterator<Item = (&Handle, Option<&Handle>)> {
        std::iter::empty()
    }

    /// Removes entries whose keys the completed mark does not retain.
    ///
    /// Called only for reachable containers that enumerated ephemerons, after
    /// every fallible collection check and before sweeping. Inspect only the
    /// entries returned by [`Self::ephemerons`], with bounded work per entry.
    /// Do not allocate, add graph edges, or change retained entries. The default
    /// keeps unrooted handles; containers that own primitive values should
    /// override this method to release those values when their keys disappear.
    fn retain_ephemerons(&mut self, _retain: impl Fn(&Handle) -> bool) {}
}

/// The result of a completed mark-and-sweep collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Collection {
    /// Number of unreachable values removed.
    pub reclaimed: usize,
    /// Number of values remaining reachable from the supplied roots.
    pub live: usize,
    /// Work used by slot scans, roots, values, fields, and ephemeron processing.
    pub work_used: usize,
}

#[derive(Debug)]
struct Slot<T> {
    generation: u64,
    value: Option<T>,
}

/// Heap-owned values addressed by generation-checked handles.
///
/// The capacity limit includes empty and permanently retired slots. Removed
/// values can free reusable slots, but generations never wrap around.
#[derive(Debug)]
pub struct Heap<T> {
    owner: Rc<Identity>,
    slots: Vec<Slot<T>>,
    free: Vec<usize>,
    live: usize,
    max_slots: Option<usize>,
}

impl<T> Heap<T> {
    /// Creates an empty heap with an explicit maximum number of storage slots.
    pub fn new(max_slots: usize) -> Self {
        Self::with_capacity_limit(Some(max_slots))
    }

    /// Creates an empty heap with an optional storage-slot limit.
    /// `None` leaves capacity to the allocator and platform address space.
    pub fn with_capacity_limit(max_slots: Option<usize>) -> Self {
        Self {
            owner: Rc::new(Identity),
            slots: Vec::new(),
            free: Vec::new(),
            live: 0,
            max_slots,
        }
    }

    /// Returns the number of occupied slots.
    pub fn len(&self) -> usize {
        self.live
    }

    /// Returns whether there are no live values.
    pub fn is_empty(&self) -> bool {
        self.live == 0
    }

    /// Returns the number of allocated slots, including reusable and retired slots.
    pub fn allocated_slots(&self) -> usize {
        self.slots.len()
    }

    /// Stores a value and returns an unrooted handle, without collecting garbage.
    pub fn insert(&mut self, value: T) -> Result<Handle, Error> {
        let index = if let Some(index) = self.free.pop() {
            debug_assert!(self.slots[index].value.is_none());
            self.slots[index].value = Some(value);
            index
        } else {
            if self
                .max_slots
                .is_some_and(|limit| self.slots.len() >= limit)
            {
                return Err(Error::Capacity);
            }
            self.slots.try_reserve(1).map_err(|_| Error::Capacity)?;
            let index = self.slots.len();
            self.slots.push(Slot {
                generation: 0,
                value: Some(value),
            });
            index
        };
        self.live += 1;
        Ok(Handle {
            owner: Rc::clone(&self.owner),
            slot: index,
            generation: self.slots[index].generation,
        })
    }

    /// Borrows a live value after validating its handle.
    pub fn get(&self, handle: &Handle) -> Result<&T, Error> {
        let index = self.index(handle)?;
        Ok(self.slots[index]
            .value
            .as_ref()
            .expect("validated occupancy"))
    }

    /// Mutably borrows a live value after validating its handle.
    pub fn get_mut(&mut self, handle: &Handle) -> Result<&mut T, Error> {
        let index = self.index(handle)?;
        Ok(self.slots[index]
            .value
            .as_mut()
            .expect("validated occupancy"))
    }

    /// Removes a value and invalidates all handles for its old generation.
    pub fn remove(&mut self, handle: &Handle) -> Result<T, Error> {
        let index = self.index(handle)?;
        Ok(self.take(index))
    }

    fn index(&self, handle: &Handle) -> Result<usize, Error> {
        if !Rc::ptr_eq(&self.owner, &handle.owner) {
            return Err(Error::ForeignHandle);
        }
        let slot = self.slots.get(handle.slot).ok_or(Error::StaleHandle)?;
        if slot.generation != handle.generation || slot.value.is_none() {
            return Err(Error::StaleHandle);
        }
        Ok(handle.slot)
    }

    fn take(&mut self, index: usize) -> T {
        let slot = &mut self.slots[index];
        let value = slot.value.take().expect("removing an occupied slot");
        self.live -= 1;
        if let Some(generation) = slot.generation.checked_add(1) {
            slot.generation = generation;
            self.free.push(index);
        }
        // A slot at u64::MAX is retired, never returned to the free list.
        value
    }
}

impl<T: Trace> Heap<T> {
    /// Collects values unreachable from the explicitly supplied roots.
    ///
    /// Traversal is iterative, so cycles and deep graphs do not use Rust recursion.
    /// All roots, strong edges and active ephemeron values are validated before
    /// sweeping; stale weak keys are ignored. A work-limit, capacity or handle
    /// error leaves every value and generation unchanged. Handle variables
    /// outside this root list do not automatically keep their values alive.
    pub fn collect<'a>(
        &mut self,
        roots: impl IntoIterator<Item = &'a Handle>,
        max_work: usize,
    ) -> Result<Collection, Error> {
        let mut remaining = max_work;
        // Reserve both scratch initialization and the final sweep before allocating
        // mark state. The sweep cannot exhaust the budget midway through removal.
        charge(
            &mut remaining,
            self.slots.len().checked_mul(2).ok_or(Error::Limit)?,
        )?;
        // Every live slot could become free. Reserve that growth before any
        // removal so the final sweep needs no allocator calls.
        self.free
            .try_reserve(self.live)
            .map_err(|_| Error::Capacity)?;
        let mut marked = Vec::new();
        marked
            .try_reserve_exact(self.slots.len())
            .map_err(|_| Error::Capacity)?;
        marked.resize(self.slots.len(), false);
        let mut pending = Vec::new();
        // Index waiting values by key slot. Each ephemeron is inspected and
        // activated at most once, including reverse-ordered chains. Neither a
        // waiting edge nor its Handle identity token keeps the key alive.
        // https://262.ecma-international.org/17.0/#sec-weakmap-objects
        let mut waiting: HashMap<usize, Vec<&Handle>> = HashMap::new();
        // Save validated key generations separately from the values so cleanup
        // can borrow a container mutably without borrowing the heap for lookup.
        let mut key_generations = HashMap::new();
        let mut containers = Vec::new();
        for root in roots {
            charge(&mut remaining, 1)?;
            self.mark(root, &mut marked, &mut pending)?;
        }
        while let Some(index) = pending.pop() {
            charge(&mut remaining, 1)?;
            if let Some(values) = waiting.remove(&index) {
                for value in values {
                    charge(&mut remaining, 1)?;
                    self.mark(value, &mut marked, &mut pending)?;
                }
            }
            let value = self.slots[index]
                .value
                .as_ref()
                .expect("marked slot is occupied");
            for field in value.trace() {
                charge(&mut remaining, 1)?;
                if let Some(edge) = field {
                    self.mark(edge, &mut marked, &mut pending)?;
                }
            }
            let mut has_ephemerons = false;
            for (key, edge) in value.ephemerons() {
                // Prepay cleanup now: no work-limit error may occur after it
                // starts releasing stored values, even for stale weak keys.
                charge(&mut remaining, 2)?;
                if !has_ephemerons {
                    charge(&mut remaining, 1)?;
                    containers.try_reserve(1).map_err(|_| Error::Capacity)?;
                    containers.push(index);
                    has_ephemerons = true;
                }
                let key_index = match self.index(key) {
                    Ok(index) => index,
                    Err(Error::StaleHandle) => continue,
                    Err(error) => return Err(error),
                };
                key_generations
                    .try_reserve(1)
                    .map_err(|_| Error::Capacity)?;
                key_generations.insert(key_index, key.generation);
                if let Some(edge) = edge {
                    if marked[key_index] {
                        charge(&mut remaining, 1)?;
                        self.mark(edge, &mut marked, &mut pending)?;
                    } else {
                        waiting.try_reserve(1).map_err(|_| Error::Capacity)?;
                        let values = waiting.entry(key_index).or_default();
                        values.try_reserve(1).map_err(|_| Error::Capacity)?;
                        values.push(edge);
                    }
                }
            }
        }
        drop(waiting);
        for index in containers {
            let value = self.slots[index]
                .value
                .as_mut()
                .expect("marked container is occupied");
            value.retain_ephemerons(|key| {
                Rc::ptr_eq(&self.owner, &key.owner)
                    && key_generations.get(&key.slot) == Some(&key.generation)
                    && marked[key.slot]
            });
        }
        let before = self.live;
        for (index, reached) in marked.into_iter().enumerate() {
            if !reached && self.slots[index].value.is_some() {
                drop(self.take(index));
            }
        }
        Ok(Collection {
            reclaimed: before - self.live,
            live: self.live,
            work_used: max_work - remaining,
        })
    }

    fn mark(
        &self,
        handle: &Handle,
        marked: &mut [bool],
        pending: &mut Vec<usize>,
    ) -> Result<(), Error> {
        let index = self.index(handle)?;
        if !marked[index] {
            pending.try_reserve(1).map_err(|_| Error::Capacity)?;
            marked[index] = true;
            pending.push(index);
        }
        Ok(())
    }
}

fn charge(remaining: &mut usize, work: usize) -> Result<(), Error> {
    *remaining = remaining.checked_sub(work).ok_or(Error::Limit)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashed_handles_preserve_heap_identity_and_generation() {
        let mut first = Heap::new(1);
        let mut second = Heap::new(1);
        let old = first.insert(1).unwrap();
        let foreign = second.insert(1).unwrap();
        let mut set = std::collections::HashSet::new();
        assert!(set.insert(old.clone()));
        assert!(!set.insert(old.clone()));
        assert!(set.insert(foreign));
        first.remove(&old).unwrap();
        let reused = first.insert(2).unwrap();
        assert_eq!(old.slot, reused.slot);
        assert!(set.insert(reused.clone()));
        assert!(set.contains(&old));
        assert!(set.contains(&reused));
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn exhausted_generations_retire_slots_instead_of_reviving_old_handles() {
        let mut heap = Heap::new(2);
        let mut old = heap.insert(1).unwrap();
        heap.slots[old.slot].generation = u64::MAX;
        old.generation = u64::MAX;
        assert_eq!(heap.remove(&old), Ok(1));
        assert_eq!(heap.get(&old), Err(Error::StaleHandle));
        let next = heap.insert(2).unwrap();
        assert_ne!(old.slot, next.slot);
        assert_eq!(heap.allocated_slots(), 2);
        assert_eq!(heap.insert(3), Err(Error::Capacity));
        assert_eq!(heap.remove(&next), Ok(2));
        assert!(heap.insert(4).is_ok());
        assert_eq!(heap.get(&old), Err(Error::StaleHandle));
    }
}
