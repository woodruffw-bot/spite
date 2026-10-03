//! Safe, capacity-bounded generational storage for single-threaded object graphs.
//!
//! Handles validate both heap identity and slot generation. Cloning a handle does
//! not keep its stored value alive; collection roots will be supplied explicitly.
//! Allocation never performs implicit collection.

use std::{fmt, rc::Rc};

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
    /// No reusable slot remains and the slot limit has been reached.
    Capacity,
    /// The handle belongs to a different heap.
    ForeignHandle,
    /// The handle's slot is empty or belongs to a later generation.
    StaleHandle,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Capacity => "heap slot limit exceeded",
            Self::ForeignHandle => "handle belongs to another heap",
            Self::StaleHandle => "handle refers to a removed value",
        })
    }
}
impl std::error::Error for Error {}

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
    max_slots: usize,
}

impl<T> Heap<T> {
    /// Creates an empty heap with an explicit maximum number of storage slots.
    pub fn new(max_slots: usize) -> Self {
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
            if self.slots.len() >= self.max_slots {
                return Err(Error::Capacity);
            }
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

#[cfg(test)]
mod tests {
    use super::*;

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
