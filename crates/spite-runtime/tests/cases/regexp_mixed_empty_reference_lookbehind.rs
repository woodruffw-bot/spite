//! Outside captures combined with proven empty same-unit backward reads.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

use crate::common::assert_true;

#[test]
fn mixed_empty_reference_lookbehind_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(a)(?<=((b)\3\1){2})c", "d", "babac"),
        (r"(a)(?<=(b\1\2){2})c", "d", "babac"),
        (r"(a)(?<=(\1(b)\3){2})c", "d", "ababc"),
        (r"(a)(?<=(\1(a)\3){2})c", "d", "aaaac"),
        (r"(a)(?<=(\1(\2)){2})b", "d", "aab"),
        (r"(ab)(?<=((b)\3\1){2})c", "d", "babbabc"),
        (r"(a)(?<=(\3b()\1){2})c", "d", "babac"),
        (r"(a)(?<=((b)\3\1){2}?)c", "d", "babac"),
        (r"(a)(?<=((b)\3\1){0})c", "d", "ac"),
        (r"(a)(?<!((b)\3\1){2}q)c", "d", "baxac"),
        (r"(a)(?<=((b)\3\1\B){2})c", "d", "babac"),
        (r"()(?<=((())\3\1){2})b", "d", "b"),
        (r"(?<=((a)\2\3){2})(b)", "d", "aab"),
        (r"(a(?<=((a)\3\1){2}))b", "d", "aab"),
        (r"(a)(?<=((b)\3\1){2})c", "di", "bAbAc"),
        (r"(?<x>a)(?<=(?<y>(?<z>b)\k<z>\k<x>){2})c", "d", "babac"),
        (r"(?:(?<x>a)|(?<x>b))(?<=(c\k<x>\2){2})d", "d", "cbcbcd"),
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
        "(a)(?<=((b)\u005c3\u005c1){2})c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=((b)\u005c\u005c3\u005c\u005c1){2})c\u0022}"
        "(a)(?<=(b\u005c1\u005c2){2})c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(b\u005c\u005c1\u005c\u005c2){2})c\u0022}"
        "(a)(?<=(\u005c1(b)\u005c3){2})c" flags="d" input="ababc" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "(a)(?<=(\u005c1(a)\u005c3){2})c" flags="d" input="aaaac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022aa\u0022,\u0022a\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022aaaac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[1,2]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1(a)\u005c\u005c3){2})c\u0022}"
        "(a)(?<=(\u005c1(\u005c2)){2})b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022,\u0022\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[1,3],[1,2],[0,1],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c1(\u005c\u005c2)){2})b\u0022}"
        "(ab)(?<=((b)\u005c3\u005c1){2})c" flags="d" input="babbabc" "{\u0022matches\u0022:[\u0022abc\u0022,\u0022ab\u0022,\u0022bab\u0022,\u0022b\u0022],\u0022index\u0022:4,\u0022input\u0022:\u0022babbabc\u0022,\u0022indices\u0022:[[4,7],[4,6],[0,3],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(ab)(?<=((b)\u005c\u005c3\u005c\u005c1){2})c\u0022}"
        "(a)(?<=(\u005c3b()\u005c1){2})c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[1,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=(\u005c\u005c3b()\u005c\u005c1){2})c\u0022}"
        "(a)(?<=((b)\u005c3\u005c1){2}?)c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=((b)\u005c\u005c3\u005c\u005c1){2}?)c\u0022}"
        "(a)(?<=((b)\u005c3\u005c1){0})c" flags="d" input="ac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,null,null],\u0022index\u0022:0,\u0022input\u0022:\u0022ac\u0022,\u0022indices\u0022:[[0,2],[0,1],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=((b)\u005c\u005c3\u005c\u005c1){0})c\u0022}"
        "(a)(?<!((b)\u005c3\u005c1){2}q)c" flags="d" input="baxac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,null,null],\u0022index\u0022:3,\u0022input\u0022:\u0022baxac\u0022,\u0022indices\u0022:[[3,5],[3,4],null,null],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<!((b)\u005c\u005c3\u005c\u005c1){2}q)c\u0022}"
        "(a)(?<=((b)\u005c3\u005c1\u005cB){2})c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=((b)\u005c\u005c3\u005c\u005c1\u005c\u005cB){2})c\u0022}"
        "()(?<=((())\u005c3\u005c1){2})b" flags="d" input="b" "{\u0022matches\u0022:[\u0022b\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022,\u0022\u0022],\u0022index\u0022:0,\u0022input\u0022:\u0022b\u0022,\u0022indices\u0022:[[0,1],[0,0],[0,0],[0,0],[0,0]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022()(?<=((())\u005c\u005c3\u005c\u005c1){2})b\u0022}"
        "(?<=((a)\u005c2\u005c3){2})(b)" flags="d" input="aab" "{\u0022matches\u0022:[\u0022b\u0022,\u0022a\u0022,\u0022a\u0022,\u0022b\u0022],\u0022index\u0022:2,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[2,3],[0,1],[0,1],[2,3]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<=((a)\u005c\u005c2\u005c\u005c3){2})(b)\u0022}"
        "(a(?<=((a)\u005c3\u005c1){2}))b" flags="d" input="aab" "{\u0022matches\u0022:[\u0022ab\u0022,\u0022a\u0022,\u0022a\u0022,\u0022a\u0022],\u0022index\u0022:1,\u0022input\u0022:\u0022aab\u0022,\u0022indices\u0022:[[1,3],[1,2],[0,1],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a(?<=((a)\u005c\u005c3\u005c\u005c1){2}))b\u0022}"
        "(a)(?<=((b)\u005c3\u005c1){2})c" flags="di" input="bAbAc" "{\u0022matches\u0022:[\u0022Ac\u0022,\u0022A\u0022,\u0022bA\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022bAbAc\u0022,\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(a)(?<=((b)\u005c\u005c3\u005c\u005c1){2})c\u0022}"
        "(?<x>a)(?<=(?<y>(?<z>b)\u005ck<z>\u005ck<x>){2})c" flags="d" input="babac" "{\u0022matches\u0022:[\u0022ac\u0022,\u0022a\u0022,\u0022ba\u0022,\u0022b\u0022],\u0022index\u0022:3,\u0022input\u0022:\u0022babac\u0022,\u0022groups\u0022:{\u0022x\u0022:\u0022a\u0022,\u0022y\u0022:\u0022ba\u0022,\u0022z\u0022:\u0022b\u0022},\u0022indices\u0022:[[3,5],[3,4],[0,2],[0,1]],\u0022indicesGroups\u0022:{\u0022x\u0022:[3,4],\u0022y\u0022:[0,2],\u0022z\u0022:[0,1]},\u0022lastIndex\u0022:0,\u0022source\u0022:\u0022(?<x>a)(?<=(?<y>(?<z>b)\u005c\u005ck<z>\u005c\u005ck<x>){2})c\u0022}"
        "(?:(?<x>a)|(?<x>b))(?<=(c\u005ck<x>\u005c2){2})d" flags="d" input="cbcbcd" "{\u0022match\u0022:null,\u0022lastIndex\u0022:0}"
        "#);
}

#[test]
fn names_aliases_open_future_and_closed_local_targets_keep_exact_ranges() {
    assert_true(r"/(a)(?<=(b\1\2){2})c/.exec('ababc')===null");
    assert_true(
        r#"
            let a = /([\uD800])(?<=(([\uDC00])\3\1){2})c/d.exec('\uDC00\uD800\uDC00\uD800c');
            a.index === 3 && a.indices[1][0] === 3 && a.indices[2][0] === 0 && a.indices[3][0] === 0
        "#,
    );

    assert_true(
        r#"
            let a = /(?<x>a)(?<=(?<y>(?<z>b)\k<z>\k<x>){2})c/d.exec('babac');
            a.index === 3 && a.groups.x === 'a' && a.groups.y === 'ba' && a.groups.z === 'b' && a.indices
                .groups.z === a.indices[3] && a.indices[3][0] === 0 && a.indices[1][0] === 3
        "#,
    );
    assert_true(
        r#"
            let a = /(a)(?<=(b\1\2){2})c/d.exec('babac'),
                b = /(a)(?<=(\1(\2)){2})b/d.exec('aab');
            a[2] === 'ba' && b[2] === 'a' && b[3] === '' && b.indices[3][0] === 1 && b.indices[1][0] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=((a)\2\3){2})(b)/d.exec('aab'),
                b = /(a(?<=((a)\3\1){2}))b/d.exec('aab');
            a.indices[1][0] === 0 && a.indices[3][0] === 2 && b.indices[1][0] === 1 && b.indices[2][0] ===
                0
        "#,
    );
    assert_true(r"let a=/(?:(?<x>a)|(?<x>b))(?<=(c\k<x>\2){2})d/d.exec('cbcbcd');a===null");
    assert_true(
        r#"
            let a = /(?:(?<x>a)|(?<x>b))(?<=(c\k<x>\3){2})d/d.exec('cbcbd');
            a.index === 3 && a[1] === undefined && a[2] === 'b' && a[3] === 'cb' && a.groups.x === 'b' && a
                .indices.groups.x === a.indices[2]
        "#,
    );
}

#[test]
fn consumers_last_index_callbacks_and_skipped_capture_effects_are_preserved() {
    assert_true(
        r#"
            let r = /(a)(?<=((b)\3\1){2})c/dy;
            r.lastIndex = 3;
            let a = r.exec('babac');
            a.index === 3 && r.lastIndex === 5 && r.exec('babac') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(a)(?<=((b)\3\1){2})c/dg,
                a = [...'babac babac'.matchAll(r)];
            a.length === 2 && a[1].index === 9 && a[1].indices[2][0] === 6 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(a)(?<=((b)\3\1){0})c/d.exec('ac');
            a[1] === 'a' && a[2] === undefined && a[3] === undefined && a.indices[2] === undefined
        "#,
    );
    assert_true(
        r"let a=[...'ab'.matchAll(/()(?<=((())\3\1){2})/dg)];a.length===3&&a[2].index===2&&a[2].indices[4][0]===2",
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'babac babac'.replace(/(a)(?<=((b)\3\1){2})c/g, (m, x, y, z, i) => {
                seen.push(x, y, z, i);
                return '_'
            });
            s === 'bab_ bab_' && seen.join('|') === 'a|ba|b|3|a|ba|b|9'
        "#,
    );
    assert_true(
        r"'babac'.search(/(a)(?<=((b)\3\1){2})c/)===3&&'babac'.split(/(a)(?<=((b)\3\1){2})c/).join('|')==='bab|a|ba|b|'",
    );
}

#[test]
fn explicit_work_aborts_preserve_last_index_and_bypass_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(
            r"let marker=0,r=/(a)(?<=((b)\3\1){2})c/g;r.lastIndex=1;let text='ba'.repeat(5000)+'c'",
        )
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
fn variable_counts_dependent_mixed_targets_choices_and_unicode_remain_pending() {
    for source in [
        r"/(a)(?<=((b)\3\1){1,2})c/.exec('babac')",
        r"/(a)(?<=(\1\3(b)){1,2})c/.exec('ababc')",
        r"/(a)(?<=((b)\3\1){2}|b)c/.exec('babac')",
        r"/(a)(?<=((b)\3\1){2})c/u.exec('babac')",
        r"/(a)(?<=((b)\3\1){2})c/v.exec('babac')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
