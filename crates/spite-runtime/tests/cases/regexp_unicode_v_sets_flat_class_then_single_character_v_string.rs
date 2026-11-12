//! Match results and consumers for single-character strings next to character classes.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn flat_class_then_single_character_v_string_state_context_and_consumers() {
    assert_true(
        r#"/[a][\q{b}]/v.exec('aab!').index===1&&!/[a][\q{b}]/v.test('ax')&&!/[a][\q{b}]/v.test('a')"#,
    );
    assert_true(
        r#"
            /[d][a\q{b|c}]/v.test('dc') && /[a][[\q{b|c}]d-f]/v.test('ae') && /[a][\q{a|b}&&[b-z]]/v.test(
                'ab') && !/[a][\q{a|b}--b]/v.test('ab') && !/[][a--a--\q{a}]/v.test('aa')
        "#,
    );
    assert_true(
        r#"let m=/[a][\q{a}]\b/v.exec('aaa!');m.index===1&&m[0]==='aa'&&/^[a][\q{b}]/v.exec('abx')[0]==='ab'"#,
    );
    assert_true(
        r#"
            let r = /[a][\q{a}]\b/dvgy;
            r.lastIndex = 0;
            let a = r.exec('aaa!');
            r.lastIndex = 1;
            let b = r.exec('aaa!');
            a === null && b.index === 1 && b[0] === 'aa' && b.indices[0][1] === 3 && r.lastIndex === 3 && r
                .exec('aaa!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /[a][\q{b}]/vg,
                a = r.exec('abab!'),
                b = r.exec('abab!');
            a.index === 0 && b.index === 2 && r.lastIndex === 4 && r.exec('abab!') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /[s][\q{K}&&k]/iv.exec('!ſK!')[0] === 'ſK' && !/[s][\q{K}&&k]/v.test('ſK') && !
                /[s][^\q{K|s}]/iv.test('ſK')
        "#,
    );
    assert_true(
        r#"
            let m = /\B[\p{ASCII}][^\q{K|s}]\B/iv.exec('KſA$');
            m.index === 2 && m[0] === 'A$' && /^\b[\p{ASCII}][\q{\!}\p{Any}]\B$/iv.test('K$') && !
                /^\b[\p{ASCII}][\q{\!}\p{Any}]\B$/v.test('K$')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\u{1f600}][\q{\uD83D\uDE80|x}--x]\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀🚀!');
            m.index === 1 && m[0] === '😀🚀' && m.indices[0][0] === 1 && m.indices[0][1] === 5 && r
                .lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let m = /[\\][\q{\uD800}]/v.exec('A\\\uD800B');
            m.index === 1 && m[0].length === 2 && m[0].charCodeAt(0) === 92 && m[0].charCodeAt(1) ===
                0xD800 && !/[\\][\q{\uD800}]/v.test('\\😀')
        "#,
    );
    assert_true(
        r#"
            let m = /^[\$][\q{\]|\\|\[}]/mv.exec('zxy\r\n$]tail\u2028');
            m.index === 5 && m[0] === '$]' && /^[\$][\q{\]|\\|\[}]/v.test('$\\') && /^[\$][\q{\]|\\|\[}]/v
                .test('$[')
        "#,
    );
    assert_true(
        r#"
            let m = /[a-c][\q{c|d}]$/mv.exec('x\r\naabc\u2028z');
            m.index === 5 && m[0] === 'bc' && !/[a-c][\q{c|d}]$/v.test('x\r\naabc\u2028z')
        "#,
    );
    assert_true(
        r#"
            let trace = '',
                r = /[a][\q{b}]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return 'aab!'
                }
            });
            trace === 'si' && m[0] === 'ab' && m.index === 1 && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[a][\q{b}]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            r.exec('aab!').index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[s][\q{K}&&k]/iv;
            r.lastIndex = 7;
            let c = new RegExp(r, 'dvy');
            c.source === '[s][\\q{K}&&k]' && c.flags === 'dvy' && c.lastIndex === 0 && c.exec('ſK') ===
                null && r.exec('ſK')[0] === 'ſK' && r.lastIndex === 7
        "#,
    );
    assert_true(
        r#"
            let a = /[a][\q{b}]/vy,
                b = /[a][\q{b}]/vy;
            a.lastIndex = 1;
            Object.defineProperty(a, 'lastIndex', {
                writable: false
            });
            Object.defineProperty(b, 'lastIndex', {
                writable: false
            });
            let x = false,
                y = false;
            try {
                a.exec('aab!')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('aab!')
            } catch (e) {
                y = e instanceof TypeError
            }
            x && y && a.lastIndex === 1 && b.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([a][\q{b}]))/dvg,
                a = [...'aab-ab!'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 4 && a[1].groups.whole === 'ab' && a[1]
                .indices[2][1] === 6 && 'aab-ab!'.replace(r, '<$<whole>>') === 'a<ab>-<ab>!' && 'aab-ab!'
                .split(/[a][\q{b}]/v).join('|') === 'a|-|!'
        "#,
    );
    assert_true(
        r#"
            let r = /[a][\q{b}]/vg;
            r.lastIndex = 4;
            'aab!'.search(r) === 1 && r.lastIndex === 4 && 'ab-ab'.match(r).join('|') === 'ab|ab' && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /[b-d][\q{a|b}]/iv.test('Da') && !/[b-d][\q{a|b}]/v.test('Da') && /[\d][\q{a}]/v.test('7a') &&
                /[\D][\q{a}]/v.test('!a')
        "#,
    );
    assert_true(
        r#"
            /[\p{ASCII}][\q{a}]/v.test('Aa') && !/[\p{ASCII}][\q{a}]/v.test('Ka') && /[\P{ASCII}][\q{a}]/v
                .test('Ka') && !/[][\q{a}]/v.test('ab')
        "#,
    );
    assert_true(
        r#"/[\p{ASCII}][\q{a}]/iv.test('KA')&&/[\p{ASCII}][\q{a}]/iv.test('ſA')&&!/[\p{ASCII}][\q{a}]/v.test('ſA')"#,
    );
    for pattern in [
        r"[c][\q{a}-b]",
        r"[c][^\q{ab}]",
        r"[b][\q{\d}]",
        r"[b][\q{\_}]",
        r"[\u{110000}][\q{a}]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('ab');").as_str());
        assert_true(&format!(
            r#"
                let marker = 0,
                    caught = false;
                try {{
                    eval({code:?})
                }} catch (e) {{
                    caught = e instanceof SyntaxError
                }}
                caught && marker === 0
            "#
        ));
    }
    {
        let source = r"/[\q{a}][\q{b}]/v.test('ab')";
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(true)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    for source in [
        r"/([a])[\q{b}]/v.test('ab')",
        r"/[a]([\q{b}])/v.test('ab')",
        r"/[a]+[\q{b}]/v.test('ab')",
        r"/[a][\q{b}]+/v.test('ab')",
        r"/(?:[a][\q{b}])+/v.test('ab')",
        r"/[a][\q{b}][c]/v.test('abc')",
        r"/[][\q{}]/v.test('ab')",
        r"/[][\q{a|bc}]/v.test('ab')",
        r"/[][a--a--\q{a|bc}]/v.test('ab')",
        r"/[][\q{a}&&[]&&\p{RGI_Emoji}]/v.test('ab')",
        r"/[\p{RGI_Emoji}][\q{a}]/v.test('ab')",
        r"/[a][\q{b}]$$/v.test('ab')",
        r"/\b\b[a][\q{b}]/v.test('ab')",
        r"/[a][\q{b}]\b\b/v.test('ab')",
        r"/\b^[a][\q{b}]/v.test('ab')",
        r"/[a]\b[\q{b}]/v.test('ab')",
        r"/[a][\q{b}]$\b/v.test('ab')",
        r"/[a][\q{b}]|c/v.test('ab')",
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        crate::common::regexp_oracle::assert_matches_reference(
            &mut realm,
            &format!("marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"),
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([a][\q{a|b}&&b]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a][\\q{a|b}&&b]))'&&kept.lastIndex===1&&kept.exec('aab!').groups.whole==='ab'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    // Construction cancellation only: no search setup is assumed reachable.
    // Over8000 source units make the new adapter's4N exceed this opt-in budget;
    // an earlier validated-construction charge may fail first.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source =
        format!(r#"try{{r=/[a][\q{{{choices}}}]/vg}}catch{{marker=1}}finally{{marker=2}}"#);
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));

    // Exact original Boolean controls now complete through checked-prefix/q-suffix proof.
    for (source, expected) in [
        (r"/[[a]][\q{b}]/v.test('ab')", true),
        (r"/[a&&a][\q{b}]/v.test('ab')", true),
        (r"/[a--a][\q{b}]/v.test('ab')", false),
    ] {
        let mut realm = Realm::default();
        realm.eval("var marker=0").unwrap();
        assert_eq!(
            realm.eval(&format!(
                "marker=1;try{{{source};}}catch{{marker=2}}finally{{marker=3}}"
            )),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
        assert_eq!(realm.eval("marker"), Ok(Value::Number(3.0)), "{source}");
    }
    assert_true(
        r#"
            let r = /(?<whole>([[a]][\q{b}]))/dvg,
                m = r.exec("ab");
            Array.isArray(m) && Object.getPrototypeOf(m) === Array.prototype && m.length === 3 && m[0] ===
                "ab" && m[1] === "ab" && m[2] === "ab" && m.index === 0 && m.input === "ab" && Object
                .getPrototypeOf(m.groups) === null && m.groups.whole === "ab" && m.indices.length === 3 &&
                m.indices[0][0] === 0 && m.indices[0][1] === 2 && m.indices[1][0] === 0 && m.indices[1][
                1] === 2 && m.indices[2][0] === 0 && m.indices[2][1] === 2 && m.indices.groups.whole[0] ===
                0 && m.indices.groups.whole[1] === 2 && r.source === "(?<whole>([[a]][\\q{b}]))" && r
                .flags === 'dgv' && r.lastIndex === 2 && r.exec("ab") === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([a&&a][\q{b}]))/dvg,
                m = r.exec("ab");
            Array.isArray(m) && Object.getPrototypeOf(m) === Array.prototype && m.length === 3 && m[0] ===
                "ab" && m[1] === "ab" && m[2] === "ab" && m.index === 0 && m.input === "ab" && Object
                .getPrototypeOf(m.groups) === null && m.groups.whole === "ab" && m.indices.length === 3 &&
                m.indices[0][0] === 0 && m.indices[0][1] === 2 && m.indices[1][0] === 0 && m.indices[1][
                1] === 2 && m.indices[2][0] === 0 && m.indices[2][1] === 2 && m.indices.groups.whole[0] ===
                0 && m.indices.groups.whole[1] === 2 && r.source === "(?<whole>([a&&a][\\q{b}]))" && r
                .flags === 'dgv' && r.lastIndex === 2 && r.exec("ab") === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([a--a][\q{b}]))/dvg;
            r.lastIndex = 1;
            let m = r.exec("ab");
            m === null && r.lastIndex === 0 && r.source === "(?<whole>([a--a][\\q{b}]))" && r.flags ===
                'dgv'
        "#,
    );
}

#[test]
fn flat_class_then_single_character_v_string_results() {
    let mut rows = String::new();
    let mut row_count = 0;

    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>([a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            0usize,
        ),
        (
            r#"(?<whole>([a][\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            1usize,
        ),
        (
            r#"(?<whole>([a-c][\q{c|d}]$))"#,
            JsString::from_code_units(vec![120, 13, 10, 97, 97, 98, 99, 8232, 122]),
            "dgmv",
            0usize,
        ),
        (
            r#"(?<whole>(^[\$][\q{\]|\\|\[}]))"#,
            JsString::from_code_units(vec![122, 120, 121, 13, 10, 36, 93, 116, 97, 105, 108, 8232]),
            "dgmv",
            0usize,
        ),
        (
            r#"(?<whole>([a][\q{K}&&k]))"#,
            JsString::from_code_units(vec![65, 8490, 33]),
            "dgiv",
            0usize,
        ),
        (
            r#"(?<whole>(\B[\p{ASCII}][^\q{K|s}]\B))"#,
            JsString::from_code_units(vec![8490, 383, 65, 36]),
            "dgiv",
            0usize,
        ),
        (
            r#"(?<whole>([\uD83D\uDE00][\q{\uD83D\uDE80}]))"#,
            JsString::from_code_units(vec![55357, 56832, 55357, 56960, 33]),
            "dvgy",
            1usize,
        ),
        (
            r#"(?<whole>([\\][\q{\uD800}]))"#,
            JsString::from_code_units(vec![92, 55296]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([][a--a--\q{a}]))"#,
            JsString::from_code_units(vec![97, 99]),
            "dvg",
            0usize,
        ),
    ] {
        let script = format!(
            r#"
                let r = new RegExp({pattern:?}, {flags:?});
                r.lastIndex = {start};
                let m = r.exec({input:?});
                JSON.stringify(m === null ? {{
                    match: null,
                    lastIndex: r.lastIndex
                }} : {{
                    matches: Array.from(m),
                    index: m.index,
                    input: m.input,
                    groups: m.groups,
                    indices: Array.from(m.indices),
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
            "{pattern:?} flags={flags:?} input_units={:?} lastIndex={start}\n{result}\n",
            input.code_units(),
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}
