//! Host-root lifetimes, budget failures, and bounded root-registry reuse.

use spite_heap::Error as HeapError;
use spite_runtime::object::{Budget, Error, Objects};

#[test]
fn last_root_clone_controls_reachability_including_prototypes() {
    let mut objects = Objects::new(3, 0);
    let prototype = objects.create(None).unwrap();
    let handle = objects.create(Some(&prototype)).unwrap();
    let garbage = objects.create(None).unwrap();
    let root = objects.root(&handle, &mut Budget::new(10)).unwrap();
    let clone = root.clone();
    assert_eq!(root.handle(), &handle);
    assert_eq!(objects.collect([], 100).unwrap().reclaimed, 1);
    assert!(matches!(
        objects.inspect(&garbage),
        Err(Error::Heap(HeapError::StaleHandle))
    ));
    drop(root);
    assert_eq!(objects.collect([], 100).unwrap().live, 2);
    drop(clone);
    assert_eq!(objects.collect([], 100).unwrap().reclaimed, 2);
    assert!(matches!(
        objects.inspect(&handle),
        Err(Error::Heap(HeapError::StaleHandle))
    ));
}

#[test]
fn repeated_roots_share_lifetime_and_registry_work() {
    let mut objects = Objects::new(1, 0);
    let handle = objects.create(None).unwrap();
    let first = objects.root(&handle, &mut Budget::new(1)).unwrap();
    let mut roots = Vec::new();
    for _ in 0..1000 {
        roots.push(objects.root(&handle, &mut Budget::new(2)).unwrap());
    }
    let work = objects.collect([], 100).unwrap().work_used;
    drop(first);
    assert_eq!(objects.collect([], 100).unwrap().work_used, work);
    let last = roots.pop().unwrap();
    drop(roots);
    assert_eq!(objects.collect([], 100).unwrap().live, 1);
    drop(last);
    assert_eq!(objects.collect([], 100).unwrap().reclaimed, 1);
}

#[test]
fn expired_entries_are_reused_across_many_slot_generations() {
    let mut objects = Objects::new(1, 0);
    for _ in 0..1000 {
        let handle = objects.create(None).unwrap();
        let root = objects.root(&handle, &mut Budget::new(2)).unwrap();
        assert_eq!(objects.collect([], 10).unwrap().live, 1);
        drop(root);
        assert_eq!(objects.collect([], 10).unwrap().reclaimed, 1);
    }
}

#[test]
fn explicit_roots_and_host_tokens_are_combined() {
    let mut objects = Objects::new(3, 0);
    let a = objects.create(None).unwrap();
    let b = objects.create(None).unwrap();
    let c = objects.create(None).unwrap();
    let root = objects.root(&a, &mut Budget::new(10)).unwrap();
    assert_eq!(objects.collect([&b], 100).unwrap().live, 2);
    assert!(objects.inspect(&a).is_ok());
    assert!(objects.inspect(&b).is_ok());
    assert!(matches!(
        objects.inspect(&c),
        Err(Error::Heap(HeapError::StaleHandle))
    ));
    drop(root);
    assert_eq!(objects.collect([&b], 100).unwrap().reclaimed, 1);
}

#[test]
fn root_and_collection_budget_failures_do_not_release_objects() {
    let mut objects = Objects::new(2, 0);
    let a = objects.create(None).unwrap();
    let b = objects.create(None).unwrap();
    let root = objects.root(&a, &mut Budget::new(10)).unwrap();
    assert_eq!(objects.root(&b, &mut Budget::new(1)), Err(Error::WorkLimit));
    for work in 0..9 {
        match objects.collect([&b], work) {
            Err(HeapError::Limit) => {
                assert!(objects.inspect(&a).is_ok());
                assert!(objects.inspect(&b).is_ok());
            }
            Ok(result) => assert_eq!(result.live, 2),
            result => panic!("unexpected {result:?}"),
        }
    }
    assert_eq!(objects.collect([], 100).unwrap().reclaimed, 1);
    assert!(objects.inspect(root.handle()).is_ok());
}

#[test]
fn invalid_handles_cannot_be_rooted_and_tokens_can_outlive_the_heap() {
    let mut objects = Objects::new(1, 0);
    let foreign = Objects::new(1, 0).create(None).unwrap();
    assert_eq!(
        objects.root(&foreign, &mut Budget::new(10)),
        Err(Error::Heap(HeapError::ForeignHandle))
    );
    let stale = objects.create(None).unwrap();
    objects.collect([], 10).unwrap();
    assert_eq!(
        objects.root(&stale, &mut Budget::new(10)),
        Err(Error::Heap(HeapError::StaleHandle))
    );
    let handle = objects.create(None).unwrap();
    let root = objects.root(&handle, &mut Budget::new(10)).unwrap();
    let mut moved = objects;
    assert_eq!(moved.collect([], 10).unwrap().live, 1);
    drop(moved);
    assert_eq!(root.handle(), &handle);
}
