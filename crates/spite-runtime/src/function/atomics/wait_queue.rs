//! Rooted asynchronous waits and native notification jobs (25.4.3.14–15).

use super::super::PromiseCapability;
use crate::{Error, Realm, Value, object};
use spite_core::{JsString, Span};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod blocking;
use object::WaitTimeout;

#[derive(Debug)]
struct PendingWait {
    id: usize,
    backing: object::SharedArrayBufferData,
    capability: PromiseCapability,
    _roots: [object::Root; 3],
    timeout: Option<WaitTimeout>,
}

#[derive(Debug, Default)]
pub(crate) struct WaitRegistry {
    owner: Arc<object::WaitOwner>,
    next_id: usize,
    pending: Vec<PendingWait>,
}

impl WaitRegistry {
    pub(crate) fn owner(&self) -> &Arc<object::WaitOwner> {
        &self.owner
    }
}

enum WaitJob {
    Notified(usize),
    TimedOut(usize, object::SharedArrayBufferData),
}

impl Drop for WaitRegistry {
    fn drop(&mut self) {
        // Retiring an agent removes its waiters without retaining a Realm or
        // exposing its heap values through shared native backing storage.
        for wait in &self.pending {
            wait.backing.cancel_waiters(&self.owner);
        }
    }
}

impl Realm {
    #[expect(
        clippy::too_many_arguments,
        reason = "validated storage, capability and timeout"
    )]
    pub(super) fn register_atomic_wait(
        &mut self,
        backing: object::SharedArrayBufferData,
        index: u64,
        access: &object::shared_page::PreparedAtomic,
        expected: u64,
        capability: PromiseCapability,
        timeout: f64,
        span: Span,
    ) -> Result<bool, Error> {
        let roots = [
            self.root_host_value(&capability.promise, span)?,
            self.root_host_value(&capability.resolve, span)?,
            self.root_host_value(&capability.reject, span)?,
        ];
        let moved = self.atomic_waits.as_ref().map_or(1, |waits| {
            if waits.pending.len() == waits.pending.capacity() {
                waits.pending.len() + 1
            } else {
                1
            }
        });
        self.object_work(span, |_, budget| budget.charge(moved))?;
        let waits = self.atomic_waits.get_or_insert_default();
        waits.pending.try_reserve(1).map_err(|_| Error::Limit {
            span,
            message: "asynchronous waiter capacity exceeded".into(),
        })?;
        let id = waits.next_id;
        let next_id = id.checked_add(1).ok_or_else(|| Error::Limit {
            span,
            message: "asynchronous waiter identity capacity exceeded".into(),
        })?;
        let owner = waits.owner.clone();
        let registered = self.buffer_value_work(span, |_, budget| {
            backing.register_waiter(index, access, expected, &owner, id, budget)
        })?;
        if !registered {
            return Ok(false);
        }
        let waits = self.atomic_waits.as_mut().expect("prepared registry");
        waits.next_id = next_id;
        waits.pending.push(PendingWait {
            id,
            backing,
            capability,
            _roots: roots,
            timeout: timeout
                .is_finite()
                .then(|| WaitTimeout::new(timeout, Instant::now())),
        });
        Ok(true)
    }

    pub(super) fn notify_atomic_waiters(
        &mut self,
        backing: &object::SharedArrayBufferData,
        index: u64,
        count: usize,
        span: Span,
    ) -> Result<usize, Error> {
        let owner = self.atomic_waits.as_ref().map(|waits| waits.owner.clone());
        let mut batch =
            self.buffer_value_work(span, |_, budget| backing.take_waiters(index, count, budget))?;
        let targets = batch.take_targets();
        let count = targets.len();
        for (target, id) in targets {
            if owner
                .as_ref()
                .is_some_and(|owner| Arc::ptr_eq(owner, &target))
            {
                // NotifyWaiter resolves locally inside the critical section.
                // A primitive result cannot run JS or reenter the backing lock.
                self.fulfill_atomic_wait(id, "ok", span)?;
            } else {
                // EnqueueResolveInAgentJob carries only an integer identity.
                // The target owns and roots its capability in its own heap.
                self.object_work(span, |_, budget| target.enqueue(id, budget))?;
            }
        }
        Ok(count)
    }

    fn fulfill_atomic_wait(&mut self, id: usize, result: &str, span: Span) -> Result<(), Error> {
        let waits = self.atomic_waits.as_mut().expect("local wait registry");
        let position = waits
            .pending
            .iter()
            .position(|wait| wait.id == id)
            .expect("local registered waiter");
        let wait = waits.pending.remove(position);
        self.call(
            wait.capability.resolve.clone(),
            Value::Undefined,
            vec![Value::String(JsString::from(result))],
            span,
        )?;
        Ok(())
    }

    /// Returns a scheduling hint for the next asynchronous wait job.
    ///
    /// Queued notifications return zero; unnotified infinite waits have no deadline.
    /// This query does not initialize a Realm
    /// or run JavaScript; evaluation never sleeps or starts background timers.
    pub fn next_atomics_wait_delay(&self) -> Option<Duration> {
        let now = Instant::now();
        let waits = self.atomic_waits.as_ref()?;
        if waits.owner.has_ready() {
            return Some(Duration::ZERO);
        }
        waits
            .pending
            .iter()
            .filter_map(|wait| wait.timeout.map(|timeout| timeout.remaining(now)))
            .min()
    }

    /// Services one foreign notification or expired wait as an idle host job.
    ///
    /// Resolving its Promise queues reactions without calling JavaScript.
    /// No ready job returns false without changing execution state. Active
    /// execution is rejected. The job uses the configured optional work budget;
    /// failure before removal preserves the waiter, while later host failures
    /// can leave the Promise unsettled and never become JavaScript rejections.
    pub fn run_next_atomics_wait_job(&mut self) -> Result<bool, Error> {
        let span = Span::new(0, 0);
        if self.call_depth != 0 || self.evaluation_depth != 0 || self.job_active {
            return Err(Error::Host {
                span,
                message: "cannot service an Atomics wait job during active execution".into(),
            });
        }
        let Some(waits) = self.atomic_waits.as_ref() else {
            return Ok(false);
        };
        let ready = waits.owner.front().map_err(|error| Error::Host {
            span,
            message: error.to_string(),
        })?;
        let now = Instant::now();
        let job = if let Some(id) = ready {
            WaitJob::Notified(id)
        } else if let Some(wait) = waits
            .pending
            .iter()
            .find(|wait| wait.timeout.is_some_and(|timeout| timeout.expired(now)))
        {
            WaitJob::TimedOut(wait.id, wait.backing.clone())
        } else {
            return Ok(false);
        };
        let owner = waits.owner.clone();
        let work = waits.pending.len();
        self.clear_kept_objects()?;
        let caller_work = self.remaining_steps;
        self.remaining_steps = self.limits.max_steps;
        self.job_active = true;
        let result = (|| {
            self.object_work(span, |_, budget| budget.charge(work + 1))?;
            match job {
                WaitJob::Notified(id) => {
                    self.object_work(span, |_, _| owner.consume(id))?;
                    self.fulfill_atomic_wait(id, "ok", span)?;
                }
                WaitJob::TimedOut(id, backing) => {
                    let mut batch = self.buffer_value_work(span, |_, budget| {
                        backing.take_timed_waiter(id, &owner, budget)
                    })?;
                    if batch.take_targets().is_empty() {
                        // A foreign notification can win after job selection.
                        // Keep its rooted capability for the queued delivery.
                        // An earlier aborted host operation may also leave this
                        // Promise unsettled; neither case invents a timeout.
                        let wait = self
                            .atomic_waits
                            .as_mut()
                            .expect("wait registry")
                            .pending
                            .iter_mut()
                            .find(|wait| wait.id == id)
                            .expect("selected timed waiter");
                        wait.timeout = None;
                    } else {
                        self.fulfill_atomic_wait(id, "timed-out", span)?;
                    }
                }
            }
            Ok(true)
        })();
        self.job_active = false;
        self.remaining_steps = caller_work;
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_jobs_are_inert_without_due_work_and_reject_active_execution() {
        let mut realm = Realm::new(crate::Limits {
            max_steps: Some(0),
            ..crate::Limits::default()
        });
        assert!(realm.next_atomics_wait_delay().is_none());
        assert!(!realm.run_next_atomics_wait_job().unwrap());
        assert!(realm.intrinsics.is_none());
        assert_eq!(realm.remaining_steps, Some(0));
        realm.call_depth = 1;
        assert!(matches!(
            realm.run_next_atomics_wait_job(),
            Err(Error::Host { .. })
        ));
        realm.call_depth = 0;
        realm.evaluation_depth = 1;
        assert!(matches!(
            realm.run_next_atomics_wait_job(),
            Err(Error::Host { .. })
        ));
        realm.evaluation_depth = 0;
        realm.job_active = true;
        assert!(matches!(
            realm.run_next_atomics_wait_job(),
            Err(Error::Host { .. })
        ));
    }

    #[test]
    fn timer_job_work_refusal_preserves_waiter_and_restores_caller_allowance() {
        let mut realm = Realm::default();
        realm.eval("let a=new Int32Array(new SharedArrayBuffer(8)),result='';Atomics.waitAsync(a,0,0,1).value.then(v=>result=v);").unwrap();
        std::thread::sleep(Duration::from_millis(2));
        realm.limits.max_steps = Some(0);
        realm.remaining_steps = Some(77);
        assert!(matches!(
            realm.run_next_atomics_wait_job(),
            Err(Error::Limit { .. })
        ));
        assert_eq!(realm.remaining_steps, Some(77));
        assert!(!realm.job_active);
        realm.limits.max_steps = None;
        assert_eq!(realm.eval("Atomics.notify(a,0)"), Ok(Value::Number(1.0)));
        assert!(realm.next_atomics_wait_delay().is_none());
        realm.run_next_promise_job().unwrap();
        assert_eq!(realm.eval("result==='ok'"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn a_removed_native_waiter_keeps_its_capability_for_a_later_foreign_delivery() {
        let mut realm = Realm::default();
        realm.eval("let a=new Int32Array(new SharedArrayBuffer(8)),result='';Atomics.waitAsync(a,0,0,1).value.then(v=>result=v)").unwrap();
        let waits = realm.atomic_waits.as_ref().unwrap();
        let backing = waits.pending[0].backing.clone();
        let owner = waits.owner.clone();
        let id = waits.pending[0].id;
        // Simulate a timeout selection overtaken by native waiter removal.
        let mut batch = backing
            .take_waiters(0, 1, &mut object::Budget::with_work_limit(None))
            .unwrap();
        assert_eq!(batch.take_targets()[0].1, id);
        drop(batch);
        std::thread::sleep(Duration::from_millis(2));
        assert!(realm.run_next_atomics_wait_job().unwrap());
        assert!(realm.next_atomics_wait_delay().is_none());
        assert_eq!(realm.atomic_waits.as_ref().unwrap().pending.len(), 1);
        assert_eq!(realm.eval("result===''"), Ok(Value::Boolean(true)));
        owner
            .enqueue(id, &mut object::Budget::with_work_limit(None))
            .unwrap();
        assert!(realm.run_next_atomics_wait_job().unwrap());
        realm.run_next_promise_job().unwrap();
        assert_eq!(realm.eval("result==='ok'"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn dropping_the_realm_retires_its_native_waiters() {
        let mut realm = Realm::default();
        let Value::Object(buffer) = realm.eval("let b=new SharedArrayBuffer(8);b").unwrap() else {
            panic!("buffer")
        };
        let backing = realm
            .object_work(Span::new(0, 0), |objects, _| {
                Ok(objects
                    .inspect(&buffer)?
                    .shared_array_buffer_data()
                    .unwrap()
                    .clone())
            })
            .unwrap();
        realm
            .eval("Atomics.waitAsync(new Int32Array(b),0,0)")
            .unwrap();
        // Keep the native owner alive independently while testing registry Drop.
        let owner = realm.atomic_waits.as_ref().unwrap().owner.clone();
        drop(realm);
        let mut batch = backing
            .take_waiters(0, usize::MAX, &mut object::Budget::with_work_limit(None))
            .unwrap();
        assert!(batch.take_targets().is_empty());
        assert!(!owner.has_ready());
    }

    #[test]
    fn pending_wait_roots_reactions_until_notification_and_releases_after_the_job() {
        let mut realm = Realm::default();
        realm.eval("let a=new Int32Array(new SharedArrayBuffer(8)),weak,log='';function keep(value){return ()=>{log+=value.tag}}{let token={tag:'ok'};weak=new WeakRef(token);Atomics.waitAsync(a,0,0).value.then(keep(token));}0").unwrap();
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("weak.deref()!==undefined"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(realm.eval("Atomics.notify(a,0)"), Ok(Value::Number(1.0)));
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        realm.run_next_promise_job().unwrap();
        assert_eq!(realm.eval("log==='ok'"), Ok(Value::Boolean(true)));
        realm.clear_kept_objects().unwrap();
        realm.collect(usize::MAX).unwrap();
        assert_eq!(
            realm.eval("weak.deref()===undefined"),
            Ok(Value::Boolean(true))
        );
    }
}
