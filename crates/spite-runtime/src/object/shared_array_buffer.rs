//! Shared data-block identity, byte access and growable lengths (25.2).
//!
//! Directory preparation, ordinary views and growth use one lock without JS.
//! Prepared atomic accesses own their pages and run after releasing that lock.
//! Backing clones retain their identity and the same implicit-zero storage.

use super::*;
use std::sync::{Arc, Mutex, Weak};

#[derive(Debug)]
struct SharedState {
    data: ArrayBufferData,
    waiters: Vec<Waiter>,
}

#[derive(Debug)]
struct Waiter {
    index: u64,
    owner: Weak<WaitOwner>,
    id: usize,
}

/// Retain the critical section through local resolution and foreign enqueueing.
pub(crate) struct WaiterBatch<'a> {
    targets: Vec<(Arc<WaitOwner>, usize)>,
    _critical: Option<std::sync::MutexGuard<'a, SharedState>>,
}

impl WaiterBatch<'_> {
    pub(crate) fn take_targets(&mut self) -> Vec<(Arc<WaitOwner>, usize)> {
        std::mem::take(&mut self.targets)
    }
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
            data: ArrayBufferData::new_shared(byte_length, max_byte_length)?,
            waiters: Vec::new(),
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

    pub(crate) fn prepare_atomic(
        &self,
        index: u64,
        width: usize,
        budget: &mut Budget,
    ) -> Result<super::shared_page::PreparedAtomic, ArrayBufferError> {
        self.with_data_mut(|data| data.prepare_atomic(index, width, budget))
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

    /// Compare and register in the same waiter-list critical section (25.4.3).
    pub(crate) fn register_waiter(
        &self,
        index: u64,
        access: &super::shared_page::PreparedAtomic,
        expected: u64,
        owner: &Arc<WaitOwner>,
        id: usize,
        budget: &mut Budget,
    ) -> Result<bool, ArrayBufferError> {
        let mut state = self.0.lock().map_err(|_| Error::PoisonedSharedStorage)?;
        budget.charge(1)?;
        let value = access.update(|old| Ok::<_, ArrayBufferError>((None, old)))?;
        if value != expected {
            return Ok(false);
        }
        budget.charge(if state.waiters.len() == state.waiters.capacity() {
            state.waiters.len() + 1
        } else {
            1
        })?;
        state
            .waiters
            .try_reserve(1)
            .map_err(|_| ArrayBufferError::Capacity)?;
        state.waiters.push(Waiter {
            index,
            owner: Arc::downgrade(owner),
            id,
        });
        Ok(true)
    }

    /// Prepare removal before mutating the list and retain selected live owners.
    pub(crate) fn take_waiters(
        &self,
        index: u64,
        count: usize,
        budget: &mut Budget,
    ) -> Result<WaiterBatch<'_>, ArrayBufferError> {
        if count == 0 {
            return Ok(WaiterBatch {
                targets: Vec::new(),
                _critical: None,
            });
        }
        let mut state = self.0.lock().map_err(|_| Error::PoisonedSharedStorage)?;
        let count = count.min(state.waiters.len());
        budget.charge(state.waiters.len())?;
        budget.charge(count)?;
        let mut selected = Vec::new();
        selected
            .try_reserve_exact(count)
            .map_err(|_| ArrayBufferError::Capacity)?;
        for (owner, id) in state
            .waiters
            .iter()
            .filter(|waiter| waiter.index == index)
            .filter_map(|waiter| waiter.owner.upgrade().map(|owner| (owner, waiter.id)))
            .take(count)
        {
            selected.push((owner, id));
        }
        let mut remaining = selected.len();
        state.waiters.retain(|waiter| {
            if waiter.owner.strong_count() == 0 {
                false
            } else if waiter.index == index && remaining != 0 {
                remaining -= 1;
                false
            } else {
                true
            }
        });
        Ok(WaiterBatch {
            targets: selected,
            _critical: Some(state),
        })
    }

    pub(crate) fn cancel_waiters(&self, owner: &Arc<WaitOwner>) {
        if let Ok(mut state) = self.0.lock() {
            let owner = Arc::downgrade(owner);
            state.waiters.retain(|waiter| !owner.ptr_eq(&waiter.owner));
        }
    }

    pub(crate) fn take_timed_waiter(
        &self,
        id: usize,
        owner: &Arc<WaitOwner>,
        budget: &mut Budget,
    ) -> Result<WaiterBatch<'_>, ArrayBufferError> {
        let mut state = self.0.lock().map_err(|_| Error::PoisonedSharedStorage)?;
        budget.charge(state.waiters.len() + 1)?;
        let weak_owner = Arc::downgrade(owner);
        let Some(position) = state
            .waiters
            .iter()
            .position(|waiter| waiter.id == id && weak_owner.ptr_eq(&waiter.owner))
        else {
            return Ok(WaiterBatch {
                targets: Vec::new(),
                _critical: None,
            });
        };
        let mut targets = Vec::new();
        targets
            .try_reserve_exact(1)
            .map_err(|_| ArrayBufferError::Capacity)?;
        budget.charge(state.waiters.len() - position)?;
        state.waiters.remove(position);
        targets.push((owner.clone(), id));
        Ok(WaiterBatch {
            targets,
            _critical: Some(state),
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
        Ok(self.create_shared_array_buffer_from_data(prototype, data, budget)?)
    }

    pub(crate) fn create_shared_array_buffer_from_data(
        &mut self,
        prototype: &Handle,
        data: SharedArrayBufferData,
        budget: &mut Budget,
    ) -> Result<Handle, Error> {
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

    fn notifications(mut batch: WaiterBatch<'_>) -> Vec<usize> {
        batch.take_targets().into_iter().map(|(_, id)| id).collect()
    }

    #[test]
    fn waiter_registration_comparison_and_notification_refusal_preserve_fifo() {
        let data = SharedArrayBufferData::new(8, None).unwrap();
        let access = data.prepare_atomic(0, 4, &mut unlimited()).unwrap();
        let owner = Arc::new(WaitOwner::default());
        assert!(
            !data
                .register_waiter(0, &access, 1, &owner, 0, &mut unlimited())
                .unwrap()
        );
        for id in [4, 9] {
            assert!(
                data.register_waiter(0, &access, 0, &owner, id, &mut unlimited())
                    .unwrap()
            );
        }
        assert!(matches!(
            data.take_waiters(0, 2, &mut Budget::new(0)),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        ));
        assert_eq!(
            notifications(data.take_waiters(0, 1, &mut unlimited()).unwrap()),
            [4]
        );
        assert_eq!(
            notifications(data.take_waiters(0, usize::MAX, &mut unlimited()).unwrap()),
            [9]
        );
        assert!(
            notifications(data.take_waiters(0, usize::MAX, &mut unlimited()).unwrap()).is_empty()
        );
    }

    #[test]
    fn mixed_owners_and_retirement_do_not_drop_other_live_waiters() {
        let data = SharedArrayBufferData::new(8, None).unwrap();
        let access = data.prepare_atomic(0, 4, &mut unlimited()).unwrap();
        let first = Arc::new(WaitOwner::default());
        let second = Arc::new(WaitOwner::default());
        assert!(
            data.register_waiter(0, &access, 0, &first, 1, &mut unlimited())
                .unwrap()
        );
        assert!(
            data.register_waiter(0, &access, 0, &second, 2, &mut unlimited())
                .unwrap()
        );
        assert!(matches!(
            data.take_waiters(0, usize::MAX, &mut Budget::new(0)),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        ));
        assert_eq!(
            notifications(data.take_waiters(0, 1, &mut unlimited()).unwrap()),
            [1]
        );
        data.cancel_waiters(&first);
        assert_eq!(
            notifications(data.take_waiters(0, usize::MAX, &mut unlimited()).unwrap()),
            [2]
        );
        assert!(
            data.register_waiter(0, &access, 0, &second, 3, &mut unlimited())
                .unwrap()
        );
        drop(second);
        assert!(
            notifications(data.take_waiters(0, usize::MAX, &mut unlimited()).unwrap()).is_empty()
        );
    }

    fn load(access: &super::super::shared_page::PreparedAtomic) -> u64 {
        access.update(|old| Ok::<_, ()>((None, old))).unwrap()
    }

    #[test]
    fn prepared_pages_survive_directory_growth_moves_and_copies() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let buffer = objects
            .create_shared_array_buffer(&prototype, 8192, Some(16384), &mut unlimited())
            .unwrap();
        let data = objects
            .inspect(&buffer)
            .unwrap()
            .shared_array_buffer_data()
            .unwrap()
            .clone();
        let access = data.prepare_atomic(0, 4, &mut unlimited()).unwrap();
        let source = objects
            .create_array_buffer(&prototype, 8, None, &mut unlimited())
            .unwrap();
        objects
            .write_array_buffer_bytes(&source, 0, &31u32.to_ne_bytes(), &mut unlimited())
            .unwrap();
        objects
            .copy_array_buffer_bytes(&source, 0, &buffer, 0, 4, &mut unlimited())
            .unwrap();
        assert_eq!(load(&access), 31);
        objects
            .write_array_buffer_bytes(&buffer, 4096, &47u32.to_ne_bytes(), &mut unlimited())
            .unwrap();
        objects
            .move_array_buffer_bytes(&buffer, 4096, 0, 4, &mut unlimited())
            .unwrap();
        assert_eq!(load(&access), 47);
        data.grow(16384).unwrap();
        data.prepare_atomic(16380, 4, &mut unlimited()).unwrap();
        assert_eq!(load(&access), 47);
        access
            .update(|old| Ok::<_, ()>((Some(old + 1), ())))
            .unwrap();
        assert_eq!(
            objects
                .read_array_buffer_bytes(&buffer, 0, 4, &mut unlimited())
                .unwrap()[..4],
            48u32.to_ne_bytes()
        );
    }

    #[test]
    fn prepared_atomic_steps_do_not_acquire_the_directory_lock_and_own_their_page() {
        let data = SharedArrayBufferData::new(8, None).unwrap();
        let access = data.prepare_atomic(0, 4, &mut unlimited()).unwrap();
        let guard = data.0.lock().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            access
                .update(|old| Ok::<_, ()>((Some(old + 1), ())))
                .unwrap();
            send.send(load(&access)).unwrap();
            access
        });
        let result = receive.recv_timeout(std::time::Duration::from_secs(5));
        drop(guard);
        let access = thread.join().unwrap();
        assert_eq!(result, Ok(1));
        drop(data);
        assert_eq!(load(&access), 1);
    }

    #[test]
    fn shared_copy_preserves_concurrent_atomic_updates_outside_its_destination() {
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let target = objects
            .create_shared_array_buffer(&prototype, 8, None, &mut unlimited())
            .unwrap();
        let source = objects
            .create_array_buffer(&prototype, 4, None, &mut unlimited())
            .unwrap();
        objects
            .write_array_buffer_bytes(&source, 0, &[3; 4], &mut unlimited())
            .unwrap();
        let data = objects
            .inspect(&target)
            .unwrap()
            .shared_array_buffer_data()
            .unwrap()
            .clone();
        let counter = data.prepare_atomic(4, 4, &mut unlimited()).unwrap();
        let thread = std::thread::spawn(move || {
            for _ in 0..10000 {
                counter
                    .update(|old| Ok::<_, ()>((Some(old + 1), ())))
                    .unwrap();
            }
            load(&counter)
        });
        for _ in 0..100 {
            objects
                .copy_array_buffer_bytes(&source, 0, &target, 0, 4, &mut unlimited())
                .unwrap();
        }
        assert_eq!(thread.join().unwrap(), 10000);
        assert_eq!(
            load(&data.prepare_atomic(4, 4, &mut unlimited()).unwrap()),
            10000
        );
        assert_eq!(
            objects
                .read_array_buffer_bytes(&target, 0, 4, &mut unlimited())
                .unwrap()[..4],
            [3; 4]
        );
    }

    #[test]
    fn concurrent_atomic_first_touch_and_adjacent_updates_preserve_every_increment() {
        let data = SharedArrayBufferData::new(8192, None).unwrap();
        let barrier = Arc::new(std::sync::Barrier::new(8));
        let threads: Vec<_> = (0..8)
            .map(|thread| {
                let data = data.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    let access = data
                        .prepare_atomic((thread % 2) * 4, 4, &mut unlimited())
                        .unwrap();
                    for _ in 0..2000 {
                        access
                            .update(|old| Ok::<_, ()>((Some(old + 1), ())))
                            .unwrap();
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        for index in [0, 4] {
            assert_eq!(
                load(&data.prepare_atomic(index, 4, &mut unlimited()).unwrap()),
                8000
            );
        }
    }

    #[test]
    fn aligned_raw_view_reads_and_writes_do_not_tear_against_atomics() {
        let width = super::super::shared_page::WORD_BYTES;
        let data = SharedArrayBufferData::new(8, None).unwrap();
        data.with_data_mut(|data| data.write_bytes(0, &[0x11; 8][..width], &mut unlimited()))
            .unwrap();
        let a = u64::from_ne_bytes([0x11; 8]);
        let b = u64::from_ne_bytes([0x22; 8]);
        let mask = u64::MAX >> ((8 - width) * 8);
        let threads: Vec<_> = (0..4)
            .map(|thread| {
                let data = data.clone();
                std::thread::spawn(move || {
                    let access = data.prepare_atomic(0, width, &mut unlimited()).unwrap();
                    for _ in 0..2000 {
                        match thread {
                            0 => {
                                access.update(|_| Ok::<_, ()>((Some(a), ()))).unwrap();
                            }
                            1 => {
                                data.with_data_mut(|data| {
                                    data.write_bytes(0, &[0x22; 8][..width], &mut unlimited())
                                })
                                .unwrap();
                            }
                            2 => {
                                let value = load(&access);
                                assert!(value == (a & mask) || value == (b & mask));
                            }
                            _ => {
                                let bytes = data
                                    .with_data(|data| data.read_bytes(0, width, &mut unlimited()))
                                    .unwrap();
                                assert!(
                                    bytes[..width] == [0x11; 8][..width]
                                        || bytes[..width] == [0x22; 8][..width]
                                );
                            }
                        }
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }

    #[test]
    fn refused_atomic_results_and_staged_copies_preserve_owned_page_bytes() {
        let data = SharedArrayBufferData::new(8192, None).unwrap();
        let access = data.prepare_atomic(0, 4, &mut unlimited()).unwrap();
        access.update(|_| Ok::<_, ()>((Some(7), ()))).unwrap();
        assert_eq!(
            access.update::<(), _>(|_| Err("work refusal")),
            Err("work refusal")
        );
        assert!(matches!(
            data.prepare_atomic(4096, 4, &mut Budget::new(100)),
            Err(ArrayBufferError::Storage(Error::WorkLimit))
        ));
        assert_eq!(load(&access), 7);
        let mut objects = Objects::with_limits(None, None);
        let prototype = objects.create(None).unwrap();
        let target = objects.create(Some(&prototype)).unwrap();
        objects.object_mut(&target).unwrap().shared_array_buffer = Some(data);
        let source = objects
            .create_array_buffer(&prototype, 8, None, &mut unlimited())
            .unwrap();
        objects
            .write_array_buffer_bytes(&source, 0, &[2; 8], &mut unlimited())
            .unwrap();
        for limit in [0, 100, 5000] {
            let result =
                objects.copy_array_buffer_bytes(&source, 0, &target, 0, 8, &mut Budget::new(limit));
            assert_eq!(result, Err(ArrayBufferError::Storage(Error::WorkLimit)));
            assert_eq!(load(&access), 7);
        }
    }

    #[test]
    fn prepared_atomic_store_buffering_has_sequentially_consistent_order() {
        for _ in 0..100 {
            let data = SharedArrayBufferData::new(16384, None).unwrap();
            let a = data.prepare_atomic(0, 4, &mut unlimited()).unwrap();
            let b = data.prepare_atomic(8192, 4, &mut unlimited()).unwrap();
            let c = data.prepare_atomic(0, 4, &mut unlimited()).unwrap();
            let d = data.prepare_atomic(8192, 4, &mut unlimited()).unwrap();
            let barrier = Arc::new(std::sync::Barrier::new(2));
            let gate = barrier.clone();
            let left = std::thread::spawn(move || {
                gate.wait();
                a.update(|_| Ok::<_, ()>((Some(1), ()))).unwrap();
                load(&b)
            });
            let right = std::thread::spawn(move || {
                barrier.wait();
                d.update(|_| Ok::<_, ()>((Some(1), ()))).unwrap();
                load(&c)
            });
            assert_ne!((left.join().unwrap(), right.join().unwrap()), (0, 0));
        }
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
