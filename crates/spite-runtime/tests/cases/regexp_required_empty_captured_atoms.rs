//! Required counts of proved empty captured references retain effects once.
use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
#[test]
fn required_empty_captured_atoms_result_snapshot() {
    let mut rows = String::new();
    for (source, flags, text) in [
        (r"(?<=(a)(\1){2})b", "d", "ab"),
        (r"(?<=(a)(\1)+)b", "d", "ab"),
        (r"(?<=(a)(\1){2,4}?)b", "d", "ab"),
        (r"(?<=(\1){2}a)b", "d", "ab"),
        (r"(?=(\2){2}(a))a", "d", "a"),
        (r"(?=(\2)+(a))a", "d", "a"),
        (r"(?<=a(?=(\2)+(b)))b", "d", "ab"),
        (r"((\3)+(b))+", "d", "bbb"),
        (r"((\2)+b)+", "d", "bbb"),
        (r"(?<=((b)(\2)+){2})c", "d", "bbc"),
        (r"(?<=((\1)+b){2})c", "d", "bbc"),
        (r"(a)((\3)+b)+", "d", "abbb"),
        (r"(?<!(a)(\1){2})b", "d", "b"),
        (r"(?!(\2){2}(a))b", "d", "b"),
        (r"(?<=(a)(()\1){2})b", "d", "ab"),
        (r"(?<=(a)(\1()){2})b", "d", "ab"),
        (r"((\1){1,2}){2}a()", "d", "a"),
        (r"(?<=(?<x>a)(?<y>\k<x>)+)b", "d", "ab"),
        (r"(?=(?<x>\k<y>)+(?<y>a))a", "d", "a"),
        (r"(?<=a(?=(?<x>\k<y>)+(?<y>b)))b", "d", "ab"),
        (r"(\2)*(b)", "d", "bb"),
        (r"(?<=(a)(\1){2})b", "di", "Ab"),
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
fn required_effects_original_gap_names_open_reads_and_negative_rollback_are_exact() {
    assert!(matches!(
        Realm::default().eval(r"/((\1){1,2}){2}a()/.exec('a')"),
        Ok(Value::Object(_))
    ));
    assert_true(
        r#"
            let a = /((\1){1,2}){2}a()/d.exec('a');
            a[1] === '' && a[2] === '' && a[3] === '' && a.indices[1][0] === 0 && a.indices[2][0] === 0 &&
                a.indices[3][0] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=(?<x>a)(?<y>\k<x>)+)b/d.exec('ab');
            a.groups.x === 'a' && a.groups.y === '' && a.indices.groups.y === a.indices[2] && a.indices[2][
                0
            ] === 1
        "#,
    );
    assert_true(
        r#"
            let a = /(?=(?<x>\k<y>)+(?<y>a))a/d.exec('a');
            a.groups.x === '' && a.groups.y === 'a' && a.indices.groups.x[0] === 0
        "#,
    );
    assert_true(
        r#"
            let a = /(?<=a(?=(?<x>\k<y>)+(?<y>b)))b/d.exec('ab');
            a.index === 1 && a.groups.x === '' && a.indices.groups.y[0] === 1
        "#,
    );
    assert_true(r"let a=/(?<=(\1){2}a)b/d.exec('ab');a[1]===''&&a.indices[1][0]===0");
    assert_true(
        r"let a=/(?<!(a)(\1){2})b/d.exec('b');a[1]===undefined&&a[2]===undefined&&a.indices[2]===undefined",
    );
    assert_true(
        r"let a=/(?<=([\uD800])(\1){2})b/d.exec('\uD800b');a.index===1&&a[2]===''&&a.indices[2][0]===1",
    );
}
#[test]
fn consumers_empty_global_sticky_advancement_and_callbacks_keep_required_slots() {
    assert_true(
        r#"
            let r = /(?<=(a)(\1)+)b/dy;
            r.lastIndex = 1;
            let a = r.exec('ab');
            a[2] === '' && r.lastIndex === 2 && r.exec('ab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<=(a)(\1)+)b/dg,
                a = [...'ab ab'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 4 && a[1].indices[2][0] === 4 && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r"let a=[...'aa'.matchAll(/(?=(\2)+(a))/dg)];a.length===2&&a[0][1]===''&&a[1].indices[1][0]===1",
    );
    assert_true(
        r#"
            let seen = [];
            let s = 'ab ab'.replace(/(?<=(a)(\1)+)b/g, (m, x, y, i) => {
                seen.push(x, y, i);
                return '_'
            });
            s === 'a_ a_' && seen.join('|') === 'a||1|a||4'
        "#,
    );
    assert_true(
        r"'ab'.search(/(?<=(a)(\1)+)b/)===1&&'ab'.split(/(?<=(a)(\1)+)b/).join('|')==='a|a||'",
    );
}

#[test]
fn opted_in_work_abort_keeps_last_index_and_bypasses_handlers() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"let marker=0,r=/(?<=(a)(\1)+)b/g;r.lastIndex=1;let text='c'.repeat(10000)+'ab'")
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
fn pattern_variants_match_reference_results() {
    for source in [
        r"/(?<=a(?=(b)(\1){2,3}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1){1,2}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1){0,2}))b/.exec('abbb')",
        r"/(?<=a(?=(b)(\1\1){2,3}))b/.exec('abbbbb')",
        r"/(?<=(a)(\1){2})b/u.exec('ab')",
        r"/(?<=(a)(\1){2})b/v.exec('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}
