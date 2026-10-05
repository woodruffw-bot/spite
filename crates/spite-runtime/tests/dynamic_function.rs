//! Ordinary dynamic Function compilation, construction, and host-abort boundaries.

mod common;

use spite_runtime::{Error, Limits, Realm, Value};

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn calls_construction_bound_calls_and_reflect_compile_the_same_source() {
    for factory in [
        "Function",
        "new Function",
        "Function.bind(null)",
        "new (Function.bind(null))",
    ] {
        check(&format!(
            "let f={factory}('a','b,c','return a+b+c');f(1,2,3)===6 && f.name==='anonymous' && f.length===3"
        ));
    }
    check(
        "Function()()===undefined && new Function()()===undefined && Function.call(7,'return 3')()===3 && Reflect.construct(Function,['return 4'])()===4",
    );
    check(
        "let f=Function('a','b=2','...rest','return [a,b,rest.length,arguments.length]');f.length===1 && f(1,undefined,3,4).join(',')==='1,2,2,4'",
    );
}

#[test]
fn global_scope_capture_ignores_the_callers_local_bindings_and_strictness() {
    check("let x=7;function make(){let x=9;return Function('return x');}make()()===7");
    check("let x=7,f;{let x=9;f=Function('return x');}f()===7");
    check(
        "'use strict';Function('return this')()===globalThis && Function('return this').call(3).valueOf()===3",
    );
    check("let f=Function('\"use strict\";return this');f()===undefined && f.call(3)===3");
    check(
        "let anonymous=7;Function('return anonymous')()===7 && Function('anonymous','return anonymous')(9)===9",
    );
    check(
        "Function('a,a','arguments[0]=7;return a')(1,2)===2 && Function('a,a','arguments[1]=7;return a')(1,2)===7",
    );
    check(
        "let caught=false;try{Function('return callerLocal')();}catch(e){caught=e instanceof ReferenceError;}caught",
    );
}

#[test]
fn conversions_precede_parsing_and_prototype_reads_in_argument_order() {
    check(
        "let log=[],proto={},target=(function(){}).bind(null);Object.defineProperty(target,'prototype',{get(){log.push('prototype');return proto;}});let p={toString(){log.push('parameter');return 'a';}},b={toString(){log.push('body');return 'return a';}},f=Reflect.construct(Function,[p,b],target);log.join(',')==='parameter,body,prototype' && Object.getPrototypeOf(f)===proto && f(7)===7",
    );
    check(
        "let log=[],target=(function(){}).bind(null);Object.defineProperty(target,'prototype',{get(){log.push('prototype');return {};}});let caught=false;try{Reflect.construct(Function,[{toString(){log.push('parameter');return 'a=';}},{toString(){log.push('body');return '1';}}],target);}catch(e){caught=e instanceof SyntaxError;}caught && log.join(',')==='parameter,body'",
    );
    check(
        "let log=[],error={},caught=false;try{Function({toString(){log.push('first');throw error;}},{toString(){log.push('second');return 'b';}},{toString(){log.push('body');return '';}});}catch(e){caught=e===error;}caught && log.join(',')==='first'",
    );
    check(
        "let n=0,caught=false;try{Function({toString(){n++;return 'a';}},Symbol());}catch(e){caught=e instanceof TypeError;}caught && n===1",
    );
}

#[test]
fn separate_goals_and_combined_early_errors_throw_catchable_syntaxerrors() {
    for expression in [
        "Function('/*','*/ ) {')",
        "Function('a) { return 1; } //','return 2')",
        "Function('} function injected() {')",
        "Function('a=','1')",
        "Function('...rest,','')",
        "Function('a,a','\"use strict\";')",
        "Function('eval','\"use strict\";')",
        "Function('a=1','\"use strict\";')",
        "Function('a','let a;')",
        "Function('break;')",
        "Function('continue;')",
        "Function('return (')",
        "Function('let a;var a;')",
    ] {
        check(&format!(
            "let caught=false,finalized=false;try{{{expression};}}catch(e){{caught=e instanceof SyntaxError;}}finally{{finalized=true;}}caught && finalized"
        ));
    }
}

#[test]
fn newtarget_controls_the_function_prototype_after_successful_parsing() {
    check(
        "function target(){}let f=Reflect.construct(Function,['return 7'],target);Object.getPrototypeOf(f)===target.prototype && f()===7 && Object.getPrototypeOf(f.prototype)===Object.prototype && f.prototype.constructor===f",
    );
    check(
        "let target=(function(){}).bind(null);target.prototype=7;let f=Reflect.construct(Function,['return 7'],target);Object.getPrototypeOf(f)===Function.prototype && f()===7",
    );
    check(
        "let count=0,error={},target=(function(){}).bind(null);Object.defineProperty(target,'prototype',{get(){count++;throw error;}});let caught=false;try{Reflect.construct(Function,['return 7'],target);}catch(e){caught=e===error;}caught && count===1",
    );
    check(
        "let n=0,caught=false;try{Reflect.construct(Function,[{toString(){n++;return ''}}],()=>0);}catch(e){caught=e instanceof TypeError;}caught && n===0",
    );
}

#[test]
fn dynamic_functions_are_ordinary_constructors_with_fresh_identity_and_newtarget() {
    check(
        "let f=Function('x','this.x=x;this.target=new.target'),a=new f(7);a.x===7 && a.target===f && a instanceof f && Object.getPrototypeOf(a)===f.prototype",
    );
    check(
        "let f=Function('return new.target'),g=function(){};f()===undefined && new f()===f && Reflect.construct(f,[],g)===g",
    );
    check("let f=Function('x=new.target','return x');f()===undefined && new f()===f");
    check(
        "let f=Function('return {}'),g=Function('return {}');f!==g && f.prototype!==g.prototype && f()!==f() && f instanceof Function",
    );
}

#[test]
fn metadata_and_tostring_preserve_exact_standard_source() {
    check(
        "let f=Function('a /* parameter */','return a // body');f.toString()==='function anonymous(a /* parameter */\\n) {\\nreturn a // body\\n}' && Function().toString()==='function anonymous(\\n) {\\n\\n}'",
    );
    check(
        "let f=Function('a','return a'),n=Object.getOwnPropertyDescriptor(f,'name'),l=Object.getOwnPropertyDescriptor(f,'length'),p=Object.getOwnPropertyDescriptor(f,'prototype');n.value==='anonymous' && !n.writable && !n.enumerable && n.configurable && l.value===1 && !l.writable && !l.enumerable && l.configurable && p.value.constructor===f && p.writable && !p.enumerable && !p.configurable && Object.getPrototypeOf(f)===Function.prototype",
    );
    check(
        "let f=Function('return 7'),s=f.toString();Object.defineProperty(f,'name',{get(){throw 1;}});f.toString()===s && f()===7",
    );
    check(
        "Function('return \"😀\"')()==='😀' && Function('return \"\\\\uD800\"')().charCodeAt(0)===0xD800",
    );
}

#[test]
fn default_source_sizes_are_unlimited_and_compiled_code_survives_collection() {
    check("Function('/*'+'a'.repeat(1050000)+'*/return 7')()===7");
    let mut realm = Realm::default();
    realm
        .eval("let value={x:7},f=Function('return value');")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("f().x"), Ok(Value::Number(7.0)));
    realm.eval("let saved=Function,proto=Function.prototype;delete globalThis.Function;delete proto.constructor;").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("Object.getPrototypeOf(saved('return 7'))===proto && saved('return 7')()===7"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn unsupported_source_is_a_host_abort_after_all_observable_conversions() {
    for body in ["class C{}", "function* g(){}", "async function f(){}"] {
        let mut realm = Realm::default();
        realm.eval("let flag=0;").unwrap();
        let body = format!("'{body}'");
        assert!(matches!(realm.eval(&format!("try{{Function({{toString(){{flag=3;return 'a';}}}},{{toString(){{flag=4;return {body};}}}});}}catch{{flag=5;}}finally{{flag=6;}}")),Err(Error::Unsupported{..})));
        assert_eq!(realm.eval("flag"), Ok(Value::Number(4.0)));
    }
}

#[test]
fn opt_in_source_work_string_and_heap_quotas_skip_javascript_handlers() {
    for limits in [
        Limits {
            max_source_bytes: Some(150),
            ..Limits::default()
        },
        Limits {
            max_steps: Some(600),
            ..Limits::default()
        },
        Limits {
            max_string_units: Some(230),
            ..Limits::default()
        },
        Limits {
            max_heap_entries: Some(common::REALM_ENTRIES + 1),
            ..Limits::default()
        },
    ] {
        let mut realm = Realm::new(limits);
        realm.eval("let flag=0;").unwrap();
        assert!(matches!(realm.eval("try{Function('/*'+'a'.repeat(210)+'*/return 7');}catch{flag=1;}finally{flag=2;}"),Err(Error::Limit{..})),"{limits:?}");
        assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    }
}
