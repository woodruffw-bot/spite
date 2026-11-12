//! Fixed capture-free ordinary lookbehind with full input assertion context.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn fixed_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a)b", "d", "ab"),
        (r"(?<!a)b", "d", "ab"),
        (r"(?<=ab)c", "d", "qabc"),
        (r"(?<!ab)c", "d", "ac"),
        (r"(?<=)a", "d", "a"),
        (r"(?<!)a", "d", "a"),
        (r"(?<=^a)b", "d", "qab"),
        (r"(?<=^a)b", "dm", "q\nab"),
        (r"(?<=a$)", "d", "a"),
        (r"(?<=a$)", "dm", "a\nb"),
        (r"(?<=\ba)b", "d", " ab"),
        (r"(?<=a\b)b", "d", "ab"),
        (r"(?<=\Ba)b", "d", "qab"),
        (r"(?<=a\B)b", "d", "ab"),
        (r"(?<=[a-c])b", "d", "cb"),
        (r"(?<=[^a])b", "d", "ab"),
        (r"(?<=.)b", "d", "\nb"),
        (r"(?<=.)b", "ds", "\nb"),
        (r"(?<=µ)Μ", "di", "Μµ"),
        (r"(?<=\w)a", "di", "ſa"),
        (r"(?<=a(?:b))c", "d", "abc"),
        (r"(a)(?<=a)\1", "d", "aa"),
        (r"(?<=a)(?<x>b)\k<x>", "d", "abb"),
        (r"(?:(?<=a)b|c)+d", "d", "abcd"),
        (r"(?:(?<=a)|(?<!b)){2}b", "d", "ab"),
        (r"((?<=a)){2}b\1", "d", "ab"),
        (r"((?<=a))*b\1", "d", "ab"),
        (r"(?<=\uD800)b", "d", "lone surrogate b"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = if text == "lone surrogate b" {
            JsString::from_code_units(vec![0xd800, 0x62])
        } else {
            JsString::from(text)
        };
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({text:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(value) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(rows, "{source:?} flags={flags:?} input={text:?} {value:?}").unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "(?<=a)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a)b\u0022}"
        "(?<!a)b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=ab)c" flags="d" input="qabc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022qabc\u0022,\u0022indices\u0022:[[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=ab)c\u0022}"
        "(?<!ab)c" flags="d" input="ac" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ac\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!ab)c\u0022}"
        "(?<=)a" flags="d" input="a" "{\u0022matches\u0022:[\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=)a\u0022}"
        "(?<!)a" flags="d" input="a" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=^a)b" flags="d" input="qab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=^a)b" flags="dm" input="q\u000aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022q\u005cnab\u0022,\u0022indices\u0022:[[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=^a)b\u0022}"
        "(?<=a$)" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a$)\u0022}"
        "(?<=a$)" flags="dm" input="a\u000ab" "{\u0022matches\u0022:[\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a\u005cnb\u0022,\u0022indices\u0022:[[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a$)\u0022}"
        "(?<=\u005cba)b" flags="d" input=" ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022 ab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cba)b\u0022}"
        "(?<=a\u005cb)b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=\u005cBa)b" flags="d" input="qab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022qab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cBa)b\u0022}"
        "(?<=a\u005cB)b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a\u005c\u005cB)b\u0022}"
        "(?<=[a-c])b" flags="d" input="cb" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022cb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=[a-c])b\u0022}"
        "(?<=[^a])b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=.)b" flags="d" input="\u000ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=.)b" flags="ds" input="\u000ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cnb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=.)b\u0022}"
        "(?<=\u00b5)\u039c" flags="di" input="\u039c\u00b5" "{\u0022matches\u0022:[\u0022\u00b5\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039c\u00b5\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u00b5)\u039c\u0022}"
        "(?<=\u005cw)a" flags="di" input="\u017fa" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=a(?:b))c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?:b))c\u0022}"
        "(a)(?<=a)\u005c1" flags="d" input="aa" "{\u0022matches\u0022:[\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022aa\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a)\u005c\u005c1\u0022}"
        "(?<=a)(?<x>b)\u005ck<x>" flags="d" input="abb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a)(?<x>b)\u005c\u005ck<x>\u0022}"
        "(?:(?<=a)b|c)+d" flags="d" input="abcd" "{\u0022matches\u0022:[\u0022bcd\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abcd\u0022,\u0022indices\u0022:[[1,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a)b|c)+d\u0022}"
        "(?:(?<=a)|(?<!b)){2}b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a)|(?<!b)){2}b\u0022}"
        "((?<=a)){2}b\u005c1" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=a)){2}b\u005c\u005c1\u0022}"
        "((?<=a))*b\u005c1" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=a))*b\u005c\u005c1\u0022}"
        "(?<=\u005cuD800)b" flags="d" input="\ud800b" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cud800b\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cuD800)b\u0022}"
        "#);
}

#[test]
fn fixed_lookbehind_keeps_outside_ranges_named_aliases_and_source_order_retries() {
    assert_true(
        r#"
            let a = /(?<=a)(?<x>b)\k<x>/d.exec('abb');
            a[0] === 'bb' && a.index === 1 && a.groups.x === 'b' && a.indices.groups.x === a.indices[1] &&
                a.indices[1][0] === 1 && a.indices[1][1] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)(?<!a)|(?<x>a)(?<=a))\k<x>/d.exec('aa');
            a[0] === 'aa' && a[1] === undefined && a[2] === 'a' && a.groups.x === 'a' && a.indices[1] ===
                undefined && a.indices.groups.x === a.indices[2]
        "#,
    );
    assert_true(
        r#"
            let a = /((?<=a)){2}b\1/d.exec('ab'),
                b = /((?<=a))*b\1/d.exec('ab');
            a[1] === '' && a.indices[1][0] === 1 && a.indices[1][1] === 1 && b[1] === undefined && b
                .indices[1] === undefined
        "#,
    );
    assert_true(
        r#"
            /(?<=a\b)b/.exec('ab') === null && /(?<=a\B)b/.exec('ab').index === 1 && /(?<=^a)b/m.exec(
                'q\nab').index === 3 && /(?<=a$)/m.exec('a\nb').index === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?=(?<x>a(?<=a)b))\k<x>/d.exec('ab');
            a[0] === 'ab' && a.groups.x === 'ab' && a.indices.groups.x[1] === 2
        "#,
    );
}

#[test]
fn fixed_lookbehind_consumers_global_sticky_empty_advancement_and_callback_arguments() {
    assert_true(
        r#"
            let r = /(?<=a)(?<x>b)/dg,
                a = [...'abqab'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 4 && a[0].groups.x === 'b' && a[1].indices
                .groups.x[0] === 4 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a)b/dy;
            r.lastIndex = 1;
            let a = r.exec('ab');
            a.index === 1 && r.lastIndex === 2 && r.exec('ab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a)/dg,
                a = [...'aa'.matchAll(r)];
            a.length === 2 && a[0][0] === '' && a[0].index === 1 && a[1].index === 2 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'abqab'.replace(/(?<=a)(?<x>b)/g, (m, c, i, s, g) => {
                seen.push(m, c, i, s, g.x);
                return '_'
            });
            s === 'a_qa_' && seen.length === 10 && seen[0] === 'b' && seen[1] === 'b' && seen[2] === 1 &&
                seen[4] === 'b' && seen[7] === 4
        "#,
    );
    assert_true(
        r"'abqab'.replace(/(?<=a)(?<x>b)/g,'<$<x>>')==='a<b>qa<b>'&&'abqab'.search(/(?<=a)b/)===1&&'abqab'.split(/(?<=a)b/).join('|')==='a|qa|'",
    );
}

#[test]
fn explicit_fixed_lookbehind_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=aaaaaaaa)b/g;r.lastIndex=1;let text='a'.repeat(5000)")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn capturing_alternative_repeated_nested_and_unicode_lookbehind_match_reference_results() {
    for source in [
        r"/(?<=(a+))b/.exec('ab')",
        r"/(?<=a|bb)c/.exec('ac')",
        r"/(?<=a+)b/.exec('ab')",
        r"/(?<=a{1,2})b/.exec('ab')",
        r"/(?<=(?=a+))a/.exec('a')",
        r"/(?<=a)b/u.exec('ab')",
        r"/(?<=a)b/v.exec('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
