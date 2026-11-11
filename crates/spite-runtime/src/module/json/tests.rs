use super::{Error, JsString, ModuleHandle, Realm, Span, Value};
use crate::{ExceptionKind, Limits, PromiseJobOutcome};

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => return,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error:?}"),
        }
    }
}

fn default_value(realm: &mut Realm, module: &ModuleHandle) -> Value {
    let Value::Object(namespace) = realm.module_namespace(module).unwrap() else {
        panic!("module namespace")
    };
    realm
        .get_property(&namespace, &JsString::from("default"), Span::new(0, 0))
        .unwrap()
}

#[test]
fn scalar_defaults_preserve_json_number_and_utf16_semantics() {
    for (source, expected) in [
        ("null", Value::Null),
        ("true", Value::Boolean(true)),
        ("false", Value::Boolean(false)),
        ("-0", Value::Number(-0.0)),
        ("1e400", Value::Number(f64::INFINITY)),
        (
            r#""x\uD800""#,
            Value::String(JsString::from_code_units(vec![120, 0xd800])),
        ),
    ] {
        let mut realm = Realm::default();
        let module = realm.create_json_module(source).unwrap();
        assert!(realm.module_requests(&module).unwrap().is_empty());
        realm
            .load_module_graph(&module, |_, _| panic!("JSON has no dependencies"))
            .unwrap();
        realm.evaluate_module_record(&module).unwrap();
        drain(&mut realm);
        match (default_value(&mut realm, &module), expected) {
            (Value::Number(actual), Value::Number(expected)) => {
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
            (actual, expected) => assert_eq!(actual, expected),
        }
    }
}

#[test]
fn lossless_utf16_json_preserves_raw_surrogates_in_values_and_property_names() {
    let mut realm = Realm::default();
    realm.eval("JSON=undefined").unwrap();
    let text = JsString::from_code_units(vec![0x22, 0xd800, 0xfffd, 0xdc00, 0x22]);
    let scalar = realm.create_json_module_utf16(&text).unwrap();
    realm.evaluate_module_record(&scalar).unwrap();
    assert_eq!(
        default_value(&mut realm, &scalar),
        Value::String(JsString::from_code_units(vec![0xd800, 0xfffd, 0xdc00]))
    );
    let object = JsString::from_code_units(vec![
        0x7b, 0x22, 0xd800, 0x22, 0x3a, 0x22, 0xdc00, 0x22, 0x7d,
    ]);
    let data = realm.create_json_module_utf16(&object).unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    let root = realm
        .create_module("import value from'data';globalThis.result=value['\\ud800']==='\\udc00'&&Object.keys(value)[0]==='\\ud800'")
        .unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(data.clone()))
        .unwrap();
    realm.evaluate_module_record(&root).unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result"), Ok(Value::Boolean(true)));
}

#[test]
fn lossless_json_source_limits_count_scalar_and_unpaired_surrogate_widths() {
    for (units, bytes) in [
        (vec![0x22, 0xd800, 0x22], 5),
        (vec![0x22, 0xd83d, 0xde00, 0x22], 6),
        (vec![0x22, 0x4e2d, 0x22], 5),
    ] {
        for limit in [bytes - 1, bytes] {
            let mut realm = Realm::new(Limits {
                max_source_bytes: Some(limit),
                ..Limits::default()
            });
            let source = JsString::from_code_units(units.clone());
            let result = realm.create_json_module_utf16(&source);
            if limit < bytes {
                assert!(matches!(result, Err(Error::Limit { .. })));
            } else {
                let record = result.unwrap();
                realm.evaluate_module_record(&record).unwrap();
                assert_eq!(
                    default_value(&mut realm, &record),
                    Value::String(JsString::from_code_units(
                        units[1..units.len() - 1].to_vec()
                    ))
                );
            }
            assert!(
                realm
                    .create_json_module_utf16(&JsString::from("true"))
                    .is_ok()
            );
        }
    }
    let mut realm = Realm::default();
    let invalid = JsString::from_code_units(vec![0x22, 0x0, 0xd800, 0x22]);
    assert!(matches!(
        realm.create_json_module_utf16(&invalid),
        Err(Error::Exception {
            kind: ExceptionKind::SyntaxError,
            ..
        })
    ));
    let mut realm = Realm::new(Limits {
        max_steps: Some(2000),
        ..Limits::default()
    });
    let source = format!("\"{}\"", "x".repeat(4096));
    let source = JsString::from(source.as_str());
    assert!(matches!(
        realm.create_json_module_utf16(&source),
        Err(Error::Limit { .. })
    ));
    assert!(
        realm
            .create_json_module_utf16(&JsString::from("true"))
            .is_ok()
    );
}

#[test]
fn synthetic_binding_initializes_before_evaluation_and_evaluation_is_cached() {
    let mut realm = Realm::default();
    let module = realm.create_json_module("7").unwrap();
    realm.link_module(&module).unwrap();
    assert_eq!(default_value(&mut realm, &module), Value::Undefined);
    let first = realm.evaluate_module_record(&module).unwrap();
    drain(&mut realm);
    assert_eq!(default_value(&mut realm, &module), Value::Number(7.0));
    assert_eq!(realm.evaluate_module_record(&module).unwrap(), first);
    assert_eq!(
        realm.module_namespace(&module).unwrap(),
        realm.module_namespace(&module).unwrap()
    );
}

#[test]
fn intrinsic_parsing_ignores_public_json_and_preserves_object_layout() {
    let mut realm = Realm::default();
    realm.eval("JSON=undefined").unwrap();
    let data = realm
        .create_json_module(r#"{"__proto__":{"x":1},"dup":1,"dup":2,"nested":[true,null]}"#)
        .unwrap();
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    let root = realm
        .create_module(
            r#"
        import data, * as ns from 'data';
        globalThis.result=Object.getPrototypeOf(data)===Object.prototype &&
            Object.hasOwn(data,'__proto__') && data.__proto__.x===1 &&
            data.dup===2 && data.nested[0]===true && data.nested[1]===null &&
            Object.isExtensible(data) && Object.getPrototypeOf(ns)===null &&
            Reflect.ownKeys(ns).length===2 && !Reflect.set(ns,'default',0);
        try {data=0;result=false} catch(e) {result=result && e instanceof TypeError}
        data.dup=3;result=result && ns.default.dup===3;
    "#,
        )
        .unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(data.clone()))
        .unwrap();
    realm.evaluate_module_record(&root).unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result"), Ok(Value::Boolean(true)));
}

#[test]
fn shared_default_reexports_and_namespaces_resolve_to_one_json_value() {
    let mut realm = Realm::default();
    let data = realm.create_json_module(r#"{"x":7}"#).unwrap();
    let bridge = realm
        .create_module("export {default as data} from 'data';export * from 'data'")
        .unwrap();
    let root = realm
        .create_module(
            r#"
        import {data} from 'bridge';import direct,* as ns from 'data';
        globalThis.same=data===direct && ns.default===data && data.x===7 &&
            Object.keys(ns).join(',')==='default';
    "#,
        )
        .unwrap();
    realm
        .load_module_graph(&root, |_, request| {
            Ok(if request.specifier == JsString::from("bridge") {
                bridge.clone()
            } else {
                data.clone()
            })
        })
        .unwrap();
    realm.evaluate_module_record(&root).unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("same"), Ok(Value::Boolean(true)));
}

#[test]
fn explicit_attributes_and_top_level_await_use_the_normal_module_graph() {
    let mut realm =
        Realm::new_with_supported_import_attributes(Limits::default(), [JsString::from("type")]);
    let data = realm.create_json_module(r#"{"x":7}"#).unwrap();
    let root = realm
        .create_module(
            "import value from 'data' with {type:'json'};await 0;globalThis.result=value.x",
        )
        .unwrap();
    realm
        .load_module_graph(&root, |_, request| {
            assert_eq!(request.attributes.len(), 1);
            assert_eq!(request.attributes[0].key, JsString::from("type"));
            assert_eq!(request.attributes[0].value, JsString::from("json"));
            Ok(data.clone())
        })
        .unwrap();
    realm.evaluate_module_record(&root).unwrap();
    drain(&mut realm);
    assert_eq!(realm.eval("result"), Ok(Value::Number(7.0)));
}

#[test]
fn dynamic_import_completion_accepts_json_records_and_reuses_namespace_identity() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    let data = realm.create_json_module(r#"{"x":7}"#).unwrap();
    realm
        .eval("let a,b;import('data').then(ns=>{a=ns});import('data').then(ns=>{b=ns})")
        .unwrap();
    for _ in 0..8 {
        drain(&mut realm);
        let Some(request) = realm.take_module_import_request() else {
            break;
        };
        realm
            .finish_module_import(request, Ok(data.clone()))
            .unwrap();
    }
    drain(&mut realm);
    assert_eq!(
        realm.eval("a===b && a.default.x===7"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn invalid_text_and_opted_in_limits_leave_later_creation_usable() {
    let mut realm = Realm::default();
    for source in ["", "{", "[1,]", "undefined", "/*comment*/0"] {
        assert!(
            matches!(
                realm.create_json_module(source),
                Err(Error::Exception {
                    kind: ExceptionKind::SyntaxError,
                    ..
                })
            ),
            "{source}"
        );
    }
    assert!(realm.create_json_module("true").is_ok());
    let mut bytes = Realm::new(Limits {
        max_source_bytes: Some(4),
        ..Limits::default()
    });
    assert!(matches!(
        bytes.create_json_module("\"😀\""),
        Err(Error::Limit { .. })
    ));
    assert!(bytes.create_json_module("true").is_ok());
    let mut work = Realm::new(Limits {
        max_steps: Some(2000),
        ..Limits::default()
    });
    assert!(matches!(
        work.create_json_module(&format!("\"{}\"", "x".repeat(4096))),
        Err(Error::Limit { .. })
    ));
    assert!(work.create_json_module("true").is_ok());
    let mut strings = Realm::new(Limits {
        max_string_units: Some(0),
        ..Limits::default()
    });
    assert!(strings.create_json_module("true").is_ok());
    assert!(matches!(
        strings.create_json_module("\"x\""),
        Err(Error::Limit { .. })
    ));
}

#[test]
fn module_handles_retain_parsed_objects_and_collection_releases_unreachable_records() {
    let mut realm = Realm::default();
    let data = realm.create_json_module(r#"{"n":7}"#).unwrap();
    let root = realm
        .create_module(
            "import value from 'data';globalThis.data=value;globalThis.weak=new WeakRef(value);let key=Symbol();value[key]=1;globalThis.map=new WeakMap([[key,1]]);globalThis.symbolWeak=new WeakRef(key);key=null",
        )
        .unwrap();
    realm
        .load_module_graph(&root, |_, _| Ok(data.clone()))
        .unwrap();
    let promise = realm.evaluate_module_record(&root).unwrap();
    drain(&mut realm);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("weak.deref().n===7"), Ok(Value::Boolean(true)));
    realm.eval("data=null").unwrap();
    drop(promise);
    drop(root);
    drop(data);
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("weak.deref()===undefined && symbolWeak.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}
