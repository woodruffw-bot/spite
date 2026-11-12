//! Flat nested fixed capture-free lookbehind and absolute boundary positions.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn nested_fixed_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(?<=a))b", "d", "ab"),
        (r"(?<!(?<=a))b", "d", "ab"),
        (r"(?<=(?<!a))b", "d", "qb"),
        (r"(?<!(?<!a))b", "d", "ab"),
        (r"(?<=a(?<=a))b", "d", "ab"),
        (r"(?<=a(?<!a))b", "d", "ab"),
        (r"(?<!a(?<=a))b", "d", "qb"),
        (r"(?<=a(?<=b))c", "d", "abc"),
        (r"(?<=(?<=a)b)c", "d", "abc"),
        (r"(?<=a(?<=a)b)c", "d", "abc"),
        (r"(?<=a(?<!b)b)c", "d", "abc"),
        (r"(?<=a(?<=a{1})b{1})c", "d", "abc"),
        (r"(?<=a{2}(?<=a{2}))b", "d", "aab"),
        (r"(?<=a{2}(?<!a{2}))b", "d", "aab"),
        (r"(?<=(?<=^a)b)c", "dm", "q\nabc"),
        (r"(?<=(?<=\ba)b)c", "d", " abc"),
        (r"(?<=c(?<=\w))\w{3}", "d", "ab cdef"),
        (r"(?<=\B)(?<=c(?<=\w))\w{3}", "d", "ab cdef"),
        (r"(?<=.(?<=.))b", "ds", "\nb"),
        (r"(?<=µ(?<=µ))Μ", "di", "Μµ"),
        (r"(?<=a(?<=(?<=a)))b", "d", "ab"),
        (r"(?<=a(?<!(?<!a)))b", "d", "ab"),
        (r"(?<=a(?<=a$))", "d", "a"),
        (r"(?<=^a(?<=a))b", "dm", "q\nab"),
        (r"(?<=a(?<=a))(?<x>b)\k<x>", "d", "abb"),
        (r"(?:(?<=a(?<=a))b|c)+d", "d", "abccd"),
        (r"(?:(?<=a(?<=a))|(?<!b)){2}b", "d", "ab"),
        (r"(?<=\uD800(?<=\uD800))b", "d", "lone surrogate b"),
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
        "(?<=(?<=a))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<=a))b\u0022}"
        "(?<!(?<=a))b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(?<!a))b" flags="d" input="qb" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<!a))b\u0022}"
        "(?<!(?<!a))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!(?<!a))b\u0022}"
        "(?<=a(?<=a))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<=a))b\u0022}"
        "(?<=a(?<!a))b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<!a(?<=a))b" flags="d" input="qb" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022qb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!a(?<=a))b\u0022}"
        "(?<=a(?<=b))c" flags="d" input="abc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(?<=a)b)c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<=a)b)c\u0022}"
        "(?<=a(?<=a)b)c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<=a)b)c\u0022}"
        "(?<=a(?<!b)b)c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<!b)b)c\u0022}"
        "(?<=a(?<=a{1})b{1})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<=a{1})b{1})c\u0022}"
        "(?<=a{2}(?<=a{2}))b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{2}(?<=a{2}))b\u0022}"
        "(?<=a{2}(?<!a{2}))b" flags="d" input="aab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=(?<=^a)b)c" flags="dm" input="q\u000aabc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022q\u005cnabc\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<=^a)b)c\u0022}"
        "(?<=(?<=\u005cba)b)c" flags="d" input=" abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022 abc\u0022,\u0022indices\u0022:[[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?<=\u005c\u005cba)b)c\u0022}"
        "(?<=c(?<=\u005cw))\u005cw{3}" flags="d" input="ab cdef" "{\u0022matches\u0022:[\u0022def\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ab cdef\u0022,\u0022indices\u0022:[[4,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=c(?<=\u005c\u005cw))\u005c\u005cw{3}\u0022}"
        "(?<=\u005cB)(?<=c(?<=\u005cw))\u005cw{3}" flags="d" input="ab cdef" "{\u0022matches\u0022:[\u0022def\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022ab cdef\u0022,\u0022indices\u0022:[[4,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cB)(?<=c(?<=\u005c\u005cw))\u005c\u005cw{3}\u0022}"
        "(?<=.(?<=.))b" flags="ds" input="\u000ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cnb\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=.(?<=.))b\u0022}"
        "(?<=\u00b5(?<=\u00b5))\u039c" flags="di" input="\u039c\u00b5" "{\u0022matches\u0022:[\u0022\u00b5\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u039c\u00b5\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u00b5(?<=\u00b5))\u039c\u0022}"
        "(?<=a(?<=(?<=a)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<=(?<=a)))b\u0022}"
        "(?<=a(?<!(?<!a)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<!(?<!a)))b\u0022}"
        "(?<=a(?<=a$))" flags="d" input="a" "{\u0022matches\u0022:[\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022a\u0022,\u0022indices\u0022:[[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<=a$))\u0022}"
        "(?<=^a(?<=a))b" flags="dm" input="q\u000aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022q\u005cnab\u0022,\u0022indices\u0022:[[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=^a(?<=a))b\u0022}"
        "(?<=a(?<=a))(?<x>b)\u005ck<x>" flags="d" input="abb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?<=a))(?<x>b)\u005c\u005ck<x>\u0022}"
        "(?:(?<=a(?<=a))b|c)+d" flags="d" input="abccd" "{\u0022matches\u0022:[\u0022bccd\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abccd\u0022,\u0022indices\u0022:[[1,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a(?<=a))b|c)+d\u0022}"
        "(?:(?<=a(?<=a))|(?<!b)){2}b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a(?<=a))|(?<!b)){2}b\u0022}"
        "(?<=\u005cuD800(?<=\u005cuD800))b" flags="d" input="\ud800b" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022\u005cud800b\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cuD800(?<=\u005c\u005cuD800))b\u0022}"
        "#);
}

#[test]
fn nested_fixed_lookbehind_child_positions_negation_and_outside_aliases_are_exact() {
    assert_true(
        r#"
            let a = /(?<=a(?<=a))(?<x>b)\k<x>/d.exec('abb');
            a[0] === 'bb' && a.index === 1 && a.groups.x === 'b' && a.indices.groups.x === a.indices[1] &&
                a.indices[1][0] === 1 && a.indices[1][1] === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)(?<!a(?<=a))|(?<x>a)(?<=a(?<=a)))\k<x>/d.exec('aa');
            a[0] === 'aa' && a[1] === undefined && a[2] === 'a' && a.indices[1] === undefined && a.indices
                .groups.x === a.indices[2]
        "#,
    );
    assert_true(
        r#"
            let a = /((?<=a(?<=a))){2}b\1/d.exec('ab'),
                b = /((?<=a(?<=a)))*b\1/d.exec('ab');
            a[1] === '' && a.indices[1][0] === 1 && a.indices[1][1] === 1 && b[1] === undefined && b
                .indices[1] === undefined
        "#,
    );
    assert_true(
        r#"
            /(?<=(?<=a))b/.test('ab') && !/(?<!(?<=a))b/.test('ab') && /(?<=(?<!a))b/.test('qb') &&
                /(?<!(?<!a))b/.test('ab') && !/(?<=a(?<!a))b/.test('ab')
        "#,
    );
    assert_true(
        r"/(?<=(?<=^a)b)c/m.exec('q\nabc').index===4&&/(?<=(?<=^a)b)c/.exec('q\nabc')===null&&/(?<=\B)(?<=c(?<=\w))\w{3}/.exec('ab cdef')[0]==='def'",
    );
    assert_true(
        r#"
            let a = /(?=(?<x>a(?<=a(?<=a))b))\k<x>/d.exec('ab');
            a[0] === 'ab' && a.groups.x === 'ab' && a.indices.groups.x[1] === 2
        "#,
    );
}

#[test]
fn nested_fixed_lookbehind_consumers_global_sticky_and_empty_matches_preserve_positions() {
    assert_true(
        r#"
            let r = /(?<=a(?<=a))(?<x>b)/dg,
                a = [...'abqab'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 4 && a[1].groups.x === 'b' && a[1].indices
                .groups.x[0] === 4 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a(?<=a))b/dy;
            r.lastIndex = 1;
            let a = r.exec('ab');
            a.index === 1 && r.lastIndex === 2 && r.exec('ab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a(?<=a))/dg,
                a = [...'aa'.matchAll(r)];
            a.length === 2 && a[0][0] === '' && a[0].index === 1 && a[1].index === 2 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'abqab'.replace(/(?<=a(?<=a))(?<x>b)/g, (m, c, i, s, g) => {
                seen.push(m, c, i, s, g.x);
                return '_'
            });
            s === 'a_qa_' && seen.length === 10 && seen[2] === 1 && seen[4] === 'b' && seen[7] === 4
        "#,
    );
    assert_true(
        r"'abqab'.replace(/(?<=a(?<=a))(?<x>b)/g,'<$<x>>')==='a<b>qa<b>'&&'abqab'.search(/(?<=a(?<=a))b/)===1&&'abqab'.split(/(?<=a(?<=a))b/).join('|')==='a|qa|'",
    );
}

#[test]
fn explicit_nested_fixed_lookbehind_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a(?<=a))b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn nested_captures_and_unicode_match_reference() {
    for source in [
        r"/(?<=(?<=(a+)))b/.exec('ab')",
        r"/(?<=(?<=a|bb))c/.exec('ac')",
        r"/(?<=a(?<=a+))b/.exec('ab')",
        r"/(?<=a(?=a+))b/.exec('ab')",
        r"/(?<=(?<=\1))(a)/.exec('a')",
        r"/(?<=a(?<=a))b/u.exec('ab')",
        r"/(?<=a(?<=a))b/v.exec('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
