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
fn invalid_statement_continuations_throw_before_eval_or_function_body_effects() {
    for source in [
        "debugger(0)",
        "debugger[0]",
        "debugger.value",
        "debugger?.value",
        "while(false){break(0)}",
        "while(false){continue[0]}",
        "label:while(false){break label.value}",
        "var x(0)",
        "let x[0]",
    ] {
        let body = format!("effects=1;{source}");
        let script = format!(
            "let effects=0,caught=0,finalized=0;const compilers=[s=>eval(s),s=>(0,eval)(s),Function,(async function(){{}}).constructor,(function*(){{}}).constructor,(async function*(){{}}).constructor];for(const compile of compilers){{try{{compile({body:?})}}catch(error){{if(error instanceof SyntaxError)caught++}}finally{{finalized++}}}}effects===0&&caught===6&&finalized===6"
        );
        assert_eq!(
            Realm::default().eval(&script),
            Ok(Value::Boolean(true)),
            "{source}"
        );
    }
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
