//! Match results and consumers for single-character string alternatives in Unicode sets.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn single_character_v_strings_state_context_and_consumers() {
    assert_true(r#"/[\q{a|b}]/v.test('a')&&/[\q{a|b}]/v.test('b')&&!/[\q{a|b}]/v.test('c')"#);
    assert_true(
        r#"! /[^\q{a|b}]/v.test('ab')&&/[^\q{a|b}]/v.test('c')&&/[^\q{a|b}]/v.exec('😀')[0]==='😀'"#,
    );
    assert_true(
        r#"let r=/[\q{K|s}]/iv;r.test('K')&&r.test('ſ')&&r.test('k')&&!r.test('!')&&!/[\q{K|s}]/v.test('K')"#,
    );
    assert_true(
        r#"! /[^\q{K|s}]/iv.test('Kſ')&&/[^\q{K|s}]/iv.test('!')&&/[^\q{K|s}]/v.test('K')"#,
    );
    assert_true(
        r#"/^\b[\q{\u212A}]\b$/iv.test('K')&&!/^\b[\q{\u212A}]\b$/v.test('K')&&/^[\q{\u212A}]$/v.test('K')"#,
    );
    assert_true(
        r#"
            /[\q{a}]/v.test('a') && /\b[\q{a}]/v.test('a') && /[\q{a}]\b/v.test('a') && /\b[\q{a}]\b/v
                .test('a') && /\B[\q{a}]/v.test('xa') && /\B[\q{a}]\b/v.test('xa!') && /\b[\q{a}]\B/v.test(
                    'ax') && /\B[\q{a}]\B/v.test('xax')
        "#,
    );
    assert_true(
        r#"
            let r = /^\B[\q{\r|\n|\u2028}]\B$/mv,
                m = r.exec('x\r\n\u2028y');
            m !== null && m.index === 2 && m[0] === '\n' && !/^\B[\q{\r|\n|\u2028}]\B$/v.test(
                'x\r\n\u2028y')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\q{\uD83D\uDE00|x}]\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀!');
            m !== null && m.index === 1 && m[0] === '😀' && m.indices[0][0] === 1 && m.indices[0][1] ===
                3 && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a}]/vy;
            r.lastIndex = 0;
            let a = r.exec('xa');
            r.lastIndex = 1;
            let b = r.exec('xa');
            a === null && b[0] === 'a' && b.index === 1 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"/[\q{😀|x}]/v.exec('😀')[0]==='😀'&&/[\q{\uD83D\uDE00|x}]/v.exec('😀')[0]==='😀'&&/[\q{\u{1f600}|x}]/v.exec('😀')[0]==='😀'"#,
    );
    assert_true(
        r#"/[\q{\uD800|\uDC00}]/v.exec('A\uD800B').index===1&&/[\q{\u{D800}}]/v.exec('\uD800')[0]==='\uD800'&&!/[\q{\uD800|\uDC00}]/v.test('😀')"#,
    );
    assert_true(
        r#"
            let r = /[\q{\b|\!|\||\}|\/}]/v;
            r.test('\b') && r.test('!') && r.test('|') && r.test('}') && r.test('/') && !r.test('b')
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{\cA|\0|\x61|\u0062}]/v;
            r.test('\x01') && r.test('\0') && r.test('a') && r.test('b') && !r.test('c')
        "#,
    );
    assert_true(
        r#"let r=/[\q{a|a|b|a}]/dgv;r.flags==='dgv'&&r.unicodeSets&&!r.unicode&&r.exec('b')[0]==='b'"#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([\q{a|b}]))/dgv,
                m = r.exec('za!');
            Array.isArray(m) && m.length === 3 && m[0] === 'a' && m[1] === 'a' && m[2] === 'a' && m
                .index === 1 && m.input === 'za!' && m.groups.whole === 'a' && m.indices[0][0] === 1 && m
                .indices[0][1] === 2 && m.indices.groups.whole[0] === 1 && m.indices.groups.whole[1] ===
                2 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let trace = '',
                r = /[\q{a|b}]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return 'xa'
                }
            });
            trace === 'si' && m[0] === 'a' && m.index === 1 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[\q{a|b}]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            r.exec('za').index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{K|s}]/iv;
            r.lastIndex = 7;
            let c = new RegExp(r, 'dvy');
            c.source === '[\\q{K|s}]' && c.flags === 'dvy' && c.lastIndex === 0 && c.exec('K') === null &&
                r.exec('K')[0] === 'K' && r.lastIndex === 7
        "#,
    );
    assert_true(
        r#"
            let a = /[\q{a}]/vy,
                b = /[\q{a}]/vy;
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
                a.exec('xa')
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
            let r = /(?<whole>([\q{a|b}]))/dgv,
                a = [...'a-b!'.matchAll(r)];
            a.length === 2 && a[0].index === 0 && a[1].index === 2 && a[1].groups.whole === 'b' && 'a-b!'
                .replace(r, '<$<whole>>') === '<a>-<b>!' && 'xa-bz'.split(/[\q{a|b}]/v).join('|') ===
                'x|-|z'
        "#,
    );
    assert_true(
        r#"
            let r = /[\q{a|b}]/vg;
            r.lastIndex = 1;
            'za'.search(r) === 1 && r.lastIndex === 1 && 'ab'.match(r).join(',') === 'a,b' && r
                .lastIndex === 0
        "#,
    );
    for pattern in [
        r#"[^\q{}]"#,
        r#"[^\q{a|ab}]"#,
        r#"[\q{\d}]"#,
        r#"[\q{\_}]"#,
        r#"[\q{a]"#,
    ] {
        assert_true(&format!(
            r#"
                let caught = false;
                try {{
                    new RegExp({pattern:?}, 'v')
                }} catch (e) {{
                    caught = e instanceof SyntaxError
                }}
                caught
            "#
        ));
    }
    for (source, expected) in [
        (r#"/[\q{a}b]/v.test('a')"#, true),
        (r#"/[[\q{a}]]/v.test('a')"#, true),
        (r#"/[\q{a}&&a]/v.test('a')"#, true),
        (r#"/[^\q{a}&&a]/v.test('a')"#, false),
        (r#"/[\q{a}\q{b}]/v.test('a')"#, true),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
    }
    assert_true(r#"/[\q{a|b}]+/v.test('ab')"#);
    assert_true(r#"/[\q{a}][b]/v.test('ab')"#);
    for (source, expected) in [
        (r#"/[\q{}]/v.test('')"#, true),
        (r#"/[\q{a|}]/v.test('a')"#, true),
        (r#"/[\q{|a}]/v.test('a')"#, true),
        (r#"/[\q{a|b|ab}]/v.test('a')"#, true),
        (r#"/[\q{ab|c}]/v.test('ab')"#, true),
        (r#"/[\q{\u{D800}\u{DC00}}]/v.test('\uD800\uDC00')"#, false),
    ] {
        assert_eq!(
            Realm::default().eval(source),
            Ok(Value::Boolean(expected)),
            "{source}"
        );
    }
    for source in [
        r#"/([\q{a}])b/v.test('ab')"#,
        r#"/\b^[\q{a}]/v.test('a')"#,
        r#"/[\q{a}]$\b/v.test('a')"#,
        r#"/[\p{RGI_Emoji}]/v.test('😀')"#,
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([\q{a|b}]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([\\q{a|b}]))'&&kept.lastIndex===1&&kept.exec('za').groups.whole==='a'&&kept.lastIndex===2"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[\q{z}]/vg,marker=0"#).unwrap();
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
fn single_character_v_strings_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON/original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>([\q{a|b}]))"#,
            JsString::from_code_units(vec![122, 98]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([^\q{a|b}]))"#,
            JsString::from_code_units(vec![97, 98, 55357, 56832]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([\q{K|s}]))"#,
            JsString::from_code_units(vec![8490]),
            "divg",
            0,
        ),
        (
            r#"(?<whole>([^\q{K|s}]))"#,
            JsString::from_code_units(vec![8490, 383]),
            "divg",
            0,
        ),
        (
            r#"^(?<whole>(\b[\q{\u212A}]\b))$"#,
            JsString::from_code_units(vec![8490]),
            "divg",
            0,
        ),
        (
            r#"^(?<whole>(\b[\q{\u212A}]\b))$"#,
            JsString::from_code_units(vec![8490]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>(\b[\q{\uD83D\uDE00|x}]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 33]),
            "dvy",
            2,
        ),
        (
            r#"(?<whole>([\q{\uD800|\uDC00}]))"#,
            JsString::from_code_units(vec![65, 55296, 66]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([\q{\b|\!|\||\}|\/}]))"#,
            JsString::from_code_units(vec![120, 8]),
            "dvg",
            0,
        ),
        (
            r#"^(?<whole>(\B[\q{\r|\n|\u2028}]\B))$"#,
            JsString::from_code_units(vec![120, 13, 10, 8232, 121]),
            "dgmv",
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
