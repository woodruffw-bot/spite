//! Bidi_Mirrored and its exact Bidi_M alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_bidi_mirrored_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Bidi_Mirrored}",
        r"\P{Bidi_M}",
        r"[^\p{Bidi_M}]",
        r"[\p{Bidi_M}K]",
        r"[\P{Bidi_Mirrored}K]",
        r"^(?<x>[\p{Bidi_M}K])$",
        r"(?<x>\P{Bidi_Mirrored})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{28}\u{2028}\u{ab}\u{2029}",
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
fn unicode_bidi_mirrored_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Bidi_Mirrored}}/{mode},
                    b = /\p{{Bidi_M}}/i{mode},
                    n = /\P{{Bidi_M}}/i{mode};
                a.test('\u0028') && a.test('\u0029') && b.test('\u0028') && !a.test('\u200D') && !b.test(
                        '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u0028') && n.test('😀') && n
                    .test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Bidi_Mirrored}}/{mode},
                    s = /\s/{mode};
                p.test('\u0028') && p.test('\u0029') && !s.test('\u0028') && !s.test('\u0029') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{Bidi_M}}K]/i{mode},
                    b = /[^\P{{Bidi_Mirrored}}K]/i{mode},
                    c = /[\p{{Bidi_M}}\P{{Bidi_Mirrored}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0028') && !a.test('ſ') && b.test('\u0029') && !b.test(
                    'K') && c.test('😀') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{Bidi_M}}$/{mode},
                    b = /^\P{{Bidi_M}}$/{mode};
                a.test('\u0028') && !a.test('\u0029\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{Bidi_M}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{Bidi_M}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\u0028');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Bidi_M}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\u0028');
                m.index === 2 && m[0] === '\u0028' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Bidi_M}}/{mode}y;r.lastIndex=1;r.exec('😀\u0028')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u0029😀\u0028G'.matchAll(/(?<x>\P{{Bidi_M}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\u0029\u0028😀X'.replace(/(?<x>\p{{Bidi_Mirrored}})/{mode}g,'<$1:$<x>>')==='<\u0029:\u0029><\u0028:\u0028>😀X'"
        ));
        assert_true(&format!(
            r#"
                '😀\u0029\u0028G'.split(/(\p{{Bidi_M}})/{mode}).join('|') === '😀|\u0029||\u0028|G' &&
                    '😀\u0028'.search(/\p{{Bidi_M}}/{mode}) === 2 && '\u0029\u0028😀'.match(
                        /\p{{Bidi_M}}/{mode}g).join('') === '\u0029\u0028'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0028\u0029😀\u0028'.replace(/(\p{{Bidi_M}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Bidi_Mirrored})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Bidi_Mirrored})$' && c.unicodeSets && !c.unicode && !c.multiline && c
                .exec('\u0028').groups.x === '\u0028' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{Bidi_M}/uy;
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
            let r = /\p{Bidi_M}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u0028')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{Bidi_M}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀\u0028')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_bidi_mirrored_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Bidi_M}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(r"/[\p{Bidi_M}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{Bidi_M}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x28),
                m = /[\p{Bidi_M}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{Bidi_M}+b/u.exec('AA')", r"/\p{Bidi_M}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_bidi_mirrored_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{Bidi_M}})+\B/d{mode}g,
                    m = r.exec('😀\u0028\u0029\u00AB!');
                m.index === 2 && m[0] === '\u0028\u0029\u00AB' && m.groups.x === '\u00AB' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_bidi_mirrored_property_unicode18_additions() {
    let mut comparisons = 0;
    for point in [
        0x2e62u32, 0x2e63u32, 0x1ceddu32, 0x1cedeu32, 0x1cedfu32, 0x1db03u32, 0x1db04u32,
        0x1db05u32, 0x1db06u32, 0x1db07u32, 0x1db08u32, 0x1db09u32, 0x1db10u32, 0x1db17u32,
        0x1db18u32, 0x1db1bu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Bidi_Mirrored", "Bidi_M"] {
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
    assert_eq!(comparisons, 128);
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0x1F600..=0x1F600,
    0x0027..=0x002A,
    0x003B..=0x003F,
    0x005A..=0x005E,
    0x007A..=0x007E,
    0x00AA..=0x00AC,
    0x00BA..=0x00BC,
    0x0F39..=0x0F3A,
    0x0F3D..=0x0F3E,
    0x169A..=0x169D,
    0x2038..=0x203B,
    0x2044..=0x2047,
    0x207C..=0x207F,
    0x208C..=0x208F,
    0x213F..=0x2141,
    0x2200..=0x2201,
    0x2204..=0x2205,
    0x2207..=0x2208,
    0x220D..=0x220E,
    0x2210..=0x2212,
    0x2214..=0x2217,
    0x2219..=0x221A,
    0x221D..=0x221F,
    0x2222..=0x2227,
    0x222A..=0x222B,
    0x2233..=0x2234,
    0x2238..=0x223B,
    0x224C..=0x224D,
    0x2251..=0x2252,
    0x2255..=0x2256,
    0x225E..=0x2264,
    0x226B..=0x226D,
    0x228C..=0x228F,
    0x2292..=0x2293,
    0x2297..=0x2299,
    0x22A1..=0x22A6,
    0x22B8..=0x22B9,
    0x22BD..=0x22C0,
    0x22C8..=0x22C9,
    0x22CD..=0x22D2,
    0x22D5..=0x22D6,
    0x22ED..=0x22F0,
    0x22FF..=0x2300,
    0x2307..=0x2308,
    0x230B..=0x230C,
    0x231F..=0x2322,
    0x2328..=0x232B,
    0x2767..=0x2768,
    0x2775..=0x2776,
    0x27BF..=0x27C3,
    0x27C6..=0x27CB,
    0x27CD..=0x27CE,
    0x27D2..=0x27D3,
    0x27D6..=0x27D7,
    0x27DB..=0x27DC,
    0x27DE..=0x27DF,
    0x27E1..=0x27E2,
    0x27EF..=0x27F0,
    0x2982..=0x2983,
    0x2998..=0x299B,
    0x29A0..=0x29A2,
    0x29AF..=0x29B0,
    0x29B7..=0x29B9,
    0x29BF..=0x29C0,
    0x29C5..=0x29C6,
    0x29C8..=0x29CA,
    0x29CD..=0x29CE,
    0x29D2..=0x29D8,
    0x29DC..=0x29DD,
    0x29E0..=0x29E3,
    0x29E5..=0x29EA,
    0x29F3..=0x29F4,
    0x29F9..=0x29FE,
    0x2A09..=0x2A0A,
    0x2A1C..=0x2A1E,
    0x2A21..=0x2A2B,
    0x2A2E..=0x2A2F,
    0x2A33..=0x2A36,
    0x2A3B..=0x2A3C,
    0x2A3E..=0x2A3F,
    0x2A56..=0x2A59,
    0x2A63..=0x2A66,
    0x2A69..=0x2A6A,
    0x2A6D..=0x2A75,
    0x2A78..=0x2A79,
    0x2AA3..=0x2AA6,
    0x2AAD..=0x2AAF,
    0x2AD6..=0x2AD7,
    0x2ADB..=0x2ADF,
    0x2AE1..=0x2AE2,
    0x2AE6..=0x2AE7,
    0x2AEB..=0x2AEC,
    0x2AEE..=0x2AEF,
    0x2AF2..=0x2AF4,
    0x2AF6..=0x2AF7,
    0x2AFB..=0x2AFE,
    0x2BFD..=0x2BFF,
    0x2E01..=0x2E02,
    0x2E05..=0x2E06,
    0x2E08..=0x2E0E,
    0x2E1B..=0x2E20,
    0x2E29..=0x2E2A,
    0x2E54..=0x2E55,
    0x2E5C..=0x2E5D,
    0x2E61..=0x2E61,
    0x2E64..=0x2E64,
    0x3007..=0x3008,
    0x3011..=0x3014,
    0x301B..=0x301C,
    0xFE58..=0xFE59,
    0xFE5E..=0xFE5F,
    0xFE63..=0xFE66,
    0xFF07..=0xFF0A,
    0xFF1B..=0xFF1F,
    0xFF3A..=0xFF3E,
    0xFF5A..=0xFF64,
    0x1CEDC..=0x1CEDC,
    0x1CEE0..=0x1CEE0,
    0x1D6DA..=0x1D6DC,
    0x1D714..=0x1D716,
    0x1D74E..=0x1D750,
    0x1D788..=0x1D78A,
    0x1D7C2..=0x1D7C4,
    0x1DB02..=0x1DB02,
    0x1DB0A..=0x1DB0A,
    0x1DB0F..=0x1DB0F,
    0x1DB11..=0x1DB11,
    0x1DB16..=0x1DB16,
    0x1DB19..=0x1DB1A,
    0x1DB1C..=0x1DB1C,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
