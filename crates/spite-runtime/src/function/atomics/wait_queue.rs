//! Rooted asynchronous waits in the current agent (25.4.3.14, NotifyWaiter).

use super::super::PromiseCapability;
use crate::{Error, Realm, Value, object};
use spite_core::{JsString, Span};
use std::sync::Arc;

#[derive(Debug)]
struct PendingWait {
    id: usize,
    backing: object::SharedArrayBufferData,
    capability: PromiseCapability,
    _roots: [object::Root; 3],
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
    pub(super) fn register_atomic_wait(
        &mut self,
        backing: object::SharedArrayBufferData,
        index: u64,
        access: &object::shared_page::PreparedAtomic,
        expected: u64,
        capability: PromiseCapability,
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
