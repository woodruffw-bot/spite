//! Emoji and its exact Emoji alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_emoji_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Emoji}",
        r"\P{Emoji}",
        r"[^\p{Emoji}]",
        r"[\p{Emoji}K]",
        r"[\P{Emoji}K]",
        r"^(?<x>[\p{Emoji}K])$",
        r"(?<x>\P{Emoji})",
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
fn unicode_emoji_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let p = /^\p{{Emoji}}$/i{mode},
                    n = /^\P{{Emoji}}$/i{mode};
                [...'\u24C2\u24DC'].every(c => p.test(c) && n.test(c) === ('{mode}' === 'u'))
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /[\p{{Emoji}}K]/i{mode},
                    n = /[\P{{Emoji}}K]/i{mode},
                    x = /[^\P{{Emoji}}K]/i{mode};
                p.test('K') && n.test('K') && !x.test('K') && [...'\u24C2\u24DC'].every(c => p.test(c) && n
                    .test(c) === ('{mode}' === 'u') && x.test(c) === ('{mode}' ===
                        'v'))
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Emoji}}/{mode},
                    b = /\p{{Emoji}}/i{mode},
                    n = /\P{{Emoji}}/i{mode};
                a.test('\u00A9') && a.test('\u00AE') && b.test('\u00A9') && !a.test('\u200D') && !b.test(
                    '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u00A9') && n.test(
                    '\uDB40\uDC01') && n.test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Emoji}}/{mode},
                    s = /\s/{mode};
                p.test('\u00A9') && p.test('\u00AE') && !s.test('\u00A9') && !s.test('\u00AE') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{Emoji}}K]/i{mode},
                    b = /[^\P{{Emoji}}K]/i{mode},
                    c = /[\p{{Emoji}}\P{{Emoji}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u00A9') && !a.test('ſ') && b.test('\u00AE') && !b.test(
                    'K') && c.test('\uDB40\uDC01') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{Emoji}}$/{mode},
                    b = /^\P{{Emoji}}$/{mode};
                a.test('\u00A9') && !a.test('\u00AE\n') && !a.test('') && !a.test('\n') && b.test(
                    '\uDB40\uDC01') && !b.test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_emoji_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{Emoji}})/d{mode}y;
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
                let r = /^(?<x>([\p{{Emoji}}K]))$/dim{mode}g,
                    m = r.exec('\u200D\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{Emoji}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u00A9');
                m.index === 0 && m[1] === '\uDB40\uDC01' && m.indices[1][1] === 2 && r.lastIndex === 2 &&
                    Object.getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Emoji}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u00A9');
                m.index === 2 && m[0] === '\u00A9' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Emoji}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u00A9')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u00AE\uDB40\uDC01\u00A9\u200D'.matchAll(/(?<x>\P{{Emoji}})/d{mode}g)];
                a.length === 2 && a[0][1] === '\uDB40\uDC01' && a[0].index === 1 && a[0].indices[1][1] === 3 &&
                    a[1][1] === '\u200D'
            "#
        ));
        assert_true(&format!(
            r"'\u00AE\u00A9\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Emoji}})/{mode}g,'<$1:$<x>>')==='<\u00AE:\u00AE><\u00A9:\u00A9>\uDB40\uDC01\u200D'"
        ));
        assert_true(&format!(
            r#"
                '\uDB40\uDC01\u00AE\u00A9\u200D'.split(/(\p{{Emoji}})/{mode}).join('|') ===
                    '\uDB40\uDC01|\u00AE||\u00A9|\u200D' && '\uDB40\uDC01\u00A9'.search(
                        /\p{{Emoji}}/{mode}) === 2 && '\u00AE\u00A9\uDB40\uDC01'.match(
                        /\p{{Emoji}}/{mode}g).join('') === '\u00AE\u00A9'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u00A9\u00AE\uDB40\uDC01\u00A9'.replace(/(\p{{Emoji}})/{mode}g, (m, c,
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
fn unicode_emoji_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Emoji})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Emoji})$' && c.unicodeSets && !c.unicode && !c.multiline && c.exec(
                '\u00A9').groups.x === '\u00A9' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{Emoji}/uy;
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
            let r = /\p{Emoji}/vy;
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
            let r = /\p{Emoji}/vy;
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
fn unicode_emoji_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Emoji}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
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
    assert_true(r"/[\p{Emoji}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{Emoji}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0xa9),
                m = /[\p{Emoji}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{Emoji}+b/u.exec('AA')", r"/\p{Emoji}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_emoji_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{Emoji}})+\B/d{mode}g,
                    m = r.exec('\uDB40\uDC01\u00A9\u00AE\u00AE\uFEFF');
                m.index === 2 && m[0] === '\u00A9\u00AE\u00AE' && m.groups.x === '\u00AE' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_emoji_property_unicode18_membership_changes() {
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'u'),
                n = new RegExp('^\\P{Emoji}$', 'u');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'ui'),
                n = new RegExp('^\\P{Emoji}$', 'ui');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'v'),
                n = new RegExp('^\\P{Emoji}$', 'v');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji}$', 'vi'),
                n = new RegExp('^\\P{Emoji}$', 'vi');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0xE0001..=0xE0001,
    0x0022..=0x0024,
    0x0029..=0x002B,
    0x002F..=0x0030,
    0x0039..=0x003A,
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
    0x1F0CE..=0x1F0D0,
    0x1F16F..=0x1F172,
    0x1F17D..=0x1F180,
    0x1F18D..=0x1F191,
    0x1F19A..=0x1F19B,
    0x1F1E5..=0x1F1E6,
    0x1F1FF..=0x1F203,
    0x1F219..=0x1F21B,
    0x1F22E..=0x1F232,
    0x1F23A..=0x1F23B,
    0x1F24F..=0x1F252,
    0x1F2FF..=0x1F300,
    0x1F321..=0x1F324,
    0x1F393..=0x1F399,
    0x1F39B..=0x1F39E,
    0x1F3F0..=0x1F3F3,
    0x1F3F5..=0x1F3F7,
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
    0x1F6DA..=0x1F6DC,
    0x1F6E5..=0x1F6E6,
    0x1F6E8..=0x1F6ED,
    0x1F6EF..=0x1F6F3,
    0x1F6FC..=0x1F6FD,
    0x1F7DF..=0x1F7E0,
    0x1F7EB..=0x1F7EC,
    0x1F7EF..=0x1F7F1,
    0x1F90B..=0x1F90C,
    0x1F93A..=0x1F93C,
    0x1F945..=0x1F947,
    0x1F9FF..=0x1FA00,
    0x1FA6F..=0x1FA70,
    0x1FA7C..=0x1FA7D,
    0x1FA7F..=0x1FA80,
    0x1FAC6..=0x1FAC9,
    0x1FACB..=0x1FACB,
    0x1FADE..=0x1FADF,
    0x1FAEC..=0x1FAEC,
    0x1FAEE..=0x1FAEF,
    0x1FAFB..=0x1FAFB,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
