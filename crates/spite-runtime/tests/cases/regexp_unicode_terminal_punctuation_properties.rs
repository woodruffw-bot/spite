//! Terminal_Punctuation and its exact Term alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_terminal_punctuation_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Terminal_Punctuation}",
        r"\P{Term}",
        r"[^\p{Term}]",
        r"[\p{Term}K]",
        r"[\P{Terminal_Punctuation}K]",
        r"^(?<x>[\p{Term}K])$",
        r"(?<x>\P{Terminal_Punctuation})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{21}\u{2028}\u{3002}\u{2029}",
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
fn unicode_terminal_punctuation_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Terminal_Punctuation}}/{mode},
                    b = /\p{{Term}}/i{mode},
                    n = /\P{{Term}}/i{mode};
                a.test('\u0021') && a.test('\u003F') && b.test('\u0021') && !a.test('\u200D') && !b.test(
                        '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u0021') && n.test('😀') && n
                    .test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Terminal_Punctuation}}/{mode},
                    s = /\s/{mode};
                p.test('\u0021') && p.test('\u003F') && !s.test('\u0021') && !s.test('\u003F') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{Term}}K]/i{mode},
                    b = /[^\P{{Terminal_Punctuation}}K]/i{mode},
                    c = /[\p{{Term}}\P{{Terminal_Punctuation}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0021') && !a.test('ſ') && b.test('\u003F') && !b.test(
                    'K') && c.test('😀') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{Term}}$/{mode},
                    b = /^\P{{Term}}$/{mode};
                a.test('\u0021') && !a.test('\u003F\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{Term}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{Term}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\u0021');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{Term}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\u0021');
                m.index === 2 && m[0] === '\u0021' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{Term}}/{mode}y;r.lastIndex=1;r.exec('😀\u0021')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u003F😀\u0021G'.matchAll(/(?<x>\P{{Term}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\u003F\u0021😀X'.replace(/(?<x>\p{{Terminal_Punctuation}})/{mode}g,'<$1:$<x>>')==='<\u003F:\u003F><\u0021:\u0021>😀X'"
        ));
        assert_true(&format!(
            r#"
                '😀\u003F\u0021G'.split(/(\p{{Term}})/{mode}).join('|') === '😀|\u003F||\u0021|G' &&
                    '😀\u0021'.search(/\p{{Term}}/{mode}) === 2 && '\u003F\u0021😀'.match(
                        /\p{{Term}}/{mode}g).join('') === '\u003F\u0021'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0021\u003F😀\u0021'.replace(/(\p{{Term}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Terminal_Punctuation})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Terminal_Punctuation})$' && c.unicodeSets && !c.unicode && !c
                .multiline && c.exec('\u0021').groups.x === '\u0021' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{Term}/uy;
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
            let r = /\p{Term}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u0021')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{Term}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀\u0021')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_terminal_punctuation_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{Term}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    assert_true(r"/[\p{Term}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{Term}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x21),
                m = /[\p{Term}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{Term}+b/u.exec('AA')", r"/\p{Term}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_terminal_punctuation_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{Term}})+\B/d{mode}g,
                    m = r.exec('😀\u0021\u003F\u3002\uFEFF');
                m.index === 2 && m[0] === '\u0021\u003F\u3002' && m.groups.x === '\u3002' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_terminal_punctuation_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [0x2e60u32, 0x2e61] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Terminal_Punctuation", "Term"] {
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
    assert_eq!(comparisons, 16);
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0x1F600..=0x1F600,
    0x0020..=0x0022,
    0x002B..=0x002F,
    0x0039..=0x003C,
    0x003E..=0x0040,
    0x037D..=0x037F,
    0x0386..=0x0388,
    0x0588..=0x058A,
    0x05C2..=0x05C4,
    0x060B..=0x060D,
    0x061A..=0x061D,
    0x061F..=0x0620,
    0x06D3..=0x06D5,
    0x06FF..=0x0700,
    0x070A..=0x070D,
    0x07F7..=0x07FA,
    0x082F..=0x0830,
    0x0835..=0x0837,
    0x083E..=0x083F,
    0x085D..=0x085F,
    0x0963..=0x0966,
    0x0E59..=0x0E5C,
    0x0F07..=0x0F09,
    0x0F0C..=0x0F0D,
    0x0F12..=0x0F13,
    0x1049..=0x104C,
    0x1360..=0x1361,
    0x1368..=0x1369,
    0x166D..=0x166F,
    0x16EA..=0x16EB,
    0x16ED..=0x16EE,
    0x1734..=0x1737,
    0x17D3..=0x17D4,
    0x17D6..=0x17D7,
    0x17D9..=0x17DB,
    0x1801..=0x1802,
    0x1805..=0x180A,
    0x1943..=0x1946,
    0x1AA7..=0x1AA8,
    0x1AAB..=0x1AAC,
    0x1B4D..=0x1B50,
    0x1B59..=0x1B5D,
    0x1B5F..=0x1B60,
    0x1B7C..=0x1B7D,
    0x1B7F..=0x1B80,
    0x1C3A..=0x1C3B,
    0x1C3F..=0x1C40,
    0x1C7D..=0x1C80,
    0x2023..=0x2025,
    0x203B..=0x203E,
    0x2046..=0x2047,
    0x2049..=0x204A,
    0x2CF8..=0x2CF9,
    0x2CFB..=0x2CFC,
    0x2E2D..=0x2E2F,
    0x2E3B..=0x2E3D,
    0x2E40..=0x2E42,
    0x2E4B..=0x2E50,
    0x2E52..=0x2E55,
    0x2E5F..=0x2E5F,
    0x2E62..=0x2E62,
    0x3000..=0x3003,
    0xA4FD..=0xA500,
    0xA60C..=0xA60D,
    0xA60F..=0xA610,
    0xA6F2..=0xA6F3,
    0xA6F7..=0xA6F8,
    0xA875..=0xA878,
    0xA8CD..=0xA8D0,
    0xA92E..=0xA930,
    0xA9C6..=0xA9C7,
    0xA9C9..=0xA9CA,
    0xAA5C..=0xAA5D,
    0xAA5F..=0xAA60,
    0xAADE..=0xAAE0,
    0xAAEF..=0xAAF2,
    0xABEA..=0xABEC,
    0xFE11..=0xFE17,
    0xFE4F..=0xFE50,
    0xFE52..=0xFE54,
    0xFE57..=0xFE58,
    0xFF00..=0xFF02,
    0xFF0B..=0xFF0F,
    0xFF19..=0xFF1C,
    0xFF1E..=0xFF20,
    0xFF60..=0xFF65,
    0x1039E..=0x103A0,
    0x103CF..=0x103D1,
    0x10856..=0x10858,
    0x1091E..=0x10920,
    0x10A55..=0x10A58,
    0x10AEF..=0x10AF0,
    0x10AF5..=0x10AF6,
    0x10B39..=0x10B3A,
    0x10B3F..=0x10B40,
    0x10B98..=0x10B99,
    0x10B9C..=0x10B9D,
    0x10F54..=0x10F55,
    0x10F59..=0x10F5A,
    0x10F85..=0x10F86,
    0x10F89..=0x10F8A,
    0x11046..=0x11047,
    0x1104D..=0x1104E,
    0x110BD..=0x110BE,
    0x110C1..=0x110C2,
    0x11140..=0x11141,
    0x11143..=0x11144,
    0x111C4..=0x111C7,
    0x111CC..=0x111CE,
    0x111DD..=0x111E0,
    0x11237..=0x11238,
    0x1123C..=0x1123D,
    0x112A8..=0x112AA,
    0x113D3..=0x113D6,
    0x1144A..=0x1144B,
    0x1144D..=0x1144E,
    0x11459..=0x1145C,
    0x115C1..=0x115C2,
    0x115C5..=0x115C6,
    0x115C8..=0x115C9,
    0x115D7..=0x115D8,
    0x11640..=0x11643,
    0x1173B..=0x1173C,
    0x1173E..=0x1173F,
    0x11943..=0x11947,
    0x11A41..=0x11A44,
    0x11A9A..=0x11A9D,
    0x11AA0..=0x11AA3,
    0x11C40..=0x11C41,
    0x11C43..=0x11C44,
    0x11C70..=0x11C72,
    0x11EF6..=0x11EF9,
    0x11F42..=0x11F45,
    0x1246F..=0x12470,
    0x12474..=0x12475,
    0x16A6D..=0x16A70,
    0x16AF4..=0x16AF6,
    0x16B36..=0x16B37,
    0x16B39..=0x16B3A,
    0x16B43..=0x16B45,
    0x16D6D..=0x16D70,
    0x16E96..=0x16E99,
    0x1BC9E..=0x1BCA0,
    0x1DA86..=0x1DA87,
    0x1DA8A..=0x1DA8B,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
