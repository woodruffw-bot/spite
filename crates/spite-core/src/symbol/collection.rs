//! Workspace collector support; never an embedding rooting requirement.

use super::{JsSymbol, WeakJsSymbol};
use std::{
    hint::spin_loop,
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

static UPGRADE_GATE: AtomicBool = AtomicBool::new(false);

pub(super) struct GateGuard {
    _thread_bound: PhantomData<Rc<()>>,
}

impl Drop for GateGuard {
    fn drop(&mut self) {
        UPGRADE_GATE.store(false, Ordering::Release);
    }
}

pub(super) fn lock() -> GateGuard {
    loop {
        if UPGRADE_GATE
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            return GateGuard {
                _thread_bound: PhantomData,
            };
        }
        // No parking/yield/initialization allocation can re-enter an upgrade.
        // Only finite, prepared scans run under this gate. Contended callers
        // can spend CPU spinning; this is the initial correctness tradeoff.
        while UPGRADE_GATE.load(Ordering::Relaxed) {
            spin_loop();
        }
    }
}

/// An opaque, non-reentrant Symbol collection commit gate.
///
/// Workspace implementation interface. Do not allocate, drop owned buffers,
/// upgrade weak symbols, invoke user code, or acquire other locks while held.
/// Prepare all fallible work before acquiring it and release it before cleanup.
#[doc(hidden)]
pub struct SymbolCollectionGuard {
    _guard: GateGuard,
}

impl SymbolCollectionGuard {
    /// Excludes public weak upgrades until this guard is dropped.
    pub fn acquire() -> Self {
        Self { _guard: lock() }
    }

    /// Reads a collector pin's physical strong count without upgrading it.
    pub fn strong_count(&self, pin: &SymbolCollectionPin) -> usize {
        Arc::strong_count(&pin.0.0)
    }

    /// Tests whether the pinned identity has already been retired.
    pub fn is_live(&self, pin: &SymbolCollectionPin) -> bool {
        !pin.0.0.retired.load(Ordering::Acquire)
    }

    /// Irreversibly retires a proven unmarked, exclusively internal identity.
    ///
    /// Workspace callers must complete an allocation-free ownership/root
    /// recheck and the joint object/Symbol fixed point before this separate
    /// infallible commit pass. Never retire an independently held strong Symbol.
    pub fn retire(&mut self, pin: &SymbolCollectionPin) {
        pin.0.0.retired.store(true, Ordering::Release);
    }
}

/// Exactly one separately accounted strong owner for a collection candidate.
///
/// Workspace implementation interface. It is intentionally not Clone. Acquire
/// pins outside the gate, subtract each pin once in ownership accounting, and
/// release all pins outside the gate even when collection fails.
#[doc(hidden)]
pub struct SymbolCollectionPin(JsSymbol);

impl SymbolCollectionPin {
    /// Pins a currently live identity, without reviving a retired one.
    pub fn acquire(symbol: &WeakJsSymbol) -> Option<Self> {
        symbol.upgrade().map(Self)
    }
}

#[cfg(test)]
mod tests;
