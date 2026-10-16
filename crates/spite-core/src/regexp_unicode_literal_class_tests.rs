use super::*;

#[derive(Clone, Copy)]
enum Class {
    A,
    UpperS,
    Empty,
    LowSurrogate,
    Any,
    AsciiWithoutB,
    NotA,
    Grinning,
}

struct Case<'a> {
    source: &'a str,
    literal: u32,
    class: Class,
    ignore_case: bool,
    multiline: bool,
    start_anchor: bool,
    end_anchor: bool,
}

// Independent facts for this finite alphabet, not production data or parsing.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
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
    // Existing approved Unicode start normalization checks the enclosing point.
    let first = boundaries.partition_point(|&at| at <= start) - 1;
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        let literal = if case.ignore_case {
            fold(points[at])
        } else {
            points[at]
        };
        let wanted = if case.ignore_case {
            fold(case.literal)
        } else {
            case.literal
        };
        let point = points[at + 1];
        let class_matches = match case.class {
            Class::A => {
                if case.ignore_case {
                    fold(point) == 0x61
                } else {
                    point == 0x61
                }
            }
            Class::UpperS => {
                if case.ignore_case {
                    fold(point) == 0x73
                } else {
                    point == 0x53
                }
            }
            Class::Empty => false,
            Class::LowSurrogate => point == 0xde00,
            Class::Any => true,
            Class::AsciiWithoutB => point <= 0x7f && point != 0x62,
            Class::NotA => point != 0x61,
            Class::Grinning => point == 0x1f601,
        };
        let begin = boundaries[at];
        let end = boundaries[at + 2];
        let start_matches = !case.start_anchor
            || begin == 0
            || (case.multiline && line(input.code_units()[begin - 1]));
        let end_matches = !case.end_anchor
            || end == input.len()
            || (case.multiline && line(input.code_units()[end]));
        if literal == wanted && class_matches && start_matches && end_matches {
            return Some(begin..end);
        }
    }
    None
}

#[test]
fn checked_literal_v_set_ranges_match_sparse_reference() {
    let cases = [
        Case {
            source: "c[a--b]",
            literal: 0x63,
            class: Class::A,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: "^c[[a]]$",
            literal: 0x63,
            class: Class::A,
            ignore_case: false,
            multiline: false,
            start_anchor: true,
            end_anchor: true,
        },
        Case {
            source: "^c[[a]]$",
            literal: 0x63,
            class: Class::A,
            ignore_case: false,
            multiline: true,
            start_anchor: true,
            end_anchor: true,
        },
        Case {
            source: "k[[S]]",
            literal: 0x6b,
            class: Class::UpperS,
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: "k[[S]]",
            literal: 0x6b,
            class: Class::UpperS,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: r"\[[[a]]",
            literal: 0x5b,
            class: Class::A,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: r"\$[[a]]$",
            literal: 0x24,
            class: Class::A,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: true,
        },
        Case {
            source: r"\u{1F600}[[\p{Any}]]",
            literal: 0x1f600,
            class: Class::Any,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: r"\uD83D\uDE00[[a]]",
            literal: 0x1f600,
            class: Class::A,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: r"\u{D83D}[[\uDE00]]",
            literal: 0xd83d,
            class: Class::LowSurrogate,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: r"\uD800[[a]]",
            literal: 0xd800,
            class: Class::A,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: "c[[a--a]]",
            literal: 0x63,
            class: Class::Empty,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: r"c[[\p{ASCII}]--[b]]",
            literal: 0x63,
            class: Class::AsciiWithoutB,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: "c[[^a]]",
            literal: 0x63,
            class: Class::NotA,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: "😀[[a]]",
            literal: 0x1f600,
            class: Class::A,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
        Case {
            source: r"\u{1F600}[[\u{1F601}]]",
            literal: 0x1f600,
            class: Class::Grinning,
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
        },
    ];
    let mut inputs: Vec<JsString> = [
        "",
        "c",
        "a",
        "ca",
        "ccca",
        "cac",
        "cbca",
        "ca\n",
        "\nca",
        "x\r\nca\u{2028}",
        "ca\rca",
        "ca\nca",
        "ca\u{2029}ca",
        "kS",
        "ks",
        "KS",
        "Kſ",
        "Kſ",
        "kſ",
        "[a",
        "$a",
        "$a\n",
        "😀a",
        "x😀a",
        "😀😁",
        "😀😀😁",
        "😀c",
        "c😀",
        "c$",
        "c[",
    ]
    .into_iter()
    .map(JsString::from)
    .collect();
    for units in [
        vec![0xd800, 0x61],
        vec![0xd800, 0xd800, 0x61],
        vec![0xd83d, 0xde00],
        vec![0xd83d, 0xde00, 0x61],
        vec![0x63, 0xd800],
        vec![0x63, 0xde00],
    ] {
        inputs.push(JsString::from_code_units(units));
    }
    let mut comparisons = 0;
    for case in cases {
        let source = JsString::from(case.source);
        let plan = RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
            &source,
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
                        expected(&case, input, start, sticky),
                        "source={:?} input={input:?} start={start} sticky={sticky}",
                        case.source
                    );
                    comparisons += 1;
                }
            }
        }
    }
    assert_eq!(comparisons, 5_568);
}

#[test]
fn checked_literal_v_set_domains_and_work_prefixes() {
    for source in [
        "",
        "ca",
        "c[a]",
        "[a--b]c",
        "cc[a--b]",
        "c[a--b]d",
        "c[a--b][d]",
        "c+[a--b]",
        "c[a--b]+",
        "c[a--b]{0}",
        "c([a--b])",
        "(c)[a--b]",
        "(c[a--b])",
        r"\bc[a--b]",
        r"c[a--b]\b",
        "c^[a--b]",
        "c$[a--b]",
        "c[a--b]$^",
        r"c[[a--a]&&[\q{ab}]]",
        r"c[[a--a]&&[\p{RGI_Emoji}]]",
    ] {
        assert!(
            RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
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
    for (text, ignore_case) in [
        (r"\u{1F600}[[\p{ASCII}]--[Q]]$", false),
        (r"^k[[S]&&[s]]$", true),
    ] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        assert!(
            RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
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
        assert!(!complete.is_empty());
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
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
            assert!(
                matches!(result, Err(RegExpUnicodeSetCompileError::Work(error)) if error == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
    }
    let source = JsString::from(format!(r"\u{{{}63}}[[a]]", "0".repeat(256)).as_str());
    let mut seen = Vec::new();
    assert!(matches!(
        RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
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
    assert_eq!(seen, [source.len()]);
    let plan = RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
        &JsString::from("c[a--b]"),
        false,
        false,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap();
    let input = JsString::from("ccca");
    let mut charges = Vec::new();
    assert_eq!(
        plan.find_with_work(&input, 0, false, |amount| {
            charges.push(amount);
            Ok::<(), usize>(())
        }),
        Ok(Some(2..4))
    );
    assert_eq!(charges, [1]);
    assert_eq!(plan.find_with_work(&input, 0, false, |_| Err(11)), Err(11));
}
