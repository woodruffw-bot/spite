//! Sequential choices restore capture checkpoints when later terms fail.

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
fn sequential_alternative_reference_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a|ab)(b|)\1\2", "d", "qabab"),
        (r"(a|ab)(c|)\1\2", "d", "abab"),
        (r"(ab|a)(c|)\1\2", "d", "aa"),
        (r"((a)|(ab))((c)|())\2\3\5\6", "d", "qabab"),
        (r"(q)(a|ab)(c|)\1\2\3", "d", "qababqabab"),
        (r"(?:(a)|(b))\1\2(?:(a)|(b))\3\4", "d", "aabb"),
        (r"((a|ab)(b|))\1", "d", "abab"),
        (r"(?:(a)|)(?:|(b))\1\2", "d", "bb"),
        (r"\2(a|b)\1(c|)\2", "d", "aacc"),
        (r"(a|b)((\1a)|b)\2", "d", "aaaaa"),
        (r"^(\w|\W)(\d|\D)\1\2$", "di", "µ1Μ1"),
        (r"([^µ]|[ab])(µ|.)\1\2", "di", "aµAΜ"),
        (r"\b(a|\w)(b|\w)\1\2\b", "d", " ab ab "),
        (r"()(|a)(|b)\1\2\3", "d", "aabb"),
        (r"x(((a)|b))(?:(c)|(d))\2\4\5y", "d", "xbdbdy"),
        (r"(a|ab)(b|)\1\2|(b)\3", "d", "bb"),
        (r"(a|ab)(b|)\1\2|", "dg", "q"),
        (r"(|(a))((b)|)\2\4", "d", "bb"),
        (r"(?:(a\1)|(b\2))(?:(a\3)|(b\4))\1\2\3\4", "d", "abab"),
        (
            r"(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|(?<y>))\k<x>\k<y>",
            "d",
            "abab",
        ),
        (r"(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|)\k<x>\k<y>", "d", "abab"),
        (
            r"(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\k<x>\k<y>",
            "dgi",
            "baBA",
        ),
        (
            r"(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\k<x>\k<y>",
            "dy",
            "qabab",
        ),
        (r"((.|\w)(.|\d))\1", "ds", "\n1\n1"),
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
fn later_choice_failures_restore_prior_and_common_capture_slots() {
    check(
        r"let a=/(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|(?<y>))\k<x>\k<y>/d.exec('qabab');a.index===1&&a.groups.x==='ab'&&a.groups.y===''&&a[1]===undefined&&a[3]===undefined&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[4]&&a.indices[4][0]===3&&a.indices[4][1]===3",
    );
    check(
        r"let a=/(?<q>q)(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|)\k<q>\k<x>\k<y>/d.exec('qabqab');a[0]==='qabqab'&&a.groups.q==='q'&&a.groups.x==='ab'&&a.groups.y===undefined&&a.indices.groups.q===a.indices[1]&&a.indices.groups.x===a.indices[3]",
    );
    check(
        r"let a=/((a|ab)(c|))\1/d.exec('abab');a[0]==='abab'&&a[1]==='ab'&&a[2]==='ab'&&a[3]===''&&a.indices[1][0]===0&&a.indices[1][1]===2",
    );
    check(
        r"let a=/((a)|(ab))((c)|())\2\3\5\6/d.exec('qabab');a[0]==='abab'&&a[1]==='ab'&&a[2]===undefined&&a[3]==='ab'&&a[4]===''&&a[5]===undefined&&a[6]===''",
    );
    check(
        r"let a=/(?:(?<x>a)|(?<x>.))(?:(?<y>b)|(?<y>.))\k<x>\k<y>/d.exec('\uD800\uDC00\uD800\uDC00');a.groups.x.charCodeAt(0)===0xD800&&a.groups.y.charCodeAt(0)===0xDC00&&a.indices.groups.x===a.indices[2]&&a.indices.groups.y===a.indices[4]",
    );
}

#[test]
fn consumers_and_empty_choices_keep_order_and_original_sources() {
    check(
        r"let r=/(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|(?<y>))\k<x>\k<y>/dg,a=[...'abab acac'.matchAll(r)];a.length===2&&a[0].groups.x==='ab'&&a[0].groups.y===''&&a[1].groups.x==='a'&&a[1].groups.y==='c'&&r.lastIndex===0",
    );
    check(
        r"'qabab'.replace(/(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|)\k<x>\k<y>/,'<$<x>:$<y>>')==='q<ab:>'&&'qabab'.search(/(a|ab)(c|)\1\2/)===1&&'qababz'.split(/(a|ab)(c|)\1\2/).join(',')==='q,ab,,z'",
    );
    check(
        r"let r=/(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\k<x>\k<y>/dy;r.lastIndex=1;let copy=new RegExp(r),a=r.exec('qabab');a.groups.x==='a'&&a.groups.y==='b'&&r.lastIndex===5&&r.exec('qabab')===null&&r.lastIndex===0&&copy.source===r.source&&copy.lastIndex===0",
    );
    check(
        r"let a=[...'q'.matchAll(/(?:(?<x>a)|)(?:(?<y>b)|)\k<x>\k<y>/dg)];a.length===2&&a[0].index===0&&a[1].index===1&&a[0].groups.x===undefined&&a[0].groups.y===undefined",
    );
}

#[test]
fn many_sequential_choices_survive_copy_and_collection_without_default_quotas() {
    let mut realm = Realm::default();
    realm
        .eval("let r=new RegExp('(a|b)'.repeat(100000)+'\\\\1','d'),copy=new RegExp(r)")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval("let a=copy.exec('a'.repeat(100001));a[0].length===100001&&a.length===100001&&a[100000]==='a'&&a.indices[100000][0]===99999&&copy.source===r.source"),Ok(Value::Boolean(true)));
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval("a[100000]==='a'&&a.indices[100000][1]===100000"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn explicit_sequential_choice_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let r=new RegExp('(?:a|b)'.repeat(100)+'(a)\\1c','g'),flag=0,text='a'.repeat(200);r.lastIndex=1").unwrap();
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
fn nested_choices_and_quantified_reference_bodies_remain_unsupported() {
    for source in [
        r"/(?:(?:(?:(?:(a|b)|c)(?:\1)+){2})|)*/.test('aa')",
        r"/(?:(?:(?:(?:a|(b|c))(?:\1)+){2})|)*/.test('bb')",
        r"/(?:(?:(?:((a|b)(c|d)|e)(?:\1)+){2})|)*/.test('acac')",
        r"/(?:(?:(?:(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\k<x>(?:\k<y>)+){2})|)*/.test('abab')",
        r"/(a|b)(c|d)\1/u.test('aca')",
    ] {
        assert!(
            matches!(
                Realm::default().eval(source),
                Err(Error::Unsupported { .. })
            ),
            "{source}"
        );
    }
    check(r"/(a|b)(c|d)\1\2/.exec('acba')===null");
}
