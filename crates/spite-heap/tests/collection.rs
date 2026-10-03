//! Explicit roots, cycles, deep graphs, and failure-before-sweep behavior.

use spite_heap::{Error, Handle, Heap, Trace};
use std::{cell::Cell, rc::Rc};

#[derive(Debug, Default)]
struct Node {
    fields: Vec<Option<Handle>>,
}
impl Trace for Node {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.fields.iter().map(Option::as_ref)
    }
}

#[test]
fn rooted_cycles_survive_and_unrooted_cycles_are_reclaimed() {
    let mut heap = Heap::new(4);
    let a = heap.insert(Node::default()).unwrap();
    let b = heap.insert(Node::default()).unwrap();
    let garbage_a = heap.insert(Node::default()).unwrap();
    let garbage_b = heap.insert(Node::default()).unwrap();
    heap.get_mut(&a).unwrap().fields = vec![Some(b.clone())];
    heap.get_mut(&b).unwrap().fields = vec![Some(a.clone())];
    heap.get_mut(&garbage_a).unwrap().fields = vec![Some(garbage_b.clone())];
    heap.get_mut(&garbage_b).unwrap().fields = vec![Some(garbage_a.clone())];
    let collection = heap.collect([&a], 100).unwrap();
    assert_eq!(collection.live, 2);
    assert_eq!(collection.reclaimed, 2);
    assert!(heap.get(&a).is_ok());
    assert!(heap.get(&b).is_ok());
    assert!(matches!(heap.get(&garbage_a), Err(Error::StaleHandle)));
    assert!(matches!(heap.get(&garbage_b), Err(Error::StaleHandle)));
    let replacement = heap.insert(Node::default()).unwrap();
    assert!(heap.get(&replacement).is_ok());
    assert!(matches!(heap.get(&garbage_b), Err(Error::StaleHandle)));
    let collection = heap.collect([], 100).unwrap();
    assert_eq!(collection.reclaimed, 3);
    assert!(heap.is_empty());
    // The Rust variables a and b were deliberately not roots in the second run.
    assert!(matches!(heap.get(&a), Err(Error::StaleHandle)));
}

#[test]
fn deep_graphs_use_iterative_traversal() {
    let mut heap = Heap::new(20_000);
    let first = heap.insert(Node::default()).unwrap();
    let mut tail = first.clone();
    for _ in 1..20_000 {
        let next = heap
            .insert(Node {
                fields: vec![Some(tail)],
            })
            .unwrap();
        tail = next;
    }
    let collection = heap.collect([&tail], 100_000).unwrap();
    assert_eq!(collection.live, 20_000);
    assert_eq!(collection.reclaimed, 0);
    assert!(heap.get(&first).is_ok());
    assert_eq!(heap.collect([], 100_000).unwrap().reclaimed, 20_000);
}

#[test]
fn duplicate_roots_and_edges_do_not_duplicate_traversal() {
    let mut heap = Heap::new(2);
    let child = heap.insert(Node::default()).unwrap();
    let parent = heap
        .insert(Node {
            fields: vec![Some(child.clone()), Some(child.clone())],
        })
        .unwrap();
    let collection = heap.collect([&parent, &parent, &child], 11).unwrap();
    assert_eq!(collection.live, 2);
    assert_eq!(collection.work_used, 11);
    assert_eq!(
        heap.collect([&parent, &parent, &child], 10),
        Err(Error::Limit)
    );
    assert!(heap.get(&parent).is_ok());
    assert!(heap.get(&child).is_ok());
}

#[test]
fn primitive_fields_and_root_scans_consume_work() {
    let mut heap = Heap::new(2);
    let root = heap
        .insert(Node {
            fields: vec![None; 100],
        })
        .unwrap();
    let garbage = heap.insert(Node::default()).unwrap();
    assert_eq!(heap.collect([&root], 105), Err(Error::Limit));
    assert_eq!(heap.len(), 2);
    assert!(heap.get(&garbage).is_ok());
    let collection = heap.collect([&root], 106).unwrap();
    assert_eq!(collection.work_used, 106);
    assert_eq!(collection.reclaimed, 1);
    assert!(matches!(heap.get(&garbage), Err(Error::StaleHandle)));
    assert_eq!(
        heap.collect(std::iter::repeat(&root), 100),
        Err(Error::Limit)
    );
    assert!(heap.get(&root).is_ok());
}

#[test]
fn every_failed_budget_leaves_values_and_generations_unchanged() {
    let mut heap = Heap::new(3);
    let child = heap.insert(Node::default()).unwrap();
    let root = heap
        .insert(Node {
            fields: vec![Some(child.clone())],
        })
        .unwrap();
    let garbage = heap.insert(Node::default()).unwrap();
    for work in 0..10 {
        assert_eq!(
            heap.collect([&root], work),
            Err(Error::Limit),
            "budget {work}"
        );
        assert_eq!(heap.len(), 3);
        for handle in [&root, &child, &garbage] {
            assert!(heap.get(handle).is_ok());
        }
    }
    let collection = heap.collect([&root], 10).unwrap();
    assert_eq!(collection.reclaimed, 1);
    assert_eq!(collection.live, 2);
    assert!(matches!(heap.get(&garbage), Err(Error::StaleHandle)));
}

#[test]
fn foreign_or_stale_roots_and_reachable_edges_abort_without_sweeping() {
    let mut heap = Heap::new(3);
    let root = heap.insert(Node::default()).unwrap();
    let garbage = heap.insert(Node::default()).unwrap();
    let stale = heap.insert(Node::default()).unwrap();
    heap.remove(&stale).unwrap();
    let mut other = Heap::new(1);
    let foreign = other.insert(Node::default()).unwrap();
    for (bad, expected) in [
        (&stale, Error::StaleHandle),
        (&foreign, Error::ForeignHandle),
    ] {
        assert_eq!(heap.collect([&root, bad], 100), Err(expected));
        heap.get_mut(&root).unwrap().fields = vec![Some(bad.clone())];
        assert_eq!(heap.collect([&root], 100), Err(expected));
        assert_eq!(heap.len(), 2);
        assert!(heap.get(&root).is_ok());
        assert!(heap.get(&garbage).is_ok());
        heap.get_mut(&root).unwrap().fields.clear();
    }
    assert_eq!(heap.collect([&root], 100).unwrap().reclaimed, 1);
}

#[test]
fn sweep_drops_each_unreachable_value_once() {
    struct Counted {
        edges: Vec<Handle>,
        drops: Rc<Cell<usize>>,
    }
    impl Trace for Counted {
        fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
            self.edges.iter().map(Some)
        }
    }
    impl Drop for Counted {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }
    let drops = Rc::new(Cell::new(0));
    let mut heap = Heap::new(2);
    let live = heap
        .insert(Counted {
            edges: Vec::new(),
            drops: Rc::clone(&drops),
        })
        .unwrap();
    heap.insert(Counted {
        edges: vec![live.clone()],
        drops: Rc::clone(&drops),
    })
    .unwrap();
    assert_eq!(heap.collect([&live], 100).unwrap().reclaimed, 1);
    assert_eq!(drops.get(), 1);
    assert_eq!(heap.collect([&live], 100).unwrap().reclaimed, 0);
    drop(heap);
    assert_eq!(drops.get(), 2);
}
