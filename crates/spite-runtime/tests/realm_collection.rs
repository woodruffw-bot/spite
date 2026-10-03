//! Collection between Scripts retains realm state and explicit embedding roots.

use spite_heap::Error as HeapError;
use spite_runtime::{Error, Limits, Realm, Value, object::Error as ObjectError};

#[test]
fn persistent_lexical_var_and_sloppy_global_bindings_are_roots() {
    let mut realm = Realm::default();
    realm
        .eval("let lexical = {}; var variable = {}; sloppy = {}; { let local = {}; } ({})")
        .unwrap();
    let result = realm.collect(1000).unwrap();
    assert_eq!(result.live, 10); // four intrinsic objects plus the three persistent values
    assert_eq!(result.reclaimed, 2);
    for name in ["lexical", "variable", "sloppy"] {
        let Value::Object(handle) = realm.eval(name).unwrap() else {
            panic!("object")
        };
        assert!(realm.inspect_object(&handle).is_ok());
    }
    realm
        .eval("lexical = null; variable = null; delete sloppy;")
        .unwrap();
    assert_eq!(realm.collect(1000).unwrap().reclaimed, 3);
    assert_eq!(realm.collect(1000).unwrap().live, 7);
}

#[test]
fn rooted_returned_and_thrown_values_survive_until_their_last_clone_drops() {
    let mut realm = Realm::default();
    let value = realm.eval("({nested: {}})").unwrap();
    let root = realm.root_value(value.clone(), 100).unwrap();
    let clone = root.clone();
    let Err(Error::Thrown(thrown)) = realm.eval("throw {a: 1}") else {
        panic!("throw")
    };
    let thrown = realm.root_value(thrown, 100).unwrap();
    assert_eq!(realm.collect(1000).unwrap().live, 10);
    drop(root);
    assert_eq!(clone.value(), &value);
    assert_eq!(realm.collect(1000).unwrap().live, 10);
    drop(clone);
    assert_eq!(realm.collect(1000).unwrap().reclaimed, 2);
    drop(thrown);
    assert_eq!(realm.collect(1000).unwrap().reclaimed, 1);
    let Value::Object(handle) = value else {
        panic!("object")
    };
    assert!(matches!(
        realm.inspect_object(&handle),
        Err(ObjectError::Heap(HeapError::StaleHandle))
    ));
}

#[test]
fn foreign_values_are_rejected_and_failed_collection_preserves_state() {
    let mut realm = Realm::default();
    let foreign = Realm::default().eval("({})").unwrap();
    assert!(matches!(
        realm.root_value(foreign, 100),
        Err(ObjectError::Heap(HeapError::ForeignHandle))
    ));
    let value = realm.eval("let a = {nested: {}}; a").unwrap();
    let Value::Object(handle) = value else {
        panic!("object")
    };
    for work in 0..10 {
        assert_eq!(realm.collect(work), Err(HeapError::Limit));
        assert!(realm.inspect_object(&handle).is_ok());
    }
    assert_eq!(realm.collect(1000).unwrap().live, 9);
    let primitive = realm.root_value(Value::Number(7.0), 0).unwrap();
    assert_eq!(primitive.value(), &Value::Number(7.0));
}

#[test]
fn explicit_collection_reuses_slots_and_allocation_never_collects_implicitly() {
    let mut realm = Realm::new(Limits {
        max_objects: 8,
        ..Limits::default()
    });
    for _ in 0..100 {
        let value = realm.eval("({})").unwrap();
        assert!(matches!(realm.eval("({})"), Err(Error::Limit { .. })));
        let Value::Object(handle) = value else {
            panic!("object")
        };
        assert!(realm.inspect_object(&handle).is_ok());
        assert_eq!(realm.collect(300).unwrap().reclaimed, 1);
    }
}

#[test]
fn abrupt_evaluation_restores_scopes_and_pending_completions_preserve_identity() {
    let mut realm = Realm::default();
    realm.eval("let saved = {}; let same = saved;").unwrap();
    assert_eq!(
        realm.eval("try { throw saved; } catch (e) { e === same; } finally { ({discarded: {}}); }"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("{ let temporary = {}; throw {thrown: temporary}; }"),
        Err(Error::Thrown(_))
    ));
    assert_eq!(realm.collect(1000).unwrap().live, 8);
    assert_eq!(realm.eval("saved === same"), Ok(Value::Boolean(true)));
}
