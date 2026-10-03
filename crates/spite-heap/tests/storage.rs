//! Heap identity, generation reuse, borrowing, and capacity invariants.

use spite_heap::{Error, Heap};
use std::{cell::Cell, rc::Rc};

#[test]
fn insertion_access_mutation_and_moving_the_heap_preserve_identity() {
    let mut heap = Heap::new(2);
    assert!(heap.is_empty());
    let first = heap.insert(String::from("one")).unwrap();
    let second = heap.insert(String::from("two")).unwrap();
    let alias = first.clone();
    assert_eq!(alias, first);
    assert_ne!(first, second);
    heap.get_mut(&alias).unwrap().push('!');
    let moved = heap;
    assert_eq!(moved.get(&first).unwrap(), "one!");
    assert_eq!(moved.get(&second).unwrap(), "two");
    assert_eq!(moved.len(), 2);
    assert_eq!(moved.allocated_slots(), 2);
}

#[test]
fn foreign_handles_are_rejected_even_with_identical_slot_numbers() {
    let mut first = Heap::new(1);
    let mut second = Heap::new(1);
    let a = first.insert(1).unwrap();
    let b = second.insert(2).unwrap();
    assert_ne!(a, b);
    assert_eq!(first.get(&b), Err(Error::ForeignHandle));
    assert_eq!(first.get_mut(&b), Err(Error::ForeignHandle));
    assert_eq!(first.remove(&b), Err(Error::ForeignHandle));
    assert_eq!(second.get(&a), Err(Error::ForeignHandle));
    assert_eq!(first.get(&a), Ok(&1));
    assert_eq!(second.get(&b), Ok(&2));
    let mut replacement = Heap::new(1);
    drop(first);
    replacement.insert(3).unwrap();
    assert_eq!(replacement.get(&a), Err(Error::ForeignHandle));
}

#[test]
fn slot_reuse_invalidates_all_old_handles() {
    let mut heap = Heap::new(1);
    let original = heap.insert(0).unwrap();
    let alias = original.clone();
    assert_eq!(heap.remove(&original), Ok(0));
    assert!(heap.is_empty());
    assert_eq!(heap.remove(&alias), Err(Error::StaleHandle));
    for value in 1..=1000 {
        let next = heap.insert(value).unwrap();
        assert_ne!(next, original);
        assert_eq!(heap.get(&original), Err(Error::StaleHandle));
        assert_eq!(heap.get_mut(&alias), Err(Error::StaleHandle));
        assert_eq!(heap.get(&next), Ok(&value));
        assert_eq!(heap.allocated_slots(), 1);
        assert_eq!(heap.remove(&next), Ok(value));
    }
}

#[test]
fn capacity_failures_preserve_live_values_and_free_slots_are_reused() {
    let mut empty = Heap::new(0);
    assert_eq!(empty.insert(1), Err(Error::Capacity));
    assert!(empty.is_empty());
    assert_eq!(empty.allocated_slots(), 0);
    let mut heap = Heap::new(3);
    let handles: Vec<_> = (0..3).map(|n| heap.insert(n).unwrap()).collect();
    assert_eq!(heap.insert(9), Err(Error::Capacity));
    assert_eq!(heap.len(), 3);
    assert_eq!(heap.remove(&handles[1]), Ok(1));
    let replacement = heap.insert(8).unwrap();
    assert_eq!(heap.allocated_slots(), 3);
    assert_eq!(heap.get(&handles[0]), Ok(&0));
    assert_eq!(heap.get(&handles[2]), Ok(&2));
    assert_eq!(heap.get(&replacement), Ok(&8));
}

#[test]
fn handles_own_only_identity_tokens_not_values() {
    struct DropCount(Rc<Cell<usize>>);
    impl Drop for DropCount {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    let count = Rc::new(Cell::new(0));
    let mut heap = Heap::new(2);
    let first = heap.insert(DropCount(Rc::clone(&count))).unwrap();
    let second = heap.insert(DropCount(Rc::clone(&count))).unwrap();
    let alias = first.clone();
    drop(first);
    assert_eq!(count.get(), 0);
    drop(heap.remove(&alias).unwrap());
    assert_eq!(count.get(), 1);
    drop(heap);
    assert_eq!(count.get(), 2);
    drop((alias, second));
    assert_eq!(count.get(), 2);
}
