//! Match results and consumers for single-character strings next to character classes.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn single_character_v_string_then_flat_class_state_context_and_consumers() {
    assert_true(
        r#"/[\q{a}][b]/v.exec('aab!').index===1&&!/[\q{a}][b]/v.test('ax')&&!/[\q{a}][b]/v.test('a')"#,
    );
    assert_true(
        r#"
            /[a\q{b|c}][d]/v.test('cd') && /[[\q{b|c}]d-f][a]/v.test('ea') && /[\q{a|b}&&[b-z]][a]/v.test(
                'ba') && !/[\q{a|b}--b][a]/v.test('ba') && !/[a--a--\q{a}][c]/v.test('ac')
        "#,
    );
    assert_true(
        r#"let m=/[\q{a}][a]\b/v.exec('aaa!');m.index===1&&m[0]==='aa'&&/^[\q{a}][b]/v.exec('abx')[0]==='ab'"#,
    );
    assert_true(
        r#"
            let r = /[\q{a}][a]\b/dvgy;
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
            let r = /[\q{a}][b]/vg,
                a = r.exec('abab!'),
                b = r.exec('abab!');
            a.index === 0 && b.index === 2 && r.lastIndex === 4 && r.exec('abab!') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /[\q{K}&&k][s]/iv.exec('!Kſ!')[0] === 'Kſ' && !/[\q{K}&&k][s]/v.test('Kſ') && !
                /[^\q{K|s}][s]/iv.test('Kſ')
        "#,
    );
    assert_true(
        r#"
            let m = /\B[^\q{K|s}][\p{ASCII}]\B/iv.exec('KſA$');
            m.index === 2 && m[0] === 'A$' && /^\b[\q{\!}\p{Any}][\$]\B$/iv.test('K$') && !
                /^\b[\q{\!}\p{Any}][\$]\B$/v.test('K$')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\q{\uD83D\uDE00|x}--x][\u{1f680}]\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀🚀!');
            m.index === 1 && m[0] === '😀🚀' && m.indices[0][0] === 1 && m.indices[0][1] === 5 && r
                .lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let m = /[\q{\uD800}][\\]/v.exec('A\uD800\\B');
            m.index === 1 && m[0].length === 2 && m[0].charCodeAt(0) === 0xD800 && m[0].charCodeAt(1) ===
                92 && !/[\q{\uD800}][\\]/v.test('😀\\')
        "#,
    );
    assert_true(
        r#"
            let m = /^[\q{\]|\\|\[}][\$]/mv.exec('zxy\r\n]$tail\u2028');
            m.index === 5 && m[0] === ']$' && /^[\q{\]|\\|\[}][\$]/v.test('\\$') && /^[\q{\]|\\|\[}][\$]/v
                .test('[$')
        "#,
    );
    assert_true(
        r#"
            let m = /[\q{a|b}][c]$/mv.exec('x\r\naabc\u2028z');
            m.index === 5 && m[0] === 'bc' && !/[\q{a|b}][c]$/v.test('x\r\naabc\u2028z')
        "#,
    );
    assert_true(
        r#"
            let trace = '',
                r = /[\q{a}][b]/vy;
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
                r = /[\q{a}][b]/v;
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
            let r = /[\q{K}&&k][s]/iv;
            r.lastIndex = 7;
            let c = new RegExp(r, 'dvy');
            c.source === '[\\q{K}&&k][s]' && c.flags === 'dvy' && c.lastIndex === 0 && c.exec('Kſ') ===
                null && r.exec('Kſ')[0] === 'Kſ' && r.lastIndex === 7
        "#,
    );
    assert_true(
        r#"
            let a = /[\q{a}][b]/vy,
                b = /[\q{a}][b]/vy;
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
            let r = /(?<whole>([\q{a}][b]))/dvg,
                a = [...'aab-ab!'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 4 && a[1].groups.whole === 'ab' && a[1]
                .indices[2][1] === 6 && 'aab-ab!'.replace(r, '<$<whole>>') === 'a<ab>-<ab>!' && 'aab-ab!'
                .split(/[\q{a}][b]/v).join('|') === 'a|-|!'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a}][b]/vg;
            r.lastIndex = 4;
            'aab!'.search(r) === 1 && r.lastIndex === 4 && 'ab-ab'.match(r).join('|') === 'ab|ab' && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /[\q{a|b}][b-d]/iv.test('aD') && !/[\q{a|b}][b-d]/v.test('aD') && /[\q{a}][\d]/v.test('a7') &&
                /[\q{a}][\D]/v.test('a!')
        "#,
    );
    assert_true(
        r#"
            /[\q{a}][\p{ASCII}]/v.test('aA') && !/[\q{a}][\p{ASCII}]/v.test('aK') && /[\q{a}][\P{ASCII}]/v
                .test('aK') && !/[\q{a}][]/v.test('ab')
        "#,
    );
    for pattern in [
        r"[\q{a}-b][c]",
        r"[^\q{ab}][c]",
        r"[\q{\d}][b]",
        r"[\q{\_}][b]",
        r"[\q{a}][\u{110000}]",
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
        let source = r"/[a][\q{b}]/v.test('ab')";
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
        r"/([\q{a}])[b]/v.test('ab')",
        r"/[\q{a}]([b])/v.test('ab')",
        r"/[\q{a}]+[b]/v.test('ab')",
        r"/[\q{a}][b]+/v.test('ab')",
        r"/(?:[\q{a}][b])+/v.test('ab')",
        r"/[\q{a}][b][c]/v.test('abc')",
        r"/[\q{}][b]/v.test('ab')",
        r"/[\q{a|bc}][b]/v.test('ab')",
        r"/[a--a--\q{a|bc}][b]/v.test('ab')",
        r"/[\q{a}&&[]&&\p{RGI_Emoji}][b]/v.test('ab')",
        r"/[a--a--\q{a}][\p{RGI_Emoji}]/v.test('ab')",
        r"/[\q{a}][b]$$/v.test('ab')",
        r"/\b\b[\q{a}][b]/v.test('ab')",
        r"/[\q{a}][b]\b\b/v.test('ab')",
        r"/\b^[\q{a}][b]/v.test('ab')",
        r"/[\q{a}]\b[b]/v.test('ab')",
        r"/[\q{a}][b]$\b/v.test('ab')",
        r"/[\q{a}][b]|c/v.test('ab')",
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
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&b][a]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&b][a]))'&&kept.lastIndex===1&&kept.exec('zba!').groups.whole==='ba'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    // Explicit construction cancellation: no search setup is assumed reachable.
    // At over8000 units the new adapter's4N alone exceeds this opt-in budget.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval("var r=7,marker=0").unwrap();
    let choices = "a|".repeat(4000) + "a";
    let source =
        format!(r#"try{{r=/[\q{{{choices}}}][b]/vg}}catch{{marker=1}}finally{{marker=2}}"#);
    assert!(matches!(realm.eval(&source), Err(Error::Limit { .. })));
    assert_eq!(realm.eval("marker===0&&r===7"), Ok(Value::Boolean(true)));
}

#[test]
fn single_character_v_string_then_flat_class_results() {
    let mut rows = String::new();
    let mut row_count = 0;

    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>([\q{a}][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([\q{a}][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            0usize,
        ),
        (
            r#"(?<whole>([\q{a}][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 33]),
            "dvgy",
            1usize,
        ),
        (
            r#"(?<whole>([\q{a|b}][c]$))"#,
            JsString::from_code_units(vec![120, 13, 10, 97, 97, 98, 99, 8232, 122]),
            "dgmv",
            0usize,
        ),
        (
            r#"(?<whole>(^[\q{\]|\\|\[}][\$]))"#,
            JsString::from_code_units(vec![122, 120, 121, 13, 10, 93, 36, 116, 97, 105, 108, 8232]),
            "dgmv",
            0usize,
        ),
        (
            r#"(?<whole>([\q{K}&&k][a]))"#,
            JsString::from_code_units(vec![8490, 65, 33]),
            "dgiv",
            0usize,
        ),
        (
            r#"(?<whole>(\B[^\q{K|s}][\p{ASCII}]\B))"#,
            JsString::from_code_units(vec![8490, 383, 65, 36]),
            "dgiv",
            0usize,
        ),
        (
            r#"(?<whole>([\q{\uD83D\uDE00}][\uD83D\uDE80]))"#,
            JsString::from_code_units(vec![55357, 56832, 55357, 56960, 33]),
            "dvgy",
            1usize,
        ),
        (
            r#"(?<whole>([\q{\uD800}][\\]))"#,
            JsString::from_code_units(vec![55296, 92]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([a--a--\q{a}][\p{ASCII}]))"#,
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
