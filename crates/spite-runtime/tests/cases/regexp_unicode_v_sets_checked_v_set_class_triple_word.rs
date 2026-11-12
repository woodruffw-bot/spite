//! Match results and consumers for three adjacent character classes next to word assertions.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_class_triple_word_state_context_and_consumers() {
    assert_true(
        r#"
            /\b[a--b][[c]][[d]]\b/v.test('acd') && !/\b[a--b][[c]][[d]]\b/v.test('bcd') && !
                /\b[a--b][[c]][[d]]\b/v.test('ac') && !/\b[a--a][[c]][[b]]\b/v.test('acb') && !
                /\b[[a]][c--c][[b]]\b/v.test('acb') && !/\b[[a]][[c]][b--b]\b/v.test('acb')
        "#,
    );
    assert_true(
        r#"let marker=0;eval("marker=1; /\\b[a--b][[c]][[d]]\\b/v.test('acd')")&&marker===1"#,
    );
    assert_true(
        r#"
            let a = /[[a]][[a]][[a]]\b/dvg,
                b = /\B[[a]][[a]][[a]]/dvg,
                x = a.exec('aaaa!'),
                y = b.exec('aaaa!');
            x.index === 1 && x[0] === 'aaa' && x.indices[0][1] === 4 && a.lastIndex === 4 && y.index ===
                1 && y[0] === 'aaa' && y.indices[0][1] === 4 && b.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let g = /\B[[a]][[a]][[a]]\b/vg,
                y = /\B[[a]][[a]][[a]]\b/vy,
                a = g.exec('aaaa!');
            a.index === 1 && a[0] === 'aaa' && g.lastIndex === 4 && g.exec('aaaa!') === null && g
                .lastIndex === 0 && y.exec('aaaa!') === null && y.lastIndex === 0 && (y.lastIndex = 1, y
                    .exec('aaaa!')[0] === 'aaa') && y.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            /\b[[a]][[b]][[c]]/v.test('abc') && /\B[[a]][[b]][[c]]/v.test('xabc') && /[[a]][[b]][[c]]\b/v
                .test('abc!') && /[[a]][[b]][[c]]\B/v.test('abcx') && /\b[[a]][[b]][[c]]\b/v.test(
                '!abc!') && /\b[[a]][[b]][[c]]\B/v.test('!abcx') && /\B[[a]][[b]][[c]]\b/v.test('xabc!') &&
                /\B[[a]][[b]][[c]]\B/v.test('xabcx')
        "#,
    );
    assert_true(
        r#"
            !/\b[[a]][[b]][[c]]/v.test('xabc') && !/[[a]][[b]][[c]]\b/v.test('abcx') && !
                /\B[[a]][[b]][[c]]/v.test('!abc') && !/[[a]][[b]][[c]]\B/v.test('abc!')
        "#,
    );
    assert_true(
        r#"
            /\b[[K]][[s]][[A]]\b/iv.test('Kſa') && !/\b[[K]][[s]][[A]]\b/v.test('Kſa') && !
                /\b[[K]][[s]][[A]]\b/v.test('Ksa') && /\b[[K]][[s]][[A]]\b/v.test('KsA') &&
                /\B[[a]][[a]][[^K]]\b/iv.exec('aaKaaſ!')[0] === 'aaſ' && !/\B[[a]][[a]][[^K]]\b/iv.test(
                    'aaK')
        "#,
    );
    assert_true(
        r#"
            /\b[[\p{Any}]][[a]][[a]]/iv.test('Kaa') && !/\b[[\p{Any}]][[a]][[a]]/v.test('Kaa') &&
                /\b[[\p{Any}]][[a]][[a]]/iv.test('ſaa') && !/\b[[\p{Any}]][[a]][[a]]/v.test('ſaa') &&
                /[[a]][[a]][[\p{Any}]]\b/iv.test('aaK!') && !/[[a]][[a]][[\p{Any}]]\b/v.test('aaK!') &&
                /[[a]][[a]][[\p{Any}]]\b/iv.test('aaſ!') && !/[[a]][[a]][[\p{Any}]]\b/v.test('aaſ!')
        "#,
    );
    assert_true(
        r#"
            /\b[\P{Ll}&&A][[x]][[s]]\b/v.test('Axs') && !/\b[\P{Ll}&&A][[x]][[s]]\b/iv.test('Axs') &&
                /\b[[\p{ASCII}]][[x]][[a]]\b/v.test('Axa') && !/\b[[\p{ASCII}]][[x]][[a]]\b/v.test('Kxa')
        "#,
    );
    assert_true(
        r#"
            let r = /\B[[\p{Any}]][[\u{1f601}]][[a]]\b/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁a!');
            m.index === 0 && m[0] === '😀😁a' && m.indices[0][1] === 5 && r.lastIndex === 5 &&
                /\B[[😀]][[😁]][[😂]]\B/v.test('!😀😁😂!')
        "#,
    );
    assert_true(
        r#"
            let r = /\B[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]\B/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁😂!');
            m.index === 0 && m[0] === '😀😁😂' && m.indices[0][1] === 6 && r.lastIndex === 6
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[\uD800]][[\uD800]][[\uD800]]\B/dv,
                m = r.exec('A\uD800\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800\uD800' && m.indices[0][1] === 4 && !
                /\B[[\uD83D]][[\uDE00]][[a]]\b/v.test('😀a') && !/\b[[a]][[\uD83D]][[\uDE00]]\B/v.test(
                    'a😀')
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[a]][[\/]][[b]]\b/v;
            r.test('a/b') && r.source === '\\b[[a]][[\\/]][[b]]\\b' && /\b[[a]][[\[]][[b]]\b/v.test(
                'a[b') && /^\b[[a]][[$]][[b]]\b$/v.test('a$b') && !/^\b[[a]][[$]][[b]]\b$/v.test('a$b\n')
        "#,
    );
    assert_true(
        r#"
            let r = /^\b[[a]][[b]][[c]]\b$/dvmg,
                m = r.exec('x\r\nabc\u2028de');
            m.index === 3 && m[0] === 'abc' && m.indices[0][1] === 6 && r.lastIndex === 6 && r.exec(
                'x\r\nabc\u2028de') === null && r.lastIndex === 0 && !/^\b[[a]][[b]][[c]]\b$/v.test(
                'abc\n') && /^\b[[a]][[b]][[c]]/v.test('abc!') && /[[a]][[b]][[c]]\b$/v.test('xabc')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\B[[a]][[a]][[a]]\b))/dvg,
                a = [...'aaaa! aaaa!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aaa' && a[0][1] === 'aaa' && a[0][2] === 'aaa' && a[0].index ===
                1 && a[0].groups.whole === 'aaa' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 4 && a[1].index === 7 && a[1].indices[2][1] === 10 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'aaaa! aaaa!'.replace(/(?<whole>(\B[[a]][[a]][[a]]\b))/vg,'<$1:$<whole>>')==='a<aaa:aaa>! a<aaa:aaa>!'&&'aaaa!'.split(/(\B[[a]][[a]][[a]]\b)/v).join('|')==='a|aaa|!'&&'aaaa!'.match(/\B[[a]][[a]][[a]]\b/v)[0]==='aaa'",
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaaa! aaaa!'.replace(/(?<whole>(\B[[a]][[a]][[a]]\b))/vg, (m, c1, c2, p, text,
                g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:aaa;7:aaa;'
        "#,
    );
    assert_true(
        r"let r=/\B[[a]][[a]][[a]]\b/vg;r.lastIndex=4;'aaaa!'.search(r)===1&&r.lastIndex===4",
    );
    assert_true(
        r#"
            let trace = '',
                r = /\B[[\p{Any}]][[\u{1f601}]][[a]]\b/vy;
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
                r = /\B[[a]][[a]][[a]]\b/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aaaa!');
            m.index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[K]][[s]][[A]]\b/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '\\b[[K]][[s]][[A]]\\b' && c.unicodeSets && !c
                .ignoreCase && c.exec('Kſa') === null && c.lastIndex === 0 && r.exec('Kſa')[0] === 'Kſa'
        "#,
    );
    assert_true(
        r#"
            let a = /\B[[a]][[a]][[a]]\b/vy,
                b = /\B[[a]][[a]][[a]]\b/vy;
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
                a.exec('aaaa!')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('aaaa!')
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
                eval('marker=1; /\\b[a--a][[b]][c&&]\\b/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/\b[[a]][[b]][[c]][[d]]\b/v.test('abcd')");
    assert_true(
        r#"
            let r = /\b[[a]][[b]][[c]][[d]]\b/v,
                m = r.exec('abcd');
            m !== null && m.length === 1 && m[0] === 'abcd' && m.index === 0 && m.input === 'abcd' && m
                .groups === undefined && m.indices === undefined && r.source ===
                '\\b[[a]][[b]][[c]][[d]]\\b' && r.lastIndex === 0 && !r.test('xbcd') && !r.test('axcd') &&
                !r.test('abxd') && !r.test('abcx') && !r.test('abc') && !r.test('xabcd') && !r.test(
                    'abcdx')
        "#,
    );
    assert_true(r"/\b[a][[b]][[c]]/v.test('abc')");
    assert_true(
        r#"
            let r = /\b[a][[b]][[c]]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '\\b[a][[b]][[c]]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('xabc') && r.exec('!abc').index === 1 && r.exec('abcx').index === 0 && r.exec(
                    'abcx')[0] === 'abc' && r.lastIndex === 0
        "#,
    );
    assert_true(r"/\b[[a]][b][[c]]/v.test('abc')");
    assert_true(
        r#"
            let r = /\b[[a]][b][[c]]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '\\b[[a]][b][[c]]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('xabc') && r.exec('!abc').index === 1 && r.exec('abcx').index === 0 && r.exec(
                    'abcx')[0] === 'abc' && r.lastIndex === 0
        "#,
    );
    assert_true(r"/\b[[a]][[b]][c]/v.test('abc')");
    assert_true(
        r#"
            let r = /\b[[a]][[b]][c]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '\\b[[a]][[b]][c]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('xabc') && r.exec('!abc').index === 1 && r.exec('abcx').index === 0 && r.exec(
                    'abcx')[0] === 'abc' && r.lastIndex === 0
        "#,
    );
    for source in [
        r"/\bx[[a]][[b]][[c]]/v.test('xabc')",
        r"/[[a]][[b]][[c]]x\b/v.test('abcx')",
        r"/\b^[[a]][[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]][[c]]$\b/v.test('abc')",
        r"/\b\b[[a]][[b]][[c]]/v.test('abc')",
        r"/[[a]][[b]][[c]]\b\b/v.test('abc')",
        r"/\b\B[[a]][[b]][[c]]/v.test('abc')",
        r"/\b[[a]]\b[[b]][[c]]\b/v.test('abc')",
        r"/\b[[a]]^[[b]][[c]]/v.test('abc')",
        r"/(\b[[a]])[[b]][[c]]/v.test('abc')",
        r"/\b[[a]]([[b]])[[c]]/v.test('abc')",
        r"/\b[[a]][[b]]([[c]])/v.test('abc')",
        r"/\b[[a]]+[[b]][[c]]/v.test('abc')",
        r"/\b[[a]][[b]]?[[c]]/v.test('abc')",
        r"/\b[[a]][[b]][[c]]+/v.test('abc')",
        r"/(?:\b[[a]][[b]][[c]]\b)+/v.test('abc')",
        r"/\b[a--a][[b]][[\p{RGI_Emoji}]]\b/v.test('abc')",
        r"/\b[[a]][b--b][[\p{RGI_Emoji}]]\b/v.test('abc')",
        r"/\b[[\p{RGI_Emoji}]][[b]][c--c]\b/v.test('abc')",
        r"/\b[a--a][[b]][[\q{c}]]\b/v.test('abc')",
        r"/\b[[a]][b--b][[\q{c}]]\b/v.test('abc')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r"var kept=/(?<whole>(\B[[a]][[a]][[a]]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaaa!');m.index===1&&m.groups.whole==='aaa'&&m.indices[2][1]===4&&kept.lastIndex===4"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[[a]][[a]][[c]]\b/vg,sticky=/\b[[a]][[a]][[a]]/vy,marker=0")
        .unwrap();
    // The first two checked classes succeed; the third fails before word gates.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval("sticky.test(text)&&sticky.lastIndex===3"),
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
fn checked_v_set_class_triple_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start) in [
        (
            r"(?<whole>(\B[[a]][[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            0,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\b[[K]][[s]][[A]]\b))",
            JsString::from("Kſa!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\b[\P{Ll}&&A][[x]][[s]]\b))",
            JsString::from("Axs"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]][[\u{1f601}]][[a]]\b))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\B[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]\B))",
            JsString::from("!😀😁😂!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\b[[\uD800]][[\uD800]][[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[^K]]\b))",
            JsString::from("aaKaaſ!"),
            "divg",
            0,
        ),
        (
            r"^(?<whole>(\b[[a]][[b]][[c]]\b))$",
            JsString::from("x\r\nabc\u{2028}de"),
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
