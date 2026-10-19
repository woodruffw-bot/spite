//! Prepared sweep support without tracing callbacks for a closed collector.

use super::{Collection, Error, Handle, Heap};

/// An exclusively borrowed heap with reserved sweep capacity and mutable marks.
///
/// Workspace implementation interface. Existing generic Trace/collect behavior
/// is unchanged. A closed collector plans and validates outside its own commit
/// gate, then applies its known entry cleanup and consumes this session. No
/// generic tracing or cleanup callback is invoked by the final sweep.
#[doc(hidden)]
pub struct SweepSession<'heap, T> {
    heap: &'heap mut Heap<T>,
    marked: Vec<bool>,
}

impl<T> Heap<T> {
    /// Prepares every potentially reclaimed slot, independently of current marks.
    ///
    /// Allocation failure changes no value or generation. The reserved free
    /// list covers late-mark retries and the final sweep without further growth.
    #[doc(hidden)]
    pub fn prepare_sweep(&mut self) -> Result<SweepSession<'_, T>, Error> {
        self.free
            .try_reserve(self.live)
            .map_err(|_| Error::Capacity)?;
        let mut marked = Vec::new();
        marked
            .try_reserve_exact(self.slots.len())
            .map_err(|_| Error::Capacity)?;
        marked.resize(self.slots.len(), false);
        Ok(SweepSession { heap: self, marked })
    }
}

impl<T> SweepSession<'_, T> {
    /// Returns the fixed slot count, including empty or retired slots.
    pub fn slot_count(&self) -> usize {
        self.marked.len()
    }

    /// Borrows an occupied slot without exposing insertion or generation edits.
    pub fn entry(&self, index: usize) -> Option<&T> {
        self.heap.slots.get(index)?.value.as_ref()
    }

    /// Borrows an occupied value for the closed collector's prepared cleanup.
    ///
    /// Call only after all fallible planning and outside any Symbol commit gate.
    pub fn entry_mut(&mut self, index: usize) -> Option<&mut T> {
        self.heap.slots.get_mut(index)?.value.as_mut()
    }

    /// Resolves a live handle with the heap's existing identity/generation checks.
    pub fn index(&self, handle: &Handle) -> Result<usize, Error> {
        self.heap.index(handle)
    }

    /// Marks an occupied slot, returning whether its state changed.
    ///
    /// Slots and generations cannot change while this session exists. Marks
    /// remain mutable during unlocked fixed-point retries before final commit.
    pub fn mark(&mut self, index: usize) -> Result<bool, Error> {
        if self.entry(index).is_none() {
            return Err(Error::CollectionState);
        }
        let previous = self.marked[index];
        self.marked[index] = true;
        Ok(!previous)
    }

    /// Returns whether a slot is currently marked; invalid indices are false.
    pub fn is_marked(&self, index: usize) -> bool {
        self.marked.get(index).copied().unwrap_or(false)
    }

    /// Applies the final mask without collection allocation, errors or tracing.
    ///
    /// Value destructors can allocate or call host code here; release every
    /// Symbol gate before consuming the session. No Trace method is called.
    pub fn finish(self, work_used: usize) -> Collection {
        let before = self.heap.live;
        for (index, reached) in self.marked.iter().copied().enumerate() {
            if !reached && self.heap.slots[index].value.is_some() {
                drop(self.heap.take(index));
            }
        }
        Collection {
            reclaimed: before - self.heap.live,
            live: self.heap.live,
            work_used,
        }
    }
}

#[cfg(test)]
mod tests;
