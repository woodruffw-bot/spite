//! Match results and consumers for pairs of character classes and literal text.

use crate::common::assert_true;
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;

#[test]
fn literal_checked_flat_v_set_pair_state_context_and_consumers() {
    assert_true(
        r#"
            let r = /(?<whole>(c[[a]][b]))/dvg,
                m = r.exec('ccab');
            m !== null && Array.isArray(m) && m.length === 3 && m[0] === 'cab' && m[1] === 'cab' && m[
                2] === 'cab' && m.index === 1 && m.input === 'ccab' && m.groups.whole === 'cab' && m
                .indices.groups.whole === m.indices[1] && m.indices[2][0] === 1 && m.indices[2][1] === 4 &&
                r.source === '(?<whole>(c[[a]][b]))' && r.lastIndex === 4 && r.exec('ccab') === null && r
                .lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /a[[a]][b]/dvg,
                m = r.exec('aaab!');
            m !== null && m.index === 1 && m[0] === 'aab' && m.indices[0][1] === 4 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let r = /a[[a]][a]\b/dvg,
                m = r.exec('aaaa!');
            m !== null && m.index === 1 && m[0] === 'aaa' && m.indices[0][1] === 4 && r.lastIndex === 4
        "#,
    );
    assert_true(
        r#"
            let r = /a[[a]][a]\b/dvy;
            r.exec('aaaa!') === null && r.lastIndex === 0 && (r.lastIndex = 1, r.exec('aaaa!')[0] ===
                'aaa') && r.lastIndex === 4 && r.exec('aaaa!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            /\bc[[a]][b]/v.test('cab') && /\Bc[[a]][b]/v.test('xcab') && /c[[a]][b]\b/v.test('cab!') &&
                /c[[a]][b]\B/v.test('cabx') && /\bc[[a]][b]\b/v.test('!cab!') && /\bc[[a]][b]\B/v.test(
                    '!cabx') && /\Bc[[a]][b]\b/v.test('xcab!') && /\Bc[[a]][b]\B/v.test('xcabx') && !
                /\bc[[a]][b]/v.test('xcab') && !/c[[a]][b]\b/v.test('cabx') && !/\Bc[[a]][b]/v.test(
                '!cab') && !/c[[a]][b]\B/v.test('cab!')
        "#,
    );
    assert_true(
        r#"
            /\bK[[s]][a]\b/iv.test('KſA!') && !/\bK[[s]][a]\b/v.test('Ksa!') && !/\bK[[s]][a]\b/v.test(
                    'Kſa!') && !/\bK[[s]][a]\b/v.test('KsA!') && /\bK[[s]][a]\b/v.test('Ksa!') && !
                /\bK[[s]][a]\b/iv.test('asa!') && !/\bK[[s]][a]\b/iv.test('Kxa!') && !/\bK[[s]][a]\b/iv
                .test('Ksb!')
        "#,
    );
    assert_true(
        r#"
            let a = /^\B\$[[\p{Any}]][\p{Any}]\b$/iv,
                b = /^\B\$[[\p{Any}]][\p{Any}]\b$/v;
            a.test('$!K') && a.test('$!ſ') && !b.test('$!K') && !b.test('$!ſ') && a.exec('$!K')[0] ===
                '$!K'
        "#,
    );
    assert_true(
        r#"
            /c[a--a][\p{Any}]/v.exec('cab') === null && /c[[a]][]/v.exec('cab') === null && /c[a--a][]/v
                .exec('cab') === null && /c[[^K]][s]/iv.exec('c!s')[0] === 'c!s' && !/c[[^K]][s]/iv.test(
                    'cKs') && !/c[[^K]][s]/iv.test('cKs') && /c[[a]][^b]/v.exec('cax')[0] === 'cax' && !
                /c[[a]][^b]/v.test('cab')
        "#,
    );
    assert_true(
        r#"
            /c[[\P{Ll}]][s]/v.test('cAs') && !/c[[\P{Ll}]][s]/iv.test('cAs') && !/c[[\P{Ll}]][s]/iv.test(
                'cas') && /c[[\P{Ll}]][s]/iv.test('c!s')
        "#,
    );
    assert_true(
        r#"
            let r = /\b\u{1f600}[[\u{1f601}]][\u{1f600}]\B/dvy;
            r.lastIndex = 2;
            let m = r.exec('A😀😁😀!');
            m !== null && m.index === 1 && m[0] === '😀😁😀' && m.indices[0][0] === 1 && m.indices[0][
                1] === 7 && r.lastIndex === 7 && r.exec('A😀😁😀!') === null && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\b\uD800[[\uD800]][\uD800]\B/dv,
                m = r.exec('A\uD800\uD800\uD800!');
            m !== null && m.index === 1 && m[0] === '\uD800\uD800\uD800' && m.indices[0][1] === 4 &&
                /\uD800[[\uD800]][\uDC00]/v.exec('\uD800\uD800\uDC00') === null
        "#,
    );
    assert_true(r#"/^\B\[[[\/]][\$]\B$/v.test('[/$')&&/^\bc[[\x61]][\u{62}]\b$/v.test('cab')"#);
    assert_true(
        r#"
            let r = /^(?<whole>(\bc[[a]][b]\b))$/dvmg,
                m = r.exec('x\r\ncab\u2028cax');
            m !== null && m.index === 3 && m[0] === 'cab' && m.indices[2][0] === 3 && m.indices[2][1] ===
                6 && r.lastIndex === 6 && r.exec('x\r\ncab\u2028cax') === null && r.lastIndex === 0 && !
                /^\bc[[a]][b]\b$/v.test('cab\n')
        "#,
    );
    assert_true(
        r#"
            /\bc[[a]][b]/v.exec('cabx')[0] === 'cab' && /\bc[[a]][b]/v.exec('cabx').index === 0 &&
                /\ba[[a]][c]/v.exec('aacx')[0] === 'aac' && /c[[a]][[b]]/v.test('cab') &&
                /\bc[[a]][[b]]\b/v.test('cab') && /c[a][b]/v.test('cab') && /\bc[a][b]\b/v.test('cab') &&
                /[a]b[c]/v.test('abc') && /\ba[b]\b/v.test('ab')
        "#,
    );
    assert_true(
        r#"
            let r = /(?<whole>(c[[a]][b]\b))/dvg,
                a = [...'😀cab ccab'.matchAll(r)];
            a.length === 2 && a[0][0] === 'cab' && a[0][1] === 'cab' && a[0][2] === 'cab' && a[0].index ===
                2 && a[0].groups.whole === 'cab' && a[0].indices.groups.whole === a[0].indices[1] && a[0]
                .indices[2][1] === 5 && a[1].index === 7 && a[1].indices[2][1] === 10 && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            '😀cab ccab'.replace(/(?<whole>(c[[a]][b]\b))/vg, '<$1:$<whole>>') ===
                '😀<cab:cab> c<cab:cab>' && '😀cabx'.split(/(c[[a]][b])/v).join('|') === '😀|cab|x' &&
                'ccab'.match(/c[[a]][b]/v)[0] === 'cab'
        "#,
    );
    assert_true(
        r#"
            let seen = '',
                out = '😀cab ccab'.replace(/(?<whole>(c[[a]][b]\b))/vg, (m, c1, c2, p, text, g) => {
                    seen += p + ':' + g.whole + ';';
                    return '!'
                });
            out === '😀! c!' && seen === '2:cab;7:cab;'
        "#,
    );
    assert_true(r#"let r=/c[[a]][b]/vg;r.lastIndex=1;'😀ccab'.search(r)===3&&r.lastIndex===1"#);
    assert_true(
        r#"
            let trace = '',
                r = /\u{1f600}[[\u{1f601}]][a]/vy;
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
                r = /c[[a]][b]/v;
            r.lastIndex = {
                valueOf() {
                    seen++;
                    return 3
                }
            };
            let m = r.exec('cabcab');
            m.index === 0 && m[0] === 'cab' && seen === 1 && typeof r.lastIndex === 'object'
        "#,
    );
    assert_true(
        r#"
            let r = /K[[s]][a]/iv,
                c = new RegExp(r, 'dvy');
            c.source === 'K[[s]][a]' && c.unicodeSets && !c.ignoreCase && c.exec('KſA') === null && c
                .lastIndex === 0 && r.exec('KſA')[0] === 'KſA'
        "#,
    );
    assert_true(
        r#"
            let a = /c[[a]][b]/vy,
                b = /c[[a]][b]/vy;
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
                a.exec('ccab')
            } catch (e) {
                x = e instanceof TypeError
            }
            try {
                b.exec('xcab')
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
                eval('marker=1; /c[a&&][b]/v;')
            } catch (e) {
                caught = e instanceof SyntaxError
            }
            caught && marker === 0
        "#,
    );
    assert_true(r#"/c[a][[b]]/v.test('cab')"#);
    assert_true(
        r#"
            let r = /c[a][[b]]/v,
                m = r.exec('cab');
            m !== null && Array.isArray(m) && m.length === 1 && m[0] === 'cab' && m.index === 0 && m
                .input === 'cab' && m.groups === undefined && m.indices === undefined && r.source ===
                'c[a][[b]]' && r.lastIndex === 0 && !r.test('xab') && !r.test('cxb') && !r.test('cax') && !
                r.test('ca') && r.test('cabx') && r.exec('xcab').index === 1
        "#,
    );
    assert_true(r#"/\bc[a][[b]]/v.test('cab')"#);
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
        r#"/c[[\q{a}]][b]/v.test('cab')"#,
        r#"/c[a--a][\q{b}]/v.test('cab')"#,
        r#"/c[a--a][\p{RGI_Emoji}]/v.test('cab')"#,
        r#"/c[[\p{RGI_Emoji}]][b]/v.test('cab')"#,
        r#"/cc[[a]][b]/v.test('ccab')"#,
        r#"/c[[a]][b]d/v.test('cabd')"#,
        r#"/c[[a]][b][c]/v.test('cabc')"#,
        r#"/c\b[[a]][b]/v.test('cab')"#,
        r#"/c[[a]]\b[b]/v.test('cab')"#,
        r#"/\b^c[[a]][b]/v.test('cab')"#,
        r#"/c[[a]][b]$\b/v.test('cab')"#,
        r#"/(c)[[a]][b]/v.test('cab')"#,
        r#"/c([[a]])[b]/v.test('cab')"#,
        r#"/c[[a]]([b])/v.test('cab')"#,
        r#"/c+[[a]][b]/v.test('cab')"#,
        r#"/c[[a]]+[b]/v.test('cab')"#,
        r#"/c[[a]][b]?/v.test('cab')"#,
        r#"/(?:c[[a]][b])+/v.test('cab')"#,
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
    let mut realm = Realm::default();
    realm
        .eval(r#"var kept=/(?<whole>(a[[a]][a]\b))/dvg;kept.lastIndex=1"#)
        .unwrap();
    realm.collect(usize::MAX).unwrap();
    assert_eq!(realm.eval(r#"kept.source==='(?<whole>(a[[a]][a]\\b))'&&kept.lastIndex===1&&kept.exec('aaaa!').groups.whole==='aaa'&&kept.lastIndex===4"#), Ok(Value::Boolean(true)));
    // Explicit opted-in host cancellation only; default JavaScript remains unlimited.
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm.eval(r#"var r=/a[[a]][c]/vg;var marker=0"#).unwrap();
    let setup = format!("var text='{}b';r.lastIndex=1", "a".repeat(20_000));
    realm.eval(&setup).unwrap();
    assert!(matches!(
        realm.eval(r#"try{r.exec(text)}catch{marker=1}finally{marker=2}"#),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval(r#"marker===0&&r.lastIndex===1"#),
        Ok(Value::Boolean(true))
    );
}

#[test]
fn literal_checked_flat_v_set_pair_results() {
    let mut rows = String::new();
    let mut row_count = 0;

    for (pattern, input, flags, start) in [
        (
            r#"(?<whole>(c[[a]][b]))"#,
            JsString::from_code_units(vec![99, 99, 97, 98]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>(a[[a]][b]))"#,
            JsString::from_code_units(vec![97, 97, 97, 98, 33]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>(a[[a]][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>(a[[a]][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            0,
        ),
        (
            r#"(?<whole>(a[[a]][a]\b))"#,
            JsString::from_code_units(vec![97, 97, 97, 97, 33]),
            "dvy",
            1,
        ),
        (
            r#"(?<whole>(\bc[[a]][b]))"#,
            JsString::from_code_units(vec![99, 97, 98, 120]),
            "dvg",
            0,
        ),
        (
            r#"(?<whole>(\b\u{1f600}[[\u{1f601}]][\u{1f600}]\B))"#,
            JsString::from_code_units(vec![65, 55357, 56832, 55357, 56833, 55357, 56832, 33]),
            "dvy",
            2,
        ),
        (
            r#"(?<whole>(\b\uD800[[\uD800]][\uD800]\B))"#,
            JsString::from_code_units(vec![65, 55296, 55296, 55296, 33]),
            "dvg",
            0,
        ),
        (
            r#"^(?<whole>(\B\$[[\p{Any}]][\p{Any}]\b))$"#,
            JsString::from_code_units(vec![36, 33, 8490]),
            "divg",
            0,
        ),
        (
            r#"(?<whole>(c[a--a][\p{Any}]))"#,
            JsString::from_code_units(vec![99, 97, 98, 33]),
            "dvg",
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
