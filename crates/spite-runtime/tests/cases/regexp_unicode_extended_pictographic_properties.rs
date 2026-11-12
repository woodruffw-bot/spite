//! Extended_Pictographic and its exact ExtPict alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_extended_pictographic_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Extended_Pictographic}",
        r"\P{ExtPict}",
        r"[^\p{ExtPict}]",
        r"[\p{ExtPict}K]",
        r"[\P{Extended_Pictographic}K]",
        r"^(?<x>[\p{ExtPict}K])$",
        r"(?<x>\P{Extended_Pictographic})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{a9}\u{2028}\u{ae}\u{2029}",
                "\u{ff10}\u{ff26}",
                "",
            ] {
                let input = JsString::from(input);
                for start in [0, 1, 2, input.len(), input.len() + 1] {
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
fn unicode_extended_pictographic_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let p = /^\p{{Extended_Pictographic}}$/i{mode},
                    n = /^\P{{Extended_Pictographic}}$/i{mode};
                [...'\u24C2\u24DC'].every(c => p.test(c) && n.test(c) === ('{mode}' === 'u'))
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /[\p{{Extended_Pictographic}}K]/i{mode},
                    n = /[\P{{Extended_Pictographic}}K]/i{mode},
                    x = /[^\P{{Extended_Pictographic}}K]/i{mode};
                p.test('K') && n.test('K') && !x.test('K') && [...'\u24C2\u24DC'].every(c => p.test(c) && n
                    .test(c) === ('{mode}' === 'u') && x.test(c) === ('{mode}' ===
                        'v'))
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Extended_Pictographic}}/{mode},
                    b = /\p{{ExtPict}}/i{mode},
                    n = /\P{{ExtPict}}/i{mode};
                a.test('\u00A9') && a.test('\u00AE') && b.test('\u00A9') && !a.test('\u200D') && !b.test(
                    '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u00A9') && n.test(
                    '\uDB40\uDC01') && n.test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Extended_Pictographic}}/{mode},
                    s = /\s/{mode};
                p.test('\u00A9') && p.test('\u00AE') && !s.test('\u00A9') && !s.test('\u00AE') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{ExtPict}}K]/i{mode},
                    b = /[^\P{{Extended_Pictographic}}K]/i{mode},
                    c = /[\p{{ExtPict}}\P{{Extended_Pictographic}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u00A9') && !a.test('ſ') && b.test('\u00AE') && !b.test(
                    'K') && c.test('\uDB40\uDC01') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{ExtPict}}$/{mode},
                    b = /^\P{{ExtPict}}$/{mode};
                a.test('\u00A9') && !a.test('\u00AE\n') && !a.test('') && !a.test('\n') && b.test(
                    '\uDB40\uDC01') && !b.test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_extended_pictographic_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{Extended_Pictographic}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uD83D\uDE00');
                m.index === 0 && m[0] === '\uD83D\uDE00' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{ExtPict}}K]))$/dim{mode}g,
                    m = r.exec('\u200D\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{ExtPict}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u00A9');
                m.index === 0 && m[1] === '\uDB40\uDC01' && m.indices[1][1] === 2 && r.lastIndex === 2 &&
                    Object.getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{ExtPict}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u00A9');
                m.index === 2 && m[0] === '\u00A9' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{ExtPict}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u00A9')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_extended_pictographic_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u00AE\uDB40\uDC01\u00A9\u200D'.matchAll(/(?<x>\P{{ExtPict}})/d{mode}g)];
                a.length === 2 && a[0][1] === '\uDB40\uDC01' && a[0].index === 1 && a[0].indices[1][1] === 3 &&
                    a[1][1] === '\u200D'
            "#
        ));
        assert_true(&format!(
            r"'\u00AE\u00A9\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Extended_Pictographic}})/{mode}g,'<$1:$<x>>')==='<\u00AE:\u00AE><\u00A9:\u00A9>\uDB40\uDC01\u200D'"
        ));
        assert_true(&format!(
            r#"
                '\uDB40\uDC01\u00AE\u00A9\u200D'.split(/(\p{{ExtPict}})/{mode}).join('|') ===
                    '\uDB40\uDC01|\u00AE||\u00A9|\u200D' && '\uDB40\uDC01\u00A9'.search(
                        /\p{{ExtPict}}/{mode}) === 2 && '\u00AE\u00A9\uDB40\uDC01'.match(
                        /\p{{ExtPict}}/{mode}g).join('') === '\u00AE\u00A9'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u00A9\u00AE\uDB40\uDC01\u00A9'.replace(/(\p{{ExtPict}})/{mode}g, (m, c,
                    p) => {{
                        t += p;
                        return '_'
                    }});
                x === '__\uDB40\uDC01_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_extended_pictographic_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Extended_Pictographic})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Extended_Pictographic})$' && c.unicodeSets && !c.unicode && !c
                .multiline && c.exec('\u00A9').groups.x === '\u00A9' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{ExtPict}/uy;
            r.lastIndex = {
                valueOf() {
                    t += 'i';
                    return 1
                }
            };
            let m = r.exec({
                toString() {
                    t += 's';
                    return '\uDB40\uDC01'
                }
            });
            t === 'si' && m[0] === '\uDB40\uDC01' && m.index === 0 && r.lastIndex === 2
        "#,
    );
    assert_true(
        r#"
            let r = /\p{ExtPict}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u00A9')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{ExtPict}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\uDB40\uDC01\u00A9')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_extended_pictographic_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{ExtPict}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
        .unwrap();
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
    assert_true(r"/[\p{ExtPict}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{ExtPict}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0xa9),
                m = /[\p{ExtPict}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [
        r"/\p{ExtPict}+b/u.exec('AA')",
        r"/\p{ExtPict}A/v.exec('AA')",
    ] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_extended_pictographic_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{ExtPict}})+\B/d{mode}g,
                    m = r.exec('\uDB40\uDC01\u00A9\u00AE\u00AE\uFEFF');
                m.index === 2 && m[0] === '\u00A9\u00AE\u00AE' && m.groups.x === '\u00AE' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_extended_pictographic_property_unicode18_membership_changes() {
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83C\uDDAE') === false && n.test('\uD83C\uDDAE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFDA') === false && n.test('\uD83D\uDFDA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFDB') === false && n.test('\uD83D\uDFDB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF1') === false && n.test('\uD83D\uDFF1') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF2') === false && n.test('\uD83D\uDFF2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF3') === false && n.test('\uD83D\uDFF3') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF4') === false && n.test('\uD83D\uDFF4') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF5') === false && n.test('\uD83D\uDFF5') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF6') === false && n.test('\uD83D\uDFF6') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF7') === false && n.test('\uD83D\uDFF7') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF8') === false && n.test('\uD83D\uDFF8') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFF9') === false && n.test('\uD83D\uDFF9') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFFA') === false && n.test('\uD83D\uDFFA') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFFB') === false && n.test('\uD83D\uDFFB') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFFC') === false && n.test('\uD83D\uDFFC') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFFD') === false && n.test('\uD83D\uDFFD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFFE') === false && n.test('\uD83D\uDFFE') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'u'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'u');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'ui'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'ui');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'v'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'v');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Extended_Pictographic}$', 'vi'),
                n = new RegExp('^\\P{Extended_Pictographic}$', 'vi');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'u'),
                n = new RegExp('^\\P{ExtPict}$', 'u');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'ui'),
                n = new RegExp('^\\P{ExtPict}$', 'ui');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'v'),
                n = new RegExp('^\\P{ExtPict}$', 'v');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{ExtPict}$', 'vi'),
                n = new RegExp('^\\P{ExtPict}$', 'vi');
            p.test('\uD83D\uDFFF') === false && n.test('\uD83D\uDFFF') === true
        "#,
    );
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0xE0001..=0xE0001,
    0x00A8..=0x00AA,
    0x00AD..=0x00AF,
    0x203B..=0x203D,
    0x2048..=0x204A,
    0x2121..=0x2123,
    0x2138..=0x213A,
    0x2193..=0x2194,
    0x2199..=0x219A,
    0x21A8..=0x21AB,
    0x2319..=0x231C,
    0x2327..=0x2329,
    0x23CE..=0x23D0,
    0x23E8..=0x23E9,
    0x23F3..=0x23F4,
    0x23F7..=0x23F8,
    0x23FA..=0x23FB,
    0x24C1..=0x24C3,
    0x24DC..=0x24DC,
    0x25A9..=0x25AC,
    0x25B5..=0x25B7,
    0x25BF..=0x25C1,
    0x25FA..=0x25FB,
    0x25FE..=0x2600,
    0x2604..=0x2605,
    0x260D..=0x2619,
    0x261C..=0x2627,
    0x2629..=0x262B,
    0x262D..=0x2630,
    0x2637..=0x2638,
    0x263A..=0x263B,
    0x263F..=0x2643,
    0x2647..=0x2648,
    0x2653..=0x2654,
    0x265E..=0x2669,
    0x267A..=0x2680,
    0x2691..=0x2692,
    0x2697..=0x269D,
    0x269F..=0x26A2,
    0x26A6..=0x26AC,
    0x26AF..=0x26B2,
    0x26BC..=0x26BF,
    0x26C3..=0x26C9,
    0x26CD..=0x26D5,
    0x26E8..=0x26EB,
    0x26EF..=0x26F0,
    0x26F5..=0x26F7,
    0x26FA..=0x26FE,
    0x2701..=0x2708,
    0x270D..=0x2717,
    0x271C..=0x271E,
    0x2720..=0x2722,
    0x2727..=0x2729,
    0x2732..=0x2735,
    0x2743..=0x2748,
    0x274B..=0x274F,
    0x2752..=0x2753,
    0x2755..=0x2758,
    0x2762..=0x2765,
    0x2794..=0x2795,
    0x2797..=0x2798,
    0x27A0..=0x27A2,
    0x27AF..=0x27B1,
    0x27BE..=0x27C0,
    0x2933..=0x2936,
    0x2B04..=0x2B05,
    0x2B07..=0x2B08,
    0x2B1A..=0x2B1D,
    0x2B4F..=0x2B51,
    0x2B54..=0x2B56,
    0x302F..=0x3031,
    0x303C..=0x303E,
    0x3296..=0x329A,
    0x1F003..=0x1F005,
    0x1F02B..=0x1F02C,
    0x1F02F..=0x1F030,
    0x1F093..=0x1F094,
    0x1F09F..=0x1F0A0,
    0x1F0AE..=0x1F0B1,
    0x1F0BF..=0x1F0C1,
    0x1F0CE..=0x1F0D1,
    0x1F0F5..=0x1F0F6,
    0x1F0FF..=0x1F100,
    0x1F16F..=0x1F172,
    0x1F17D..=0x1F180,
    0x1F18D..=0x1F191,
    0x1F19A..=0x1F19B,
    0x1F1AF..=0x1F1AF,
    0x1F1E5..=0x1F1E6,
    0x1F200..=0x1F201,
    0x1F20F..=0x1F210,
    0x1F219..=0x1F21B,
    0x1F22E..=0x1F232,
    0x1F23A..=0x1F23C,
    0x1F23F..=0x1F240,
    0x1F248..=0x1F249,
    0x1F25F..=0x1F260,
    0x1F265..=0x1F266,
    0x1F321..=0x1F324,
    0x1F393..=0x1F399,
    0x1F39B..=0x1F39E,
    0x1F3F0..=0x1F3F3,
    0x1F3F5..=0x1F3F7,
    0x1F3FA..=0x1F3FB,
    0x1F3FF..=0x1F400,
    0x1F4FD..=0x1F4FF,
    0x1F53D..=0x1F53E,
    0x1F548..=0x1F549,
    0x1F54E..=0x1F550,
    0x1F567..=0x1F568,
    0x1F56E..=0x1F573,
    0x1F57A..=0x1F57B,
    0x1F586..=0x1F58A,
    0x1F58D..=0x1F591,
    0x1F594..=0x1F597,
    0x1F5A3..=0x1F5A9,
    0x1F5B0..=0x1F5B3,
    0x1F5BB..=0x1F5BD,
    0x1F5C1..=0x1F5C2,
    0x1F5C4..=0x1F5C5,
    0x1F5D0..=0x1F5D1,
    0x1F5D3..=0x1F5D4,
    0x1F5DB..=0x1F5DC,
    0x1F5DE..=0x1F5E4,
    0x1F5E7..=0x1F5E9,
    0x1F5EE..=0x1F5F0,
    0x1F5F2..=0x1F5F4,
    0x1F5F9..=0x1F5FA,
    0x1F64F..=0x1F650,
    0x1F67F..=0x1F680,
    0x1F6C5..=0x1F6C6,
    0x1F6CA..=0x1F6CB,
    0x1F6D2..=0x1F6D5,
    0x1F6E5..=0x1F6E6,
    0x1F6E8..=0x1F6EB,
    0x1F6F0..=0x1F6F3,
    0x1F6FF..=0x1F700,
    0x1F7DC..=0x1F7DC,
    0x1F7F0..=0x1F7F0,
    0x1F80B..=0x1F80C,
    0x1F80F..=0x1F810,
    0x1F847..=0x1F848,
    0x1F84F..=0x1F850,
    0x1F859..=0x1F85A,
    0x1F85F..=0x1F860,
    0x1F887..=0x1F888,
    0x1F88F..=0x1F890,
    0x1F8AD..=0x1F8B0,
    0x1F8BB..=0x1F8BC,
    0x1F8BF..=0x1F8C2,
    0x1F8CF..=0x1F8D0,
    0x1F8D8..=0x1F8D9,
    0x1F8FF..=0x1F900,
    0x1F90B..=0x1F90C,
    0x1F93A..=0x1F93C,
    0x1F945..=0x1F947,
    0x1F9FF..=0x1FA00,
    0x1FA57..=0x1FA58,
    0x1FA5F..=0x1FA60,
    0x1FA6D..=0x1FA6E,
    0x1FAFF..=0x1FB00,
    0x1FBFF..=0x1FC00,
    0x1FFFD..=0x1FFFE,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
