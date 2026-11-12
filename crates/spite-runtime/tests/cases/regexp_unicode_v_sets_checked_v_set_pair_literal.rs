//! Match results and consumers for pairs of character classes and literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_pair_literal_state_context_and_consumers() {
    assert_true(
        r#"
            /[a--b][[c]]d/v.test('acd') && !/[a--b][[c]]d/v.test('bcd') && !/[a--b][[c]]d/v.test('ac') &&
                /[[a]][[a]]c/v.exec('aaac').index === 1 && !/[a--a][[b]]c/v.test('abc') && !/[[a]][b--b]c/v
                .test('abc')
        "#,
    );
    assert_true(r#"let marker=0;eval("marker=1; /[a--b][[c]]d/v.test('acd')")&&marker===1"#);
    assert_true(
        r#"
            let g = /[[a]][[a]]c/vg,
                y = /[[a]][[a]]c/vy,
                a = g.exec('aaac');
            a.index === 1 && a[0] === 'aac' && g.lastIndex === 4 && g.exec('aaac') === null && g
                .lastIndex === 0 && y.exec('aaac') === null && y.lastIndex === 0 && (y.lastIndex = 1, y
                    .exec('aaac')[0] === 'aac') && y.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            /[[K]][[s]]a/iv.test('KſA') && !/[[K]][[s]]a/v.test('KſA') && !/[[K]][[s]]a/v.test('KsA') &&
                /[[K]][[s]]a/v.test('Ksa') && /[[a]][[^K]]s/iv.exec('aKsaſſ')[0] === 'aſſ' && !
                /[[a]][[^K]]s/iv.test('aKs')
        "#,
    );
    assert_true(
        r#"
            /[\P{Ll}&&A][[s]]x/v.test('Asx') && !/[\P{Ll}&&A][[s]]x/iv.test('Asx') &&
                /[[\p{ASCII}]][[a]]x/v.test('Aax') && !/[[\p{ASCII}]][[a]]x/v.test('Kax')
        "#,
    );
    assert_true(
        r#"
            let r = /[[\p{Any}]][[\p{Any}]]a/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁a!');
            m.index === 0 && m[0] === '😀😁a' && m.indices[0][1] === 5 && r.lastIndex === 5 &&
                /[[😀]][[😁]]😂/v.test('😀😁😂') && /[[\u{1f600}]][[\u{1f601}]]\uD83D\uDE02/v.test(
                    '😀😁😂')
        "#,
    );
    assert_true(
        r#"
            let r = /[[\u{1f600}]][[\u{1f601}]]\u{1f602}/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁😂!');
            m.index === 0 && m[0] === '😀😁😂' && m.indices[0][1] === 6 && r.lastIndex === 6
        "#,
    );
    assert_true(
        r#"
            let r = /[[\uD800]][[\uD800]]\uD800/dv,
                m = r.exec('A\uD800\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800\uD800' && m.indices[0][1] === 4 && !
                /[[a]][[\uD83D]]\uDE00/v.test('a😀')
        "#,
    );
    assert_true(
        r#"
            /[[a]][[b]]\[/v.test('ab[') && /[[a]][[b]]\//v.test('ab/') && /[[a]][[b]]\//v.source ===
                '[[a]][[b]]\\/' && /^[[a]][[b]]\$$/v.test('ab$') && !/^[[a]][[b]]\$$/v.test('ab$\n')
        "#,
    );
    assert_true(
        r#"
            let r = /^[[a]][[b]]c$/dvmg,
                m = r.exec('x\r\nabc\u2028de');
            m.index === 3 && m[0] === 'abc' && m.indices[0][1] === 6 && r.lastIndex === 6 && r.exec(
                'x\r\nabc\u2028de') === null && r.lastIndex === 0 && !/^[[a]][[b]]c$/v.test('abc\n')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([[a]][[a]]c))/dvg,
                a = [...'aaac aaac'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aac' && a[0][1] === 'aac' && a[0][2] === 'aac' && a[0].index ===
                1 && a[0].groups.whole === 'aac' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 4 && a[1].index === 6 && a[1].indices[2][1] === 9 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'aaac aaac'.replace(/(?<whole>([[a]][[a]]c))/vg,'<$1:$<whole>>')==='a<aac:aac> a<aac:aac>'&&'aaac'.split(/([[a]][[a]]c)/v).join('|')==='a|aac|'&&'aaac'.match(/[[a]][[a]]c/v)[0]==='aac'",
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaac aaac'.replace(/(?<whole>([[a]][[a]]c))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX aX' && seen === '1:aac;6:aac;'
        "#,
    );
    assert_true(r"let r=/[[a]][[a]]c/vg;r.lastIndex=4;'aaac'.search(r)===1&&r.lastIndex===4");
    assert_true(
        r#"
            let trace = '',
                r = /[[\p{Any}]][[\p{Any}]]a/vy;
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
                r = /[[a]][[a]]c/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aaac');
            m.index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[[K]][[s]]a/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '[[K]][[s]]a' && c.unicodeSets && !c.ignoreCase && c
                .exec('KſA') === null && c.lastIndex === 0 && r.exec('KſA')[0] === 'KſA'
        "#,
    );
    assert_true(
        r#"
            let a = /[[a]][[a]]c/vy,
                b = /[[a]][[a]]c/vy;
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
                a.exec('aaac')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('aaac')
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
                eval('marker=1; /[a--a][b&&]c/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/[[a]][[b]][[c]]/v.test('abc')");
    assert_true(
        r#"
            let r = /[[a]][[b]][[c]]/v,
                m = r.exec('abc');
            m.length === 1 && m[0] === 'abc' && m.index === 0 && m.input === 'abc' && m.groups ===
                undefined && m.indices === undefined && r.source === '[[a]][[b]][[c]]' && r.lastIndex ===
                0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !r.test('ab')
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
    assert_true(r"/[[a]][b]c/v.test('abc')");
    assert_true(
        r#"
            let r = /[[a]][b]c/v,
                m = r.exec('abc');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'abc' && m.index === 0 && m
                .input === 'abc' && m.groups === undefined && m.indices === undefined && r.source ===
                '[[a]][b]c' && r.lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !
                r.test('ab') && r.exec('abcx')[0] === 'abc' && r.exec('abcx').index === 0 && r.exec('xabc')
                .index === 1
        "#,
    );
    assert_true(r"/[a][[b]]c/v.test('abc')");
    assert_true(
        r#"
            let r = /[a][[b]]c/v,
                m = r.exec('abc');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'abc' && m.index === 0 && m
                .input === 'abc' && m.groups === undefined && m.indices === undefined && r.source ===
                '[a][[b]]c' && r.lastIndex === 0 && !r.test('xbc') && !r.test('axc') && !r.test('abx') && !
                r.test('ab') && r.exec('abcx')[0] === 'abc' && r.exec('abcx').index === 0 && r.exec('xabc')
                .index === 1
        "#,
    );
    for source in [
        r"/[[a]][[b]]cd/v.test('abcd')",
        r"/[[a]][[b]]^c/v.test('abc')",
        r"/[[a]]x[[b]]c/v.test('axbc')",
        r"/[[a]][[b]]\bc/v.test('abc')",
        r"/([[a]])[[b]]c/v.test('abc')",
        r"/[[a]]([[b]])c/v.test('abc')",
        r"/[[a]][[b]](c)/v.test('abc')",
        r"/[[a]]+[[b]]c/v.test('abc')",
        r"/[[a]][[b]]?c/v.test('abc')",
        r"/[[a]][[b]]c+/v.test('abc')",
        r"/(?:[[a]][[b]]c)+/v.test('abc')",
        r"/[a--a][[\q{b}]]c/v.test('abc')",
        r"/[[\q{a}]][b--b]c/v.test('abc')",
        r"/[a--a][[\p{RGI_Emoji}]]c/v.test('abc')",
        r"/[[\p{RGI_Emoji}]][b--b]c/v.test('abc')",
        r"/c[[a]][[b]]d/v.test('cabd')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/[[a]][[a]]c/vg,sticky=/[[a]][[a]]a/vy,marker=0")
        .unwrap();
    // Both checked classes succeed while the final literal fails.
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
fn checked_v_set_pair_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start) in [
        (r"(?<whole>([[a]][[a]]c))", JsString::from("aaac"), "dvg", 0),
        (r"(?<whole>([[a]][[a]]c))", JsString::from("aaac"), "dvy", 0),
        (r"(?<whole>([[a]][[a]]c))", JsString::from("aaac"), "dvy", 1),
        (
            r"(?<whole>([[K]][[s]]a))",
            JsString::from("KſA!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>([\P{Ll}&&A][[s]]x))",
            JsString::from("Asx"),
            "divg",
            0,
        ),
        (
            r"(?<whole>([[\p{Any}]][[\p{Any}]]a))",
            JsString::from("😀😁a!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>([[\u{1f600}]][[\u{1f601}]]\u{1f602}))",
            JsString::from("x😀😁😂!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>([[\uD800]][[\uD800]]\uD800))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>([[a]][[^K]]s))",
            JsString::from("aKsaſſ!"),
            "divg",
            0,
        ),
        (
            r"^(?<whole>([[a]][[b]]c))$",
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
