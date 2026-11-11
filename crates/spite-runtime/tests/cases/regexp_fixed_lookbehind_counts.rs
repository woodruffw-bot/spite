//! Exact counts of single-unit character and class terms inside lookbehind.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn fixed_count_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a{1})b", "d", "ab"),
        (r"(?<=a{2})b", "d", "aab"),
        (r"(?<!a{2})b", "d", "ab"),
        (r"(?<=a{0})b", "d", "b"),
        (r"(?<!a{0})b", "d", "b"),
        (r"(?<=a{2,2})b", "d", "aab"),
        (r"(?<=a{2}?)b", "d", "aab"),
        (r"(?<!a{2}?)b", "d", "aab"),
        (r"(?<=a{1}b{1})c", "d", "abc"),
        (r"(?<=a[a-z]{2})d", "d", "abcd"),
        (r"(?<!a[a-z]{2})d", "d", "abcd"),
        (r"(?<=.{2})b", "ds", "\n\nb"),
        (r"(?<=.{2})b", "d", "\n\nb"),
        (r"(?<=\w{2})b", "d", "aab"),
        (r"(?<=\s{1})b", "d", " b"),
        (r"(?<=[^a]{2})b", "d", "ccb"),
        (r"(?<=\b[a-z]{2})c", "d", " abc"),
        (r"(?<=a{2}\B)b", "d", "aab"),
        (r"(?<=^a{2})b", "dm", "q\naab"),
        (r"(?<=a{2}$)", "dm", "aa\nb"),
        (r"(?<=µ{2})Μ", "di", "ΜµΜ"),
        (r"(?<=(?:a){2})b", "d", "aab"),
        (r"(?<=a{0}b)c", "d", "bc"),
        (r"(?<=a{1})(?<x>b)\k<x>", "d", "abb"),
        (r"(?:(?<=a{2})b|c)+d", "d", "aabcd"),
        (r"(?:(?<=a{2})|(?<!b{2})){2}b", "d", "aab"),
        (r"((?<=a{2})){2}b\1", "d", "aab"),
        (r"(?<=[\uD800]{2})b", "d", "lone surrogates b"),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let text = if text == "lone surrogates b" {
            JsString::from_code_units(vec![0xd800, 0xd800, 0x62])
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
        "(?<=a{1})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{1})b\u0022}"
        "(?<=a{2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{2})b\u0022}"
        "(?<!a{2})b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<!a{2})b\u0022}"
        "(?<=a{0})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{0})b\u0022}"
        "(?<!a{0})b" flags="d" input="b" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=a{2,2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{2,2})b\u0022}"
        "(?<=a{2}?)b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{2}?)b\u0022}"
        "(?<!a{2}?)b" flags="d" input="aab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=a{1}b{1})c" flags="d" input="abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022abc\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{1}b{1})c\u0022}"
        "(?<=a[a-z]{2})d" flags="d" input="abcd" "{\u0022matches\u0022:[\u0022d\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022abcd\u0022,\u0022indices\u0022:[[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a[a-z]{2})d\u0022}"
        "(?<!a[a-z]{2})d" flags="d" input="abcd" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=.{2})b" flags="ds" input="\u000a\u000ab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u005cn\u005cnb\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=.{2})b\u0022}"
        "(?<=.{2})b" flags="d" input="\u000a\u000ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=\u005cw{2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cw{2})b\u0022}"
        "(?<=\u005cs{1})b" flags="d" input=" b" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022 b\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cs{1})b\u0022}"
        "(?<=[^a]{2})b" flags="d" input="ccb" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022ccb\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=[^a]{2})b\u0022}"
        "(?<=\u005cb[a-z]{2})c" flags="d" input=" abc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022 abc\u0022,\u0022indices\u0022:[[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u005c\u005cb[a-z]{2})c\u0022}"
        "(?<=a{2}\u005cB)b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{2}\u005c\u005cB)b\u0022}"
        "(?<=^a{2})b" flags="dm" input="q\u000aaab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022q\u005cnaab\u0022,\u0022indices\u0022:[[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=^a{2})b\u0022}"
        "(?<=a{2}$)" flags="dm" input="aa\u000ab" "{\u0022matches\u0022:[\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aa\u005cnb\u0022,\u0022indices\u0022:[[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{2}$)\u0022}"
        "(?<=\u00b5{2})\u039c" flags="di" input="\u039c\u00b5\u039c" "{\u0022matches\u0022:[\u0022\u039c\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u039c\u00b5\u039c\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=\u00b5{2})\u039c\u0022}"
        "(?<=(?:a){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=(?:a){2})b\u0022}"
        "(?<=a{0}b)c" flags="d" input="bc" "{\u0022matches\u0022:[\u0022c\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022bc\u0022,\u0022indices\u0022:[[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{0}b)c\u0022}"
        "(?<=a{1})(?<x>b)\u005ck<x>" flags="d" input="abb" "{\u0022matches\u0022:[\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a{1})(?<x>b)\u005c\u005ck<x>\u0022}"
        "(?:(?<=a{2})b|c)+d" flags="d" input="aabcd" "{\u0022matches\u0022:[\u0022bcd\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabcd\u0022,\u0022indices\u0022:[[2,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a{2})b|c)+d\u0022}"
        "(?:(?<=a{2})|(?<!b{2})){2}b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?:(?<=a{2})|(?<!b{2})){2}b\u0022}"
        "((?<=a{2})){2}b\u005c1" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[2,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022((?<=a{2})){2}b\u005c\u005c1\u0022}"
        "(?<=[\u005cuD800]{2})b" flags="d" input="\ud800\ud800b" "{\u0022matches\u0022:[\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022\u005cud800\u005cud800b\u0022,\u0022indices\u0022:[[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=[\u005c\u005cuD800]{2})b\u0022}"
        "#);
}

#[test]
fn fixed_count_lookbehind_ranges_named_aliases_zero_counts_and_outer_retries_are_exact() {
    assert_true(
        r#"
            let a = /(?<=a{2})(?<x>b)\k<x>/d.exec('aabb');
            a[0] === 'bb' && a.index === 2 && a.groups.x === 'b' && a.indices.groups.x === a.indices[1] &&
                a.indices[1][0] === 2 && a.indices[1][1] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?:(?<x>a)(?<!a{1})|(?<x>a)(?<=a{1}))\k<x>/d.exec('aa');
            a[0] === 'aa' && a[1] === undefined && a[2] === 'a' && a.indices[1] === undefined && a.indices
                .groups.x === a.indices[2]
        "#,
    );
    assert_true(
        r#"
            let a = /((?<=a{2})){2}b\1/d.exec('aab'),
                b = /((?<=a{2}))*b\1/d.exec('aab');
            a[1] === '' && a.indices[1][0] === 2 && a.indices[1][1] === 2 && b[1] === undefined && b
                .indices[1] === undefined
        "#,
    );
    assert_true(
        r#"
            /(?<=a{0}b)c/.exec('bc').index === 1 && /(?<!a{0})b/.exec('b') === null && /(?<=^a{2})b/m.exec(
                'q\naab').index === 4 && /(?<=a{2}$)/m.exec('aa\nb').index === 2
        "#,
    );
    assert_true(
        r#"
            let a = /(?=(?<x>a{2}(?<=a{2})b))\k<x>/d.exec('aab');
            a[0] === 'aab' && a.groups.x === 'aab' && a.indices.groups.x[1] === 3
        "#,
    );
}

#[test]
fn fixed_count_lookbehind_consumers_global_sticky_and_empty_advancement_preserve_positions() {
    assert_true(
        r#"
            let r = /(?<=a{2})(?<x>b)/dg,
                a = [...'aabqaab'.matchAll(r)];
            a.length === 2 && a[0].index === 2 && a[1].index === 6 && a[1].groups.x === 'b' && a[1].indices
                .groups.x[0] === 6 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a{2})b/dy;
            r.lastIndex = 2;
            let a = r.exec('aab');
            a.index === 2 && r.lastIndex === 3 && r.exec('aab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a{2})/dg,
                a = [...'aaa'.matchAll(r)];
            a.length === 2 && a[0][0] === '' && a[0].index === 2 && a[1].index === 3 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'aabqaab'.replace(/(?<=a{2})(?<x>b)/g, (m, c, i, s, g) => {
                seen.push(m, c, i, s, g.x);
                return '_'
            });
            s === 'aa_qaa_' && seen.length === 10 && seen[2] === 2 && seen[4] === 'b' && seen[7] === 6
        "#,
    );
    assert_true(
        r"'aabqaab'.replace(/(?<=a{2})(?<x>b)/g,'<$<x>>')==='aa<b>qaa<b>'&&'aabqaab'.search(/(?<=a{2})b/)===2&&'aabqaab'.split(/(?<=a{2})b/).join('|')==='aa|qaa|'",
    );
}

#[test]
fn explicit_fixed_count_lookbehind_unit_work_aborts_before_last_index_and_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let marker=0,r=/(?<=a{8})b/g;r.lastIndex=1;let text='a'.repeat(5000)")
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
fn variable_counts_capture_effects_multiple_terms_and_nested_lookbehind_match_reference_results() {
    for source in [
        r"/(?<=a{1,2})b/.exec('ab')",
        r"/(?<=(a{1,2}))b/.exec('aab')",
        r"/(?<=(?:ab){1,2})c/.exec('ababc')",
        r"/(?<=(?:(?=a+)){2})a/.exec('a')",
        r"/(?<=a{2}(?=a+))b/.exec('aab')",
        r"/(?<=a{2})b/u.exec('aab')",
        r"/(?<=a{2})b/v.exec('aab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
