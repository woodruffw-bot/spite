//! Math and its exact Math alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_math_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Math}",
        r"\P{Math}",
        r"[^\p{Math}]",
        r"[\p{Math}K]",
        r"[\P{Math}K]",
        r"^(?<x>[\p{Math}K])$",
        r"(?<x>\P{Math})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{2b}\u{2028}\u{d7}\u{2029}",
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
fn unicode_math_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let p = /^\p{{Math}}$/i{mode},
                    n = /^\P{{Math}}$/i{mode};
                '\u0392\u0395\u0398\u039A\u03A1\u03A6\u03B2\u03B5\u03B8\u03BA\u03C1\u03C6\u03D0\u03D1\u03D5\u03F0\u03F1\u03F4\u03F5'
                .split('').every(c => p.test(c) && n.test(c) === ('{mode}' === 'u'))
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /[\p{{Math}}K]/i{mode},
                    n = /[\P{{Math}}K]/i{mode},
                    x = /[^\P{{Math}}K]/i{mode};
                p.test('K') && n.test('K') && !x.test('K') &&
                    '\u0392\u0395\u0398\u039A\u03A1\u03A6\u03B2\u03B5\u03B8\u03BA\u03C1\u03C6\u03D0\u03D1\u03D5\u03F0\u03F1\u03F4\u03F5'
                    .split('').every(c => p.test(c) && n.test(c) === ('{mode}' === 'u') && x.test(
                        c) ===
                        ('{mode}' === 'v'))
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Math}}/{mode},
                    b = /\p{{Math}}/i{mode},
                    n = /\P{{Math}}/i{mode};
                a.test('\u002B') && a.test('\u00D7') && b.test('\u002B') && !a.test('\u200D') && !b.test(
                    '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u002B') && n.test(
                    '\uDB40\uDC01') && n.test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Math}}/{mode},
                    s = /\s/{mode};
                p.test('\u002B') && p.test('\u00D7') && !s.test('\u002B') && !s.test('\u00D7') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{Math}}K]/i{mode},
                    b = /[^\P{{Math}}K]/i{mode},
                    c = /[\p{{Math}}\P{{Math}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u002B') && !a.test('ſ') && b.test('\u00D7') && !b.test(
                    'K') && c.test('\uDB40\uDC01') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{Math}}$/{mode},
                    b = /^\P{{Math}}$/{mode};
                a.test('\u002B') && !a.test('\u00D7\n') && !a.test('') && !a.test('\n') && b.test(
                    '\uDB40\uDC01') && !b.test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_math_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{Math}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uD835\uDC00');
                m.index === 0 && m[0] === '\uD835\uDC00' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{Math}}K]))$/dim{mode}g,
                    m = r.exec('\u200D\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{Math}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u002B');
                m.index === 0 && m[1] === '\uDB40\uDC01' && m.indices[1][1] === 2 && r.lastIndex === 2 &&
                    Object.getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Math}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u002B');
                m.index === 2 && m[0] === '\u002B' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Math}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u002B')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_math_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u00D7\uDB40\uDC01\u002B\u200D'.matchAll(/(?<x>\P{{Math}})/d{mode}g)];
                a.length === 2 && a[0][1] === '\uDB40\uDC01' && a[0].index === 1 && a[0].indices[1][1] === 3 &&
                    a[1][1] === '\u200D'
            "#
        ));
        assert_true(&format!(
            r"'\u00D7\u002B\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Math}})/{mode}g,'<$1:$<x>>')==='<\u00D7:\u00D7><\u002B:\u002B>\uDB40\uDC01\u200D'"
        ));
        assert_true(&format!(
            r#"
                '\uDB40\uDC01\u00D7\u002B\u200D'.split(/(\p{{Math}})/{mode}).join('|') ===
                    '\uDB40\uDC01|\u00D7||\u002B|\u200D' && '\uDB40\uDC01\u002B'.search(
                        /\p{{Math}}/{mode}) === 2 && '\u00D7\u002B\uDB40\uDC01'.match(
                        /\p{{Math}}/{mode}g).join('') === '\u00D7\u002B'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u002B\u00D7\uDB40\uDC01\u002B'.replace(/(\p{{Math}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__\uDB40\uDC01_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_math_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Math})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Math})$' && c.unicodeSets && !c.unicode && !c.multiline && c.exec(
                '\u002B').groups.x === '\u002B' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{Math}/uy;
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
            let r = /\p{Math}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u002B')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{Math}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\uDB40\uDC01\u002B')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_math_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Math}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
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
    assert_true(r"/[\p{Math}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{Math}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x2b),
                m = /[\p{Math}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{Math}+b/u.exec('AA')", r"/\p{Math}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_math_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{Math}})+\B/d{mode}g,
                    m = r.exec('\uDB40\uDC01\u002B\u00D7\u00D7\uFEFF');
                m.index === 2 && m[0] === '\u002B\u00D7\u00D7' && m.groups.x === '\u00D7' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_math_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x1ceddu32, 0x1cedeu32, 0x1cedfu32, 0x1cef1u32, 0x1cef7u32, 0x1cefdu32, 0x1d6a6u32,
        0x1db00u32, 0x1db0eu32, 0x1db1cu32, 0x1f7dbu32, 0x1f7f1u32, 0x1f7f8u32, 0x1f7ffu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Math"] {
            for flags in ["u", "ui", "v", "vi"] {
                assert_true(&format!(
                    r#"
                        let p = new RegExp('^\\p{{{name}}}$', '{flags}'),
                            n = new RegExp('^\\P{{{name}}}$', '{flags}');
                        p.test({text:?}) && !n.test({text:?})
                    "#
                ));
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 56);
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0xE0001..=0xE0001,
    0x002A..=0x002C,
    0x003B..=0x003C,
    0x003E..=0x003F,
    0x005D..=0x005F,
    0x007B..=0x007F,
    0x00AB..=0x00AD,
    0x00B0..=0x00B2,
    0x00D6..=0x00D8,
    0x00F6..=0x00F8,
    0x0392..=0x0392,
    0x0395..=0x0395,
    0x0398..=0x0398,
    0x039A..=0x039A,
    0x03A1..=0x03A1,
    0x03A6..=0x03A6,
    0x03B2..=0x03B2,
    0x03B5..=0x03B5,
    0x03B8..=0x03B8,
    0x03BA..=0x03BA,
    0x03C1..=0x03C1,
    0x03C6..=0x03C6,
    0x03CF..=0x03D6,
    0x03EF..=0x03F7,
    0x0605..=0x0606,
    0x0608..=0x0609,
    0x2015..=0x2017,
    0x2031..=0x2032,
    0x2034..=0x2035,
    0x203F..=0x2041,
    0x2043..=0x2045,
    0x2051..=0x2053,
    0x2060..=0x2061,
    0x2064..=0x2065,
    0x2079..=0x207A,
    0x207E..=0x207F,
    0x2089..=0x208A,
    0x208E..=0x208F,
    0x20CF..=0x20D0,
    0x20DC..=0x20DD,
    0x20E0..=0x20E2,
    0x20E4..=0x20E7,
    0x20EA..=0x20EB,
    0x20EF..=0x20F0,
    0x2101..=0x2103,
    0x2106..=0x210A,
    0x2113..=0x2118,
    0x211D..=0x211E,
    0x2123..=0x2125,
    0x2127..=0x212F,
    0x2131..=0x2133,
    0x2138..=0x2139,
    0x213B..=0x213C,
    0x2149..=0x214C,
    0x218F..=0x2190,
    0x21A7..=0x21A9,
    0x21AE..=0x21B2,
    0x21B5..=0x21B8,
    0x21BB..=0x21BC,
    0x21DB..=0x21DE,
    0x21E3..=0x21E6,
    0x21F3..=0x21F4,
    0x22FF..=0x2300,
    0x2307..=0x2308,
    0x230B..=0x230C,
    0x231F..=0x2322,
    0x237B..=0x237D,
    0x239A..=0x239B,
    0x23B5..=0x23B8,
    0x23CF..=0x23D1,
    0x23DB..=0x23DC,
    0x23E2..=0x23E3,
    0x259F..=0x25A2,
    0x25AD..=0x25AE,
    0x25B7..=0x25B8,
    0x25BB..=0x25BC,
    0x25C1..=0x25C2,
    0x25C5..=0x25CC,
    0x25CE..=0x25CF,
    0x25D3..=0x25D4,
    0x25E1..=0x25E7,
    0x25EC..=0x25ED,
    0x25F7..=0x25F8,
    0x25FF..=0x2600,
    0x2604..=0x2607,
    0x263F..=0x2643,
    0x265F..=0x2660,
    0x2663..=0x2664,
    0x266C..=0x266D,
    0x266F..=0x2670,
    0x27BF..=0x27C0,
    0x27FF..=0x2800,
    0x28FF..=0x2900,
    0x2AFF..=0x2B00,
    0x2B2F..=0x2B30,
    0x2B44..=0x2B47,
    0x2B4C..=0x2B4D,
    0xFB28..=0xFB2A,
    0xFE60..=0xFE61,
    0xFE66..=0xFE69,
    0xFF0A..=0xFF0C,
    0xFF1B..=0xFF1C,
    0xFF1E..=0xFF1F,
    0xFF3B..=0xFF3F,
    0xFF5B..=0xFF5F,
    0xFFE1..=0xFFE3,
    0xFFE8..=0xFFE9,
    0xFFEC..=0xFFED,
    0x10D8D..=0x10D90,
    0x1CEDC..=0x1CEDC,
    0x1CEE0..=0x1CEE0,
    0x1CEEF..=0x1CEF0,
    0x1CEFE..=0x1CEFE,
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
    0x1DAFF..=0x1DAFF,
    0x1DB1D..=0x1DB1D,
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
    0x1EEEF..=0x1EEF2,
    0x1F7DA..=0x1F7DA,
    0x1F7DC..=0x1F7DC,
    0x1F7F0..=0x1F7F0,
    0x1F800..=0x1F800,
    0x1F8CF..=0x1F8D0,
    0x1F8D8..=0x1F8D9,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
