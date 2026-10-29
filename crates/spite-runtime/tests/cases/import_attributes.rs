//! HostGetSupportedImportAttributes and loading-time attribute errors.

use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, PromiseJobOutcome, Realm, Value};

fn check(realm: &mut Realm, source: &str) {
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

fn drain(realm: &mut Realm) {
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => panic!("{error}"),
        }
    }
}

fn syntax(error: Error) {
    assert!(
        matches!(
            error,
            Error::Exception {
                kind: ExceptionKind::SyntaxError,
                ..
            }
        ),
        "{error}"
    );
}

#[test]
fn default_keys_reject_imports_through_fresh_intrinsic_promises_before_the_host_boundary() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let NativePromise=Promise,errors=0,events=[];
        globalThis.Promise=function(){throw 'public constructor'};
        let p,q;try{p=import('source',{with:{type:'json'}});q=import('source',{with:{type:'javascript'}});events.push('after')}
        catch(e){events.push('wrong')}finally{events.push('finally')}
        let fresh=p!==q&&Object.getPrototypeOf(p)===NativePromise.prototype;
        p.catch(e=>{if(e instanceof TypeError)errors++});q.catch(e=>{if(e instanceof TypeError)errors++});
    "#).unwrap();
    check(
        &mut realm,
        "fresh&&errors===0&&events.join(',')==='after,finally'",
    );
    drain(&mut realm);
    check(&mut realm, "errors===2");
}

#[test]
fn all_getters_and_value_validation_precede_unsupported_key_rejection() {
    let mut realm = Realm::default();
    realm.enable_module_imports();
    realm.eval(r#"
        let events=[],marker={},same=false,typed=false;
        import('source',{with:{get unknown(){events.push('unknown');return 'x'},get later(){events.push('later');throw marker}}}).catch(e=>same=e===marker);
        import('source',{with:{get unknown(){events.push('again');return 'x'},get later(){events.push('value');return 7}}}).catch(e=>typed=e instanceof TypeError);
    "#).unwrap();
    check(&mut realm, "events.join(',')==='unknown,later,again,value'");
    assert!(realm.take_module_import_request().is_none());
    drain(&mut realm);
    check(&mut realm, "same&&typed");
}

#[test]
fn supported_keys_forward_arbitrary_string_values_in_canonical_order_and_keep_utf16_identity() {
    let key = JsString::from_code_units(vec![0xdc00]);
    let mut realm = Realm::new_with_supported_import_attributes(
        Limits::default(),
        [key.clone(), JsString::from("type"), JsString::from("type")],
    );
    realm.enable_module_imports();
    realm
        .eval(r"import('source',{with:{'\uDC00':'\uDFFF',type:'host-defined'}})")
        .unwrap();
    let request = realm.take_module_import_request().unwrap();
    assert_eq!(request.request().attributes.len(), 2);
    assert_eq!(request.request().attributes[0].key, JsString::from("type"));
    assert_eq!(
        request.request().attributes[0].value,
        JsString::from("host-defined")
    );
    assert_eq!(request.request().attributes[1].key, key);
    assert_eq!(
        request.request().attributes[1].value.code_units(),
        &[0xdfff]
    );
    realm.eval("let rejected=false;import('bad',{with:{Type:'x'}}).catch(e=>rejected=e instanceof TypeError)").unwrap();
    assert!(realm.take_module_import_request().is_none());
    drain(&mut realm);
    check(&mut realm, "rejected");
}

#[test]
fn ignored_keys_do_not_require_support_and_normalization_is_not_applied() {
    let mut realm =
        Realm::new_with_supported_import_attributes(Limits::default(), [JsString::from("é")]);
    realm.enable_module_imports();
    realm
        .eval(
            r#"
        let rejected=false,attrs={[Symbol()]:7};Object.defineProperty(attrs,'unknown',{value:7});
        import('ignored',{with:attrs});import('exact',{with:{é:'x'}});
        import('different',{with:{'e\u0301':'x'}}).catch(e=>rejected=e instanceof TypeError);
    "#,
        )
        .unwrap();
    assert!(
        realm
            .take_module_import_request()
            .unwrap()
            .request()
            .attributes
            .is_empty()
    );
    assert_eq!(
        realm
            .take_module_import_request()
            .unwrap()
            .request()
            .attributes[0]
            .key,
        JsString::from("é")
    );
    assert!(realm.take_module_import_request().is_none());
    drain(&mut realm);
    check(&mut realm, "rejected");
}

#[test]
fn static_unknown_keys_fail_loading_and_linking_before_resolver_or_body_execution() {
    for source in [
        "import 'source' with {type:'json'};globalThis.entered=true",
        "export*from'source' with {unknown:'x'};globalThis.entered=true",
        "export {x} from 'source' with {unknown:'x'};globalThis.entered=true",
    ] {
        let mut realm = Realm::default();
        let module = realm.create_module(source).unwrap();
        assert_eq!(realm.module_requests(&module).unwrap().len(), 1);
        syntax(
            realm
                .load_module_graph(&module, |_, _| panic!("unsupported request reached host"))
                .unwrap_err(),
        );
        syntax(realm.link_module(&module).unwrap_err());
        syntax(realm.evaluate_module_record(&module).unwrap_err());
        check(&mut realm, "typeof entered==='undefined'");
    }
}

#[test]
fn nested_invalid_attributes_leave_graph_links_unpublished_and_retry_in_source_order() {
    let mut realm = Realm::default();
    let root = realm
        .create_module("import 'first';import 'second' with {unknown:'x'}")
        .unwrap();
    let first = realm.create_module("export{}").unwrap();
    let mut calls = Vec::new();
    for _ in 0..2 {
        syntax(
            realm
                .load_module_graph(&root, |_, request| {
                    calls.push(request.specifier.clone());
                    Ok(first.clone())
                })
                .unwrap_err(),
        );
    }
    assert_eq!(calls, [JsString::from("first"), JsString::from("first")]);
    let child = realm
        .create_module("import 'leaf' with {unknown:'x'}")
        .unwrap();
    let outer = realm.create_module("import 'child'").unwrap();
    let mut count = 0;
    syntax(
        realm
            .load_module_graph(&outer, |_, _| {
                count += 1;
                Ok(child.clone())
            })
            .unwrap_err(),
    );
    assert_eq!(count, 1);
}

#[test]
fn configured_static_attributes_load_and_evaluate_with_the_same_fixed_key_list() {
    let mut realm =
        Realm::new_with_supported_import_attributes(Limits::default(), [JsString::from("type")]);
    let root = realm.create_module("import {x} from 'source' with {type:'first'};export * from 'source' with {type:'second'};globalThis.result=x").unwrap();
    let dependency = realm.create_module("export const x=7").unwrap();
    let mut values = Vec::new();
    realm
        .load_module_graph(&root, |_, request| {
            values.push(request.attributes[0].value.clone());
            Ok(dependency.clone())
        })
        .unwrap();
    assert_eq!(values, [JsString::from("first"), JsString::from("second")]);
    realm.evaluate_module_record(&root).unwrap();
    check(&mut realm, "result===7");
    realm.enable_module_imports();
    realm
        .eval("import('source',{with:{type:'third'}})")
        .unwrap();
    assert_eq!(
        realm
            .take_module_import_request()
            .unwrap()
            .request()
            .attributes[0]
            .value,
        JsString::from("third")
    );
}
