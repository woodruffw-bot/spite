//! Stable imported captures in nested ordinary backward assertions.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn nested_outside_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=a(?<=\1))b", "d", "ab"),
        (r"(a)(?<=a(?<=(\1)))b", "d", "ab"),
        (r"(a)(?<=a(?<=((b)\1){2}))c", "d", "babac"),
        (r"(a)(?<=a(?<=((b)\1){2}?))c", "d", "babac"),
        (r"(a)(?<=\1(?<=(b\1){2}))c", "d", "babac"),
        (r"(a)(?<=a(?<!((b)\1){2}q))c", "d", "babac"),
        (r"(a)(?<=a(?<=((b)\1){0}))c", "d", "ac"),
        (r"(a)(?<=a(?=(?<=((b)\1){2})c))c", "d", "babac"),
        (r"(a)(?<=a(?<=(\3(b)\1){2}))c", "d", "bbabbac"),
        (r"(a)(?<=a(?<=(?<=\1)\1))b", "d", "ab"),
        (r"(a)(?<=(a\1){2}(?<=\1))b", "d", "aaaab"),
        (r"(a)(?<=a{2}(?<=\1))b", "d", "aab"),
        (r"(a)(?<=(?<=\1))b", "d", "ab"),
        (r"(?<x>a)(?<=a(?<=(?<y>(?<z>b)\k<x>){2}))c", "d", "babac"),
        (
            r"(?<x>a)(?<=a(?<=(?<y>\k<z>(?<z>b)\k<x>){2}))c",
            "d",
            "bbabbac",
        ),
        (r"(a)(?<=a(?<=((b)\1){2}))c", "di", "bAbAc"),
        (r"()(a)(?<=a(?<=(\1\2)))b", "d", "ab"),
        (r"(a)?(?<=a(?<=(\1)))b", "d", "ab"),
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
        "(a)(?<=a(?<=\u005c1))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<=\u005c\u005c1))b\u0022}"
        "(a)(?<=a(?<=(\u005c1)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<=(\u005c\u005c1)))b\u0022}"
        "(a)(?<=a(?<=((b)\u005c1){2}))c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<=((b)\u005c\u005c1){2}))c\u0022}"
        "(a)(?<=a(?<=((b)\u005c1){2}?))c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<=((b)\u005c\u005c1){2}?))c\u0022}"
        "(a)(?<=\u005c1(?<=(b\u005c1){2}))c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=\u005c\u005c1(?<=(b\u005c\u005c1){2}))c\u0022}"
        "(a)(?<=a(?<!((b)\u005c1){2}q))c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,null,null],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<!((b)\u005c\u005c1){2}q))c\u0022}"
        "(a)(?<=a(?<=((b)\u005c1){0}))c" flags="d" input="ac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ac\u0022,\u0022indices\u0022:[[0,2],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<=((b)\u005c\u005c1){0}))c\u0022}"
        "(a)(?<=a(?=(?<=((b)\u005c1){2})c))c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?=(?<=((b)\u005c\u005c1){2})c))c\u0022}"
        "(a)(?<=a(?<=(\u005c3(b)\u005c1){2}))c" flags="d" input="bbabbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bba\u0022,\u0022b\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022bbabbac\u0022,\u0022indices\u0022:[[5,7],[5,6],[0,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<=(\u005c\u005c3(b)\u005c\u005c1){2}))c\u0022}"
        "(a)(?<=a(?<=(?<=\u005c1)\u005c1))b" flags="d" input="ab" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<=(a\u005c1){2}(?<=\u005c1))b" flags="d" input="aaaab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022aa\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaaab\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(a\u005c\u005c1){2}(?<=\u005c\u005c1))b\u0022}"
        "(a)(?<=a{2}(?<=\u005c1))b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[1,3],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a{2}(?<=\u005c\u005c1))b\u0022}"
        "(a)(?<=(?<=\u005c1))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(?<=\u005c\u005c1))b\u0022}"
        "(?<x>a)(?<=a(?<=(?<y>(?<z>b)\u005ck<x>){2}))c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022ba\u0022,\u0022z\u0022:\u0022b\u0022},\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[3,4],\u0022y\u0022:[0,2],\u0022z\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<=a(?<=(?<y>(?<z>b)\u005c\u005ck<x>){2}))c\u0022}"
        "(?<x>a)(?<=a(?<=(?<y>\u005ck<z>(?<z>b)\u005ck<x>){2}))c" flags="d" input="bbabbac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022bba\u0022,\u0022b\u0022],\u0022index\u0022:5,\u0022input\u0022:\u0022bbabbac\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022bba\u0022,\u0022z\u0022:\u0022b\u0022},\u0022indices\u0022:[[5,7],[5,6],[0,3],[1,2]],\u0022indicesGroups\u0022:{\u0022x\u0022:[5,6],\u0022y\u0022:[0,3],\u0022z\u0022:[1,2]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<=a(?<=(?<y>\u005c\u005ck<z>(?<z>b)\u005c\u005ck<x>){2}))c\u0022}"
        "(a)(?<=a(?<=((b)\u005c1){2}))c" flags="di" input="bAbAc" "{\u0022matches\u0022:[\u0022Ac\u0022,\u0022A\u0022,\u0022bA\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022bAbAc\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=a(?<=((b)\u005c\u005c1){2}))c\u0022}"
        "()(a)(?<=a(?<=(\u005c1\u005c2)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,0],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(a)(?<=a(?<=(\u005c\u005c1\u005c\u005c2)))b\u0022}"
        "(a)?(?<=a(?<=(\u005c1)))b" flags="d" input="ab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022ab\u0022,\u0022indices\u0022:[[0,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)?(?<=a(?<=(\u005c\u005c1)))b\u0022}"
        "#);
}
#[test]
fn nested_names_backward_dependencies_negative_rollback_and_surrogates_keep_precise_ranges() {
    assert_true(
        r#"
            let a = /(?<x>a)(?<=a(?<=(?<y>(?<z>b)\k<x>){2}))c/d.exec('babac');
            a.groups.x === 'a' && a.groups.y === 'ba' && a.groups.z === 'b' && a.indices.groups.z === a
                .indices[3] && a.indices[1][0] === 3 && a.indices[2][0] === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(?<x>a)(?<=a(?<=(?<y>\k<z>(?<z>b)\k<x>){2}))c/d.exec('bbabbac');
            a.groups.y === 'bba' && a.indices.groups.z[0] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(a)(?<=a(?!q)(?<!((b)\1){2}q))c/d.exec('babac');
            a[1] === 'a' && a[2] === undefined && a[3] === undefined && a.indices[2] === undefined
        "#,
    );
    assert_true(
        r"let a=/(a)(?<=a(?<=((b)\1){0}))c/d.exec('ac');a[1]==='a'&&a[2]===undefined&&a[3]===undefined",
    );
    assert_true(
        r#"
            let a = /([\uD800])(?<=[\uD800](?<=(([\uDC00])\1){2}))c/d.exec('\uDC00\uD800\uDC00\uD800c');
            a.index === 3 && a.indices[2][0] === 0 && a.indices[3][0] === 0
        "#,
    );
}
#[test]
fn consumers_sticky_global_empty_advance_and_callbacks_preserve_backward_child_positions() {
    assert_true(
        r#"
            let r = /(a)(?<=a(?<=((b)\1){2}))c/dy;
            r.lastIndex = 3;
            let a = r.exec('babac');
            a.index === 3 && r.lastIndex === 5 && r.exec('babac') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(a)(?<=a(?<=((b)\1){2}))c/dg,
                a = [...'babac babac'.matchAll(r)];
            a.length === 2 && a[1].index === 9 && a[1].indices[2][0] === 6 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = [...'babac'.matchAll(/(?<=(a))(?<=a(?<=((b)\1){2}))/dg)];
            a.length === 1 && a[0].index === 4 && a[0][0] === '' && a[0].indices[2][0] === 0
        "#,
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'babac babac'.replace(/(a)(?<=a(?<=((b)\1){2}))c/g, (m, x, y, z, i) => {
                seen.push(x, y, z, i);
                return '_'
            });
            s === 'bab_ bab_' && seen.join('|') === 'a|ba|b|3|a|ba|b|9'
        "#,
    );
    assert_true(
        r"'babac'.search(/(a)(?<=a(?<=((b)\1){2}))c/)===3&&'babac'.split(/(a)(?<=a(?<=((b)\1){2}))c/).join('|')==='bab|a|ba|b|'",
    );
}

#[test]
fn opted_in_work_abort_preserves_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"let marker=0,r=/(a)(?<=a(?<=((b)\1){2}))c/g;r.lastIndex=1;let text='b'.repeat(5000)+'babac'").unwrap();
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
fn changing_owner_captures_variable_counts_choices_and_unicode_remain_pending() {
    for source in [
        r"/(a)(?<=a(?<=((b)\1){1,2}))c/.exec('babac')",
        r"/(a)(?<=a(?<=(\1|b)))c/.exec('ac')",
        r"/(a)(?<=a(?<=\2))(b)/.exec('ab')",
        r"/(a)(?<=((b)(?<=\2)))c/.exec('abc')",
        r"/(a)(?<=a(?=(\1)(?<=\2)))a/.exec('aaa')",
        r"/(a)(?<=a(?<=\1))b/u.exec('ab')",
        r"/(a)(?<=a(?<=\1))b/v.exec('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
