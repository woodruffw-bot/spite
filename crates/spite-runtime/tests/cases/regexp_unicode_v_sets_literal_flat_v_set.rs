//! Match results and consumers for character classes and literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn literal_flat_v_set_state_context_and_consumers() {
    assert_true(r#"/a[\q{b}]/v.test('ab')"#);
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
            let r = /(?<whole>(a[b]))/dvg,
                m = r.exec('aab!');
            m !== null && Array.isArray(m) && m.length === 3 && m[0] === 'ab' && m[1] === 'ab' && m[2] ===
                'ab' && m.index === 1 && m.input === 'aab!' && m.groups.whole === 'ab' && m.indices.groups
                .whole === m.indices[1] && m.indices[2][0] === 1 && m.indices[2][1] === 3 && r.source ===
                '(?<whole>(a[b]))' && r.lastIndex === 3 && r.exec('aab!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /a[b]/dvy;
            r.exec('aab!') === null && r.lastIndex === 0 && (r.lastIndex = 1, r.exec('aab!')[0] ===
                'ab') && r.lastIndex === 3 && r.exec('aab!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /s[K]/iv,
                m = r.exec('ſK!');
            m !== null && m[0] === 'ſK' && m.index === 0 && r.test('sk') && !/s[K]/v.test('ſK') && !/s[K]/v
                .test('sK') && !r.test('aK') && !r.test('sa')
        "#,
    );
    assert_true(
        r#"
            !/a[]/v.test('aa!') && !/a[]/v.test('') && /a[^b]/v.test('aX!') && !/a[^b]/v.test('ab!') &&
                /a[b]/v.test('abx') && /a[b]/v.exec('xab').index === 1
        "#,
    );
    assert_true(
        r#"
            let r = /\u{1f600}[\p{Any}]/dvy;
            r.lastIndex = 1;
            let a = r.exec('😀😁😀😁!'),
                b = r.exec('😀😁😀😁!');
            a.index === 0 && a[0] === '😀😁' && a.indices[0][1] === 4 && b.index === 4 && b[0] ===
                '😀😁' && r.lastIndex === 8 && r.exec('😀😁😀😁!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\uD800[\uD800]/dv,
                m = r.exec('A\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800' && m.indices[0][1] === 3 && /\uD83D[\uDE00]/v.exec(
                '😀') === null
        "#,
    );
    assert_true(
        r#"
            /\$[\]]/v.exec('x$]!')[0] === '$]' && /^\u{61}[\u{62}]$/v.test('ab') && /^\u0061[\u0062]$/v
                .test('ab') && !/^\$[\]]$/v.test('$]\n')
        "#,
    );
    assert_true(
        r#"
            let r = /^(?<whole>(a[b]))$/dvmg,
                m = r.exec('x\r\nab\u2028ac');
            m.index === 3 && m[0] === 'ab' && m.indices[2][0] === 3 && m.indices[2][1] === 5 && r
                .lastIndex === 5 && r.exec('x\r\nab\u2028ac') === null && r.lastIndex === 0 && !/^a[b]$/v
                .test('ab\n') && /^a[b]/v.test('ab!') && /a[b]$/v.test('xab')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(a[b]))/dvg,
                a = [...'aab! aab!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'ab' && a[0][1] === 'ab' && a[0][2] === 'ab' && a[0].index ===
                1 && a[0].groups.whole === 'ab' && a[0].indices.groups.whole === a[0].indices[1] && a[1]
                .index === 6 && a[1].indices[2][1] === 8 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"'aab! aab!'.replace(/(?<whole>(a[b]))/vg,'<$1:$<whole>>')==='a<ab:ab>! a<ab:ab>!'&&'aab!'.split(/(a[b])/v).join('|')==='a|ab|!'&&'aab!'.match(/a[b]/v)[0]==='ab'"#,
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aab! aab!'.replace(/(?<whole>(a[b]))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:ab;6:ab;'
        "#,
    );
    assert_true(r#"let r=/a[b]/vg;r.lastIndex=4;'aab!'.search(r)===1&&r.lastIndex===4"#);
    assert_true(
        r#"
            let trace = '',
                r = /\u{1f600}[\p{Any}]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀😁!'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀😁' && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /a[b]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aab!');
            m.index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /s[K]/iv,
                c = new RegExp(r, 'dvy');
            c.source === 's[K]' && c.unicodeSets && !c.ignoreCase && c.exec('ſK') === null && c
                .lastIndex === 0 && r.exec('ſK')[0] === 'ſK'
        "#,
    );
    assert_true(
        r#"
            let a = /a[b]/vy,
                b = /a[b]/vy;
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
            let marker = 0,
                caught = false;
            try {
                eval('marker=1; /\\u{110000}[]/v;')
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
                result = eval("marker=1; /a[b]/v.test('ab')")
            } catch {
                caught = true
            } finally {
                marker = 3
            }!caught && marker === 3 && result && !/a[b]/v.test('xb') && !/a[b]/v.test('ac') && !/a[b]/v
                .test('a') && /a[[b]]/v.test('ab') && /a[b--c]/v.test('ab') && /[a]b/v.test('ab')
        "#,
    );
    assert_true(
        r#"
            /s[\p{Any}]/iv.exec('ſK')[0] === 'ſK' && !/s[\p{Any}]/v.test('ſK') && /s[\p{Any}]/iv.exec(
                'ſ😁')[0] === 'ſ😁' && !/s[\p{Any}]/iv.test('ſ')
        "#,
    );
    assert_true(r#"/a[b][c]/v.test('abc')"#);
    assert_true(
        r#"
            let r = /a[b][c]/v,
                m = r.exec('abc');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'abc' && m.index === 0 && m
                .input === 'abc' && m.groups === undefined && m.indices === undefined && r.source ===
                'a[b][c]' && r.lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r
                .test('ab') && r.test('xabc') && r.test('abcx')
        "#,
    );
    for source in [
        r#"/a\b[b]/v.test('ab')"#,
        r#"/a^[b]/v.test('ab')"#,
        r#"/a$[b]/v.test('ab')"#,
        r#"/ab[c]/v.test('abc')"#,
        r#"/xa[b]/v.test('xab')"#,
        r#"/a[b]c/v.test('abc')"#,
        r#"/a+[b]/v.test('ab')"#,
        r#"/a?[b]/v.test('ab')"#,
        r#"/a[b]+/v.test('ab')"#,
        r#"/a[b]?/v.test('ab')"#,
        r#"/(a)[b]/v.test('ab')"#,
        r#"/a([b])/v.test('ab')"#,
        r#"/(a[b])+/v.test('ab')"#,
        r#"/a[b]|c/v.test('ab')"#,
        r#"/a[\p{RGI_Emoji}]/v.test('')"#,
        r#"/\p{RGI_Emoji}[]/v.test('')"#,
        r#"/a[b]/u.test('ab')"#,
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }

    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>(a[b]))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"let m=kept.exec('aab!');m.index===1&&m.groups.whole==='ab'&&m.indices[2][1]===3&&kept.lastIndex===3"#), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r#"var r=/a[b]/vg,sticky=/a[a]/vy,marker=0"#)
        .unwrap();
    // Literal candidates succeed while the flat suffix class fails; cancellation is explicitly opted in.
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
fn literal_flat_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full JSON and original UTF16 spans are independently hand-derived finite expectations.
    for (pattern, input, flags, start) in [
        (r#"(?<whole>(a[b]))"#, JsString::from("aab!"), "dvg", 0),
        (r#"(?<whole>(a[b]))"#, JsString::from("aab!"), "dvy", 0),
        (r#"(?<whole>(a[b]))"#, JsString::from("aab!"), "dvy", 1),
        (r#"(?<whole>(s[K]))"#, JsString::from("ſK!"), "divg", 0),
        (
            r#"^(?<whole>(a[b]))$"#,
            JsString::from("x\r\nab ac"),
            "dvmg",
            0,
        ),
        (r#"(?<whole>(a[]))"#, JsString::from("aa!"), "dvg", 0),
        (r#"(?<whole>(a[^b]))"#, JsString::from("aX!"), "dvg", 0),
        (
            r#"(?<whole>(\u{1f600}[\p{Any}]))"#,
            JsString::from("A😀😁!"),
            "dvy",
            2,
        ),
        (
            r#"(?<whole>(\uD800[\uD800]))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 33]),
            "dvg",
            0,
        ),
        (r#"^(?<whole>(\$[\]]))$"#, JsString::from("$]"), "dvg", 0),
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
