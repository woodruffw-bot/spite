//! Resource early errors and remaining await-using host boundaries.

use crate::common::UnavailableHostRealm;
use spite_runtime::{Error, Realm, Value};

#[test]
fn await_using_boundaries_abort_handlers_for_statements_and_loops() {
    for source in [
        "async function f(){await using x=null;}f();",
        "async function f(){let value;using x=([value=await spiteUnavailableHost()]=[]);}f();",
        "async function f(){for await(await using x of []){}}f();",
    ] {
        let mut realm = Realm::with_unavailable_host();
        realm.eval("let flag=0").unwrap();
        assert!(
            matches!(
                realm.eval(&format!(
                    "try{{eval({source:?})}}catch(e){{flag=1}}finally{{flag=2}}"
                )),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}

#[test]
fn actual_using_early_errors_are_catchable_in_eval_and_dynamic_functions() {
    assert_eq!(Realm::default().eval("let caught=0;for(const source of ['using x=null;','{using x;}','switch(0){case 0:using x=null;}']){try{eval(source)}catch(e){if(e instanceof SyntaxError)caught++}}try{Function('x','using x=null;')}catch(e){if(e instanceof SyntaxError)caught++}caught"),Ok(Value::Number(4.0)));
}

#[test]
fn ordinary_using_identifiers_and_unexecuted_resource_bodies_are_preserved() {
    assert_eq!(Realm::default().eval("let using=1;{using\nusing=2;}var of=[[9],[8],[7]],result=[];for(using of of[0,1,2])result.push(using);function f(){using x=null;}let g=Function('using x=null;');using===7&&result.join(',')==='7'&&typeof f==='function'&&g.toString().includes('using x=null;')"),Ok(Value::Boolean(true)));
}
