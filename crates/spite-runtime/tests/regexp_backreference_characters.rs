//! Character predicates and references share ordinary input capture semantics.

use spite_core::JsString;
use spite_runtime::{Error, Realm, Value};
use std::fmt::Write;

fn check(source: &str) {
    assert_eq!(
        Realm::default().eval(source),
        Ok(Value::Boolean(true)),
        "{source}"
    );
}

#[test]
fn character_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"([ab])\1", "d", "qbb"),
        (r"([a-z])\1", "di", "qAa"),
        (r"([^a])\1", "d", "qbb"),
        (r"([^µ])\1", "di", "µΣσ"),
        (r"(\w)\1", "d", "q__"),
        (r"(\W)\1", "di", "µΜ"),
        (r"(\d)\1", "d", "q11"),
        (r"(\D)\1", "d", "11aa"),
        (r"(\s)\1", "d", "q  "),
        (r"(\S)\1", "d", "  aa"),
        (r"(.)\1", "d", "\n\n"),
        (r"(.)\1", "ds", "\n\n"),
        (r"([^])\1", "d", "\n\n"),
        (r"([])\1", "d", "aa"),
        (r"\1(\w)", "d", "qa"),
        (r"(\w\1)", "d", "qa"),
        (r"([ab])(\1.)\2", "d", "qaacac"),
        (r"(?<x>[a-z])\k<x>", "di", "qAa"),
        (r"(?<x>.)\k<x>", "ds", "\n\n"),
        (r"(?<x>\w)\k<x>", "dg", "qaa"),
        (r"(?<x>\w)\k<x>", "dy", "qaa"),
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
fn classes_dot_case_and_surrogates_retain_input_captures_and_indices_aliases() {
    check(
        r"let a=/(?<x>[a-z])\k<x>/di.exec('qAa');a[0]==='Aa'&&a.groups.x==='A'&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2",
    );
    check(
        r"let a=/(?<x>.)\k<x>/ds.exec('\n\n');a[0]==='\n\n'&&a.groups.x==='\n'&&a.indices.groups.x===a.indices[1]&&a.indices[0][1]===2",
    );
    check(
        r"let a=/(.)\1/d.exec('\uD800\uD800');a[0].length===2&&a[1].length===1&&a[1].charCodeAt(0)===0xD800&&a.indices[1][1]===1",
    );
    check(
        r"/(\w)\1/i.exec('ſſ')===null&&/(\W)\1/i.exec('ſſ')[0]==='ſſ'&&/([^µ])\1/i.exec('µΜ')===null",
    );
    check(
        r"let a=/([ab])(\1.)\2/d.exec('qaacac');a[0]==='aacac'&&a[1]==='a'&&a[2]==='ac'&&a.indices[2][0]===2&&a.indices[2][1]===4",
    );
}

#[test]
fn generic_consumers_and_stateful_exec_use_character_reference_matches() {
    check(
        r"let r=/(?<x>\w)\k<x>/dgi,a=[...'aa bB cc'.matchAll(r)];a.length===3&&a[1].groups.x==='b'&&a[1].indices.groups.x===a[1].indices[1]&&r.lastIndex===0",
    );
    check(
        r"'qaa'.replace(/(?<x>\w)\k<x>/,'<$<x>>')==='q<a>'&&'qaa'.split(/(\w)\1/).join(',')==='q,a,'&&'qaa'.search(/(\w)\1/)===1",
    );
    check(
        r"let r=/(?<x>[ab])\k<x>/dy;r.lastIndex=1;let a=r.exec('qbb');a.index===1&&r.lastIndex===3&&r.exec('qbb')===null&&r.lastIndex===0",
    );
}

#[test]
fn deep_shared_predicates_and_captures_survive_copy_and_collection_without_quotas() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('(\\\\w)'.repeat(10000)+'\\\\1','d'),copy=new RegExp(r)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a'.repeat(10001));a.length===10001&&a[10000]==='a'&&a.indices[10000][0]===9999&&a.indices[10000][1]===10000&&a[0].length===10001&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[10000]==='a'&&a.indices[10000][1]===10000"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("new RegExp('('.repeat(100000)+'(?<x>\\\\w)'+')'.repeat(100000)+'\\\\k<x>','d').exec('aa').groups.x==='a'"),Ok(Value::Boolean(true)));
}

#[test]
fn remaining_reference_compositions_keep_explicit_unsupported_outcomes() {
    for source in [
        r"/([ab])\1+/.test('aa')",
        r"/(a|b)\1/.test('aa')",
        r"/^([ab])\1+/.test('aa')",
        r"/(?<x>[ab])\k<x>/u.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/([ab])\1/.exec('ab')===null&&/([])\1/.exec('aa')===null");
}
