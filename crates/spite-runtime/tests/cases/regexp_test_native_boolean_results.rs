//! RegExp test preserves RegExpExec effects while omitting discarded native results.
use spite_core::JsString;
use spite_runtime::{Error, ExceptionKind, Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn native_test_boolean_results_snapshot() {
    let mut rows = String::new();
    for pattern in [
        "a",
        "(a+)",
        "(?<x>a*)",
        "[ab]",
        "^a$",
        "\\b[a]\\B",
        "😀",
        "\\uD800",
        "",
        "(a|){2,3}",
        "(|a){3,}",
        "([ſK]+)",
    ] {
        let pattern = JsString::from(pattern);
        for flags in ["", "g", "y", "dgy", "dug", "dvy", "diug", "dimvg"] {
            for input in [
                JsString::from("baaa"),
                JsString::from("😀a"),
                JsString::from("ſK\na"),
                JsString::from_code_units(vec![0xd800, 0x61, 0xdc00]),
                JsString::from(""),
            ] {
                for start in [0, 1, 2, input.len(), input.len() + 1] {
                    // Nullable alternatives belong to the supported Unicode proof.
                    if pattern.code_units().contains(&u16::from(b'|'))
                        && pattern.code_units().contains(&u16::from(b'{'))
                        && !flags.contains(['u', 'v'])
                    {
                        continue;
                    }
                    let script = format!(
                        "let r=new RegExp({pattern:?},'{flags}'),e=new RegExp(r);r.lastIndex={start};e.lastIndex={start};let b=r.test({input:?}),m=e.exec({input:?});JSON.stringify({{test:b,lastIndex:r.lastIndex,exec:m!==null,execLastIndex:e.lastIndex}})"
                    );
                    let Value::String(result) = Realm::default()
                        .eval(&script)
                        .unwrap_or_else(|error| panic!("{script}: {error:?}"))
                    else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} {result:?}"
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn native_test_preserves_conversion_lookup_and_last_index_order() {
    check(
        "let t='',r=/a/g;Object.defineProperty(r,'exec',{get(){t+='e';return RegExp.prototype.exec;}});r.lastIndex={valueOf(){t+='i';return 0}};let b=r.test({toString(){t+='s';return 'a'}});b&&t==='sei'&&r.lastIndex===1",
    );
    check(
        "let t='',r=/a/;Object.defineProperty(r,'exec',{get(){t+='e';return RegExp.prototype.exec;}});let i={valueOf(){t+='i';return 99}};r.lastIndex=i;let b=r.test({toString(){t+='s';return 'a'}});b&&t==='sei'&&r.lastIndex===i",
    );
    check(
        "let t='',r=/a/g;r.lastIndex={valueOf(){t+='i';throw 7}};try{r.test({toString(){t+='s';return 'a'}})}catch(e){t+=e}t==='si7'",
    );
    check(
        "let t='',r=/a/g;Object.defineProperty(r,'exec',{get(){t+='e';throw 8}});try{r.test({toString(){t+='s';throw 7}})}catch(e){t+=e}t==='s7'",
    );
    check(
        "let r=/a/g;Object.defineProperty(r,'exec',{get(){r.lastIndex=1;return RegExp.prototype.exec}});!r.test('a')&&r.lastIndex===0",
    );
    check(
        "let r=/a/g,b=r.test({toString(){r.exec=function(s){return s==='a'?{}:null};return 'a'}});b&&r.lastIndex===0",
    );
}

#[test]
fn custom_exec_results_and_bound_native_exec_keep_their_calls() {
    check(
        "let r=/a/g,n=0;r.exec=function(s){n++;return this===r&&s==='a'&&arguments.length===1?{}:null};r.test('a')&&n===1&&r.lastIndex===0",
    );
    check(
        "let r=/a/g;r.exec=function(){return {get index(){throw 7},get length(){throw 8}}};r.test('')&&r.lastIndex===0",
    );
    check("let r=/a/g;r.exec=function(){return null};!r.test('a')&&r.lastIndex===0");
    check(
        "let r=/a/g,f=function(){return null};Object.defineProperty(f,'name',{value:'exec'});r.exec=f;!r.test('a')&&r.lastIndex===0",
    );
    check(
        "let r=/a/g,n=0,e=RegExp.prototype.exec;r.exec=function(s){n++;return e.call(this,s)};r.test('a')&&n===1&&r.lastIndex===1",
    );
    check(
        "let r=/a/g,s=/b/g;r.exec=RegExp.prototype.exec.bind(s);r.test('b')&&r.lastIndex===0&&s.lastIndex===1",
    );
    for value in ["undefined", "true", "1", "'x'", "1n", "Symbol()"] {
        assert!(matches!(
            Realm::default().eval(&format!(
                "let r=/a/;r.exec=function(){{return {value}}};r.test('a')"
            )),
            Err(Error::Exception {
                kind: ExceptionKind::TypeError,
                ..
            })
        ));
    }
}

#[test]
fn native_test_fallback_brand_checks_and_strict_index_writes() {
    for value in ["undefined", "null", "0", "{}"] {
        check(&format!(
            "let r=/a/g;r.exec={value};r.test('a')&&r.lastIndex===1&&!r.test('a')&&r.lastIndex===0"
        ));
    }
    for program in [
        "RegExp.prototype.test.call({exec:RegExp.prototype.exec},'a')",
        "RegExp.prototype.test.call(Object.create(/a/),'a')",
        "let r=/a/g;Object.defineProperty(r,'lastIndex',{writable:false});r.test('a')",
        "let r=/a/y;Object.defineProperty(r,'lastIndex',{writable:false});r.test('b')",
        "let r=/a/g;r.lastIndex=2;Object.defineProperty(r,'lastIndex',{writable:false});r.test('a')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(program),
                Err(Error::Exception {
                    kind: ExceptionKind::TypeError,
                    ..
                })
            ),
            "{program}"
        );
    }
    check(
        "let r=/a/;Object.defineProperty(r,'lastIndex',{writable:false});r.test('a')&&!r.test('b')&&r.lastIndex===0",
    );
    check("let r=/()/dgy;r.lastIndex=1;r.test('a')&&r.lastIndex===1");
    check("let r=/😀/duy;r.lastIndex=1;r.test('😀')&&r.lastIndex===2");
}

#[test]
fn native_test_discards_no_capture_arrays_and_survives_collection() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('('.repeat(1000)+'a'+')'.repeat(1000),'dvg');")
        .unwrap();
    realm.collect(1_000_000).unwrap();
    check("let r=/(?<x>a)/dg;r.test('a')&&r.lastIndex===1");
    assert_eq!(
        realm.eval("let ok=true;for(let i=0;i<20;i++){r.lastIndex=0;ok=ok&&r.test('a')}ok"),
        Ok(Value::Boolean(true))
    );
    let boolean_collection = realm.collect(1_000_000).unwrap();
    assert_eq!(
        realm.eval("for(let i=0;i<20;i++){r.lastIndex=0;r.exec('a')}r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    let exec_collection = realm.collect(1_000_000).unwrap();
    assert!(
        exec_collection.reclaimed > boolean_collection.reclaimed + 20_000,
        "{boolean_collection:?} {exec_collection:?}"
    );
    assert_eq!(
        realm.eval("r.lastIndex=0;r.test('a')&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn native_test_search_quota_and_unsupported_boundary_are_preserved() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30_000),
        ..Limits::default()
    });
    realm
        .eval("let r=/(.|(?:)){999999999999999999999999}?$/usg,m=0;")
        .unwrap();
    realm
        .eval("let s='😀'.repeat(10_000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.test(s)}catch{m=1}finally{m=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("r.lastIndex===1&&m===0"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        Realm::default().eval("/(a)+b/u.test('ab')"),
        Err(Error::Unsupported { .. })
    ));
    check("let r=/(a)+b/ug;r.lastIndex=10;!r.test('ab')&&r.lastIndex===0");
}
