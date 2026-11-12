//! Changes_When_Titlecased and its exact CWT alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_changes_titlecased_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Changes_When_Titlecased}",
        r"\P{CWT}",
        r"[^\p{CWT}]",
        r"[\p{CWT}K]",
        r"[\P{Changes_When_Titlecased}K]",
        r"^(?<x>[\p{CWT}K])$",
        r"(?<x>\P{Changes_When_Titlecased})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{61}\u{2028}\u{62}\u{2029}",
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
fn unicode_changes_titlecased_properties_aliases_complements_and_unions() {
    let case_insensitive_members =
        JsString::from(unicode_samples::from_ranges(CASE_INSENSITIVE_MEMBERS).as_str());
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let p = /^\p{{Changes_When_Titlecased}}$/i{mode},
                    n = /^\P{{Changes_When_Titlecased}}$/i{mode};
                [...
                    {case_insensitive_members:?}]
                .every(c => p.test(c) && n.test(c) === ('{mode}' === 'u'))
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /[\p{{Changes_When_Titlecased}}K]/i{mode},
                    n = /[\P{{Changes_When_Titlecased}}K]/i{mode},
                    x = /[^\P{{Changes_When_Titlecased}}K]/i{mode};
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
                let a = /\p{{Changes_When_Titlecased}}/{mode},
                    b = /\p{{CWT}}/i{mode},
                    n = /\P{{CWT}}/i{mode};
                a.test('\u0061') && a.test('\u0062') && b.test('\u0061') && !a.test('\u200D') && !b.test(
                    '\u005f') && b.test('ſ') && b.test('K') && n.test('\u0061') === (
                    '{mode}' ===
                    'u') && n.test('\uDB40\uDC01') && n.test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Changes_When_Titlecased}}/{mode},
                    s = /\s/{mode};
                p.test('\u0061') && p.test('\u0062') && !s.test('\u0061') && !s.test('\u0062') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{CWT}}K]/i{mode},
                    b = /[^\P{{Changes_When_Titlecased}}K]/i{mode},
                    c = /[\p{{CWT}}\P{{Changes_When_Titlecased}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0061') && a.test('ſ') && b.test('\u0062') === (
                        '{mode}' === 'v') && !b.test('K') && c.test('\uDB40\uDC01') && c.test(
                    'A') && c
                    .test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{CWT}}$/{mode},
                    b = /^\P{{CWT}}$/{mode};
                a.test('\u0061') && !a.test('\u0062\n') && !a.test('') && !a.test('\n') && b.test(
                    '\uDB40\uDC01') && !b.test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_changes_titlecased_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{Changes_When_Titlecased}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uD801\uDC28');
                m.index === 0 && m[0] === '\uD801\uDC28' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{CWT}}K]))$/dim{mode}g,
                    m = r.exec('\u200D\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{CWT}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u0061');
                m.index === 0 && m[1] === '\uDB40\uDC01' && m.indices[1][1] === 2 && r.lastIndex === 2 &&
                    Object.getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{CWT}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u0061');
                m.index === 2 && m[0] === '\u0061' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{CWT}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u0061')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_changes_titlecased_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u0062\uDB40\uDC01\u0061\u200D'.matchAll(/(?<x>\P{{CWT}})/d{mode}g)];
                a.length === 2 && a[0][1] === '\uDB40\uDC01' && a[0].index === 1 && a[0].indices[1][1] === 3 &&
                    a[1][1] === '\u200D'
            "#
        ));
        assert_true(&format!(
            r"'\u0062\u0061\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Changes_When_Titlecased}})/{mode}g,'<$1:$<x>>')==='<\u0062:\u0062><\u0061:\u0061>\uDB40\uDC01\u200D'"
        ));
        assert_true(&format!(
            r#"
                '\uDB40\uDC01\u0062\u0061\u200D'.split(/(\p{{CWT}})/{mode}).join('|') ===
                    '\uDB40\uDC01|\u0062||\u0061|\u200D' && '\uDB40\uDC01\u0061'.search(
                        /\p{{CWT}}/{mode}) === 2 && '\u0062\u0061\uDB40\uDC01'.match(
                        /\p{{CWT}}/{mode}g).join('') === '\u0062\u0061'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0061\u0062\uDB40\uDC01\u0061'.replace(/(\p{{CWT}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__\uDB40\uDC01_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_changes_titlecased_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Changes_When_Titlecased})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Changes_When_Titlecased})$' && c.unicodeSets && !c.unicode && !c
                .multiline && c.exec('\u0061').groups.x === '\u0061' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{CWT}/uy;
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
            let r = /\p{CWT}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u0061')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{CWT}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\uDB40\uDC01\u0061')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_changes_titlecased_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{CWT}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(r"/[\p{CWT}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{CWT}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x61),
                m = /[\p{CWT}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{CWT}+b/u.exec('AA')", r"/\p{CWT}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_changes_titlecased_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\b(?<x>\p{{CWT}})+\b/d{mode}g,
                    m = r.exec('\uDB40\uDC01\u0061\u0062\u0062\uFEFF');
                m.index === 2 && m[0] === '\u0061\u0062\u0062' && m.groups.x === '\u0062' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_changes_titlecased_property_unicode18_addition() {
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\u0277') === true && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\u0277') === true && n.test('\u0277') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\u0277') === true && n.test('\u0277') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\u027C') === true && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\u027C') === true && n.test('\u027C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\u027C') === true && n.test('\u027C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uA7DD') === false && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uA7DD') === true && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uA7DD') === false && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uA7DD') === false && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uA7DD') === true && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uA7DD') === false && n.test('\uA7DD') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uA7DD') === true && n.test('\uA7DD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uA7E2') === false && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uA7E2') === true && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uA7E2') === false && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uA7E2') === false && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uA7E2') === true && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uA7E2') === false && n.test('\uA7E2') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uA7E2') === true && n.test('\uA7E2') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uAB4B') === true && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uAB4B') === true && n.test('\uAB4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uAB4B') === true && n.test('\uAB4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uAB4C') === true && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uAB4C') === true && n.test('\uAB4C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uAB4C') === true && n.test('\uAB4C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uAB6C') === false && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uAB6C') === true && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uAB6C') === false && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uAB6C') === false && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uAB6C') === true && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uAB6C') === false && n.test('\uAB6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uAB6C') === true && n.test('\uAB6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uAB6D') === false && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uAB6D') === true && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uAB6D') === false && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uAB6D') === false && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uAB6D') === true && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uAB6D') === false && n.test('\uAB6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uAB6D') === true && n.test('\uAB6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF40') === false && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF40') === false && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF40') === false && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF40') === false && n.test('\uD837\uDF40') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF40') === true && n.test('\uD837\uDF40') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF41') === true && n.test('\uD837\uDF41') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF48') === false && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF48') === false && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF48') === false && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF48') === false && n.test('\uD837\uDF48') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF48') === true && n.test('\uD837\uDF48') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF49') === true && n.test('\uD837\uDF49') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF4A') === false && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF4A') === false && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF4A') === false && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF4A') === false && n.test('\uD837\uDF4A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF4A') === true && n.test('\uD837\uDF4A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF4B') === true && n.test('\uD837\uDF4B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF4D') === false && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF4D') === false && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF4D') === false && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF4D') === false && n.test('\uD837\uDF4D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF4D') === true && n.test('\uD837\uDF4D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF4E') === true && n.test('\uD837\uDF4E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF51') === false && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF51') === false && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF51') === false && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF51') === false && n.test('\uD837\uDF51') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF51') === true && n.test('\uD837\uDF51') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF52') === true && n.test('\uD837\uDF52') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF68') === false && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF68') === false && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF68') === false && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF68') === false && n.test('\uD837\uDF68') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF68') === true && n.test('\uD837\uDF68') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF69') === true && n.test('\uD837\uDF69') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF6A') === false && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF6A') === false && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF6A') === false && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF6A') === false && n.test('\uD837\uDF6A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF6A') === true && n.test('\uD837\uDF6A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF6B') === true && n.test('\uD837\uDF6B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF6C') === false && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF6C') === false && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF6C') === false && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF6C') === false && n.test('\uD837\uDF6C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF6C') === true && n.test('\uD837\uDF6C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF6D') === true && n.test('\uD837\uDF6D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF6E') === false && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF6E') === false && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF6E') === false && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF6E') === false && n.test('\uD837\uDF6E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF6E') === true && n.test('\uD837\uDF6E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF6F') === true && n.test('\uD837\uDF6F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF72') === false && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF72') === false && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF72') === false && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF72') === false && n.test('\uD837\uDF72') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF72') === true && n.test('\uD837\uDF72') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF73') === true && n.test('\uD837\uDF73') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF74') === false && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF74') === false && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF74') === false && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF74') === false && n.test('\uD837\uDF74') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF74') === true && n.test('\uD837\uDF74') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF75') === true && n.test('\uD837\uDF75') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF76') === false && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF76') === false && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF76') === false && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF76') === false && n.test('\uD837\uDF76') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF76') === true && n.test('\uD837\uDF76') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF77') === true && n.test('\uD837\uDF77') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF78') === false && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF78') === false && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF78') === false && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF78') === false && n.test('\uD837\uDF78') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF78') === true && n.test('\uD837\uDF78') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF79') === true && n.test('\uD837\uDF79') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF7A') === false && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF7A') === false && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF7A') === false && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF7A') === false && n.test('\uD837\uDF7A') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF7A') === true && n.test('\uD837\uDF7A') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF7B') === true && n.test('\uD837\uDF7B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF7C') === false && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF7C') === false && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF7C') === false && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF7C') === false && n.test('\uD837\uDF7C') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF7C') === true && n.test('\uD837\uDF7C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF7D') === true && n.test('\uD837\uDF7D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF7E') === false && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF7E') === false && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF7E') === false && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF7E') === false && n.test('\uD837\uDF7E') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF7E') === true && n.test('\uD837\uDF7E') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF7F') === true && n.test('\uD837\uDF7F') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'u'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'u');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'ui'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'ui');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'v'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'v');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Changes_When_Titlecased}$', 'vi'),
                n = new RegExp('^\\P{Changes_When_Titlecased}$', 'vi');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'u'),
                n = new RegExp('^\\P{CWT}$', 'u');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'ui'),
                n = new RegExp('^\\P{CWT}$', 'ui');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === true
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'v'),
                n = new RegExp('^\\P{CWT}$', 'v');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{CWT}$', 'vi'),
                n = new RegExp('^\\P{CWT}$', 'vi');
            p.test('\uD837\uDF95') === true && n.test('\uD837\uDF95') === false
        "#,
    );
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0xE0001..=0xE0001,
    0x0041..=0x005A,
    0x0060..=0x007B,
    0x00B4..=0x00B6,
    0x00C0..=0x00D6,
    0x00D8..=0x01A9,
    0x01AC..=0x01BA,
    0x01BC..=0x01C0,
    0x01C3..=0x01EF,
    0x01F1..=0x0220,
    0x0222..=0x0234,
    0x023A..=0x025D,
    0x025F..=0x0276,
    0x0278..=0x0278,
    0x027B..=0x027B,
    0x027D..=0x0284,
    0x0286..=0x028D,
    0x0291..=0x0293,
    0x029C..=0x029F,
    0x0344..=0x0346,
    0x0370..=0x0374,
    0x0376..=0x0378,
    0x037A..=0x037F,
    0x0386..=0x0386,
    0x0388..=0x038A,
    0x038C..=0x038C,
    0x038E..=0x03A1,
    0x03A3..=0x03AF,
    0x03B1..=0x03D2,
    0x03D4..=0x0482,
    0x048A..=0x0556,
    0x0560..=0x0588,
    0x10A0..=0x10C5,
    0x10C7..=0x10C7,
    0x10CD..=0x10CD,
    0x13A0..=0x13F5,
    0x13F7..=0x13FE,
    0x1C7F..=0x1C8B,
    0x1D78..=0x1D7A,
    0x1D7C..=0x1D7E,
    0x1D8D..=0x1D8F,
    0x1E00..=0x1E95,
    0x1E9B..=0x1E9C,
    0x1E9E..=0x1E9E,
    0x1EA0..=0x1F16,
    0x1F18..=0x1F1D,
    0x1F1F..=0x1F46,
    0x1F48..=0x1F4D,
    0x1F4F..=0x1F51,
    0x1F53..=0x1F53,
    0x1F55..=0x1F55,
    0x1F57..=0x1F59,
    0x1F5B..=0x1F5B,
    0x1F5D..=0x1F5D,
    0x1F5F..=0x1FB1,
    0x1FB3..=0x1FBF,
    0x1FC1..=0x1FCC,
    0x1FCF..=0x1FD1,
    0x1FD3..=0x1FDB,
    0x1FDF..=0x1FE1,
    0x1FE5..=0x1FE5,
    0x1FE7..=0x1FEC,
    0x1FF1..=0x1FFC,
    0x2126..=0x2126,
    0x212A..=0x212B,
    0x2132..=0x2132,
    0x214D..=0x214F,
    0x2160..=0x2180,
    0x2183..=0x2185,
    0x24B6..=0x24EA,
    0x2C00..=0x2C70,
    0x2C72..=0x2C77,
    0x2C7E..=0x2CE4,
    0x2CEB..=0x2CEF,
    0x2CF2..=0x2CF4,
    0x2CFF..=0x2D28,
    0x2D2C..=0x2D2E,
    0xA640..=0xA66E,
    0xA680..=0xA69C,
    0xA722..=0xA730,
    0xA732..=0xA770,
    0xA779..=0xA788,
    0xA78B..=0xA78D,
    0xA790..=0xA7AE,
    0xA7B0..=0xA7DC,
    0xA7F5..=0xA7F7,
    0xAB4A..=0xAB4A,
    0xAB4D..=0xAB4D,
    0xAB52..=0xAB54,
    0xAB6F..=0xABC0,
    0xFAFF..=0xFB00,
    0xFB06..=0xFB07,
    0xFB12..=0xFB13,
    0xFB17..=0xFB18,
    0xFF21..=0xFF3A,
    0xFF40..=0xFF5B,
    0x10400..=0x10450,
    0x104B0..=0x104D3,
    0x104D7..=0x104FC,
    0x10570..=0x1057A,
    0x1057C..=0x1058A,
    0x1058C..=0x10592,
    0x10594..=0x105BD,
    0x10C80..=0x10CB2,
    0x10CBF..=0x10CF3,
    0x10D50..=0x10D65,
    0x10D6F..=0x10D86,
    0x118A0..=0x118E0,
    0x16E40..=0x16E80,
    0x16EA0..=0x16EB8,
    0x16EBA..=0x16ED4,
    0x1DF42..=0x1DF42,
    0x1DF4C..=0x1DF4C,
    0x1DF4F..=0x1DF4F,
    0x1DF53..=0x1DF53,
    0x1DF70..=0x1DF70,
    0x1DF80..=0x1DF80,
    0x1DF94..=0x1DF94,
    0x1DF96..=0x1DF96,
    0x1E900..=0x1E944,
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
    0x00D8..=0x00F6,
    0x00F8..=0x012F,
    0x0132..=0x0137,
    0x0139..=0x0148,
    0x014A..=0x018C,
    0x018E..=0x01A9,
    0x01AC..=0x01B9,
    0x01BC..=0x01BD,
    0x01BF..=0x01BF,
    0x01C4..=0x01EF,
    0x01F1..=0x0220,
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
    0x038E..=0x038F,
    0x0391..=0x03A1,
    0x03A3..=0x03AF,
    0x03B1..=0x03D1,
    0x03D5..=0x03F5,
    0x03F7..=0x03FB,
    0x03FD..=0x0481,
    0x048A..=0x052F,
    0x0531..=0x0556,
    0x0561..=0x0586,
    0x10A0..=0x10C5,
    0x10C7..=0x10C7,
    0x10CD..=0x10CD,
    0x13A0..=0x13F5,
    0x13F8..=0x13FD,
    0x1C80..=0x1C8A,
    0x1D79..=0x1D79,
    0x1D7D..=0x1D7D,
    0x1D8E..=0x1D8E,
    0x1E00..=0x1E95,
    0x1E9B..=0x1E9B,
    0x1E9E..=0x1E9E,
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
    0x1F5F..=0x1F7D,
    0x1F80..=0x1FB1,
    0x1FB3..=0x1FB3,
    0x1FB8..=0x1FBC,
    0x1FBE..=0x1FBE,
    0x1FC3..=0x1FC3,
    0x1FC8..=0x1FCC,
    0x1FD0..=0x1FD1,
    0x1FD8..=0x1FDB,
    0x1FE0..=0x1FE1,
    0x1FE5..=0x1FE5,
    0x1FE8..=0x1FEC,
    0x1FF3..=0x1FF3,
    0x1FF8..=0x1FFC,
    0x2126..=0x2126,
    0x212A..=0x212B,
    0x2132..=0x2132,
    0x214E..=0x214E,
    0x2160..=0x217F,
    0x2183..=0x2184,
    0x24B6..=0x24E9,
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
    0xFF21..=0xFF3A,
    0xFF41..=0xFF5A,
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
