use super::*;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    Any,
    NotK,
    CapitalWithoutLower,
    Empty,
}

struct Case<'a> {
    source: &'a str,
    sets: &'a [Set],
    ignore_case: bool,
    multiline: bool,
    anchors: [bool; 2],
    words: [Option<bool>; 2],
}

struct OracleInput {
    input: JsString,
    points: Vec<u32>,
    boundaries: Vec<usize>,
}

impl OracleInput {
    fn new(input: JsString) -> Self {
        let mut points = Vec::new();
        let mut boundaries = vec![0];
        // Standard decoding, not a production matcher or character helper.
        for decoded in char::decode_utf16(input.code_units().iter().copied()) {
            let (point, width) = match decoded {
                Ok(character) => (u32::from(character), character.len_utf16()),
                Err(error) => (u32::from(error.unpaired_surrogate()), 1),
            };
            points.push(point);
            boundaries.push(boundaries.last().unwrap() + width);
        }
        Self {
            input,
            points,
            boundaries,
        }
    }
}

// Independent finite facts: ASCII common/simple folds, Kelvin and long s.
// Full/Turkic folds and production word/category data are not oracle inputs.
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
    let canonical = |point| if ignore_case { fold(point) } else { point };
    match set {
        Set::Point(value) => canonical(point) == canonical(value),
        Set::Any => true,
        Set::NotK => canonical(point) != canonical(0x4b),
        // v+i complements the folded Ll set, which contains folded A.
        Set::CapitalWithoutLower => !ignore_case && point == 0x41,
        Set::Empty => false,
    }
}

fn expected(
    case: &Case<'_>,
    input: &OracleInput,
    start: usize,
    sticky: bool,
) -> Option<Range<usize>> {
    if start > input.input.len() {
        return None;
    }
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    let boundary = |at: usize| {
        let before = at
            .checked_sub(1)
            .and_then(|at| input.points.get(at))
            .copied();
        word(before, case.ignore_case) != word(input.points.get(at).copied(), case.ignore_case)
    };
    for at in first..input.points.len() {
        if sticky && at != first {
            break;
        }
        let end_point = at + case.sets.len();
        let Some(points) = input.points.get(at..end_point) else {
            break;
        };
        let begin = input.boundaries[at];
        let end = input.boundaries[end_point];
        if case
            .sets
            .iter()
            .zip(points)
            .all(|(&set, &point)| contains(set, point, case.ignore_case))
            && (!case.anchors[0]
                || begin == 0
                || (case.multiline && line(input.input.code_units()[begin - 1])))
            && (!case.anchors[1]
                || end == input.input.len()
                || (case.multiline && line(input.input.code_units()[end])))
            && case.words[0].is_none_or(|value| boundary(at) == value)
            && case.words[1].is_none_or(|value| boundary(end_point) == value)
        {
            return Some(begin..end);
        }
    }
    None
}

// One concrete sequence compiler callback serves every fresh work transcript.
fn compile_recording_work(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassSequenceMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassSequenceMatcher::compile_v_set_with_work(source, i, m, |amount| {
        work.push(amount);
        if fail == Some(work.len()) {
            Err(work.len())
        } else {
            Ok(())
        }
    })
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassSequenceMatcher> {
    compile_recording_work(&JsString::from(source), i, m, &mut Vec::new(), None).unwrap()
}

fn check_case(case: &Case<'_>, inputs: &[OracleInput]) -> usize {
    let matcher = compile(case.source, case.ignore_case, case.multiline).unwrap();
    assert_eq!(
        matcher.max_match_code_units(),
        case.sets.len().saturating_mul(2)
    );
    let shared = matcher.clone();
    drop(matcher);
    let mut queries = 0;
    for input in inputs {
        for start in 0..=input.input.len() + 1 {
            for sticky in [false, true] {
                assert_eq!(
                    shared.find(&input.input, start, sticky),
                    expected(case, input, start, sticky),
                    "{} i={} m={} input={:?} start={start} sticky={sticky}",
                    case.source,
                    case.ignore_case,
                    case.multiline,
                    input.input
                );
                queries += 1;
            }
        }
    }
    queries
}

#[test]
fn checked_class_sequence_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, CapitalWithoutLower, Empty, NotK, Point};
    let cases = [
        Case {
            source: "[[a]][[a]][[a]][[b]]",
            sets: &[Point(0x61), Point(0x61), Point(0x61), Point(0x62)],
            ignore_case: false,
            multiline: false,
            anchors: [false; 2],
            words: [None; 2],
        },
        Case {
            source: r"\b[[K]][[s]][[a]][[a]][[b]]\b",
            sets: &[
                Point(0x4b),
                Point(0x73),
                Point(0x61),
                Point(0x61),
                Point(0x62),
            ],
            ignore_case: true,
            multiline: false,
            anchors: [false; 2],
            words: [Some(true); 2],
        },
        Case {
            source: r"\b[[K]][[s]][[a]][[a]][[b]]\b",
            sets: &[
                Point(0x4b),
                Point(0x73),
                Point(0x61),
                Point(0x61),
                Point(0x62),
            ],
            ignore_case: false,
            multiline: false,
            anchors: [false; 2],
            words: [Some(true); 2],
        },
        Case {
            source: r"[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]",
            sets: &[
                Point(0x1f600),
                Point(0x1f601),
                Point(0x1f602),
                Point(0x1f603),
                Point(0x1f604),
                Point(0x1f605),
            ],
            ignore_case: false,
            multiline: false,
            anchors: [false; 2],
            words: [None; 2],
        },
        Case {
            source: r"\b[[\uD800]][[\uD800]][[\uD800]][[\uD800]][[\uD800]]\B",
            sets: &[Point(0xd800); 5],
            ignore_case: false,
            multiline: false,
            anchors: [false; 2],
            words: [Some(true), Some(false)],
        },
        Case {
            source: r"\b[[\p{Any}]][[a]][[a]][[a]]\b",
            sets: &[Any, Point(0x61), Point(0x61), Point(0x61)],
            ignore_case: true,
            multiline: false,
            anchors: [false; 2],
            words: [Some(true); 2],
        },
        Case {
            source: r"\b[[\p{Any}]][[a]][[a]][[a]]\b",
            sets: &[Any, Point(0x61), Point(0x61), Point(0x61)],
            ignore_case: false,
            multiline: false,
            anchors: [false; 2],
            words: [Some(true); 2],
        },
        Case {
            source: r"[\P{Ll}&&A][[s]][[a]][[a]]",
            sets: &[CapitalWithoutLower, Point(0x73), Point(0x61), Point(0x61)],
            ignore_case: true,
            multiline: false,
            anchors: [false; 2],
            words: [None; 2],
        },
        Case {
            source: "[[a]][[^K]][[s]][[b]]",
            sets: &[Point(0x61), NotK, Point(0x73), Point(0x62)],
            ignore_case: true,
            multiline: false,
            anchors: [false; 2],
            words: [None; 2],
        },
        Case {
            source: "[a--a][[b]][[c]][[d]]",
            sets: &[Empty, Point(0x62), Point(0x63), Point(0x64)],
            ignore_case: false,
            multiline: false,
            anchors: [false; 2],
            words: [None; 2],
        },
        Case {
            source: r"^[[a]][[\p{Any}]][[b]][[c]]$",
            sets: &[Point(0x61), Any, Point(0x62), Point(0x63)],
            ignore_case: false,
            multiline: true,
            anchors: [true; 2],
            words: [None; 2],
        },
        Case {
            source: r"^[[a]][[\p{Any}]][[b]][[c]]$",
            sets: &[Point(0x61), Any, Point(0x62), Point(0x63)],
            ignore_case: false,
            multiline: false,
            anchors: [true; 2],
            words: [None; 2],
        },
        Case {
            source: r"\B[[a]][[a]][[a]][[a]]\b",
            sets: &[Point(0x61); 4],
            ignore_case: false,
            multiline: false,
            anchors: [false; 2],
            words: [Some(false), Some(true)],
        },
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("aaaab"),
        JsString::from("aaaaa!"),
        JsString::from("Kſaab!"),
        JsString::from("Ksaab"),
        JsString::from("Kaaa!"),
        JsString::from("aKsb"),
        JsString::from("aαsb"),
        JsString::from("😀😁😂😃😄😅"),
        JsString::from("x😀😁😂😃😄😅!"),
        JsString::from("x\r\na😀bc\u{2028}x"),
        JsString::from("aaaa"),
        JsString::from_code_units(vec![0x61, 0xd800, 0xd800, 0xd800, 0xd800, 0xd800, 0x21]),
        JsString::from_code_units(vec![
            0x61, 0xd800, 0xd800, 0xdc00, 0xd800, 0xd800, 0xd800, 0x21,
        ]),
    ]
    .map(OracleInput::new);
    let mut queries = 0;
    for case in &cases {
        queries += check_case(case, &inputs);
    }
    // Every canonical edge/anchor frame, including the plain zero-word shape.
    // The separate cases above distinguish actual i/m and longer term counts.
    let framed_inputs =
        [JsString::from("Kſaa"), JsString::from("x\nKſaa\nx")].map(OracleInput::new);
    for start_word in [None, Some(false), Some(true)] {
        for end_word in [None, Some(false), Some(true)] {
            for start_anchor in [false, true] {
                for end_anchor in [false, true] {
                    let gate = |value| match value {
                        None => "",
                        Some(false) => r"\B",
                        Some(true) => r"\b",
                    };
                    let source = format!(
                        "{}{}[[K]][[s]][[a]][[a]]{}{}",
                        if start_anchor { "^" } else { "" },
                        gate(start_word),
                        gate(end_word),
                        if end_anchor { "$" } else { "" }
                    );
                    let case = Case {
                        source: source.as_str(),
                        sets: &[Point(0x4b), Point(0x73), Point(0x61), Point(0x61)],
                        ignore_case: true,
                        multiline: true,
                        anchors: [start_anchor, end_anchor],
                        words: [start_word, end_word],
                    };
                    queries += check_case(&case, &framed_inputs);
                }
            }
        }
    }
    assert_eq!(queries, 4220);
}

fn record_class(source: &str, i: bool, work: &mut Vec<usize>) -> Option<RegExpUnicodeClassMatcher> {
    let source = JsString::from(source);
    RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
        source.code_units(),
        i,
        false,
        |amount| {
            work.push(amount);
            Ok::<(), usize>(())
        },
    )
    .unwrap()
}

#[test]
fn checked_class_sequence_domains_full_proofs_and_work_prefixes() {
    for source in [
        "",
        "[[a]]",
        "[[a]][[b]]",
        "[[a]][[b]][[c]]",
        "[a][[b]][[c]][[d]]",
        "[[a]][[b]][[c]][d]",
        "x[[a]][[b]][[c]][[d]]",
        "[[a]][[b]][[c]][[d]]x",
        r"[[a]]\b[[b]][[c]][[d]]",
        r"[[a]][[b]]^[[c]][[d]]",
        r"[[a]][[b]][[c]]$[[d]]",
        r"\b^[[a]][[b]][[c]][[d]]",
        r"[[a]][[b]][[c]][[d]]$\b",
        r"\b\b[[a]][[b]][[c]][[d]]",
        r"[[a]][[b]][[c]][[d]]\b\B",
        "^^[[a]][[b]][[c]][[d]]",
        "[[a]][[b]][[c]][[d]]$$",
        "([[a]])[[b]][[c]][[d]]",
        "([[a]][[b]][[c]][[d]])",
        "(?:[[a]][[b]])[[c]][[d]]",
        "[[a]]+[[b]][[c]][[d]]",
        "[[a]][[b]][[c]][[d]]?",
        "[[a]][[b]]|[[c]][[d]]",
        r"[a--a][[b]][[c]][[\q{d}]]",
        r"[a--a][[b]][[c]][[\p{RGI_Emoji}]]",
    ] {
        assert!(compile(source, false, false).is_none(), "{source}");
    }
    // Expected component order is specified independently, without class_end or
    // the new sequence's framing/counting loop. Six successes and two late gaps.
    for (source, parts, i, outside_passes, supported) in [
        (
            "[[a]][[b]][[c]][[d]]",
            &["[[a]]", "[[b]]", "[[c]]", "[[d]]"][..],
            false,
            0usize,
            true,
        ),
        (
            r"^\b[[K]][[s]][[A]][[a]][[b]]\b$",
            &["[[K]]", "[[s]]", "[[A]]", "[[a]]", "[[b]]"][..],
            true,
            18,
            true,
        ),
        (
            "[a--a][[b]][[c]][[d]]",
            &["[a--a]", "[[b]]", "[[c]]", "[[d]]"][..],
            false,
            0,
            true,
        ),
        (
            "[[a]][[b]][c--c][[d]][[e]]",
            &["[[a]]", "[[b]]", "[c--c]", "[[d]]", "[[e]]"][..],
            false,
            0,
            true,
        ),
        (
            "[[a]][[b]][[c]][[d]][[e]][[f]]",
            &["[[a]]", "[[b]]", "[[c]]", "[[d]]", "[[e]]", "[[f]]"][..],
            false,
            0,
            true,
        ),
        (
            r"\B[[!]][[a]][[b]][[!]]\B",
            &["[[!]]", "[[a]]", "[[b]]", "[[!]]"][..],
            false,
            16,
            true,
        ),
        (
            r"\B[a--a][[b]][[c]][[d]][[\p{RGI_Emoji}]]\B",
            &["[a--a]", "[[b]]", "[[c]]", "[[d]]", r"[[\p{RGI_Emoji}]]"][..],
            false,
            16,
            false,
        ),
        (
            r"[[a]][b--b][[c]][[\p{RGI_Emoji}]]",
            &["[[a]]", "[b--b]", "[[c]]", r"[[\p{RGI_Emoji}]]"][..],
            false,
            0,
            false,
        ),
    ] {
        let source = JsString::from(source);
        let mut observed = Vec::new();
        let matcher = compile_recording_work(&source, i, false, &mut observed, None).unwrap();
        let mut expected_work = vec![source.len().saturating_mul(4)];
        let mut expected_passes = parts.len().saturating_add(outside_passes);
        let mut component_supported = true;
        for part in parts {
            let Some(class) = record_class(part, i, &mut expected_work) else {
                component_supported = false;
                break;
            };
            expected_passes = expected_passes.saturating_add(class.search_passes());
        }
        assert_eq!(component_supported, supported);
        assert_eq!(matcher.is_some(), supported);
        assert_eq!(observed, expected_work);
        if let Some(matcher) = matcher {
            assert_eq!(matcher.search_passes(), expected_passes);
            assert_eq!(
                matcher.max_match_code_units(),
                parts.len().saturating_mul(2)
            );
        }
        for fail in 1..=observed.len() {
            let mut seen = Vec::new();
            assert!(matches!(
                compile_recording_work(&source, i, false, &mut seen, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail
            ));
            assert_eq!(seen, observed[..fail]);
        }
    }
    // q and a trailing atom both fail complete framing before any class proof.
    for source in [r"[a--a][[b]][[c]][[\q{d}]]", "[[a]][[b]][[c]][[d]]x"] {
        let source = JsString::from(source);
        let mut work = Vec::new();
        assert!(
            compile_recording_work(&source, false, false, &mut work, None)
                .unwrap()
                .is_none()
        );
        assert_eq!(work, [source.len().saturating_mul(4)]);
        assert!(matches!(
            compile_recording_work(&source, false, false, &mut Vec::new(), Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
    }
    let matcher = compile("[[a]][[a]][[a]][[b]]", false, false).unwrap();
    let input = JsString::from("aaaab");
    let mut work = Vec::new();
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..5))
    );
    assert_eq!(work, [1]);
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |_| Err(31)),
        Err(31)
    );
    assert_eq!(matcher.find(&input, 0, true), None);
    assert_eq!(matcher.find(&input, 1, true), Some(1..5));
    assert_eq!(
        compile(r"[[a]][[a]][[a]][[a]]\b", false, false)
            .unwrap()
            .find(&JsString::from("aaaaa!"), 0, false),
        Some(1..5)
    );
    let astral = compile(
        r"[[\u{1f600}]][[\u{1f601}]][[\u{1f602}]][[\u{1f603}]][[\u{1f604}]][[\u{1f605}]]",
        false,
        false,
    )
    .unwrap();
    assert_eq!(astral.max_match_code_units(), 12);
    assert_eq!(
        astral.find(&JsString::from("😀😁😂😃😄😅"), 1, true),
        Some(0..12)
    );
    // A larger finite witness proves the new type is not another small-N cap.
    let source = "[[a]]".repeat(65);
    let many = compile(source.as_str(), false, false).unwrap();
    let input = JsString::from(format!("x{}!", "a".repeat(65)).as_str());
    assert_eq!(many.max_match_code_units(), 130);
    assert_eq!(many.find(&input, 0, false), Some(1..66));
    assert_eq!(many.find(&input, 0, true), None);
    assert_eq!(many.find(&input, 1, true), Some(1..66));
    // Existing public typed domains stay closed; Runtime admission uses the new type.
    let source = JsString::from("[[a]][[b]][[c]][[d]]");
    assert!(
        crate::RegExpUnicodeClassTripleMatcher::compile_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), usize>(())
        )
        .unwrap()
        .is_none()
    );
    let source = JsString::from(r"\b[[a]][[b]][[c]][[d]]\b");
    assert!(
        crate::RegExpUnicodeClassTripleWordMatcher::compile_v_set_with_work(
            &source,
            false,
            false,
            |_| Ok::<(), usize>(())
        )
        .unwrap()
        .is_none()
    );
}
