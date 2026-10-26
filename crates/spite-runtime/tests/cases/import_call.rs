//! Parsed dynamic import remains a host boundary only when executed.

use spite_runtime::{Error, Realm, Value};

#[test]
fn unexecuted_imports_and_dynamic_function_source_are_available_without_a_loader() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        let f=Function("return import('source')"),a=(async()=>{}).constructor("return import('source')"),g=(function*(){}).constructor("return import('source')");
        false&&import('unreached');if(false)import('unreached',{});
        typeof f==='function'&&typeof a==='function'&&typeof g==='function'&&f.toString().includes("import('source')")
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn executing_an_import_evaluates_operands_then_aborts_handlers_without_jobs() {
    let mut realm = Realm::default();
    realm.eval("let effects=0,caught=0,finalized=0").unwrap();
    let error = realm
        .eval("try{import((effects++,'source'),(effects++,{}))}catch{caught++}finally{finalized++}")
        .unwrap_err();
    assert!(matches!(error, Error::Unsupported { .. }), "{error}");
    assert_eq!(
        realm.eval("effects===2&&caught===0&&finalized===0"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.run_next_promise_job().unwrap(),
        spite_runtime::PromiseJobOutcome::NoJob
    ));
}

#[test]
fn malformed_dynamic_imports_are_catchable_syntax_errors_in_eval_and_function() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        let errors=0;for(const source of ["import()","import('x',,)","import(...args)","import('x',1,2)","new import('x')"]){try{eval(source)}catch(error){if(error instanceof SyntaxError)errors++}try{Function(source)}catch(error){if(error instanceof SyntaxError)errors++}}
        errors===10
    "#),Ok(Value::Boolean(true)));
}
