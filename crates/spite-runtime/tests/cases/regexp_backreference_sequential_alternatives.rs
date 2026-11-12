//! Sequential choices restore capture checkpoints when later terms fail.

use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

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
    insta::assert_snapshot!(rows, @r#"
        "(a|ab)(b|)\u005c1\u005c2" flags="d" input="qabab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabab\u0022,\u0022indices\u0022:[[1,5],[1,2],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)(b|)\u005c\u005c1\u005c\u005c2\u0022}"
        "(a|ab)(c|)\u005c1\u005c2" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,2],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)(c|)\u005c\u005c1\u005c\u005c2\u0022}"
        "(ab|a)(c|)\u005c1\u005c2" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab|a)(c|)\u005c\u005c1\u005c\u005c2\u0022}"
        "((a)|(ab))((c)|())\u005c2\u005c3\u005c5\u005c6" flags="d" input="qabab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022,null,\u0022ab\u0022,\u0022\u0022,null,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qabab\u0022,\u0022indices\u0022:[[1,5],[1,3],null,[1,3],[3,3],null,[3,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a)|(ab))((c)|())\u005c\u005c2\u005c\u005c3\u005c\u005c5\u005c\u005c6\u0022}"
        "(q)(a|ab)(c|)\u005c1\u005c2\u005c3" flags="d" input="qababqabab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?:(a)|(b))\u005c1\u005c2(?:(a)|(b))\u005c3\u005c4" flags="d" input="aabb" "{\u0022matches\u0022:[\u0022aabb\u0022,\u0022a\u0022,null,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabb\u0022,\u0022indices\u0022:[[0,4],[0,1],null,null,[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|(b))\u005c\u005c1\u005c\u005c2(?:(a)|(b))\u005c\u005c3\u005c\u005c4\u0022}"
        "((a|ab)(b|))\u005c1" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022ab\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((a|ab)(b|))\u005c\u005c1\u0022}"
        "(?:(a)|)(?:|(b))\u005c1\u005c2" flags="d" input="bb" "{\u0022matches\u0022:[\u0022\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,0],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a)|)(?:|(b))\u005c\u005c1\u005c\u005c2\u0022}"
        "\u005c2(a|b)\u005c1(c|)\u005c2" flags="d" input="aacc" "{\u0022matches\u0022:[\u0022aacc\u0022,\u0022a\u0022,\u0022c\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aacc\u0022,\u0022indices\u0022:[[0,4],[0,1],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022\u005c\u005c2(a|b)\u005c\u005c1(c|)\u005c\u005c2\u0022}"
        "(a|b)((\u005c1a)|b)\u005c2" flags="d" input="aaaaa" "{\u0022matches\u0022:[\u0022aaaaa\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022aa\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aaaaa\u0022,\u0022indices\u0022:[[0,5],[0,1],[1,3],[1,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|b)((\u005c\u005c1a)|b)\u005c\u005c2\u0022}"
        "^(\u005cw|\u005cW)(\u005cd|\u005cD)\u005c1\u005c2$" flags="di" input="\u00b51\u039c1" "{\u0022matches\u0022:[\u0022\u00b51\u039c1\u0022,\u0022\u00b5\u0022,\u00221\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u00b51\u039c1\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022^(\u005c\u005cw|\u005c\u005cW)(\u005c\u005cd|\u005c\u005cD)\u005c\u005c1\u005c\u005c2$\u0022}"
        "([^\u00b5]|[ab])(\u00b5|.)\u005c1\u005c2" flags="di" input="a\u00b5A\u039c" "{\u0022matches\u0022:[\u0022a\u00b5A\u039c\u0022,\u0022a\u0022,\u0022\u00b5\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u00b5A\u039c\u0022,\u0022indices\u0022:[[0,4],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022([^\u00b5]|[ab])(\u00b5|.)\u005c\u005c1\u005c\u005c2\u0022}"
        "\u005cb(a|\u005cw)(b|\u005cw)\u005c1\u005c2\u005cb" flags="d" input=" ab ab " "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "()(|a)(|b)\u005c1\u005c2\u005c3" flags="d" input="aabb" "{\u0022matches\u0022:[\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aabb\u0022,\u0022indices\u0022:[[0,0],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(|a)(|b)\u005c\u005c1\u005c\u005c2\u005c\u005c3\u0022}"
        "x(((a)|b))(?:(c)|(d))\u005c2\u005c4\u005c5y" flags="d" input="xbdbdy" "{\u0022matches\u0022:[\u0022xbdbdy\u0022,\u0022b\u0022,\u0022b\u0022,null,null,\u0022d\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022xbdbdy\u0022,\u0022indices\u0022:[[0,6],[1,2],[1,2],null,null,[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022x(((a)|b))(?:(c)|(d))\u005c\u005c2\u005c\u005c4\u005c\u005c5y\u0022}"
        "(a|ab)(b|)\u005c1\u005c2|(b)\u005c3" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,null,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,2],null,null,[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)(b|)\u005c\u005c1\u005c\u005c2|(b)\u005c\u005c3\u0022}"
        "(a|ab)(b|)\u005c1\u005c2|" flags="dg" input="q" "{\u0022matches\u0022:[\u0022\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022q\u0022,\u0022indices\u0022:[[0,0],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a|ab)(b|)\u005c\u005c1\u005c\u005c2|\u0022}"
        "(|(a))((b)|)\u005c2\u005c4" flags="d" input="bb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022\u0022,null,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bb\u0022,\u0022indices\u0022:[[0,2],[0,0],null,[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(|(a))((b)|)\u005c\u005c2\u005c\u005c4\u0022}"
        "(?:(a\u005c1)|(b\u005c2))(?:(a\u005c3)|(b\u005c4))\u005c1\u005c2\u005c3\u005c4" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,\u0022a\u0022,null,null,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022indices\u0022:[[0,4],[0,1],null,null,[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(a\u005c\u005c1)|(b\u005c\u005c2))(?:(a\u005c\u005c3)|(b\u005c\u005c4))\u005c\u005c1\u005c\u005c2\u005c\u005c3\u005c\u005c4\u0022}"
        "(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|(?<y>))\u005ck<x>\u005ck<y>" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,null,\u0022ab\u0022,null,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022ab\u0022,\u0022y\u0022:\u0022\u0022},\u0022indices\u0022:[[0,4],null,[0,2],null,[2,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,2],\u0022y\u0022:[2,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|(?<y>))\u005c\u005ck<x>\u005c\u005ck<y>\u0022}"
        "(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|)\u005ck<x>\u005ck<y>" flags="d" input="abab" "{\u0022matches\u0022:[\u0022abab\u0022,null,\u0022ab\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022abab\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022ab\u0022},\u0022indices\u0022:[[0,4],null,[0,2],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|)\u005c\u005ck<x>\u005c\u005ck<y>\u0022}"
        "(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\u005ck<x>\u005ck<y>" flags="dgi" input="baBA" "{\u0022matches\u0022:[\u0022baBA\u0022,null,\u0022b\u0022,\u0022a\u0022,null],\u0022index\u0022:0,\u0022input\u0022:\u0022baBA\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022,\u0022y\u0022:\u0022a\u0022},\u0022indices\u0022:[[0,4],null,[0,1],[1,2],null],\u0022indicesGroups\u0022:{\u0022x\u0022:[0,1],\u0022y\u0022:[1,2]},\u0022lastIndex\u0022:4,\u0022source\u0022:\u0022(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\u005c\u005ck<x>\u005c\u005ck<y>\u0022}"
        "(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\u005ck<x>\u005ck<y>" flags="dy" input="qabab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "((.|\u005cw)(.|\u005cd))\u005c1" flags="ds" input="\u000a1\u000a1" "{\u0022matches\u0022:[\u0022\u005cn1\u005cn1\u0022,\u0022\u005cn1\u0022,\u0022\u005cn\u0022,\u00221\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022\u005cn1\u005cn1\u0022,\u0022indices\u0022:[[0,4],[0,2],[0,1],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((.|\u005c\u005cw)(.|\u005c\u005cd))\u005c\u005c1\u0022}"
        "#);
}

#[test]
fn later_choice_failures_restore_prior_and_common_capture_slots() {
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|(?<y>))\k<x>\k<y>/d.exec('qabab');
            a.index === 1 && a.groups.x === 'ab' && a.groups.y === '' && a[1] === undefined && a[3] ===
                undefined && a.indices.groups.x === a.indices[2] && a.indices.groups.y === a.indices[4] &&
                a.indices[4][0] === 3 && a.indices[4][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<q>q)(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|)\k<q>\k<x>\k<y>/d.exec('qabqab');
            a[0] === 'qabqab' && a.groups.q === 'q' && a.groups.x === 'ab' && a.groups.y === undefined && a
                .indices.groups.q === a.indices[1] && a.indices.groups.x === a.indices[3]
        "#,
    );
    assert_true(
        r#"
            let a = /((a|ab)(c|))\1/d.exec('abab');
            a[0] === 'abab' && a[1] === 'ab' && a[2] === 'ab' && a[3] === '' && a.indices[1][0] === 0 && a
                .indices[1][1] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /((a)|(ab))((c)|())\2\3\5\6/d.exec('qabab');
            a[0] === 'abab' && a[1] === 'ab' && a[2] === undefined && a[3] === 'ab' && a[4] === '' && a[
                5] === undefined && a[6] === ''
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>.))(?:(?<y>b)|(?<y>.))\k<x>\k<y>/d.exec('\uD800\uDC00\uD800\uDC00');
            a.groups.x.charCodeAt(0) === 0xD800 && a.groups.y.charCodeAt(0) === 0xDC00 && a.indices.groups
                .x === a.indices[2] && a.indices.groups.y === a.indices[4]
        "#,
    );
}

#[test]
fn consumers_and_empty_choices_keep_order_and_original_sources() {
    assert_true(
        r#"
            let r = /(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|(?<y>))\k<x>\k<y>/dg,
                a = [...'abab acac'.matchAll(r)];
            a.length === 2 && a[0].groups.x === 'ab' && a[0].groups.y === '' && a[1].groups.x === 'a' && a[
                1].groups.y === 'c' && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            'qabab'.replace(/(?:(?<x>a)|(?<x>ab))(?:(?<y>c)|)\k<x>\k<y>/, '<$<x>:$<y>>') === 'q<ab:>' &&
                'qabab'.search(/(a|ab)(c|)\1\2/) === 1 && 'qababz'.split(/(a|ab)(c|)\1\2/).join(',') ===
                'q,ab,,z'
        "#,
    );
    assert_true(
        r#"
            let r = /(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\k<x>\k<y>/dy;
            r.lastIndex = 1;
            let copy = new RegExp(r),
                a = r.exec('qabab');
            a.groups.x === 'a' && a.groups.y === 'b' && r.lastIndex === 5 && r.exec('qabab') === null && r
                .lastIndex === 0 && copy.source === r.source && copy.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = [...'q'.matchAll(/(?:(?<x>a)|)(?:(?<y>b)|)\k<x>\k<y>/dg)];
            a.length === 2 && a[0].index === 0 && a[1].index === 1 && a[0].groups.x === undefined && a[0]
                .groups.y === undefined
        "#,
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
fn nested_choices_and_quantified_reference_bodies_match_reference_results() {
    for source in [
        r"/(?:(?:(?:(?:(a|b)|c)(?:\1)+){2})|){2,3}/.test('aa')",
        r"/(?:(?:(?:(?:a|(b|c))(?:\1)+){2})|){2,3}/.test('bb')",
        r"/(?:(?:(?:((a|b)(c|d)|e)(?:\1)+){2})|){2,3}/.test('acac')",
        r"/(?:(?:(?:(?:(?<x>a)|(?<x>b))(?:(?<y>a)|(?<y>b))\k<x>(?:\k<y>)+){2})|){2,3}/.test('abab')",
        r"/(a|b)(c|d)\1/u.test('aca')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    assert_true(r"/(a|b)(c|d)\1\2/.exec('acba')===null");
}
