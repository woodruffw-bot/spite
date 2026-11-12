//! Match results and consumers for pairs of character classes and literal text next to word assertions.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn literal_checked_v_set_pair_word_state_context_and_consumers() {
    assert_true(
        r#"
            /\bc[a--b][[c]]\b/v.test('cac') && !/\bc[a--b][[c]]\b/v.test('cbc') && !/\bc[a--b][[c]]\b/v
                .test('ca') && !/\bc[a--a][[b]]\b/v.test('cab') && !/\bc[[a]][b--b]\b/v.test('cab')
        "#,
    );
    assert_true(r#"let marker=0;eval("marker=1; /\\bc[a--b][[c]]\\b/v.test('cac')")&&marker===1"#);
    assert_true(
        r#"
            let a = /a[[a]][[a]]\b/dvg,
                b = /\Ba[[a]][[a]]/dvg,
                x = a.exec('aaaa!'),
                y = b.exec('aaaa!');
            x.index === 1 && x[0] === 'aaa' && x.indices[0][1] === 4 && a.lastIndex === 4 && y.index ===
                1 && y[0] === 'aaa' && y.indices[0][1] === 4 && b.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let g = /\Ba[[a]][[a]]\b/vg,
                y = /\Ba[[a]][[a]]\b/vy,
                a = g.exec('aaaa!');
            a.index === 1 && a[0] === 'aaa' && g.lastIndex === 4 && g.exec('aaaa!') === null && g
                .lastIndex === 0 && y.exec('aaaa!') === null && y.lastIndex === 0 && (y.lastIndex = 1, y
                    .exec('aaaa!')[0] === 'aaa') && y.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            /\bc[[a]][[b]]/v.test('cab') && /\Bc[[a]][[b]]/v.test('xcab') && /c[[a]][[b]]\b/v.test(
                'cab!') && /c[[a]][[b]]\B/v.test('cabx') && /\bc[[a]][[b]]\b/v.test('!cab!') &&
                /\bc[[a]][[b]]\B/v.test('!cabx') && /\Bc[[a]][[b]]\b/v.test('xcab!') && /\Bc[[a]][[b]]\B/v
                .test('xcabx')
        "#,
    );
    assert_true(
        r#"
            !/\bc[[a]][[b]]/v.test('xcab') && !/c[[a]][[b]]\b/v.test('cabx') && !/\Bc[[a]][[b]]/v.test(
                '!cab') && !/c[[a]][[b]]\B/v.test('cab!')
        "#,
    );
    assert_true(
        r#"
            /\bk[[s]][[A]]\b/iv.test('Kſa') && !/\bk[[s]][[A]]\b/v.test('Kſa') && !/\bk[[s]][[A]]\b/v.test(
                    'Ksa') && /\bk[[s]][[A]]\b/v.test('ksA') && /\Ba[[a]][[^K]]\b/iv.exec('aaKaaſ!')[0] ===
                'aaſ' && !/\Ba[[a]][[^K]]\b/iv.test('aaK')
        "#,
    );
    assert_true(
        r#"
            /\b\u{212a}[[a]][[a]]/iv.test('Kaa') && !/\b\u{212a}[[a]][[a]]/v.test('Kaa') &&
                /\b\u{17f}[[a]][[a]]/iv.test('ſaa') && !/\b\u{17f}[[a]][[a]]/v.test('ſaa') &&
                /a[[a]][[\p{Any}]]\b/iv.test('aaK!') && !/a[[a]][[\p{Any}]]\b/v.test('aaK!') &&
                /a[[a]][[\p{Any}]]\b/iv.test('aaſ!') && !/a[[a]][[\p{Any}]]\b/v.test('aaſ!')
        "#,
    );
    assert_true(
        r#"
            /\bx[\P{Ll}&&A][[s]]\b/v.test('xAs') && !/\bx[\P{Ll}&&A][[s]]\b/iv.test('xAs') &&
                /\bx[[\p{ASCII}]][[a]]\b/v.test('xAa') && !/\bx[[\p{ASCII}]][[a]]\b/v.test('xKa')
        "#,
    );
    assert_true(
        r#"
            let r = /\B\u{1f600}[[\p{Any}]][[a]]\b/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁a!');
            m.index === 0 && m[0] === '😀😁a' && m.indices[0][1] === 5 && r.lastIndex === 5 &&
                /\B😀[[😁]][[😂]]\B/v.test('!😀😁😂!') && /\B\uD83D\uDE00[[\u{1f601}]][[\u{1f602}]]\B/v
                .test('!😀😁😂!')
        "#,
    );
    assert_true(
        r#"
            let r = /\B\u{1f600}[[\u{1f601}]][[\u{1f602}]]\B/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁😂!');
            m.index === 0 && m[0] === '😀😁😂' && m.indices[0][1] === 6 && r.lastIndex === 6
        "#,
    );
    assert_true(
        r#"
            let r = /\b\uD800[[\uD800]][[\uD800]]\B/dv,
                m = r.exec('A\uD800\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800\uD800' && m.indices[0][1] === 4 && !
                /\B\uD83D[[\uDE00]][[a]]\b/v.test('😀a') && !/\ba[[\uD83D]][[\uDE00]]\B/v.test('a😀')
        "#,
    );
    assert_true(
        r#"
            let r = /\B\/[[a]][[b]]\b/v;
            r.test('!/ab!') && r.source === '\\B\\/[[a]][[b]]\\b' && /\b\[[[a]][[b]]\b/v.test('A[ab!') &&
                /^\B\$[[a]][[b]]\b$/v.test('$ab') && !/^\B\$[[a]][[b]]\b$/v.test('$ab\n')
        "#,
    );
    assert_true(
        r#"
            let r = /^\bc[[a]][[b]]\b$/dvmg,
                m = r.exec('x\r\ncab\u2028cd');
            m.index === 3 && m[0] === 'cab' && m.indices[0][1] === 6 && r.lastIndex === 6 && r.exec(
                'x\r\ncab\u2028cd') === null && r.lastIndex === 0 && !/^\bc[[a]][[b]]\b$/v.test(
                'cab\n') && /^\bc[[a]][[b]]/v.test('cab!') && /c[[a]][[b]]\b$/v.test('xcab')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\Ba[[a]][[a]]\b))/dvg,
                a = [...'aaaa! aaaa!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aaa' && a[0][1] === 'aaa' && a[0][2] === 'aaa' && a[0].index ===
                1 && a[0].groups.whole === 'aaa' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 4 && a[1].index === 7 && a[1].indices[2][1] === 10 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'aaaa! aaaa!'.replace(/(?<whole>(\Ba[[a]][[a]]\b))/vg,'<$1:$<whole>>')==='a<aaa:aaa>! a<aaa:aaa>!'&&'aaaa!'.split(/(\Ba[[a]][[a]]\b)/v).join('|')==='a|aaa|!'&&'aaaa!'.match(/\Ba[[a]][[a]]\b/v)[0]==='aaa'",
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaaa! aaaa!'.replace(/(?<whole>(\Ba[[a]][[a]]\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:aaa;7:aaa;'
        "#,
    );
    assert_true(r"let r=/\Ba[[a]][[a]]\b/vg;r.lastIndex=4;'aaaa!'.search(r)===1&&r.lastIndex===4");
    assert_true(
        r#"
            let trace = '',
                r = /\B\u{1f600}[[\p{Any}]][[a]]\b/vy;
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
                r = /\Ba[[a]][[a]]\b/v;
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
            let r = /\bk[[s]][[A]]\b/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '\\bk[[s]][[A]]\\b' && c.unicodeSets && !c.ignoreCase &&
                c.exec('Kſa') === null && c.lastIndex === 0 && r.exec('Kſa')[0] === 'Kſa'
        "#,
    );
    assert_true(
        r#"
            let a = /\Ba[[a]][[a]]\b/vy,
                b = /\Ba[[a]][[a]]\b/vy;
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
                eval('marker=1; /\\bc[a--a][b&&]\\b/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/\b[[a]][[b]]c/v.test('abc')");
    assert_true(
        r#"
            let r = /\b[[a]][[b]]c/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '\\b[[a]][[b]]c' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('xabc') && r.test('abcx')
        "#,
    );
    assert_true(r"/[[a]][[b]]c\b/v.test('abc')");
    assert_true(
        r#"
            let r = /[[a]][[b]]c\b/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '[[a]][[b]]c\\b' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('abcx') && r.test('xabc')
        "#,
    );
    assert_true(r"/\b[[a]]b[[c]]/v.test('abc')");
    assert_true(
        r#"
            let r = /\b[[a]]b[[c]]/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '\\b[[a]]b[[c]]' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('xabc') && r.test('abcx')
        "#,
    );
    assert_true(r"/[[a]]b[[c]]\b/v.test('abc')");
    assert_true(
        r#"
            let r = /[[a]]b[[c]]\b/v,
                m = r.exec('abc');
            m !== null && m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m
                .groups === undefined && m.indices === undefined && r.source === '[[a]]b[[c]]\\b' && r
                .lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab') &&
                !r.test('abcx') && r.test('xabc')
        "#,
    );
    assert_true(r"/\bc[a][b]/v.test('cab')");
    assert_true(
        r#"
            let r = /\bc[a][b]/v,
                m = r.exec('cab');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'cab' && m.index === 0 && m
                .input === 'cab' && m.groups === undefined && m.indices === undefined && r.source ===
                '\\bc[a][b]' && r.lastIndex === 0 && !r.test('xab') && !r.test('cxb') && !r.test('cax') &&
                !r.test('ca') && !r.test('xcab') && r.test('cabx')
        "#,
    );
    assert_true(r"/\bc[[a]][b]/v.test('cab')");
    assert_true(
        r#"
            let r = /\bc[[a]][b]/v,
                m = r.exec('cab');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'cab' && m.index === 0 && m
                .input === 'cab' && m.groups === undefined && m.indices === undefined && r.source ===
                '\\bc[[a]][b]' && r.lastIndex === 0 && !r.test('xab') && !r.test('cxb') && !r.test(
                'cax') && !r.test('ca') && r.test('cabx') && !r.test('xcab') && r.exec('!cab').index === 1
        "#,
    );
    assert_true(r"/\bc[a][[b]]/v.test('cab')");
    assert_true(
        r#"
            let r = /\bc[a][[b]]/v,
                m = r.exec('cab');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'cab' && m.index === 0 && m
                .input === 'cab' && m.groups === undefined && m.indices === undefined && r.source ===
                '\\bc[a][[b]]' && r.lastIndex === 0 && !r.test('xab') && !r.test('cxb') && !r.test(
                'cax') && !r.test('ca') && r.test('cabx') && !r.test('xcab') && r.exec('!cab').index === 1
        "#,
    );
    for source in [
        r"/\bca[[a]][[b]]/v.test('caab')",
        r"/\bc[[a]][[b]]d/v.test('cabd')",
        r"/\bc[[a]][[b]][[c]]/v.test('cabc')",
        r"/\bc[[a]]\b[[b]]\b/v.test('cab')",
        r"/\b^c[[a]][[b]]/v.test('cab')",
        r"/c[[a]][[b]]$\b/v.test('cab')",
        r"/\b\bc[[a]][[b]]/v.test('cab')",
        r"/c[[a]][[b]]\b\b/v.test('cab')",
        r"/(\bc)[[a]][[b]]/v.test('cab')",
        r"/\bc([[a]])[[b]]/v.test('cab')",
        r"/\bc[[a]]([[b]])/v.test('cab')",
        r"/\bc+[[a]][[b]]/v.test('cab')",
        r"/\bc[[a]]+[[b]]/v.test('cab')",
        r"/\bc[[a]][[b]]?/v.test('cab')",
        r"/(?:\bc[[a]][[b]]\b)+/v.test('cab')",
        r"/\bc[a--a][[\q{b}]]\b/v.test('cab')",
        r"/\bc[[\q{a}]][b--b]\b/v.test('cab')",
        r"/\bc[a--a][[\p{RGI_Emoji}]]\b/v.test('cab')",
        r"/\bc[[\p{RGI_Emoji}]][b--b]\b/v.test('cab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r"var kept=/(?<whole>(\Ba[[a]][[a]]\b))/dvg;kept.lastIndex=1")
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
        .eval(r"var r=/\Ba[[a]][[c]]\b/vg,sticky=/\ba[[a]][[a]]/vy,marker=0")
        .unwrap();
    // The literal and first class succeed; the second fails before word gates.
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
fn literal_checked_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start) in [
        (
            r"(?<whole>(\Ba[[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\Ba[[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            0,
        ),
        (
            r"(?<whole>(\Ba[[a]][[a]]\b))",
            JsString::from("aaaa!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\bk[[s]][[A]]\b))",
            JsString::from("Kſa!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\bx[\P{Ll}&&A][[s]]\b))",
            JsString::from("xAs!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\B\u{1f600}[[\p{Any}]][[a]]\b))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\B\u{1f600}[[\u{1f601}]][[\u{1f602}]]\B))",
            JsString::from("!😀😁😂!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\b\uD800[[\uD800]][[\uD800]]\B))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\ba[[a]][[\p{Any}]]\b))",
            JsString::from("aaK!"),
            "divg",
            0,
        ),
        (
            r"^(?<whole>(\bc[[a]][[b]]\b))$",
            JsString::from("x\r\ncab\u{2028}cd"),
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
