use super::*;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    CapitalWithoutLower,
    Any,
    Empty,
}

struct Case<'a> {
    source: &'a str,
    sets: [Set; 3],
    ignore_case: bool,
    multiline: bool,
    start_anchor: bool,
    end_anchor: bool,
    start_word: Option<bool>,
    end_word: Option<bool>,
}

// Independent finite facts, not production decoders or word predicates.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
}

fn word(point: Option<u32>, ignore_case: bool) -> bool {
    point.is_some_and(|point| {
        let point = if ignore_case { fold(point) } else { point };
        matches!(point, 0x41..=0x5a | 0x61..=0x7a | 0x30..=0x39 | 0x5f)
    })
}

fn contains(set: Set, point: u32, ignore_case: bool) -> bool {
    match set {
        Set::Point(value) => {
            let canonical = |p| if ignore_case { fold(p) } else { p };
            canonical(point) == canonical(value)
        }
        Set::CapitalWithoutLower => !ignore_case && point == 0x41,
        Set::Any => true,
        Set::Empty => false,
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
    let first = boundaries.partition_point(|&at| at <= start) - 1;
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    let boundary = |at: usize| {
        let before = at.checked_sub(1).and_then(|at| points.get(at)).copied();
        word(before, case.ignore_case) != word(points.get(at).copied(), case.ignore_case)
    };
    for at in first..points.len().saturating_sub(2) {
        if sticky && at != first {
            break;
        }
        let begin = boundaries[at];
        let end = boundaries[at + 3];
        if contains(case.sets[0], points[at], case.ignore_case)
            && contains(case.sets[1], points[at + 1], case.ignore_case)
            && contains(case.sets[2], points[at + 2], case.ignore_case)
            && (!case.start_anchor
                || begin == 0
                || (case.multiline && line(input.code_units()[begin - 1])))
            && (!case.end_anchor
                || end == input.len()
                || (case.multiline && line(input.code_units()[end])))
            && case.start_word.is_none_or(|value| boundary(at) == value)
            && case.end_word.is_none_or(|value| boundary(at + 3) == value)
        {
            return Some(begin..end);
        }
    }
    None
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassTripleWordMatcher> {
    RegExpUnicodeClassTripleWordMatcher::compile_v_set_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), Infallible>(()),
    )
    .unwrap()
}

#[test]
fn checked_class_triple_word_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, CapitalWithoutLower, Empty, Point};
    let cases = [
        Case {
            source: r"\B[[a]][[a]][[a]]\b",
            sets: [Point(0x61); 3],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(false),
            end_word: Some(true),
        },
        Case {
            source: r"\b[[K]][[s]][[A]]\b",
            sets: [Point(0x4b), Point(0x73), Point(0x41)],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[\P{Ll}&&A][[s]][[c]]\b",
            sets: [CapitalWithoutLower, Point(0x73), Point(0x63)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[\P{Ll}&&A][[s]][[c]]\b",
            sets: [CapitalWithoutLower, Point(0x73), Point(0x63)],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\B[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]\B",
            sets: [Point(0x1f600), Point(0x1f601), Point(0x1f602)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(false),
            end_word: Some(false),
        },
        Case {
            source: r"\b[[\uD800]][[\uD800]][[\uD800]]\B",
            sets: [Point(0xd800); 3],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(false),
        },
        Case {
            source: r"^\b[[a]][[\p{Any}]][[b]]\b$",
            sets: [Point(0x61), Any, Point(0x62)],
            ignore_case: false,
            multiline: true,
            start_anchor: true,
            end_anchor: true,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[[\p{Any}]][[s]][[A]]\b",
            sets: [Any, Point(0x73), Point(0x41)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[[\p{Any}]][[s]][[A]]\b",
            sets: [Any, Point(0x73), Point(0x41)],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[a--a][[b]][[a]]\b",
            sets: [Empty, Point(0x62), Point(0x61)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("aaaa!"),
        JsString::from("xaaaa!"),
        JsString::from("abcx"),
        JsString::from("Kſa!"),
        JsString::from("KsA!"),
        JsString::from("Asc!"),
        JsString::from("aαs"),
        JsString::from("aKs"),
        JsString::from("!😀😁😂!"),
        JsString::from("x😀😁😂!"),
        JsString::from("a😀b"),
        JsString::from("x\r\na😀b\u{2028}x"),
        JsString::from("aab\n"),
        JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
        JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
    ];
    let mut queries = 0;
    for case in &cases {
        let matcher = compile(case.source, case.ignore_case, case.multiline).unwrap();
        for input in &inputs {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(input, start, sticky),
                        expected(case, input, start, sticky),
                        "{} input={input:?} start={start} sticky={sticky}",
                        case.source
                    );
                    queries += 1;
                }
            }
        }
    }
    // Every canonical gate pair, with each optional outside-anchor frame.
    for start_word in [None, Some(false), Some(true)] {
        for end_word in [None, Some(false), Some(true)] {
            if start_word.is_none() && end_word.is_none() {
                continue;
            }
            for start_anchor in [false, true] {
                for end_anchor in [false, true] {
                    let gate = |value| match value {
                        None => "",
                        Some(false) => r"\B",
                        Some(true) => r"\b",
                    };
                    let source = format!(
                        "{}{}[[a]][[a]][[a]]{}{}",
                        if start_anchor { "^" } else { "" },
                        gate(start_word),
                        gate(end_word),
                        if end_anchor { "$" } else { "" }
                    );
                    let case = Case {
                        source: source.as_str(),
                        sets: [Point(0x61); 3],
                        ignore_case: false,
                        multiline: true,
                        start_anchor,
                        end_anchor,
                        start_word,
                        end_word,
                    };
                    let matcher = compile(case.source, false, true).unwrap();
                    for text in ["aaaa!", "aaa", "x\naaa\nx", "xaaa!"] {
                        let input = JsString::from(text);
                        for start in [0, 1] {
                            for sticky in [false, true] {
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
                                    expected(&case, &input, start, sticky),
                                    "{source} input={input:?} start={start} sticky={sticky}"
                                );
                                queries += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(queries, 2632);
}

#[test]
fn checked_class_triple_word_domains_full_proofs_and_work_prefixes() {
    for source in [
        "",
        "[[a]][[b]][[c]]",
        r"\b[[a]][[b]]",
        r"\b[a][[b]][[c]]",
        r"\b[[a]][b][[c]]",
        r"\b[[a]][[b]][c]",
        r"\b[[a]][[b]][[c]][[d]]",
        r"x\b[[a]][[b]][[c]]",
        r"\b[[a]][[b]][[c]]x",
        r"[[a]]\b[[b]][[c]]",
        r"[[a]][[b]]\B[[c]]",
        r"\b^[[a]][[b]][[c]]",
        r"[[a]][[b]][[c]]$\b",
        r"\b\b[[a]][[b]][[c]]",
        r"\b[[a]][[b]][[c]]\b\B",
        r"\b([[a]])[[b]][[c]]",
        r"\b[[a]]([[b]])[[c]]",
        r"\b[[a]][[b]]([[c]])",
        r"(\b[[a]][[b]][[c]])",
        r"\b[[a]]+[[b]][[c]]",
        r"\b[[a]][[b]]?[[c]]",
        r"\b[[a]][[b]][[c]]+",
        r"\b[[a]][[b]][[c]]\\b",
        r"\b[a--a][[b]][[\q{c}]]",
        r"\b[a--a][[b]][[\p{RGI_Emoji}]]",
    ] {
        assert!(compile(source, false, false).is_none(), "{source}");
    }
    for (source, i) in [
        (r"^\b[[a]][[a]][[b]]\b$".to_owned(), false),
        (r"\b[a--a][[b]][[a]]\b".to_owned(), false),
        (r"\B[[a]][b--b][[K]]\b".to_owned(), true),
        (r"\b[[K]][[s]][a--a]\B".to_owned(), true),
        (
            format!(r"\B[[a]][[\u{{{}61}}]][[b]]\b", "0".repeat(32)),
            false,
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut observed = Vec::new();
        let matcher =
            RegExpUnicodeClassTripleWordMatcher::compile_v_set_with_work(&source, i, false, |n| {
                observed.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .unwrap();
        let units = source.code_units();
        let begin = units.iter().position(|&u| u == 0x5b).unwrap();
        let first_end = class_end(units, begin).unwrap();
        let second_end = class_end(units, first_end).unwrap();
        let third_end = class_end(units, second_end).unwrap();
        let mut expected = vec![source.len().saturating_mul(4)];
        for part in [
            &units[begin..first_end],
            &units[first_end..second_end],
            &units[second_end..third_end],
        ] {
            RegExpUnicodeClassMatcher::compile_v_set_units_with_work(part, i, false, |n| {
                expected.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .unwrap();
        }
        assert_eq!(observed, expected);
        assert_eq!(
            matcher.search_passes(),
            matcher
                .matcher
                .search_passes()
                .saturating_add(matcher.assertions.search_passes())
        );
        for fail in 1..=observed.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassTripleWordMatcher::compile_v_set_with_work(
                &source,
                i,
                false,
                |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail));
            assert_eq!(seen, observed[..fail]);
        }
    }
    for source in [
        r"\b[a--a][[b]][[\p{RGI_Emoji}]]\b",
        r"\B[[a]][b--b][[\p{RGI_Emoji}]]\B",
    ] {
        let source = JsString::from(source);
        let units = source.code_units();
        let begin = units.iter().position(|&u| u == 0x5b).unwrap();
        let first_end = class_end(units, begin).unwrap();
        let second_end = class_end(units, first_end).unwrap();
        let third_end = class_end(units, second_end).unwrap();
        let mut observed = Vec::new();
        assert!(
            RegExpUnicodeClassTripleWordMatcher::compile_v_set_with_work(
                &source,
                false,
                false,
                |n| {
                    observed.push(n);
                    Ok::<(), Infallible>(())
                }
            )
            .unwrap()
            .is_none()
        );
        let mut expected = vec![source.len().saturating_mul(4)];
        for part in [&units[begin..first_end], &units[first_end..second_end]] {
            RegExpUnicodeClassMatcher::compile_v_set_units_with_work(part, false, false, |n| {
                expected.push(n);
                Ok::<(), Infallible>(())
            })
            .unwrap()
            .unwrap();
        }
        assert!(
            RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
                &units[second_end..third_end],
                false,
                false,
                |n| {
                    expected.push(n);
                    Ok::<(), Infallible>(())
                }
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(observed, expected);
    }
    let matcher = compile(r"\B[[a]][[a]][[a]]\b", false, false).unwrap();
    let input = JsString::from("aaaa!");
    let mut charges = Vec::new();
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |n| {
            charges.push(n);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..4))
    );
    assert_eq!(charges, [1]);
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |_| Err(31)),
        Err(31)
    );
    assert_eq!(matcher.find(&input, 0, true), None);
    assert_eq!(matcher.find(&input, 1, true), Some(1..4));
    let astral = compile(r"\B[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]]\B", false, false).unwrap();
    assert_eq!(astral.find(&JsString::from("😀😁😂"), 1, true), Some(0..6));
    for (source, input, range) in [
        (r"\b[[a]][[b]][[c]]", "abcx", 0..3),
        (r"[[a]][[b]][[c]]\b", "xabc", 1..4),
        (r"\B[[a]][[b]][[c]]", "xabcx", 1..4),
        (r"[[a]][[b]][[c]]\B", "xabcx", 1..4),
        (r"\b[[a]][[b]][[$]]\B$", "ab$", 0..3),
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            Some(range)
        );
    }
    assert_eq!(
        compile(r"\b[[\p{Any}]][[s]][[A]]\b", false, false)
            .unwrap()
            .find(&JsString::from("KsA!"), 0, false),
        None
    );
    assert_eq!(
        compile(r"\b[[\p{Any}]][[s]][[A]]\b", true, false)
            .unwrap()
            .find(&JsString::from("KsA!"), 0, false),
        Some(0..3)
    );
    assert_eq!(
        compile(r"\B[[\uD800]][[\uD800]][[\uDC00]]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
                0,
                false
            ),
        None
    );
}
