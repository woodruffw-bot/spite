use super::*;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    NotK,
    CapitalWithoutLower,
    Any,
    Empty,
}

struct Case<'a> {
    source: &'a str,
    literal: u32,
    sets: [Set; 2],
    ignore_case: bool,
    multiline: bool,
    start_anchor: bool,
    end_anchor: bool,
    start_word: Option<bool>,
    end_word: Option<bool>,
}

// Independent finite facts only; production decoders/word predicates are not inputs.
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
    let canonical = if ignore_case { fold(point) } else { point };
    match set {
        Set::Point(value) => canonical == if ignore_case { fold(value) } else { value },
        Set::NotK => canonical != if ignore_case { 0x6b } else { 0x4b },
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
    // The initial inside-pair offset retains the complete leading boundary.
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
        let canonical = |p| if case.ignore_case { fold(p) } else { p };
        if canonical(points[at]) == canonical(case.literal)
            && contains(case.sets[0], points[at + 1], case.ignore_case)
            && contains(case.sets[1], points[at + 2], case.ignore_case)
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

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeLiteralClassPairWordMatcher> {
    RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), Infallible>(()),
    )
    .unwrap()
}

#[test]
fn literal_checked_pair_word_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, CapitalWithoutLower, Empty, NotK, Point};
    let cases = [
        Case {
            source: r"\Ba[[a]][[a]]\b",
            literal: 0x61,
            sets: [Point(0x61), Point(0x61)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(false),
            end_word: Some(true),
        },
        Case {
            source: r"\bK[[s]][[A]]\b",
            literal: 0x4b,
            sets: [Point(0x73), Point(0x41)],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\bc[\P{Ll}&&A][[s]]\b",
            literal: 0x63,
            sets: [CapitalWithoutLower, Point(0x73)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\bc[\P{Ll}&&A][[s]]\b",
            literal: 0x63,
            sets: [CapitalWithoutLower, Point(0x73)],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\B\u{1f600}[[\u{1f601}]][[\u{1f602}]]\B",
            literal: 0x1f600,
            sets: [Point(0x1f601), Point(0x1f602)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(false),
            end_word: Some(false),
        },
        Case {
            source: r"\b\uD800[[\uD800]][[\uD800]]\B",
            literal: 0xd800,
            sets: [Point(0xd800), Point(0xd800)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(false),
        },
        Case {
            source: r"^\ba[[\p{Any}]][[b]]\b$",
            literal: 0x61,
            sets: [Any, Point(0x62)],
            ignore_case: false,
            multiline: true,
            start_anchor: true,
            end_anchor: true,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"^\ba[[\p{Any}]][[b]]\b$",
            literal: 0x61,
            sets: [Any, Point(0x62)],
            ignore_case: false,
            multiline: false,
            start_anchor: true,
            end_anchor: true,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\ba[[s]][[\p{Any}]]\b",
            literal: 0x61,
            sets: [Point(0x73), Any],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\ba[[s]][[\p{Any}]]\b",
            literal: 0x61,
            sets: [Point(0x73), Any],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\ba[[^K]][[s]]\b",
            literal: 0x61,
            sets: [NotK, Point(0x73)],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\ba[a--a][[b]]\b",
            literal: 0x61,
            sets: [Empty, Point(0x62)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\Ba[[b]][b--b]\B",
            literal: 0x61,
            sets: [Point(0x62), Empty],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(false),
            end_word: Some(false),
        },
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("aaaa!"),
        JsString::from("xaaaa!"),
        JsString::from("cabx"),
        JsString::from("Kſa!"),
        JsString::from("asK!"),
        JsString::from("asſ!"),
        JsString::from("cAs"),
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
    // All8 canonical gate pairs x4 anchor frames x both actual i/m values.
    // Every cursor, including one past the end, and both sticky choices.
    for start_word in [None, Some(false), Some(true)] {
        for end_word in [None, Some(false), Some(true)] {
            if start_word.is_none() && end_word.is_none() {
                continue;
            }
            for start_anchor in [false, true] {
                for end_anchor in [false, true] {
                    for ignore_case in [false, true] {
                        for multiline in [false, true] {
                            let gate = |value| match value {
                                None => "",
                                Some(false) => r"\B",
                                Some(true) => r"\b",
                            };
                            let source = format!(
                                "{}{}a[[a]][[a]]{}{}",
                                if start_anchor { "^" } else { "" },
                                gate(start_word),
                                gate(end_word),
                                if end_anchor { "$" } else { "" }
                            );
                            let case = Case {
                                source: source.as_str(),
                                literal: 0x61,
                                sets: [Point(0x61); 2],
                                ignore_case,
                                multiline,
                                start_anchor,
                                end_anchor,
                                start_word,
                                end_word,
                            };
                            let matcher = compile(case.source, ignore_case, multiline).unwrap();
                            for text in ["aaaa!", "x\naaa\nx"] {
                                let input = JsString::from(text);
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        assert_eq!(
                                            matcher.find(&input, start, sticky),
                                            expected(&case, &input, start, sticky),
                                            "{source} i={ignore_case} m={multiline} input={input:?} start={start} sticky={sticky}"
                                        );
                                        queries += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(queries, 6982);
}

#[test]
fn literal_checked_pair_word_domains_full_proofs_and_work_prefixes() {
    for source in [
        "",
        r"\ba",
        r"\ba[[b]]",
        "a[[a]][[b]]",
        r"\b[[a]][[b]][[c]]",
        r"\bab[[a]][[b]]",
        r"\ba[b][[c]]",
        r"\ba[[b]][c]",
        r"\ba[b][c]",
        r"\ba[[b]][[c]][[d]]",
        r"\ba[[b]][[c]]d",
        r"\ba[[b]]+[[c]]",
        r"\ba[[b]][[c]]?",
        r"\ba^[[b]][[c]]",
        r"\ba[[b]]$[[c]]",
        r"\ba\b[[b]][[c]]",
        r"\ba[[b]]\B[[c]]",
        r"\b^a[[b]][[c]]",
        r"a[[b]][[c]]$\b",
        r"\b\ba[[b]][[c]]",
        r"\ba[[b]][[c]]\b\B",
        r"\ba([[b]])[[c]]",
        r"\ba[[b]]([[c]])",
        r"(\ba[[b]][[c]])",
        r"\b.a[[b]][[c]]",
        r"\b.[[b]][[c]]",
        r"\b\w[[b]][[c]]",
        r"\ba[[b]][[c]]\\b",
        r"\ba[[b]][[c]]\$",
        r"\ba[[\q{b}]][[c]]",
        r"\ba[a--a][[\q{c}]]",
        r"\ba[a--a][[\p{RGI_Emoji}]]",
    ] {
        assert!(compile(source, false, false).is_none(), "{source}");
    }
    // Each callback transcript is independently reconstructed from the literal
    // and two class compilers. Empty classes do not erase a later proof.
    for (source, i, anchored) in [
        (r"^\ba[[a]][[b]]\b$".to_owned(), false, true),
        (r"\bK[[K]][[s]]\b".to_owned(), true, false),
        (r"\ba[a--a][[b]]\b".to_owned(), false, false),
        (r"\Ba[[b]][b--b]\B".to_owned(), false, false),
        (
            format!(r"\B\u{{{}61}}[[a]][[b]]\b", "0".repeat(32)),
            false,
            false,
        ),
        (r"\B\[[[a]][[b]]\b".to_owned(), false, false),
    ] {
        let source = JsString::from(source.as_str());
        let mut observed = Vec::new();
        let matcher = RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
            &source,
            i,
            false,
            |n| {
                observed.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        let units = source.code_units();
        let body = &units[2 + usize::from(anchored)..units.len() - 2 - usize::from(anchored)];
        let mut literal_end = 1;
        unicode_literal_atom(body, &mut literal_end, body[0]).unwrap();
        let split = class_end(body, literal_end).unwrap();
        assert_eq!(class_end(body, split), Some(body.len()));
        let mut expected = vec![source.len().saturating_mul(4)];
        RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
            &body[..literal_end],
            i,
            false,
            |n| {
                expected.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        for part in [&body[literal_end..split], &body[split..]] {
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
            let result = RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
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
    // A valid but unsupported string property is rejected by the full second
    // class proof after the literal and empty first class have both compiled.
    let source = JsString::from(r"\ba[a--a][[\p{RGI_Emoji}]]\b");
    let units = source.code_units();
    let body = &units[2..units.len() - 2];
    let split = class_end(body, 1).unwrap();
    let mut observed = Vec::new();
    assert!(
        RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
            &source,
            false,
            false,
            |n| {
                observed.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .is_none()
    );
    let mut expected_work = vec![source.len().saturating_mul(4)];
    RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
        &body[..1],
        false,
        false,
        |n| {
            expected_work.push(n);
            Ok::<(), usize>(())
        },
    )
    .unwrap()
    .unwrap();
    RegExpUnicodeClassMatcher::compile_v_set_units_with_work(&body[1..split], false, false, |n| {
        expected_work.push(n);
        Ok::<(), usize>(())
    })
    .unwrap()
    .unwrap();
    assert!(
        RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &body[split..],
            false,
            false,
            |n| {
                expected_work.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(observed, expected_work);
    for fail in 1..=observed.len() {
        let mut seen = Vec::new();
        let result = RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
            &source,
            false,
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
    let q_source = JsString::from(r"\ba[a--a][[\q{b}]]\b");
    let mut q_work = Vec::new();
    assert!(
        RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
            &q_source,
            false,
            false,
            |n| {
                q_work.push(n);
                Ok::<(), Infallible>(())
            },
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(q_work, [q_source.len().saturating_mul(4)]);
    // A gate-only failure must retry the literal end, preserving the overlap.
    let matcher = compile(r"\Ba[[a]][[a]]\b", false, false).unwrap();
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
    for source in [
        r"\B\u{1f600}[[\u{1f601}]][[\u{1f602}]]\B",
        r"\B\uD83D\uDE00[[\u{1f601}]][[\u{1f602}]]\B",
        r"\B😀[[😁]][[😂]]\B",
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from("😀😁😂"), 1, true),
            Some(0..6)
        );
    }
    for (source, input) in [
        (r"\B\[[[a]][[b]]\b", "[ab!"),
        (r"\B\/[[a]][[b]]\b", "/ab!"),
        (r"\B\$[[a]][[b]]\b", "$ab!"),
        (r"\B\\[[a]][[b]]\b", "\\ab!"),
        (r"\B\[[[\/]][[\$]]\B$", "[/$"),
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from(input), 0, true),
            Some(0..3)
        );
    }
    for (source, input, range) in [
        (r"\bc[[a]][[b]]", "cabx", 0..3),
        (r"c[[a]][[b]]\b", "xcab", 1..4),
        (r"\Bc[[a]][[b]]", "xcabx", 1..4),
        (r"c[[a]][[b]]\B", "xcabx", 1..4),
        (r"^\ba[[b]][[c]]$", "x\r\nabc\u{2028}x", 3..6),
    ] {
        assert_eq!(
            compile(source, false, true)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            Some(range)
        );
    }
    // Literal/class membership stays true; only actual-i word classification changes.
    for (source, input) in [
        (r"\b\u212A[[s]][[A]]\b", "KsA!"),
        (r"\ba[[s]][[\p{Any}]]\b", "asK!"),
        (r"\ba[[s]][[\p{Any}]]\b", "asſ!"),
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            None
        );
        assert_eq!(
            compile(source, true, false)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            Some(0..3)
        );
    }
    assert_eq!(
        compile(r"\B\uD800[[\uD800]][[\uDC00]]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
                0,
                false
            ),
        None
    );
    // The old typed constructor still declines these new word-domain programs.
    for source in [r"\ba[[a]][[b]]", r"a[[a]][[b]]\b"] {
        assert!(
            RegExpUnicodeLiteralClassPairMatcher::compile_v_set_with_work(
                &JsString::from(source),
                false,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap()
            .is_none()
        );
    }
}
