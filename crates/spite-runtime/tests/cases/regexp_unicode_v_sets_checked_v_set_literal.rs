//! Match results and consumers for character classes and literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_literal_state_context_and_consumers() {
    assert_true(
        r#"
            /[a--b]c/v.test('ac') && !/[a--b]c/v.test('bc') && !/[a--b]c/v.test('a') && !/[a--b]c/v.test(
                'abc') && /[a--b]c/v.exec('aac').index === 1
        "#,
    );
    assert_true(
        r#"
            let r = /[a--b]c/dvy;
            r.exec('aac') === null && r.lastIndex === 0 && (r.lastIndex = 1, r.exec('aac')[0] === 'ac') &&
                r.lastIndex === 3 && r.exec('aac') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([[K]]s))/div,
                m = r.exec('Kſ!');
            m.length === 3 && m[0] === 'Kſ' && m[1] === 'Kſ' && m[2] === 'Kſ' && m.groups.whole === 'Kſ' &&
                m.indices.groups.whole === m.indices[1] && m.indices[2][1] === 2 && /[[K]]s/v.test('Ks') &&
                !/[[K]]s/v.test('Ks') && !/[[K]]s/v.test('Kſ')
        "#,
    );
    assert_true(
        r#"
            let r = /[[\p{Any}]]\u{1f601}/dvy;
            r.lastIndex = 1;
            let a = r.exec('😀😁😀😁'),
                b = r.exec('😀😁😀😁');
            a.index === 0 && a[0] === '😀😁' && a.indices[0][1] === 4 && b.index === 4 && r.lastIndex ===
                8 && r.exec('😀😁😀😁') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /[[\uD800]]a/dv,
                m = r.exec('\uD800a!');
            m[0] === '\uD800a' && m.index === 0 && m.indices[0][1] === 2 && /[[\uD83D]]\uDE00/v.exec(
                '😀') === null
        "#,
    );
    assert_true(
        r#"
            /[[\]]]\$/v.exec('x]$!')[0] === ']$' && /^[[a]]\$$/v.test('a$') && !/^[[a]]\$$/v.test(
                'a$\n') && /[[a]]\]/v.test('a]')
        "#,
    );
    assert_true(
        r#"
            let r = /^(?<whole>([[a]]c))$/dvmg,
                m = r.exec('x\r\nac\u2028bc');
            m[0] === 'ac' && m[1] === 'ac' && m[2] === 'ac' && m.index === 3 && m.indices[2][0] === 3 && m
                .indices[2][1] === 5 && r.lastIndex === 5 && r.exec('x\r\nac\u2028bc') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>([[a]]c))/dvg,
                a = [...'😀ac aac'.matchAll(r)];
            a.length === 2 && a[0][0] === 'ac' && a[0][1] === 'ac' && a[0][2] === 'ac' && a[0].index ===
                2 && a[0].indices.groups.whole === a[0].indices[1] && a[1].groups.whole === 'ac' && a[1]
                .index === 6 && a[1].indices[2][1] === 8 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r"'😀ac aac'.replace(/(?<whole>([[a]]c))/vg,'<$1:$<whole>>')==='😀<ac:ac> a<ac:ac>'&&'😀acx'.split(/([[a]]c)/v).join('|')==='😀|ac|x'&&'aac'.match(/[a--b]c/v)[0]==='ac'",
    );
    assert_true(
        r#"
            let seen = '',
                out = '😀ac aac'.replace(/(?<whole>([[a]]c))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return '!'
                });
            out === '😀! a!' && seen === '2:ac;6:ac;'
        "#,
    );
    assert_true(r"let r=/[[a]]c/vg;r.lastIndex=1;'😀aac'.search(r)===3&&r.lastIndex===1");
    assert_true(
        r#"
            let trace = '',
                r = /[[\p{Any}]]c/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀c'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀c' && r.lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /[[a]]c/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aac');
            m.index === 1 && m[0] === 'ac' && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /[[K]]s/iv,
                c = new RegExp(r, 'dvy');
            c.source === '[[K]]s' && c.unicodeSets && !c.ignoreCase && c.exec('Kſ') === null && c
                .lastIndex === 0 && r.exec('Kſ')[0] === 'Kſ'
        "#,
    );
    assert_true(
        r#"
            let r = /[[a]]c/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('aac')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let marker = 0,
                caught = false;
            try {
                eval('marker=1; /[a&&]c/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_eq!(
        Realm::default().eval(r"/[a--a--\q{a}]c/v.test('ac')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/[a--b]+c/v.test('ac')",
        r"/[a--b]cd/v.test('acd')",
        r"/([a--b])c/v.test('ac')",
        r"/[a--b](c)/v.test('ac')",
        r"/([a--b]c)+/v.test('ac')",
        r"/[a--b]^c/v.test('ac')",
        r"/[a--b]$c/v.test('ac')",
        r"/[\p{RGI_Emoji}&&a]c/v.test('ac')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/[a--b]c/vg,marker=0").unwrap();
    // Every prefix succeeds while its literal continuation fails. Build this
    // input outside JavaScript's opted-in string-construction work.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
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
fn checked_v_set_literal_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    for (pattern, input, flags, start) in [
        (r"(?<whole>([a--b]c))", JsString::from("aac"), "dvg", 0),
        (r"(?<whole>([a--b]c))", JsString::from("aac"), "dvy", 0),
        (r"(?<whole>([[K]]s))", JsString::from("Kſ!"), "divg", 0),
        (r"(?<whole>([[K]]s))", JsString::from("Kſ!"), "dvg", 0),
        (
            r"(?<whole>([[\p{Any}]]\u{1f601}))",
            JsString::from("😀😁!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>([[\uD800]]a))",
            JsString::from_code_units(vec![0xd800, 0x61, 0x21]),
            "dvg",
            0,
        ),
        (
            r"(?<whole>([[\uD83D]]\uDE00))",
            JsString::from("😀"),
            "dvg",
            0,
        ),
        (r"(?<whole>([[\]]]\$))", JsString::from("x]$!"), "dvg", 0),
        (
            r"^(?<whole>([[a]]c))$",
            JsString::from("x\r\nac\u{2028}"),
            "dvmg",
            0,
        ),
        (r"^(?<whole>([[a]]\$))$", JsString::from("a$"), "dvg", 0),
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
