//! Diacritic and its exact Dia alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_diacritic_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Diacritic}",
        r"\P{Dia}",
        r"[^\p{Dia}]",
        r"[\p{Dia}K]",
        r"[\P{Diacritic}K]",
        r"^(?<x>[\p{Dia}K])$",
        r"(?<x>\P{Diacritic})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{300}\u{2028}\u{301}\u{2029}",
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
fn unicode_diacritic_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let p = /^\p{{Dia}}$/i{mode},
                    n = /^\P{{Dia}}$/i{mode};
                '\u0345\u0399\u03B9\u1FBE'.split('').every(c => p.test(c) && n.test(c) === (
                    '{mode}' ===
                    'u'))
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /[\p{{Dia}}K]/i{mode},
                    n = /[\P{{Dia}}K]/i{mode},
                    x = /[^\P{{Dia}}K]/i{mode};
                p.test('K') && n.test('K') && !x.test('K') && '\u0345\u0399\u03B9\u1FBE'.split('').every(c => p
                    .test(c) && n.test(c) === ('{mode}' === 'u') && x.test(c) === (
                        '{mode}' === 'v'))
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Diacritic}}/{mode},
                    b = /\p{{Dia}}/i{mode},
                    n = /\P{{Dia}}/i{mode};
                a.test('\u0300') && a.test('\u0301') && b.test('\u0300') && !a.test('\u200D') && !b.test(
                        '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u0300') && n.test('😀') && n
                    .test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Diacritic}}/{mode},
                    s = /\s/{mode};
                p.test('\u0300') && p.test('\u0301') && !s.test('\u0300') && !s.test('\u0301') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{Dia}}K]/i{mode},
                    b = /[^\P{{Diacritic}}K]/i{mode},
                    c = /[\p{{Dia}}\P{{Diacritic}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0300') && !a.test('ſ') && b.test('\u0301') && !b.test(
                    'K') && c.test('😀') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{Dia}}$/{mode},
                    b = /^\P{{Dia}}$/{mode};
                a.test('\u0300') && !a.test('\u0301\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_diacritic_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{Dia}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uD834\uDD67');
                m.index === 0 && m[0] === '\uD834\uDD67' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{Dia}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{Dia}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\u0300');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Dia}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\u0300');
                m.index === 2 && m[0] === '\u0300' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Dia}}/{mode}y;r.lastIndex=1;r.exec('😀\u0300')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_diacritic_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u0301😀\u0300G'.matchAll(/(?<x>\P{{Dia}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\u0301\u0300😀X'.replace(/(?<x>\p{{Diacritic}})/{mode}g,'<$1:$<x>>')==='<\u0301:\u0301><\u0300:\u0300>😀X'"
        ));
        assert_true(&format!(
            r"'😀\u0301\u0300G'.split(/(\p{{Dia}})/{mode}).join('|')==='😀|\u0301||\u0300|G'&&'😀\u0300'.search(/\p{{Dia}}/{mode})===2&&'\u0301\u0300😀'.match(/\p{{Dia}}/{mode}g).join('')==='\u0301\u0300'"
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0300\u0301😀\u0300'.replace(/(\p{{Dia}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_diacritic_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Diacritic})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Diacritic})$' && c.unicodeSets && !c.unicode && !c.multiline && c.exec(
                '\u0300').groups.x === '\u0300' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{Dia}/uy;
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
            let r = /\p{Dia}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u0300')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{Dia}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀\u0300')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_diacritic_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Dia}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    assert_true(r"/[\p{Dia}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{Dia}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x300),
                m = /[\p{Dia}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{Dia}+b/u.exec('AA')", r"/\p{Dia}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_diacritic_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{Dia}})+\B/d{mode}g,
                    m = r.exec('😀\u0300\u0301\u0301\uFEFF');
                m.index === 2 && m[0] === '\u0300\u0301\u0301' && m.groups.x === '\u0301' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_diacritic_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x558u32, 0x58bu32, 0x58cu32, 0x5c8u32, 0x5c9u32, 0x656u32, 0x6e2u32, 0x8d3u32, 0xb53u32,
        0xb54u32, 0x1adeu32, 0x1adfu32, 0x1aecu32, 0x1aedu32, 0x1aeeu32, 0x1aefu32, 0x1af0u32,
        0x208fu32, 0x10ecbu32, 0x10eccu32, 0x10ecdu32, 0x10eceu32, 0x10ecfu32, 0x10ef0u32,
        0x10ef1u32, 0x10ef2u32, 0x10ef3u32, 0x10ef4u32, 0x10ef5u32, 0x10ef6u32, 0x10ef7u32,
        0x10ef9u32, 0x1d250u32, 0x1d251u32, 0x1d252u32, 0x1d25bu32, 0x1d25cu32, 0x1dfcdu32,
        0x1dfceu32, 0x1dfcfu32, 0x1dfd1u32, 0x1dfd2u32, 0x1dfd3u32, 0x1dfd4u32, 0x1dfd5u32,
        0x1dfd6u32, 0x1dfd7u32, 0x1dfd8u32, 0x1dfd9u32, 0x1dfdau32, 0x1dfdbu32, 0x1dfdcu32,
        0x1dfddu32, 0x1dfdeu32, 0x1dfdfu32, 0x1dfe0u32, 0x1dfe1u32, 0x1dfe2u32, 0x1dfe3u32,
        0x1dfe4u32, 0x1dfe5u32, 0x1dfe6u32, 0x1dfe7u32, 0x1dfe8u32, 0x1dfe9u32, 0x1dfeau32,
        0x1dfebu32, 0x1dfecu32, 0x1dfedu32, 0x1dfeeu32, 0x1dfefu32, 0x1dff0u32, 0x1dff1u32,
        0x1dff2u32, 0x1dff3u32, 0x1dff4u32, 0x1dff5u32, 0x1dff6u32, 0x1dff7u32, 0x1dff8u32,
        0x1dff9u32, 0x1dffau32, 0x1dffbu32, 0x1dffcu32, 0x1dffdu32, 0x1dffeu32, 0x1dfffu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Diacritic", "Dia"] {
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
    assert_eq!(comparisons, 696);
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0x1F600..=0x1F600,
    0x005D..=0x0061,
    0x00A7..=0x00A9,
    0x00AE..=0x00B0,
    0x00B3..=0x00B9,
    0x02AF..=0x02B0,
    0x0345..=0x0345,
    0x034E..=0x0350,
    0x0357..=0x0358,
    0x035C..=0x035D,
    0x0362..=0x0363,
    0x0373..=0x0376,
    0x0379..=0x037B,
    0x0383..=0x0386,
    0x0399..=0x0399,
    0x03B9..=0x03B9,
    0x0482..=0x0483,
    0x0487..=0x0488,
    0x0557..=0x0557,
    0x0559..=0x055A,
    0x058A..=0x058A,
    0x058D..=0x058D,
    0x0590..=0x0591,
    0x05BD..=0x05C7,
    0x05CA..=0x05CA,
    0x064A..=0x064B,
    0x0652..=0x0653,
    0x0655..=0x0655,
    0x0658..=0x0659,
    0x06DE..=0x06E1,
    0x06E3..=0x06E7,
    0x06E9..=0x06EA,
    0x06EC..=0x06ED,
    0x072F..=0x0730,
    0x074A..=0x074B,
    0x07A5..=0x07A6,
    0x07B0..=0x07B1,
    0x07EA..=0x07EB,
    0x07F5..=0x07F6,
    0x0817..=0x081A,
    0x0897..=0x0898,
    0x089F..=0x08A0,
    0x08C8..=0x08C9,
    0x08D4..=0x08D4,
    0x08E2..=0x08E3,
    0x08FE..=0x08FF,
    0x093B..=0x093D,
    0x094C..=0x094E,
    0x0950..=0x0951,
    0x0954..=0x0955,
    0x0970..=0x0972,
    0x09BB..=0x09BD,
    0x09CC..=0x09CE,
    0x0A3B..=0x0A3D,
    0x0A4C..=0x0A4E,
    0x0ABB..=0x0ABD,
    0x0ACC..=0x0ACE,
    0x0AFC..=0x0AFD,
    0x0AFF..=0x0B00,
    0x0B3B..=0x0B3D,
    0x0B4C..=0x0B4E,
    0x0B52..=0x0B52,
    0x0B55..=0x0B56,
    0x0BCC..=0x0BCE,
    0x0C3B..=0x0C3D,
    0x0C4C..=0x0C4E,
    0x0CBB..=0x0CBD,
    0x0CCC..=0x0CCE,
    0x0D3A..=0x0D3D,
    0x0D4C..=0x0D4E,
    0x0DC9..=0x0DCB,
    0x0E39..=0x0E3B,
    0x0E46..=0x0E47,
    0x0E4C..=0x0E4F,
    0x0EB9..=0x0EBB,
    0x0EC7..=0x0EC8,
    0x0ECC..=0x0ECD,
    0x0F17..=0x0F1A,
    0x0F34..=0x0F3A,
    0x0F3D..=0x0F40,
    0x0F81..=0x0F82,
    0x0F84..=0x0F88,
    0x0FC5..=0x0FC7,
    0x1036..=0x103B,
    0x1062..=0x1065,
    0x1068..=0x1069,
    0x106D..=0x106E,
    0x1086..=0x1087,
    0x108D..=0x1090,
    0x1099..=0x109C,
    0x135C..=0x135D,
    0x135F..=0x1360,
    0x1713..=0x1716,
    0x1733..=0x1735,
    0x17C8..=0x17C9,
    0x17D3..=0x17D4,
    0x17DC..=0x17DE,
    0x1938..=0x1939,
    0x193B..=0x193C,
    0x1A5F..=0x1A61,
    0x1A74..=0x1A75,
    0x1A7C..=0x1A80,
    0x1AAF..=0x1AB0,
    0x1ABE..=0x1AC1,
    0x1ACB..=0x1ACC,
    0x1ACE..=0x1ACF,
    0x1AF1..=0x1AF1,
    0x1B33..=0x1B35,
    0x1B43..=0x1B45,
    0x1B6A..=0x1B6B,
    0x1B73..=0x1B74,
    0x1BA9..=0x1BAC,
    0x1BE5..=0x1BE7,
    0x1BF1..=0x1BF4,
    0x1C35..=0x1C38,
    0x1C77..=0x1C78,
    0x1C7D..=0x1C7E,
    0x1CCF..=0x1CD0,
    0x1CE8..=0x1CE9,
    0x1CEC..=0x1CEE,
    0x1CF3..=0x1CF7,
    0x1CF9..=0x1CFA,
    0x1D2B..=0x1D2C,
    0x1D6A..=0x1D6B,
    0x1D9A..=0x1D9B,
    0x1DBE..=0x1DBF,
    0x1DC3..=0x1DC4,
    0x1DCF..=0x1DD0,
    0x1DF4..=0x1DF5,
    0x1DFF..=0x1E00,
    0x1FBC..=0x1FBF,
    0x1FC1..=0x1FC2,
    0x1FCC..=0x1FCD,
    0x1FCF..=0x1FD0,
    0x1FDC..=0x1FDD,
    0x1FDF..=0x1FE0,
    0x1FEC..=0x1FED,
    0x1FEF..=0x1FF0,
    0x1FFC..=0x1FFF,
    0x208E..=0x208E,
    0x2090..=0x2090,
    0x2CEE..=0x2CEF,
    0x2CF1..=0x2CF2,
    0x2E2E..=0x2E30,
    0x3029..=0x302A,
    0x302F..=0x3030,
    0x3098..=0x3099,
    0x309C..=0x309D,
    0x30FB..=0x30FD,
    0xA66E..=0xA670,
    0xA67B..=0xA680,
    0xA69B..=0xA69E,
    0xA6EF..=0xA6F2,
    0xA6FF..=0xA700,
    0xA721..=0xA722,
    0xA787..=0xA788,
    0xA78A..=0xA78B,
    0xA7F0..=0xA7F2,
    0xA7F7..=0xA7FA,
    0xA805..=0xA807,
    0xA82B..=0xA82D,
    0xA8C3..=0xA8C5,
    0xA8DF..=0xA8E0,
    0xA8F1..=0xA8F2,
    0xA92A..=0xA92B,
    0xA92E..=0xA92F,
    0xA952..=0xA954,
    0xA9B2..=0xA9B4,
    0xA9BF..=0xA9C1,
    0xA9E4..=0xA9E6,
    0xAA7A..=0xAA7B,
    0xAA7D..=0xAA7E,
    0xAABE..=0xAABF,
    0xAAC2..=0xAAC3,
    0xAAF5..=0xAAF7,
    0xAB5A..=0xAB5B,
    0xAB5F..=0xAB60,
    0xAB68..=0xAB69,
    0xAB6B..=0xAB6C,
    0xABEB..=0xABEE,
    0xFB1D..=0xFB1F,
    0xFE1F..=0xFE20,
    0xFE2F..=0xFE30,
    0xFF3D..=0xFF41,
    0xFF6F..=0xFF71,
    0xFF9D..=0xFFA0,
    0xFFE2..=0xFFE4,
    0x102DF..=0x102E1,
    0x1077F..=0x10780,
    0x10785..=0x10787,
    0x107B0..=0x107B2,
    0x107BA..=0x107BB,
    0x10A37..=0x10A38,
    0x10A3A..=0x10A3B,
    0x10A3E..=0x10A40,
    0x10AE4..=0x10AE7,
    0x10D21..=0x10D22,
    0x10D27..=0x10D28,
    0x10D4D..=0x10D4F,
    0x10D68..=0x10D69,
    0x10D6D..=0x10D6E,
    0x10ECA..=0x10ECA,
    0x10ED0..=0x10ED0,
    0x10EEF..=0x10EEF,
    0x10EF8..=0x10EF8,
    0x10EFA..=0x10EFD,
    0x10EFF..=0x10F00,
    0x10F45..=0x10F46,
    0x10F50..=0x10F51,
    0x10F81..=0x10F82,
    0x10F85..=0x10F86,
    0x11045..=0x11047,
    0x1106F..=0x11071,
    0x110B8..=0x110BB,
    0x11132..=0x11135,
    0x11172..=0x11174,
    0x111BF..=0x111C1,
    0x111C9..=0x111CA,
    0x111CC..=0x111CD,
    0x11234..=0x11237,
    0x112E8..=0x112EB,
    0x1133A..=0x1133D,
    0x1134C..=0x1134E,
    0x11365..=0x11366,
    0x1136C..=0x1136D,
    0x1136F..=0x11370,
    0x11374..=0x11375,
    0x113CD..=0x113CE,
    0x113D0..=0x113D4,
    0x113E0..=0x113E3,
    0x11441..=0x11443,
    0x11445..=0x11447,
    0x114C1..=0x114C4,
    0x115BE..=0x115C1,
    0x1163E..=0x11640,
    0x116B5..=0x116B8,
    0x1172A..=0x1172C,
    0x11838..=0x1183B,
    0x1193C..=0x1193F,
    0x11942..=0x11944,
    0x119DF..=0x119E1,
    0x11A33..=0x11A35,
    0x11A46..=0x11A48,
    0x11A98..=0x11A9A,
    0x11C3E..=0x11C40,
    0x11D41..=0x11D46,
    0x11D96..=0x11D98,
    0x11DD8..=0x11DDA,
    0x11F40..=0x11F43,
    0x11F59..=0x11F5B,
    0x13446..=0x13447,
    0x13455..=0x13456,
    0x1612E..=0x16130,
    0x16AEF..=0x16AF0,
    0x16AF4..=0x16AF5,
    0x16B2F..=0x16B30,
    0x16B36..=0x16B37,
    0x16D6A..=0x16D6D,
    0x16F8E..=0x16F8F,
    0x16F9F..=0x16FA0,
    0x16FEF..=0x16FF2,
    0x1AFEF..=0x1AFF0,
    0x1AFF3..=0x1AFF5,
    0x1AFFB..=0x1AFFF,
    0x1CEFF..=0x1CF00,
    0x1CF2D..=0x1CF30,
    0x1CF46..=0x1CF47,
    0x1D166..=0x1D167,
    0x1D169..=0x1D16A,
    0x1D16C..=0x1D16D,
    0x1D172..=0x1D173,
    0x1D17A..=0x1D17B,
    0x1D182..=0x1D185,
    0x1D18B..=0x1D18C,
    0x1D1A9..=0x1D1AA,
    0x1D1AD..=0x1D1AE,
    0x1D24F..=0x1D24F,
    0x1D253..=0x1D253,
    0x1D25A..=0x1D25A,
    0x1D25D..=0x1D25D,
    0x1DFCC..=0x1DFCC,
    0x1DFD0..=0x1DFD0,
    0x1E000..=0x1E000,
    0x1E02F..=0x1E030,
    0x1E06D..=0x1E06E,
    0x1E12F..=0x1E130,
    0x1E136..=0x1E137,
    0x1E2AD..=0x1E2AF,
    0x1E2EB..=0x1E2EC,
    0x1E2EF..=0x1E2F0,
    0x1E5ED..=0x1E5F0,
    0x1E8CF..=0x1E8D0,
    0x1E8D6..=0x1E8D7,
    0x1E943..=0x1E944,
    0x1E946..=0x1E948,
    0x1E94A..=0x1E94B,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
