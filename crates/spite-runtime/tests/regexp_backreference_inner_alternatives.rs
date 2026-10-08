//! Inner choices share surrounding terms and preserve capture checkpoints.

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
fn inner_alternative_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a|b)\1", "d", "qbb"),
        (r"(a|ab)\1", "d", "abab"),
        (r"(ab|a)\1", "d", "aa"),
        (r"x(a|ab)\1y", "d", "xababy"),
        (r"(a|ab)\1b", "d", "ababb"),
        (r"((a)|(ab))\1", "d", "qabab"),
        (r"((a)|(ab))\2\3", "d", "abab"),
        (r"(?:(a)|(b))\1\2", "d", "bb"),
        (r"(?:(a)|)\1", "d", "b"),
        (r"(?:|(a))\1", "d", "aa"),
        (r"(q)(?:(a)|(b))\1\2\3", "d", "qbqb"),
        (r"(?:(a\1)|(b\2))\1\2", "d", "bb"),
        (r"((\1a)|b)\1", "d", "aa"),
        (r"^(\w|\W)\1$", "di", "µΜ"),
        (r"([^µ]|[ab])\1", "di", "µΜ"),
        (r"\b(a|\w)\1\b", "d", " bb "),
        (r"((a)|())\1", "d", "b"),
        (r"x(((a)|b))\2y", "d", "xbby"),
        (r"(?:(?<x>a)|(?<x>ab))\k<x>", "d", "abab"),
        (r"^(?:(?<x>a)|(?<x>b)|z)\k<x>$", "d", "z"),
        (r"(q)(?:(?<x>a)|(?<x>b))\1\k<x>", "dgi", "qbqB"),
        (r"(?:(?<x>a)|(?<x>b))\k<x>", "dy", "qbb"),
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
fn common_enclosing_and_inactive_capture_slots_survive_failed_continuations() {
    check(
        r"let a=/(?:(?<x>a)|(?<x>ab))\k<x>/d.exec('qabab');a.index===1&&a[1]===undefined&&a[2]==='ab'&&a.groups.x==='ab'&&a.indices.groups.x===a.indices[2]&&a.indices[2][0]===1&&a.indices[2][1]===3",
    );
    check(
        r"let a=/((a)|(ab))\1/d.exec('qabab');a[0]==='abab'&&a[1]==='ab'&&a[2]===undefined&&a[3]==='ab'&&a.indices[1][0]===1&&a.indices[1][1]===3",
    );
    check(
        r"let a=/(?<q>q)(?:(?<x>a)|(?<x>b))\k<q>\k<x>/d.exec('qbqb');a.groups.q==='q'&&a.groups.x==='b'&&a[2]===undefined&&a.indices.groups.q===a.indices[1]&&a.indices.groups.x===a.indices[3]",
    );
    check(
        r"let a=/((a)|())\1/d.exec('b');a[0]===''&&a[1]===''&&a[2]===undefined&&a[3]===''&&a.indices[3][0]===0&&a.indices[3][1]===0",
    );
    check(
        r"let a=/(?:(?<x>a)|\B(?<x>.)\B)\k<x>/d.exec('\uD800\uD800');a.groups.x.charCodeAt(0)===0xD800&&a.indices.groups.x===a.indices[2]&&a.indices[2][1]===1",
    );
}

#[test]
fn consumers_sticky_positions_and_empty_choices_keep_order_and_original_sources() {
    check(
        r"let r=/(?:(?<x>a)|(?<x>ab))\k<x>/dg,a=[...'abab aa'.matchAll(r)];a.length===2&&a[0].groups.x==='ab'&&a[1].groups.x==='a'&&a[0].indices.groups.x===a[0].indices[2]&&r.lastIndex===0",
    );
    check(
        r"'qabab'.replace(/(?:(?<x>a)|(?<x>ab))\k<x>/,'<$<x>>')==='q<ab>'&&'qabab'.search(/(a|ab)\1/)===1&&'qababz'.split(/(?:(a)|(ab))\1\2/).join(',')==='q,,ab,z'",
    );
    check(
        r"let r=/(?:(?<x>a)|(?<x>b))\k<x>/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qbb');a.groups.x==='b'&&r.lastIndex===3&&r.exec('qbb')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?:(?<x>a)|)\k<x>/dg)];a.length===2&&a[0].index===0&&a[1].index===1&&a[0].groups.x===undefined",
    );
}

#[test]
fn wide_shared_prefixes_and_deep_enclosing_groups_survive_copy_and_collection() {
    let mut realm = Realm::default();
    realm.eval("let r=new RegExp('(a)'.repeat(10000)+'(?:(?:'+'b|'.repeat(10000)+')a)\\\\1','d'),copy=new RegExp(r)").unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a'.repeat(10002));a[0].length===10002&&a.length===10001&&a[10000]==='a'&&a.indices[10000][0]===9999&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[10000]==='a'&&a.indices[10000][1]===10000"),
        Ok(Value::Boolean(true))
    );
    assert_eq!(realm.eval("new RegExp('('.repeat(100000)+'(?<x>a|ab)'+')'.repeat(100000)+'\\\\k<x>','d').exec('abab').groups.x==='ab'"),Ok(Value::Boolean(true)));
}

#[test]
fn explicit_inner_choice_search_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let r=/(?:(?<x>a)|(?<x>b))\k<x>c/g,flag=0,text='ab'.repeat(5000);r.lastIndex=1")
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
fn additional_inner_choices_and_repeated_reference_bodies_keep_unsupported_outcomes() {
    for source in [
        r"/(?:(?:(?:(a|b)(c|d)(?:\1)+){2})|){2,3}/.test('aca')",
        r"/(?:(?:(?:(?:(a|b)|c)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?:(?<x>a)|(?<x>b))(?:\k<x>)+){2})|){2,3}/.test('bb')",
        r"/(?:(?<x>a)|(?<x>b))\k<x>/u.test('bb')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(?:(?<x>a)|(?<x>b))\k<x>/.exec('ab')===null");
}
