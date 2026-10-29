//! Shared data-block identity, byte access and growable lengths (25.2).
//!
//! Byte access and growth use one lock without running JavaScript while held.
//! Backing clones retain their identity and the same implicit-zero storage.

use super::*;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct SharedState {
    data: ArrayBufferData,
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
    fn new(byte_length: u64, max_byte_length: Option<u64>) -> Result<Self, ArrayBufferError> {
        Ok(Self(Arc::new(Mutex::new(SharedState {
            data: ArrayBufferData::new(byte_length, max_byte_length)?,
        }))))
    }

    pub(super) fn with_data<T>(
        &self,
        read: impl FnOnce(&ArrayBufferData) -> Result<T, ArrayBufferError>,
    ) -> Result<T, ArrayBufferError> {
        let state = self.0.lock().map_err(|_| Error::PoisonedSharedStorage)?;
        read(&state.data)
    }

    pub(super) fn with_data_mut<T>(
        &self,
        write: impl FnOnce(&mut ArrayBufferData) -> Result<T, ArrayBufferError>,
    ) -> Result<T, ArrayBufferError> {
        let mut state = self.0.lock().map_err(|_| Error::PoisonedSharedStorage)?;
        write(&mut state.data)
    }

    pub(super) fn same_backing(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub(crate) fn metadata(&self) -> Result<(u64, Option<u64>), SharedBufferError> {
        let state = self.0.lock().map_err(|_| SharedBufferError::Poisoned)?;
        Ok((
            state.data.byte_length(),
            state
                .data
                .is_resizable()
                .then(|| state.data.max_byte_length()),
        ))
    }

    pub(crate) fn grow(&self, new_length: u64) -> Result<(), SharedBufferError> {
        let mut state = self.0.lock().map_err(|_| SharedBufferError::Poisoned)?;
        state
            .data
            .grow_shared(new_length)
            .map_err(|error| match error {
                ArrayBufferError::NotResizable => SharedBufferError::NotGrowable,
                ArrayBufferError::OutOfBounds => SharedBufferError::OutOfBounds,
                _ => unreachable!("attached shared backing growth"),
            })
    }
}

impl Objects {
    pub(crate) fn create_shared_array_buffer(
        &mut self,
        prototype: &Handle,
        byte_length: u64,
        max_byte_length: Option<u64>,
        budget: &mut Budget,
    ) -> Result<Handle, ArrayBufferError> {
        let data = SharedArrayBufferData::new(byte_length, max_byte_length)?;
        budget.charge(1)?;
        let buffer = self.create(Some(prototype))?;
        self.object_mut(&buffer)?.shared_array_buffer = Some(data);
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unlimited() -> Budget {
        Budget::with_work_limit(None)
    }

    #[test]
    fn distinct_wrappers_with_one_shared_backing_refuse_raw_self_copy() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let left = objects
            .create_shared_array_buffer(&prototype, 8, None, &mut unlimited())
            .unwrap();
        let data = objects
            .inspect(&left)
            .unwrap()
            .shared_array_buffer_data()
            .unwrap()
            .clone();
        let right = objects.create(Some(&prototype)).unwrap();
        objects.object_mut(&right).unwrap().shared_array_buffer = Some(data);
        assert_ne!(left, right);
        assert!(objects.same_buffer_data(&left, &right).unwrap());
        for count in [0, 8] {
            assert_eq!(
                objects.copy_array_buffer_bytes(&left, 0, &right, 0, count, &mut unlimited()),
                Err(ArrayBufferError::SameBuffer)
            );
        }
    }

    #[test]
    fn concurrent_cross_page_elements_do_not_tear_and_growth_retains_zero_tail() {
        let data = SharedArrayBufferData::new(8192, Some(16384)).unwrap();
        let threads: Vec<_> = (1..=4u8)
            .map(|value| {
                let data = data.clone();
                std::thread::spawn(move || {
                    for _ in 0..128 {
                        data.with_data_mut(|data| {
                            data.write_bytes(4093, &[value; 8], &mut unlimited())
                        })
                        .unwrap();
                        let bytes = data
                            .with_data(|data| data.read_bytes(4093, 8, &mut unlimited()))
                            .unwrap();
                        assert!((1..=4).contains(&bytes[0]));
                        assert!(bytes.iter().all(|byte| *byte == bytes[0]));
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        let before = data
            .with_data(|data| data.read_bytes(4093, 8, &mut unlimited()))
            .unwrap();
        data.grow(16384).unwrap();
        assert_eq!(
            data.with_data(|data| data.read_bytes(4093, 8, &mut unlimited()))
                .unwrap(),
            before
        );
        assert_eq!(
            data.with_data(|data| data.read_bytes(8192, 8, &mut unlimited()))
                .unwrap(),
            [0; 8]
        );
    }

    #[test]
    fn refused_cross_page_write_preserves_bytes_and_releases_shared_lock() {
        let data = SharedArrayBufferData::new(8192, Some(16384)).unwrap();
        data.with_data_mut(|data| data.write_bytes(4093, &[2; 3], &mut unlimited()))
            .unwrap();
        assert_eq!(
            data.with_data_mut(|data| data.write_bytes(4093, &[3; 8], &mut Budget::new(100))),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        );
        let clone = data.clone();
        std::thread::spawn(move || {
            assert_eq!(
                clone
                    .with_data(|data| data.read_bytes(4093, 8, &mut unlimited()))
                    .unwrap(),
                [2, 2, 2, 0, 0, 0, 0, 0]
            );
            clone.grow(16384).unwrap();
        })
        .join()
        .unwrap();
        assert_eq!(data.metadata().unwrap(), (16384, Some(16384)));
    }

    #[test]
    fn cloned_backings_share_totally_ordered_racing_grows_without_lost_updates() {
        let data = SharedArrayBufferData::new(0, Some(1024)).unwrap();
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
