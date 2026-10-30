//! Positive finite timeouts without overflowing the native monotonic clock.

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub(super) struct WaitTimeout {
    started: Instant,
    millis: u128,
}

impl WaitTimeout {
    pub(super) fn new(millis: f64, started: Instant) -> Self {
        debug_assert!(millis.is_finite() && millis > 0.0);
        // Round up, providing nonnegative implementation padding (DoWait).
        // Saturation is beyond Duration's entire elapsed range, so even huge
        // finite JS Numbers cannot expire early or overflow Instant addition.
        Self {
            started,
            millis: millis.ceil() as u128,
        }
    }

    pub(super) fn expired(self, now: Instant) -> bool {
        now.saturating_duration_since(self.started).as_millis() >= self.millis
    }

    pub(super) fn remaining(self, now: Instant) -> Duration {
        let millis = self
            .millis
            .saturating_sub(now.saturating_duration_since(self.started).as_millis());
        // This is a host scheduling hint, not a truncated logical deadline.
        Duration::from_millis(millis.min(u128::from(u64::MAX)) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submillisecond_requests_round_up_without_expiring_early() {
        let start = Instant::now();
        for millis in [f64::from_bits(1), 0.1, 0.999, 1.0] {
            let timeout = WaitTimeout::new(millis, start);
            assert!(!timeout.expired(start + Duration::from_micros(999)));
            assert!(timeout.expired(start + Duration::from_millis(1)));
        }
        let timeout = WaitTimeout::new(1.1, start);
        assert!(!timeout.expired(start + Duration::from_micros(1999)));
        assert!(timeout.expired(start + Duration::from_millis(2)));
    }

    #[test]
    fn huge_finite_values_keep_their_logical_deadline_and_only_bound_the_hint() {
        let start = Instant::now();
        for millis in [u64::MAX as f64, u128::MAX as f64, f64::MAX] {
            let timeout = WaitTimeout::new(millis, start);
            assert!(!timeout.expired(start + Duration::from_secs(1)));
            assert_eq!(timeout.remaining(start), Duration::from_millis(u64::MAX));
        }
    }

    #[test]
    fn remaining_delay_saturates_and_backwards_clock_observations_do_not_expire() {
        let start = Instant::now();
        let timeout = WaitTimeout::new(3.0, start + Duration::from_secs(1));
        assert!(!timeout.expired(start));
        assert_eq!(timeout.remaining(start), Duration::from_millis(3));
        assert_eq!(
            timeout.remaining(start + Duration::from_millis(1001)),
            Duration::from_millis(2)
        );
        assert_eq!(
            timeout.remaining(start + Duration::from_millis(1003)),
            Duration::ZERO
        );
    }
}
