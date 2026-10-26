//! Catchable Script module-syntax errors and retained dynamic-import host gaps.

use spite_runtime::{Error, Realm, Value};

#[test]
fn eval_and_dynamic_function_constructors_report_catchable_syntax_errors() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(r#"
        let sources=["import 'source'","import value from 'source'","export {}","export default 1","import.meta"],count=0,constructors=[Function,(async function(){}).constructor,(function*(){}).constructor];
        for(const source of sources){try{eval(source)}catch(error){if(error instanceof SyntaxError)count++}for(const constructor of constructors){try{constructor(source)}catch(error){if(error instanceof SyntaxError)count++}}}
        count===20
    "#),Ok(Value::Boolean(true)));
}

#[test]
fn dynamic_import_still_aborts_language_handlers_as_an_unsupported_feature() {
    let mut realm = Realm::default();
    realm.eval("let caught=0,finalized=0").unwrap();
    let error = realm
        .eval("try{Function(\"import('source')\")()}catch{caught++}finally{finalized++}")
        .unwrap_err();
    assert!(matches!(error, Error::Unsupported { .. }), "{error}");
    assert_eq!(
        realm.eval("caught===0&&finalized===0"),
        Ok(Value::Boolean(true))
    );
}
