//! Conditional reachability without retaining weak keys or unreachable cycles.

use spite_heap::{Error, Handle, Heap, Trace};

#[derive(Default)]
struct Node {
    strong: Vec<Handle>,
    entries: Vec<(Handle, Option<Handle>)>,
}

impl Trace for Node {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.strong.iter().map(Some)
    }

    fn ephemerons(&self) -> impl Iterator<Item = (&Handle, Option<&Handle>)> {
        self.entries
            .iter()
            .map(|(key, value)| (key, value.as_ref()))
    }
}

fn heap() -> Heap<Node> {
    Heap::with_capacity_limit(None)
}

#[test]
fn both_container_and_key_must_be_reachable_to_retain_the_value() {
    for root_container in [false, true] {
        for root_key in [false, true] {
            let mut heap = heap();
            let key = heap.insert(Node::default()).unwrap();
            let value = heap.insert(Node::default()).unwrap();
            let container = heap
                .insert(Node {
                    entries: vec![(key.clone(), Some(value.clone()))],
                    ..Node::default()
                })
                .unwrap();
            let mut roots = Vec::new();
            if root_container {
                roots.push(&container);
            }
            if root_key {
                roots.push(&key);
            }
            let collection = heap.collect(roots, usize::MAX).unwrap();
            assert_eq!(heap.get(&container).is_ok(), root_container);
            assert_eq!(heap.get(&key).is_ok(), root_key);
            assert_eq!(heap.get(&value).is_ok(), root_container && root_key);
            assert_eq!(
                collection.live,
                usize::from(root_container)
                    + usize::from(root_key)
                    + usize::from(root_container && root_key)
            );
        }
    }
}

#[test]
fn value_back_edges_and_weak_cycles_cannot_resurrect_unreachable_keys() {
    let mut heap = heap();
    let first = heap.insert(Node::default()).unwrap();
    let second = heap
        .insert(Node {
            strong: vec![first.clone()],
            ..Node::default()
        })
        .unwrap();
    let container = heap
        .insert(Node {
            entries: vec![
                (first.clone(), Some(second.clone())),
                (second.clone(), Some(first.clone())),
            ],
            ..Node::default()
        })
        .unwrap();
    let collection = heap.collect([&container], usize::MAX).unwrap();
    assert_eq!(collection.live, 1);
    assert_eq!(collection.reclaimed, 2);
    assert!(heap.get(&first).is_err());
    assert!(heap.get(&second).is_err());
    assert!(heap.get(&container).is_ok());
    // Keeping a Handle in Rust is deliberately not an independent GC root.
}

#[test]
fn an_independently_rooted_container_can_itself_be_a_key() {
    let mut heap = heap();
    let value = heap.insert(Node::default()).unwrap();
    let container = heap.insert(Node::default()).unwrap();
    heap.get_mut(&container)
        .unwrap()
        .entries
        .push((container.clone(), Some(value.clone())));
    assert_eq!(heap.collect([&container], usize::MAX).unwrap().live, 2);
    assert!(heap.get(&value).is_ok());
    assert_eq!(heap.collect([], usize::MAX).unwrap().reclaimed, 2);
}

#[test]
fn reverse_ordered_ephemeron_chains_activate_iteratively_in_either_root_order() {
    for container_first in [false, true] {
        let mut heap = heap();
        let keys: Vec<_> = (0..20_000)
            .map(|_| heap.insert(Node::default()).unwrap())
            .collect();
        let container = heap
            .insert(Node {
                entries: keys
                    .windows(2)
                    .rev()
                    .map(|pair| (pair[0].clone(), Some(pair[1].clone())))
                    .collect(),
                ..Node::default()
            })
            .unwrap();
        let roots = if container_first {
            [&container, &keys[0]]
        } else {
            [&keys[0], &container]
        };
        let collection = heap.collect(roots, usize::MAX).unwrap();
        assert_eq!(collection.live, 20_001);
        assert_eq!(collection.reclaimed, 0);
        assert!(heap.get(keys.last().unwrap()).is_ok());
        assert_eq!(heap.collect([], usize::MAX).unwrap().reclaimed, 20_001);
    }
}

#[test]
fn activated_values_can_reveal_more_containers_keys_and_strong_edges() {
    let mut heap = heap();
    let first_key = heap.insert(Node::default()).unwrap();
    let second_key = heap.insert(Node::default()).unwrap();
    let value = heap.insert(Node::default()).unwrap();
    let second_container = heap
        .insert(Node {
            strong: vec![second_key.clone()],
            entries: vec![(second_key.clone(), Some(value.clone()))],
        })
        .unwrap();
    let first_container = heap
        .insert(Node {
            entries: vec![(first_key.clone(), Some(second_container.clone()))],
            ..Node::default()
        })
        .unwrap();
    let collection = heap
        .collect([&first_container, &first_key], usize::MAX)
        .unwrap();
    assert_eq!(collection.live, 5);
    assert!(heap.get(&value).is_ok());
    assert_eq!(
        heap.collect([&first_container], usize::MAX)
            .unwrap()
            .reclaimed,
        4
    );
}

#[test]
fn stale_weak_keys_do_not_activate_reused_slots_or_inspect_inactive_values() {
    let mut heap = heap();
    let old_key = heap.insert(Node::default()).unwrap();
    let value = heap.insert(Node::default()).unwrap();
    let mut other = Heap::with_capacity_limit(None);
    let foreign_value = other.insert(Node::default()).unwrap();
    let container = heap
        .insert(Node {
            entries: vec![
                (old_key.clone(), Some(value.clone())),
                (old_key.clone(), Some(foreign_value)),
            ],
            ..Node::default()
        })
        .unwrap();
    heap.remove(&old_key).unwrap();
    let replacement = heap.insert(Node::default()).unwrap();
    assert_ne!(old_key, replacement);
    let collection = heap
        .collect([&container, &replacement], usize::MAX)
        .unwrap();
    assert_eq!(collection.live, 2);
    assert_eq!(collection.reclaimed, 1);
    assert!(heap.get(&value).is_err());
    // A later collection can inspect the retained stale weak entries safely.
    assert_eq!(heap.collect([&container], usize::MAX).unwrap().live, 1);
}

#[test]
fn foreign_keys_and_invalid_active_values_fail_before_sweeping() {
    let mut heap = heap();
    let key = heap.insert(Node::default()).unwrap();
    let container = heap.insert(Node::default()).unwrap();
    let garbage = heap.insert(Node::default()).unwrap();
    let stale = heap.insert(Node::default()).unwrap();
    heap.remove(&stale).unwrap();
    let mut other = Heap::with_capacity_limit(None);
    let foreign = other.insert(Node::default()).unwrap();
    for (entry, error) in [
        ((foreign.clone(), None), Error::ForeignHandle),
        ((key.clone(), Some(foreign)), Error::ForeignHandle),
        ((key.clone(), Some(stale)), Error::StaleHandle),
    ] {
        heap.get_mut(&container).unwrap().entries = vec![entry];
        assert_eq!(heap.collect([&container, &key], usize::MAX), Err(error));
        assert_eq!(heap.len(), 3);
        assert!(heap.get(&container).is_ok());
        assert!(heap.get(&key).is_ok());
        assert!(heap.get(&garbage).is_ok());
    }
}

fn budget_graph() -> (Heap<Node>, Handle, Handle, Handle, Handle) {
    let mut heap = heap();
    let key = heap.insert(Node::default()).unwrap();
    let value = heap.insert(Node::default()).unwrap();
    let bridge = heap
        .insert(Node {
            strong: vec![key.clone()],
            ..Node::default()
        })
        .unwrap();
    let container = heap
        .insert(Node {
            strong: vec![bridge],
            entries: vec![(key.clone(), Some(value.clone())); 5],
        })
        .unwrap();
    let garbage = heap.insert(Node::default()).unwrap();
    (heap, container, key, value, garbage)
}

#[test]
fn every_work_failure_during_ephemeron_activation_preserves_values_and_generations() {
    let (mut reference, container, _, _, _) = budget_graph();
    let required = reference
        .collect([&container], usize::MAX)
        .unwrap()
        .work_used;
    let (mut heap, container, key, value, garbage) = budget_graph();
    for work in 0..required {
        assert_eq!(
            heap.collect([&container], work),
            Err(Error::Limit),
            "work {work}"
        );
        assert_eq!(heap.len(), 5);
        for handle in [&container, &key, &value, &garbage] {
            assert!(heap.get(handle).is_ok());
        }
    }
    let collection = heap.collect([&container], required).unwrap();
    assert_eq!(collection.live, 4);
    assert_eq!(collection.reclaimed, 1);
}

#[test]
fn primitive_ephemeron_values_still_consume_inspection_work() {
    let mut heap = heap();
    let key = heap.insert(Node::default()).unwrap();
    let container = heap
        .insert(Node {
            entries: vec![(key.clone(), None); 100],
            ..Node::default()
        })
        .unwrap();
    let garbage = heap.insert(Node::default()).unwrap();
    assert_eq!(heap.collect([&container, &key], 100), Err(Error::Limit));
    assert!(heap.get(&garbage).is_ok());
    let collection = heap.collect([&container, &key], usize::MAX).unwrap();
    assert_eq!(collection.live, 2);
    assert_eq!(collection.reclaimed, 1);
}
