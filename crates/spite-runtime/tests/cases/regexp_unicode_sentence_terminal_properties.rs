//! Sentence_Terminal and its exact STerm alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_sentence_terminal_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Sentence_Terminal}",
        r"\P{STerm}",
        r"[^\p{STerm}]",
        r"[\p{STerm}K]",
        r"[\P{Sentence_Terminal}K]",
        r"^(?<x>[\p{STerm}K])$",
        r"(?<x>\P{Sentence_Terminal})",
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
fn unicode_sentence_terminal_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Sentence_Terminal}}/{mode},
                    b = /\p{{STerm}}/i{mode},
                    n = /\P{{STerm}}/i{mode};
                a.test('\u0021') && a.test('\u003F') && b.test('\u0021') && !a.test('\u200D') && !b.test(
                        '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u0021') && n.test('😀') && n
                    .test('\uD800') && n.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Sentence_Terminal}}/{mode},
                    s = /\s/{mode};
                p.test('\u0021') && p.test('\u003F') && !s.test('\u0021') && !s.test('\u003F') && !p.test(
                    '\uFEFF') && s.test('\uFEFF')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{STerm}}K]/i{mode},
                    b = /[^\P{{Sentence_Terminal}}K]/i{mode},
                    c = /[\p{{STerm}}\P{{Sentence_Terminal}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u0021') && !a.test('ſ') && b.test('\u003F') && !b.test(
                    'K') && c.test('😀') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{STerm}}$/{mode},
                    b = /^\P{{STerm}}$/{mode};
                a.test('\u0021') && !a.test('\u003F\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_sentence_terminal_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{STerm}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{STerm}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\u0021');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{STerm}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\u0021');
                m.index === 2 && m[0] === '\u0021' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{STerm}}/{mode}y;r.lastIndex=1;r.exec('😀\u0021')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_sentence_terminal_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u003F😀\u0021G'.matchAll(/(?<x>\P{{STerm}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\u003F\u0021😀X'.replace(/(?<x>\p{{Sentence_Terminal}})/{mode}g,'<$1:$<x>>')==='<\u003F:\u003F><\u0021:\u0021>😀X'"
        ));
        assert_true(&format!(
            r"'😀\u003F\u0021G'.split(/(\p{{STerm}})/{mode}).join('|')==='😀|\u003F||\u0021|G'&&'😀\u0021'.search(/\p{{STerm}}/{mode})===2&&'\u003F\u0021😀'.match(/\p{{STerm}}/{mode}g).join('')==='\u003F\u0021'"
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u0021\u003F😀\u0021'.replace(/(\p{{STerm}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_sentence_terminal_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Sentence_Terminal})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Sentence_Terminal})$' && c.unicodeSets && !c.unicode && !c.multiline &&
                c.exec('\u0021').groups.x === '\u0021' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{STerm}/uy;
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
            let r = /\p{STerm}/vy;
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
            let r = /\p{STerm}/vy;
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
fn unicode_sentence_terminal_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{STerm}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    assert_true(r"/[\p{STerm}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{STerm}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x21),
                m = /[\p{STerm}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{STerm}+b/u.exec('AA')", r"/\p{STerm}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_sentence_terminal_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{STerm}})+\B/d{mode}g,
                    m = r.exec('😀\u0021\u003F\u3002\uFEFF');
                m.index === 2 && m[0] === '\u0021\u003F\u3002' && m.groups.x === '\u3002' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_sentence_terminal_property_unicode18_addition() {
    let mut comparisons = 0;
    for point in [0x2e60u32, 0x2e61] {
        let text = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for name in ["Sentence_Terminal", "STerm"] {
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
    0x002D..=0x002F,
    0x003E..=0x0040,
    0x0588..=0x058A,
    0x061C..=0x061D,
    0x061F..=0x0620,
    0x06D3..=0x06D5,
    0x06FF..=0x0700,
    0x0702..=0x0703,
    0x07F8..=0x07FA,
    0x0836..=0x083A,
    0x083C..=0x083F,
    0x0963..=0x0966,
    0x1049..=0x104C,
    0x1361..=0x1363,
    0x1366..=0x1369,
    0x166D..=0x166F,
    0x1734..=0x1737,
    0x17D3..=0x17D6,
    0x1802..=0x1804,
    0x1808..=0x180A,
    0x1943..=0x1946,
    0x1AA7..=0x1AA8,
    0x1AAB..=0x1AAC,
    0x1B4D..=0x1B50,
    0x1B59..=0x1B60,
    0x1B7C..=0x1B7D,
    0x1B7F..=0x1B80,
    0x1C3A..=0x1C3D,
    0x1C7D..=0x1C80,
    0x2023..=0x2025,
    0x203B..=0x203E,
    0x2046..=0x2047,
    0x2049..=0x204A,
    0x2CF8..=0x2CF9,
    0x2CFB..=0x2CFC,
    0x2E2D..=0x2E2F,
    0x2E3B..=0x2E3D,
    0x2E52..=0x2E55,
    0x2E5F..=0x2E5F,
    0x2E62..=0x2E62,
    0x3001..=0x3003,
    0xA4FE..=0xA500,
    0xA60D..=0xA610,
    0xA6F2..=0xA6F4,
    0xA6F6..=0xA6F8,
    0xA875..=0xA878,
    0xA8CD..=0xA8D0,
    0xA92E..=0xA930,
    0xA9C7..=0xA9CA,
    0xAA5C..=0xAA5D,
    0xAA5F..=0xAA60,
    0xAAEF..=0xAAF2,
    0xABEA..=0xABEC,
    0xFE11..=0xFE17,
    0xFE51..=0xFE53,
    0xFE55..=0xFE58,
    0xFF00..=0xFF02,
    0xFF0D..=0xFF0F,
    0xFF1E..=0xFF20,
    0xFF60..=0xFF62,
    0x10A55..=0x10A58,
    0x10F54..=0x10F55,
    0x10F59..=0x10F5A,
    0x10F85..=0x10F86,
    0x10F89..=0x10F8A,
    0x11046..=0x11049,
    0x110BD..=0x110BE,
    0x110C1..=0x110C2,
    0x11140..=0x11141,
    0x11143..=0x11144,
    0x111C4..=0x111C7,
    0x111CC..=0x111CE,
    0x111DD..=0x111E0,
    0x11237..=0x1123D,
    0x112A8..=0x112AA,
    0x113D3..=0x113D6,
    0x1144A..=0x1144D,
    0x115C1..=0x115C4,
    0x115C8..=0x115C9,
    0x115D7..=0x115D8,
    0x11640..=0x11643,
    0x1173B..=0x1173C,
    0x1173E..=0x1173F,
    0x11943..=0x11947,
    0x11A41..=0x11A44,
    0x11A9A..=0x11A9D,
    0x11C40..=0x11C43,
    0x11EF6..=0x11EF9,
    0x11F42..=0x11F45,
    0x16A6D..=0x16A70,
    0x16AF4..=0x16AF6,
    0x16B36..=0x16B39,
    0x16B43..=0x16B45,
    0x16D6D..=0x16D70,
    0x16E97..=0x16E99,
    0x1BC9E..=0x1BCA0,
    0x1DA87..=0x1DA89,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
