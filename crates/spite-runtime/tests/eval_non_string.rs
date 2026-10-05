//! %eval% metadata and PerformEval's non-String identity branch (19.2.1.1).

use spite_runtime::{Error, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn non_string_values_return_unchanged_without_coercion() {
    check(
        "eval()===undefined && eval(undefined)===undefined && eval(null)===null && eval(true)===true && eval(7)===7 && Object.is(eval(-0),-0) && Object.is(eval(NaN),NaN) && eval(3n)===3n",
    );
    check(
        "let value={toString(){throw 1;},valueOf(){throw 2;},[Symbol.toPrimitive](){throw 3;}};eval(value)===value && eval(Symbol.for('x'))===Symbol.for('x')",
    );
    check("let value=new String('throw 7');eval(value)===value");
    check(
        "let value={};function f(){'use strict';return eval(value)===value && (0,eval)(value)===value && eval.call(null,value)===value && Reflect.apply(eval,undefined,[value])===value;}f()",
    );
}

#[test]
fn intrinsic_metadata_and_construction_match_an_ordinary_builtin() {
    check(
        "eval.name==='eval' && eval.length===1 && Object.getPrototypeOf(eval)===Function.prototype && Object.getOwnPropertyNames(eval).join(',')==='length,name' && !('prototype' in eval) && eval.toString()==='function eval() { [native code] }'",
    );
    check(
        "let d=Object.getOwnPropertyDescriptor(globalThis,'eval'),n=Object.getOwnPropertyDescriptor(eval,'name'),l=Object.getOwnPropertyDescriptor(eval,'length');d.value===eval && d.writable && !d.enumerable && d.configurable && !n.writable && !n.enumerable && n.configurable && !l.writable && !l.enumerable && l.configurable",
    );
    check(
        "let flag=0,error;try{new eval(flag++);}catch(e){error=e instanceof TypeError;}error && flag===1",
    );
}

#[test]
fn declarations_preserve_the_intrinsic_and_assignment_can_replace_it() {
    check("let original=eval;var eval;eval===original");
    check("var eval=7;eval===7 && globalThis.eval===7");
    check("function f(eval){return eval();}f(()=>7)===7");
    check(
        "var eval;let object={eval:()=>7};with(object){if(eval()!==7)throw 1;}typeof eval==='function'",
    );
    check("eval=()=>7;eval()===7 && globalThis.eval()===7");
    check("let original=eval;delete globalThis.eval;typeof eval==='undefined' && original(7)===7");
    let mut realm = Realm::default();
    realm.eval("delete globalThis.eval;").unwrap();
    assert_eq!(realm.eval("var eval;eval"), Ok(Value::Undefined));
}

#[test]
fn unsupported_eval_syntax_preserves_argument_effects_and_bypasses_handlers() {
    for source in [
        "eval('class C{field;}')",
        "(eval)('class C{field;}')",
        "e\\u0076al('class C{field;}')",
        "function f(eval){return eval('class C{field;}');}f(globalThis.eval)",
        "with({eval:globalThis.eval}){eval('class C{field;}');}",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    let mut realm = Realm::default();
    realm.eval("let flag=0;").unwrap();
    assert!(matches!(
        realm.eval("try{eval((flag=7,'class C{field;}'));}catch(e){flag=1;}finally{flag=2;}"),
        Err(Error::Unsupported { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(7.0)));
}

#[test]
fn the_original_intrinsic_remains_live_after_global_replacement_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let original=eval;eval=7;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("original(9)"), Ok(Value::Number(9.0)));
}
