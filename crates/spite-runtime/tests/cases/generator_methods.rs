//! Generator method definitions preserve key effects and defer their bodies.

use spite_runtime::{PromiseJobOutcome, Realm, Value};

#[test]
fn object_class_static_and_private_methods_create_functions_without_running_bodies() {
    for (definition, before) in [
        ("({*[(before+=2,'g')](){operand++;yield 1}})", 3.0),
        ("class C{*[(before+=2,'g')](){operand++;yield 1}}", 3.0),
        (
            "class C{static *[(before+=2,'g')](){operand++;yield 1}}",
            3.0,
        ),
        ("class C{*#g(){operand++;yield 1}}", 1.0),
    ] {
        let mut realm = Realm::default();
        realm.eval("let before=0,operand=0,handled=0").unwrap();
        realm
            .eval(&format!(
                "try{{before++;{definition}}}catch{{handled=1}}finally{{handled=2}}"
            ))
            .unwrap();
        assert_eq!(realm.eval("before"), Ok(Value::Number(before)));
        assert_eq!(
            realm.eval("operand===0&&handled===2"),
            Ok(Value::Boolean(true))
        );
        assert_eq!(
            realm.eval("typeof C==='undefined'"),
            Ok(Value::Boolean(true))
        );
    }
}

#[test]
fn method_creation_after_an_awaited_key_completes_with_disposal_and_no_body_execution() {
    let mut realm = Realm::default();
    realm.eval(r#"
        let release,gate=new Promise(resolve=>{release=resolve}),operand=0,disposed=0,finalized=0,caught=false,result;
        let resource={[Symbol.dispose](){disposed++}},weak=new WeakRef(resource[Symbol.dispose]);
        async function f(){using r=resource;try{return{*[await gate](){operand++;yield 1}}}finally{finalized++}}f().then(value=>{result=value},()=>{caught=true});delete resource[Symbol.dispose];release('g');
    "#).unwrap();
    loop {
        match realm.run_next_promise_job().unwrap() {
            PromiseJobOutcome::NoJob => break,
            PromiseJobOutcome::Completed => (),
            PromiseJobOutcome::Reported(error) => {
                panic!("{error}");
            }
        }
    }
    realm.clear_kept_objects().unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("operand===0&&disposed===1&&finalized===1&&!caught&&typeof result.g==='function'&&weak.deref()===undefined"),
        Ok(Value::Boolean(true))
    );
}
