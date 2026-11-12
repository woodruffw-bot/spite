//! Match results and consumers for character classes and literal text next to word assertions.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn flat_v_set_class_literal_word_state_context_and_consumers() {
    assert_true(r#"/\ba[b]\b/v.test('ab')"#);
    assert_true(
        r#"
            let r = /\ba[b]\b/v,
                m = r.exec('ab');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'ab' && m.index === 0 && m
                .input === 'ab' && m.groups === undefined && m.indices === undefined && r.source ===
                '\\ba[b]\\b' && r.lastIndex === 0 && r.exec('xb') === null && r.exec('ac') === null && r
                .exec('a') === null && r.exec('xab') === null && r.exec('abx') === null && r.exec('!ab!')
                .index === 1
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\B[a]a\b))/dvg,
                m = r.exec('aaa!');
            m !== null && m.length === 3 && m[0] === 'aa' && m[1] === 'aa' && m[2] === 'aa' && m.index ===
                1 && m.input === 'aaa!' && m.groups.whole === 'aa' && m.indices.groups.whole === m.indices[
                    1] && m.indices[2][0] === 1 && m.indices[2][1] === 3 && r.lastIndex === 3 && r.exec(
                    'aaa!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\B[a]a\b/dvy;
            r.exec('aaa!') === null && r.lastIndex === 0 && (r.lastIndex = 1, r.exec('aaa!')[0] ===
                'aa') && r.lastIndex === 3 && r.exec('aaa!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /\b[\p{Any}]s\b/iv.test('Kſ!') && !/\b[\p{Any}]s\b/v.test('Kſ!') && /\b[K]s\b/iv.exec('Kſ!')[
                0] === 'Kſ' && !/\b[K]s\b/v.test('Ks!') && !/\b[K]s\b/v.test('Kſ!') && !/\b[K]s\b/iv.test(
                    'Kas!')
        "#,
    );
    assert_true(
        r#"
            /\b[\P{Ll}]a\b/v.test('Aa!') && !/\b[\P{Ll}]a\b/iv.test('Aa!') && !/\B[]b\B/v.test('ab') && !
                /\B[]b\B/v.test('') && /\b[^a]b\b/v.test('xb!') && !/\b[^a]b\b/v.test('ab!')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\p{Any}]\u{1f601}\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀😁!');
            m !== null && m.index === 1 && m[0] === '😀😁' && m.indices[0][0] === 1 && m.indices[0][1] ===
                5 && r.lastIndex === 5 && r.exec('A😀😁!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\uD800]\uD800\B/dv,
                m = r.exec('A\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800' && m.indices[0][1] === 3 && /\b[\uD83D]\uDE00\B/v
                .exec('A😀!') === null
        "#,
    );
    assert_true(
        r#"
            /\b[\]]\$\B/v.exec('x]$!')[0] === ']$' && /^\b[\u{61}]\u{62}\b$/v.test('ab') &&
                /^\b[\u0061]\u0062\b$/v.test('ab') && !/^\b[a]b\b$/v.test('ab\n')
        "#,
    );
    assert_true(
        r#"
            let r = /^(?<whole>(\b[a]b\b))$/dvmg,
                m = r.exec('x\r\nab\u2028ac');
            m.index === 3 && m[0] === 'ab' && m.indices[2][0] === 3 && m.indices[2][1] === 5 && r
                .lastIndex === 5 && r.exec('x\r\nab\u2028ac') === null && r.lastIndex === 0 && /^\b[a]b\b/v
                .test('ab!') && /\b[a]b\b$/v.test('!ab')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\B[a]a\b))/dvg,
                a = [...'aaa! aaa!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aa' && a[0][1] === 'aa' && a[0][2] === 'aa' && a[0].index ===
                1 && a[0].groups.whole === 'aa' && a[0].indices.groups.whole === a[0].indices[1] && a[1]
                .index === 6 && a[1].indices[2][1] === 8 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            'aaa! aaa!'.replace(/(?<whole>(\B[a]a\b))/vg, '<$1:$<whole>>') === 'a<aa:aa>! a<aa:aa>!' &&
                'aaa!'.split(/(\B[a]a\b)/v).join('|') === 'a|aa|!' && 'aaa!'.match(/\B[a]a\b/v)[0] === 'aa'
        "#,
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaa! aaa!'.replace(/(?<whole>(\B[a]a\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:aa;6:aa;'
        "#,
    );
    assert_true(r#"let r=/\B[a]a\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4"#);
    assert_true(
        r#"
            let trace = '',
                r = /\b[\p{Any}]\u{1f601}\B/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 2
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return 'A😀😁'
                }
            });
            trace === 'si' && m.index === 1 && m[0] === '😀😁' && r.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /\B[a]a\b/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aaa!');
            m.index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /\b[K]s\b/iv,
                c = new RegExp(r, 'dvy');
            c.source === '\\b[K]s\\b' && c.unicodeSets && !c.ignoreCase && c.exec('Kſ') === null && c
                .lastIndex === 0 && r.exec('Kſ')[0] === 'Kſ'
        "#,
    );
    assert_true(
        r#"
            let a = /\B[a]a\b/vy,
                b = /\B[a]a\b/vy;
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
                a.exec('aaa!')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('aaa!')
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
                eval('marker=1; /\\B[]\\u{110000}\\B/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(
        r#"
            let marker = 0,
                result, caught = false;
            try {
                result = eval("marker=1; /\\b[a]b\\b/v.test('ab')")
            } catch {
                caught = true
            } finally {
                marker = 3
            }!caught && marker === 3 && result && !/\b[a]b\b/v.test('xb') && !/\b[a]b\b/v.test('ax') && !
                /\b[a]b\b/v.test('a') && !/\b[a]b\b/v.test('abx')
        "#,
    );
    assert_true(
        r#"
            /\b[a]b/v.exec('abcx')[0] === 'ab' && /\b[a]b/v.exec('!ab').index === 1 && !/\b[a]b/v.test(
                    'xab') && /[a]b\b/v.exec('xab').index === 1 && !/[a]b\b/v.test('abx') && /\b[[a]]b\b/v
                .test('ab') && /\ba[[b]]\b/v.test('ab')
        "#,
    );
    assert_true(r#"/\b[a]b[c]\b/v.test('abc')"#);
    assert_true(
        r#"
            let r = /\b[a]b[c]\b/v,
                m = r.exec('abc');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'abc' && m.index === 0 && m
                .input === 'abc' && m.groups === undefined && m.indices === undefined && r.source ===
                '\\b[a]b[c]\\b' && r.lastIndex === 0 && r.exec('xbc') === null && r.exec('axc') === null &&
                r.exec('abd') === null && r.exec('ab') === null && r.exec('xabc') === null && r.exec(
                    'abcx') === null && r.exec('!abc!').index === 1
        "#,
    );
    assert_true(r#"/\b[\q{a}]b\b/v.test('ab')"#);
    for source in [
        r#"/\b[a]bc\b/v.test('abc')"#,
        r#"/\bx[a]b\b/v.test('xab')"#,
        r#"/\b([a])b\b/v.test('ab')"#,
        r#"/\b[a](b)\b/v.test('ab')"#,
        r#"/(\b[a]b\b)+/v.test('ab')"#,
        r#"/\b[a]+b\b/v.test('ab')"#,
        r#"/\b[a]b+\b/v.test('ab')"#,
        r#"/\b[a]b?\b/v.test('ab')"#,
        r#"/\b[a]b\b|c/v.test('ab')"#,
        r#"/\b[a]\bb\b/v.test('ab')"#,
        r#"/\b^[a]b\b/v.test('ab')"#,
        r#"/\b[a]b$\b/v.test('ab')"#,
        r#"/\b\b[a]b\b/v.test('ab')"#,
        r#"/\b[a]b\b\b/v.test('ab')"#,
        r#"/\b[\p{RGI_Emoji}]b\b/v.test('ab')"#,
        r#"/\b[]\p{RGI_Emoji}\b/v.test('')"#,
        r#"/\b[a]b\b/u.test('ab')"#,
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>(\B[a]a\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"let m=kept.exec('aaa!');m.index===1&&m.groups.whole==='aa'&&m.indices[2][1]===3&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r#"var r=/\B[a]b\b/vg,sticky=/[a]a\B/vy,marker=0"#)
        .unwrap();
    // Each flat class succeeds and the suffix literal fails; cancellation is explicitly opted in.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r#"sticky.test(text)&&sticky.lastIndex===2"#),
        Ok(Value::Boolean(true))
    );
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
fn flat_v_set_class_literal_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original spans are independently hand-derived finite expectations.
    for (pattern, input, flags, start) in [
        (r#"(?<whole>(\B[a]a\b))"#, JsString::from("aaa!"), "dvg", 0),
        (r#"(?<whole>(\B[a]a\b))"#, JsString::from("aaa!"), "dvy", 0),
        (r#"(?<whole>(\B[a]a\b))"#, JsString::from("aaa!"), "dvy", 1),
        (
            r#"(?<whole>(\b[\p{Any}]s\b))"#,
            JsString::from("Kſ!"),
            "divg",
            0,
        ),
        (
            r#"(?<whole>(\b[\P{Ll}]a\b))"#,
            JsString::from("Aa!"),
            "divg",
            0,
        ),
        (
            r#"(?<whole>(\b[\p{Any}]\u{1f601}\B))"#,
            JsString::from("A😀😁!"),
            "dvy",
            2,
        ),
        (
            r#"(?<whole>(\b[\uD800]\uD800\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 33]),
            "dvg",
            0,
        ),
        (
            r#"^(?<whole>(\b[a]b\b))$"#,
            JsString::from("x\r\nab ac"),
            "dvmg",
            0,
        ),
        (
            r#"(?<whole>(\b[\]]\$\B))"#,
            JsString::from("x]$!"),
            "dvg",
            0,
        ),
        (
            r#"^(?<whole>(\b[\u{61}]\u{62}\b))$"#,
            JsString::from("ab"),
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
