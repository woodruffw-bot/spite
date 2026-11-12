//! Match results and consumers for character classes and literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_literal_flat_v_set_state_context_and_consumers() {
    assert_true(
        r#"
            let r = /(?<whole>([[a]]b[c]))/dvg,
                m = r.exec('aabc');
            m !== null && Array.isArray(m) && m.length === 3 && m[0] === 'abc' && m[1] === 'abc' && m[
                2] === 'abc' && m.index === 1 && m.input === 'aabc' && m.groups.whole === 'abc' && m
                .indices.groups.whole === m.indices[1] && m.indices[2][0] === 1 && m.indices[2][1] === 4 &&
                r.source === '(?<whole>([[a]]b[c]))' && r.lastIndex === 4 && r.exec('aabc') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /[[a]]a[b]/dvg,
                m = r.exec('aaab!');
            m !== null && m.index === 1 && m[0] === 'aab' && m.indices[0][1] === 4 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let r = /[[a]]a[a]\b/dvg,
                m = r.exec('aaaa!');
            m !== null && m.index === 1 && m[0] === 'aaa' && m.indices[0][1] === 4 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let r = /[[a]]a[a]\b/dvy;
            r.exec('aaaa!') === null && r.lastIndex === 0 && (r.lastIndex = 1, r.exec('aaaa!')[0] ===
                'aaa') && r.lastIndex === 4 && r.exec('aaaa!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /\b[[a]]b[c]/v.test('abc') && /\B[[a]]b[c]/v.test('xabc') && /[[a]]b[c]\b/v.test('abc!') &&
                /[[a]]b[c]\B/v.test('abcx') && /\b[[a]]b[c]\b/v.test('!abc!') && /\b[[a]]b[c]\B/v.test(
                    '!abcx') && /\B[[a]]b[c]\b/v.test('xabc!') && /\B[[a]]b[c]\B/v.test('xabcx') && !
                /\b[[a]]b[c]/v.test('xabc') && !/[[a]]b[c]\b/v.test('abcx') && !/\B[[a]]b[c]/v.test(
                '!abc') && !/[[a]]b[c]\B/v.test('abc!')
        "#,
    );
    assert_true(
        r#"
            /\b[[K]]s[a]\b/iv.test('KſA!') && !/\b[[K]]s[a]\b/v.test('Ksa!') && !/\b[[K]]s[a]\b/v.test(
                    'Kſa!') && !/\b[[K]]s[a]\b/v.test('KsA!') && /\b[[K]]s[a]\b/v.test('Ksa!') && !
                /\b[[K]]s[a]\b/iv.test('asa!') && !/\b[[K]]s[a]\b/iv.test('Kxa!') && !/\b[[K]]s[a]\b/iv
                .test('Ksb!')
        "#,
    );
    assert_true(
        r#"
            let a = /^\B[[\p{Any}]]\$[\p{Any}]\b$/iv,
                b = /^\B[[\p{Any}]]\$[\p{Any}]\b$/v;
            a.test('!$K') && a.test('!$ſ') && !b.test('!$K') && !b.test('!$ſ') && a.exec('!$K')[0] ===
                '!$K'
        "#,
    );
    assert_true(
        r#"
            /[a--a]b[c]/v.exec('abc') === null && /[[a]]b[]/v.exec('abc') === null && /[[^a]]b[c]/v.exec(
                    'xbc!')[0] === 'xbc' && !/[[^a]]b[c]/v.test('abc!') && /[[a]]b[^c]/v.exec('abx!')[
                0] === 'abx' && !/[[a]]b[^c]/v.test('abc!')
        "#,
    );
    assert_true(
        r#"
            /[[\P{Ll}]]b[c]/v.test('Abc') && !/[[\P{Ll}]]b[c]/iv.test('Abc') && !/[[\P{Ll}]]b[c]/iv.test(
                'abc') && /[[\P{Ll}]]b[c]/iv.test('!bc')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[\u{1f600}]]\u{1f601}[\u{1f600}]\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀😁😀!');
            m !== null && m.index === 1 && m[0] === '😀😁😀' && m.indices[0][0] === 1 && m.indices[0][
                1] === 7 && r.lastIndex === 7 && r.exec('A😀😁😀!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[\uD800]]\uD800[\uD800]\B/dv,
                m = r.exec('A\uD800\uD800\uD800!');
            m !== null && m.index === 1 && m[0] === '\uD800\uD800\uD800' && m.indices[0][1] === 4 &&
                /[[\uD83D]]\uDE00[a]/v.exec('😀a') === null && /[[a]]\uD83D[\uDE00]/v.exec('a😀') === null
        "#,
    );
    assert_true(
        r#"
            /^\B[[\]]]\$[!]\B$/v.test(']$!') && !/^\B[[\]]]\$[!]\B$/v.test(']$!\n') &&
                /^[[\u{61}]]\u{62}[\u{63}]$/v.test('abc') && /^[[\u0061]]\u0062[\u0063]$/v.test('abc')
        "#,
    );
    assert_true(
        r#"
            let r = /^(?<whole>(\b[[a]]b[c]\b))$/dvmg,
                m = r.exec('x\r\nabc\u2028abx');
            m !== null && m.index === 3 && m[0] === 'abc' && m.indices[2][0] === 3 && m.indices[2][1] ===
                6 && r.lastIndex === 6 && r.exec('x\r\nabc\u2028abx') === null && r.lastIndex === 0 && !
                /^\b[[a]]b[c]\b$/v.test('abc\n')
        "#,
    );
    assert_true(
        r#"
            /\b[[a]]b[c]/v.exec('abcx')[0] === 'abc' && /\b[[a]]b[c]/v.exec('abcx').index === 0 &&
                /[[a]]b[c]\b/v.exec('xabc').index === 1 && /[[a]]b[[c]]/v.test('abc') && /\b[[a]]b[[c]]\b/v
                .test('abc') && /[a]b[c]/v.test('abc') && /[[a]][b]c/v.test('abc') && /[a][[b]]c/v.test(
                    'abc') && /a[[b]][c]/v.test('abc') && /a[b][[c]]/v.test('abc')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([[a]]b[c]\b))/dvg,
                a = [...'😀abc aabc'.matchAll(r)];
            a.length === 2 && a[0][0] === 'abc' && a[0][1] === 'abc' && a[0][2] === 'abc' && a[0].index ===
                2 && a[0].groups.whole === 'abc' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 5 && a[1].index === 7 && a[1].indices[2][1] === 10 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            '😀abc aabc'.replace(/(?<whole>([[a]]b[c]\b))/vg, '<$1:$<whole>>') ===
                '😀<abc:abc> a<abc:abc>' && '😀abcx'.split(/([[a]]b[c])/v).join('|') === '😀|abc|x' &&
                'aabc'.match(/[[a]]b[c]/v)[0] === 'abc'
        "#,
    );
    assert_true(
        r#"
            let seen = '',
                out = '😀abc aabc'.replace(/(?<whole>([[a]]b[c]\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return '!'
                });
            out === '😀! a!' && seen === '2:abc;7:abc;'
        "#,
    );
    assert_true(r#"let r=/[[a]]b[c]/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    assert_true(
        r#"
            let trace = '',
                r = /[[\p{Any}]]\$[\p{Any}]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀$K'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀$K' && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[[a]]b[c]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            let m = r.exec('aabc');
            m.index === 1 && m[0] === 'abc' && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[[K]]s[a]/iv,
                c = new RegExp(r, 'dvy');
            c.source === '[[K]]s[a]' && c.unicodeSets && !c.ignoreCase && c.exec('KſA') === null && c
                .lastIndex === 0 && r.exec('KſA')[0] === 'KſA'
        "#,
    );
    assert_true(
        r#"
            let a = /[[a]]b[c]/vy,
                b = /[[a]]b[c]/vy;
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
                a.exec('aabc')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('xabc')
            } catch (e) {
                y = e instanceof TypeError
            }
            x && y && a.lastIndex === 1 && b.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let marker = 0,
                caught = false;
            try {
                eval('marker=1; /[a&&]b[c]/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r#"/[a]b[[c]]/v.test('abc')"#);
    assert_true(
        r#"
            let r = /[a]b[[c]]/v,
                m = r.exec('abc');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'abc' && m.index === 0 && m
                .input === 'abc' && m.groups === undefined && m.indices === undefined && r.source ===
                '[a]b[[c]]' && r.lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !
                r.test('ab') && r.exec('abcx')[0] === 'abc' && r.exec('abcx').index === 0 && r.exec('xabc')
                .index === 1
        "#,
    );
    for source in [
        r#"/[[a]]b[c]d/v.test('abcd')"#,
        r#"/x[[a]]b[c]/v.test('xabc')"#,
        r#"/[[a]]b[c][d]/v.test('abcd')"#,
        r#"/([[a]])b[c]/v.test('abc')"#,
        r#"/[[a]](b)[c]/v.test('abc')"#,
        r#"/[[a]]b([c])/v.test('abc')"#,
        r#"/(?:[[a]]b[c])+/v.test('abc')"#,
        r#"/[[a]]+b[c]/v.test('abc')"#,
        r#"/[[a]]b[c]?/v.test('abc')"#,
        r#"/[[a]]\bb[c]/v.test('abc')"#,
        r#"/[[a]]b\b[c]/v.test('abc')"#,
        r#"/\b^[[a]]b[c]/v.test('abc')"#,
        r#"/[[a]]b[c]$\b/v.test('abc')"#,
        r#"/\b\b[[a]]b[c]/v.test('abc')"#,
        r#"/[[a]]b[c]\b\b/v.test('abc')"#,
        r#"/[\q{a}&&a]b[]/v.test('')"#,
        r#"/[a--a]b[\p{RGI_Emoji}]/v.test('')"#,
        r#"/[a--a]b[\q{c}]/v.test('')"#,
        r#"/[\p{RGI_Emoji}&&a]b[]/v.test('')"#,
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([[a]]a[a]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([[a]]a[a]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[[a]]a[c]/vg,marker=0"#).unwrap();
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
fn checked_v_set_literal_flat_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original UTF16 spans were hand-derived before Runtime source.
    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>([[a]]b[c]))"#,
            JsString::from_code_units(vec![97, 97, 98, 99]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([[a]]a[b]))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([[a]]a[a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([[a]]a[a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
        ),
        (
            r#"(?<whole>([[a]]a[a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
        ),
        (
            r#"(?<whole>(\b[[a]]b[c]))"#,
            JsString::from_code_units(vec![97, 98, 99, 120]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>(\b[[\u{1f600}]]\u{1f601}[\u{1f600}]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
        ),
        (
            r#"(?<whole>(\b[[\uD800]]\uD800[\uD800]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
        ),
        (
            r#"^(?<whole>(\B[[\p{Any}]]\$[\p{Any}]\b))$"#,
            JsString::from_code_units(vec![33, 36, 8490]),
            "divg",
            0,
        ),
        (
            r#"(?<whole>([a--a]c[\p{Any}]))"#,
            JsString::from_code_units(vec![97, 99, 98, 33]),
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
