//! Match results and consumers for single-character strings next to literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn literal_single_character_v_string_class_state_context_and_consumers() {
    assert_true(
        r#"/a[\q{b}]/v.exec('aab!').index===1&&!/a[\q{b}]/v.test('ax')&&!/a[\q{b}]/v.test('a')"#,
    );
    assert_true(
        r#"
            /a[a\q{b|c}]/v.test('ac') && /a[[\q{b|c}]d-f]/v.test('ae') && /a[\q{a|b}&&[b-z]]/v.test(
                'ab') && !/a[\q{a|b}--b]/v.test('ab') && !/a[\q{a}--a]/v.test('aa')
        "#,
    );
    assert_true(
        r#"
            let r = /\Ba[\q{a}]\b/v,
                m = r.exec('aaa aa!');
            m.index === 1 && m[0] === 'aa' && /\ba[\q{b}]\b/v.exec('xab ab!').index === 4
        "#,
    );
    assert_true(
        r#"
            let r = /a[\q{b}]/dvy;
            r.lastIndex = 0;
            let a = r.exec('aab');
            r.lastIndex = 1;
            let b = r.exec('aab');
            a === null && b.index === 1 && b[0] === 'ab' && b.indices[0][1] === 3 && r.lastIndex === 3 && r
                .exec('aab') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /a[\q{b}]/vg,
                a = r.exec('abab!'),
                b = r.exec('abab!');
            a.index === 0 && b.index === 2 && r.lastIndex === 4 && r.exec('abab!') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(r#"/s[\q{K}&&k]/iv.exec('!ſK!')[0]==='ſK'&&!/s[\q{K}&&k]/v.test('ſK')"#);
    assert_true(
        r#"let r=/K[^\q{K|s}]/iv,m=r.exec('KſK!');m.index===2&&m[0]==='K!'&&!/s[^\q{K|s}]/iv.test('ſK')"#,
    );
    assert_true(
        r#"
            let r = /\b\u{1f600}[\q{\uD83D\uDE01|x}--x]\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀😁!');
            m.index === 1 && m[0] === '😀😁' && m.indices[0][0] === 1 && m.indices[0][1] === 5 && r
                .lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let m = /\uD800[\q{\uD800|x}--x]/v.exec('A\uD800\uD800B');
            m.index === 1 && m[0].length === 2 && m[0].charCodeAt(0) === 0xD800 && m[0].charCodeAt(1) ===
                0xD800 && !/\uD800[\q{\uD800}]/v.test('😀')
        "#,
    );
    assert_true(
        r#"
            let r = /^\B\r[\q{\n|\u2028}]\B$/vm,
                m = r.exec('x\n\r\n\rz');
            m.index === 2 && m[0] === '\r\n' && !/^\B\r[\q{\n|\u2028}]\B$/v.test('x\n\r\n\rz')
        "#,
    );
    assert_true(r#"/^\B\$[\q{\]|\|}]\B$/v.test('$]')&&/^\B\$[\q{\]|\|}]\B$/v.test('$|')"#);
    assert_true(
        r#"
            let trace = '',
                r = /a[\q{b}]/vy;
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
                r = /a[\q{b}]/v;
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
            let r = /s[\q{K}&&k]/iv;
            r.lastIndex = 7;
            let c = new RegExp(r, 'dvy');
            c.source === 's[\\q{K}&&k]' && c.flags === 'dvy' && c.lastIndex === 0 && c.exec('ſK') ===
                null && r.exec('ſK')[0] === 'ſK' && r.lastIndex === 7
        "#,
    );
    assert_true(
        r#"
            let a = /a[\q{b}]/vy,
                b = /a[\q{b}]/vy;
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
            let r = /(?<whole>(a[\q{b}]))/dvg,
                a = [...'aab-ab!'.matchAll(r)];
            a.length === 2 && a[0].index === 1 && a[1].index === 4 && a[1].groups.whole === 'ab' && a[1]
                .indices[2][1] === 6 && 'aab-ab!'.replace(r, '<$<whole>>') === 'a<ab>-<ab>!' && 'aab-ab!'
                .split(/a[\q{b}]/v).join('|') === 'a|-|!'
        "#,
    );
    assert_true(
        r#"
            let r = /a[\q{b}]/vg;
            r.lastIndex = 4;
            'aab!'.search(r) === 1 && r.lastIndex === 4 && 'ab-ab'.match(r).join('|') === 'ab|ab' && r
                .lastIndex === 0
        "#,
    );
    for pattern in [
        r"a[\q{a}-b]",
        r"a[^\q{ab}]",
        r"a[\q{\d}]",
        r"a[\q{\_}]",
        r"\u{110000}[\q{b}]",
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
        let source = r"/[\q{a}]b/v.test('ab')";
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
        r"/a[\q{b}]c/v.test('abc')",
        r"/(a)[\q{b}]/v.test('ab')",
        r"/a([\q{b}])/v.test('ab')",
        r"/a[\q{b}]+/v.test('ab')",
        r"/a+[\q{b}]/v.test('ab')",
        r"/(a[\q{b}])+/v.test('ab')",
        r"/a[\q{}]/v.test('a')",
        r"/a[\q{a|bc}]/v.test('aa')",
        r"/a[\q{a}&&[]&&\q{bc}]/v.test('aa')",
        r"/a[\q{a}--a--\p{RGI_Emoji}]/v.test('aa')",
        r"/a[\q{b}]$$/v.test('ab')",
        r"/\b\ba[\q{b}]\b/v.test('ab')",
        r"/\ba[\q{b}]\b\b/v.test('ab')",
        r"/\b^a[\q{b}]/v.test('ab')",
        r"/a\b[\q{b}]/v.test('ab')",
        r"/a[\q{b}]$\b/v.test('ab')",
        r"/a[\q{b}]|c/v.test('ab')",
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
        .eval(r#"var kept=/(?<whole>(a[\q{a|b}&&b]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(a[\\q{a|b}&&b]))'&&kept.lastIndex===1&&kept.exec('aab!').groups.whole==='ab'&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/a[\q{z}]/vg,marker=0"#).unwrap();
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
fn literal_single_character_v_string_class_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete JSON and original UTF16 spans were frozen before Runtime source.
    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>(a[\q{b}]))"#,
            JsString::from_code_units(vec![97, 97, 98]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(\Ba[\q{a}]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 32, 97, 97, 33]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(s[\q{K}&&k]))"#,
            JsString::from_code_units(vec![33, 383, 8490, 33]),
            "divg",
            0usize,
        ),
        (
            r#"(?<whole>(K[^\q{K|s}]))"#,
            JsString::from_code_units(vec![8490, 383, 75, 33]),
            "divg",
            0usize,
        ),
        (
            r#"(?<whole>(\b\u{1f600}[\q{\uD83D\uDE01|x}--x]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 33]),
            "dvy",
            2usize,
        ),
        (
            r#"(?<whole>(\uD800[\q{\uD800|x}--x]))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 66]),
            "dvg",
            0usize,
        ),
        (
            r#"^(?<whole>(\B\r[\q{\n|\u2028}]\B))$"#,
            JsString::from_code_units(vec![120, 10, 13, 10, 13, 122]),
            "dgmv",
            0usize,
        ),
        (
            r#"^(?<whole>(\B\$[\q{\]|\|}]\B))$"#,
            JsString::from_code_units(vec![36, 93]),
            "dvg",
            0usize,
        ),
        (
            r#"(?<whole>(s[^\q{K|s}]))"#,
            JsString::from_code_units(vec![383, 8490]),
            "divg",
            0usize,
        ),
        (
            r#"(?<whole>(a[\q{b}]))"#,
            JsString::from_code_units(vec![97, 97, 98]),
            "dvy",
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
