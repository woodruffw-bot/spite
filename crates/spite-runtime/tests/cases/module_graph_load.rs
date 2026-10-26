//! Host-provided graph loading and stable Module request identities (16.2.1).

use spite_core::{JsString, Span};
use spite_runtime::{Error, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}
fn collect(realm: &mut Realm) {
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
}

#[test]
fn loading_visits_depth_first_source_order_deduplicates_requests_and_caches_shared_records() {
    let mut realm = Realm::default();
    let root = realm
        .create_module("import 'a';export*from'b';import 'a'")
        .unwrap();
    let a = realm
        .create_module("import 'leaf';export const a=1")
        .unwrap();
    let b = realm
        .create_module("import 'leaf';export const b=1")
        .unwrap();
    let leaf = realm.create_module("export const value=1").unwrap();
    let mut events = Vec::new();
    realm
        .load_module_graph(&root, |from, request| {
            let name = String::from_utf16(request.specifier.code_units()).unwrap();
            let parent = if from == &root {
                "root"
            } else if from == &a {
                "a"
            } else {
                "b"
            };
            events.push(format!("{parent}:{name}"));
            Ok(match name.as_str() {
                "a" => a.clone(),
                "b" => b.clone(),
                "leaf" => leaf.clone(),
                _ => panic!("request"),
            })
        })
        .unwrap();
    assert_eq!(events, ["root:a", "a:leaf", "root:b", "b:leaf"]);
    realm
        .load_module_graph(&root, |_, _| panic!("cached graph called resolver"))
        .unwrap();
    realm
        .load_module_graph(&a, |_, _| panic!("cached shared subtree called resolver"))
        .unwrap();
}

#[test]
fn aliases_and_cycles_preserve_record_identity_without_entering_module_bodies() {
    let mut realm = Realm::default();
    realm.eval("let effects=0").unwrap();
    let root = realm
        .create_module("import 'a';import 'alias';effects++")
        .unwrap();
    let a = realm.create_module("import 'root';effects++").unwrap();
    let mut calls = 0;
    realm
        .load_module_graph(&root, |_, request| {
            calls += 1;
            Ok(if request.specifier == JsString::from("root") {
                root.clone()
            } else {
                a.clone()
            })
        })
        .unwrap();
    assert_eq!(calls, 3);
    check(&mut realm, "effects===0");
    realm.evaluate_module_record(&root).unwrap();
    check(&mut realm, "effects===2");
}

#[test]
fn canonical_attributes_deduplicate_equal_requests_and_keep_distinct_requests_separate() {
    let mut realm = Realm::default();
    let root=realm.create_module("import 'same' with {b:'2',a:'1'};import 'same' with {a:'1',b:'2'};import 'same' with {a:'1',b:'3'}").unwrap();
    let one = realm.create_module("export const value=1").unwrap();
    let two = realm.create_module("export const value=2").unwrap();
    let requests = realm.module_requests(&root).unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].attributes[0].key, JsString::from("a"));
    assert_eq!(requests[0].attributes[1].value, JsString::from("2"));
    assert_eq!(requests[1].attributes[1].value, JsString::from("3"));
    let mut calls = 0;
    realm
        .load_module_graph(&root, |_, request| {
            calls += 1;
            Ok(if request.attributes[1].value == JsString::from("2") {
                one.clone()
            } else {
                two.clone()
            })
        })
        .unwrap();
    assert_eq!(calls, 2);
    let lone = realm
        .create_module(r"import '\uD800' with {'\uDC00':'\uDFFF'}")
        .unwrap();
    realm
        .load_module_graph(&lone, |_, request| {
            assert_eq!(request.specifier.code_units(), &[0xd800]);
            assert_eq!(request.attributes[0].key.code_units(), &[0xdc00]);
            assert_eq!(request.attributes[0].value.code_units(), &[0xdfff]);
            Ok(one.clone())
        })
        .unwrap();
}

#[test]
fn resolver_errors_publish_no_partial_links_and_can_be_retried() {
    let mut realm = Realm::default();
    let root = realm.create_module("import 'a';import 'b'").unwrap();
    let a = realm.create_module("import 'leaf'").unwrap();
    let b = realm.create_module("export{}").unwrap();
    let leaf = realm.create_module("export{}").unwrap();
    let error = Error::Unsupported {
        span: Span::new(0, 0),
        message: "missing host record".into(),
    };
    assert_eq!(
        realm.load_module_graph(&root, |_, request| {
            if request.specifier == JsString::from("b") {
                Err(error.clone())
            } else if request.specifier == JsString::from("leaf") {
                Ok(leaf.clone())
            } else {
                Ok(a.clone())
            }
        }),
        Err(error)
    );
    let mut calls = 0;
    realm
        .load_module_graph(&root, |_, request| {
            calls += 1;
            Ok(if request.specifier == JsString::from("a") {
                a.clone()
            } else if request.specifier == JsString::from("leaf") {
                leaf.clone()
            } else {
                b.clone()
            })
        })
        .unwrap();
    assert_eq!(calls, 3);
}

#[test]
fn foreign_records_fail_ownership_validation_before_any_graph_publication() {
    let mut realm = Realm::default();
    let root = realm.create_module("import 'a'").unwrap();
    let foreign = Realm::default().create_module("export{}").unwrap();
    assert!(matches!(
        realm.module_requests(&foreign),
        Err(Error::InvalidObject(_))
    ));
    assert!(matches!(
        realm.load_module_graph(&foreign, |_, _| panic!("foreign root called resolver")),
        Err(Error::InvalidObject(_))
    ));
    assert!(matches!(
        realm.load_module_graph(&root, |_, _| Ok(foreign.clone())),
        Err(Error::InvalidObject(_))
    ));
    let local = realm.create_module("export{}").unwrap();
    let mut calls = 0;
    realm
        .load_module_graph(&root, |_, _| {
            calls += 1;
            Ok(local.clone())
        })
        .unwrap();
    assert_eq!(calls, 1);
}

#[test]
fn dependency_edges_keep_record_environments_alive_until_the_root_is_dropped() {
    let mut realm = Realm::default();
    realm.eval("let weak").unwrap();
    let root = realm.create_module("import 'child'").unwrap();
    let child = realm
        .create_module("const held=Symbol();weak=new WeakRef(held)")
        .unwrap();
    realm.evaluate_module_record(&child).unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(child.clone()))
        .unwrap();
    drop(child);
    collect(&mut realm);
    check(&mut realm, "weak.deref()!==undefined");
    drop(root);
    collect(&mut realm);
    check(&mut realm, "weak.deref()===undefined");
}
