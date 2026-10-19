//! Prepared-sweep planning, marks and generation regressions.

use super::*;

#[test]
fn dropped_preparation_preserves_values_and_generations_after_invalid_marks() {
    let mut heap = Heap::with_capacity_limit(None);
    let first = heap.insert(11).unwrap();
    let removed = heap.insert(22).unwrap();
    assert_eq!(heap.remove(&removed), Ok(22));
    let mut other = Heap::with_capacity_limit(None);
    let foreign = other.insert(33).unwrap();
    {
        let mut session = heap.prepare_sweep().unwrap();
        assert_eq!(session.index(&removed), Err(Error::StaleHandle));
        assert_eq!(session.index(&foreign), Err(Error::ForeignHandle));
        assert_eq!(session.mark(usize::MAX), Err(Error::CollectionState));
        assert_eq!(session.mark(removed.slot), Err(Error::CollectionState));
        let index = session.index(&first).unwrap();
        assert_eq!(session.mark(index), Ok(true));
        assert_eq!(session.mark(index), Ok(false));
        assert_eq!(session.entry(index), Some(&11));
    }
    assert_eq!(heap.get(&first), Ok(&11));
    assert_eq!(heap.get(&removed), Err(Error::StaleHandle));
    let reused = heap.insert(44).unwrap();
    assert_ne!(reused, removed);
    assert_eq!(heap.get(&first), Ok(&11));
}

#[test]
fn a_single_prepared_mask_accepts_late_marks_before_infallible_sweep() {
    let mut heap = Heap::with_capacity_limit(None);
    let first = heap.insert(11).unwrap();
    let late = heap.insert(22).unwrap();
    let dead = heap.insert(33).unwrap();
    let mut session = heap.prepare_sweep().unwrap();
    let first_index = session.index(&first).unwrap();
    let late_index = session.index(&late).unwrap();
    let dead_index = session.index(&dead).unwrap();
    session.mark(first_index).unwrap();
    assert!(!session.is_marked(late_index));
    // Root discovery can extend the same mask after an earlier complete mark.
    session.mark(late_index).unwrap();
    *session.entry_mut(late_index).unwrap() = 24;
    assert!(!session.is_marked(dead_index));
    assert_eq!(
        session.finish(17),
        Collection {
            reclaimed: 1,
            live: 2,
            work_used: 17
        }
    );
    assert_eq!(heap.get(&first), Ok(&11));
    assert_eq!(heap.get(&late), Ok(&24));
    assert_eq!(heap.get(&dead), Err(Error::StaleHandle));
    let reused = heap.insert(44).unwrap();
    assert_ne!(reused, dead);
    assert_eq!(heap.allocated_slots(), 3);
}

#[test]
fn prepared_sweep_handles_retired_generations_and_empty_heaps_without_trace() {
    // T need not implement Trace: final sweep never calls an extensible walker.
    struct Untraced;
    let mut heap = Heap::with_capacity_limit(None);
    let retired = heap.insert(Untraced).unwrap();
    heap.slots[retired.slot].generation = u64::MAX;
    let current = Handle {
        generation: u64::MAX,
        ..retired
    };
    let session = heap.prepare_sweep().unwrap();
    assert!(session.index(&current).is_ok());
    assert_eq!(
        session.finish(2),
        Collection {
            reclaimed: 1,
            live: 0,
            work_used: 2
        }
    );
    assert_eq!(heap.index(&current), Err(Error::StaleHandle));
    let replacement = heap.insert(Untraced).unwrap();
    assert_ne!(replacement.slot, current.slot);
    heap.remove(&replacement).unwrap();
    let empty = heap.prepare_sweep().unwrap();
    assert_eq!(
        empty.finish(0),
        Collection {
            reclaimed: 0,
            live: 0,
            work_used: 0
        }
    );
}
