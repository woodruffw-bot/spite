//! Match results and consumers for character classes and literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_literal_v_set_state_context_and_consumers() {
    assert_true(r"!/c[a--a--\q{a}]/v.test('ca')");
    assert_true(
        r#"
            let marker = 0,
                result, caught = false;
            try {
                result = eval("marker=1; /c[a--b]/v.test('ca')")
            } catch {
                caught = true
            } finally {
                marker = 3
            }!caught && marker === 3 && result && !/c[a--b]/v.test('cb') && !/c[a--b]/v.test('c') && !
                /c[a--b]/v.test('cba') && /c[a--b]/v.exec('ccca').index === 2
        "#,
    );
    assert_true(
        r#"
            let r = /c[a--b]/dvy;
            r.exec('ccca') === null && r.lastIndex === 0 && (r.lastIndex = 2, r.exec('ccca')[0] ===
                'ca') && r.lastIndex === 4 && r.exec('ccca') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(k[[s]]))/div,
                m = r.exec('Kſ!');
            m.length === 3 && m[0] === 'Kſ' && m[1] === 'Kſ' && m[2] === 'Kſ' && m.groups.whole === 'Kſ' &&
                m.indices.groups.whole === m.indices[1] && m.indices[2][1] === 2 && /k[[s]]/v.test('ks') &&
                !/k[[s]]/v.test('Ks') && !/k[[s]]/v.test('kſ')
        "#,
    );
    assert_true(
        r#"
            let r = /\u{1f600}[[\p{Any}]]/dvy;
            r.lastIndex = 1;
            let a = r.exec('😀😁😀😁'),
                b = r.exec('😀😁😀😁');
            a.index === 0 && a[0] === '😀😁' && a.indices[0][1] === 4 && b.index === 4 && r.lastIndex ===
                8 && r.exec('😀😁😀😁') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\uD800[[a]]/dv,
                m = r.exec('\uD800a!');
            m[0] === '\uD800a' && m.index === 0 && m.indices[0][1] === 2 && /\uD83D[[\uDE00]]/v.exec(
                '😀') === null
        "#,
    );
    assert_true(
        r#"
            /\[[[a]]/v.exec('x[a!')[0] === '[a' && /\/[[a]]/v.test('/a') && /\/[[a]]/v.source ===
                '\\/[[a]]' && /^\$[[a]]$/v.test('$a') && !/^\$[[a]]$/v.test('$a\n')
        "#,
    );
    assert_true(
        r#"
            let r = /^(?<whole>(c[[a]]))$/dvmg,
                m = r.exec('x\r\nca\u2028cb');
            m[0] === 'ca' && m[1] === 'ca' && m[2] === 'ca' && m.index === 3 && m.indices[2][0] === 3 && m
                .indices[2][1] === 5 && r.lastIndex === 5 && r.exec('x\r\nca\u2028cb') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            !/c[a--a]/v.test('ca') && !/c[a--a]/v.test('') && /c[^[[a--a]]]/v.test('ca') && /c[^[[a--a]]]/v
                .test('c😀') && !/c[^[[a--a]]]/v.test('c')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(c[[a]]))/dvg,
                a = [...'😀ca cca'.matchAll(r)];
            a.length === 2 && a[0][0] === 'ca' && a[0][1] === 'ca' && a[0][2] === 'ca' && a[0].index ===
                2 && a[0].indices.groups.whole === a[0].indices[1] && a[1].groups.whole === 'ca' && a[1]
                .index === 6 && a[1].indices[2][1] === 8 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            '😀ca cca'.replace(/(?<whole>(c[[a]]))/vg, '<$1:$<whole>>') === '😀<ca:ca> c<ca:ca>' && '😀cax'
                .split(/(c[[a]])/v).join('|') === '😀|ca|x' && 'ccca'.match(/c[a--b]/v)[0] === 'ca'
        "#,
    );
    assert_true(
        r#"
            let seen = '',
                out = '😀ca cca'.replace(/(?<whole>(c[[a]]))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return '!'
                });
            out === '😀! c!' && seen === '2:ca;6:ca;'
        "#,
    );
    assert_true(r"let r=/c[[a]]/vg;r.lastIndex=1;'😀ccca'.search(r)===4&&r.lastIndex===1");
    assert_true(
        r#"
            let trace = '',
                r = /\u{1f600}[[a]]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀a'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀a' && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /c[[a]]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('ccca');
            m.index === 2 && m[0] === 'ca' && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /k[[s]]/iv,
                c = new RegExp(r, 'dvy');
            c.source === 'k[[s]]' && c.unicodeSets && !c.ignoreCase && c.exec('Kſ') === null && c
                .lastIndex === 0 && r.exec('Kſ')[0] === 'Kſ'
        "#,
    );
    assert_true(
        r#"
            let r = /c[[a]]/vy;
            r.lastIndex = 2;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('ccca')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let marker = 0,
                caught = false;
            try {
                eval('marker=1; /c[a&&]/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/c[a--b][c]/v.test('cac')");
    assert_true(
        r#"
            let r = /c[a--b][c]/v,
                m = r.exec('cac');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'cac' && m.index === 0 && m
                .input === 'cac' && m.groups === undefined && m.indices === undefined && r.source ===
                'c[a--b][c]' && r.lastIndex === 0 && !r.test('xac') && !r.test('cxc') && !r.test('cax') &&
                !r.test('ca') && r.test('cacx') && r.exec('xcac').index === 1
        "#,
    );
    for source in [
        r"/cc[a--b]/v.test('cca')",
        r"/c[a--b]+/v.test('ca')",
        r"/(c)[a--b]/v.test('ca')",
        r"/c([a--b])/v.test('ca')",
        r"/(c[a--b])+/v.test('ca')",
        r"/c^[a--b]/v.test('ca')",
        r"/c$[a--b]/v.test('ca')",
        r"/c[\p{RGI_Emoji}&&a]/v.test('ca')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/c[a--b]/vg,marker=0").unwrap();
    // Every literal prefix succeeds while its checked class fails. Build the
    // input outside JavaScript's opted-in string-construction work.
    let setup = format!("var text='{}';r.lastIndex=1", "c".repeat(20_000));
    realm.eval(&setup).unwrap();
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
fn checked_literal_v_set_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, flags, start) in [
        (r"(?<whole>(c[a--b]))", JsString::from("ccca"), "dvg", 0),
        (r"(?<whole>(c[a--b]))", JsString::from("ccca"), "dvy", 0),
        (r"(?<whole>(k[[s]]))", JsString::from("Kſ!"), "divg", 0),
        (r"(?<whole>(k[[s]]))", JsString::from("kſ!"), "dvg", 0),
        (
            r"(?<whole>(\u{1f600}[[\p{Any}]]))",
            JsString::from("😀😁!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\uD800[[a]]))",
            JsString::from_code_units(vec![0xd800, 0x61, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\uD83D[[\uDE00]]))",
            JsString::from("😀"),
            "dvg",
            0,
        ),
        (r"(?<whole>(\[[[a]]))", JsString::from("x[a!"), "dvg", 0),
        (
            r"^(?<whole>(c[[a]]))$",
            JsString::from("x\r\nca\u{2028}"),
            "dvmg",
            0,
        ),
        (r"^(?<whole>(\$[[a]]))$", JsString::from("$a"), "dvg", 0),
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
            "{pattern:?} flags={flags:?} input={input:?} lastIndex={start}\n{result}\n",
            result = result.to_utf8().expect("JSON output must be valid UTF-8"),
        )
        .unwrap();
        row_count += 1;
    }
    assert_eq!(row_count, 10);
    insta::assert_snapshot!(rows);
}
