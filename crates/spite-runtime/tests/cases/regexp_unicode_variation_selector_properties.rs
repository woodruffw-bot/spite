//! Variation_Selector and its exact VS alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_variation_selector_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Variation_Selector}",
        r"\P{VS}",
        r"[^\p{VS}]",
        r"[\p{VS}K]",
        r"[\P{Variation_Selector}K]",
        r"^(?<x>[\p{VS}K])$",
        r"(?<x>\P{Variation_Selector})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{fe00}\u{2028}\u{fe0f}\u{2029}",
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
fn unicode_variation_selector_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Variation_Selector}}/{mode},
                    b = /\p{{VS}}/i{mode},
                    n = /\P{{VS}}/i{mode};
                a.test('\uFE00') && a.test('\uFE01') && b.test('\uFE00') && !a.test('\u200D') && !b.test(
                        '\u2010') && !b.test('ſ') && !b.test('K') && !n.test('\uFE00') && n.test('😀') && n
                    .test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Variation_Selector}}/{mode},
                    s = /\s/{mode};
                p.test('\uFE00') && p.test('\uFE01') && !s.test('\uFE00') && !s.test('\uFE01') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{VS}}K]/i{mode},
                    b = /[^\P{{Variation_Selector}}K]/i{mode},
                    c = /[\p{{VS}}\P{{Variation_Selector}}]/{mode};
                a.test('K') && a.test('k') && a.test('\uFE00') && !a.test('ſ') && b.test('\uFE01') && !b.test(
                    'K') && c.test('😀') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{VS}}$/{mode},
                    b = /^\P{{VS}}$/{mode};
                a.test('\uFE00') && !a.test('\uFE01\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_variation_selector_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{VS}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDD00');
                m.index === 0 && m[0] === '\uDB40\uDD00' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{VS}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{VS}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\ufe00');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{VS}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\ufe00');
                m.index === 2 && m[0] === '\ufe00' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{VS}}/{mode}y;r.lastIndex=1;r.exec('😀\ufe00')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_variation_selector_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\ufe01😀\ufe00G'.matchAll(/(?<x>\P{{VS}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\ufe01\ufe00😀X'.replace(/(?<x>\p{{Variation_Selector}})/{mode}g,'<$1:$<x>>')==='<\ufe01:\ufe01><\ufe00:\ufe00>😀X'"
        ));
        assert_true(&format!(
            r#"
                '😀\ufe01\ufe00G'.split(/(\p{{VS}})/{mode}).join('|') === '😀|\ufe01||\ufe00|G' &&
                    '😀\ufe00'.search(/\p{{VS}}/{mode}) === 2 && '\ufe01\ufe00😀'.match(
                        /\p{{VS}}/{mode}g).join('') === '\ufe01\ufe00'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\ufe00\ufe01😀\ufe00'.replace(/(\p{{VS}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_variation_selector_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Variation_Selector})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Variation_Selector})$' && c.unicodeSets && !c.unicode && !c
                .multiline && c.exec('\ufe00').groups.x === '\ufe00' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{VS}/uy;
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
            let r = /\p{VS}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\ufe00')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{VS}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀\ufe00')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_variation_selector_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{VS}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(r"/[\p{VS}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{VS}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0xfe00),
                m = /[\p{VS}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{VS}+b/u.exec('AA')", r"/\p{VS}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_variation_selector_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{VS}})+\B/d{mode}g,
                    m = r.exec('😀\ufe00\ufe01\ufe0f!');
                m.index === 2 && m[0] === '\ufe00\ufe01\ufe0f' && m.groups.x === '\ufe0f' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0x1F600..=0x1F600,
    0x180B..=0x180D,
    0x180F..=0x180F,
    0xFE00..=0xFE0F,
    0xE0100..=0xE01EF,
    0x180A..=0x180A,
    0x180E..=0x180E,
    0x1810..=0x1810,
    0xFDFF..=0xFDFF,
    0xFE10..=0xFE10,
    0xE00FF..=0xE00FF,
    0xE01F0..=0xE01F0,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
