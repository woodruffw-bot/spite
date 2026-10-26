//! Async-generator declarations instantiate without executing their bodies.

use spite_runtime::{Realm, Value};

#[test]
fn unexecuted_async_generator_syntax_and_dynamic_bodies_compile_without_host_failure() {
    let mut realm = Realm::default();
    assert_eq!(realm.eval("let f=()=>{async function*g(){yield await 1}};if(false){async function*h(){}}if(false)({async*g(){}});let compiled=Function('async function*g(){yield await 1}');typeof f==='function'&&typeof compiled==='function'&&compiled.toString().includes('async function*g()')"),Ok(Value::Boolean(true)));
}

#[test]
fn async_generator_function_creation_completes_without_entering_bodies() {
    for statement in [
        "(async function*g(){yield await 1})",
        "({async*g(){yield await 1}})",
        "class C{async*g(){yield await 1}}",
        "class C{static async*#g(){yield await 1}}",
        "{async function*g(){yield await 1}}",
    ] {
        let mut realm = Realm::default();
        realm.eval("let flag=0").unwrap();
        realm
            .eval(&format!(
                "try{{flag=3;{statement};}}catch{{flag=1}}finally{{flag=2}}"
            ))
            .unwrap();
        assert_eq!(realm.eval("flag"), Ok(Value::Number(2.0)), "{statement}");
    }
}

#[test]
fn dynamic_function_constructors_compile_and_instantiate_nested_async_generator_declarations() {
    for expression in [
        "Function('async function*g(){}')",
        "new Function('async function*g(){}')",
        "Function.call(null,'async function*g(){}')",
        "new (Function.bind(null))('async function*g(){}')",
        "Reflect.construct(Function,['async function*g(){}'])",
    ] {
        let mut realm = Realm::default();
        assert_eq!(
            realm.eval(&format!(
                "let flag=0,compiled={expression};typeof compiled==='function'"
            )),
            Ok(Value::Boolean(true)),
            "{expression}"
        );
        realm
            .eval("try{flag=3;compiled()}catch{flag=1}finally{flag=2}")
            .unwrap();
        assert_eq!(realm.eval("flag"), Ok(Value::Number(2.0)), "{expression}");
    }
}
