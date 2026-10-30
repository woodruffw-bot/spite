//! Synchronous DoWait and SuspendThisAgent using native wake signals.

use super::WaitRegistry;
use crate::{Error, Realm, Value, object};
use spite_core::Span;
use std::sync::Arc;
use std::time::Instant;

struct BlockingWait {
    backing: object::SharedArrayBufferData,
    owner: Arc<object::WaitOwner>,
    id: usize,
}

impl Drop for BlockingWait {
    fn drop(&mut self) {
        // Acquiring the backing lock first waits for any selected notification
        // to enqueue its signal. Cancel only this wait, including on host abort.
        self.backing.cancel_waiter(self.id, &self.owner);
        self.owner.cancel(self.id);
    }
}

impl Realm {
    pub(in crate::function::atomics) fn blocking_atomic_wait(
        &mut self,
        backing: object::SharedArrayBufferData,
        index: u64,
        access: &object::shared_page::PreparedAtomic,
        expected: u64,
        timeout: f64,
        span: Span,
    ) -> Result<Value, Error> {
        self.object_work(span, |_, budget| budget.charge(1))?;
        let waits = self.atomic_waits.get_or_insert_with(|| WaitRegistry {
            owner: Arc::new(object::WaitOwner::default()),
            next_id: 0,
            pending: Vec::new(),
        });
        let id = waits.next_id;
        waits.next_id = id.checked_add(1).ok_or_else(|| Error::Limit {
            span,
            message: "waiter identity capacity exceeded".into(),
        })?;
        let owner = waits.owner.clone();
        if !self.buffer_value_work(span, |_, budget| {
            backing.register_waiter(index, access, expected, &owner, id, budget)
        })? {
            return Ok(Value::String("not-equal".into()));
        }
        let _registration = BlockingWait {
            backing: backing.clone(),
            owner: owner.clone(),
            id,
        };
        let timeout = timeout
            .is_finite()
            .then(|| object::WaitTimeout::new(timeout, Instant::now()));
        let notified = self.object_work(span, |_, budget| owner.wait_for(id, timeout, budget))?;
        // SuspendThisAgent reenters the original critical section before JS
        // resumes, including after a notification. Timeout and notify compete
        // here; synchronous zero timeouts also register and participate.
        let mut batch = self.buffer_value_work(span, |_, budget| {
            backing.take_timed_waiter(id, &owner, budget)
        })?;
        let timed_out = !batch.take_targets().is_empty();
        drop(batch);
        if notified {
            return Ok(Value::String("ok".into()));
        }
        if timed_out {
            return Ok(Value::String("timed-out".into()));
        }
        // The winning notifier enqueued before releasing its critical section.
        // A host-aborted notification may have removed without delivering; this
        // remains a host failure, never an invented JavaScript wait result.
        let immediate = object::WaitTimeout::new(0.0, Instant::now());
        if self.object_work(span, |_, budget| {
            owner.wait_for(id, Some(immediate), budget)
        })? {
            Ok(Value::String("ok".into()))
        } else {
            Err(Error::Host {
                span,
                message: "blocking waiter was removed without a wake notification".into(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_refusal_retires_only_the_blocking_registration_and_its_wake_signal() {
        let mut realm = Realm::default();
        let Value::Object(buffer) = realm.eval("new SharedArrayBuffer(8)").unwrap() else {
            panic!("buffer")
        };
        let backing = realm
            .objects
            .inspect(&buffer)
            .unwrap()
            .shared_array_buffer_data()
            .unwrap()
            .clone();
        let owner = Arc::new(object::WaitOwner::default());
        let mut budget = object::Budget::with_work_limit(None);
        for (index, id) in [(0, 1), (4, 2)] {
            let access = backing.prepare_atomic(index, 4, &mut budget).unwrap();
            assert!(
                backing
                    .register_waiter(index, &access, 0, &owner, id, &mut budget)
                    .unwrap()
            );
        }
        let registration = BlockingWait {
            backing: backing.clone(),
            owner: owner.clone(),
            id: 1,
        };
        let mut batch = backing.take_waiters(0, 1, &mut budget).unwrap();
        assert_eq!(batch.take_targets()[0].1, 1);
        drop(batch);
        owner.enqueue(1, &mut budget).unwrap();
        assert!(matches!(
            owner.wait_for(1, None, &mut object::Budget::new(0)),
            Err(object::Error::WorkLimit)
        ));
        drop(registration);
        assert!(!owner.has_ready());
        let mut batch = backing.take_waiters(4, 1, &mut budget).unwrap();
        assert_eq!(batch.take_targets()[0].1, 2);
    }
}
