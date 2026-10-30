//! Native FIFO deliveries for EnqueueResolveInAgentJob (25.4.3.15).
//!
//! Only waiter identities cross agents. Promise capabilities stay in the
//! receiver's rooted registry, and only that Realm can consume this queue.

use super::{Budget, Error};
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub(crate) struct WaitOwner {
    ready: Mutex<VecDeque<usize>>,
}

impl WaitOwner {
    pub(crate) fn has_ready(&self) -> bool {
        // A poisoned queue must still schedule a host job to report its error.
        self.ready.lock().map_or(true, |ready| !ready.is_empty())
    }

    pub(crate) fn front(&self) -> Result<Option<usize>, Error> {
        let ready = self
            .ready
            .lock()
            .map_err(|_| Error::PoisonedSharedStorage)?;
        Ok(ready.front().copied())
    }

    pub(crate) fn enqueue(&self, id: usize, budget: &mut Budget) -> Result<(), Error> {
        let mut ready = self
            .ready
            .lock()
            .map_err(|_| Error::PoisonedSharedStorage)?;
        budget.charge(if ready.len() == ready.capacity() {
            ready.len() + 1
        } else {
            1
        })?;
        ready
            .try_reserve(1)
            .map_err(|_| Error::Heap(spite_heap::Error::Capacity))?;
        ready.push_back(id);
        Ok(())
    }

    pub(crate) fn consume(&self, id: usize) -> Result<(), Error> {
        let mut ready = self
            .ready
            .lock()
            .map_err(|_| Error::PoisonedSharedStorage)?;
        // Only the owning Realm consumes; senders append while it preflights.
        assert_eq!(ready.pop_front(), Some(id));
        Ok(())
    }
}
