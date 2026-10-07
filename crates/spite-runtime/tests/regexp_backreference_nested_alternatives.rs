//! Nested ordinary alternatives use flat control flow and capture checkpoints.

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
fn nested_alternative_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?:(a|ab)|b)\1", "d", "qabab"),
        (r"(?:a|(b|bc))\1", "d", "bcbc"),
        (r"((a|ab)(c|)|e)\1", "d", "abab"),
        (r"((a|ab)(c|)|e)\1", "d", "ee"),
        (r"((a)|(ab|abc))\1\2\3", "d", "abcabcabc"),
        (r"(?:(?:(a)|(ab))|(b))\1\2\3", "d", "qabab"),
        (r"(q)(?:(a|ab)|b)(c|)\1\2\3", "d", "qabqab"),
        (r"(?:(?:(a)|)|(?:(b)|))\1\2", "d", "bb"),
        (r"((a|)|(b|))\1", "d", "b"),
        (r"(?:(a\1|b)|c)\1", "d", "aa"),
        (r"((\1a|b)|c)\1", "d", "aa"),
        (r"(?:(\w|\W)|(\d|\D))\1\2", "di", "µΜ"),
        (r"([^µ]|(?:[ab]|µ))\1", "di", "µΜ"),
        (r"\b((a|\w)|b)\1\b", "d", " bb "),
        (r"((a|ab)|c)(b|(?:c|))\1\3", "d", "abab"),
        (r"(?:(a|ab)|b)\1|(c|cd)\2", "d", "cdcd"),
        (r"(?:(a|ab)|b)\1|", "dg", "q"),
        (r"((?:(a)|b)|())\1\2\3", "d", "b"),
        (r"x(((a)|(?:b|c)))\2y", "d", "xccy"),
        (r"(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>", "d", "abab"),
        (r"(?:(?:(?<x>a)|(?<x>b))|z)\k<x>", "d", "z"),
        (r"(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>", "dgi", "cC"),
        (r"(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>", "dy", "qcc"),
        (r"((.|(?:\w|\W))(.|\d))\1", "ds", "\n1\n1"),
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
fn nested_failure_restores_prior_names_and_enclosing_capture_slots() {
    check(
        r"let a=/(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>/d.exec('qabab');a.index===1&&a.groups.x==='ab'&&a[1]===undefined&&a[3]===undefined&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===3",
    );
    check(
        r"let a=/(?<q>q)(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<q>\k<x>/d.exec('qabqab');a[0]==='qabqab'&&a.groups.q==='q'&&a.groups.x==='ab'&&a.indices.groups.q===a.indices[1]&&a.indices.groups.x===a.indices[3]",
    );
    check(
        r"let a=/((a|ab)(c|)|e)\1/d.exec('ee');a[0]==='ee'&&a[1]==='e'&&a[2]===undefined&&a[3]===undefined&&a.indices[2]===undefined&&a.indices[1][1]===1",
    );
    check(
        r"let a=/(?:(?:(?<x>a)|(?<x>b))|z)\k<x>/d.exec('z');a[0]==='z'&&a.groups.x===undefined&&a.indices.groups.x===undefined",
    );
    check(
        r"let a=/(?:(a)|(?:(?<x>b)|(?<x>.)))\k<x>/d.exec('\uD800\uD800');a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[3]&&a.indices[1]===undefined",
    );
}

#[test]
fn nested_consumers_and_empty_choices_keep_source_order_and_original_text() {
    check(
        r"let r=/(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>/dg,a=[...'abab bb'.matchAll(r)];a.length===2&&a[0].groups.x==='ab'&&a[1].groups.x==='b'&&a[0].indices.groups.x===a[0].indices[2]&&r.lastIndex===0",
    );
    check(
        r"'qabab'.replace(/(?:(?:(?<x>a)|(?<x>ab))|(?<x>b))\k<x>/,'<$<x>>')==='q<ab>'&&'qabab'.search(/(?:(a|ab)|b)\1/)===1&&'qababz'.split(/(?:(?:(a)|(ab))|(b))\1\2\3/).join(',')==='q,,ab,,z'",
    );
    check(
        r"let r=/(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qcc');a.groups.x==='c'&&r.lastIndex===3&&r.exec('qcc')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?:(?:(?<x>a)|)|(?<x>b))\k<x>/dg)];a.length===2&&a[0].index===0&&a[1].index===1&&a[0].groups.x===undefined",
    );
}

#[test]
fn deep_nested_choices_and_named_scopes_survive_copy_collection_and_failure() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('('.repeat(100000)+'a'+'|b)'.repeat(100000)+'\\\\1','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('aa');a[0]==='aa'&&a.length===100001&&a[100000]==='a'&&a.indices[100000][0]===0&&copy.source===r.source&&copy.exec('x')===null"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[100000]==='a'&&a.indices[100000][1]===1"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("let named=new RegExp('(?:'.repeat(100000)+'(?<x>a)'+'|b)'.repeat(100000)+'\\\\k<x>','d'),b=named.exec('aa'),c=named.exec('b');b.groups.x==='a'&&b.indices.groups.x===b.indices[1]&&c[0]==='b'&&c.groups.x===undefined"),Ok(Value::Boolean(true)));
}

#[test]
fn explicit_nested_choice_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let r=/(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>d/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1").unwrap();
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
fn quantified_reference_bodies_and_lookaround_remain_unsupported() {
    for source in [
        r"/(?:(a|b)|c)(?:\1)+/.test('aa')",
        r"/(?:a|(b|c))(?:\1)+/.test('bb')",
        r"/((a|b)(c|d)|e)(?:\1)+/.test('acac')",
        r"/(?:(?:(?<x>a)|(?<x>b))|(?<x>c))\k<x>/u.test('cc')",
        r"/(?=(a|b))\1/.test('aa')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(?:(a|b)|c)\1/.exec('ab')===null");
}
