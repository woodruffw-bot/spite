//! Match results and consumers for pairs of character classes.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn flat_then_checked_v_set_pair_state_context_and_consumers() {
    assert_true(
        r#"
            /[a][b--c]/v.test('ab') && !/[a][b--c]/v.test('ac') && /[a][c--b]/v.test('ac') && !/[a][c--b]/v
                .test('ab') && /[abc][[d]]/v.test('ad') && /[a-c][[d]]/iv.test('aD') && !/[a-c][[d]]/v
                .test('aD')
        "#,
    );
    assert_true(r#"let marker=0;eval("marker=1; /[a][b--c]/v.test('ab')")&&marker===1"#);
    assert_true(
        r#"
            let g = /[a][c--b]/vg,
                y = /[a][c--b]/vy,
                a = g.exec('aac');
            a.index === 1 && a[0] === 'ac' && g.lastIndex === 3 && g.exec('aac') === null && g
                .lastIndex === 0 && y.exec('aac') === null && y.lastIndex === 0 && (y.lastIndex = 1, y
                    .exec('aac')[0] === 'ac') && y.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            /[K][[s]]/iv.test('Kſ') && !/[K][[s]]/v.test('Kſ') && !/[a][[^K]]/iv.test('aK') &&
                /[a][[^K]]/iv.exec('aKaſ')[0] === 'aſ' && /[\p{ASCII}][[a]]/v.test('Aa') && !
                /[\p{ASCII}][[a]]/v.test('Ka')
        "#,
    );
    assert_true(
        r#"
            /[A][\P{Ll}&&A]/v.test('AA') && !/[A][\P{Ll}&&A]/iv.test('AA') && !/[][c--b]/v.test('ac') && !
                /[a][b--b]/v.test('a') && /[a][[^]]/v.test('a😀') && /[\]][[\[]]/v.test('][')
        "#,
    );
    assert_true(
        r#"
            let r = /[\p{Any}][[a]]/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀a!');
            m.index === 0 && m[0] === '😀a' && m.indices[0][1] === 3 && r.lastIndex === 3 &&
                /[\u{1f600}][[\u{1f601}]]/v.exec('x😀😁').index === 1
        "#,
    );
    assert_true(
        r#"
            let r = /[\uD800][[\uD800]]/dv,
                m = r.exec('A\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800' && m.indices[0][1] === 3 && !/[\uD800][[\uDC00]]/v
                .test('\uD800\uDC00')
        "#,
    );
    assert_true(
        r#"
            let r = /^[a][[b]]$/dvmg,
                m = r.exec('x\r\nab\u2028cd');
            m.index === 3 && m[0] === 'ab' && m.indices[0][1] === 5 && r.lastIndex === 5 && r.exec(
                'x\r\nab\u2028cd') === null && r.lastIndex === 0 && !/^[a][[b]]$/v.test('ab\n')
        "#,
    );
    assert_true(r"/^[a][[\$]]$/v.test('a$')&&!/^[a][[\$]]$/v.test('a$\n')");
    assert_true(
        r#"
            let r = /(?<whole>([a][c--b]))/dvg,
                a = [...'aac aac'.matchAll(r)];
            a.length === 2 && a[0][0] === 'ac' && a[0][1] === 'ac' && a[0][2] === 'ac' && a[0].index ===
                1 && a[0].groups.whole === 'ac' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 3 && a[1].index === 5 && a[1].indices[2][1] === 7 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'aac aac'.replace(/(?<whole>([a][c--b]))/vg,'<$1:$<whole>>')==='a<ac:ac> a<ac:ac>'&&'aac'.split(/([a][c--b])/v).join('|')==='a|ac|'&&'aac'.match(/[a][c--b]/v)[0]==='ac'",
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aac aac'.replace(/(?<whole>([a][c--b]))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX aX' && seen === '1:ac;5:ac;'
        "#,
    );
    assert_true(r"let r=/[a][c--b]/vg;r.lastIndex=4;'aac'.search(r)===1&&r.lastIndex===4");
    assert_true(
        r#"
            let trace = '',
                r = /[\p{Any}][[a]]/vy;
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
            trace === 'si' && m.index === 0 && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[a][c--b]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aac');
            m.index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[K][[s]]/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '[K][[s]]' && c.unicodeSets && !c.ignoreCase && c.exec(
                'Kſ') === null && c.lastIndex === 0 && r.exec('Kſ')[0] === 'Kſ'
        "#,
    );
    assert_true(
        r#"
            let a = /[a][c--b]/vy,
                b = /[a][c--b]/vy;
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
                a.exec('aac')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('aac')
            } catch (e) {
                y = e instanceof TypeError
            }
            x && y && a.lastIndex === 1 && b.lastIndex === 0
        "#,
    );
    assert_true(
        r"/\b[a][c--b]/v.test('ac')&&!/\b[a][c--b]/v.test('ab')&&!/\b[a][c--b]/v.test('xac')",
    );
    assert_true(
        r"/[a][c--b]\b/v.test('ac')&&!/[a][c--b]\b/v.test('ab')&&!/[a][c--b]\b/v.test('acx')",
    );
    assert_true(r"/[a][b]/v.test('ab')&&!/[a][b]/v.test('ac')");
    assert_true(r"/[a][c--b][d]/v.test('acd')");
    assert_true(
        r#"
            let r = /[a][c--b][d]/v,
                m = r.exec('acd');
            m !== null && m.length === 1 && m[0] === 'acd' && m.index === 0 && m.input === 'acd' && m
                .groups === undefined && m.indices === undefined && r.source === '[a][c--b][d]' && r
                .lastIndex === 0 && !r.test('xcd') && !r.test('axd') && !r.test('acx') && !r.test('ac') &&
                r.exec('xacd').index === 1 && r.exec('acdx').index === 0 && r.lastIndex === 0
        "#,
    );
    assert_true(r"/c[a][c--b]/v.test('cac')");
    assert_true(
        r#"
            let r = /c[a][c--b]/v,
                m = r.exec('cac');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'cac' && m.index === 0 && m
                .input === 'cac' && m.groups === undefined && m.indices === undefined && r.source ===
                'c[a][c--b]' && r.lastIndex === 0 && !r.test('xac') && !r.test('cxc') && !r.test('cax') &&
                !r.test('ca') && r.test('cacx') && r.exec('xcac').index === 1
        "#,
    );
    assert_eq!(
        Realm::default().eval(r"/[][[\q{c}]]/v.test('ac')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a]^[b--c]/v.test('ac')",
        r"/([a])[b--c]/v.test('ac')",
        r"/[a]([b--c])/v.test('ac')",
        r"/[a]+[b--c]/v.test('ac')",
        r"/[a][c--b]?/v.test('ac')",
        r"/[][[\p{RGI_Emoji}]]/v.test('ac')",
        r"/[\p{RGI_Emoji}][[b]]/v.test('ab')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/[a][c--b]/vg,sticky=/[a][[a]]/vy,marker=0")
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
fn flat_then_checked_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full field objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start) in [
        (r"(?<whole>([a][c--b]))", JsString::from("aac"), "dvg", 0),
        (r"(?<whole>([a][c--b]))", JsString::from("aac"), "dvy", 0),
        (r"(?<whole>([a][c--b]))", JsString::from("aac"), "dvy", 1),
        (r"(?<whole>([K][[s]]))", JsString::from("Kſ!"), "divg", 0),
        (
            r"(?<whole>([A][\P{Ll}&&A]))",
            JsString::from("AA"),
            "divg",
            0,
        ),
        (
            r"(?<whole>([\p{Any}][[a]]))",
            JsString::from("😀a!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>([\u{1f600}][[\u{1f601}]]))",
            JsString::from("x😀😁!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>([\uD800][[\uD800]]))",
            JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
            "dvg",
            0,
        ),
        (r"(?<whole>([a][[^K]]))", JsString::from("aKaſ!"), "divg", 0),
        (
            r"^(?<whole>([a][[b]]))$",
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
