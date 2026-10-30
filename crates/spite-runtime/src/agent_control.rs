//! Owned native agent termination without sharing an interpreter or its heap.

use crate::{Realm, object};
use std::sync::Arc;

/// A thread-safe host control containing only native agent state.
///
/// Clones address the same agent and retain no JavaScript heap values. A control
/// can outlive its Realm. Termination is permanent and cooperative: execution
/// checks at work boundaries, and blocking `Atomics.wait` wakes to a host error.
/// Other native operations observe termination at their next execution check.
#[derive(Clone, Debug)]
pub struct AgentControl {
    owner: Arc<object::WaitOwner>,
}

impl AgentControl {
    /// Requests permanent termination, waking any synchronous atomic wait.
    ///
    /// Repeated requests have the same effect. Termination is a host failure
    /// that bypasses JavaScript catch, finally and disposal callbacks.
    pub fn terminate(&self) {
        self.owner.terminate();
    }

    /// Returns whether the host has requested this agent's termination.
    pub fn is_terminated(&self) -> bool {
        self.owner.is_terminated()
    }
}

impl Realm {
    /// Returns an owned host control for this agent without initializing it.
    ///
    /// The handle can cross native threads while this Realm stays with its
    /// caller. Requesting it preserves the current execution allowance.
    pub fn agent_control(&mut self) -> AgentControl {
        AgentControl {
            owner: self.atomic_waits.get_or_insert_default().owner().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Error, Limits, SharedBuffer, Value};
    use std::time::{Duration, Instant};

    fn shared(realm: &mut Realm) -> SharedBuffer {
        let Value::Object(buffer) = realm
            .eval("let buffer=new SharedArrayBuffer(16),a=new Int32Array(buffer);buffer")
            .unwrap()
        else {
            panic!("buffer")
        };
        realm.share_array_buffer(&buffer).unwrap()
    }

    fn await_execution(realm: &mut Realm) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while realm.eval("Atomics.load(a,1)").unwrap() != Value::Number(1.0) {
            assert!(Instant::now() < deadline, "agent did not enter execution");
            std::thread::yield_now();
        }
    }

    #[test]
    fn native_controls_preserve_idle_state_and_outlive_their_realm() {
        let mut realm = Realm::new(Limits {
            max_steps: Some(0),
            ..Limits::default()
        });
        let control = Arc::new(realm.agent_control());
        assert!(realm.intrinsics.is_none());
        assert_eq!(realm.remaining_steps, Some(0));
        assert!(!control.is_terminated());
        let first = control.clone();
        let second = control.clone();
        let first = std::thread::spawn(move || first.terminate());
        let second = std::thread::spawn(move || second.terminate());
        first.join().unwrap();
        second.join().unwrap();
        assert!(control.is_terminated());
        assert!(
            matches!(realm.eval("1"), Err(Error::Host { message, .. }) if message == "agent terminated by host")
        );
        drop(realm);
        control.terminate();
        assert!(control.is_terminated());
    }

    #[test]
    fn termination_interrupts_running_javascript_without_language_cleanup() {
        let mut parent = Realm::default();
        let buffer = shared(&mut parent);
        let (ready, control) = std::sync::mpsc::channel();
        let (finished, done) = std::sync::mpsc::channel();
        let worker = std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(move || {
            let mut realm = Realm::default();
            realm.bind_shared_array_buffer("buffer", &buffer).unwrap();
            realm.eval("let a=new Int32Array(buffer)").unwrap();
            ready.send(realm.agent_control()).unwrap();
            let result = realm.eval("try{Atomics.store(a,1,1);while(Atomics.load(a,0)===0){}}catch(e){Atomics.store(a,2,1)}finally{Atomics.store(a,3,1)}");
            finished.send(matches!(result, Err(Error::Host { message, .. }) if message == "agent terminated by host")).unwrap();
        }).unwrap();
        let control = control.recv_timeout(Duration::from_secs(5)).unwrap();
        await_execution(&mut parent);
        control.terminate();
        let stopped = done.recv_timeout(Duration::from_secs(5));
        if stopped.is_err() {
            // Failure cleanup releases the program loop; it cannot make this
            // test pass if termination failed to interrupt execution.
            parent.eval("Atomics.store(a,0,1)").unwrap();
        }
        worker.join().unwrap();
        assert!(stopped.expect("agent termination did not interrupt execution"));
        assert_eq!(parent.eval("a[2]===0&&a[3]===0"), Ok(Value::Boolean(true)));
    }

    #[test]
    fn termination_wakes_infinite_atomic_waits_and_retires_native_entries() {
        let mut parent = Realm::default();
        let buffer = shared(&mut parent);
        let (ready, control) = std::sync::mpsc::channel();
        let (finished, done) = std::sync::mpsc::channel();
        let worker = std::thread::Builder::new().stack_size(2 * 1024 * 1024).spawn(move || {
            let mut realm = Realm::new_suspending(Limits::default());
            realm.bind_shared_array_buffer("buffer", &buffer).unwrap();
            realm.eval("let a=new Int32Array(buffer)").unwrap();
            ready.send(realm.agent_control()).unwrap();
            let result = realm.eval("try{Atomics.store(a,1,1);Atomics.wait(a,0,0)}catch(e){Atomics.store(a,2,1)}finally{Atomics.store(a,3,1)}");
            finished.send(matches!(result, Err(Error::Host { message, .. }) if message == "agent terminated by host")).unwrap();
        }).unwrap();
        let control = control.recv_timeout(Duration::from_secs(5)).unwrap();
        await_execution(&mut parent);
        control.terminate();
        let stopped = done.recv_timeout(Duration::from_secs(5));
        if stopped.is_err() {
            // Wake a regressed waiter before failing, so the test does not
            // strand a native thread. A rescued wait never earns a pass.
            parent.eval("Atomics.notify(a,0)").unwrap();
        }
        worker.join().unwrap();
        assert!(stopped.expect("agent termination did not wake its wait"));
        assert_eq!(
            parent.eval("a[2]===0&&a[3]===0&&Atomics.notify(a,0)===0"),
            Ok(Value::Boolean(true))
        );
    }
}
