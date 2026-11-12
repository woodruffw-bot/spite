//! Changes_When_NFKC_Casefolded and its exact CWKCF alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_changes_nfkc_casefolded_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Changes_When_NFKC_Casefolded}",
        r"\P{CWKCF}",
        r"[^\p{CWKCF}]",
        r"[\p{CWKCF}K]",
        r"[\P{Changes_When_NFKC_Casefolded}K]",
        r"^(?<x>[\p{CWKCF}K])$",
        r"(?<x>\P{Changes_When_NFKC_Casefolded})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{41}\u{2028}\u{42}\u{2029}",
                "\u{ff10}\u{ff26}",
                "",
            ] {
                let input = JsString::from(input);
                for start in last_index_cases(input.len()) {
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
                            }})
                        "#
                    );
                    let Value::String(result) = Realm::default().eval(&script).unwrap() else {
                        panic!("{script}")
                    };
                    writeln!(
                        rows,
                        "{pattern:?} flags={flags:?} input={input:?} lastIndex={start} => {result}",
                        result = result.to_utf8().expect("JSON output must be valid UTF-8"),
                    )
                    .unwrap();
                }
            }
        }
    }
    insta::assert_snapshot!(rows);
}

#[test]
fn unicode_changes_nfkc_casefolded_properties_aliases_complements_and_unions() {
    let case_insensitive_members =
        JsString::from(unicode_samples::from_ranges(CASE_INSENSITIVE_MEMBERS).as_str());
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let p = /^\p{{Changes_When_NFKC_Casefolded}}$/i{mode},
                    n = /^\P{{Changes_When_NFKC_Casefolded}}$/i{mode};
                [...
                    {case_insensitive_members:?}]
                .every(c => p.test(c) && n.test(c) === ('{mode}' === 'u'))
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /[\p{{Changes_When_NFKC_Casefolded}}K]/i{mode},
                    n = /[\P{{Changes_When_NFKC_Casefolded}}K]/i{mode},
                    x = /[^\P{{Changes_When_NFKC_Casefolded}}K]/i{mode};
                p.test('K') && n.test('K') && !x.test('K') && [...
                    {case_insensitive_members:?}
                ].every(c => p.test(c) && n.test(c) === ('{mode}' === 'u' || c === 'K' || c ===
                    'k' || c === 'K') && x.test(c) === ('{mode}' === 'v' && c !== 'K' && c !==
                    'k' && c !== 'K'))
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Changes_When_NFKC_Casefolded}}/{mode},
                    b = /\p{{CWKCF}}/i{mode},
                    n = /\P{{CWKCF}}/i{mode};
                a.test('\u0041') && a.test('\u0042') && b.test('\u0041') && !a.test('!') && !b.test(
                    '\u005f') && b.test('ſ') && b.test('K') && n.test('\u0041') === (
                    '{mode}' ===
                    'u') && n.test('😀') && n.test('\uD800') && !n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Changes_When_NFKC_Casefolded}}/{mode},
                    s = /\s/{mode};
                p.test('\u0041') && p.test('\u0042') && !s.test('\u0041') && !s.test('\u0042') && p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{CWKCF}}K]/i{mode},
                    b = /[^\P{{Changes_When_NFKC_Casefolded}}K]/i{mode},
                    c = /[\p{{CWKCF}}\P{{Changes_When_NFKC_Casefolded}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0041') && a.test('ſ') && b.test('\u0042') === (
                    '{mode}' === 'v') && !b.test('K') && c.test('😀') && c.test('A') && c.test(
                    '\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{CWKCF}}$/{mode},
                    b = /^\P{{CWKCF}}$/{mode};
                a.test('\u0041') && !a.test('\u0042\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_changes_nfkc_casefolded_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{Changes_When_NFKC_Casefolded}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uD801\uDC00');
                m.index === 0 && m[0] === '\uD801\uDC00' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{CWKCF}}K]))$/dim{mode}g,
                    m = r.exec('\u0021\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{CWKCF}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\u0041');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{CWKCF}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\u0041');
                m.index === 2 && m[0] === '\u0041' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{CWKCF}}/{mode}y;r.lastIndex=1;r.exec('😀\u0041')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_changes_nfkc_casefolded_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u0042😀\u0041\u0021'.matchAll(/(?<x>\P{{CWKCF}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === '\u0021'
            "#
        ));
        assert_true(&format!(
            r"'\u0042\u0041😀\u0021'.replace(/(?<x>\p{{Changes_When_NFKC_Casefolded}})/{mode}g,'<$1:$<x>>')==='<\u0042:\u0042><\u0041:\u0041>😀\u0021'"
        ));
        assert_true(&format!(
            r#"
                '😀\u0042\u0041\u0021'.split(/(\p{{CWKCF}})/{mode}).join('|') ===
                    '😀|\u0042||\u0041|\u0021' && '😀\u0041'.search(/\p{{CWKCF}}/{mode}) === 2 &&
                    '\u0042\u0041😀'.match(/\p{{CWKCF}}/{mode}g).join('') === '\u0042\u0041'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0041\u0042😀\u0041'.replace(/(\p{{CWKCF}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_changes_nfkc_casefolded_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Changes_When_NFKC_Casefolded})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Changes_When_NFKC_Casefolded})$' && c.unicodeSets && !c.unicode && !c
                .multiline && c.exec('\u0041').groups.x === '\u0041' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{CWKCF}/uy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return '😀'
                }
            });
            t === 'si' && m[0] === '😀' && m.index === 0 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let r = /\p{CWKCF}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u0041')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{CWKCF}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀\u0041')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_changes_nfkc_casefolded_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{CWKCF}/ug,marker=0;r.lastIndex=1")
        .unwrap();
    realm.eval("let text='😀'.repeat(10000)").unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(
        r#"
            let m = /\p{Script_Extensions=Latin}/u.exec('A');
            JSON.stringify(m) === '["A"]' && m.index === 0 && m.input === 'A'
        "#,
    );
    assert_true(
        r"let m=/[\p{CWKCF}&&A]/v.exec('A');m!==null&&m.length===1&&m[0]==='A'&&m.index===0&&m.input==='A'",
    );
    assert_true(r"/[\p{CWKCF}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x42),
                m = /[\p{CWKCF}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{CWKCF}+b/u.exec('AA')", r"/\p{CWKCF}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_changes_nfkc_casefolded_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\b(?<x>\p{{CWKCF}})+\b/d{mode}g,
                    m = r.exec('😀\u0041\u0042\u0042😀');
                m.index === 2 && m[0] === '\u0041\u0042\u0042' && m.groups.x === '\u0042' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_changes_nfkc_casefolded_property_unicode18_addition() {
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u0277') === false && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u0277') === true && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u0277') === false && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u0277') === false && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u0277') === true && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u0277') === false && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u027C') === false && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u027C') === true && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u027C') === false && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u027C') === false && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u027C') === true && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u027C') === false && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u0558') === true && n.test('\u0558') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u058B') === true && n.test('\u058B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u058C') === true && n.test('\u058C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u209D') === true && n.test('\u209D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u209E') === true && n.test('\u209E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\u209F') === true && n.test('\u209F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uA7DD') === true && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uA7DD') === true && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uA7E2') === true && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uA7E2') === true && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uAB4B') === false && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uAB4B') === true && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uAB4B') === false && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uAB4B') === false && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uAB4B') === true && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uAB4B') === false && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uAB4C') === false && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uAB4C') === true && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uAB4C') === false && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uAB4C') === false && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uAB4C') === true && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uAB4C') === false && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uAB6C') === true && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uAB6C') === true && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uAB6D') === true && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uAB6D') === true && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD801\uDFBB') === true && n.test('\uD801\uDFBB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD801\uDFBC') === true && n.test('\uD801\uDFBC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD801\uDFBD') === true && n.test('\uD801\uDFBD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD801\uDFBE') === true && n.test('\uD801\uDFBE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD801\uDFBF') === true && n.test('\uD801\uDFBF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD82C\uDD23') === true && n.test('\uD82C\uDD23') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD82C\uDD24') === true && n.test('\uD82C\uDD24') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD82C\uDD25') === true && n.test('\uD82C\uDD25') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD82C\uDD26') === true && n.test('\uD82C\uDD26') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD835\uDEA6') === true && n.test('\uD835\uDEA6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF41') === false && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF41') === false && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF41') === false && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF41') === false && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF49') === false && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF49') === false && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF49') === false && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF49') === false && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF4B') === false && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF4B') === false && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF4B') === false && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF4B') === false && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF4E') === false && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF4E') === false && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF4E') === false && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF4E') === false && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF52') === false && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF52') === false && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF52') === false && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF52') === false && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF69') === false && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF69') === false && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF69') === false && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF69') === false && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF6B') === false && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF6B') === false && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF6B') === false && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF6B') === false && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF6D') === false && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF6D') === false && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF6D') === false && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF6D') === false && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF6F') === false && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF6F') === false && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF6F') === false && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF6F') === false && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF73') === false && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF73') === false && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF73') === false && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF73') === false && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF75') === false && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF75') === false && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF75') === false && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF75') === false && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF77') === false && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF77') === false && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF77') === false && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF77') === false && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF79') === false && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF79') === false && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF79') === false && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF79') === false && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF7B') === false && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF7B') === false && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF7B') === false && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF7B') === false && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF7D') === false && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF7D') === false && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF7D') === false && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF7D') === false && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF7F') === false && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF7F') === false && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF7F') === false && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF7F') === false && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFCD') === true && n.test('\uD837\uDFCD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFCE') === true && n.test('\uD837\uDFCE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFCF') === true && n.test('\uD837\uDFCF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD0') === true && n.test('\uD837\uDFD0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD1') === true && n.test('\uD837\uDFD1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD2') === true && n.test('\uD837\uDFD2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD3') === true && n.test('\uD837\uDFD3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD4') === true && n.test('\uD837\uDFD4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD5') === true && n.test('\uD837\uDFD5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD6') === true && n.test('\uD837\uDFD6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD7') === true && n.test('\uD837\uDFD7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD8') === true && n.test('\uD837\uDFD8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFD9') === true && n.test('\uD837\uDFD9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFDA') === true && n.test('\uD837\uDFDA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFDB') === true && n.test('\uD837\uDFDB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFDC') === true && n.test('\uD837\uDFDC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFDD') === true && n.test('\uD837\uDFDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFDE') === true && n.test('\uD837\uDFDE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFDF') === true && n.test('\uD837\uDFDF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE0') === true && n.test('\uD837\uDFE0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE1') === true && n.test('\uD837\uDFE1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE2') === true && n.test('\uD837\uDFE2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE3') === true && n.test('\uD837\uDFE3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE4') === true && n.test('\uD837\uDFE4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE5') === true && n.test('\uD837\uDFE5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE6') === true && n.test('\uD837\uDFE6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE7') === true && n.test('\uD837\uDFE7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE8') === true && n.test('\uD837\uDFE8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFE9') === true && n.test('\uD837\uDFE9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFEA') === true && n.test('\uD837\uDFEA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFEB') === true && n.test('\uD837\uDFEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFEC') === true && n.test('\uD837\uDFEC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFED') === true && n.test('\uD837\uDFED') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFEE') === true && n.test('\uD837\uDFEE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFEF') === true && n.test('\uD837\uDFEF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF0') === true && n.test('\uD837\uDFF0') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF1') === true && n.test('\uD837\uDFF1') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF2') === true && n.test('\uD837\uDFF2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF3') === true && n.test('\uD837\uDFF3') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF4') === true && n.test('\uD837\uDFF4') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF5') === true && n.test('\uD837\uDFF5') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF6') === true && n.test('\uD837\uDFF6') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF7') === true && n.test('\uD837\uDFF7') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF8') === true && n.test('\uD837\uDFF8') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFF9') === true && n.test('\uD837\uDFF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFFA') === true && n.test('\uD837\uDFFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFFB') === true && n.test('\uD837\uDFFB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFFC') === true && n.test('\uD837\uDFFC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFFD') === true && n.test('\uD837\uDFFD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFFE') === true && n.test('\uD837\uDFFE') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'u'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'u');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'ui');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'v'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'v');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_NFKC_Casefolded}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_NFKC_Casefolded}$', 'vi');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'u'),
                n = new RegExp('^\\P{CWKCF}$', 'u');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'ui'),
                n = new RegExp('^\\P{CWKCF}$', 'ui');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'v'),
                n = new RegExp('^\\P{CWKCF}$', 'v');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWKCF}$', 'vi'),
                n = new RegExp('^\\P{CWKCF}$', 'vi');
            p.test('\uD837\uDFFF') === true && n.test('\uD837\uDFFF') === false
        "#,
    );
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0x1F600..=0x1F600,
    0x0040..=0x005B,
    0x0061..=0x007A,
    0x009F..=0x00A1,
    0x00A7..=0x00B2,
    0x00B5..=0x00B8,
    0x00BA..=0x00BC,
    0x00BE..=0x00F6,
    0x00F8..=0x0132,
    0x0134..=0x013F,
    0x0141..=0x01B9,
    0x01BB..=0x01BD,
    0x01BF..=0x01BF,
    0x01C3..=0x01C4,
    0x01CD..=0x01F1,
    0x01F4..=0x0233,
    0x0239..=0x0254,
    0x0256..=0x0257,
    0x0259..=0x0259,
    0x025B..=0x025C,
    0x0260..=0x0261,
    0x0263..=0x0266,
    0x0268..=0x026C,
    0x026F..=0x026F,
    0x0271..=0x0272,
    0x0275..=0x0275,
    0x027D..=0x027D,
    0x0280..=0x0280,
    0x0282..=0x0283,
    0x0287..=0x028C,
    0x0292..=0x0292,
    0x029D..=0x029E,
    0x02AF..=0x02B0,
    0x02B8..=0x02B9,
    0x02D7..=0x02D8,
    0x02DD..=0x02E0,
    0x02E4..=0x02E5,
    0x033F..=0x0343,
    0x0345..=0x0346,
    0x034E..=0x0350,
    0x036F..=0x0377,
    0x0379..=0x0380,
    0x0383..=0x0384,
    0x0386..=0x0386,
    0x0388..=0x03D1,
    0x03D5..=0x0481,
    0x0489..=0x0557,
    0x0559..=0x0559,
    0x0561..=0x0588,
    0x058A..=0x058A,
    0x058D..=0x058D,
    0x061B..=0x061D,
    0x0674..=0x0675,
    0x0678..=0x0679,
    0x0957..=0x0958,
    0x095F..=0x0960,
    0x09DB..=0x09E0,
    0x0A32..=0x0A37,
    0x0A58..=0x0A59,
    0x0A5B..=0x0A5F,
    0x0B5B..=0x0B5E,
    0x0E32..=0x0E34,
    0x0EB2..=0x0EB4,
    0x0EDB..=0x0EDE,
    0x0F0B..=0x0F0D,
    0x0F42..=0x0F44,
    0x0F4C..=0x0F4E,
    0x0F51..=0x0F53,
    0x0F56..=0x0F58,
    0x0F5B..=0x0F5D,
    0x0F68..=0x0F6A,
    0x0F72..=0x0F75,
    0x0F79..=0x0F7A,
    0x0F80..=0x0F82,
    0x0F92..=0x0F94,
    0x0F9C..=0x0F9E,
    0x0FA1..=0x0FA3,
    0x0FA6..=0x0FA8,
    0x0FAB..=0x0FAD,
    0x0FB8..=0x0FBA,
    0x109F..=0x10C8,
    0x10CC..=0x10CE,
    0x10D0..=0x10FF,
    0x115E..=0x1161,
    0x13A0..=0x13F5,
    0x13F7..=0x13FE,
    0x17B3..=0x17B6,
    0x180A..=0x180B,
    0x180F..=0x1810,
    0x1C7F..=0x1C8A,
    0x1C8F..=0x1CC0,
    0x1D2B..=0x1D2C,
    0x1D2E..=0x1D30,
    0x1D3A..=0x1D3C,
    0x1D4D..=0x1D4F,
    0x1D6A..=0x1D6B,
    0x1D77..=0x1D79,
    0x1D7D..=0x1D7D,
    0x1D8E..=0x1D8E,
    0x1D9A..=0x1D9B,
    0x1DBF..=0x1DC0,
    0x1DFF..=0x1E95,
    0x1E99..=0x1F15,
    0x1F17..=0x1F1E,
    0x1F20..=0x1F45,
    0x1F47..=0x1F4E,
    0x1F51..=0x1F51,
    0x1F53..=0x1F53,
    0x1F55..=0x1F55,
    0x1F57..=0x1F80,
    0x1FAF..=0x1FB2,
    0x1FB4..=0x1FBA,
    0x1FBE..=0x1FBE,
    0x1FC4..=0x1FC8,
    0x1FCA..=0x1FCA,
    0x1FCF..=0x1FD4,
    0x1FD7..=0x1FDD,
    0x1FDF..=0x1FE5,
    0x1FE7..=0x1FEA,
    0x1FEC..=0x1FEC,
    0x1FEF..=0x1FF2,
    0x1FF4..=0x1FF8,
    0x1FFA..=0x1FFA,
    0x1FFE..=0x2000,
    0x200F..=0x2012,
    0x2016..=0x2018,
    0x2023..=0x2024,
    0x2026..=0x2027,
    0x2029..=0x202A,
    0x202F..=0x2030,
    0x2032..=0x2038,
    0x203B..=0x203F,
    0x2046..=0x2047,
    0x2049..=0x204A,
    0x2056..=0x2058,
    0x205E..=0x205F,
    0x2071..=0x2074,
    0x208E..=0x2090,
    0x20A0..=0x20A0,
    0x20A7..=0x20A9,
    0x20FF..=0x2100,
    0x2103..=0x2105,
    0x2107..=0x2109,
    0x2113..=0x2119,
    0x211D..=0x2120,
    0x2122..=0x212B,
    0x212D..=0x212F,
    0x2132..=0x2132,
    0x2139..=0x213B,
    0x2140..=0x2141,
    0x2144..=0x2145,
    0x2149..=0x214A,
    0x214E..=0x2150,
    0x217F..=0x2180,
    0x2182..=0x2184,
    0x2188..=0x218A,
    0x222B..=0x2231,
    0x2328..=0x232B,
    0x245F..=0x2460,
    0x24EA..=0x24EB,
    0x2A0B..=0x2A0D,
    0x2A73..=0x2A74,
    0x2A76..=0x2A77,
    0x2ADB..=0x2ADD,
    0x2BFF..=0x2C76,
    0x2C7B..=0x2C7C,
    0x2C7E..=0x2CE3,
    0x2CEA..=0x2CEE,
    0x2CF1..=0x2CF3,
    0x2D00..=0x2D25,
    0x2D27..=0x2D27,
    0x2D2D..=0x2D2D,
    0x2D6E..=0x2D70,
    0x2E9E..=0x2EA0,
    0x2EF2..=0x2EF4,
    0x2EFF..=0x2F00,
    0x2FD5..=0x2FD6,
    0x2FFF..=0x3001,
    0x3035..=0x3038,
    0x303A..=0x303B,
    0x309A..=0x30A0,
    0x30FE..=0x3100,
    0x3130..=0x3131,
    0x318E..=0x318F,
    0x3191..=0x3192,
    0x319F..=0x31A0,
    0x31FF..=0x3200,
    0x321E..=0x3220,
    0x3247..=0x3248,
    0x324F..=0x3250,
    0x327E..=0x3280,
    0x33FF..=0x3400,
    0xA63F..=0xA66D,
    0xA67F..=0xA69E,
    0xA721..=0xA72F,
    0xA731..=0xA771,
    0xA778..=0xA787,
    0xA78A..=0xA7DC,
    0xA7DE..=0xA7DE,
    0xA7E1..=0xA7E1,
    0xA7E3..=0xA7E3,
    0xA7F0..=0xA7F1,
    0xA7F5..=0xA7FA,
    0xAB53..=0xAB53,
    0xAB5B..=0xAB5C,
    0xAB5F..=0xAB60,
    0xAB68..=0xAB6B,
    0xAB6E..=0xABC0,
    0xF8FF..=0xF900,
    0xFA0D..=0xFA15,
    0xFA1E..=0xFA27,
    0xFA29..=0xFA2A,
    0xFA6D..=0xFA70,
    0xFAD9..=0xFADA,
    0xFAFF..=0xFB00,
    0xFB06..=0xFB07,
    0xFB12..=0xFB13,
    0xFB17..=0xFB18,
    0xFB1C..=0xFB1F,
    0xFB36..=0xFB38,
    0xFB3C..=0xFB46,
    0xFBB1..=0xFBB2,
    0xFBD2..=0xFBD3,
    0xFD3D..=0xFD3E,
    0xFD4F..=0xFD50,
    0xFD8F..=0xFD92,
    0xFDC7..=0xFDC8,
    0xFDEF..=0xFDF0,
    0xFDFC..=0xFDFD,
    0xFDFF..=0xFE00,
    0xFE19..=0xFE1A,
    0xFE2F..=0xFE30,
    0xFE44..=0xFE47,
    0xFE52..=0xFE54,
    0xFE66..=0xFE68,
    0xFE6B..=0xFE6C,
    0xFE6F..=0xFE70,
    0xFE72..=0xFE76,
    0xFEFC..=0xFF01,
    0xFFBE..=0xFFBF,
    0xFFC1..=0xFFC2,
    0xFFC7..=0xFFCA,
    0xFFCF..=0xFFD2,
    0xFFD7..=0xFFDA,
    0xFFDC..=0xFFDD,
    0xFFDF..=0xFFE0,
    0xFFE6..=0xFFE8,
    0xFFEE..=0xFFF0,
    0xFFF8..=0xFFF9,
    0x103FF..=0x1044F,
    0x104AF..=0x104D4,
    0x104D8..=0x104FB,
    0x1056F..=0x105A1,
    0x105A3..=0x105B1,
    0x105B3..=0x105B9,
    0x105BB..=0x105BC,
    0x10780..=0x10781,
    0x10785..=0x10787,
    0x107B0..=0x107B2,
    0x107C0..=0x107C0,
    0x10C7F..=0x10CB3,
    0x10CC0..=0x10CF2,
    0x10D4F..=0x10D66,
    0x10D70..=0x10D85,
    0x1189F..=0x118DF,
    0x16E3F..=0x16E7F,
    0x16E9F..=0x16EB9,
    0x16EBB..=0x16ED3,
    0x1B122..=0x1B122,
    0x1B127..=0x1B127,
    0x1BC9F..=0x1BCA0,
    0x1BCA3..=0x1BCA4,
    0x1CCD5..=0x1CCD6,
    0x1CCF9..=0x1CCFA,
    0x1D15D..=0x1D15E,
    0x1D164..=0x1D165,
    0x1D172..=0x1D173,
    0x1D17A..=0x1D17B,
    0x1D1BA..=0x1D1BB,
    0x1D1C0..=0x1D1C1,
    0x1D3FF..=0x1D400,
    0x1D454..=0x1D456,
    0x1D49C..=0x1D4A9,
    0x1D4AC..=0x1D4AE,
    0x1D4B9..=0x1D4BD,
    0x1D4C3..=0x1D4C5,
    0x1D505..=0x1D507,
    0x1D50A..=0x1D50D,
    0x1D514..=0x1D516,
    0x1D51C..=0x1D51E,
    0x1D539..=0x1D53B,
    0x1D53E..=0x1D540,
    0x1D544..=0x1D547,
    0x1D549..=0x1D54A,
    0x1D550..=0x1D552,
    0x1D6A7..=0x1D6A8,
    0x1D7CB..=0x1D7CE,
    0x1D7FF..=0x1D800,
    0x1DF3F..=0x1DF3F,
    0x1DF47..=0x1DF47,
    0x1DF4C..=0x1DF4C,
    0x1DF50..=0x1DF50,
    0x1DF67..=0x1DF67,
    0x1DF71..=0x1DF71,
    0x1DF94..=0x1DF94,
    0x1DF96..=0x1DF96,
    0x1DFCC..=0x1DFCC,
    0x1E000..=0x1E000,
    0x1E02F..=0x1E030,
    0x1E06D..=0x1E06E,
    0x1E8FF..=0x1E943,
    0x1EDFF..=0x1EE00,
    0x1EE03..=0x1EE05,
    0x1EE1F..=0x1EE29,
    0x1EE32..=0x1EE34,
    0x1EE37..=0x1EE3C,
    0x1EE41..=0x1EE43,
    0x1EE46..=0x1EE4D,
    0x1EE4F..=0x1EE67,
    0x1EE6A..=0x1EE6C,
    0x1EE72..=0x1EE74,
    0x1EE77..=0x1EE79,
    0x1EE7C..=0x1EE80,
    0x1EE89..=0x1EE8B,
    0x1EE9B..=0x1EE9C,
    0x1EEA0..=0x1EEA1,
    0x1EEA3..=0x1EEA5,
    0x1EEA9..=0x1EEAB,
    0x1EEBB..=0x1EEBC,
    0x1F0FF..=0x1F100,
    0x1F10A..=0x1F10B,
    0x1F10F..=0x1F110,
    0x1F12E..=0x1F130,
    0x1F14F..=0x1F150,
    0x1F169..=0x1F16A,
    0x1F16C..=0x1F16D,
    0x1F18F..=0x1F191,
    0x1F1FF..=0x1F200,
    0x1F202..=0x1F203,
    0x1F20F..=0x1F210,
    0x1F23B..=0x1F23C,
    0x1F23F..=0x1F240,
    0x1F248..=0x1F249,
    0x1F24F..=0x1F252,
    0x1FBEF..=0x1FBF0,
    0x1FBF9..=0x1FBFA,
    0x2F7FF..=0x2F800,
    0x2FA1D..=0x2FA1E,
    0xDFFFF..=0xE0000,
    0xE0FFF..=0xE1000,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];

// Characters checked against the case-insensitive property and its complement.
const CASE_INSENSITIVE_MEMBERS: &[RangeInclusive<u32>] = &[
    0x0041..=0x005A,
    0x0061..=0x007A,
    0x00B5..=0x00B5,
    0x00C0..=0x00D6,
    0x00D8..=0x00DE,
    0x00E0..=0x00F6,
    0x00F8..=0x012F,
    0x0134..=0x0137,
    0x0139..=0x013E,
    0x0141..=0x0148,
    0x014A..=0x018C,
    0x018E..=0x01A9,
    0x01AC..=0x01B9,
    0x01BC..=0x01BD,
    0x01BF..=0x01BF,
    0x01CD..=0x01EF,
    0x01F4..=0x0220,
    0x0222..=0x0233,
    0x023A..=0x0254,
    0x0256..=0x0257,
    0x0259..=0x0259,
    0x025B..=0x025C,
    0x0260..=0x0261,
    0x0263..=0x0266,
    0x0268..=0x026C,
    0x026F..=0x026F,
    0x0271..=0x0272,
    0x0275..=0x0275,
    0x027D..=0x027D,
    0x0280..=0x0280,
    0x0282..=0x0283,
    0x0287..=0x028C,
    0x0292..=0x0292,
    0x029D..=0x029E,
    0x0345..=0x0345,
    0x0370..=0x0373,
    0x0376..=0x0377,
    0x037B..=0x037D,
    0x037F..=0x037F,
    0x0386..=0x0386,
    0x0388..=0x038A,
    0x038C..=0x038C,
    0x038E..=0x03A1,
    0x03A3..=0x03D1,
    0x03D5..=0x03F1,
    0x03F3..=0x03F5,
    0x03F7..=0x03F8,
    0x03FA..=0x03FB,
    0x03FD..=0x0481,
    0x048A..=0x052F,
    0x0531..=0x0556,
    0x0561..=0x0586,
    0x10A0..=0x10C5,
    0x10C7..=0x10C7,
    0x10CD..=0x10CD,
    0x10D0..=0x10FA,
    0x10FD..=0x10FF,
    0x13A0..=0x13F5,
    0x13F8..=0x13FD,
    0x1C80..=0x1C8A,
    0x1C90..=0x1CBA,
    0x1CBD..=0x1CBF,
    0x1D79..=0x1D79,
    0x1D7D..=0x1D7D,
    0x1D8E..=0x1D8E,
    0x1E00..=0x1E95,
    0x1E9B..=0x1E9B,
    0x1EA0..=0x1F15,
    0x1F18..=0x1F1D,
    0x1F20..=0x1F45,
    0x1F48..=0x1F4D,
    0x1F51..=0x1F51,
    0x1F53..=0x1F53,
    0x1F55..=0x1F55,
    0x1F57..=0x1F57,
    0x1F59..=0x1F59,
    0x1F5B..=0x1F5B,
    0x1F5D..=0x1F5D,
    0x1F5F..=0x1F70,
    0x1F72..=0x1F72,
    0x1F74..=0x1F74,
    0x1F76..=0x1F76,
    0x1F78..=0x1F78,
    0x1F7A..=0x1F7A,
    0x1F7C..=0x1F7C,
    0x1FB0..=0x1FB1,
    0x1FB8..=0x1FBA,
    0x1FBE..=0x1FBE,
    0x1FC8..=0x1FC8,
    0x1FCA..=0x1FCA,
    0x1FD0..=0x1FD1,
    0x1FD3..=0x1FD3,
    0x1FD8..=0x1FDA,
    0x1FE0..=0x1FE1,
    0x1FE3..=0x1FE3,
    0x1FE5..=0x1FE5,
    0x1FE8..=0x1FEA,
    0x1FEC..=0x1FEC,
    0x1FF8..=0x1FF8,
    0x1FFA..=0x1FFA,
    0x2126..=0x2126,
    0x212A..=0x212B,
    0x2132..=0x2132,
    0x214E..=0x214E,
    0x2183..=0x2184,
    0x2C00..=0x2C70,
    0x2C72..=0x2C73,
    0x2C75..=0x2C76,
    0x2C7E..=0x2CE3,
    0x2CEB..=0x2CEE,
    0x2CF2..=0x2CF3,
    0x2D00..=0x2D25,
    0x2D27..=0x2D27,
    0x2D2D..=0x2D2D,
    0xA640..=0xA66D,
    0xA680..=0xA69B,
    0xA722..=0xA72F,
    0xA732..=0xA76F,
    0xA779..=0xA787,
    0xA78B..=0xA78D,
    0xA790..=0xA794,
    0xA796..=0xA7AE,
    0xA7B0..=0xA7DC,
    0xA7F5..=0xA7F6,
    0xAB53..=0xAB53,
    0xAB70..=0xABBF,
    0x10400..=0x1044F,
    0x104B0..=0x104D3,
    0x104D8..=0x104FB,
    0x10570..=0x1057A,
    0x1057C..=0x1058A,
    0x1058C..=0x10592,
    0x10594..=0x10595,
    0x10597..=0x105A1,
    0x105A3..=0x105B1,
    0x105B3..=0x105B9,
    0x105BB..=0x105BC,
    0x10C80..=0x10CB2,
    0x10CC0..=0x10CF2,
    0x10D50..=0x10D65,
    0x10D70..=0x10D85,
    0x118A0..=0x118DF,
    0x16E40..=0x16E7F,
    0x16EA0..=0x16EB8,
    0x16EBB..=0x16ED3,
    0x1E900..=0x1E943,
];
