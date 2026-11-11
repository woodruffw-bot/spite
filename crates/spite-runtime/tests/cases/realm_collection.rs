//! Collection between Scripts retains realm state and explicit embedding roots.

use crate::common;
use common::REALM_ENTRIES;

use spite_heap::Error as HeapError;
use spite_runtime::object::Error as ObjectError;
use spite_runtime::{Error, Limits, PromiseJobOutcome, Realm, Value};

#[test]
fn suspended_class_metadata_consumes_collection_work() {
    fn suspend(fields: usize, static_fields: bool, declaration: bool, census: bool) -> Realm {
        let prefix = if static_fields { "static " } else { "" };
        let fields: String = (0..fields)
            .map(|index| format!("{prefix}f{index};"))
            .collect();
        let definition = format!("class Ready{{{fields}[await gate];}}");
        let body = if declaration {
            format!("{definition}C=Ready;")
        } else {
            format!("C={definition};")
        };
        let weak_map = if census {
            "let marker=Symbol(),census=new WeakMap([[marker,0]]);"
        } else {
            ""
        };
        let mut realm = Realm::default();
        realm.eval(&format!(
            "let release,C,done=false;let gate=new Promise(resolve=>{{release=resolve}});{weak_map}let pending=(async()=>{{{body}done=true}})();"
        )).unwrap();
        realm.collect(usize::MAX).unwrap();
        realm
    }

    const FIELDS: usize = 512;
    for static_fields in [false, true] {
        for declaration in [false, true] {
            for census in [false, true] {
                let mut baseline = suspend(0, static_fields, declaration, census);
                let baseline_work = baseline.collect(usize::MAX).unwrap().work_used;
                let mut realm = suspend(FIELDS, static_fields, declaration, census);
                assert_eq!(
                    realm.collect(baseline_work + FIELDS / 2),
                    Err(HeapError::Limit),
                    "static={static_fields}, declaration={declaration}, census={census}"
                );
                realm.eval("release('late')").unwrap();
                loop {
                    match realm.run_next_promise_job().unwrap() {
                        PromiseJobOutcome::Completed => {}
                        PromiseJobOutcome::NoJob => break,
                        PromiseJobOutcome::Reported(error) => panic!("{error}"),
                    }
                }
                let target = if static_fields { "C" } else { "instance" };
                assert_eq!(realm.eval(&format!(
                    r#"
                        let instance = new C();
                        done && Object.hasOwn(instance, 'late') && instance.late === undefined && Object.keys(
                                {target}).length === {} && Object.hasOwn({target}, 'f0') && Object
                            .hasOwn({target}, 'f511') && {target}.f0 === undefined && {target}.f511 ===
                            undefined
                    "#,
                    FIELDS + usize::from(!static_fields)
                )), Ok(Value::Boolean(true)));
            }
        }
    }
}

#[test]
fn suspended_binding_string_keys_consume_symbol_scan_work() {
    fn suspend(fields: usize) -> Realm {
        let keys: String = (0..fields).map(|index| format!("f{index}:sink,")).collect();
        let mut realm = Realm::default();
        realm.eval(&format!(
            "let release,sink,rest,done=false,source={{}};let gate=new Promise(resolve=>{{release=resolve}});let marker=Symbol(),census=new WeakMap([[marker,0]]);let pending=(async()=>{{({{{keys}[await gate]:sink,...rest}}=source);done=true}})();"
        )).unwrap();
        realm.collect(usize::MAX).unwrap();
        realm
    }

    let mut baseline = suspend(0);
    let baseline_work = baseline.collect(usize::MAX).unwrap().work_used;
    const KEYS: usize = 1024;
    let mut realm = suspend(KEYS);
    assert_eq!(
        realm.collect(baseline_work + KEYS / 2),
        Err(HeapError::Limit)
    );
    realm
        .eval("source.f0='excluded';source.keep='kept';source.later='current';release('later')")
        .unwrap();
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
    assert_eq!(
        realm.eval(
            "done&&sink==='current'&&Object.keys(rest).join(',')==='keep'&&rest.keep==='kept'"
        ),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn collection_charges_for_uninitialized_class_field_metadata() {
    let mut baseline = Realm::default();
    baseline.eval("class C {}").unwrap();
    baseline.collect(usize::MAX).unwrap();
    let baseline_work = baseline.collect(usize::MAX).unwrap().work_used;

    const FIELDS: usize = 1024;
    let fields: String = (0..FIELDS).map(|index| format!("f{index};")).collect();
    let mut realm = Realm::default();
    realm.eval(&format!("class C {{{fields}}}")).unwrap();
    let live = realm.collect(usize::MAX).unwrap().live;
    // Fields without initializers still require inspecting the metadata entries.
    assert_eq!(
        realm.collect(baseline_work + FIELDS / 2),
        Err(HeapError::Limit)
    );
    assert_eq!(realm.collect(usize::MAX).unwrap().live, live);
    assert_eq!(
        realm.eval("let instance=new C;Object.keys(instance).length===1024&&instance.f0===undefined&&instance.f1023===undefined"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn persistent_lexical_var_and_sloppy_global_bindings_are_roots() {
    let mut realm = Realm::default();
    realm
        .eval("let lexical = {}; var variable = {}; sloppy = {}; { let local = {}; } ({})")
        .unwrap();
    let result = realm.collect(usize::MAX).unwrap();
    assert_eq!(result.live, REALM_ENTRIES + 3); // global environment, global object, nine intrinsics, three persistent objects
    assert_eq!(result.reclaimed, 3);
    for name in ["lexical", "variable", "sloppy"] {
        let Value::Object(handle) = realm.eval(name).unwrap() else {
            panic!("object")
        };
        assert!(realm.inspect_object(&handle).is_ok());
    }
    realm
        .eval("lexical = null; variable = null; delete sloppy;")
        .unwrap();
    assert_eq!(realm.collect(usize::MAX).unwrap().reclaimed, 3);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES);
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
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 3);
    drop(root);
    assert_eq!(clone.value(), &value);
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 3);
    drop(clone);
    assert_eq!(realm.collect(usize::MAX).unwrap().reclaimed, 2);
    drop(thrown);
    assert_eq!(realm.collect(usize::MAX).unwrap().reclaimed, 1);
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
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 2);
    let primitive = realm.root_value(Value::Number(7.0), 0).unwrap();
    assert_eq!(primitive.value(), &Value::Number(7.0));
}

#[test]
fn explicit_collection_reuses_slots_and_allocation_never_collects_implicitly() {
    let mut realm = Realm::new(Limits {
        max_heap_entries: Some(REALM_ENTRIES + 1),
        ..Limits::default()
    });
    for _ in 0..100 {
        let value = realm.eval("({})").unwrap();
        assert!(matches!(realm.eval("({})"), Err(Error::Limit { .. })));
        let Value::Object(handle) = value else {
            panic!("object")
        };
        assert!(realm.inspect_object(&handle).is_ok());
        assert_eq!(realm.collect(usize::MAX).unwrap().reclaimed, 1);
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
    assert_eq!(realm.collect(usize::MAX).unwrap().live, REALM_ENTRIES + 1);
    assert_eq!(realm.eval("saved === same"), Ok(Value::Boolean(true)));
}
