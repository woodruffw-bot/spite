//! Match results and consumers for single-character strings in set operations.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn single_character_v_string_operands_state_context_and_consumers() {
    assert_true(r#"/[a\q{b|c}]/v.test('a')&&/[a\q{b|c}]/v.test('b')&&!/[a\q{b|c}]/v.test('z')"#);
    assert_true(
        r#"/[a[\q{b|c}]d-f]/v.test('e')&&/[a[\q{b|c}]d-f]/v.test('c')&&!/[a[\q{b|c}]d-f]/v.test('x')"#,
    );
    assert_true(r#"/[\q{a|b}\d]/v.test('7')&&/[\q{a|b}\d]/v.test('b')&&!/[\q{a|b}\d]/v.test('z')"#);
    assert_true(r#"/[\q{a|b}&&[b-z]]/v.exec('aab').index===2&&!/[\q{a|b}&&[b-z]]/v.test('a')"#);
    assert_true(
        r#"
            /[\q{a|b}--b]/v.test('a') && !/[\q{a|b}--b]/v.test('b') && !/[a--\q{a|b}]/v.test('a') &&
                /[[a-c]--\q{b}]/v.test('c')
        "#,
    );
    assert_true(
        r#"
            /[\q{a|b}&&[^\q{a}]]/v.test('b') && !/[\q{a|b}&&[^\q{a}]]/v.test('a') && !/[^\q{a}&&a]/v.test(
                'a') && /[^\q{a}&&a]/v.test('b')
        "#,
    );
    assert_true(
        r#"/[\q{a}\q{b}]/v.test('b')&&/[\q{a|b}&&\q{b|c}]/v.test('b')&&!/[\q{a|b}&&\q{b|c}]/v.test('a')"#,
    );
    assert_true(r#"/[\q{a|😀}&&\p{ASCII}]/v.test('a')&&!/[\q{a|😀}&&\p{ASCII}]/v.test('😀')"#);
    assert_true(
        r#"/[\q{K|s}--k]/iv.test('ſ')&&!/[\q{K|s}--k]/iv.test('KKk')&&/[\q{K|s}--k]/v.test('K')"#,
    );
    assert_true(r#"! /[^[[\q{K|s}]--[s]]]/iv.test('KKk')&&/[^[[\q{K|s}]--[s]]]/iv.test('ſ!')"#);
    assert_true(r#"/[\q{Σ|µ}--ς]/iv.test('μ')&&!/[\q{Σ|µ}--ς]/iv.test('Σςσ')"#);
    assert_true(
        r#"
            let r = /[\q{\b|\!|\||\}|\/}&&[^\q{!}]]/v;
            r.test('\b') && r.test('|') && r.test('}') && r.test('/') && !r.test('!') && !r.test('b')
        "#,
    );
    assert_true(
        r#"
            /[\q{\]|\[|\}}x]/v.test(']') && /[\q{\]|\[|\}}x]/v.test('[') && /[\q{\]|\[|\}}x]/v.test('}') &&
                /[\q{\]|\[|\}}x]/v.test('x')
        "#,
    );
    assert_true(
        r#"
            /^\b[\q{K}&&k]\b$/ivm.test('x\nK\nz') && !/^\b[\q{K}&&k]\b$/vm.test('x\nK\nz') && !
                /^\b[\q{K}&&k]\b$/iv.test('x\nK\nz')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀!');
            m.index === 1 && m[0] === '😀' && m.indices[0][0] === 1 && m.indices[0][1] === 3 && r
                .lastIndex === 3
        "#,
    );
    assert_true(
        r#"/[\q{\uD800|\uDC00}--[\q{\uDC00}]]/v.exec('A\uD800B').index===1&&!/[\q{\uD800|\uDC00}--[\q{\uDC00}]]/v.test('😀')"#,
    );
    assert_true(
        r#"
            let trace = '',
                r = /[\q{a|b}&&[b]]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return 'xb'
                }
            });
            trace === 'si' && m[0] === 'b' && m.index === 1 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[\q{a|b}&&[b]]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            r.exec('xb').index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{K|s}--k]/iv;
            r.lastIndex = 7;
            let c = new RegExp(r, 'dvy');
            c.source === '[\\q{K|s}--k]' && c.flags === 'dvy' && c.lastIndex === 0 && c.exec('ſ') ===
                null && r.exec('ſ')[0] === 'ſ' && r.lastIndex === 7
        "#,
    );
    assert_true(
        r#"
            let a = /[\q{a|b}&&[b]]/vy,
                b = /[\q{a|b}&&[b]]/vy;
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
                a.exec('xb')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('x')
            } catch (e) {
                y = e instanceof TypeError
            }
            x && y && a.lastIndex === 1 && b.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([\q{a|b}&&[b]]))/dvg,
                a = [...'a-bb!'.matchAll(r)];
            a.length === 2 && a[0].index === 2 && a[1].index === 3 && a[1].groups.whole === 'b' && 'a-bb!'
                .replace(r, '<$<whole>>') === 'a-<b><b>!' && 'a-bb!'.split(/[\q{a|b}&&[b]]/v).join('|') ===
                'a-||!'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a|b}&&[b]]/vg;
            r.lastIndex = 1;
            'xab'.search(r) === 2 && r.lastIndex === 1 && 'abbb'.match(r).join(',') === 'b,b,b' && r
                .lastIndex === 0
        "#,
    );
    for pattern in [
        r"[^\q{a|ab}]",
        r"[\q{a}-b]",
        r"[a-\q{b}]",
        r"[\q{a}&&a--a]",
        r"[\q{\d}&&a]",
        r"[\q{!!}a]",
    ] {
        let code = JsString::from(format!("marker=1; /{pattern}/v.test('a');").as_str());
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
    for (source, expected) in [
        (r"/[^\q{ab}&&a]/v.test('a')", true),
        (r"/[[a--a]&&[\q{ab}]]/v.test('a')", false),
        (r"/[\q{a|b}]+/v.test('ab')", true),
        (r"/[[\q{a|b}]]{0}/v.test('')", true),
        (r"/a[\q{b}]/v.test('ab')", true),
        (r"/[a--a--\q{ab}]/v.test('a')", false),
        (r"/[\q{}&&a]/v.test('a')", false),
        (r"/[[\q{a|b}]--[\q{a|bc}]]/v.test('b')", true),
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
    {
        let source = r"/[\q{a}&&a]b/v.test('ab')";
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
        r"/[[\q{a|b}]&&[\p{RGI_Emoji}]]/v.test('a')",
        r"/\b^[\q{a}&&a]/v.test('a')",
        r"/[\q{a}&&a]$\b/v.test('a')",
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
        .eval(r#"var kept=/(?<whole>([\q{a|b}&&[b]]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}&&[b]]))'&&kept.lastIndex===1&&kept.exec('zb').groups.whole==='b'&&kept.lastIndex===2"#),Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[\q{z}&&z]/vg,marker=0"#).unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn single_character_v_string_operands_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original UTF16 spans were fixed before this Runtime source.
    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>([a\q{b|c}]))"#,
            JsString::from_code_units(vec![122, 98]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([\q{a|b}&&[b-z]]))"#,
            JsString::from_code_units(vec![97, 97, 98]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([\q{K|s}--k]))"#,
            JsString::from_code_units(vec![8490, 115]),
            "divg",
            0usize,
        ),
        (
            r#"(?<whole>([^[[\q{K|s}]--[s]]]))"#,
            JsString::from_code_units(vec![8490]),
            "divg",
            0usize,
        ),
        (
            r#"^(?<whole>(\b[\q{K}&&k]\b))$"#,
            JsString::from_code_units(vec![120, 10, 8490, 10, 122]),
            "dgimv",
            0usize,
        ),
        (
            r#"^(?<whole>(\b[\q{K}&&k]\b))$"#,
            JsString::from_code_units(vec![120, 10, 8490, 10, 122]),
            "dgmv",
            0usize,
        ),
        (
            r#"(?<whole>(\b[\q{\uD83D\uDE00|x}&&[^\q{x}]]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 33]),
            "dvy",
            2usize,
        ),
        (
            r#"(?<whole>([\q{\uD800|\uDC00}--[\q{\uDC00}]]))"#,
            JsString::from_code_units(vec![65, 55296, 66]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>([\q{\b|\!|\||\}|\/}&&[^\q{!}]]))"#,
            JsString::from_code_units(vec![33, 124]),
            "dvg",
            0usize,
        ),
        (
            r#"^(?<whole>(\B[[\q{\r|\n|\u2028}]--[\q{\r}]]\B))$"#,
            JsString::from_code_units(vec![120, 13, 10, 8232, 121]),
            "dgmv",
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
