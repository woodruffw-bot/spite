//! Ordinary numbered references retain input captures and host-abort semantics.

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
fn numbered_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)\1", "d", "qaa"),
        (r"(a)\1", "di", "qAa"),
        (r"(ab)(\1c)\2", "d", "qabab cabc"),
        (r"(ab)(\1c)\2", "d", "qababcabc"),
        (r"\1(a)", "d", "qa"),
        (r"(a\1)", "d", "qa"),
        (r"(a(\1b)\2)\1", "d", "qabbabb"),
        (r"()(a)\1\2", "d", "qaa"),
        (r"(?:q(a))\1", "d", "qaa"),
        (r"\2(a)(b)\1\2", "di", "qABab"),
        (r"()\1", "dg", "q"),
        (r"(a)\1", "dg", "qaa"),
        (r"(a)\1", "dy", "qaa"),
        (r"(a)\1", "dy", "aa"),
        (r"(a)\1", "d", "qaba"),
        (r"(µ)\1", "di", "qΜµ"),
        (r"(ſ)\1", "di", "ſS"),
        (r"(ſ)\1", "di", "ſſ"),
        (r"(\uD800\uDC00)\1", "d", "𐀀𐀀"),
        (r"(a)\x31\1", "d", "a1a"),
        (r"(a)\\1\1", "d", r"a\1a"),
        (r"()()()()()()()()()(a)\10", "d", "aa"),
        (r"(?<x>a)\1", "d", "qaa"),
        (r"\1(?<x>a)", "d", "qa"),
        (r"(?<outer>a(?<x>\1b)\2)\1", "d", "qabbabb"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex}})"
        );
        let Value::String(value) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON");
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn native_results_keep_captures_indices_aliases_and_case_from_the_input() {
    check(
        r"let a=/(?<x>a)\1/di.exec('qAa');a[0]==='Aa'&&a[1]==='A'&&a.groups.x==='A'&&a.indices[0][0]===1&&a.indices[0][1]===3&&a.indices.groups.x===a.indices[1]&&a.indices[1][0]===1&&a.indices[1][1]===2",
    );
    check(
        r"let a=/(a(\1b)\2)\1/d.exec('qabbabb');a[0]==='abbabb'&&a[1]==='abb'&&a[2]==='b'&&a.indices[1][0]===1&&a.indices[1][1]===4&&a.indices[2][0]===2&&a.indices[2][1]===3",
    );
    check(
        r"let a=/\1(a)/d.exec('qa'),b=/(a\1)/d.exec('qa'),c=/()\1/d.exec('q');a[0]==='a'&&a[1]==='a'&&b[0]==='a'&&b[1]==='a'&&c[0]===''&&c[1]===''&&c.indices[1][0]===0&&c.indices[1][1]===0",
    );
    check(
        r"let a=/()()()()()()()()()(a)\10/d.exec('qaa');a.length===11&&a[10]==='a'&&a.indices[10][0]===1&&a.indices[10][1]===2&&a[0]==='aa'",
    );
}

#[test]
fn stateful_exec_and_generic_consumers_use_the_same_reference_program() {
    check(
        r"let r=/(a)\1/dg,a=r.exec('qaa aa'),b=r.exec('qaa aa'),c=r.exec('qaa aa');a.index===1&&b.index===4&&c===null&&r.lastIndex===0",
    );
    check(
        r"let r=/(a)\1/dy;r.lastIndex=1;let a=r.exec('qaa');a.index===1&&r.lastIndex===3&&r.exec('qaa')===null&&r.lastIndex===0",
    );
    check(
        r"let r=/()\1/g,a=r.exec('q');a[0]===''&&r.lastIndex===0&&[...'q'.matchAll(r)].length===2",
    );
    check(
        r"'qaa'.replace(/(?<x>a)\1/,'<$<x>>')==='q<a>'&&'qaa'.split(/(a)\1/).join(',')==='q,a,'&&'qaa'.search(/(a)\1/)===1",
    );
    check(
        r"let seen;'qAa'.replace(/(?<x>a)\1/i,(whole,x,index,input,groups)=>{seen=[whole,x,index,input,groups.x];return 'z'});seen.join(',')==='Aa,A,1,qAa,A'",
    );
    check(
        r"let r=/(?<x>a)\1/dg;Object.defineProperty(r,'source',{get(){throw 7}});let copy=new RegExp(r);copy.source==='(?<x>a)\\1'&&copy.exec('aa').groups.x==='a'&&copy.lastIndex===2&&r.lastIndex===0",
    );
}

#[test]
fn explicit_search_work_aborts_before_last_index_writes_and_language_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(a)\1/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
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
fn unsupported_reference_compositions_remain_distinct_from_match_failures() {
    for source in [
        r"/(?:(?:(?:(a)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(a|b)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:([ab])(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:^(a)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?<x>a)(?:\k<x>)+){2})|){2,3}/.test('aa')",
        r"/(a)\1/u.test('aa')",
    ] {
        assert!(
            crate::common::regexp_oracle::check(&mut Realm::default(), source),
            "{source}"
        );
    }
    check(r"/(a)\1/.exec('ab')===null");
}
