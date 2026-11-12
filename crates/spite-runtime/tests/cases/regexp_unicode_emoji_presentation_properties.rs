//! Emoji_Presentation and its exact EPres alias over complete Unicode characters.
use spite_core::JsString;
use spite_runtime::{Error, Limits, Realm, Value};
use std::fmt::Write;
use std::ops::RangeInclusive;

use crate::common::regexp_matches::last_index_cases;
use crate::common::{assert_true, unicode_samples};

#[test]
fn unicode_emoji_presentation_property_results_snapshot() {
    let boundary_samples = unicode_samples::from_ranges(BOUNDARY_SAMPLES);
    let mut rows = String::new();
    for body in [
        r"\p{Emoji_Presentation}",
        r"\P{EPres}",
        r"[^\p{EPres}]",
        r"[\p{EPres}K]",
        r"[\P{Emoji_Presentation}K]",
        r"^(?<x>[\p{EPres}K])$",
        r"(?<x>\P{Emoji_Presentation})",
    ] {
        let pattern = JsString::from(body);
        for flags in ["dug", "diug", "divg", "divmy"] {
            for input in [
                boundary_samples.as_str(),
                "X\r\n\u{231a}\u{2028}\u{23f3}\u{2029}",
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
fn unicode_emoji_presentation_properties_aliases_complements_and_unions() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = /\p{{Emoji_Presentation}}/{mode},
                    b = /\p{{EPres}}/i{mode},
                    n = /\P{{EPres}}/i{mode};
                a.test('\u231A') && a.test('\u23F3') && b.test('\u231A') && !a.test('\u005f') && !b.test(
                    '\u005f') && !b.test('ſ') && !b.test('K') && !n.test('\u231A') && n.test(
                    '\uDB40\uDC01') && n.test('\uD800') && n.test('\u0020')
            "#
        ));
        assert_true(&format!(
            r#"
                let p = /\p{{Emoji_Presentation}}/{mode},
                    s = /\s/{mode};
                p.test('\u231A') && p.test('\u23F3') && !s.test('\u231A') && !s.test('\u23F3') && !p.test(
                    '\u0020') && s.test('\u0020')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /[\p{{EPres}}K]/i{mode},
                    b = /[^\P{{Emoji_Presentation}}K]/i{mode},
                    c = /[\p{{EPres}}\P{{Emoji_Presentation}}]/{mode};
                a.test('K') && a.test('k') && a.test('\u231A') && !a.test('ſ') && b.test('\u23F3') && !b.test(
                    'K') && c.test('\uDB40\uDC01') && c.test('A') && c.test('\uD800')
            "#
        ));
        assert_true(&format!(
            r#"
                let a = /^\p{{EPres}}$/{mode},
                    b = /^\P{{EPres}}$/{mode};
                a.test('\u231A') && !a.test('\u23F3\n') && !a.test('') && !a.test('\n') && b.test(
                    '\uDB40\uDC01') && !b.test('AA')
            "#
        ));
    }
}

#[test]
fn unicode_emoji_presentation_properties_captures_context_and_pair_starts() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /(?<x>\p{{EPres}})/d{mode}y;
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
                let r = /^(?<x>([\p{{EPres}}K]))$/dim{mode}g,
                    m = r.exec('x\r\nK\u2028');
                m.index === 3 && m[1] === 'K' && m[2] === 'K' && m.indices.groups.x === m.indices[1] && m
                    .indices[2][1] === 4 && r.lastIndex === 4
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /(?<__proto__>\P{{EPres}})/d{mode}y;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u231A');
                m.index === 0 && m[1] === '\uDB40\uDC01' && m.indices[1][1] === 2 && r.lastIndex === 2 &&
                    Object.getPrototypeOf(m.groups) === null && m.indices.groups.__proto__ === m.indices[1]
            "#
        ));
        assert_true(&format!(
            r#"
                let r = /\p{{EPres}}/d{mode}g;
                r.lastIndex = 1;
                let m = r.exec('\uDB40\uDC01\u231A');
                m.index === 2 && m[0] === '\u231A' && r.lastIndex === 3
            "#
        ));
        assert_true(&format!(
            r"let r=/\p{{EPres}}/{mode}y;r.lastIndex=1;r.exec('\uDB40\uDC01\u231A')===null&&r.lastIndex===0"
        ));
    }
}

#[test]
fn unicode_emoji_presentation_properties_consumers_original_text() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let a = [...'\u23F3\uDB40\uDC01\u231AG'.matchAll(/(?<x>\P{{EPres}})/d{mode}g)];
                a.length === 2 && a[0][1] === '\uDB40\uDC01' && a[0].index === 1 && a[0].indices[1][1] === 3 &&
                    a[1][1] === 'G'
            "#
        ));
        assert_true(&format!(
            r"'\u23F3\u231A\uDB40\uDC01X'.replace(/(?<x>\p{{Emoji_Presentation}})/{mode}g,'<$1:$<x>>')==='<\u23F3:\u23F3><\u231A:\u231A>\uDB40\uDC01X'"
        ));
        assert_true(&format!(
            r#"
                '\uDB40\uDC01\u23F3\u231AG'.split(/(\p{{EPres}})/{mode}).join('|') ===
                    '\uDB40\uDC01|\u23F3||\u231A|G' && '\uDB40\uDC01\u231A'.search(
                        /\p{{EPres}}/{mode}) === 2 && '\u23F3\u231A\uDB40\uDC01'.match(
                        /\p{{EPres}}/{mode}g).join('') === '\u23F3\u231A'
            "#
        ));
        assert_true(&format!(
            r#"
                let t = '';
                let x = '\u231A\u23F3\uDB40\uDC01\u231A'.replace(/(\p{{EPres}})/{mode}g, (m, c,
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
fn unicode_emoji_presentation_properties_clones_coercions_and_strict_writes() {
    assert_true(
        r#"
            let r = /^(?<x>\p{Emoji_Presentation})$/dmu,
                c = new RegExp(r, 'divy');
            c.source === '^(?<x>\\p{Emoji_Presentation})$' && c.unicodeSets && !c.unicode && !c
                .multiline && c.exec('\u231A').groups.x === '\u231A' && c.lastIndex === 1
        "#,
    );
    assert_true(
        r#"
            let t = '',
                r = /\P{EPres}/uy;
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
            let r = /\p{EPres}/vy;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\u231A')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 0
        "#,
    );
    assert_true(
        r#"
            let r = /\p{EPres}/vy;
            r.lastIndex = 1;
            Object.defineProperty(r, 'lastIndex', {
                writable: false
            });
            let caught = false;
            try {
                r.exec('\uDB40\uDC01\u231A')
            } catch (e) {
                caught = e instanceof TypeError
            }
            caught && r.lastIndex === 1
        "#,
    );
}

#[test]
fn unicode_emoji_presentation_properties_opted_work_and_remaining_sets() {
    let mut realm = Realm::new(Limits {
        max_steps: Some(30000),
        ..Limits::default()
    });
    realm
        .eval("let r=/\\p{EPres}/ug,marker=0,text='\\uDB40\\uDC01'.repeat(10000);r.lastIndex=1")
        .unwrap();
    assert!(matches!(
        realm.eval("try{r.exec(text)}catch{marker=1}finally{marker=2}"),
        Err(Error::Limit { .. })
    ));
    assert_eq!(
        realm.eval("marker===0&&r.lastIndex===1"),
        Ok(Value::Boolean(true))
    );
    assert_true(r"/[\p{EPres}&&A]/v.exec('A')===null");
    assert_true(r"/[\p{EPres}--A]/v.exec('A')===null");
    assert_true(
        r#"
            let s = String.fromCodePoint(0x231a),
                m = /[\p{EPres}--A]/v.exec(s);
            m !== null && m.length === 1 && m[0] === s && m.index === 0 && m.input === s
        "#,
    );
    for source in [r"/\p{EPres}+b/u.exec('AA')", r"/\p{EPres}A/v.exec('AA')"] {
        crate::common::regexp_oracle::assert_matches_reference(&mut Realm::default(), source);
    }
}

#[test]
fn unicode_emoji_presentation_repetitions_retain_last_capture_and_outside_words() {
    for mode in ["u", "v"] {
        assert_true(&format!(
            r#"
                let r = /\B(?<x>\p{{EPres}})+\B/d{mode}g,
                    m = r.exec('\uDB40\uDC01\u231A\u23F3\u23F3\u0020');
                m.index === 2 && m[0] === '\u231A\u23F3\u23F3' && m.groups.x === '\u23F3' && m.indices.groups
                    .x[0] === 4 && m.indices.groups.x[1] === 5 && r.lastIndex === 5
            "#
        ));
    }
}

#[test]
fn unicode_emoji_presentation_property_unicode18_addition() {
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83D\uDED9') === true && n.test('\uD83D\uDED9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDE8B') === true && n.test('\uD83E\uDE8B') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDE8C') === true && n.test('\uD83E\uDE8C') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDE8D') === true && n.test('\uD83E\uDE8D') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDECC') === true && n.test('\uD83E\uDECC') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDEDD') === true && n.test('\uD83E\uDEDD') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDEEB') === true && n.test('\uD83E\uDEEB') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDEF9') === true && n.test('\uD83E\uDEF9') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'u'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'u');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'ui'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'ui');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'v'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'v');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{Emoji_Presentation}$', 'vi'),
                n = new RegExp('^\\P{Emoji_Presentation}$', 'vi');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'u'),
                n = new RegExp('^\\P{EPres}$', 'u');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'ui'),
                n = new RegExp('^\\P{EPres}$', 'ui');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'v'),
                n = new RegExp('^\\P{EPres}$', 'v');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
    assert_true(
        r#"
            let p = new RegExp('^\\p{EPres}$', 'vi'),
                n = new RegExp('^\\P{EPres}$', 'vi');
            p.test('\uD83E\uDEFA') === true && n.test('\uD83E\uDEFA') === false
        "#,
    );
}

// Property boundary probes in iteration order, including neighboring code points.
const BOUNDARY_SAMPLES: &[RangeInclusive<u32>] = &[
    0xE0001..=0xE0001,
    0x2319..=0x231C,
    0x23E8..=0x23E9,
    0x23EC..=0x23ED,
    0x23EF..=0x23F4,
    0x25FC..=0x25FF,
    0x2613..=0x2616,
    0x2647..=0x2648,
    0x2653..=0x2654,
    0x267E..=0x2680,
    0x2692..=0x2694,
    0x26A0..=0x26A2,
    0x26A9..=0x26AC,
    0x26BC..=0x26BF,
    0x26C3..=0x26C6,
    0x26CD..=0x26CF,
    0x26D3..=0x26D5,
    0x26E9..=0x26EB,
    0x26F1..=0x26F6,
    0x26F9..=0x26FE,
    0x2704..=0x2706,
    0x2709..=0x270C,
    0x2727..=0x2729,
    0x274B..=0x274F,
    0x2752..=0x2753,
    0x2755..=0x2758,
    0x2794..=0x2795,
    0x2797..=0x2798,
    0x27AF..=0x27B1,
    0x27BE..=0x27C0,
    0x2B1A..=0x2B1D,
    0x2B4F..=0x2B51,
    0x2B54..=0x2B56,
    0x1F003..=0x1F005,
    0x1F0CE..=0x1F0D0,
    0x1F18D..=0x1F191,
    0x1F19A..=0x1F19B,
    0x1F1E5..=0x1F1E6,
    0x1F1FF..=0x1F202,
    0x1F219..=0x1F21B,
    0x1F22E..=0x1F232,
    0x1F236..=0x1F238,
    0x1F23A..=0x1F23B,
    0x1F24F..=0x1F252,
    0x1F2FF..=0x1F300,
    0x1F320..=0x1F321,
    0x1F32C..=0x1F32D,
    0x1F335..=0x1F337,
    0x1F37C..=0x1F37E,
    0x1F393..=0x1F394,
    0x1F39F..=0x1F3A0,
    0x1F3CA..=0x1F3CB,
    0x1F3CE..=0x1F3CF,
    0x1F3D3..=0x1F3D4,
    0x1F3DF..=0x1F3E0,
    0x1F3F0..=0x1F3F1,
    0x1F3F3..=0x1F3F5,
    0x1F3F7..=0x1F3F8,
    0x1F43E..=0x1F442,
    0x1F4FC..=0x1F4FF,
    0x1F53D..=0x1F53E,
    0x1F54A..=0x1F54B,
    0x1F54E..=0x1F550,
    0x1F567..=0x1F568,
    0x1F579..=0x1F57B,
    0x1F594..=0x1F597,
    0x1F5A3..=0x1F5A5,
    0x1F5FA..=0x1F5FB,
    0x1F64F..=0x1F650,
    0x1F67F..=0x1F680,
    0x1F6C5..=0x1F6C6,
    0x1F6CB..=0x1F6CD,
    0x1F6CF..=0x1F6D0,
    0x1F6D2..=0x1F6D5,
    0x1F6DA..=0x1F6DC,
    0x1F6DF..=0x1F6E0,
    0x1F6EA..=0x1F6ED,
    0x1F6F3..=0x1F6F4,
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
