//! Assertions inspect absolute input positions around ordinary references.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn asserted_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"^(a)\1$", "d", "aa"),
        (r"^(a)\1$", "d", "aa\n"),
        (r"^(a)\1$", "dm", "q\naa\nz"),
        (r"^(a)\1$", "dim", "q\nAa\nz"),
        (r"^(a)\1$", "dm", "\raa\r\n"),
        (r"^(a)\1$", "dm", "\u{2028}aa\u{2029}"),
        (r"\b(\w)\1\b", "d", " aa "),
        (r"\b(\w)\1\b", "d", "qaa"),
        (r"\B(.)\1\B", "di", "µΜ"),
        (r"\B(.)\1\B", "di", "aa"),
        (r"(\w)\b\1", "d", "aa"),
        (r"(\w)\B\1", "d", "aa"),
        (r"(a($))\1", "dm", "aa\n"),
        (r"(\1^a)\1", "d", "aa"),
        (r"^()\1$", "d", ""),
        (r"\b()\1\b", "d", " a"),
        (r"\B()\1\B", "d", " a"),
        (r"(\b)\1\w", "d", " a"),
        (r"^([\s\S])\1$", "d", "\n\n"),
        (r"^(.)\1$", "ds", "\n\n"),
        (r"\b\B(\w)\1", "d", "aa"),
        (r"^^(\w)\1$$", "d", "aa"),
        (r"^(?<x>\w)\k<x>$", "dmi", "q\nAa\nz"),
        (r"\b(?<x>)\k<x>\b", "dg", " a"),
        (r"\b(?<x>\w)\k<x>\b", "dy", " aa "),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(value) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn input_line_and_word_context_preserve_captures_and_sticky_search_positions() {
    check(
        r"let r=/^(?<x>\w)\k<x>$/dmiy;r.lastIndex=2;let a=r.exec('q\nAa\nz');a.index===2&&a.groups.x==='A'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===2&&a.indices[1][1]===3&&r.lastIndex===4&&r.exec('q\nAa\nz')===null&&r.lastIndex===0",
    );
    check(r"let r=/\b(\w)\1\b/dy;r.lastIndex=1;r.exec('qaa')===null&&r.lastIndex===0");
    check(
        r"let r=/\b(\w)\1\b/dy;r.lastIndex=1;let a=r.exec(' aa ');a.index===1&&a[1]==='a'&&r.lastIndex===3",
    );
    check(
        r"/^(a)\1$/.exec('aa\n')===null&&/^(a)\1$/m.exec('aa\n')[0]==='aa'&&/\b(\w)\1\b/i.exec('ſſ')===null&&/\B(.)\1\B/i.exec('µΜ')[1]==='µ'",
    );
    check(
        r"let a=/\B(.)\1\B/d.exec('\uD800\uD800');a[0].length===2&&a[1].charCodeAt(0)===0xD800&&a.indices[1][0]===0&&a.indices[1][1]===1",
    );
    check(
        r"let a=/(\b)\1\w/d.exec(' a');a[1]===''&&a.index===1&&a.indices[1][0]===1&&a.indices[1][1]===1&&/(\1^a)\1/.exec('aa')[1]==='a'",
    );
    check(r"/([\b])\1/.exec('\b\b')[0]==='\b\b'&&/(\^)(\$)\1\2/.exec('^$^$')[0]==='^$^$'");
}

#[test]
fn generic_consumers_use_asserted_references_and_advance_empty_matches() {
    check(
        r"let r=/^(?<x>\w)\k<x>$/dgmi,a=[...'aa\nbB\ncc'.matchAll(r)];a.length===3&&a[1].index===3&&a[1].groups.x==='b'&&a[1].indices.groups.x===a[1].indices[1]&&r.lastIndex===0",
    );
    check(
        r"'q\naa\nz'.replace(/^(?<x>\w)\k<x>$/m,'<$<x>>')==='q\n<a>\nz'&&'q\naa\nz'.search(/^(\w)\1$/m)===2&&'q aa z'.split(/\b(\w)\1\b/).join(',')==='q ,a, z'",
    );
    check(
        r"let r=/\b(?<x>)\k<x>\b/dg,a=[...' a '.matchAll(r)];a.length===2&&a[0].index===1&&a[1].index===2&&a[0].groups.x===''&&a[0].indices.groups.x===a[0].indices[1]&&r.lastIndex===0",
    );
}

#[test]
fn deep_assertion_programs_survive_copy_and_collection_without_default_quotas() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('^'.repeat(100000)+'(?<x>a)\\\\k<x>'+'$'.repeat(100000),'d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aa');a[0]==='aa'&&a.groups.x==='a'&&a.indices.groups.x===a.indices[1]&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a.groups.x==='a'&&a.indices[1][1]===1"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("new RegExp('('.repeat(100000)+'\\\\b(?<x>\\\\w)\\\\k<x>\\\\b'+')'.repeat(100000),'d').exec(' aa ').groups.x==='a'"),Ok(Value::Boolean(true)));
}

#[test]
fn explicit_assertion_search_work_aborts_before_last_index_and_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/\B(?<x>a)\k<x>\B/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{flag=1}finally{flag=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("flag===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn pending_reference_compositions_keep_unsupported_outcomes() {
    for source in [
        r"/(?:(?:(?:^([ab])(?:\1)+$){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:^(a|b)(?:\1)+$){2})|){2,3}/.test('aa')",
        r"/(?<=(a+))\1/.test('a')",
        r"/\b(?<x>a)\k<x>\b/u.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/^([ab])\1$/.exec('ab')===null&&/\b\B(\w)\1/.exec('aa')===null");
}
