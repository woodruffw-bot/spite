//! Shared data-block identity and serialized growable lengths (25.2).
//!
//! The exposed region starts at zero. Shared byte access is a separate boundary;
//! constructor/getter/grow operations retain one backing identity across clones.

use super::*;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct SharedState {
    byte_length: u64,
    max_byte_length: Option<u64>,
}

#[derive(Clone, Debug)]
pub(crate) struct SharedArrayBufferData(Arc<Mutex<SharedState>>);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SharedBufferError {
    Poisoned,
    NotGrowable,
    OutOfBounds,
}

impl SharedArrayBufferData {
    fn new(byte_length: u64, max_byte_length: Option<u64>) -> Self {
        Self(Arc::new(Mutex::new(SharedState {
            byte_length,
            max_byte_length,
        })))
    }

    pub(crate) fn metadata(&self) -> Result<(u64, Option<u64>), SharedBufferError> {
        let state = self.0.lock().map_err(|_| SharedBufferError::Poisoned)?;
        Ok((state.byte_length, state.max_byte_length))
    }

    pub(crate) fn grow(&self, new_length: u64) -> Result<(), SharedBufferError> {
        let mut state = self.0.lock().map_err(|_| SharedBufferError::Poisoned)?;
        let maximum = state
            .max_byte_length
            .ok_or(SharedBufferError::NotGrowable)?;
        // HostGrowSharedArrayBuffer totally orders racing grows. A later smaller
        // grow must fail rather than shrink or silently lose the earlier grow.
        if new_length < state.byte_length || new_length > maximum {
            return Err(SharedBufferError::OutOfBounds);
        }
        state.byte_length = new_length;
        Ok(())
    }
}

impl Objects {
    pub(crate) fn create_shared_array_buffer(
        &mut self,
        prototype: &Handle,
        byte_length: u64,
        max_byte_length: Option<u64>,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
        budget.charge(1)?;
        let buffer = self.create(Some(prototype))?;
        self.object_mut(&buffer)?.shared_array_buffer =
            Some(SharedArrayBufferData::new(byte_length, max_byte_length));
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloned_backings_share_totally_ordered_racing_grows_without_lost_updates() {
        let data = SharedArrayBufferData::new(0, Some(1024));
        let threads: Vec<_> = (0..4)
            .map(|thread| {
                let data = data.clone();
                std::thread::spawn(move || {
                    for length in (1..=256).map(|n| n * 4 - thread) {
                        assert!(matches!(
                            data.grow(length),
                            Ok(()) | Err(SharedBufferError::OutOfBounds)
                        ));
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        assert_eq!(data.metadata().unwrap(), (1024, Some(1024)));
        assert_eq!(data.grow(1023), Err(SharedBufferError::OutOfBounds));
        assert_eq!(data.grow(1024), Ok(()));
    }
}
