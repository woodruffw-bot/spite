use super::*;

enum Class {
    A,
    UpperS,
    Any,
    HighSurrogate,
    LowSurrogate,
    Grinning,
}

struct Case<'a> {
    source: &'a str,
    class: Class,
    literal: u32,
    class_first: bool,
    ignore_case: bool,
    multiline: bool,
    start_anchor: bool,
    end_anchor: bool,
    start_word: Option<bool>,
    end_word: Option<bool>,
}

const CASES: [Case<'static>; 16] = [
    Case {
        source: r"\B[[a]]a\b",
        class: Class::A,
        literal: 0x61,
        class_first: true,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(false),
        end_word: Some(true),
    },
    Case {
        source: r"\Ba[[a]]\b",
        class: Class::A,
        literal: 0x61,
        class_first: false,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(false),
        end_word: Some(true),
    },
    Case {
        source: r"^\b[[a]]a\b$",
        class: Class::A,
        literal: 0x61,
        class_first: true,
        ignore_case: false,
        multiline: true,
        start_anchor: true,
        end_anchor: true,
        start_word: Some(true),
        end_word: Some(true),
    },
    Case {
        source: r"k[[S]]\b",
        class: Class::UpperS,
        literal: 0x6b,
        class_first: false,
        ignore_case: true,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: None,
        end_word: Some(true),
    },
    Case {
        source: r"\ba[[\p{Any}]]\B",
        class: Class::Any,
        literal: 0x61,
        class_first: false,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(true),
        end_word: Some(false),
    },
    Case {
        source: r"\B[[\p{Any}]]a\b",
        class: Class::Any,
        literal: 0x61,
        class_first: true,
        ignore_case: true,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(false),
        end_word: Some(true),
    },
    Case {
        source: r"\ba[[\p{Any}]]\B",
        class: Class::Any,
        literal: 0x61,
        class_first: false,
        ignore_case: true,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(true),
        end_word: Some(false),
    },
    Case {
        source: r"^\b[[a]]a\b$",
        class: Class::A,
        literal: 0x61,
        class_first: true,
        ignore_case: false,
        multiline: false,
        start_anchor: true,
        end_anchor: true,
        start_word: Some(true),
        end_word: Some(true),
    },
    Case {
        source: r"^\ba[[a]]\b$",
        class: Class::A,
        literal: 0x61,
        class_first: false,
        ignore_case: false,
        multiline: true,
        start_anchor: true,
        end_anchor: true,
        start_word: Some(true),
        end_word: Some(true),
    },
    Case {
        source: r"\b[[a]]\$\B$",
        class: Class::A,
        literal: 0x24,
        class_first: true,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: true,
        start_word: Some(true),
        end_word: Some(false),
    },
    Case {
        source: r"\B\$[[a]]\b$",
        class: Class::A,
        literal: 0x24,
        class_first: false,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: true,
        start_word: Some(false),
        end_word: Some(true),
    },
    Case {
        source: r"\B[[\u{1F600}]]\u{1F601}\B",
        class: Class::Grinning,
        literal: 0x1f601,
        class_first: true,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(false),
        end_word: Some(false),
    },
    Case {
        source: r"\B\u{1F600}[[\p{Any}]]",
        class: Class::Any,
        literal: 0x1f600,
        class_first: false,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(false),
        end_word: None,
    },
    Case {
        source: r"\B[[\uD800]]a\b",
        class: Class::HighSurrogate,
        literal: 0x61,
        class_first: true,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(false),
        end_word: Some(true),
    },
    Case {
        source: r"\ba[[\uDC00]]\B",
        class: Class::LowSurrogate,
        literal: 0x61,
        class_first: false,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(true),
        end_word: Some(false),
    },
    Case {
        source: r"\B[[\p{Any}]]a\b",
        class: Class::Any,
        literal: 0x61,
        class_first: true,
        ignore_case: false,
        multiline: false,
        start_anchor: false,
        end_anchor: false,
        start_word: Some(false),
        end_word: Some(true),
    },
];

// Independent facts for the selected alphabet. WordCharacters is a separate
// predicate over original neighbors, never inferred from class membership.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
}

fn word(point: u32, ignore_case: bool) -> bool {
    matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
        || (ignore_case && matches!(point, 0x17f | 0x212a))
}

fn expected(case: &Case<'_>, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
    if start > input.len() {
        return None;
    }
    let mut points = Vec::new();
    let mut boundaries = vec![0];
    for decoded in char::decode_utf16(input.code_units().iter().copied()) {
        let (point, width) = match decoded {
            Ok(character) => (u32::from(character), character.len_utf16()),
            Err(error) => (u32::from(error.unpaired_surrogate()), 1),
        };
        points.push(point);
        boundaries.push(boundaries.last().unwrap() + width);
    }
    // The documented Node-compatible Unicode search starts at the enclosing
    // character when the supplied UTF-16 index is its low-surrogate position.
    let first = boundaries.partition_point(|&at| at <= start) - 1;
    let boundary = |at: usize| {
        let before = at
            .checked_sub(1)
            .and_then(|at| points.get(at))
            .is_some_and(|&point| word(point, case.ignore_case));
        let after = points
            .get(at)
            .is_some_and(|&point| word(point, case.ignore_case));
        before != after
    };
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        let (class_point, literal_point) = if case.class_first {
            (points[at], points[at + 1])
        } else {
            (points[at + 1], points[at])
        };
        let class_matches = match case.class {
            Class::A => {
                if case.ignore_case {
                    fold(class_point) == 0x61
                } else {
                    class_point == 0x61
                }
            }
            Class::UpperS => {
                if case.ignore_case {
                    fold(class_point) == 0x73
                } else {
                    class_point == 0x53
                }
            }
            Class::Any => true,
            Class::HighSurrogate => class_point == 0xd800,
            Class::LowSurrogate => class_point == 0xdc00,
            Class::Grinning => class_point == 0x1f600,
        };
        let literal_matches = if case.ignore_case {
            fold(literal_point) == fold(case.literal)
        } else {
            literal_point == case.literal
        };
        let begin = boundaries[at];
        let end = boundaries[at + 2];
        let start_matches = !case.start_anchor
            || begin == 0
            || (case.multiline && line(input.code_units()[begin - 1]));
        let end_matches = !case.end_anchor
            || end == input.len()
            || (case.multiline && line(input.code_units()[end]));
        if class_matches
            && literal_matches
            && start_matches
            && end_matches
            && case.start_word.is_none_or(|wanted| boundary(at) == wanted)
            && case
                .end_word
                .is_none_or(|wanted| boundary(at + 2) == wanted)
        {
            return Some(begin..end);
        }
    }
    None
}

#[test]
fn checked_v_set_pair_word_ranges_match_sparse_reference() {
    let mut inputs: Vec<JsString> = [
        "",
        "a",
        "aa",
        "aaa!",
        "!aa?",
        "_aa0",
        "\naa\n",
        "\raa\r",
        "x\r\naa\u{2028}",
        "aa\r\naa\u{2029}",
        "Ks",
        "Kſ",
        "Ka!",
        "ſa!",
        "aK!",
        "aſ!",
        "αa!",
        "aα!",
        "a$",
        "$a",
        "a$\n",
        "$a\n",
        "😀😁",
        "😀😀a",
    ]
    .into_iter()
    .map(JsString::from)
    .collect();
    for units in [
        vec![0xd800, 0x61],
        vec![0x61, 0xdc00],
        vec![0xd800, 0xd800, 0x61],
        vec![0xd83d, 0xde00, 0xdc00, 0x61],
    ] {
        inputs.push(JsString::from_code_units(units));
    }
    let mut comparisons = 0;
    for case in &CASES {
        let plan = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
            &JsString::from(case.source),
            case.ignore_case,
            case.multiline,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        for input in &inputs {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        plan.find(input, start, sticky),
                        expected(case, input, start, sticky),
                        "source={:?} i={} m={} input={input:?} start={start} sticky={sticky}",
                        case.source,
                        case.ignore_case,
                        case.multiline,
                    );
                    comparisons += 1;
                }
            }
        }
    }
    assert_eq!(comparisons, 4_608);
}

#[test]
fn checked_v_set_pair_word_domains_and_work_prefixes() {
    for source in [
        "",
        "aa",
        "[[a]]a",
        "a[[a]]",
        r"\baa",
        r"\ba[a]",
        r"\b[[a]]",
        r"\b[[a]]aa",
        r"\ba[[a]]a",
        r"\b[[a]]a[[b]]",
        r"\b[[a]]a+",
        r"\b[[a]]+a",
        r"\b[[a]]{0}a",
        r"\b(a[[a]])",
        r"\b(?:[[a]])a",
        r"\b[[a]](a)",
        r"\b[[a]]a|\b[[b]]b",
        r"\b[[a]]^a",
        r"\ba$[[a]]",
        r"\b^[[a]]a",
        r"\b[[a]]a$\b",
        r"\b\b[[a]]a",
        r"\b[[a]]a\b\B",
        r"\b[[a]]a$$",
        r"\b[[a]]a$^",
        r"\b[[a]]a\b$^",
        r"\b[[a--a]&&[\q{ab}]]a",
        r"\ba[[a--a]&&[\q{ab}]]",
        r"\b[[a--a]&&[\p{RGI_Emoji}]]a",
        r"\ba[[a--a]&&[\p{RGI_Emoji}]]",
        r"\b[[\q{}]]a",
        r"\ba[[\q{a}]]",
        r"\b[[\p{RGI_Emoji}]]a",
    ] {
        assert!(
            RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                &JsString::from(source),
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
    // Every admitted word-bearing case remains outside both original domains.
    // Their original early framing charges are retained rather than routed
    // through the new constructor.
    for case in &CASES {
        let source = JsString::from(case.source);
        let mut class_charges = Vec::new();
        assert!(
            RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
                &source,
                case.ignore_case,
                case.multiline,
                |amount| {
                    class_charges.push(amount);
                    Ok::<(), ()>(())
                },
            )
            .unwrap()
            .is_none()
        );
        assert!(class_charges.is_empty());
        let mut literal_charges = Vec::new();
        assert!(
            RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
                &source,
                case.ignore_case,
                case.multiline,
                |amount| {
                    literal_charges.push(amount);
                    Ok::<(), ()>(())
                },
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(literal_charges.first(), Some(&source.len()));
    }
    for (text, ignore_case) in [
        (r"^\B[[\p{ASCII}]--[Q]]\u{1F600}\B$", false),
        (r"^\b\u{1F600}[[\p{ASCII}]--[Q]]\B$", false),
        (r"^\b[[K]&&[k]]ſ\b$", true),
        (r"^\bk[[S]&&[s]]\b$", true),
    ] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        assert!(
            RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                &source,
                ignore_case,
                false,
                |amount| {
                    complete.push(amount);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .is_some()
        );
        assert!(complete.len() > 1);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                    &source,
                    ignore_case,
                    false,
                    |amount| {
                        seen.push(amount);
                        if seen.len() == fail {
                            Err(fail)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail));
            assert_eq!(seen, complete[..fail]);
        }
    }
    for text in [
        format!(r"\b[[a]]\u{{{}61}}\b", "0".repeat(256)),
        format!(r"\b\u{{{}61}}[[a]]\b", "0".repeat(256)),
    ] {
        let source = JsString::from(text.as_str());
        let mut seen = Vec::new();
        assert!(matches!(
            RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                &source,
                false,
                false,
                |amount| {
                    seen.push(amount);
                    Err(7)
                },
            ),
            Err(RegExpUnicodeSetCompileError::Work(7))
        ));
        assert_eq!(seen, [source.len().saturating_mul(3)]);
    }
    for text in [r"\B[[a]]a\b", r"\Ba[[a]]\b"] {
        let plan = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
            &JsString::from(text),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let input = JsString::from("aaa!");
        assert_eq!(plan.find(&input, 0, false), Some(1..3));
        assert_eq!(plan.find(&input, 0, true), None);
        assert_eq!(plan.find(&input, 1, true), Some(1..3));
        let mut complete = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |amount| {
                complete.push(amount);
                Ok::<(), usize>(())
            }),
            Ok(Some(1..3))
        );
        assert_eq!(complete, [1]);
        let mut seen = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |amount| {
                seen.push(amount);
                Err(11)
            }),
            Err(11)
        );
        assert_eq!(seen, complete);
    }
    for (text, input, wanted) in [
        (r"\b[[a]]\$", "a$\n", Some(0..2)),
        (r"\b[[a]]\$\B$", "a$\n", None),
        (r"\b[[a]]\\\B$", "a\\", Some(0..2)),
        (r"\B\\[[a]]\b$", "\\a", Some(0..2)),
        (r"\B[[a--a]]a\b", "aaa!", None),
    ] {
        let plan = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
            &JsString::from(text),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            plan.find(&JsString::from(input), 0, false),
            wanted,
            "{text}"
        );
    }
    // One word gate needs eight extra neighbor passes; the two line gates
    // each retain one additional pass. These are caller precharges, not quotas.
    for (text, bare, literal_first, added) in [
        (r"\b[[a]]a", "[[a]]a", false, 8),
        (r"a[[a]]\b", "a[[a]]", true, 8),
        (r"^\b[[a]]a\b$", "[[a]]a", false, 18),
        (r"^\ba[[a]]\b$", "a[[a]]", true, 18),
    ] {
        let plan = RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
            &JsString::from(text),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let bare_source = JsString::from(bare);
        let bare_plan = if literal_first {
            RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
                &bare_source,
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
        } else {
            RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(
                &bare_source,
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
        }
        .unwrap()
        .unwrap();
        assert_eq!(plan.search_passes(), bare_plan.search_passes() + added);
    }
}
