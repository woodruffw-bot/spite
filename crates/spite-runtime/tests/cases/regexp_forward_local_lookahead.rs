//! Completed same-unit references in fixed forward assertions inside lookbehind.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn forward_local_lookahead_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=a(?=((b)\2){2}))b", "d", "abbbb"),
        (r"(?<=a(?=((b)\2){2}?))b", "d", "abbbb"),
        (r"(?<=a(?=(([ab])\2){2}))b", "d", "abbaa"),
        (r"(?<=a(?=((b)(\2)\3){2}))b", "d", "abbbbbb"),
        (r"(?<=a(?=((b)\2(\1)){2}))b", "d", "abbbb"),
        (r"(?<=a(?=((b)\2(\2)){2}))b", "d", "abbbbbb"),
        (r"(?<=((a)\1){2}(?=((b)\4){2}))b", "d", "aabbbb"),
        (r"(?<=a(?=((b)\2){0}))b", "d", "ab"),
        (r"(?<=a(?!((b)\2){2}q))b", "d", "abbbbx"),
        (r"(?<=a(?=((b)\2\B){2}))b", "d", "abbbbb"),
        (r"(?<=a(?=((b)\2){1}))b", "d", "abb"),
        (r"(?=((b)\2){2})b", "d", "bbbb"),
        (r"(a)(?<=a(?=((b)\3){2}))b", "d", "abbbb"),
        (r"(?<=((a)\2){2}(?=(((a)\5){2})))b", "d", "aaaab"),
        (r"(?<=a(?=((b)\2){2}))b", "di", "aBBBb"),
        (r"(?<=a(?=(?<y>(?<x>b)\k<x>){2}))b", "d", "abbbb"),
        (
            r"(?<=a(?=(?<y>(?<x>b)(?<z>\k<x>)\k<z>){2}))b",
            "d",
            "abbbbbb",
        ),
    ] {
        let source = JsString::from(source);
        let flags = JsString::from(flags);
        let input = JsString::from(text);
        let program = format!(
            "let r=new RegExp({source:?},{flags:?}),a=r.exec({input:?});JSON.stringify(a===null?{{match:null,lastIndex:r.lastIndex}}:{{matches:[...a],index:a.index,input:a.input,groups:a.groups,indices:a.indices,indicesGroups:a.indices.groups,lastIndex:r.lastIndex,source:r.source}})"
        );
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(
            rows,
            "{source:?} flags={flags:?} input={input:?} {result:?}"
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows, @r#"
        "(?<=a(?=((b)\u005c2){2}))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbb\u0022,\u0022indices\u0022:[[1,2],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2){2}))b\u0022}"
        "(?<=a(?=((b)\u005c2){2}?))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbb\u0022,\u0022indices\u0022:[[1,2],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2){2}?))b\u0022}"
        "(?<=a(?=(([ab])\u005c2){2}))b" flags="d" input="abbaa" "{\u0022matches\u0022:[\u0022b\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbaa\u0022,\u0022indices\u0022:[[1,2],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(([ab])\u005c\u005c2){2}))b\u0022}"
        "(?<=a(?=((b)(\u005c2)\u005c3){2}))b" flags="d" input="abbbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bbb\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbbb\u0022,\u0022indices\u0022:[[1,2],[4,7],[4,5],[5,6]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)(\u005c\u005c2)\u005c\u005c3){2}))b\u0022}"
        "(?<=a(?=((b)\u005c2(\u005c1)){2}))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbb\u0022,\u0022indices\u0022:[[1,2],[3,5],[3,4],[5,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2(\u005c\u005c1)){2}))b\u0022}"
        "(?<=a(?=((b)\u005c2(\u005c2)){2}))b" flags="d" input="abbbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bbb\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbbb\u0022,\u0022indices\u0022:[[1,2],[4,7],[4,5],[6,7]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2(\u005c\u005c2)){2}))b\u0022}"
        "(?<=((a)\u005c1){2}(?=((b)\u005c4){2}))b" flags="d" input="aabbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aabbbb\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1],[4,6],[4,5]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c1){2}(?=((b)\u005c\u005c4){2}))b\u0022}"
        "(?<=a(?=((b)\u005c2){0}))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2){0}))b\u0022}"
        "(?<=a(?!((b)\u005c2){2}q))b" flags="d" input="abbbbx" "{\u0022matches\u0022:[\u0022b\u0022,null,null],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbx\u0022,\u0022indices\u0022:[[1,2],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?!((b)\u005c\u005c2){2}q))b\u0022}"
        "(?<=a(?=((b)\u005c2\u005cB){2}))b" flags="d" input="abbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbb\u0022,\u0022indices\u0022:[[1,2],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2\u005c\u005cB){2}))b\u0022}"
        "(?<=a(?=((b)\u005c2){1}))b" flags="d" input="abb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abb\u0022,\u0022indices\u0022:[[1,2],[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2){1}))b\u0022}"
        "(?=((b)\u005c2){2})b" flags="d" input="bbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022bbbb\u0022,\u0022indices\u0022:[[0,1],[2,4],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?=((b)\u005c\u005c2){2})b\u0022}"
        "(a)(?<=a(?=((b)\u005c3){2}))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022abbbb\u0022,\u0022indices\u0022:[[0,2],[0,1],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?=((b)\u005c\u005c3){2}))b\u0022}"
        "(?<=((a)\u005c2){2}(?=(((a)\u005c5){2})))b" flags="d" input="aaaab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(?<=a(?=((b)\u005c2){2}))b" flags="di" input="aBBBb" "{\u0022matches\u0022:[\u0022B\u0022,\u0022Bb\u0022,\u0022B\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022aBBBb\u0022,\u0022indices\u0022:[[1,2],[3,5],[3,4]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=((b)\u005c\u005c2){2}))b\u0022}"
        "(?<=a(?=(?<y>(?<x>b)\u005ck<x>){2}))b" flags="d" input="abbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bb\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbb\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022bb\u0022,\u0022x\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[3,5],[3,4]],\u0022indicesGroups\u0022:{\u0022y\u0022:[3,5],\u0022x\u0022:[3,4]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<y>(?<x>b)\u005c\u005ck<x>){2}))b\u0022}"
        "(?<=a(?=(?<y>(?<x>b)(?<z>\u005ck<x>)\u005ck<z>){2}))b" flags="d" input="abbbbbb" "{\u0022matches\u0022:[\u0022b\u0022,\u0022bbb\u0022,\u0022b\u0022,\u0022b\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022abbbbbb\u0022,\u0022groups\u0022:{\u0022y\u0022:\u0022bbb\u0022,\u0022x\u0022:\u0022b\u0022,\u0022z\u0022:\u0022b\u0022},\u0022indices\u0022:[[1,2],[4,7],[4,5],[5,6]],\u0022indicesGroups\u0022:{\u0022y\u0022:[4,7],\u0022x\u0022:[4,5],\u0022z\u0022:[5,6]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=a(?=(?<y>(?<x>b)(?<z>\u005c\u005ck<x>)\u005c\u005ck<z>){2}))b\u0022}"
        "#);
}

#[test]
fn nested_forward_dependencies_names_both_directions_and_rollback_keep_exact_ranges() {
    assert_true(
        r#"
            let a = /(?<=a(?=(?<y>(?<x>b)\k<x>){2}))b/d.exec('abbbb');
            a.index === 1 && a.groups.y === 'bb' && a.groups.x === 'b' && a.indices.groups.x === a.indices[
                2] && a.indices[1][0] === 3 && a.indices[2][0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?=(?<y>(?<x>b)(?<z>\k<x>)\k<z>){2}))b/d.exec('abbbbbb');
            a.groups.y === 'bbb' && a.indices.groups.x[0] === 4 && a.indices.groups.z[0] === 5 && a.groups
                .z === 'b'
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(?<left>(a)\k<left>){2}(?=(?<right>(b)\4){2}))b/d.exec('aabbbb');
            a.groups.left === 'a' && a.groups.right === 'bb' && a.indices.groups.left[0] === 0 && a.indices
                .groups.right[0] === 4 && a.indices[4][0] === 4
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?!((b)\2){2}q))b/d.exec('abbbbx');
            a.index === 1 && a[1] === undefined && a[2] === undefined && a.indices[1] === undefined
        "#,
    );
    assert_true(r"/(?<=((a)\2){2}(?=(((a)\5){2})))b/.exec('aaaab')===null");
    assert_true(
        r"let a=/(?<=a(?=(([ab])\2){2}))b/d.exec('abbaa');a[1]==='aa'&&a[2]==='a'&&a.indices[2][0]===3",
    );
    assert_true(
        r#"
            let a = /(?<=a(?=(([\uD800])\2){2}))[\uD800]/d.exec('a\uD800\uD800\uD800\uD800\uDC00');
            a.index === 1 && a.indices[1][0] === 3 && a.indices[2][0] === 3
        "#,
    );
}

#[test]
fn consumers_global_sticky_empty_advance_callbacks_and_zero_skips_preserve_future_ranges() {
    assert_true(
        r#"
            let r = /(?<=a(?=((b)\2){2}))b/dy;
            r.lastIndex = 1;
            let a = r.exec('abbbb');
            a.index === 1 && r.lastIndex === 2 && r.exec('abbbb') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=a(?=((b)\2){2}))b/dg,
                a = [...'abbbb abbbb'.matchAll(r)];
            a.length === 2 && a[1].index === 7 && a[1].indices[2][0] === 9 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"let a=/(?<=a(?=((b)\2){0}))b/d.exec('ab');a[1]===undefined&&a[2]===undefined&&a.indices[2]===undefined",
    );
    assert_true(
        r"let a=[...'abbbb'.matchAll(/(?<=a(?=((b)\2){2}))/dg)];a.length===1&&a[0].index===1&&a[0].indices[2][0]===3",
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'abbbb abbbb'.replace(/(?<=a(?=((b)\2){2}))b/g, (m, x, y, i) => {
                seen.push(x, y, i);
                return '_'
            });
            s === 'a_bbb a_bbb' && seen.join('|') === 'bb|b|1|bb|b|7'
        "#,
    );
    assert_true(
        r"'abbbb'.search(/(?<=a(?=((b)\2){2}))b/)===1&&'abbbb'.split(/(?<=a(?=((b)\2){2}))b/).join('|')==='a|bb|b|bbb'",
    );
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(?<=a(?=((b)\2){2}))b/g;r.lastIndex=1;let text='b'.repeat(5000)+'abbbb'").unwrap();
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
fn variable_counts_choices_bare_capture_dependencies_and_unicode_remain_pending() {
    for source in [
        r"/(?<=a(?=((b)\2){1,2}))b/.exec('abbbb')",
        r"/(?<=a(?=((b)\2){2}|b))b/.exec('abbbb')",
        r"/(?<=a(?=(b{1,2})\1))b/.exec('abbbb')",
        r"/(?<=a(?=((b)\2){2}))b/u.exec('abbbb')",
        r"/(?<=a(?=((b)\2){2}))b/v.exec('abbbb')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
