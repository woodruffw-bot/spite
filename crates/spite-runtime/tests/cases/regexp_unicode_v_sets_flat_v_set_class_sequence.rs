//! Match results and consumers for sequences of character classes.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn flat_v_set_class_sequence_state_context_and_consumers() {
    assert_true(
        r#"
            /[a][b][c]/v.test('abc') && !/[a][b][c]/v.test('xbc') && !/[a][b][c]/v.test('axc') && !
                /[a][b][c]/v.test('abx') && !/[a][b][c]/v.test('ab') && !/[][b][c]/v.test('abc') && !
                /[a][][c]/v.test('abc') && !/[a][b][]/v.test('abc')
        "#,
    );
    assert_true(
        r#"
            let ok = true;
            for (let n of [3, 4, 17]) {
                let text = 'a'.repeat(n),
                    r = new RegExp('(?<whole>(' + '[a]'.repeat(n) + '))', 'dvy'),
                    m = r.exec(text);
                ok = ok && m !== null && m.length === 3 && m[0] === text && m[1] === text && m[2] ===
                    text && m.groups.whole === text && m.indices[0][0] === 0 && m.indices[2][1] === n && m
                    .indices.groups.whole === m.indices[1] && r.lastIndex === n;
                r.lastIndex = 0;
                ok = ok && r.exec(text.slice(1)) === null && r.lastIndex === 0
            }
            ok
        "#,
    );
    assert_true(r#"let marker=0;eval("marker=1; /\\b[a][b][c]\\b/v.test('abc')")&&marker===1"#);
    assert_true(
        r#"
            let a = /\B[a][a][b]\b/dvg,
                b = /\B[a][a][ab]\b/dvg,
                x = a.exec('aaab!'),
                y = b.exec('aaaa!');
            x.index === 1 && x[0] === 'aab' && x.indices[0][1] === 4 && a.lastIndex === 4 && y.index ===
                1 && y[0] === 'aaa' && y.indices[0][1] === 4 && b.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let g = /\B[a][a][b]\b/vg,
                y = /\B[a][a][b]\b/vy,
                a = g.exec('aaab!');
            a.index === 1 && a[0] === 'aab' && g.lastIndex === 4 && g.exec('aaab!') === null && g
                .lastIndex === 0 && y.exec('aaab!') === null && y.lastIndex === 0 && (y.lastIndex = 1, y
                    .exec('aaab!')[0] === 'aab') && y.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            /\b[a][b][c]/v.test('abc') && /\B[a][b][c]/v.test('xabc') && /[a][b][c]\b/v.test('abc!') &&
                /[a][b][c]\B/v.test('abcx') && /\b[a][b][c]\b/v.test('!abc!') && /\b[a][b][c]\B/v.test(
                    '!abcx') && /\B[a][b][c]\b/v.test('xabc!') && /\B[a][b][c]\B/v.test('xabcx')
        "#,
    );
    assert_true(
        r#"
            !/\b[a][b][c]/v.test('xabc') && !/[a][b][c]\b/v.test('abcx') && !/\B[a][b][c]/v.test('!abc') &&
                !/[a][b][c]\B/v.test('abc!') && /\b[a][b][c]/v.exec('abcx')[0] === 'abc'
        "#,
    );
    assert_true(
        r#"
            /\b[K][s][a]\b/iv.test('KſA') && !/\b[K][s][a]\b/v.test('Ksa') && !/\b[K][s][a]\b/v.test(
                'Kſa') && !/\b[K][s][a]\b/v.test('KsA') && /\b[K][s][a]\b/v.test('Ksa') &&
                /^[^K][s][a][b]$/iv.test('ſSAb') && !/^[^K][s][a][b]$/iv.test('KSAb')
        "#,
    );
    assert_true(
        r#"
            /\b[\p{Any}][a][b][\p{Any}]\b/iv.test('Kabſ!') && !/\b[\p{Any}][a][b][\p{Any}]\b/v.test(
                    'Kabſ!') && /\b[\p{Any}][a][b][\p{Any}]/iv.test('Kabſ') && !
                /\b[\p{Any}][a][b][\p{Any}]/v.test('Kabſ') && /[\p{Any}][a][b][\p{Any}]\b/iv.test(
                'Kabſ!') && !/[\p{Any}][a][b][\p{Any}]\b/v.test('Kabſ!')
        "#,
    );
    assert_true(r"/\b[\P{Ll}][s][a]\b/v.test('Asa')&&!/\b[\P{Ll}][s][a]\b/iv.test('Asa')");
    assert_true(
        r#"
            let r = /[\p{Any}][\u{1f601}][\u{1f602}]/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁😂!');
            m.index === 0 && m[0] === '😀😁😂' && m.indices[0][1] === 6 && r.lastIndex === 6
        "#,
    );
    assert_true(
        r#"
            let r = /[\u{1f600}][\u{1f601}][\u{1f602}][\u{1f603}]/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁😂😃!');
            m.index === 0 && m[0] === '😀😁😂😃' && m.indices[0][1] === 8 && r.lastIndex === 8
        "#,
    );
    assert_true(
        r#"
            let r = /\b[\uD800][\uD800][\uD800]\B/dv,
                m = r.exec('A\uD800\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800\uD800' && m.indices[0][1] === 4 && !
                /[\uD83D][\uDE00][a]/v.test('😀a') && !/[a][\uD83D][\uDE00]/v.test('a😀')
        "#,
    );
    assert_true(
        r#"
            let r = /^\b[a][b][c][d]\b$/dvmg,
                m = r.exec('x\r\nabcd\u2028ef');
            m.index === 3 && m[0] === 'abcd' && m.indices[0][1] === 7 && r.lastIndex === 7 && r.exec(
                    'x\r\nabcd\u2028ef') === null && r.lastIndex === 0 && !/^[a][b][c][d]$/v.test(
                'abcd\n') && /^[a][b][c][d]/v.test('abcd!') && /[a][b][c][d]$/v.test('xabcd')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\B[a][a][b]\b))/dvg,
                a = [...'aaab! aaab!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aab' && a[0][1] === 'aab' && a[0][2] === 'aab' && a[0].index ===
                1 && a[0].groups.whole === 'aab' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 4 && a[1].index === 7 && a[1].indices[2][1] === 10 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'aaab! aaab!'.replace(/(?<whole>(\B[a][a][b]\b))/vg,'<$1:$<whole>>')==='a<aab:aab>! a<aab:aab>!'&&'aaab!'.split(/(\B[a][a][b]\b)/v).join('|')==='a|aab|!'&&'aaab!'.match(/\B[a][a][b]\b/v)[0]==='aab'",
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaab! aaab!'.replace(/(?<whole>(\B[a][a][b]\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:aab;7:aab;'
        "#,
    );
    assert_true(r"let r=/\B[a][a][b]\b/vg;r.lastIndex=4;'aaab!'.search(r)===1&&r.lastIndex===4");
    assert_true(
        r#"
            let trace = '',
                r = /[\p{Any}][\u{1f601}][\u{1f602}]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀😁😂!'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀😁😂' && r.lastIndex === 6
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /\B[a][a][b]\b/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aaab!');
            m.index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /\b[K][s][a]\b/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '\\b[K][s][a]\\b' && c.unicodeSets && !c.ignoreCase && c
                .exec('KſA') === null && c.lastIndex === 0 && r.exec('KſA')[0] === 'KſA'
        "#,
    );
    assert_true(
        r#"
            let a = /\B[a][a][b]\b/vy,
                b = /\B[a][a][b]\b/vy;
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
                a.exec('aaab!')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('aaab!')
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
                eval('marker=1; /[][b][c&&]/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/[[a]][b][c]/v.test('abc')");
    assert_true(
        r#"
            let r = /[[a]][b][c]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '[[a]][b][c]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                r.exec('xabc').index === 1 && r.exec('abcx').index === 0 && r.lastIndex === 0
        "#,
    );
    assert_true(r"/[a][[b]][c]/v.test('abc')");
    assert_true(
        r#"
            let r = /[a][[b]][c]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '[a][[b]][c]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                r.exec('xabc').index === 1 && r.exec('abcx').index === 0 && r.lastIndex === 0
        "#,
    );
    assert_true(r"/[a][b][[c]]/v.test('abc')");
    assert_true(
        r#"
            let r = /[a][b][[c]]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '[a][b][[c]]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                r.exec('xabc').index === 1 && r.exec('abcx').index === 0 && r.lastIndex === 0
        "#,
    );
    assert_true(r"/[a&&a][b][c]/v.test('abc')");
    assert_true(
        r#"
            let r = /[a&&a][b][c]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '[a&&a][b][c]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                r.exec('xabc').index === 1 && r.exec('abcx').index === 0 && r.lastIndex === 0
        "#,
    );
    assert_true(r"/[a][b][c--d]/v.test('abc')");
    assert_true(
        r#"
            let r = /[a][b][c--d]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '[a][b][c--d]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                r.exec('xabc').index === 1 && r.exec('abcx').index === 0 && r.lastIndex === 0
        "#,
    );
    for source in [
        r"/a[a][b][c]/v.test('aabc')",
        r"/[a]b[c][d]/v.test('abcd')",
        r"/[a][b][c]d/v.test('abcd')",
        r"/[a]\b[b][c]/v.test('abc')",
        r"/\b^[a][b][c]/v.test('abc')",
        r"/[a][b][c]$\b/v.test('abc')",
        r"/\b\b[a][b][c]/v.test('abc')",
        r"/[a][b][c]\b\b/v.test('abc')",
        r"/([a])[b][c]/v.test('abc')",
        r"/[a]([b])[c]/v.test('abc')",
        r"/[a][b]([c])/v.test('abc')",
        r"/[a]+[b][c]/v.test('abc')",
        r"/[a][b][c]?/v.test('abc')",
        r"/(?:\b[a][b][c]\b)+/v.test('abc')",
        r"/[a][b][c]|[d][e][f]/v.test('abc')",
        r"/[][b][\q{c}]/v.test('abc')",
        r"/[\q{a}][b][]/v.test('abc')",
        r"/[][b][\p{RGI_Emoji}]/v.test('abc')",
        r"/[\p{RGI_Emoji}][b][]/v.test('abc')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r"var kept=/(?<whole>(\B[a][a][b]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r"let m=kept.exec('aaab!');m.index===1&&m.groups.whole==='aab'&&m.indices[2][1]===4&&kept.lastIndex===4"), Ok(Value::Boolean(true)));
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[a][a][c]\b/vg,sticky=/[a][a][a]/vy,marker=0")
        .unwrap();
    // The first two flat classes succeed; the third fails before word gates.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r"sticky.test(text)&&sticky.lastIndex===3"),
        Ok(Value::Boolean(true))
    );
    assert!(matches!(
        realm.eval(r"try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r"marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn flat_v_set_class_sequence_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start) in [
        (
            r"(?<whole>(\B[a][a][b]\b))",
            JsString::from("aaab!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\B[a][a][b]\b))",
            JsString::from("aaab!"),
            "dvy",
            0,
        ),
        (
            r"(?<whole>(\B[a][a][b]\b))",
            JsString::from("aaab!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\b[\p{Any}][a][b][\p{Any}]\b))",
            JsString::from("Kabſ!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\b[\P{Ll}][s][a]\b))",
            JsString::from("Asa!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>([\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}][\p{Any}]))",
            JsString::from("😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\b[\uD800][\uD800][\uD800]\B))",
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
        ),
        (
            r"^(?<whole>(\b[a][b][c][d]\b))$",
            JsString::from("x\r\nabcd ef"),
            "dvmg",
            0,
        ),
        (
            r"(?<whole>(\B[a][a][ab]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
        ),
        (
            r"^(?<whole>([^K][s][a][b]))$",
            JsString::from("ſSAb"),
            "divy",
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
