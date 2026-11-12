//! Cased and its exact Cased alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_cased_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Cased}",
        r"\P{Cased}",
        r"[^\p{Cased}]",
        r"[\p{Cased}K]",
        r"[\P{Cased}K]",
        r"^(?<x>[\p{Cased}K])$",
        r"(?<x>\P{Cased})",
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
fn unicode_cased_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Cased}}/{mode},
                    b = /\p{{Cased}}/i{mode},
                    n = /\P{{Cased}}/i{mode};
                a.test('\u0041') && a.test('\u0042') && b.test('\u0041') && !a.test('\u200D') && !b.test(
                    '\u005f') && b.test('ſ') && b.test('K') && !n.test('\u0041') && n.test(
                    '\uDB40\uDC01') && n.test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Cased}}/{mode},
                    s = /\s/{mode};
                p.test('\u0041') && p.test('\u0042') && !s.test('\u0041') && !s.test('\u0042') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{Cased}}K]/i{mode},
                    b = /[^\P{{Cased}}K]/i{mode},
                    c = /[\p{{Cased}}\P{{Cased}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0041') && a.test('ſ') && b.test('\u0042') && !b.test(
                    'K') && c.test('\uDB40\uDC01') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{Cased}}$/{mode},
                    b = /^\P{{Cased}}$/{mode};
                a.test('\u0041') && !a.test('\u0042\n') && !a.test('') && !a.test('\n') && b.test(
                    '\uDB40\uDC01') && !b.test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_cased_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{Cased}}K]))$/dim{mode}g,
                    m = r.exec('\u200D\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{Cased}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u0041');
                m.index === 0 && m[1] === '\uDB40\uDC01' && m.indices[1][1] === 2 && r.lastIndex === 2 &&
                    Object.getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Cased}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u0041');
                m.index === 2 && m[0] === '\u0041' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Cased}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u0041')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_cased_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u0042\uDB40\uDC01\u0041\u200D'.matchAll(/(?<x>\P{{Cased}})/d{mode}g)];
                a.length === 2 && a[0][1] === '\uDB40\uDC01' && a[0].index === 1 && a[0].indices[1][1] === 3 &&
                    a[1][1] === '\u200D'
            "#
        ));
        assert_true(&format!(
            r"'\u0042\u0041\uDB40\uDC01\u200D'.replace(/(?<x>\p{{Cased}})/{mode}g,'<$1:$<x>>')==='<\u0042:\u0042><\u0041:\u0041>\uDB40\uDC01\u200D'"
        ));
        assert_true(&format!(
            r#"
                '\uDB40\uDC01\u0042\u0041\u200D'.split(/(\p{{Cased}})/{mode}).join('|') ===
                    '\uDB40\uDC01|\u0042||\u0041|\u200D' && '\uDB40\uDC01\u0041'.search(
                        /\p{{Cased}}/{mode}) === 2 && '\u0042\u0041\uDB40\uDC01'.match(
                        /\p{{Cased}}/{mode}g).join('') === '\u0042\u0041'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0041\u0042\uDB40\uDC01\u0041'.replace(/(\p{{Cased}})/{mode}g, (m, c,
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
fn unicode_cased_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Cased})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Cased})$' && c.unicodeSets && !c.unicode && !c.multiline && c.exec(
                '\u0041').groups.x === '\u0041' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{Cased}/uy;
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
            let r = /\p{Cased}/vy;
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
            let r = /\p{Cased}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\uDB40\uDC01\u0041')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_cased_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Cased}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
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
    assert_true(
        r"let m=/[\p{Cased}&&A]/v.exec('A');m!==null&&m.length===1&&m[0]==='A'&&m.index===0&&m.input==='A'",
    );
    assert_true(r"/[\p{Cased}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x42),
                m = /[\p{Cased}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{Cased}+b/u.exec('AA')", r"/\p{Cased}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_cased_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\b(?<x>\p{{Cased}})+\b/d{mode}g,
                    m = r.exec('\uDB40\uDC01\u0041\u0042\u0042\uFEFF');
                m.index === 2 && m[0] === '\u0041\u0042\u0042' && m.groups.x === '\u0042' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_cased_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x558u32, 0x58bu32, 0x58cu32, 0x209du32, 0x209eu32, 0x209fu32, 0xa7ddu32, 0xa7e2u32,
        0xab6cu32, 0xab6du32, 0x107bbu32, 0x107bcu32, 0x107bdu32, 0x107beu32, 0x107bfu32,
        0x1d6a6u32, 0x1df1fu32, 0x1df20u32, 0x1df21u32, 0x1df22u32, 0x1df23u32, 0x1df24u32,
        0x1df2bu32, 0x1df2cu32, 0x1df2du32, 0x1df2eu32, 0x1df2fu32, 0x1df30u32, 0x1df31u32,
        0x1df32u32, 0x1df33u32, 0x1df34u32, 0x1df35u32, 0x1df36u32, 0x1df37u32, 0x1df38u32,
        0x1df39u32, 0x1df3au32, 0x1df3bu32, 0x1df3cu32, 0x1df3du32, 0x1df3eu32, 0x1df3fu32,
        0x1df40u32, 0x1df41u32, 0x1df42u32, 0x1df43u32, 0x1df44u32, 0x1df45u32, 0x1df46u32,
        0x1df47u32, 0x1df48u32, 0x1df49u32, 0x1df4au32, 0x1df4bu32, 0x1df4cu32, 0x1df4du32,
        0x1df4eu32, 0x1df4fu32, 0x1df50u32, 0x1df51u32, 0x1df52u32, 0x1df53u32, 0x1df54u32,
        0x1df55u32, 0x1df56u32, 0x1df57u32, 0x1df58u32, 0x1df59u32, 0x1df5au32, 0x1df5bu32,
        0x1df5cu32, 0x1df5du32, 0x1df5eu32, 0x1df5fu32, 0x1df60u32, 0x1df61u32, 0x1df62u32,
        0x1df63u32, 0x1df64u32, 0x1df65u32, 0x1df66u32, 0x1df67u32, 0x1df68u32, 0x1df69u32,
        0x1df6au32, 0x1df6bu32, 0x1df6cu32, 0x1df6du32, 0x1df6eu32, 0x1df6fu32, 0x1df70u32,
        0x1df71u32, 0x1df72u32, 0x1df73u32, 0x1df74u32, 0x1df75u32, 0x1df76u32, 0x1df77u32,
        0x1df78u32, 0x1df79u32, 0x1df7au32, 0x1df7bu32, 0x1df7cu32, 0x1df7du32, 0x1df7eu32,
        0x1df7fu32, 0x1df90u32, 0x1df91u32, 0x1df92u32, 0x1df93u32, 0x1df94u32, 0x1df95u32,
        0x1df96u32, 0x1dfcdu32, 0x1dfceu32, 0x1dfcfu32, 0x1dfd0u32, 0x1dfd1u32, 0x1dfd2u32,
        0x1dfd3u32, 0x1dfd4u32, 0x1dfd5u32, 0x1dfd6u32, 0x1dfd7u32, 0x1dfd8u32, 0x1dfd9u32,
        0x1dfdau32, 0x1dfdbu32, 0x1dfdcu32, 0x1dfddu32, 0x1dfdeu32, 0x1dfdfu32, 0x1dfe0u32,
        0x1dfe1u32, 0x1dfe2u32, 0x1dfe3u32, 0x1dfe4u32, 0x1dfe5u32, 0x1dfe6u32, 0x1dfe7u32,
        0x1dfe8u32, 0x1dfe9u32, 0x1dfeau32, 0x1dfebu32, 0x1dfecu32, 0x1dfedu32, 0x1dfeeu32,
        0x1dfefu32, 0x1dff0u32, 0x1dff1u32, 0x1dff2u32, 0x1dff3u32, 0x1dff4u32, 0x1dff5u32,
        0x1dff6u32, 0x1dff7u32, 0x1dff8u32, 0x1dff9u32, 0x1dffau32, 0x1dffbu32, 0x1dffcu32,
        0x1dffdu32, 0x1dffeu32, 0x1dfffu32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for flags in ["u", "ui", "v", "vi"] {
            assert_true(&format!(
                r#"
                    let p = new RegExp('^\\p{{Cased}}$', '{flags}'),
                        n = new RegExp('^\\P{{Cased}}$', '{flags}');
                    p.test({text:?}) && !n.test({text:?})
                "#
            ));
            comparisons += 1;
        }
    }
    assert_eq!(comparisons, 660);
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0xE0001..=0xE0001,
    0x0040..=0x0041,
    0x005A..=0x005B,
    0x0060..=0x0061,
    0x007A..=0x007B,
    0x00A9..=0x00AB,
    0x00B4..=0x00B6,
    0x00B9..=0x00BB,
    0x00BF..=0x00C0,
    0x00D6..=0x00D8,
    0x00F6..=0x00F8,
    0x01BA..=0x01BC,
    0x01BF..=0x01C0,
    0x01C3..=0x01C4,
    0x0293..=0x0296,
    0x02B8..=0x02B9,
    0x02BF..=0x02C2,
    0x02DF..=0x02E0,
    0x02E4..=0x02E5,
    0x0344..=0x0346,
    0x036F..=0x0370,
    0x0373..=0x037A,
    0x037D..=0x0380,
    0x0385..=0x0388,
    0x038A..=0x038E,
    0x03A1..=0x03A3,
    0x03F5..=0x03F7,
    0x0481..=0x0482,
    0x0489..=0x048A,
    0x052F..=0x0531,
    0x0556..=0x0557,
    0x0559..=0x0559,
    0x055F..=0x0560,
    0x0588..=0x058A,
    0x058D..=0x058D,
    0x109F..=0x10A0,
    0x10C5..=0x10C8,
    0x10CC..=0x10D0,
    0x10FA..=0x10FC,
    0x10FF..=0x1100,
    0x139F..=0x13A0,
    0x13F5..=0x13F8,
    0x13FD..=0x13FE,
    0x1C7F..=0x1C80,
    0x1C8A..=0x1C8B,
    0x1C8F..=0x1C90,
    0x1CBA..=0x1CBD,
    0x1CBF..=0x1CC0,
    0x1CFF..=0x1D00,
    0x1DBF..=0x1DC0,
    0x1DFF..=0x1E00,
    0x1F15..=0x1F18,
    0x1F1D..=0x1F20,
    0x1F45..=0x1F48,
    0x1F4D..=0x1F50,
    0x1F57..=0x1F5F,
    0x1F7D..=0x1F80,
    0x1FB4..=0x1FB6,
    0x1FBC..=0x1FBF,
    0x1FC1..=0x1FC2,
    0x1FC4..=0x1FC6,
    0x1FCC..=0x1FCD,
    0x1FCF..=0x1FD0,
    0x1FD3..=0x1FD6,
    0x1FDB..=0x1FDC,
    0x1FDF..=0x1FE0,
    0x1FEC..=0x1FED,
    0x1FF1..=0x1FF2,
    0x1FF4..=0x1FF6,
    0x1FFC..=0x1FFD,
    0x2070..=0x2072,
    0x207E..=0x2080,
    0x208F..=0x2090,
    0x20A0..=0x20A0,
    0x2101..=0x2103,
    0x2106..=0x210A,
    0x2113..=0x2116,
    0x2118..=0x2119,
    0x211D..=0x211E,
    0x2123..=0x212A,
    0x212D..=0x212F,
    0x2134..=0x2135,
    0x2138..=0x213C,
    0x213F..=0x2140,
    0x2144..=0x2145,
    0x2149..=0x214A,
    0x214D..=0x214F,
    0x215F..=0x2160,
    0x217F..=0x2180,
    0x2182..=0x2185,
    0x24B5..=0x24B6,
    0x24E9..=0x24EA,
    0x2BFF..=0x2C00,
    0x2CE4..=0x2CE5,
    0x2CEA..=0x2CEB,
    0x2CEE..=0x2CEF,
    0x2CF1..=0x2CF4,
    0x2CFF..=0x2D00,
    0x2D25..=0x2D28,
    0x2D2C..=0x2D2E,
    0xA63F..=0xA640,
    0xA66D..=0xA66E,
    0xA67F..=0xA680,
    0xA69D..=0xA69E,
    0xA721..=0xA722,
    0xA787..=0xA788,
    0xA78A..=0xA78B,
    0xA78E..=0xA790,
    0xA7DE..=0xA7DE,
    0xA7E1..=0xA7E1,
    0xA7E3..=0xA7E3,
    0xA7F0..=0xA7F1,
    0xA7F6..=0xA7F8,
    0xA7FA..=0xA7FB,
    0xAB2F..=0xAB30,
    0xAB5A..=0xAB5C,
    0xAB69..=0xAB6B,
    0xAB6E..=0xAB70,
    0xABBF..=0xABC0,
    0xFAFF..=0xFB00,
    0xFB06..=0xFB07,
    0xFB12..=0xFB13,
    0xFB17..=0xFB18,
    0xFF20..=0xFF21,
    0xFF3A..=0xFF3B,
    0xFF40..=0xFF41,
    0xFF5A..=0xFF5B,
    0x103FF..=0x10400,
    0x1044F..=0x10450,
    0x104AF..=0x104B0,
    0x104D3..=0x104D4,
    0x104D7..=0x104D8,
    0x104FB..=0x104FC,
    0x1056F..=0x10570,
    0x1057A..=0x1057C,
    0x1058A..=0x1058C,
    0x10592..=0x10597,
    0x105A1..=0x105A3,
    0x105B1..=0x105B3,
    0x105B9..=0x105BD,
    0x1077F..=0x10783,
    0x10785..=0x10787,
    0x107B0..=0x107B2,
    0x107C0..=0x107C0,
    0x10C7F..=0x10C80,
    0x10CB2..=0x10CB3,
    0x10CBF..=0x10CC0,
    0x10CF2..=0x10CF3,
    0x10D4F..=0x10D50,
    0x10D65..=0x10D66,
    0x10D6F..=0x10D70,
    0x10D85..=0x10D86,
    0x1189F..=0x118A0,
    0x118DF..=0x118E0,
    0x16E3F..=0x16E40,
    0x16E7F..=0x16E80,
    0x16E9F..=0x16EA0,
    0x16EB8..=0x16EBB,
    0x16ED3..=0x16ED4,
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
    0x1D6C0..=0x1D6C2,
    0x1D6DA..=0x1D6DC,
    0x1D6FA..=0x1D6FC,
    0x1D714..=0x1D716,
    0x1D734..=0x1D736,
    0x1D74E..=0x1D750,
    0x1D76E..=0x1D770,
    0x1D788..=0x1D78A,
    0x1D7A8..=0x1D7AA,
    0x1D7C2..=0x1D7C4,
    0x1D7CB..=0x1D7CC,
    0x1DEFF..=0x1DF00,
    0x1DF09..=0x1DF0B,
    0x1DF80..=0x1DF80,
    0x1DF8F..=0x1DF8F,
    0x1DF97..=0x1DF97,
    0x1DFCC..=0x1DFCC,
    0x1E000..=0x1E000,
    0x1E02F..=0x1E030,
    0x1E06D..=0x1E06E,
    0x1E8FF..=0x1E900,
    0x1E943..=0x1E944,
    0x1F12F..=0x1F130,
    0x1F149..=0x1F14A,
    0x1F14F..=0x1F150,
    0x1F169..=0x1F16A,
    0x1F16F..=0x1F170,
    0x1F189..=0x1F18A,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
