//! Emoji_Modifier_Base and its exact EBase alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_emoji_modifier_base_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Emoji_Modifier_Base}",
        r"\P{EBase}",
        r"[^\p{EBase}]",
        r"[\p{EBase}K]",
        r"[\P{Emoji_Modifier_Base}K]",
        r"^(?<x>[\p{EBase}K])$",
        r"(?<x>\P{Emoji_Modifier_Base})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{261d}\u{2028}\u{270a}\u{2029}",
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
fn unicode_emoji_modifier_base_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Emoji_Modifier_Base}}/{mode},
                    b = /\p{{EBase}}/i{mode},
                    n = /\P{{EBase}}/i{mode};
                a.test('\u261D') && a.test('\u270A') && b.test('\u261D') && !a.test('\u005f') && !b.test(
                        '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u261D') && n.test('😀') && n
                    .test('\uD800') && n.test('\u0020')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Emoji_Modifier_Base}}/{mode},
                    s = /\s/{mode};
                p.test('\u261D') && p.test('\u270A') && !s.test('\u261D') && !s.test('\u270A') && !p.test(
                    '\u0020') && s.test('\u0020')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{EBase}}K]/i{mode},
                    b = /[^\P{{Emoji_Modifier_Base}}K]/i{mode},
                    c = /[\p{{EBase}}\P{{Emoji_Modifier_Base}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u261D') && !a.test('ſ') && b.test('\u270A') && !b.test(
                    'K') && c.test('😀') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{EBase}}$/{mode},
                    b = /^\P{{EBase}}$/{mode};
                a.test('\u261D') && !a.test('\u270A\n') && !a.test('') && !a.test('\n') && b.test('😀') && !b
                    .test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{EBase}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uD83C\uDFC3');
                m.index === 0 && m[0] === '\uD83C\uDFC3' && m.groups.x === m[0] && m.indices[1][0] === 0 && m
                    .indices[1][1] === 2 && r.lastIndex === 2
            "#
        ));
    }

    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /^(?<x>([\p{{EBase}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{EBase}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('😀\u261D');
                m.index === 0 && m[1] === '😀' && m.indices[1][1] === 2 && r.lastIndex === 2 && Object
                    .getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{EBase}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('😀\u261D');
                m.index === 2 && m[0] === '\u261D' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{EBase}}/{mode}y;r.lastIndex=1;r.exec('😀\u261D')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u270A😀\u261DG'.matchAll(/(?<x>\P{{EBase}})/d{mode}g)];
                a.length === 2 && a[0][1] === '😀' && a[0].index === 1 && a[0].indices[1][1] === 3 && a[1][
                    1
                ] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\u270A\u261D😀X'.replace(/(?<x>\p{{Emoji_Modifier_Base}})/{mode}g,'<$1:$<x>>')==='<\u270A:\u270A><\u261D:\u261D>😀X'"
        ));
        assert_true(&format!(
            r#"
                '😀\u270A\u261DG'.split(/(\p{{EBase}})/{mode}).join('|') === '😀|\u270A||\u261D|G' &&
                    '😀\u261D'.search(/\p{{EBase}}/{mode}) === 2 && '\u270A\u261D😀'.match(
                        /\p{{EBase}}/{mode}g).join('') === '\u270A\u261D'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u261D\u270A😀\u261D'.replace(/(\p{{EBase}})/{mode}g, (m, c, p) => {{
                    t += p;
                    return '_'
                }});
                x === '__😀_' && t === '014'
            "#
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Emoji_Modifier_Base})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Emoji_Modifier_Base})$' && c.unicodeSets && !c.unicode && !c
                .multiline && c.exec('\u261D').groups.x === '\u261D' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{EBase}/uy;
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
            let r = /\p{EBase}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u261D')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{EBase}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('😀\u261D')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_emoji_modifier_base_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{EBase}/ug,marker=0,text='😀'.repeat(10000);r.lastIndex=1")
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
    assert_true(r"/[\p{EBase}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{EBase}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x261d),
                m = /[\p{EBase}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{EBase}+b/u.exec('AA')", r"/\p{EBase}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_emoji_modifier_base_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{EBase}})+\B/d{mode}g,
                    m = r.exec('😀\u261D\u270A\u270A\u0020');
                m.index === 2 && m[0] === '\u261D\u270A\u270A' && m.groups.x === '\u270A' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_emoji_modifier_base_property_unicode18_addition() {
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'u'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'u');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'ui');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'v'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'v');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'vi');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'u'),
                n = new RegExp('^\\P{EBase}$', 'u');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'ui'),
                n = new RegExp('^\\P{EBase}$', 'ui');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'v'),
                n = new RegExp('^\\P{EBase}$', 'v');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'vi'),
                n = new RegExp('^\\P{EBase}$', 'vi');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'u'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'u');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'ui');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'v'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'v');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Modifier_Base}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Modifier_Base}$', 'vi');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'u'),
                n = new RegExp('^\\P{EBase}$', 'u');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'ui'),
                n = new RegExp('^\\P{EBase}$', 'ui');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'v'),
                n = new RegExp('^\\P{EBase}$', 'v');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EBase}$', 'vi'),
                n = new RegExp('^\\P{EBase}$', 'vi');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0x1F600..=0x1F600,
    0x261C..=0x261E,
    0x26F8..=0x26FA,
    0x2709..=0x270A,
    0x270D..=0x270E,
    0x1F384..=0x1F386,
    0x1F3C1..=0x1F3C2,
    0x1F3C4..=0x1F3CA,
    0x1F3CC..=0x1F3CD,
    0x1F441..=0x1F446,
    0x1F450..=0x1F451,
    0x1F465..=0x1F466,
    0x1F478..=0x1F479,
    0x1F47B..=0x1F47D,
    0x1F480..=0x1F481,
    0x1F483..=0x1F485,
    0x1F487..=0x1F488,
    0x1F48E..=0x1F492,
    0x1F4A9..=0x1F4AB,
    0x1F573..=0x1F576,
    0x1F579..=0x1F57B,
    0x1F58F..=0x1F591,
    0x1F594..=0x1F597,
    0x1F644..=0x1F645,
    0x1F647..=0x1F648,
    0x1F64A..=0x1F64B,
    0x1F64F..=0x1F650,
    0x1F6A2..=0x1F6A4,
    0x1F6B3..=0x1F6B4,
    0x1F6B6..=0x1F6B7,
    0x1F6BF..=0x1F6C1,
    0x1F6CB..=0x1F6CD,
    0x1F90B..=0x1F910,
    0x1F917..=0x1F918,
    0x1F91F..=0x1F920,
    0x1F925..=0x1F927,
    0x1F92F..=0x1F930,
    0x1F939..=0x1F93C,
    0x1F93E..=0x1F93F,
    0x1F976..=0x1F978,
    0x1F9B4..=0x1F9BC,
    0x1F9CC..=0x1F9CD,
    0x1F9CF..=0x1F9D1,
    0x1F9DD..=0x1F9DE,
    0x1FAC2..=0x1FAC3,
    0x1FAC5..=0x1FAC6,
    0x1FAEF..=0x1FAF0,
    0x1FAFB..=0x1FAFB,
    0xFEFF..=0xFEFF,
    0x017F..=0x017F,
    0x212A..=0x212A,
    0x000A..=0x000A,
];
