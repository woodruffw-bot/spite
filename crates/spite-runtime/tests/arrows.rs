//! Arrow closure capture, call environments, function metadata, and limits.

mod common;
use common::REALM_ENTRIES;

use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};

fn number(source: &str, expected: f64) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Number(expected)),
        "{source}"
    );
}

fn string(source: &str, expected: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::String(expected.into())),
        "{source}"
    );
}

#[test]
fn arguments_bind_in_order_and_expression_bodies_return_their_value() {
    number("((x,y)=>x+y)(3,4)", 7.0);
    number(
        "let f = x => x; let flag = 0; f(flag = 1, flag = 2); flag",
        2.0,
    );
    number("((x,y)=>x-y).call(null, 7, 2)", 5.0);
    number("((x,y)=>x-y).apply(null, {0:7, 1:2, length:2})", 5.0);
    number("((x,y)=>x-y).bind(null, 7)(2)", 5.0);
    number("let f = x=>x=3; let x=9; f(1)+x", 12.0);
    assert_eq!(
        Realm::default().eval("((x,y)=>x===1 && y===undefined)(1)"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        Realm::default().eval("let o={}; (x=>x)(o)===o"),
        Ok(Value::Boolean(true))
    );
    number("((x)=>({value:x}))(7).value", 7.0);
    number("let flag=0; let f=()=>flag=1; flag", 0.0);
}

#[test]
fn closures_retain_shared_bindings_and_use_defining_not_calling_scopes() {
    number("let x=1; let f=()=>x; {let x=9; f();}", 1.0);
    number("let x=1; let f=()=>x; x=7; f()", 7.0);
    number("let make=x=>()=>++x; let f=make(0); f()+f()", 3.0);
    number(
        "let make=x=>()=>++x; let a=make(0), b=make(10); a()+b()+a()",
        14.0,
    );
    number(
        "let pair=x=>({get:()=>x, set:y=>x=y}); let p=pair(1); p.set(8); p.get()",
        8.0,
    );
    number("let f; {let x=7; f=()=>x;} f()", 7.0);
    number("let f; try {throw 9;} catch(e) {f=()=>e;} f()", 9.0);
    number("let f=n=>n<2?1:n*f(n-1); f(5)", 120.0);
    number("let x=3; let f=()=>g(); let g=()=>x; f()", 3.0);
    assert!(matches!(
        Realm::default().eval("let f=()=>x; f(); let x=1"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert!(matches!(
        Realm::default().eval("const x=1; let f=()=>x=2; f()"),
        Err(Error::Exception {
            kind: ExceptionKind::TypeError,
            ..
        })
    ));
}

#[test]
fn loop_closures_retain_each_iteration_and_var_remains_shared() {
    number(
        "let fs={}; for(let i=0;i<3;i++) fs[i]=()=>i; fs[0]()*100+fs[1]()*10+fs[2]()",
        12.0,
    );
    number(
        "let fs={}; for(var i=0;i<3;i++) fs[i]=()=>i; fs[0]()+fs[1]()+fs[2]()",
        9.0,
    );
    number(
        "let fs={}; for(let i=0;i<3;i++) { let x=i+10; fs[i]=()=>x; } fs[0]()+fs[1]()+fs[2]()",
        33.0,
    );
    number(
        "let fs={}; for(let i=0;i<3;i++) { fs[i]=()=>++i; } fs[0]()+fs[1]()+fs[2]()",
        6.0,
    );
}

#[test]
fn strictness_and_scope_restore_after_success_language_errors_and_host_limits() {
    let mut realm = Realm::default();
    realm.eval("let sloppy=()=>created=7").unwrap();
    assert_eq!(
        realm.eval("'use strict'; sloppy(); created"),
        Ok(Value::Number(7.0))
    );
    realm
        .eval("'use strict'; let strict=()=>unbound=1")
        .unwrap();
    assert!(matches!(
        realm.eval("strict()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
    assert_eq!(
        realm.eval("let f=x=>missing; let x=9; try {f(1);} catch {} x"),
        Ok(Value::Number(9.0))
    );
    assert_eq!(
        realm.eval("let flag=0; try {strict();} catch {flag=2;} finally {flag+=3;} flag"),
        Ok(Value::Number(5.0))
    );
    realm.eval("let recurse=()=>recurse(); flag=0").unwrap();
    assert!(matches!(
        realm.eval("try {recurse();} catch {flag=1;} finally {flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    assert_eq!(realm.eval("sloppy()"), Ok(Value::Number(7.0)));
}

#[test]
fn name_inference_distinguishes_identifier_targets_and_prototype_setters() {
    for (source, expected) in [
        ("let f=()=>1; f.name", "f"),
        ("var f=()=>1; f.name", "f"),
        ("let f; f=()=>1; f.name", "f"),
        ("let f; f=((()=>1)); f.name", "f"),
        ("let f; (f)=()=>1; f.name", ""),
        ("let f; f ||= ()=>1; f.name", "f"),
        ("let f=1; f &&= ()=>1; f.name", "f"),
        ("let f; f ??= ()=>1; f.name", "f"),
        ("let f; (f) ||= ()=>1; f.name", ""),
        ("let o={}; o.x=()=>1; o.x.name", ""),
        ("let o={}; o.x ||= ()=>1; o.x.name", ""),
        ("({x:()=>1}).x.name", "x"),
        ("({['x']:()=>1}).x.name", "x"),
        ("({[3]:()=>1})[3].name", "3"),
        ("({['__proto__']:()=>1})['__proto__'].name", "__proto__"),
        ("({__proto__:()=>1}).name", ""),
        ("let f=(0,()=>1); f.name", ""),
        ("let f=true?()=>1:()=>2; f.name", ""),
        ("let f=()=>1; let g=f; g.name", "f"),
    ] {
        string(source, expected);
    }
}

#[test]
fn arrow_metadata_source_and_lexical_arguments_are_standard() {
    string(
        "let f=(x /* source */ , y) => (x+y); f.toString()",
        "(x /* source */ , y) => (x+y)",
    );
    string("let f=()=>1; delete f.name; f.toString()", "()=>1");
    string("let f=()=>1; ({}).toString.call(f)", "[object Function]");
    for source in [
        "let f=(x,y)=>x; f.length===2 && f.name==='f' && typeof f==='function'",
        "!('prototype' in (()=>1))",
        "let f=()=>1; f!==(()=>1)",
        "let arguments=7; (()=>arguments)()===7",
        "((arguments)=>arguments)(7)===7",
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(true)),
            "{source}"
        );
    }
    for source in [
        "(()=>1).caller",
        "(()=>1).arguments",
        "(()=>1).caller=7",
        "'use strict'; let f=()=>1; f.length=7",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{source}"
        );
    }
    assert!(matches!(
        Realm::default().eval("(()=>arguments)()"),
        Err(Error::Exception {
            kind: ExceptionKind::ReferenceError,
            ..
        })
    ));
}

#[test]
fn closures_and_captured_cycles_survive_host_roots_and_are_reclaimed() {
    let mut realm = Realm::default();
    let value = realm
        .eval("let f; {let object={x:7}; f=()=>object; object.f=f;} f")
        .unwrap();
    let root = realm.root_value(value, 100).unwrap();
    realm.collect(2000).unwrap();
    assert_eq!(
        realm.eval("f().f===f && f().x===7"),
        Ok(Value::Boolean(true))
    );
    // The receiver of a getter/coercion and captured object both stay alive
    // through explicit collection, including an unreachable capture cycle later.
    realm.collect(2000).unwrap();
    realm.eval("f=null").unwrap();
    assert_eq!(realm.collect(2000).unwrap().live, REALM_ENTRIES + 3); // global, intrinsics, block, object, arrow
    drop(root);
    assert_eq!(realm.collect(2000).unwrap().reclaimed, 3);
}

#[test]
fn user_conversion_methods_observe_hint_and_operand_evaluation_order() {
    number(
        "let n=0; let o={valueOf:()=>(n=n*10+1,{}), toString:()=>(n=n*10+2,'3')}; +o; n",
        12.0,
    );
    number(
        "let n=0; let o={valueOf:()=>(n=9,3), toString:()=>(n=2,'3')}; `${o}`; n",
        2.0,
    );
    number(
        "let n=0; let a={valueOf:()=>(n=n*10+3,1)}; let b={valueOf:()=>(n=n*10+4,2)}; (n=1,a)+(n=12,b); n",
        1234.0,
    );
    number(
        "let n=0; let a={valueOf:()=>(n=n*10+1,1)}; let b={valueOf:()=>(n=n*10+2,2)}; a>b; n",
        12.0,
    );
    number(
        "let flag=0; let f=()=>missing; try {let o={valueOf:f}; +o;} catch {flag=7;} flag",
        7.0,
    );
}

#[test]
fn arrow_limits_and_early_errors_precede_later_effects() {
    let mut realm = Realm::default();
    realm.eval("let flag=0").unwrap();
    assert!(matches!(
        realm.eval("flag=1; (x,x)=>x"),
        Err(Error::Parse(_))
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
    let mut realm = Realm::new(Limits {
        max_string_units: 10,
        ..Limits::default()
    });
    realm
        .eval("let f=(parameter)=>parameter; let flag=0")
        .unwrap();
    assert!(matches!(
        realm.eval("try {f.toString();} finally {flag=1;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}

#[test]
fn user_key_conversion_observes_deferred_simple_writes_and_cached_compound_keys() {
    string(
        "let log=''; let key={toString:()=>(log+='k','x')}; let o={}; o[key]=(log+='r',1); log",
        "rk",
    );
    string(
        "let log=''; let key={toString:()=>(log+='k','x')}; let o={x:1}; o[key]+=(log+='r',2); log",
        "kr",
    );
    number(
        "let key={toString:()=>'x'}; let o={x:1}; o[key]+=(key.toString=()=>missing,2); o.x",
        3.0,
    );
    string(
        "let log=''; let key={toString:()=>(log+='k','x')}; try {null[key]=(log+='r',1);} catch {} log",
        "r",
    );
    string(
        "let log=''; let key={toString:()=>(log+='k','x')}; try {key in (log+='r',0);} catch {} log",
        "r",
    );
    string(
        "let log=''; let key={toString:()=>(log+='k','x')}; ({[key]:(log+='v',1)}); log",
        "kv",
    );
}
