//! Ordinary-mode native RegExpBuiltinExec and its generic consumers.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    let mut realm = Realm::default();
    assert_eq!(realm.eval(source), Ok(Value::Boolean(true)), "{source}");
}

#[test]
fn native_literal_results_and_lastindex_snapshot() {
    let cases = [
        ("", "", "abc", "0"),
        ("", "g", "abc", "2"),
        ("", "dy", "abc", "3"),
        ("", "g", "abc", "4"),
        ("a", "", "baa", "2"),
        ("a", "g", "baa", "2"),
        ("a", "y", "baa", "0"),
        ("a", "y", "baa", "1"),
        ("a", "gy", "baa", "2"),
        ("a", "dg", "baa", "0"),
        ("a", "", "bbb", "7"),
        ("a", "g", "bbb", "2"),
        ("a", "", "baa", "-0"),
        ("a", "g", "baa", "-0"),
        ("a", "g", "baa", "-3"),
        ("a", "g", "baa", "1.9"),
        ("a", "g", "baa", "'2.9'"),
        ("a", "g", "baa", "NaN"),
        ("a", "g", "baa", "Infinity"),
        ("a", "", "baa", "Infinity"),
        ("aba", "dg", "aababa", "0"),
        ("abc", "dims", "xABC", "0"),
        (r"\x61\u0062\0", "d", "xab\0", "0"),
        (r"\f\n\r\t\v", "d", "\u{c}\n\r\t\u{b}", "0"),
        (r"\uD83D\uDCA9", "dg", "x💩y", "0"),
        (r"\uDC00", "dy", "", "0"),
        ("µ", "i", "xΜ", "0"),
        ("σ", "i", "xς", "0"),
        ("s", "i", "xſ", "0"),
        ("k", "i", "xK", "0"),
        ("ß", "i", "SS", "0"),
        ("ß", "i", "ẞ", "0"),
    ];
    let mut rows = String::new();
    for (source, flags, input, initial) in cases {
        let source = JsString::from(source);
        let input = JsString::from(input);
        let script = format!(
            "let r=new RegExp({source:?},'{flags}');r.lastIndex={initial};let m=r.exec({input:?});JSON.stringify({{match:m===null?null:{{text:m[0],length:m.length,index:m.index,input:m.input,groups:m.groups===undefined,hasGroups:Object.hasOwn(m,'groups'),indices:m.indices===undefined?null:{{pair:m.indices[0],groups:m.indices.groups===undefined}}}},lastIndex:r.lastIndex,negativeZero:Object.is(r.lastIndex,-0)}})"
        );
        let mut realm = Realm::default();
        let Value::String(result) = realm.eval(&script).unwrap() else {
            panic!("expected a JSON String: {script}");
        };
        writeln!(
            rows,
            "{source:?} flags={flags:?} input={input:?} lastIndex={initial} -> {}",
            result.to_utf8().unwrap()
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn lastindex_is_converted_after_input_even_without_global_or_sticky_flags() {
    check(
        "let r=new RegExp('a'),t='';r.lastIndex={valueOf(){t+='i';return 9;}};let m=r.exec({toString(){t+='s';return 'ba';}});t==='si' && m.index===1 && typeof r.lastIndex==='object'",
    );
    check(
        "let r=new RegExp('a'),t='';r.lastIndex={valueOf(){t+='i';throw 7;}};let caught=false;try{r.exec({toString(){t+='s';return 'a';}});}catch(e){caught=e===7;}caught && t==='si'",
    );
    for index in ["Symbol()", "1n"] {
        check(&format!(
            "let r=new RegExp('a');r.lastIndex={index};let caught=false;try{{r.exec('a');}}catch(e){{caught=e instanceof TypeError;}}caught"
        ));
    }
    check(
        "let r=new RegExp('a','g'),t='';r.lastIndex={valueOf(){t+='i';r.lastIndex=99;return 1;}};let m=r.exec({toString(){t+='s';return 'baa';}});t==='si' && m.index===1 && r.lastIndex===2",
    );
    check(
        "let r=new RegExp('a','g');let m=r.exec({toString(){r.lastIndex=2;return 'baa';}});m.index===2 && r.lastIndex===3",
    );
}

#[test]
fn lastindex_writes_are_strict_on_success_failure_and_past_end() {
    for (flags, input, index) in [
        ("g", "a", 0),
        ("y", "a", 0),
        ("g", "b", 0),
        ("y", "b", 0),
        ("g", "a", 2),
        ("y", "a", 2),
    ] {
        check(&format!(
            "let r=new RegExp('a','{flags}');r.lastIndex={index};Object.defineProperty(r,'lastIndex',{{writable:false}});let caught=false;try{{r.exec('{input}');}}catch(e){{caught=e instanceof TypeError;}}caught && r.lastIndex==={index}"
        ));
    }
    check(
        "let r=new RegExp('a');r.lastIndex=7;Object.freeze(r);r.exec('ba').index===1 && r.exec('b')===null && r.lastIndex===7",
    );
    check(
        "let r=new RegExp('','g');Object.freeze(r);let caught=false;try{r.exec('');}catch(e){caught=e instanceof TypeError;}caught",
    );
    check(
        "let r=new RegExp('a','g');r.lastIndex={valueOf(){Object.freeze(r);return 0;}};let caught=false;try{r.exec('a');}catch(e){caught=e instanceof TypeError;}caught",
    );
}

#[test]
fn results_and_indices_are_intrinsic_arrays_with_own_data_properties() {
    check(
        "let AP=Array.prototype;for(let k of ['0','index','input','groups','indices'])Object.defineProperty(AP,k,{get(){throw 7;},set(){throw 8;},configurable:true});Array=function(){throw 9;};let m=new RegExp('a','d').exec('ba');let ok=Object.getPrototypeOf(m)===AP && Object.getPrototypeOf(m.indices)===AP && Object.getPrototypeOf(m.indices[0])===AP && m[0]==='a' && m.index===1 && m.input==='ba' && m.groups===undefined && m.indices[0][0]===1 && m.indices[0][1]===2;for(let k of ['0','index','input','groups','indices']){let d=Object.getOwnPropertyDescriptor(m,k);ok=ok && d.writable && d.enumerable && d.configurable && !Object.hasOwn(d,'get');}ok && Reflect.ownKeys(m).join(',')==='0,length,index,input,groups,indices' && Reflect.ownKeys(m.indices).join(',')==='0,length,groups'",
    );
    check(
        "let r=new RegExp('a','d'),a=r.exec('a'),b=r.exec('a');a!==b && a.indices!==b.indices && a.indices[0]!==b.indices[0] && Object.hasOwn(a,'groups') && Object.hasOwn(a.indices,'groups') && a.indices.groups===undefined && !Object.hasOwn(new RegExp('a').exec('a'),'indices')",
    );
}

#[test]
fn matching_uses_original_slots_and_noncallable_exec_falls_back_to_native() {
    check(
        "let r=new RegExp('a','gi');for(let k of ['source','flags','global','sticky','ignoreCase','hasIndices'])Object.defineProperty(r,k,{get(){throw 7;}});let a=r.exec('bAA'),b=r.exec('bAA'),c=r.exec('bAA');a.index===1 && b.index===2 && c===null && r.lastIndex===0",
    );
    for exec in ["undefined", "null", "7", "{}"] {
        check(&format!(
            "let r=new RegExp('a');r.exec={exec};RegExp.prototype.test.call(r,'ba') && RegExp.prototype[Symbol.search].call(r,'ba')===1 && RegExp.prototype[Symbol.match].call(r,'ba')[0]==='a'"
        ));
    }
    check(
        "let r=new RegExp('a');r.exec=function(s){return {0:s,index:3,length:1};};r.test('x') && r[Symbol.search]('x')===3",
    );
}

#[test]
fn ordinary_matching_preserves_surrogate_units_and_exact_utf16_indices() {
    check(
        r"let s='\uD800\uDC00',r=new RegExp('\\uDC00','dg'),m=r.exec(s);m[0]==='\uDC00' && m.index===1 && m.input===s && m.indices[0][0]===1 && m.indices[0][1]===2 && r.lastIndex===2",
    );
    check(
        r"let s='\uD800\uDC00',r=new RegExp('\\uD800','dy');r.lastIndex=0;let m=r.exec(s);m[0]==='\uD800' && m.index===0 && r.lastIndex===1 && r.exec(s)===null && r.lastIndex===0",
    );
    check(
        r"let r=new RegExp('\\uD800\\uDC00','d'),m=r.exec('x\uD800\uDC00');m[0].length===2 && m.index===1 && m.indices[0][0]===1 && m.indices[0][1]===3",
    );
}

#[test]
fn generic_match_search_replace_split_and_matchall_use_native_literal_execution() {
    check(
        "let r=new RegExp('a','g'),m='baac'.match(r);m.join(',')==='a,a' && r.lastIndex===0 && 'bbb'.match(r)===null && r.lastIndex===0",
    );
    check(
        "let r=new RegExp('');r.lastIndex=-0;'ba'.search(r)===0 && Object.is(r.lastIndex,-0) && 'ba'.match(r)[0]===''",
    );
    check(
        "'aba'.replace(new RegExp('a','g'),'[$&]')==='[a]b[a]' && 'aba'.replace(new RegExp('a'),(m,i,s)=>m+i+s)==='a0ababa'",
    );
    check(
        "'a,b,'.split(new RegExp(',')).join('|')==='a|b|' && 'ab'.split(new RegExp('')).join('|')==='a|b' && ''.split(new RegExp('')).length===0",
    );
    check(
        "let r=new RegExp('a','g');r.lastIndex=1;let a=[...'baa'.matchAll(r)];a.length===2 && a[0].index===1 && a[1].index===2 && r.lastIndex===1",
    );
    check(
        "let m=[...'ab'.matchAll('')];m.length===3 && m[0].index===0 && m[1].index===1 && m[2].index===2 && 'ab'.match('')[0]==='' && 'ab'.search('b')===1",
    );
}

#[test]
fn unsupported_patterns_remain_host_failures_after_ordered_coercions() {
    for (source, flags) in [
        ("(a|b)", ""),
        ("a|[b]", ""),
        (".", ""),
        ("[a]", ""),
        ("a+", ""),
        ("a", "u"),
        ("a", "v"),
    ] {
        let mut realm = Realm::default();
        realm.eval(&format!("let r=new RegExp('{source}','{flags}'),t='';r.lastIndex={{valueOf(){{t+='i';return 0;}}}};")).unwrap();
        assert!(matches!(
            realm.eval("r.exec({toString(){t+='s';return 'a';}})"),
            Err(Error::Unsupported { .. })
        ));
        assert_eq!(realm.eval("t"), Ok(Value::String(JsString::from("si"))));
    }
    check("let r=new RegExp('(a|b)','g');r.lastIndex=2;r.exec('a')===null && r.lastIndex===0");
}

#[test]
fn cloned_matchers_and_large_linear_searches_survive_collection() {
    let mut realm = Realm::default();
    check(
        "let r=new RegExp('a','g'),copy=new RegExp(r);r.lastIndex=1;copy.lastIndex=2;r.exec('baa').index===1 && copy.exec('baa').index===2 && r.lastIndex===2 && copy.lastIndex===3",
    );
    realm.eval("let r=new RegExp('a'.repeat(60000)+'b','dg'),copy=new RegExp(r),input='a'.repeat(120000)+'b';").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let m=copy.exec(input);m.index===60000 && m[0].length===60001 && m.indices[0][1]===120001 && copy.lastIndex===120001 && r.lastIndex===0"),Ok(Value::Boolean(true)));
}

#[test]
fn opted_in_work_abort_remains_a_host_failure_and_skips_javascript_cleanup() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(20_000),
        ..Limits::default()
    });
    realm
        .eval("let r=new RegExp('a'),s='b'.repeat(10000),flag=0;")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(s);}catch{flag=1;}finally{flag=2;}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(realm.eval("flag"), Ok(Value::Number(0.0)));
}
