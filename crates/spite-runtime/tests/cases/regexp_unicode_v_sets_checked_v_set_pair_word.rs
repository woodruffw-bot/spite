//! Match results and consumers for pairs of character classes next to word assertions.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_pair_word_state_context_and_consumers() {
    // ECMA-262 22.2.2.4 tests these assertions at the complete pair's edges.
    // A failed assertion must retry at the first character's end: the valid
    // overlapping pair begins at 1, although the first pair ends at 2.
    assert_true(
        r#"
            let a = /\B[[a]]a\b/dvg,
                b = /\Ba[[a]]\b/dvg,
                x = a.exec('aaa!'),
                y = b.exec('aaa!');
            x[0] === 'aa' && x.index === 1 && x.indices[0][1] === 3 && a.lastIndex === 3 && y[0] ===
                'aa' && y.index === 1 && y.indices[0][1] === 3 && b.lastIndex === 3 && a.exec('aaa!') ===
                null && a.lastIndex === 0 && b.exec('aaa!') === null && b.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let a = /\B[[a]]a\b/vy,
                b = /\Ba[[a]]\b/vy;
            a.exec('aaa!') === null && a.lastIndex === 0 && b.exec('aaa!') === null && b.lastIndex === 0 &&
                (a.lastIndex = 1, b.lastIndex = 1, a.exec('aaa!')[0] === 'aa') && a.lastIndex === 3 && b
                .exec('aaa!')[0] === 'aa' && b.lastIndex === 3
        "#,
    );
    for (source, input, nonmember) in [(r"\b[a--b]c", "ac", "bc"), (r"\bc[a--b]", "ca", "cb")] {
        let code = JsString::from(format!("marker=1; /{source}/v.test('{input}');").as_str());
        assert_true(&format!(
            "let marker=0,result,caught=false;try{{result=eval({code:?})}}catch{{caught=true}}finally{{marker=3}}!caught&&marker===3&&result&&!/{source}/v.test('{nonmember}')"
        ));
    }
    // Any is fold-invariant, but WordCharacters still depends on actual i.
    assert_true(
        r#"
            /\B[[\p{Any}]]a\b/v.test('Ka!') && !/\B[[\p{Any}]]a\b/iv.test('Ka!') && !/\b[[\p{Any}]]a\b/v
                .test('Ka!') && /\b[[\p{Any}]]a\b/iv.test('Ka!') && /\b[[\p{Any}]]a\b/iv.test('ſa!') && !
                /\b[[\p{Any}]]a\b/v.test('ſa!')
        "#,
    );
    assert_true(
        r#"
            !/\ba[[\p{Any}]]\b/v.test('aK!') && /\ba[[\p{Any}]]\b/iv.test('aK!') && !/\ba[[\p{Any}]]\b/v
                .test('aſ!') && /\ba[[\p{Any}]]\b/iv.test('aſ!') && /\b[[K]]s\b/iv.test('Kſ!') &&
                /\bk[[s]]\b/iv.test('Kſ!') && !/\b[[K]]s\b/v.test('Kſ!') && !/\bk[[s]]\b/v.test('Ks!')
        "#,
    );
    assert_true(
        r#"
            /[[a]]c\b/v.exec('xac!').index === 1 && /a[[a]]\b/v.exec('aaa!').index === 1 && /\B[[!]]\?\B/v
                .test('!?') && /\B\?[[!]]\B/v.test('?!') && !/\b[a--a]c/v.test('ac') && !/\bc[a--a]/v.test(
                    'ca')
        "#,
    );
    assert_true(
        r#"
            let a = /\B[[\p{Any}]]a\b/dvy,
                b = /\B\u{1f600}[[a]]\b/dvy;
            a.lastIndex = 1;
            b.lastIndex = 1;
            let x = a.exec('😀a!'),
                y = b.exec('😀a!');
            x.index === 0 && x[0] === '😀a' && x.indices[0][0] === 0 && x.indices[0][1] === 3 && a
                .lastIndex === 3 && y.index === 0 && y[0] === '😀a' && y.indices[0][1] === 3 && b
                .lastIndex === 3
        "#,
    );
    assert_true(
        r#"
            let a = /\b[[\uD800]]a\b/dv,
                b = /\b\uD800[[a]]\b/dv,
                x = a.exec('A\uD800a!'),
                y = b.exec('A\uD800a!');
            x.index === 1 && x[0] === '\uD800a' && x.indices[0][1] === 3 && y.index === 1 && y[0] ===
                '\uD800a' && y.indices[0][1] === 3 && /\B[[\uD83D]]\uDE00\B/v.exec('😀') === null &&
                /\B\uD83D[[\uDE00]]\B/v.exec('😀') === null
        "#,
    );
    assert_true(
        r#"
            /^\b[[a]]\$\B$/v.test('a$') && !/^\b[[a]]\$\B$/v.test('a$\n') && /^\B\$[[a]]\b$/v.test('$a') &&
                !/^\B\$[[a]]\b$/v.test('$a\n')
        "#,
    );
    assert_true(
        r#"
            let a = /^\b[[a]]c\b$/dvmg,
                b = /^\bc[[a]]\b$/dvmg,
                x = a.exec('x\r\nac\u2028bc'),
                y = b.exec('x\r\nca\u2028cb');
            x.index === 3 && x[0] === 'ac' && x.indices[0][1] === 5 && a.lastIndex === 5 && y.index ===
                3 && y[0] === 'ca' && y.indices[0][1] === 5 && b.lastIndex === 5 && a.exec(
                    'x\r\nac\u2028bc') === null && a.lastIndex === 0 && b.exec('x\r\nca\u2028cb') ===
                null && b.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\B[[a]]a\b))/dvg,
                a = [...'aaa! aaa!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aa' && a[0][1] === 'aa' && a[0][2] === 'aa' && a[0].index ===
                1 && a[0].indices.groups.whole === a[0].indices[1] && a[0].indices[2][1] === 3 && a[1]
                .groups.whole === 'aa' && a[1].index === 6 && a[1].indices[2][1] === 8 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            'aaa! aaa!'.replace(/(?<whole>(\B[[a]]a\b))/vg, '<$1:$<whole>>') === 'a<aa:aa>! a<aa:aa>!' &&
                'aaa!'.split(/(\Ba[[a]]\b)/v).join('|') === 'a|aa|!' && 'aaa!'.match(/\Ba[[a]]\b/v)[0] ===
                'aa'
        "#,
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaa! aaa!'.replace(/(?<whole>(\Ba[[a]]\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:aa;6:aa;'
        "#,
    );
    assert_true(r"let r=/\B[[a]]a\b/vg;r.lastIndex=4;'aaa!'.search(r)===1&&r.lastIndex===4");
    assert_true(
        r#"
            let trace = '',
                r = /\B\u{1f600}[[a]]\b/vy;
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
                r = /\B[[a]]a\b/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aaa!');
            m.index === 1 && m[0] === 'aa' && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[K]]s\b/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '\\b[[K]]s\\b' && c.unicodeSets && !c.ignoreCase && c
                .exec('Kſ') === null && c.lastIndex === 0 && r.exec('Kſ')[0] === 'Kſ'
        "#,
    );
    assert_true(
        r#"
            let a = /\B[[a]]a\b/vy,
                b = /\Ba[[a]]\b/vy;
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
    assert_true(r"/\ba[[a]][c]/v.test('aac')");
    assert_true(
        r#"
            let r = /\ba[[a]][c]/v,
                m = r.exec('aac');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'aac' && m.index === 0 && m
                .input === 'aac' && m.groups === undefined && m.indices === undefined && r.source ===
                '\\ba[[a]][c]' && r.lastIndex === 0 && !r.test('xac') && !r.test('axc') && !r.test(
                'aax') && !r.test('aa') && r.test('aacx') && !r.test('xaac') && r.exec('!aac').index === 1
        "#,
    );
    assert_eq!(
        Realm::default().eval(r"/\b[[a--a]&&[\q{a}]]c/v.test('ac')"),
        Ok(Value::Boolean(false))
    );
    for source in [
        r"/\b^[[a]]c/v.test('ac')",
        r"/[[a]]c$\b/v.test('ac')",
        r"/\b\b[[a]]c/v.test('ac')",
        r"/\b[[a]]\Bc/v.test('ac')",
        r"/\ba[[a]]+/v.test('aa')",
        r"/(\b[[a]])c/v.test('ac')",
        r"/\ba([[a]])/v.test('aa')",
        r"/(\b[[a]]c)+/v.test('ac')",
        r"/\b[[a]]cd/v.test('acd')",
        r"/\ba[[a--a]&&[\p{RGI_Emoji}]]/v.test('aa')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r"var r=/\B[[a]]a\b/vg,marker=0").unwrap();
    // Every pair before the final one fails an outside assertion. Build this
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
fn checked_v_set_pair_word_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // These full expected objects are specified independently of the matcher.
    // Explicit UTF-16 input units keep lone surrogates and start offsets visible.
    for (pattern, input, flags, start) in [
        (r"(?<whole>(\B[[a]]a\b))", JsString::from("aaa!"), "dvg", 0),
        (r"(?<whole>(\Ba[[a]]\b))", JsString::from("aaa!"), "dvg", 0),
        (r"(?<whole>(\B[[a]]a\b))", JsString::from("aaa!"), "dvy", 0),
        (
            r"(?<whole>(\B[[\p{Any}]]a\b))",
            JsString::from("Ka!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\B[[\p{Any}]]a\b))",
            JsString::from("Ka!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\B\u{1f600}[[a]]\b))",
            JsString::from("😀a!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\b[[\uD800]]a\b))",
            JsString::from_code_units(vec![0x41, 0xd800, 0x61, 0x21]),
            "dvg",
            0,
        ),
        (r"(?<whole>(\b[[a]]\$\B))", JsString::from("a$!"), "dvg", 0),
        (r"(?<whole>(\B\$[[a]]\b))", JsString::from("$a!"), "dvg", 0),
        (
            r"^(?<whole>(\b[[a]]c\b))$",
            JsString::from("x\r\nac\u{2028}bc"),
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
