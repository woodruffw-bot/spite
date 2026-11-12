//! Match results and consumers for repeated character classes next to word assertions.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_repeated_word_state_context_and_consumers() {
    assert_true(
        r#"
            let r = /\b[a--a]*\b/dvg,
                m = r.exec('a');
            m[0] === '' && m.index === 0 && m.indices[0][1] === 0 && r.lastIndex === 0 && r.exec('a')
                .index === 0 && r.lastIndex === 0 && !/\b[a--a]+\b/v.test('a') && /\B[a--a]{0}\B/v.exec(
                '')[0] === '' && /\b[a--a]*\B/v.exec('a') === null
        "#,
    );
    assert_true(
        r#"
            let r = /\b[a--a]*\b/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀B');
            m[0] === '' && m.index === 1 && m.indices[0][0] === 1 && m.indices[0][1] === 1 && r
                .lastIndex === 1 && (r.lastIndex = 1, r.exec('😀!') === null) && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\b[a--a]*\b/dvg,
                a = [...'A😀B'.matchAll(r)];
            a.map(m => m.index).join(',') === '0,1,3,4' && r.lastIndex === 0 && 'A😀B'.replace(
                /\b[a--a]*\b/vg, '|') === '|A|😀|B|'
        "#,
    );
    assert_true(
        r"'aa_!'.replace(/(?<whole>(\b[[a_]]+\B))/v,'<$1:$<whole>>')==='<aa:aa>_!'&&'aaa!'.match(/\b[[a]]+?\b/v)[0]==='aaa'",
    );
    assert_true(r"let r=/\b[[a]]+?\b/vg;r.lastIndex=1;'x aaa!'.search(r)===2&&r.lastIndex===1");
    assert_true(
        r#"
            let r = /\b[[\p{Any}]]{2}\b/dvy,
                m = r.exec('A\uD800B');
            m.index === 0 && m[0] === 'A\uD800' && m.indices[0][1] === 2 && r.lastIndex === 2 && r.exec(
                'A\uD800B') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let marker = 0,
                caught = false;
            try {
                eval('marker=1; /\\b[[a]]{3,2}\\b/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/\b[[\q{a}]]{0}\b/v.test('a')");
    for source in [
        r"/\b^[[a]]+/v.test('a')",
        r"/\b[[a]]+$\b/v.test('a')",
        r"/(\b[[a]]\b)+/v.test('a')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn checked_v_set_repeated_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, flags, start) in [
        (r"(?<whole>(\b[[a_]]+\B))", JsString::from("aa_!"), "dvg", 0),
        (
            r"(?<whole>(\b[[a_]]+?\B))",
            JsString::from("aa_!"),
            "dvg",
            0,
        ),
        (r"(?<whole>(\b[[a]]+?\b))", JsString::from("aaa!"), "dvg", 0),
        (r"(?<whole>(\b[[K]]+\b))", JsString::from("KK!"), "dvg", 0),
        (r"(?<whole>(\b[[K]]+\b))", JsString::from("KK!"), "divg", 0),
        (r"(?<whole>(\B[a--a]*\B))", JsString::from("😀!"), "dvy", 1),
        (r"(?<whole>(\b[a--a]*\b))", JsString::from("A😀B"), "dvy", 2),
        (
            r"^(?<whole>(\b[[a]]{2,3}?\b))$",
            JsString::from("x\r\naaa\u{2028}"),
            "dvmg",
            0,
        ),
        (
            r"(?<whole>(\b[[\p{Any}]]{2}\b))",
            JsString::from("A😀B"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\b[[a]]{0,1}?\b))",
            JsString::from("a!"),
            "dvg",
            0,
        ),
    ] {
        let pattern = JsString::from(pattern);
        let script = format!(
            r#"
                let r = new RegExp({pattern:?}, '{flags}');
                r.lastIndex = {start};
                let m = r.exec({input:?});
                JSON.stringify(m === null ? {{
                    match: null,
                    lastIndex: r.lastIndex
                }} : {{
                    matches: [...m],
                    index: m.index,
                    input: m.input,
                    groups: m.groups,
                    indices: m.indices,
                    indexGroups: m.indices.groups,
                    lastIndex: r.lastIndex,
                    source: r.source
                }}, null, 2)
            "#
        );
        let Value::String(result) = Realm::default().eval(&script).unwrap() else {
            panic!("{script}")
        };
        writeln!(
            rows,
            "{pattern:?} flags={flags:?} input={input:?} lastIndex={start}\n{result}\n",
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}
