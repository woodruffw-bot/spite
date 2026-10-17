use super::*;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    Any,
    NotK,
    NotLower,
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
        // Exact Ll facts for this finite input set; vi tests folded membership.
        Set::NotLower => !matches!(canonical(point), 0x61..=0x7a | 0x3b1 | 0x17f),
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
    RegExpUnicodeClassSequenceMatcher::compile_mixed_v_set_with_work(source, i, m, |amount| {
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
fn mixed_class_sequence_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, Empty, NotK, NotLower, Point};
    let cases = [
        Case {
            source: r"[a][[a]][b]",
            sets: &[Point(0x61), Point(0x61), Point(0x62)],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"[[a]][a][b]",
            sets: &[Point(0x61), Point(0x61), Point(0x62)],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"[a][a][[b]]",
            sets: &[Point(0x61), Point(0x61), Point(0x62)],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"\b[K][[s]][a][a][b]\b",
            sets: &[
                Point(0x4b),
                Point(0x73),
                Point(0x61),
                Point(0x61),
                Point(0x62),
            ],
            ignore_case: true,
            multiline: false,
            anchors: [false, false],
            words: [Some(true), Some(true)],
        },
        Case {
            source: r"\b[K][[s]][a][a][b]\b",
            sets: &[
                Point(0x4b),
                Point(0x73),
                Point(0x61),
                Point(0x61),
                Point(0x62),
            ],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [Some(true), Some(true)],
        },
        Case {
            source: r"[\p{Any}][[\u{1f601}]][\u{1f602}][[\u{1f603}]][\u{1f604}]",
            sets: &[
                Any,
                Point(0x1f601),
                Point(0x1f602),
                Point(0x1f603),
                Point(0x1f604),
            ],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"\b[\uD800][[\uD800]][\uD800][\uD800]\B",
            sets: &[Point(0xd800), Point(0xd800), Point(0xd800), Point(0xd800)],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [Some(true), Some(false)],
        },
        Case {
            source: r"\b[\p{Any}][[a]][a][a]\b",
            sets: &[Any, Point(0x61), Point(0x61), Point(0x61)],
            ignore_case: true,
            multiline: false,
            anchors: [false, false],
            words: [Some(true), Some(true)],
        },
        Case {
            source: r"\b[\p{Any}][[a]][a][a]\b",
            sets: &[Any, Point(0x61), Point(0x61), Point(0x61)],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [Some(true), Some(true)],
        },
        Case {
            source: r"[\P{Ll}][[s]][a][a]",
            sets: &[NotLower, Point(0x73), Point(0x61), Point(0x61)],
            ignore_case: true,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"[\P{Ll}][[s]][a][a]",
            sets: &[NotLower, Point(0x73), Point(0x61), Point(0x61)],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"[a][[^K]][s][b]",
            sets: &[Point(0x61), NotK, Point(0x73), Point(0x62)],
            ignore_case: true,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"[][[b]][c]",
            sets: &[Empty, Point(0x62), Point(0x63)],
            ignore_case: false,
            multiline: false,
            anchors: [false, false],
            words: [None, None],
        },
        Case {
            source: r"^[a][[\p{Any}]][b][c]$",
            sets: &[Point(0x61), Any, Point(0x62), Point(0x63)],
            ignore_case: false,
            multiline: true,
            anchors: [true, true],
            words: [None, None],
        },
        Case {
            source: r"^[a][[\p{Any}]][b][c]$",
            sets: &[Point(0x61), Any, Point(0x62), Point(0x63)],
            ignore_case: false,
            multiline: false,
            anchors: [true, true],
            words: [None, None],
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
    let framed_inputs = [JsString::from("Kſa"), JsString::from("x\nKſa\nx")].map(OracleInput::new);
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
                        "{}{}[K][[s]][a]{}{}",
                        if start_anchor { "^" } else { "" },
                        gate(start_word),
                        gate(end_word),
                        if end_anchor { "$" } else { "" }
                    );
                    let case = Case {
                        source: source.as_str(),
                        sets: &[Point(0x4b), Point(0x73), Point(0x61)],
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
    assert_eq!(queries, 4548);
}

// Expected proof order comes from explicit component lists below, not class_end
// or the mixed constructor. Every checked attempt retains its failed-flat work.
fn record_component(
    source: &str,
    i: bool,
    work: &mut Vec<usize>,
) -> (Option<RegExpUnicodeClassMatcher>, bool) {
    let source = JsString::from(source);
    let flat = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
        source.code_units(),
        i,
        false,
        |amount| {
            work.push(amount);
            Ok::<(), usize>(())
        },
    )
    .unwrap();
    if flat.is_some() {
        return (flat, false);
    }
    let checked = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
        source.code_units(),
        i,
        false,
        |amount| {
            work.push(amount);
            Ok::<(), usize>(())
        },
    )
    .unwrap();
    (checked, true)
}

fn no_work(_: usize) -> Result<(), usize> {
    Ok(())
}

#[test]
fn mixed_class_sequence_domains_full_proofs_and_work_prefixes() {
    for source in [
        "",
        "[a]",
        "[[a]]",
        "[a][[b]]",
        "[[a]][b]",
        "[a][b][c]",
        "[a][b][c][d]",
        "[[a]][[b]][[c]]",
        "[[a]][[b]][[c]][[d]]",
        r"\b[a][b][c]\b",
        r"\b[[a]][[b]][[c]]\b",
        "x[a][[b]][c]",
        "[a][[b]][c]x",
        "[a]x[[b]][c]",
        r"[a]\b[[b]][c]",
        r"[a]^[[b]][c]",
        r"[a][[b]]$[c]",
        r"\b^[a][[b]][c]",
        r"[a][[b]][c]$\b",
        r"\b\b[a][[b]][c]",
        r"[a][[b]][c]\b\B",
        "^^[a][[b]][c]",
        "[a][[b]][c]$$",
        "([a])[[b]][c]",
        "([a][[b]][c])",
        "(?:[a][[b]])[c]",
        "[a]+[[b]][c]",
        "[a][[b]][c]?",
        "[a][[b]]|[c][d]",
        r"[][[b]][\q{c}]",
        r"[][[b]][\p{RGI_Emoji}]",
        r"[[a--a]][b][\q{c}]",
        r"[[a--a]][b][\p{RGI_Emoji}]",
    ] {
        assert!(compile(source, false, false).is_none(), "{source}");
    }
    // Six mixed successes, two late string gaps and both homogeneous rejections.
    // A true flag means flat returned None and checked must be attempted; it
    // records a successful checked kind only if that second proof succeeds.
    for (source, parts, i, outside_passes, supported) in [
        (
            "[a][[b]][c]",
            &[("[a]", false), ("[[b]]", true), ("[c]", false)][..],
            false,
            0usize,
            true,
        ),
        (
            r"^\b[[K]][s][[A]][b]\b$",
            &[
                ("[[K]]", true),
                ("[s]", false),
                ("[[A]]", true),
                ("[b]", false),
            ][..],
            true,
            18,
            true,
        ),
        (
            "[][[b]][c]",
            &[("[]", false), ("[[b]]", true), ("[c]", false)][..],
            false,
            0,
            true,
        ),
        (
            "[[a--a]][b][c]",
            &[("[[a--a]]", true), ("[b]", false), ("[c]", false)][..],
            false,
            0,
            true,
        ),
        (
            "[a][[b]][c][[d]][e]",
            &[
                ("[a]", false),
                ("[[b]]", true),
                ("[c]", false),
                ("[[d]]", true),
                ("[e]", false),
            ][..],
            false,
            0,
            true,
        ),
        (
            r"\B[!][[a]][!]\B",
            &[("[!]", false), ("[[a]]", true), ("[!]", false)][..],
            false,
            16,
            true,
        ),
        (
            r"\B[][[b]][c][\p{RGI_Emoji}]\B",
            &[
                ("[]", false),
                ("[[b]]", true),
                ("[c]", false),
                (r"[\p{RGI_Emoji}]", true),
            ][..],
            false,
            16,
            false,
        ),
        (
            r"[[a--a]][b][\p{RGI_Emoji}]",
            &[
                ("[[a--a]]", true),
                ("[b]", false),
                (r"[\p{RGI_Emoji}]", true),
            ][..],
            false,
            0,
            false,
        ),
        (
            "[a][b][c]",
            &[("[a]", false), ("[b]", false), ("[c]", false)][..],
            false,
            0,
            false,
        ),
        (
            "[[a]][[b]][[c]]",
            &[("[[a]]", true), ("[[b]]", true), ("[[c]]", true)][..],
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
        let mut complete = true;
        let mut flat_kind = false;
        let mut checked_kind = false;
        for &(part, expected_checked_attempt) in parts {
            let (class, checked_attempted) = record_component(part, i, &mut expected_work);
            assert_eq!(checked_attempted, expected_checked_attempt, "{part}");
            let Some(class) = class else {
                complete = false;
                break;
            };
            flat_kind |= !checked_attempted;
            checked_kind |= checked_attempted;
            expected_passes = expected_passes.saturating_add(class.search_passes());
        }
        assert_eq!(complete && flat_kind && checked_kind, supported);
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
    // Complete framing rejects q or extra terms before either compiler runs.
    for source in [r"[][[b]][\q{c}]", "[a][[b]][c]x"] {
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
    for source in ["[][[b]][c]", "[[a--a]][b][c]"] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from("abc"), 0, false),
            None
        );
    }
    let matcher = compile("[a][[a]][b]", false, false).unwrap();
    let input = JsString::from("aaab");
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
    assert_eq!(
        compile(r"[a][[a]][a]\b", false, false)
            .unwrap()
            .find(&JsString::from("aaaa!"), 0, false),
        Some(1..4)
    );
    let astral = compile(
        r"[\u{1f600}][[\u{1f601}]][\u{1f602}][[\u{1f603}]][\u{1f604}]",
        false,
        false,
    )
    .unwrap();
    assert_eq!(astral.max_match_code_units(), 10);
    assert_eq!(
        astral.find(&JsString::from("😀😁😂😃😄!"), 1, true),
        Some(0..10)
    );
    for count in [17usize, 65] {
        let source = format!("[[a]]{}", "[a]".repeat(count - 1));
        let many = compile(source.as_str(), false, false).unwrap();
        let input = JsString::from(format!("x{}!", "a".repeat(count)).as_str());
        assert_eq!(many.max_match_code_units(), count.saturating_mul(2));
        assert_eq!(many.find(&input, 0, false), Some(1..count + 1));
        assert_eq!(many.find(&input, 0, true), None);
        assert_eq!(many.find(&input, 1, true), Some(1..count + 1));
    }
    // Older homogeneous and fixed-pair entry points retain their closed domains.
    let source = JsString::from("[a][[b]][c]");
    assert!(
        RegExpUnicodeClassSequenceMatcher::compile_v_set_with_work(&source, false, false, no_work)
            .unwrap()
            .is_none()
    );
    assert!(
        RegExpUnicodeClassSequenceMatcher::compile_flat_v_set_with_work(
            &source, false, false, no_work
        )
        .unwrap()
        .is_none()
    );
    assert!(
        crate::RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_work(
            &source, false, false, no_work
        )
        .unwrap()
        .is_none()
    );
    let source = JsString::from(r"\b[a][[b]][c]\b");
    assert!(crate::RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_word_boundaries_and_work(&source, false, false, no_work).unwrap().is_none());
}
