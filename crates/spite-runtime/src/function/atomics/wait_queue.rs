//! Rooted asynchronous waits in the current agent (25.4.3.14, NotifyWaiter).

use super::super::PromiseCapability;
use crate::{Error, Realm, Value, object};
use spite_core::{JsString, Span};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod timeout;
use timeout::WaitTimeout;

#[derive(Debug)]
struct PendingWait {
    id: usize,
    backing: object::SharedArrayBufferData,
    capability: PromiseCapability,
    _roots: [object::Root; 3],
    timeout: Option<WaitTimeout>,
}

#[derive(Debug)]
pub(crate) struct WaitRegistry {
    owner: Arc<()>,
    next_id: usize,
    pending: Vec<PendingWait>,
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
        let waits = self.atomic_waits.get_or_insert_with(|| WaitRegistry {
            owner: Arc::new(()),
            next_id: 0,
            pending: Vec::new(),
        });
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
        let notification = self.buffer_value_work(span, |_, budget| {
            backing.take_waiters(index, count, owner.as_ref(), budget)
        })?;
        let object::WaiterNotification::Local(mut batch) = notification else {
            return Err(Self::unsupported(
                span,
                "notification across agents is not implemented",
            ));
        };
        let ids = batch.take_ids();
        let count = ids.len();
        for id in ids {
            let waits = self.atomic_waits.as_mut().expect("local wait registry");
            let position = waits
                .pending
                .iter()
                .position(|wait| wait.id == id)
                .expect("local registered waiter");
            let wait = waits.pending.remove(position);
            // The batch retains the critical section. This private intrinsic
            // resolving function receives a primitive string, so it only queues
            // reactions and cannot run JS or reenter the backing mutex.
            self.call(
                wait.capability.resolve.clone(),
                Value::Undefined,
                vec![Value::String(JsString::from("ok"))],
                span,
            )?;
        }
        Ok(count)
    }

    /// Returns a scheduling hint for the next finite asynchronous wait timeout.
    ///
    /// Infinite waits have no deadline. This query does not initialize a Realm
    /// or run JavaScript; evaluation never sleeps or starts background timers.
    pub fn next_atomics_wait_delay(&self) -> Option<Duration> {
        let now = Instant::now();
        self.atomic_waits
            .as_ref()?
            .pending
            .iter()
            .filter_map(|wait| wait.timeout.map(|timeout| timeout.remaining(now)))
            .min()
    }

    /// Services one expired asynchronous wait as an idle host job.
    ///
    /// Resolving its Promise queues reactions without calling JavaScript.
    /// No due timer returns false without changing execution state. Active
    /// execution is rejected. The job uses the configured optional work budget;
    /// failure before removal preserves the waiter, while later host failures
    /// can leave the Promise unsettled and never become JavaScript rejections.
    pub fn run_next_atomics_wait_job(&mut self) -> Result<bool, Error> {
        let span = Span::new(0, 0);
        if self.call_depth != 0 || self.evaluation_depth != 0 || self.job_active {
            return Err(Error::Host {
                span,
                message: "cannot service an Atomics timeout during active execution".into(),
            });
        }
        let now = Instant::now();
        let Some((id, backing, owner, work)) = self.atomic_waits.as_ref().and_then(|waits| {
            waits
                .pending
                .iter()
                .find(|wait| wait.timeout.is_some_and(|timeout| timeout.expired(now)))
                .map(|wait| {
                    (
                        wait.id,
                        wait.backing.clone(),
                        waits.owner.clone(),
                        waits.pending.len(),
                    )
                })
        }) else {
            return Ok(false);
        };
        self.clear_kept_objects()?;
        let caller_work = self.remaining_steps;
        self.remaining_steps = self.limits.max_steps;
        self.job_active = true;
        let result = (|| {
            self.object_work(span, |_, budget| budget.charge(work + 1))?;
            let mut batch = self.buffer_value_work(span, |_, budget| {
                backing.take_timed_waiter(id, &owner, budget)
            })?;
            let removed = !batch.take_ids().is_empty();
            let waits = self.atomic_waits.as_mut().expect("timed wait registry");
            let position = waits
                .pending
                .iter()
                .position(|wait| wait.id == id)
                .expect("selected timed waiter");
            let wait = waits.pending.remove(position);
            // A previous aborted notification may already have removed the
            // native entry. Retiring that record must not invent a resolution.
            if removed {
                self.call(
                    wait.capability.resolve.clone(),
                    Value::Undefined,
                    vec![Value::String(JsString::from("timed-out"))],
                    span,
                )?;
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
        assert!(matches!(
            backing
                .take_waiters(
                    0,
                    usize::MAX,
                    None,
                    &mut object::Budget::with_work_limit(None)
                )
                .unwrap(),
            object::WaiterNotification::ForeignAgent
        ));
        drop(realm);
        let object::WaiterNotification::Local(mut batch) = backing
            .take_waiters(
                0,
                usize::MAX,
                None,
                &mut object::Budget::with_work_limit(None),
            )
            .unwrap()
        else {
            panic!("retired agent")
        };
        assert!(batch.take_ids().is_empty());
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
