use super::*;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    CapitalWithoutLower,
    Any,
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
        Set::CapitalWithoutLower => !ignore_case && point == 0x41,
        Set::Any => true,
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
        if contains(case.sets[0], points[at], case.ignore_case)
            && canonical(points[at + 1]) == canonical(case.literal)
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

// One concrete callback site for every new-wrapper proof and abort transcript.
// Each caller supplies a fresh recorder; no failed construction state is reused.
fn compile_recording_work(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassLiteralClassWordMatcher>, RegExpUnicodeSetCompileError<usize>>
{
    RegExpUnicodeClassLiteralClassWordMatcher::compile_v_set_with_work(source, i, m, |amount| {
        work.push(amount);
        if fail == Some(work.len()) {
            Err(work.len())
        } else {
            Ok(())
        }
    })
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassLiteralClassWordMatcher> {
    let mut work = Vec::new();
    compile_recording_work(&JsString::from(source), i, m, &mut work, None).unwrap()
}

#[test]
fn checked_literal_checked_word_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, CapitalWithoutLower, Point};
    let cases = [
        Case {
            source: r"\B[[a]]a[[a]]\b",
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
            source: r"\b[[K]]A[[s]]\b",
            literal: 0x41,
            sets: [Point(0x4b), Point(0x73)],
            ignore_case: true,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[\P{Ll}&&A]c[[s]]\b",
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
            source: r"\b[\P{Ll}&&A]c[[s]]\b",
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
            source: r"\B[[\u{1f600}]]\u{1f601}[[\u{1f602}]]\B",
            literal: 0x1f601,
            sets: [Point(0x1f600), Point(0x1f602)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(false),
            end_word: Some(false),
        },
        Case {
            source: r"\b[[\uD800]]\uD800[[\uD800]]\B",
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
            source: r"^\b[[a]]\u{1f600}[[\p{Any}]]\b$",
            literal: 0x1f600,
            sets: [Point(0x61), Any],
            ignore_case: false,
            multiline: true,
            start_anchor: true,
            end_anchor: true,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"^\b[[a]]\u{1f600}[[\p{Any}]]\b$",
            literal: 0x1f600,
            sets: [Point(0x61), Any],
            ignore_case: false,
            multiline: false,
            start_anchor: true,
            end_anchor: true,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[[\p{Any}]]A[[s]]\b",
            literal: 0x41,
            sets: [Any, Point(0x73)],
            ignore_case: false,
            multiline: false,
            start_anchor: false,
            end_anchor: false,
            start_word: Some(true),
            end_word: Some(true),
        },
        Case {
            source: r"\b[[\p{Any}]]A[[s]]\b",
            literal: 0x41,
            sets: [Any, Point(0x73)],
            ignore_case: true,
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
        JsString::from("Kaſ!"),
        JsString::from("KAs!"),
        JsString::from("ſAs!"),
        JsString::from("Acs"),
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
    // Two fixed inputs, every cursor (including one past end), both sticky choices.
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
                                "{}{}[[a]]a[[a]]{}{}",
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
    assert_eq!(queries, 6316);
}

#[test]
fn checked_literal_checked_word_domains_full_proofs_and_work_prefixes() {
    for source in [
        "",
        "[[a]]b[[c]]",
        r"\b[[a]]b",
        r"\b[[a]][[c]]",
        r"\ba[[b]][[c]]",
        r"\b[[a]][[b]]c",
        r"\b[a]b[[c]]",
        r"\b[[a]]b[c]",
        r"\b[a]b[c]",
        r"\b[[a]]bc[[d]]",
        r"\bx[[a]]b[[c]]",
        r"\b[[a]]b[[c]]d",
        r"\b[[a]]\bb[[c]]",
        r"\b[[a]]b\B[[c]]",
        r"\b[[a]]^[[c]]",
        r"\b[[a]]$[[c]]",
        r"\b[[a]]b[[c]]$\b",
        r"\b^[[a]]b[[c]]",
        r"\b\b[[a]]b[[c]]",
        r"[[a]]b[[c]]\b\B",
        r"\b([[a]])b[[c]]",
        r"\b[[a]](b)[[c]]",
        r"\b[[a]]b([[c]])",
        r"(\b[[a]]b[[c]])",
        r"\b[[a]]+b[[c]]",
        r"\b[[a]]b?[[c]]",
        r"\b[[a]]b[[c]]+",
        r"(?:\b[[a]]b[[c]]\b)+",
        r"\b[[a]].[[c]]",
        r"\b[[a]]\w[[c]]",
        r"\b[[a]]b[[c]]\\b",
        r"\b[a--a]b[[\q{c}]]",
        r"\b[[\q{a}]]b[c--c]",
        r"\b[a--a]b[[\p{RGI_Emoji}]]",
        r"\b[[\p{RGI_Emoji}]]b[c--c]",
    ] {
        assert!(compile(source, false, false).is_none(), "{source}");
    }
    // One concrete new-wrapper callback site serves every fresh construction.
    // The independent expected transcript follows first class, literal, last class.
    for (source, i, anchored, supported) in [
        (r"^\b[[a]]b[[a]]\b$".to_owned(), false, true, true),
        (r"\b[[K]]A[[s]]\b".to_owned(), true, false, true),
        (r"\b[a--a]b[[a]]\b".to_owned(), false, false, true),
        (r"\B[[a]]K[b--b]\b".to_owned(), true, false, true),
        (
            format!(r"\B[[a]]\u{{{}62}}[[a]]\b", "0".repeat(32)),
            false,
            false,
            true,
        ),
        (r"\B[[!]]\\[[!]]\B".to_owned(), false, false, true),
        (
            r"\b[a--a]b[[\p{RGI_Emoji}]]\b".to_owned(),
            false,
            false,
            false,
        ),
        (
            r"\b[[a]]b[[\p{RGI_Emoji}]]\b".to_owned(),
            false,
            false,
            false,
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut observed = Vec::new();
        let matcher = compile_recording_work(&source, i, false, &mut observed, None).unwrap();
        let units = source.code_units();
        let body = &units[2 + usize::from(anchored)..units.len() - 2 - usize::from(anchored)];
        let split = class_end(body, 0).unwrap();
        let mut literal_end = split + 1;
        unicode_literal_atom(body, &mut literal_end, body[split]).unwrap();
        assert_eq!(class_end(body, literal_end), Some(body.len()));
        let mut expected = vec![source.len().saturating_mul(4)];
        let mut component_supported = true;
        for (is_class, part) in [
            (true, &body[..split]),
            (false, &body[split..literal_end]),
            (true, &body[literal_end..]),
        ] {
            let proved = if is_class {
                RegExpUnicodeClassMatcher::compile_v_set_units_with_work(part, i, false, |n| {
                    expected.push(n);
                    Ok::<(), usize>(())
                })
                .unwrap()
                .is_some()
            } else {
                RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                    part,
                    i,
                    false,
                    |n| {
                        expected.push(n);
                        Ok::<(), usize>(())
                    },
                )
                .unwrap()
                .is_some()
            };
            if !proved {
                component_supported = false;
                break;
            }
        }
        assert_eq!(component_supported, supported);
        assert_eq!(matcher.is_some(), supported);
        assert_eq!(observed, expected);
        if let Some(matcher) = matcher {
            assert_eq!(
                matcher.search_passes(),
                matcher
                    .matcher
                    .search_passes()
                    .saturating_add(matcher.assertions.search_passes())
            );
        }
        for fail in 1..=observed.len() {
            let mut seen = Vec::new();
            let result = compile_recording_work(&source, i, false, &mut seen, Some(fail));
            assert!(matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail));
            assert_eq!(seen, observed[..fail]);
        }
    }
    // q fails during framing, unlike the late string-property component proof.
    let source = JsString::from(r"\b[a--a]b[[\q{c}]]\b");
    let mut work = Vec::new();
    assert!(
        compile_recording_work(&source, false, false, &mut work, None)
            .unwrap()
            .is_none()
    );
    assert_eq!(work, [source.len().saturating_mul(4)]);
    for source in [r"\b[a--a]b[[a]]\b", r"\B[[a]]b[a--a]\B"] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from("aba"), 0, false),
            None
        );
    }
    // A later component or word-gate failure retries the first class's end.
    let matcher = compile(r"\B[[a]]a[[a]]\b", false, false).unwrap();
    let input = JsString::from("aaaa!");
    let mut work = Vec::new();
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..4))
    );
    assert_eq!(work, [1]);
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |_| Err(31)),
        Err(31)
    );
    assert_eq!(matcher.find(&input, 0, true), None);
    assert_eq!(matcher.find(&input, 1, true), Some(1..4));
    for source in [r"[[a]]a[[a]]\b", r"\B[[a]]a[[a]]"] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&input, 0, false),
            Some(1..4)
        );
    }
    assert_eq!(
        compile(r"\B[[a]]b[[c]]\b", false, false)
            .unwrap()
            .find(&JsString::from("aabc!"), 0, false),
        Some(1..4)
    );
    assert_eq!(
        compile(r"\B[[a]]a[[b]]\b", false, false)
            .unwrap()
            .find(&JsString::from("aaab!"), 0, false),
        Some(1..4)
    );
    for source in [
        r"\B[[\u{1f600}]]\u{1f601}[[\u{1f602}]]\B",
        r"\B[[\u{1f600}]]\uD83D\uDE01[[\u{1f602}]]\B",
        r"\B[[😀]]😁[[😂]]\B",
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from("😀😁😂!"), 1, true),
            Some(0..6)
        );
    }
    assert_eq!(
        compile(r"\B[[\p{Any}]]\u{1f601}[[a]]\b", false, false)
            .unwrap()
            .find(&JsString::from("😀😁a!"), 1, true),
        Some(0..5)
    );
    assert_eq!(
        compile(r"\b[[\uD800]]\uD800[[\uD800]]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
                0,
                false
            ),
        Some(1..4)
    );
    assert_eq!(
        compile(r"\B[[\uD800]]\uD800[[\uDC00]]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
                0,
                false
            ),
        None
    );
    for (source, input) in [
        (r"\b[[a]]\[[[b]]\b", "a[b"),
        (r"\b[[a]]\/[[b]]\b", "a/b"),
        (r"\b[[a]]\$[[b]]\b", "a$b"),
        (r"^\b[[a]]\$[[b]]\b$", "a$b"),
        (r"\B[[!]]\\[[!]]\B", "!\\!"),
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from(input), 0, true),
            Some(0..3)
        );
    }
    for (source, input, range) in [
        (r"\b[[a]]b[[c]]", "abcx", 0..3),
        (r"[[a]]b[[c]]\b", "xabc", 1..4),
        (r"\B[[a]]b[[c]]", "xabcx", 1..4),
        (r"[[a]]b[[c]]\B", "xabcx", 1..4),
        (r"^\b[[a]]b[[c]]\b$", "x\r\nabc\u{2028}x", 3..6),
    ] {
        assert_eq!(
            compile(source, false, true)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            Some(range)
        );
    }
    // Fold-invariant Any isolates actual-i word classification at either edge.
    for (source, input) in [
        (r"\b[[\p{Any}]]A[[s]]\b", "KAs!"),
        (r"\b[[\p{Any}]]A[[s]]\b", "ſAs!"),
        (r"\b[[a]]s[[\p{Any}]]\b", "asK!"),
        (r"\b[[a]]s[[\p{Any}]]\b", "asſ!"),
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
        compile(r"\B[[a]]s[[^K]]\b", true, false).unwrap().find(
            &JsString::from("aasKaasſ!"),
            0,
            false
        ),
        Some(5..8)
    );
    // The old plain middle API retains its exact no-word domain.
    for source in [r"\b[[a]]b[[c]]", r"[[a]]b[[c]]\b"] {
        assert!(
            RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
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
