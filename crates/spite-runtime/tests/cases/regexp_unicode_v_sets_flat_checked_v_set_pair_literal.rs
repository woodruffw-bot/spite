//! Match results and consumers for pairs of character classes and literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn flat_checked_v_set_pair_literal_state_context_and_consumers() {
    assert_true(
        r#"
            let r = /(?<whole>([a][[b]]c))/dvg,
                m = r.exec('aabc');
            m !== null && Array.isArray(m) && m.length === 3 && m[0] === 'abc' && m[1] === 'abc' && m[
                2] === 'abc' && m.index === 1 && m.input === 'aabc' && m.groups.whole === 'abc' && m
                .indices.groups.whole === m.indices[1] && m.indices[2][0] === 1 && m.indices[2][1] === 4 &&
                r.source === '(?<whole>([a][[b]]c))' && r.lastIndex === 4 && r.exec('aabc') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /[a][[a]]b/dvg,
                m = r.exec('aaab!');
            m !== null && m.index === 1 && m[0] === 'aab' && m.indices[0][1] === 4 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let r = /[a][[a]]a\b/dvg,
                m = r.exec('aaaa!');
            m !== null && m.index === 1 && m[0] === 'aaa' && m.indices[0][1] === 4 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let r = /[a][[a]]a\b/dvy;
            r.exec('aaaa!') === null && r.lastIndex === 0 && (r.lastIndex = 1, r.exec('aaaa!')[0] ===
                'aaa') && r.lastIndex === 4 && r.exec('aaaa!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /\b[a][[b]]c/v.test('abc') && /\B[a][[b]]c/v.test('xabc') && /[a][[b]]c\b/v.test('abc!') &&
                /[a][[b]]c\B/v.test('abcx') && /\b[a][[b]]c\b/v.test('!abc!') && /\b[a][[b]]c\B/v.test(
                    '!abcx') && /\B[a][[b]]c\b/v.test('xabc!') && /\B[a][[b]]c\B/v.test('xabcx') && !
                /\b[a][[b]]c/v.test('xabc') && !/[a][[b]]c\b/v.test('abcx') && !/\B[a][[b]]c/v.test(
                '!abc') && !/[a][[b]]c\B/v.test('abc!')
        "#,
    );
    assert_true(
        r#"
            /\b[K][[s]]a\b/iv.test('KſA!') && !/\b[K][[s]]a\b/v.test('Ksa!') && !/\b[K][[s]]a\b/v.test(
                    'Kſa!') && !/\b[K][[s]]a\b/v.test('KsA!') && /\b[K][[s]]a\b/v.test('Ksa!') && !
                /\b[K][[s]]a\b/iv.test('asa!') && !/\b[K][[s]]a\b/iv.test('Kxa!') && !/\b[K][[s]]a\b/iv
                .test('Ksb!')
        "#,
    );
    assert_true(
        r#"
            let a = /^\b[\p{Any}][[\p{Any}]]\$\B$/iv,
                b = /^\b[\p{Any}][[\p{Any}]]\$\B$/v;
            a.test('K!$') && a.test('ſ!$') && !b.test('K!$') && !b.test('ſ!$') && a.exec('K!$')[0] ===
                'K!$'
        "#,
    );
    assert_true(
        r#"
            /[][[\p{Any}]]c/v.exec('abc') === null && /[a][a--a]c/v.exec('abc') === null && /[^a][[b]]c/v
                .exec('xbc!')[0] === 'xbc' && !/[^a][[b]]c/v.test('abc!') && /[a][[^b]]c/v.exec('axc!')[
                0] === 'axc' && !/[a][[^b]]c/v.test('abc!')
        "#,
    );
    assert_true(
        r#"
            /[\P{Ll}][[s]]x/v.test('Asx') && !/[\P{Ll}][[s]]x/iv.test('Asx') && !/[\P{Ll}][[s]]x/iv.test(
                'asx') && /[\P{Ll}][[s]]x/iv.test('!sx')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\u{1f600}][[\u{1f601}]]\u{1f600}\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀😁😀!');
            m !== null && m.index === 1 && m[0] === '😀😁😀' && m.indices[0][0] === 1 && m.indices[0][
                1] === 7 && r.lastIndex === 7 && r.exec('A😀😁😀!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\uD800][[\uD800]]\uD800\B/dv,
                m = r.exec('A\uD800\uD800\uD800!');
            m !== null && m.index === 1 && m[0] === '\uD800\uD800\uD800' && m.indices[0][1] === 4 &&
                /[\uD83D][[\uDE00]]a/v.exec('😀a') === null && /[a][[\uD83D]]\uDE00/v.exec('a😀') === null
        "#,
    );
    assert_true(
        r#"
            /^\b[a][[\]]]\$\B$/v.test('a]$') && !/^\b[a][[\]]]\$\B$/v.test('a]$\n') &&
                /^[\u{61}][[\u{62}]]\u{63}$/v.test('abc') && /^[\u0061][[\u0062]]\u0063$/v.test('abc')
        "#,
    );
    assert_true(
        r#"
            let r = /^(?<whole>(\b[a][[b]]c\b))$/dvmg,
                m = r.exec('x\r\nabc\u2028axc');
            m !== null && m.index === 3 && m[0] === 'abc' && m.indices[2][0] === 3 && m.indices[2][1] ===
                6 && r.lastIndex === 6 && r.exec('x\r\nabc\u2028axc') === null && r.lastIndex === 0 && !
                /^\b[a][[b]]c\b$/v.test('abc\n')
        "#,
    );
    assert_true(
        r#"
            /\b[a][[b]]c/v.exec('abcx')[0] === 'abc' && /\b[a][[b]]c/v.exec('abcx').index === 0 &&
                /[a][[b]]c\b/v.exec('xabc').index === 1 && /[[a]][[b]]c/v.test('abc') && /\b[[a]][[b]]c\b/v
                .test('abc') && /[a]b[c]/v.test('abc') && /\ba[b]\b/v.test('ab') && /\b[a]b\b/v.test(
                'ab') && /[a][b][c]/v.test('abc')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([a][[b]]c\b))/dvg,
                a = [...'😀abc aabc'.matchAll(r)];
            a.length === 2 && a[0][0] === 'abc' && a[0][1] === 'abc' && a[0][2] === 'abc' && a[0].index ===
                2 && a[0].groups.whole === 'abc' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 5 && a[1].index === 7 && a[1].indices[2][1] === 10 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            '😀abc aabc'.replace(/(?<whole>([a][[b]]c\b))/vg, '<$1:$<whole>>') ===
                '😀<abc:abc> a<abc:abc>' && '😀abcx'.split(/([a][[b]]c)/v).join('|') === '😀|abc|x' &&
                'aabc'.match(/[a][[b]]c/v)[0] === 'abc'
        "#,
    );
    assert_true(
        r#"
            let seen = '',
                out = '😀abc aabc'.replace(/(?<whole>([a][[b]]c\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return '!'
                });
            out === '😀! a!' && seen === '2:abc;7:abc;'
        "#,
    );
    assert_true(r#"let r=/[a][[b]]c/vg;r.lastIndex=1;'😀aabc'.search(r)===3&&r.lastIndex===1"#);
    assert_true(
        r#"
            let trace = '',
                r = /[\p{Any}][[\u{1f601}]]a/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀😁a!'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀😁a' && r.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[a][[b]]c/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            let m = r.exec('abcabc');
            m.index === 0 && m[0] === 'abc' && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[K][[s]]a/iv,
                c = new RegExp(r, 'dvy');
            c.source === '[K][[s]]a' && c.unicodeSets && !c.ignoreCase && c.exec('KſA') === null && c
                .lastIndex === 0 && r.exec('KſA')[0] === 'KſA'
        "#,
    );
    assert_true(
        r#"
            let a = /[a][[b]]c/vy,
                b = /[a][[b]]c/vy;
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
                eval('marker=1; /[a][b&&]c/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r#"/[[a]]c[b]/v.test('acb')"#);
    assert_true(
        r#"
            let r = /[[a]]c[b]/v,
                m = r.exec('acb');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'acb' && m.index === 0 && m
                .input === 'acb' && m.groups === undefined && m.indices === undefined && r.source ===
                '[[a]]c[b]' && r.lastIndex === 0 && !r.test('xcb') && !r.test('axb') && !r.test('acx') && !
                r.test('ac') && r.exec('acbx')[0] === 'acb' && r.exec('acbx').index === 0 && r.exec('xacb')
                .index === 1
        "#,
    );
    assert_true(r#"/[a]c[[b]]/v.test('acb')"#);
    assert_true(
        r#"
            let r = /[a]c[[b]]/v,
                m = r.exec('acb');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'acb' && m.index === 0 && m
                .input === 'acb' && m.groups === undefined && m.indices === undefined && r.source ===
                '[a]c[[b]]' && r.lastIndex === 0 && !r.test('xcb') && !r.test('axb') && !r.test('acx') && !
                r.test('ac') && r.exec('acbx')[0] === 'acb' && r.exec('acbx').index === 0 && r.exec('xacb')
                .index === 1
        "#,
    );
    for source in [
        r#"/[a][[b]]cd/v.test('abcd')"#,
        r#"/[a][[b]][c]d/v.test('abcd')"#,
        r#"/a[a][[b]]c/v.test('aabc')"#,
        r#"/[a][[b]]c[d]/v.test('abcd')"#,
        r#"/([a])[[b]]c/v.test('abc')"#,
        r#"/[a]([[b]])c/v.test('abc')"#,
        r#"/[a][[b]](c)/v.test('abc')"#,
        r#"/[a]+[[b]]c/v.test('abc')"#,
        r#"/[a][[b]]?c/v.test('abc')"#,
        r#"/[a][[b]]c+/v.test('abc')"#,
        r#"/(?:[a][[b]]c)+/v.test('abc')"#,
        r#"/[a]\b[[b]]c/v.test('abc')"#,
        r#"/[a][[b]]\bc/v.test('abc')"#,
        r#"/\b^[a][[b]]c/v.test('abc')"#,
        r#"/[a][[b]]c$\b/v.test('abc')"#,
        r#"/\b\b[a][[b]]c/v.test('abc')"#,
        r#"/[][\p{RGI_Emoji}&&a]c/v.test('')"#,
        r#"/[][\q{b}&&b]c/v.test('')"#,
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>([a][[a]]a\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>([a][[a]]a\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/[a][[a]]c/vg,marker=0"#).unwrap();
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
fn flat_checked_v_set_pair_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Complete expected JSON and original UTF16 spans were hand-derived before execution.
    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>([a][[b]]c))"#,
            JsString::from_code_units(vec![97, 97, 98, 99]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([a][[a]]b))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([a][[a]]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>([a][[a]]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
        ),
        (
            r#"(?<whole>([a][[a]]a\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
        ),
        (
            r#"(?<whole>(\b[a][[b]]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 120]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>(\b[\u{1f600}][[\u{1f601}]]\u{1f600}\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
        ),
        (
            r#"(?<whole>(\b[\uD800][[\uD800]]\uD800\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
        ),
        (
            r#"^(?<whole>(\b[\p{Any}][[\p{Any}]]\$\B))$"#,
            JsString::from_code_units(vec![8490, 33, 36]),
            "divg",
            0,
        ),
        (
            r#"(?<whole>([][[\p{Any}]]c))"#,
            JsString::from_code_units(vec![97, 98, 99, 33]),
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
