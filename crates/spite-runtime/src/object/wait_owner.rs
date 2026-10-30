//! Native notification jobs and synchronous wake signals (25.4.3.13–15).
//!
//! Only waiter identities cross agents. Promise capabilities stay in the
//! receiver's rooted registry, and only that Realm can consume this queue.

use super::{Budget, Error, WaitTimeout};
use std::collections::VecDeque;
use std::sync::{
    Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

#[derive(Debug, Default)]
pub(crate) struct WaitOwner {
    ready: Mutex<VecDeque<usize>>,
    wake: Condvar,
    terminated: AtomicBool,
}

impl WaitOwner {
    pub(crate) fn terminate(&self) {
        // The same mutex protects the check-before-park transition, so a stop
        // arriving immediately before suspension cannot lose its wakeup.
        let _critical = self.ready.lock();
        self.terminated.store(true, Ordering::Release);
        self.wake.notify_all();
    }

    pub(crate) fn is_terminated(&self) -> bool {
        self.terminated.load(Ordering::Acquire)
    }

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
        self.wake.notify_all();
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

    /// A synchronous agent consumes only its own wake signal. Existing async
    /// jobs keep their FIFO order and run after the synchronous call returns.
    pub(crate) fn wait_for(
        &self,
        id: usize,
        timeout: Option<WaitTimeout>,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        self.wait(Some(id), timeout, budget)
    }

    pub(crate) fn sleep(
        &self,
        timeout: Option<WaitTimeout>,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        self.wait(None, timeout, budget).map(|_| ())
    }

    fn wait(
        &self,
        id: Option<usize>,
        timeout: Option<WaitTimeout>,
        budget: &mut Budget,
    ) -> Result<bool, Error> {
        let mut ready = self
            .ready
            .lock()
            .map_err(|_| Error::PoisonedSharedStorage)?;
        loop {
            if self.is_terminated() {
                return Err(Error::AgentTerminated);
            }
            budget.charge(if id.is_some() { ready.len() + 1 } else { 1 })?;
            if let Some(id) = id
                && let Some(position) = ready.iter().position(|queued| *queued == id)
            {
                ready.remove(position);
                return Ok(true);
            }
            if let Some(timeout) = timeout {
                let now = Instant::now();
                if timeout.expired(now) {
                    return Ok(false);
                }
                // A scheduling slice avoids oversized native OS deadlines; the
                // original logical deadline is retained and never shortened.
                let delay = timeout.remaining(now).min(Duration::from_secs(86_400));
                (ready, _) = self
                    .wake
                    .wait_timeout(ready, delay)
                    .map_err(|_| Error::PoisonedSharedStorage)?;
            } else {
                ready = self
                    .wake
                    .wait(ready)
                    .map_err(|_| Error::PoisonedSharedStorage)?;
            }
        }
    }

    pub(crate) fn cancel(&self, id: usize) {
        if let Ok(mut ready) = self.ready.lock() {
            ready.retain(|queued| *queued != id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preexisting_wake_signals_do_not_reorder_asynchronous_jobs() {
        let owner = WaitOwner::default();
        let mut budget = Budget::with_work_limit(None);
        for id in [4, 9, 7] {
            owner.enqueue(id, &mut budget).unwrap();
        }
        assert!(owner.wait_for(9, None, &mut budget).unwrap());
        assert_eq!(owner.front().unwrap(), Some(4));
        owner.consume(4).unwrap();
        assert_eq!(owner.front().unwrap(), Some(7));
        owner.consume(7).unwrap();
        let zero = WaitTimeout::new(0.0, Instant::now());
        assert!(!owner.wait_for(1, Some(zero), &mut budget).unwrap());
        assert!(!owner.has_ready());
    }

    #[test]
    fn work_refusal_preserves_wake_signals_and_unrelated_jobs() {
        let owner = WaitOwner::default();
        let mut budget = Budget::with_work_limit(None);
        for id in [4, 9, 7] {
            owner.enqueue(id, &mut budget).unwrap();
        }
        assert!(matches!(
            owner.wait_for(9, None, &mut Budget::new(0)),
            Err(Error::WorkLimit)
        ));
        owner.cancel(9);
        assert_eq!(owner.front().unwrap(), Some(4));
        owner.consume(4).unwrap();
        assert_eq!(owner.front().unwrap(), Some(7));
        owner.consume(7).unwrap();
        assert!(!owner.has_ready());
    }
}
