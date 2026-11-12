//! Grapheme_Extend and its exact Gr_Ext alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_grapheme_extend_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Grapheme_Extend}",
        r"\P{Gr_Ext}",
        r"[^\p{Gr_Ext}]",
        r"[\p{Gr_Ext}K]",
        r"[\P{Grapheme_Extend}K]",
        r"^(?<x>[\p{Gr_Ext}K])$",
        r"(?<x>\P{Grapheme_Extend})",
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
fn unicode_grapheme_extend_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let p = /^\p{{Gr_Ext}}$/i{mode},
                    n = /^\P{{Gr_Ext}}$/i{mode};
                '\u0345\u0399\u03B9\u1FBE'.split('').every(c => p.test(c) && n.test(c) === ('{mode}' ===
                    'u'))
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /[\p{{Gr_Ext}}K]/i{mode},
                    n = /[\P{{Gr_Ext}}K]/i{mode},
                    x = /[^\P{{Gr_Ext}}K]/i{mode};
                p.test('K') && n.test('K') && !x.test('K') && '\u0345\u0399\u03B9\u1FBE'.split('').every(c => p
                    .test(c) && n.test(c) === ('{mode}' === 'u') && x.test(c) === (
                        '{mode}' === 'v'))
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Grapheme_Extend}}/{mode},
                    b = /\p{{Gr_Ext}}/i{mode},
                    n = /\P{{Gr_Ext}}/i{mode};
                a.test('\u0300') && a.test('\u0301') && b.test('\u0300') && !a.test('\u200D') && !b.test(
                        '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u0300') && n.test('😀') && n
                    .test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Grapheme_Extend}}/{mode},
                    s = /\s/{mode};
                p.test('\u0300') && p.test('\u0301') && !s.test('\u0300') && !s.test('\u0301') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{Gr_Ext}}K]/i{mode},
                    b = /[^\P{{Grapheme_Extend}}K]/i{mode},
                    c = /[\p{{Gr_Ext}}\P{{Grapheme_Extend}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0300') && !a.test('ſ') && b.test('\u0301') && !b.test(
                    'K') && c.test('😀') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{Gr_Ext}}$/{mode},
                    b = /^\P{{Gr_Ext}}$/{mode};
                a.test('\u0300') && !a.test('\u0301\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_grapheme_extend_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{Gr_Ext}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uD834\uDD65');
                m.index === 0 && m[0] === '\uD834\uDD65' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{Gr_Ext}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{Gr_Ext}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\u0300');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Gr_Ext}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\u0300');
                m.index === 2 && m[0] === '\u0300' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Gr_Ext}}/{mode}y;r.lastIndex=1;r.exec('😀\u0300')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_grapheme_extend_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u0301😀\u0300G'.matchAll(/(?<x>\P{{Gr_Ext}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\u0301\u0300😀X'.replace(/(?<x>\p{{Grapheme_Extend}})/{mode}g,'<$1:$<x>>')==='<\u0301:\u0301><\u0300:\u0300>😀X'"
        ));
        assert_true(&format!(
            r"'😀\u0301\u0300G'.split(/(\p{{Gr_Ext}})/{mode}).join('|')==='😀|\u0301||\u0300|G'&&'😀\u0300'.search(/\p{{Gr_Ext}}/{mode})===2&&'\u0301\u0300😀'.match(/\p{{Gr_Ext}}/{mode}g).join('')==='\u0301\u0300'"
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0300\u0301😀\u0300'.replace(/(\p{{Gr_Ext}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_grapheme_extend_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Grapheme_Extend})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Grapheme_Extend})$' && c.unicodeSets && !c.unicode && !c.multiline && c
                .exec('\u0300').groups.x === '\u0300' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{Gr_Ext}/uy;
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
            let r = /\p{Gr_Ext}/vy;
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
            let r = /\p{Gr_Ext}/vy;
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
fn unicode_grapheme_extend_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Gr_Ext}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
        r#"let m=/\p{Script_Extensions=Latin}/u.exec('A');JSON.stringify(m)==='["A"]'&&m.index===0&&m.input==='A'"#,
    );
    assert_true(r"/[\p{Gr_Ext}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{Gr_Ext}--A]/v.exec('A')===null");
    assert_true(
        r"let s=String.fromCodePoint(0x300),m=/[\p{Gr_Ext}--A]/v.exec(s);m!==null&&m.length===1&&m[0]===s&&m.index===0&&m.input===s",
    );
    for source in [r"/\p{Gr_Ext}+b/u.exec('AA')", r"/\p{Gr_Ext}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_grapheme_extend_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{Gr_Ext}})+\B/d{mode}g,
                    m = r.exec('😀\u0300\u0301\u0301\uFEFF');
                m.index === 2 && m[0] === '\u0300\u0301\u0301' && m.groups.x === '\u0301' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_grapheme_extend_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [
        0x5c8u32, 0x5c9u32, 0xb53u32, 0xb54u32, 0x1adeu32, 0x1adfu32, 0x1aecu32, 0x1aedu32,
        0x1aeeu32, 0x1aefu32, 0x1af0u32, 0x10ecbu32, 0x10eccu32, 0x10ecdu32, 0x10eceu32,
        0x10ecfu32, 0x10ef0u32, 0x10ef1u32, 0x10ef2u32, 0x10ef3u32, 0x10ef4u32, 0x10ef5u32,
        0x10ef6u32, 0x10ef7u32, 0x10ef8u32, 0x10ef9u32, 0x11df0u32, 0x1d127u32, 0x1d128u32,
        0x1d250u32, 0x1d251u32, 0x1d252u32, 0x1d25bu32, 0x1d25cu32, 0x1d25fu32, 0x1d280u32,
        0x1d281u32,
    ] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Grapheme_Extend", "Gr_Ext"] {
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
    assert_eq!(comparisons, 296);
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0x1F600..=0x1F600,
    0x02FF..=0x0300,
    0x0345..=0x0345,
    0x036F..=0x0370,
    0x0399..=0x0399,
    0x03B9..=0x03B9,
    0x0482..=0x0483,
    0x0489..=0x048A,
    0x0590..=0x0591,
    0x05BD..=0x05C7,
    0x05CA..=0x05CA,
    0x060F..=0x0610,
    0x061A..=0x061B,
    0x064A..=0x064B,
    0x065F..=0x0660,
    0x066F..=0x0671,
    0x06D5..=0x06D6,
    0x06DC..=0x06DF,
    0x06E4..=0x06EA,
    0x06ED..=0x06EE,
    0x0710..=0x0712,
    0x072F..=0x0730,
    0x074A..=0x074B,
    0x07A5..=0x07A6,
    0x07B0..=0x07B1,
    0x07EA..=0x07EB,
    0x07F3..=0x07F4,
    0x07FC..=0x07FE,
    0x0815..=0x0816,
    0x0819..=0x081B,
    0x0823..=0x0825,
    0x0827..=0x0829,
    0x082D..=0x082E,
    0x0858..=0x0859,
    0x085B..=0x085C,
    0x0896..=0x0897,
    0x089F..=0x08A0,
    0x08C9..=0x08CA,
    0x08E1..=0x08E3,
    0x0902..=0x0903,
    0x0939..=0x093D,
    0x0940..=0x0941,
    0x0948..=0x0949,
    0x094C..=0x094E,
    0x0950..=0x0951,
    0x0957..=0x0958,
    0x0961..=0x0964,
    0x0980..=0x0982,
    0x09BB..=0x09C1,
    0x09C4..=0x09C5,
    0x09CC..=0x09CE,
    0x09D6..=0x09D8,
    0x09E1..=0x09E4,
    0x09FD..=0x0A03,
    0x0A3B..=0x0A3D,
    0x0A40..=0x0A43,
    0x0A46..=0x0A4B,
    0x0A4D..=0x0A4E,
    0x0A50..=0x0A52,
    0x0A6F..=0x0A72,
    0x0A74..=0x0A76,
    0x0A80..=0x0A83,
    0x0ABB..=0x0ABD,
    0x0AC0..=0x0AC1,
    0x0AC5..=0x0AC9,
    0x0ACC..=0x0ACE,
    0x0AE1..=0x0AE4,
    0x0AF9..=0x0AFA,
    0x0AFF..=0x0B02,
    0x0B3B..=0x0B41,
    0x0B44..=0x0B45,
    0x0B4C..=0x0B4E,
    0x0B52..=0x0B52,
    0x0B57..=0x0B58,
    0x0B61..=0x0B64,
    0x0B81..=0x0B83,
    0x0BBD..=0x0BC1,
    0x0BCC..=0x0BCE,
    0x0BD6..=0x0BD8,
    0x0BFF..=0x0C01,
    0x0C03..=0x0C05,
    0x0C3B..=0x0C3E,
    0x0C40..=0x0C41,
    0x0C45..=0x0C46,
    0x0C48..=0x0C4A,
    0x0C4D..=0x0C4E,
    0x0C54..=0x0C57,
    0x0C61..=0x0C64,
    0x0C80..=0x0C82,
    0x0CBB..=0x0CC3,
    0x0CC5..=0x0CC6,
    0x0CC8..=0x0CCA,
    0x0CCD..=0x0CCE,
    0x0CD4..=0x0CD7,
    0x0CE1..=0x0CE4,
    0x0CFF..=0x0D02,
    0x0D3A..=0x0D41,
    0x0D44..=0x0D45,
    0x0D4C..=0x0D4E,
    0x0D56..=0x0D58,
    0x0D61..=0x0D64,
    0x0D80..=0x0D82,
    0x0DC9..=0x0DCB,
    0x0DCE..=0x0DD2,
    0x0DD4..=0x0DD7,
    0x0DDE..=0x0DE0,
    0x0E30..=0x0E34,
    0x0E3A..=0x0E3B,
    0x0E46..=0x0E47,
    0x0E4E..=0x0E4F,
    0x0EB0..=0x0EB4,
    0x0EBC..=0x0EBD,
    0x0EC7..=0x0EC8,
    0x0ECE..=0x0ECF,
    0x0F17..=0x0F1A,
    0x0F34..=0x0F3A,
    0x0F70..=0x0F71,
    0x0F7E..=0x0F80,
    0x0F84..=0x0F88,
    0x0F8C..=0x0F8D,
    0x0F97..=0x0F99,
    0x0FBC..=0x0FBD,
    0x0FC5..=0x0FC7,
    0x102C..=0x102D,
    0x1030..=0x1032,
    0x1037..=0x103F,
    0x1057..=0x105A,
    0x105D..=0x105E,
    0x1060..=0x1061,
    0x1070..=0x1071,
    0x1074..=0x1075,
    0x1081..=0x1087,
    0x108C..=0x108E,
    0x109C..=0x109E,
    0x135C..=0x135D,
    0x135F..=0x1360,
    0x1711..=0x1712,
    0x1715..=0x1716,
    0x1731..=0x1732,
    0x1734..=0x1735,
    0x1751..=0x1754,
    0x1771..=0x1774,
    0x17B3..=0x17B7,
    0x17BD..=0x17BE,
    0x17C5..=0x17C9,
    0x17D3..=0x17D4,
    0x17DC..=0x17DE,
    0x180A..=0x180B,
    0x180D..=0x1810,
    0x1884..=0x1887,
    0x18A8..=0x18AA,
    0x191F..=0x1920,
    0x1922..=0x1923,
    0x1926..=0x1929,
    0x1931..=0x1933,
    0x1938..=0x1939,
    0x193B..=0x193C,
    0x1A16..=0x1A1C,
    0x1A55..=0x1A58,
    0x1A5E..=0x1A65,
    0x1A6C..=0x1A6D,
    0x1A72..=0x1A73,
    0x1A7C..=0x1A80,
    0x1AAF..=0x1AB0,
    0x1AF1..=0x1AF1,
    0x1AFF..=0x1B00,
    0x1B03..=0x1B04,
    0x1B33..=0x1B34,
    0x1B3D..=0x1B3E,
    0x1B41..=0x1B42,
    0x1B44..=0x1B45,
    0x1B6A..=0x1B6B,
    0x1B73..=0x1B74,
    0x1B7F..=0x1B82,
    0x1BA1..=0x1BA2,
    0x1BA5..=0x1BA8,
    0x1BAD..=0x1BAE,
    0x1BE5..=0x1BEA,
    0x1BEC..=0x1BEF,
    0x1BF3..=0x1BF4,
    0x1C2B..=0x1C2C,
    0x1C33..=0x1C38,
    0x1CCF..=0x1CD0,
    0x1CD2..=0x1CD4,
    0x1CE0..=0x1CE2,
    0x1CE8..=0x1CE9,
    0x1CEC..=0x1CEE,
    0x1CF3..=0x1CF5,
    0x1CF7..=0x1CFA,
    0x1DBF..=0x1DC0,
    0x1DFF..=0x1E00,
    0x1FBE..=0x1FBE,
    0x200B..=0x200D,
    0x20CF..=0x20D0,
    0x20F0..=0x20F1,
    0x2CEE..=0x2CEF,
    0x2CF1..=0x2CF2,
    0x2D7E..=0x2D80,
    0x2DDF..=0x2DE0,
    0x2DFF..=0x2E00,
    0x3029..=0x302A,
    0x302F..=0x3030,
    0x3098..=0x309B,
    0xA66E..=0xA66F,
    0xA672..=0xA674,
    0xA67D..=0xA67E,
    0xA69D..=0xA6A0,
    0xA6EF..=0xA6F2,
    0xA801..=0xA803,
    0xA805..=0xA807,
    0xA80A..=0xA80C,
    0xA824..=0xA827,
    0xA82B..=0xA82D,
    0xA8C3..=0xA8C6,
    0xA8DF..=0xA8E0,
    0xA8F1..=0xA8F2,
    0xA8FE..=0xA900,
    0xA925..=0xA926,
    0xA92D..=0xA92E,
    0xA946..=0xA947,
    0xA951..=0xA954,
    0xA97F..=0xA980,
    0xA982..=0xA983,
    0xA9B2..=0xA9B6,
    0xA9B9..=0xA9C1,
    0xA9E4..=0xA9E6,
    0xAA28..=0xAA29,
    0xAA2E..=0xAA37,
    0xAA42..=0xAA44,
    0xAA4B..=0xAA4D,
    0xAA7B..=0xAA7D,
    0xAAAF..=0xAAB2,
    0xAAB4..=0xAAB9,
    0xAABD..=0xAAC2,
    0xAAEB..=0xAAEE,
    0xAAF5..=0xAAF7,
    0xABE4..=0xABE9,
    0xABEC..=0xABEE,
    0xFB1D..=0xFB1F,
    0xFDFF..=0xFE00,
    0xFE0F..=0xFE10,
    0xFE1F..=0xFE20,
    0xFE2F..=0xFE30,
    0xFF9D..=0xFFA0,
    0x101FC..=0x101FE,
    0x102DF..=0x102E1,
    0x10375..=0x10376,
    0x1037A..=0x1037B,
    0x10A00..=0x10A01,
    0x10A03..=0x10A07,
    0x10A0B..=0x10A0C,
    0x10A0F..=0x10A10,
    0x10A37..=0x10A38,
    0x10A3A..=0x10A3B,
    0x10A3E..=0x10A40,
    0x10AE4..=0x10AE7,
    0x10D23..=0x10D24,
    0x10D27..=0x10D28,
    0x10D68..=0x10D69,
    0x10D6D..=0x10D6E,
    0x10EAA..=0x10EAD,
    0x10ECA..=0x10ECA,
    0x10ED0..=0x10ED0,
    0x10EEF..=0x10EEF,
    0x10EFF..=0x10F00,
    0x10F45..=0x10F46,
    0x10F50..=0x10F51,
    0x10F81..=0x10F82,
    0x10F85..=0x10F86,
    0x11000..=0x11002,
    0x11037..=0x11038,
    0x11046..=0x11047,
    0x1106F..=0x11075,
    0x1107E..=0x1107F,
    0x11081..=0x11082,
    0x110B2..=0x110B3,
    0x110B6..=0x110BB,
    0x110C1..=0x110C3,
    0x110FF..=0x11100,
    0x11102..=0x11103,
    0x11126..=0x11127,
    0x1112B..=0x1112D,
    0x11134..=0x11135,
    0x11172..=0x11174,
    0x1117F..=0x11182,
    0x111B5..=0x111B6,
    0x111BE..=0x111C1,
    0x111C8..=0x111C9,
    0x111CC..=0x111D0,
    0x1122E..=0x1122F,
    0x11231..=0x11234,
    0x11237..=0x11238,
    0x1123D..=0x11242,
    0x112DE..=0x112E0,
    0x112E2..=0x112E3,
    0x112EA..=0x112EB,
    0x112FF..=0x11302,
    0x1133A..=0x11341,
    0x1134C..=0x1134E,
    0x11356..=0x11358,
    0x11365..=0x11366,
    0x1136C..=0x1136D,
    0x1136F..=0x11370,
    0x11374..=0x11375,
    0x113B7..=0x113BB,
    0x113C0..=0x113C7,
    0x113C9..=0x113CA,
    0x113CD..=0x113CE,
    0x113D0..=0x113D3,
    0x113E0..=0x113E3,
    0x11437..=0x11438,
    0x1143F..=0x11442,
    0x11444..=0x11447,
    0x1145D..=0x1145F,
    0x114AF..=0x114B3,
    0x114B8..=0x114C4,
    0x115AE..=0x115B2,
    0x115B5..=0x115B6,
    0x115BB..=0x115C1,
    0x115DB..=0x115DE,
    0x11632..=0x11633,
    0x1163A..=0x11641,
    0x116AA..=0x116B0,
    0x116B7..=0x116B8,
    0x1171C..=0x11722,
    0x11725..=0x11727,
    0x1172B..=0x1172C,
    0x1182E..=0x1182F,
    0x11837..=0x1183B,
    0x1192F..=0x11931,
    0x1193A..=0x1193B,
    0x1193E..=0x1193F,
    0x11942..=0x11944,
    0x119D3..=0x119D4,
    0x119D7..=0x119DC,
    0x119DF..=0x119E1,
    0x11A00..=0x11A01,
    0x11A0A..=0x11A0B,
    0x11A32..=0x11A33,
    0x11A38..=0x11A3B,
    0x11A3E..=0x11A3F,
    0x11A46..=0x11A48,
    0x11A50..=0x11A51,
    0x11A56..=0x11A59,
    0x11A5B..=0x11A5C,
    0x11A89..=0x11A8A,
    0x11A96..=0x11A9A,
    0x11B5F..=0x11B62,
    0x11B64..=0x11B67,
    0x11C2F..=0x11C30,
    0x11C36..=0x11C38,
    0x11C3D..=0x11C40,
    0x11C91..=0x11C92,
    0x11CA7..=0x11CAA,
    0x11CB0..=0x11CB7,
    0x11D30..=0x11D31,
    0x11D36..=0x11D37,
    0x11D39..=0x11D3F,
    0x11D45..=0x11D48,
    0x11D8F..=0x11D92,
    0x11D94..=0x11D98,
    0x11DEF..=0x11DEF,
    0x11DF1..=0x11DF1,
    0x11EF2..=0x11EF5,
    0x11EFF..=0x11F02,
    0x11F35..=0x11F36,
    0x11F3A..=0x11F3B,
    0x11F3F..=0x11F40,
    0x11F42..=0x11F43,
    0x11F59..=0x11F5B,
    0x1343F..=0x13441,
    0x13446..=0x13447,
    0x13455..=0x13456,
    0x1611D..=0x1611E,
    0x16129..=0x1612A,
    0x1612C..=0x1612D,
    0x1612F..=0x16130,
    0x16AEF..=0x16AF0,
    0x16AF4..=0x16AF5,
    0x16B2F..=0x16B30,
    0x16B36..=0x16B37,
    0x16F4E..=0x16F50,
    0x16F8E..=0x16F8F,
    0x16F92..=0x16F93,
    0x16FE3..=0x16FE5,
    0x16FEF..=0x16FF2,
    0x1BC9C..=0x1BC9F,
    0x1CEFF..=0x1CF00,
    0x1CF2D..=0x1CF30,
    0x1CF46..=0x1CF47,
    0x1D126..=0x1D126,
    0x1D129..=0x1D129,
    0x1D164..=0x1D165,
    0x1D169..=0x1D16A,
    0x1D16C..=0x1D16D,
    0x1D172..=0x1D173,
    0x1D17A..=0x1D17B,
    0x1D182..=0x1D185,
    0x1D18B..=0x1D18C,
    0x1D1A9..=0x1D1AA,
    0x1D1AD..=0x1D1AE,
    0x1D241..=0x1D242,
    0x1D244..=0x1D245,
    0x1D24F..=0x1D24F,
    0x1D253..=0x1D253,
    0x1D25A..=0x1D25A,
    0x1D25D..=0x1D25E,
    0x1D260..=0x1D260,
    0x1D27F..=0x1D27F,
    0x1D282..=0x1D282,
    0x1D9FF..=0x1DA00,
    0x1DA36..=0x1DA37,
    0x1DA3A..=0x1DA3B,
    0x1DA6C..=0x1DA6D,
    0x1DA74..=0x1DA76,
    0x1DA83..=0x1DA85,
    0x1DA9A..=0x1DA9B,
    0x1DA9F..=0x1DAA1,
    0x1DAAF..=0x1DAB0,
    0x1DFFF..=0x1E000,
    0x1E006..=0x1E008,
    0x1E018..=0x1E01B,
    0x1E021..=0x1E026,
    0x1E02A..=0x1E02B,
    0x1E08E..=0x1E090,
    0x1E12F..=0x1E130,
    0x1E136..=0x1E137,
    0x1E2AD..=0x1E2AF,
    0x1E2EB..=0x1E2EC,
    0x1E2EF..=0x1E2F0,
    0x1E4EB..=0x1E4EC,
    0x1E4EF..=0x1E4F0,
    0x1E5ED..=0x1E5F0,
    0x1E6E2..=0x1E6E7,
    0x1E6ED..=0x1E6F0,
    0x1E6F4..=0x1E6F6,
    0x1E8CF..=0x1E8D0,
    0x1E8D6..=0x1E8D7,
    0x1E943..=0x1E944,
    0x1E94A..=0x1E94B,
    0xE001F..=0xE0020,
    0xE007F..=0xE0080,
    0xE00FF..=0xE0100,
    0xE01EF..=0xE01F0,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
