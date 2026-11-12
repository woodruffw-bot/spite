//! Match results and consumers for pairs of character classes next to word assertions.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn flat_then_checked_v_set_pair_word_state_context_and_consumers() {
    assert_true(
        r#"
            /\b[a][b--c]/v.test('ab') && !/\b[a][b--c]/v.test('ac') && !/\b[a][b--c]/v.test('xab') &&
                /[a][c--b]\b/v.test('ac') && !/[a][c--b]\b/v.test('acx')
        "#,
    );
    assert_true(r#"let marker=0;eval("marker=1; /\\b[a][b--c]/v.test('ab')")&&marker===1"#);
    assert_true(
        r#"
            /\b[a][[b]]/v.test('ab') && !/\b[a][[b]]/v.test('xab') && /[a][[b]]\b/v.test('ab') && !
                /[a][[b]]\b/v.test('abx')
        "#,
    );
    assert_true(
        r#"
            let g = /\B[a][[a]]\b/vg,
                y = /\B[a][[a]]\b/vy,
                a = g.exec('aaa!');
            a.index === 1 && a[0] === 'aa' && g.lastIndex === 3 && g.exec('aaa!') === null && g
                .lastIndex === 0 && y.exec('aaa!') === null && y.lastIndex === 0 && (y.lastIndex = 1, y
                    .exec('aaa!')[0] === 'aa') && y.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            /\B[\p{Any}][[a]]\b/v.test('Ka!') && !/\B[\p{Any}][[a]]\b/iv.test('Ka!') && !
                /\b[\p{Any}][[a]]\b/v.test('Ka!') && /\b[\p{Any}][[a]]\b/iv.test('Ka!') && /\b[K][[s]]\b/iv
                .test('Kſ!') && !/\b[K][[s]]\b/v.test('Kſ!')
        "#,
    );
    assert_true(
        r#"
            !/\b[a][[\p{Any}]]\b/v.test('aK!') && /\b[a][[\p{Any}]]\b/iv.test('aK!') && !
                /\b[a][[\p{Any}]]\b/v.test('aſ!') && /\b[a][[\p{Any}]]\b/iv.test('aſ!') &&
                /\B[\p{Any}][[a]]\b/iv.test('αa!') && /\B[!][[\?]]\B/v.test('!?')
        "#,
    );
    assert_true(
        r#"
            !/\b[][a--b]/v.test('ab') && !/\b[a][[a--a]]/v.test('aa') && /\b[A][\P{Ll}&&A]\b/v.test(
                'AA!') && !/\b[A][\P{Ll}&&A]\b/iv.test('AA!')
        "#,
    );
    assert_true(
        r#"
            let r = /\B[\p{Any}][[a]]\b/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀a!');
            m.index === 0 && m[0] === '😀a' && m.indices[0][1] === 3 && r.lastIndex === 3 &&
                /\B[\u{1f600}][[\u{1f601}]]\B/v.exec('!😀😁!').index === 1
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\uD800][[\uD800]]\B/dv,
                m = r.exec('A\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800' && m.indices[0][1] === 3 && !/\B[\uD800][[\uDC00]]\B/v
                .test('\uD800\uDC00')
        "#,
    );
    assert_true(
        r#"
            let r = /^\b[a][[b]]\b$/dvmg,
                m = r.exec('x\r\nab\u2028cd');
            m.index === 3 && m[0] === 'ab' && m.indices[0][1] === 5 && r.lastIndex === 5 && r.exec(
                'x\r\nab\u2028cd') === null && r.lastIndex === 0 && !/^\b[a][[b]]\b$/v.test('ab\n')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\B[a][[a]]\b))/dvg,
                a = [...'aaa! aaa!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aa' && a[0][1] === 'aa' && a[0][2] === 'aa' && a[0].index ===
                1 && a[0].groups.whole === 'aa' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 3 && a[1].index === 6 && a[1].indices[2][1] === 8 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'aaa! aaa!'.replace(/(?<whole>(\B[a][[a]]\b))/vg,'<$1:$<whole>>')==='a<aa:aa>! a<aa:aa>!'&&'aaa!'.split(/(\B[a][[a]]\b)/v).join('|')==='a|aa|!'&&'aaa!'.match(/\B[a][[a]]\b/v)[0]==='aa'",
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaa! aaa!'.replace(/(?<whole>(\B[a][[a]]\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:aa;6:aa;'
        "#,
    );
    assert_true(r"let r=/\B[a][[a]]\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4");
    assert_true(
        r#"
            let trace = '',
                r = /\B[\p{Any}][[a]]\b/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀a!'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀a' && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /\B[a][[a]]\b/v;
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
            let r = /\b[K][[s]]\b/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '\\b[K][[s]]\\b' && c.unicodeSets && !c.ignoreCase && c
                .exec('Kſ!') === null && c.lastIndex === 0 && r.exec('Kſ!')[0] === 'Kſ'
        "#,
    );
    assert_true(
        r#"
            let a = /\B[a][[a]]\b/vy,
                b = /\B[a][[a]]\b/vy;
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
                eval('marker=1; /\\b[a][b&&]/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/\b[a][b]/v.test('ab')&&!/\b[a][b]/v.test('xab')&&!/\b[a][b]/v.test('ac')");
    assert_true(r"/\b[a][[b]][c]/v.test('abc')");
    assert_true(
        r#"
            let r = /\b[a][[b]][c]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '\\b[a][[b]][c]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('xabc') && r.exec('!abc').index === 1 && r.exec('abcx').index === 0 && r.exec(
                    'abcx')[0] === 'abc' && r.lastIndex === 0
        "#,
    );
    assert_eq!(
        Realm::default().eval(r"/\b[][[\q{b}]]/v.test('ab')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a]\b[[b]]/v.test('ab')",
        r"/\b^[a][[b]]/v.test('ab')",
        r"/[a][[b]]$\b/v.test('ab')",
        r"/\b\b[a][[b]]/v.test('ab')",
        r"/(\b[a])[[b]]/v.test('ab')",
        r"/\b[a]([[b]])/v.test('ab')",
        r"/\b[a]+[[b]]/v.test('ab')",
        r"/\b[a][[b]]?/v.test('ab')",
        r"/\b[][[\p{RGI_Emoji}]]/v.test('ab')",
        r"/\b[\p{RGI_Emoji}][[a--a]]/v.test('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[a][[a]]\b/vg,sticky=/\b[a][[a]]/vy,marker=0")
        .unwrap();
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===2"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_then_checked_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full field objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start) in [
        (
            r"(?<whole>(\B[a][[a]]\b))",
            JsString::from("aaa!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\B[a][[a]]\b))",
            JsString::from("aaa!"),
            "dvy",
            0,
        ),
        (
            r"(?<whole>(\B[a][[a]]\b))",
            JsString::from("aaa!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\b[K][[s]]\b))",
            JsString::from("Kſ!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\B[\p{Any}][[a]]\b))",
            JsString::from("Ka!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\B[\p{Any}][[a]]\b))",
            JsString::from("😀a!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\B[\u{1f600}][[\u{1f601}]]\B))",
            JsString::from("!😀😁!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\b[\uD800][[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\b[a][[\p{Any}]]\b))",
            JsString::from("aK!"),
            "divg",
            0,
        ),
        (
            r"^(?<whole>(\b[a][[b]]\b))$",
            JsString::from("x\r\nab\u{2028}cd"),
            "dvmg",
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
