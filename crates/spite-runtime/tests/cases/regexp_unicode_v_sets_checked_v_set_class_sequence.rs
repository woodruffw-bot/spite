//! Match results and consumers for sequences of character classes.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn checked_v_set_class_sequence_state_context_and_consumers() {
    assert_true(
        r#"
            /[a--b][[b]][[c]][[d]]/v.test('abcd') && !/[a--b][[b]][[c]][[d]]/v.test('xbcd') && !
                /[a--b][[b]][[c]][[d]]/v.test('axcd') && !/[a--b][[b]][[c]][[d]]/v.test('abxd') && !
                /[a--b][[b]][[c]][[d]]/v.test('abcx') && !/[a--b][[b]][[c]][[d]]/v.test('abc') && !
                /[a--a][[b]][[c]][[d]]/v.test('abcd') && !/[[a]][[b]][[c]][d--d]/v.test('abcd')
        "#,
    );
    assert_true(
        r#"
            let ok = true;
            for (let n of [5, 6, 17]) {
                let text = 'a'.repeat(n),
                    r = new RegExp('(?<whole>(' + '[[a]]'.repeat(n) + '))', 'dvy'),
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
    assert_true(
        r#"let marker=0;eval("marker=1; /\\b[[a]][[b]][[c]][[d]]\\b/v.test('abcd')")&&marker===1"#,
    );
    assert_true(
        r#"
            let a = /\B[[a]][[a]][[a]][[b]]\b/dvg,
                b = /\B[[a]][[a]][[a]][[ab]]\b/dvg,
                x = a.exec('aaaab!'),
                y = b.exec('aaaaa!');
            x.index === 1 && x[0] === 'aaab' && x.indices[0][1] === 5 && a.lastIndex === 5 && y.index ===
                1 && y[0] === 'aaaa' && y.indices[0][1] === 5 && b.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            let g = /\B[[a]][[a]][[a]][[b]]\b/vg,
                y = /\B[[a]][[a]][[a]][[b]]\b/vy,
                a = g.exec('aaaab!');
            a.index === 1 && a[0] === 'aaab' && g.lastIndex === 5 && g.exec('aaaab!') === null && g
                .lastIndex === 0 && y.exec('aaaab!') === null && y.lastIndex === 0 && (y.lastIndex = 1, y
                    .exec('aaaab!')[0] === 'aaab') && y.lastIndex === 5
        "#,
    );
    assert_true(
        r#"
            /\b[[a]][[b]][[c]][[d]]/v.test('abcd') && /\B[[a]][[b]][[c]][[d]]/v.test('xabcd') &&
                /[[a]][[b]][[c]][[d]]\b/v.test('abcd!') && /[[a]][[b]][[c]][[d]]\B/v.test('abcdx') &&
                /\b[[a]][[b]][[c]][[d]]\b/v.test('!abcd!') && /\b[[a]][[b]][[c]][[d]]\B/v.test('!abcdx') &&
                /\B[[a]][[b]][[c]][[d]]\b/v.test('xabcd!') && /\B[[a]][[b]][[c]][[d]]\B/v.test('xabcdx')
        "#,
    );
    assert_true(
        r#"
            !/\b[[a]][[b]][[c]][[d]]/v.test('xabcd') && !/[[a]][[b]][[c]][[d]]\b/v.test('abcdx') && !
                /\B[[a]][[b]][[c]][[d]]/v.test('!abcd') && !/[[a]][[b]][[c]][[d]]\B/v.test('abcd!')
        "#,
    );
    assert_true(
        r#"
            /\b[[K]][[s]][[a]][[b]]\b/iv.test('KſAb') && !/\b[[K]][[s]][[a]][[b]]\b/v.test('Ksab') && !
                /\b[[K]][[s]][[a]][[b]]\b/v.test('Kſab') && !/\b[[K]][[s]][[a]][[b]]\b/v.test('KsAb') &&
                /\b[[K]][[s]][[a]][[b]]\b/v.test('Ksab') && /^[[^K]][[s]][[a]][[b]]$/iv.test('ſSAb') && !
                /^[[^K]][[s]][[a]][[b]]$/iv.test('KSAb')
        "#,
    );
    assert_true(
        r#"
            /\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/iv.test('Kabcſ!') && !
                /\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/v.test('Kabcſ!') &&
                /\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]/iv.test('Kabcſ') && !
                /\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]/v.test('Kabcſ') &&
                /[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/iv.test('Kabcſ!') && !
                /[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b/v.test('Kabcſ!')
        "#,
    );
    assert_true(
        r"/\b[\P{Ll}&&A][[s]][[x]][[a]]\b/v.test('Asxa')&&!/\b[\P{Ll}&&A][[s]][[x]][[a]]\b/iv.test('Asxa')",
    );
    assert_true(
        r#"
            let r = /[[\p{Any}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁😂😃😄😅!');
            m.index === 0 && m[0] === '😀😁😂😃😄😅' && m.indices[0][1] === 12 && r.lastIndex === 12
        "#,
    );
    assert_true(
        r#"
            let r = /[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]]/dvy;
            r.lastIndex = 1;
            let m = r.exec('😀😁😂😃!');
            m.index === 0 && m[0] === '😀😁😂😃' && m.indices[0][1] === 8 && r.lastIndex === 8
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[\uD800]][[\uD800]][[\uD800]][[\uD800]][[\uD800]]\B/dv,
                m = r.exec('A\uD800\uD800\uD800\uD800\uD800!');
            m.index === 1 && m[0] === '\uD800\uD800\uD800\uD800\uD800' && m.indices[0][1] === 6 && !
                /[[\uD83D]][[\uDE00]][[a]][[b]]/v.test('😀ab') && !/[[a]][[\uD83D]][[\uDE00]][[b]]/v.test(
                    'a😀b')
        "#,
    );
    assert_true(
        r#"
            let r = /^\b[[a]][[b]][[c]][[d]]\b$/dvmg,
                m = r.exec('x\r\nabcd\u2028ef');
            m.index === 3 && m[0] === 'abcd' && m.indices[0][1] === 7 && r.lastIndex === 7 && r.exec(
                'x\r\nabcd\u2028ef') === null && r.lastIndex === 0 && !/^[[a]][[b]][[c]][[d]]$/v.test(
                'abcd\n') && /^[[a]][[b]][[c]][[d]]/v.test('abcd!') && /[[a]][[b]][[c]][[d]]$/v.test(
                'xabcd')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/dvg,
                a = [...'aaaab! aaaab!'.matchAll(r)];
            a.length === 2 && a[0][0] === 'aaab' && a[0][1] === 'aaab' && a[0][2] === 'aaab' && a[0]
                .index === 1 && a[0].groups.whole === 'aaab' && a[0].indices.groups.whole === a[0].indices[
                    1] && a[0].indices[2][1] === 5 && a[1].index === 8 && a[1].indices[2][1] === 12 && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            'aaaab! aaaab!'.replace(/(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/vg, '<$1:$<whole>>') ===
                'a<aaab:aaab>! a<aaab:aaab>!' && 'aaaab!'.split(/(\B[[a]][[a]][[a]][[b]]\b)/v).join(
                '|') === 'a|aaab|!' && 'aaaab!'.match(/\B[[a]][[a]][[a]][[b]]\b/v)[0] === 'aaab'
        "#,
    );
    assert_true(
        r#"
            let seen = '',
                out = 'aaaab! aaaab!'.replace(/(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/vg, (m, c1, c2, p,
                    text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return 'X'
                });
            out === 'aX! aX!' && seen === '1:aaab;8:aaab;'
        "#,
    );
    assert_true(
        r"let r=/\B[[a]][[a]][[a]][[b]]\b/vg;r.lastIndex=4;'aaaab!'.search(r)===1&&r.lastIndex===4",
    );
    assert_true(
        r#"
            let trace = '',
                r = /[[\p{Any}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]/vy;
            r.lastIndex = {
                valueOf() {
                    trace += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    trace += 's';
                    return '😀😁😂😃😄😅!'
                }
            });
            trace === 'si' && m.index === 0 && m[0] === '😀😁😂😃😄😅' && r.lastIndex === 12
        "#,
    );
    assert_true(
        r#"
            let seen = 0,
                r = /\B[[a]][[a]][[a]][[b]]\b/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 1
                }
            };
            let m = r.exec('aaaab!');
            m.index === 1 && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /\b[[K]][[s]][[a]][[b]]\b/iv,
                c = new RegExp(r, 'dvy');
            c.source === r.source && r.source === '\\b[[K]][[s]][[a]][[b]]\\b' && c.unicodeSets && !c
                .ignoreCase && c.exec('KſAb') === null && c.lastIndex === 0 && r.exec('KſAb')[0] === 'KſAb'
        "#,
    );
    assert_true(
        r#"
            let a = /\B[[a]][[a]][[a]][[b]]\b/vy,
                b = /\B[[a]][[a]][[a]][[b]]\b/vy;
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
                a.exec('aaaab!')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('aaaab!')
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
                eval('marker=1; /[a--a][[b]][[c]][d&&]/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r"/[a][[b]][[c]][[d]]/v.test('abcd')");
    assert_true(
        r#"
            let r = /[a][[b]][[c]][[d]]/v,
                m = r.exec('abcd');
            m !== null && m.length === 1 && m[0] === 'abcd' && m.index === 0 && m.input === 'abcd' && m
                .groups === undefined && m.indices === undefined && r.source === '[a][[b]][[c]][[d]]' && r
                .lastIndex === 0 && !r.test('xbcd') && !r.test('axcd') && !r.test('abxd') && !r.test(
                    'abcx') && !r.test('abc') && r.exec('xabcd').index === 1 && r.exec('abcdx').index ===
                0 && r.lastIndex === 0
        "#,
    );
    assert_true(r"/[[a]][b][[c]][[d]]/v.test('abcd')");
    assert_true(
        r#"
            let r = /[[a]][b][[c]][[d]]/v,
                m = r.exec('abcd');
            m !== null && m.length === 1 && m[0] === 'abcd' && m.index === 0 && m.input === 'abcd' && m
                .groups === undefined && m.indices === undefined && r.source === '[[a]][b][[c]][[d]]' && r
                .lastIndex === 0 && !r.test('xbcd') && !r.test('axcd') && !r.test('abxd') && !r.test(
                    'abcx') && !r.test('abc') && r.exec('xabcd').index === 1 && r.exec('abcdx').index ===
                0 && r.lastIndex === 0
        "#,
    );
    assert_true(r"/[[a]][[b]][[c]][d]/v.test('abcd')");
    assert_true(
        r#"
            let r = /[[a]][[b]][[c]][d]/v,
                m = r.exec('abcd');
            m !== null && m.length === 1 && m[0] === 'abcd' && m.index === 0 && m.input === 'abcd' && m
                .groups === undefined && m.indices === undefined && r.source === '[[a]][[b]][[c]][d]' && r
                .lastIndex === 0 && !r.test('xbcd') && !r.test('axcd') && !r.test('abxd') && !r.test(
                    'abcx') && !r.test('abc') && r.exec('xabcd').index === 1 && r.exec('abcdx').index ===
                0 && r.lastIndex === 0
        "#,
    );
    for source in [
        r"/a[[b]][[c]][[d]][[e]]/v.test('abcde')",
        r"/[[a]]b[[c]][[d]][[e]]/v.test('abcde')",
        r"/[[a]][[b]][[c]][[d]]e/v.test('abcde')",
        r"/[[a]][[b]][[c]]d/v.test('abcd')",
        r"/[[a]]\b[[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]]\b[[d]]/v.test('abcd')",
        r"/\b^[[a]][[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]$\b/v.test('abcd')",
        r"/\b\b[[a]][[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]\b\b/v.test('abcd')",
        r"/([[a]])[[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]]([[b]][[c]])[[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]]([[d]])/v.test('abcd')",
        r"/[[a]]+[[b]][[c]][[d]]/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]?/v.test('abcd')",
        r"/(?:\b[[a]][[b]][[c]][[d]]\b)+/v.test('abcd')",
        r"/[[a]][[b]][[c]][[d]]|[[e]][[f]][[g]][[h]]/v.test('abcd')",
        r"/[a--a][[b]][[c]][[\q{d}]]/v.test('abcd')",
        r"/[[\q{a}]][[b]][[c]][d--d]/v.test('abcd')",
        r"/[a--a][[b]][[c]][[\p{RGI_Emoji}]]/v.test('abcd')",
        r"/[[\p{RGI_Emoji}]][[b]][[c]][d--d]/v.test('abcd')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r"var kept=/(?<whole>(\B[[a]][[a]][[a]][[b]]\b))/dvg;kept.lastIndex=1")
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(
        realm.eval(r"let m=kept.exec('aaaab!');m.index===1&&m.groups.whole==='aaab'&&m.indices[2][1]===5&&kept.lastIndex===5"),
        Ok(Value::Boolean(true))
    );
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval(r"var r=/\B[[a]][[a]][[a]][[c]]\b/vg,sticky=/[[a]][[a]][[a]][[a]]/vy,marker=0")
        .unwrap();
    // The first three checked classes succeed; the fourth fails before word gates.
    let setup = format!("var text='{}';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert_eq!(
        realm.eval(r"sticky.test(text)&&sticky.lastIndex===4"),
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
fn checked_v_set_class_sequence_results() {
    let mut rows = String::new();
    let mut row_count = 0;
    // Full objects and original spans below are independent manual expectations.
    for (pattern, input, flags, start) in [
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[b]]\b))",
            JsString::from("aaaab!"),
            "dvg",
            0,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[b]]\b))",
            JsString::from("aaaab!"),
            "dvy",
            0,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[b]]\b))",
            JsString::from("aaaab!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\b[[\p{Any}]][[a]][[b]][[c]][[\p{Any}]]\b))",
            JsString::from("Kabcſ!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>(\b[\P{Ll}&&A][[s]][[x]][[a]]\b))",
            JsString::from("Asxa!"),
            "divg",
            0,
        ),
        (
            r"(?<whole>([[\p{Any}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]))",
            JsString::from("😀😁😂😃😄😅!"),
            "dvy",
            1,
        ),
        (
            r"(?<whole>(\b[[\uD800]][[\uD800]][[\uD800]][[\uD800]][[\uD800]]\B))",
            JsString::from_code_units(vec![65, 0xd800, 0xd800, 0xd800, 0xd800, 0xd800, 33]),
            "dvg",
            0,
        ),
        (
            r"^(?<whole>(\b[[a]][[b]][[c]][[d]]\b))$",
            JsString::from("x\r\nabcd\u{2028}ef"),
            "dvmg",
            0,
        ),
        (
            r"(?<whole>(\B[[a]][[a]][[a]][[ab]]\b))",
            JsString::from("aaaaa!"),
            "dvg",
            0,
        ),
        (
            r"^(?<whole>([[^K]][[s]][[a]][[b]]))$",
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
