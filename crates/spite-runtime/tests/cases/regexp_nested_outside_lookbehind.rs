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
            r#"
                let r = new RegExp({source:?}, {flags:?}),
                    a = r.exec({input:?});
                JSON.stringify(a === null ? {{
                    match: null,
                    lastIndex: r.lastIndex
                }} : {{
                    matches: [...a],
                    index: a.index,
                    input: a.input,
                    groups: a.groups,
                    indices: a.indices,
                    indicesGroups: a.indices.groups,
                    lastIndex: r.lastIndex,
                    source: r.source
                }}, null, 2)
            "#
        );
        let Value::String(result) = Realm::default().eval(&program).unwrap() else {
            panic!("expected JSON")
        };
        writeln!(
            rows,
            "{source:?} flags={flags:?} input={input:?}\n{result}\n",
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
    }
    insta::assert_snapshot!(rows);
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
fn pattern_variants_match_reference_results() {
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
