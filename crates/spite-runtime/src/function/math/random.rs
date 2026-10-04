//! Realm-specific pseudo-random sequences (sec-math.random).

use crate::{Error, Realm, Value};
use spite_core::Span;
use std::{
    collections::hash_map::RandomState,
    hash::BuildHasher,
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};

static PROCESS_SEED: OnceLock<u64> = OnceLock::new();
static NEXT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(super) struct RandomSequence {
    state: u64,
}

impl RandomSequence {
    pub(super) fn new(span: Span) -> Result<Self, Error> {
        let identity = reserve_sequence(&NEXT_SEQUENCE, span)?;
        // RandomState supplies the standard library's process randomization.
        // Adding a checked unique identity gives every realm a distinct initial
        // state, including realms initialized concurrently on different threads.
        let seed = *PROCESS_SEED.get_or_init(|| RandomState::new().hash_one("spite Math.random"));
        Ok(Self {
            state: seed.wrapping_add(identity),
        })
    }

    fn next_word(&mut self) -> u64 {
        // SplitMix64: an odd Weyl increment followed by a bijective mixer.
        // Algorithm: https://prng.di.unimi.it/splitmix64.c
        // Wrapping operations are intentional; the state period is 2^64.
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^ (value >> 31)
    }

    fn next(&mut self) -> f64 {
        unit_interval(self.next_word())
    }
}

fn reserve_sequence(next: &AtomicU64, span: Span) -> Result<u64, Error> {
    let mut identity = next.load(Ordering::Relaxed);
    loop {
        let following = identity.checked_add(1).ok_or_else(|| Error::Limit {
            span,
            message: "Math.random realm sequence identity capacity exhausted".into(),
        })?;
        match next.compare_exchange_weak(identity, following, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => return Ok(identity),
            Err(actual) => identity = actual,
        }
    }
}

fn unit_interval(word: u64) -> f64 {
    // Every 53-bit integer is exactly representable, as is scaling by 2^-53.
    // Discard low bits before conversion so rounding can never produce one.
    ((word >> 11) as f64) * (1.0 / ((1u64 << 53) as f64))
}

impl Realm {
    pub(in crate::function) fn math_random(&mut self) -> Value {
        let intrinsics = self.intrinsics.as_mut().expect("initialized");
        Value::Number(intrinsics.math.random.next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_matches_published_zero_seed_vectors() {
        let mut sequence = RandomSequence { state: 0 };
        for expected in [0xe220a8397b1dcdaf, 0x6e789e6aa1b965f4, 0x06c45d188009454f] {
            assert_eq!(sequence.next_word(), expected);
        }
    }

    #[test]
    fn exact_mapping_preserves_positive_zero_and_excludes_one() {
        assert_eq!(unit_interval(0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(unit_interval(2047).to_bits(), 0.0_f64.to_bits());
        assert_eq!(unit_interval(2048), 2.0_f64.powi(-53));
        assert_eq!(unit_interval(u64::MAX), 1.0 - 2.0_f64.powi(-53));
        let mut sequence = RandomSequence { state: u64::MAX };
        for _ in 0..65536 {
            let value = sequence.next();
            assert!(value.is_sign_positive() && (0.0..1.0).contains(&value));
        }
    }

    #[test]
    fn sequence_identity_capacity_is_checked_without_reuse() {
        let next = AtomicU64::new(u64::MAX - 1);
        assert_eq!(reserve_sequence(&next, Span::new(0, 0)), Ok(u64::MAX - 1));
        assert!(matches!(
            reserve_sequence(&next, Span::new(0, 0)),
            Err(Error::Limit { .. })
        ));
        assert_eq!(next.load(Ordering::Relaxed), u64::MAX);
    }

    #[test]
    fn projecting_to_numbers_retains_the_full_period_and_distinct_realm_sequences() {
        // Any proper period of the 2^64 cycle divides 2^63. An odd increment
        // advances by 2^63 in half a cycle; these two projected outputs differ,
        // ruling out every proper period. Distinct initial states therefore
        // produce distinct Number sequences, even after discarding low bits.
        let increment = 0x9e3779b97f4a7c15;
        let mut zero = RandomSequence {
            state: 0u64.wrapping_sub(increment),
        };
        let mut halfway = RandomSequence {
            state: (1u64 << 63).wrapping_sub(increment),
        };
        assert_eq!(zero.next().to_bits(), 0.0_f64.to_bits());
        assert_ne!(halfway.next().to_bits(), 0.0_f64.to_bits());
    }

    #[test]
    fn concurrently_initialized_sequences_have_distinct_starting_states() {
        let handles: Vec<_> = (0..32)
            .map(|_| std::thread::spawn(|| RandomSequence::new(Span::new(0, 0)).unwrap().state))
            .collect();
        let states: std::collections::BTreeSet<_> = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect();
        assert_eq!(states.len(), 32);
    }
}
