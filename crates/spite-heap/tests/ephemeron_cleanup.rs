//! Release conditional ownership only after a successful reachability fixpoint.

use spite_heap::{Error, Handle, Heap, Trace};
use std::{cell::Cell, rc::Rc};

struct Entry {
    key: Handle,
    value: Option<Handle>,
    payload: Rc<()>,
}

#[derive(Default)]
struct Node {
    strong: Vec<Handle>,
    entries: Vec<Entry>,
    cleanups: Rc<Cell<usize>>,
}

impl Trace for Node {
    fn trace(&self) -> impl Iterator<Item = Option<&Handle>> {
        self.strong.iter().map(Some)
    }

    fn ephemerons(&self) -> impl Iterator<Item = (&Handle, Option<&Handle>)> {
        self.entries
            .iter()
            .map(|entry| (&entry.key, entry.value.as_ref()))
    }

    fn retain_ephemerons(&mut self, retain: impl Fn(&Handle) -> bool) {
        self.cleanups.set(self.cleanups.get() + 1);
        self.entries.retain(|entry| retain(&entry.key));
    }
}

fn entry(key: &Handle, value: Option<&Handle>) -> Entry {
    Entry {
        key: key.clone(),
        value: value.cloned(),
        payload: Rc::new(()),
    }
}

fn heap() -> Heap<Node> {
    Heap::with_capacity_limit(None)
}

#[test]
fn inactive_primitive_values_are_dropped_from_reachable_containers() {
    let mut heap = heap();
    let live_key = heap.insert(Node::default()).unwrap();
    let dead_key = heap.insert(Node::default()).unwrap();
    let live = entry(&live_key, None);
    let dead = entry(&dead_key, None);
    let live_payload = Rc::downgrade(&live.payload);
    let dead_payload = Rc::downgrade(&dead.payload);
    let container = heap
        .insert(Node {
            entries: vec![live, dead, entry(&live_key, None)],
            ..Node::default()
        })
        .unwrap();
    assert_eq!(
        heap.collect([&container, &live_key], usize::MAX)
            .unwrap()
            .live,
        2
    );
    assert!(dead_payload.upgrade().is_none());
    assert!(live_payload.upgrade().is_some());
    assert_eq!(heap.get(&container).unwrap().entries.len(), 2);
    assert_eq!(heap.get(&container).unwrap().cleanups.get(), 1);
    assert_eq!(heap.collect([&container], usize::MAX).unwrap().live, 1);
    assert!(live_payload.upgrade().is_none());
    assert!(heap.get(&container).unwrap().entries.is_empty());
    assert_eq!(heap.get(&container).unwrap().cleanups.get(), 2);
    heap.collect([&container], usize::MAX).unwrap();
    assert_eq!(heap.get(&container).unwrap().cleanups.get(), 2);
}

#[test]
fn a_live_reused_slot_does_not_retain_entries_for_its_old_generation() {
    let mut heap = heap();
    let old = heap.insert(Node::default()).unwrap();
    let stale = entry(&old, None);
    let stale_payload = Rc::downgrade(&stale.payload);
    let container = heap
        .insert(Node {
            entries: vec![stale],
            ..Node::default()
        })
        .unwrap();
    heap.remove(&old).unwrap();
    let replacement = heap.insert(Node::default()).unwrap();
    heap.get_mut(&container)
        .unwrap()
        .entries
        .push(entry(&replacement, None));
    heap.collect([&container, &replacement], usize::MAX)
        .unwrap();
    assert!(stale_payload.upgrade().is_none());
    let entries = &heap.get(&container).unwrap().entries;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, replacement);
}

#[test]
fn cleanup_uses_keys_reached_by_late_ephemeron_and_strong_edge_activation() {
    let mut heap = heap();
    let first_key = heap.insert(Node::default()).unwrap();
    let late_key = heap.insert(Node::default()).unwrap();
    let value = heap.insert(Node::default()).unwrap();
    let late = entry(&late_key, Some(&value));
    let payload = Rc::downgrade(&late.payload);
    let second = heap
        .insert(Node {
            strong: vec![late_key.clone()],
            ..Node::default()
        })
        .unwrap();
    let container = heap
        .insert(Node {
            entries: vec![late, entry(&first_key, Some(&second))],
            ..Node::default()
        })
        .unwrap();
    assert_eq!(
        heap.collect([&first_key, &container], usize::MAX)
            .unwrap()
            .live,
        5
    );
    assert!(payload.upgrade().is_some());
    assert_eq!(heap.get(&container).unwrap().entries.len(), 2);
    assert!(heap.get(&value).is_ok());
}

struct Graph {
    heap: Heap<Node>,
    container: Handle,
    handles: Vec<Handle>,
    payload: std::rc::Weak<()>,
}

fn budget_graph() -> Graph {
    let mut heap = heap();
    let key = heap.insert(Node::default()).unwrap();
    let value = heap.insert(Node::default()).unwrap();
    let garbage_key = heap.insert(Node::default()).unwrap();
    let bridge = heap
        .insert(Node {
            strong: vec![key.clone()],
            ..Node::default()
        })
        .unwrap();
    let inactive = entry(&garbage_key, None);
    let payload = Rc::downgrade(&inactive.payload);
    let container = heap
        .insert(Node {
            strong: vec![bridge.clone()],
            entries: vec![inactive, entry(&key, Some(&value))],
            ..Node::default()
        })
        .unwrap();
    Graph {
        heap,
        container: container.clone(),
        handles: vec![key, value, garbage_key, bridge, container],
        payload,
    }
}

#[test]
fn all_insufficient_work_budgets_leave_weak_entries_and_values_unchanged() {
    let mut reference = budget_graph();
    let work = reference
        .heap
        .collect([&reference.container], usize::MAX)
        .unwrap()
        .work_used;
    let mut graph = budget_graph();
    for budget in 0..work {
        assert_eq!(
            graph.heap.collect([&graph.container], budget),
            Err(Error::Limit)
        );
        assert!(graph.payload.upgrade().is_some());
        let node = graph.heap.get(&graph.container).unwrap();
        assert_eq!(node.entries.len(), 2);
        assert_eq!(node.cleanups.get(), 0);
        for handle in &graph.handles {
            assert!(graph.heap.get(handle).is_ok());
        }
    }
    assert_eq!(
        graph
            .heap
            .collect([&graph.container], work)
            .unwrap()
            .reclaimed,
        1
    );
    assert!(graph.payload.upgrade().is_none());
    assert_eq!(graph.heap.get(&graph.container).unwrap().entries.len(), 1);
}

#[test]
fn handle_errors_do_not_prune_otherwise_inactive_entries() {
    let mut heap = heap();
    let key = heap.insert(Node::default()).unwrap();
    let inactive_key = heap.insert(Node::default()).unwrap();
    let container = heap.insert(Node::default()).unwrap();
    let stale = heap.insert(Node::default()).unwrap();
    heap.remove(&stale).unwrap();
    let mut other = Heap::with_capacity_limit(None);
    let foreign = other.insert(Node::default()).unwrap();
    for (bad_entry, error) in [
        (entry(&foreign, None), Error::ForeignHandle),
        (entry(&key, Some(&foreign)), Error::ForeignHandle),
        (entry(&key, Some(&stale)), Error::StaleHandle),
    ] {
        let inactive = entry(&inactive_key, None);
        let payload = Rc::downgrade(&inactive.payload);
        heap.get_mut(&container).unwrap().entries = vec![inactive, bad_entry];
        assert_eq!(heap.collect([&key, &container], usize::MAX), Err(error));
        assert!(payload.upgrade().is_some());
        assert_eq!(heap.get(&container).unwrap().entries.len(), 2);
        assert_eq!(heap.get(&container).unwrap().cleanups.get(), 0);
        assert!(heap.get(&inactive_key).is_ok());
    }
}

#[test]
fn unreachable_containers_drop_their_values_without_running_cleanup() {
    let mut heap = heap();
    let key = heap.insert(Node::default()).unwrap();
    let value = entry(&key, None);
    let payload = Rc::downgrade(&value.payload);
    let cleanups = Rc::new(Cell::new(0));
    let container = heap
        .insert(Node {
            entries: vec![value],
            cleanups: Rc::clone(&cleanups),
            ..Node::default()
        })
        .unwrap();
    assert_eq!(heap.collect([&key], usize::MAX).unwrap().reclaimed, 1);
    assert!(payload.upgrade().is_none());
    assert_eq!(cleanups.get(), 0);
    assert!(heap.get(&container).is_err());
}
