//! Deterministic unit instrumentation, absent from production builds.

use super::super::Objects;
use std::cell::RefCell;

type Hook = Box<dyn FnOnce()>;

thread_local! {
    static BEFORE: RefCell<Option<Hook>> = const { RefCell::new(None) };
    static AFTER: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

struct Reset {
    before: Option<Hook>,
    after: Option<Hook>,
}
impl Drop for Reset {
    fn drop(&mut self) {
        // Drop captured payloads after the RefCell borrow and outside any gate.
        let before = BEFORE.with(|slot| slot.replace(self.before.take()));
        let after = AFTER.with(|slot| slot.replace(self.after.take()));
        drop((before, after));
    }
}

pub(super) fn before_recheck() {
    let callback = BEFORE.with(|slot| slot.borrow_mut().take());
    if let Some(callback) = callback {
        callback();
    }
}

pub(super) fn after_commit() {
    let callback = AFTER.with(|slot| slot.borrow_mut().take());
    if let Some(callback) = callback {
        callback();
    }
}

impl Objects {
    // Callbacks record observations rather than asserting/panicking inside a
    // committed test collection. Test assertions belong after collect returns.
    // This exposes no production API, callback point or quota allowance.
    pub(crate) fn with_symbol_gc_test_hooks<T>(
        before: Option<Hook>,
        after: Option<Hook>,
        run: impl FnOnce() -> T,
    ) -> T {
        let old_before = BEFORE.with(|slot| slot.replace(before));
        let old_after = AFTER.with(|slot| slot.replace(after));
        let _reset = Reset {
            before: old_before,
            after: old_after,
        };
        run()
    }
}
