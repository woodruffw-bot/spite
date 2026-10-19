//! Logical Symbol retirement, host promotion and upgrade-gate regressions.

use super::*;
use crate::JsString;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[test]
fn logical_retirement_prevents_weak_resurrection_before_internal_storage_drops() {
    let internal = JsSymbol::new(Some(JsString::from("internal")));
    let weak = internal.downgrade();
    let pin = SymbolCollectionPin::acquire(&weak).unwrap();
    assert_eq!(internal.description(), Some(&JsString::from("internal")));
    drop(internal);
    let observed = {
        let mut guard = SymbolCollectionGuard::acquire();
        let before = (guard.strong_count(&pin), guard.is_live(&pin));
        // No external strong owner remains; only the prepared collector pin.
        guard.retire(&pin);
        (before, guard.is_live(&pin))
    };
    assert_eq!(observed, ((1, true), false));
    assert_eq!(weak.upgrade(), None);
    let mut identities = std::collections::HashSet::new();
    identities.insert(weak.clone());
    drop(pin);
    assert_eq!(weak.upgrade(), None);
    assert!(identities.contains(&weak));
}

#[test]
fn a_completed_host_upgrade_is_an_external_owner_in_the_guarded_count() {
    let internal = JsSymbol::new(None);
    let weak = internal.downgrade();
    let pin = SymbolCollectionPin::acquire(&weak).unwrap();
    let worker_weak = weak.clone();
    let external = std::thread::spawn(move || worker_weak.upgrade().unwrap())
        .join()
        .unwrap();
    let observed = {
        let guard = SymbolCollectionGuard::acquire();
        // This independent owner forbids the collector's retirement decision.
        (guard.strong_count(&pin), guard.is_live(&pin))
    };
    assert_eq!(observed, (3, true));
    assert_eq!(weak.upgrade(), Some(external.clone()));
    drop((internal, pin));
    assert_eq!(weak.upgrade(), Some(external.clone()));
    drop(external);
    assert_eq!(weak.upgrade(), None);
}

#[test]
fn a_concurrent_upgrade_cannot_cross_the_logical_retirement_commit() {
    let internal = JsSymbol::new(None);
    let weak = internal.downgrade();
    let pin = SymbolCollectionPin::acquire(&weak).unwrap();
    drop(internal);
    let start = Arc::new(AtomicBool::new(false));
    let attempted = Arc::new(AtomicBool::new(false));
    let worker_start = start.clone();
    let worker_attempted = attempted.clone();
    let worker_weak = weak.clone();
    // Spawn/allocation is before gate acquisition. Synchronization within the
    // guarded test interval uses only atomics, never a second lock or callback.
    let worker = std::thread::spawn(move || {
        while !worker_start.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
        worker_attempted.store(true, Ordering::Release);
        worker_weak.upgrade()
    });
    {
        let mut guard = SymbolCollectionGuard::acquire();
        start.store(true, Ordering::Release);
        while !attempted.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
        guard.retire(&pin);
    }
    assert_eq!(worker.join().unwrap(), None);
    // The pin still physically owns the identity: None is logical death.
    assert_eq!(weak.upgrade(), None);
    drop(pin);
}

#[test]
fn an_unwound_guard_does_not_poison_or_leave_the_gate_locked() {
    let internal = JsSymbol::new(None);
    let weak = internal.downgrade();
    // Precreate the payload and bypass the arbitrary process panic hook.
    let payload = Box::new(());
    let result = std::panic::catch_unwind(move || {
        let _guard = SymbolCollectionGuard::acquire();
        std::panic::resume_unwind(payload);
    });
    assert!(result.is_err());
    assert_eq!(weak.upgrade(), Some(internal.clone()));
}
