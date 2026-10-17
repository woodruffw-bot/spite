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
    set: Set,
    literal: u32,
    ignore_case: bool,
    multiline: bool,
    anchors: [bool; 2],
    words: [Option<bool>; 2],
}

struct Input {
    text: JsString,
    points: Vec<u32>,
    boundaries: Vec<usize>,
}

impl Input {
    fn new(text: JsString) -> Self {
        let mut points = Vec::new();
        let mut boundaries = vec![0];
        // Decode original characters independently of all matcher helpers.
        for decoded in char::decode_utf16(text.code_units().iter().copied()) {
            let (point, width) = match decoded {
                Ok(c) => (u32::from(c), c.len_utf16()),
                Err(e) => (u32::from(e.unpaired_surrogate()), 1),
            };
            points.push(point);
            boundaries.push(boundaries.last().unwrap() + width);
        }
        Self {
            text,
            points,
            boundaries,
        }
    }
}

// Complete simple/common-fold and category facts for this selected alphabet.
// No production data, membership helper, boundary helper, or oracle is reused.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
}

fn word(point: u32, i: bool) -> bool {
    matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
        || (i && matches!(point, 0x17f | 0x212a))
}

fn expected(case: &Case<'_>, input: &Input, start: usize, sticky: bool) -> Option<Range<usize>> {
    if start > input.text.len() {
        return None;
    }
    // A supplied initial low-pair index selects its enclosing original point.
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let canonical = |p| if case.ignore_case { fold(p) } else { p };
    let boundary = |at: usize| {
        let before = at
            .checked_sub(1)
            .and_then(|at| input.points.get(at))
            .is_some_and(|&p| word(p, case.ignore_case));
        let after = input
            .points
            .get(at)
            .is_some_and(|&p| word(p, case.ignore_case));
        before != after
    };
    let line = |u| matches!(u, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..input.points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        let p = input.points[at];
        let member = match case.set {
            Set::Point(value) => canonical(p) == canonical(value),
            Set::Any => true,
            Set::NotK => canonical(p) != canonical(0x4b),
            Set::NotLower if case.ignore_case => !matches!(fold(p), 0x61..=0x7a | 0x3b1),
            Set::NotLower => !matches!(p, 0x61..=0x7a | 0x17f | 0x3b1),
            Set::Empty => false,
        };
        let begin = input.boundaries[at];
        let end = input.boundaries[at + 2];
        if member
            && canonical(input.points[at + 1]) == canonical(case.literal)
            && (!case.anchors[0]
                || begin == 0
                || (case.multiline && line(input.text.code_units()[begin - 1])))
            && (!case.anchors[1]
                || end == input.text.len()
                || (case.multiline && line(input.text.code_units()[end])))
            && case.words[0].is_none_or(|wanted| boundary(at) == wanted)
            && case.words[1].is_none_or(|wanted| boundary(at + 2) == wanted)
        {
            return Some(begin..end);
        }
    }
    None
}

// One concrete callback callsite for all new compiler transcripts/abort prefixes.
fn recording(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassLiteralMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_word_boundaries_and_work(
        source,
        i,
        m,
        |amount| {
            work.push(amount);
            if fail == Some(work.len()) {
                Err(work.len())
            } else {
                Ok(())
            }
        },
    )
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassLiteralMatcher> {
    recording(&JsString::from(source), i, m, &mut Vec::new(), None).unwrap()
}

fn check_case(case: &Case<'_>, inputs: &[Input]) -> usize {
    let original = compile(case.source, case.ignore_case, case.multiline).unwrap();
    let matcher = original.clone();
    drop(original);
    let mut queries = 0;
    for input in inputs {
        for start in 0..=input.text.len() + 1 {
            for sticky in [false, true] {
                assert_eq!(
                    matcher.find(&input.text, start, sticky),
                    expected(case, input, start, sticky),
                    "{} i={} m={} input={:?} start={start} sticky={sticky}",
                    case.source,
                    case.ignore_case,
                    case.multiline,
                    input.text
                );
                queries += 1;
            }
        }
    }
    queries
}

#[test]
fn flat_class_literal_word_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, Empty, NotK, NotLower, Point};
    let inputs = [
        JsString::from(""),
        JsString::from("aaa!"),
        JsString::from("abcx !ab xab abx"),
        JsString::from("Kſ Ks ks"),
        JsString::from("Ka!ſa!αa!"),
        JsString::from("😀😁!"),
        JsString::from("x😀😁a"),
        JsString::from_code_units(vec![0xd800, 0x61, 0x21, 0x61, 0xd800, 0x21, 0x61, 0xdc00]),
        JsString::from_code_units(vec![0xd83d, 0xde00, 0xdc00, 0x61]),
        JsString::from("a$\na\\"),
        JsString::from("a\r\nab\u{2028}AB"),
        JsString::from("0a _a !a!"),
    ]
    .map(Input::new);
    let mut queries = 0;
    for (source, set, literal, i, words) in [
        (
            r"\B[a]a\b",
            Point(0x61),
            0x61,
            false,
            [Some(false), Some(true)],
        ),
        (r"\b[a]b", Point(0x61), 0x62, false, [Some(true), None]),
        (r"[a]b\b", Point(0x61), 0x62, false, [None, Some(true)]),
        (
            r"\b[K]s\b",
            Point(0x4b),
            0x73,
            true,
            [Some(true), Some(true)],
        ),
        (
            r"\b[K]s\b",
            Point(0x4b),
            0x73,
            false,
            [Some(true), Some(true)],
        ),
        (r"\B[^K]s\b", NotK, 0x73, true, [Some(false), Some(true)]),
        (
            r"\B[\p{Any}]a\b",
            Any,
            0x61,
            true,
            [Some(false), Some(true)],
        ),
        (
            r"\B[\p{Any}]a\b",
            Any,
            0x61,
            false,
            [Some(false), Some(true)],
        ),
        (r"[\P{Ll}]a\b", NotLower, 0x61, true, [None, Some(true)]),
        (r"[\P{Ll}]a\b", NotLower, 0x61, false, [None, Some(true)]),
        (r"\B[]a\b", Empty, 0x61, true, [Some(false), Some(true)]),
        (
            r"\B[\u{1f600}]\u{1f601}\B",
            Point(0x1f600),
            0x1f601,
            false,
            [Some(false), Some(false)],
        ),
        (
            r"\B[\uD800]a\b",
            Point(0xd800),
            0x61,
            false,
            [Some(false), Some(true)],
        ),
        (
            r"\b[a]\uD800\B",
            Point(0x61),
            0xd800,
            false,
            [Some(true), Some(false)],
        ),
        (
            r"\B[\uDC00]a\b",
            Point(0xdc00),
            0x61,
            false,
            [Some(false), Some(true)],
        ),
    ] {
        queries += check_case(
            &Case {
                source,
                set,
                literal,
                ignore_case: i,
                multiline: false,
                anchors: [false, false],
                words,
            },
            &inputs,
        );
    }
    // All eight nonempty outside-word combinations, four line frames and i/m.
    // Only three short inputs are used for this orthogonal gate/line check.
    let frame_inputs = ["ab", "x\r\nAB\u{2028}ab", "!abx"].map(|s| Input::new(JsString::from(s)));
    for (left, right, words) in [
        (r"\b", "", [Some(true), None]),
        (r"\B", "", [Some(false), None]),
        ("", r"\b", [None, Some(true)]),
        ("", r"\B", [None, Some(false)]),
        (r"\b", r"\b", [Some(true), Some(true)]),
        (r"\b", r"\B", [Some(true), Some(false)]),
        (r"\B", r"\b", [Some(false), Some(true)]),
        (r"\B", r"\B", [Some(false), Some(false)]),
    ] {
        for (prefix, suffix, anchors) in [
            ("", "", [false, false]),
            ("^", "", [true, false]),
            ("", "$", [false, true]),
            ("^", "$", [true, true]),
        ] {
            let source = format!("{prefix}{left}[a]b{right}{suffix}");
            for i in [false, true] {
                for m in [false, true] {
                    queries += check_case(
                        &Case {
                            source: &source,
                            set: Point(0x61),
                            literal: 0x62,
                            ignore_case: i,
                            multiline: m,
                            anchors,
                            words,
                        },
                        &frame_inputs,
                    );
                }
            }
        }
    }
    // Count every selected UTF-16 start (including one beyond the end), both search modes.
    let semantic_per_case = inputs.iter().map(|s| 2 * (s.text.len() + 2)).sum::<usize>();
    let frame_per_case = frame_inputs
        .iter()
        .map(|s| 2 * (s.text.len() + 2))
        .sum::<usize>();
    assert_eq!(
        queries,
        15 * semantic_per_case + 8 * 4 * 2 * 2 * frame_per_case
    );
    assert_eq!(queries, 8_300);
}

// Explicit bare component slices supply the callback oracle. No framing or
// matching implementation is reused; the old component compilers define their
// own established work sequence. Outside gates belong only to OutsideAssertions.
fn expected_work(
    source: &JsString,
    class: &str,
    literal: &str,
    i: bool,
    m: bool,
    words: [Option<bool>; 2],
    anchors: [bool; 2],
) -> (Vec<usize>, Option<usize>) {
    let mut work = vec![source.len().saturating_mul(3)];
    let class_source = JsString::from(class);
    let literal_source = JsString::from(literal);
    let Some(class) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
        class_source.code_units(),
        i,
        m,
        |n| {
            work.push(n);
            Ok::<(), usize>(())
        },
    )
    .unwrap() else {
        return (work, None);
    };
    let literal = RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
        literal_source.code_units(),
        i,
        m,
        |n| {
            work.push(n);
            Ok::<(), usize>(())
        },
    )
    .unwrap();
    let passes = literal.map(|literal| {
        class
            .search_passes()
            .saturating_add(literal.search_passes())
            .saturating_add(2)
            .saturating_add(
                8 * (usize::from(words[0].is_some()) + usize::from(words[1].is_some()))
                    + usize::from(anchors[0])
                    + usize::from(anchors[1]),
            )
    });
    (work, passes)
}

#[test]
fn flat_class_literal_word_domains_full_proofs_and_work_prefixes() {
    let long_literal = format!(r"\u{{{}61}}", "0".repeat(64));
    let long_source = format!(r"\B[]{long_literal}\b");
    for (text, class, literal, i, m, words, anchors, supported) in [
        (
            r"\b[a]b",
            "[a]",
            "b",
            false,
            false,
            [Some(true), None],
            [false, false],
            true,
        ),
        (
            r"^\b[K]ſ\b$",
            "[K]",
            "ſ",
            true,
            true,
            [Some(true), Some(true)],
            [true, true],
            true,
        ),
        (
            r"\B[\p{Any}]\uD83D\uDE01\B",
            r"[\p{Any}]",
            r"\uD83D\uDE01",
            true,
            false,
            [Some(false), Some(false)],
            [false, false],
            true,
        ),
        (
            r"\B[\uD800]a\b",
            r"[\uD800]",
            "a",
            false,
            false,
            [Some(false), Some(true)],
            [false, false],
            true,
        ),
        (
            long_source.as_str(),
            "[]",
            long_literal.as_str(),
            true,
            false,
            [Some(false), Some(true)],
            [false, false],
            true,
        ),
        (
            r"[\P{Ll}]a\b",
            r"[\P{Ll}]",
            "a",
            true,
            false,
            [None, Some(true)],
            [false, false],
            true,
        ),
        (
            r"\b[[a]]b",
            "[[a]]",
            "b",
            false,
            false,
            [Some(true), None],
            [false, false],
            false,
        ),
        (
            r"\b[a--a]b",
            "[a--a]",
            "b",
            false,
            false,
            [Some(true), None],
            [false, false],
            false,
        ),
        (
            r"\b[\p{RGI_Emoji}]a",
            r"[\p{RGI_Emoji}]",
            "a",
            false,
            false,
            [Some(true), None],
            [false, false],
            false,
        ),
    ] {
        let source = JsString::from(text);
        let (expected, passes) = expected_work(&source, class, literal, i, m, words, anchors);
        assert_eq!(passes.is_some(), supported, "{text}");
        let mut work = Vec::new();
        let plan = recording(&source, i, m, &mut work, None).unwrap();
        assert_eq!(work, expected, "{text}");
        assert_eq!(plan.is_some(), supported, "{text}");
        if let Some(plan) = plan {
            assert_eq!(Some(plan.search_passes()), passes);
        }
        for fail in 1..=expected.len() {
            let mut seen = Vec::new();
            assert!(matches!(recording(&source, i, m, &mut seen, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail));
            assert_eq!(seen, expected[..fail], "{text} fail={fail}");
            let mut fresh = Vec::new();
            assert_eq!(
                recording(&source, i, m, &mut fresh, None)
                    .unwrap()
                    .is_some(),
                supported
            );
            assert_eq!(fresh, expected);
        }
        if supported {
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(&source, i, m, |_| Ok::<
                    (),
                    usize,
                >(
                    ()
                ))
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
                    &source,
                    i,
                    m,
                    |_| Ok::<(), usize>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                    &source,
                    i,
                    m,
                    |_| Ok::<(), usize>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_work(
                    &source,
                    i,
                    m,
                    |_| Ok::<(), usize>(())
                )
                .unwrap()
                .is_none()
            );
        }
    }
    for text in [
        "",
        "[a]b",
        "^[a]b$",
        r"\baa",
        r"\b[a]",
        r"\ba[b]",
        r"\b[a][b]",
        r"\b[a]bc",
        r"\b[a].",
        r"\b[a]\d",
        r"\b[a]\p{Any}",
        r"\b[a]^b",
        r"\b[a]$b",
        r"\b^[a]b",
        r"\b[a]b$\b",
        r"\b\b[a]b",
        r"\b[a]b\b\B",
        r"\b[a]b$$",
        r"^\b[a]b\b$^",
        r"\b([a]b)",
        r"\b([a])b",
        r"\b[a](b)",
        r"\b[a]+b",
        r"\b[a]b?",
        r"\b(?:[a]b)+",
        r"\b[]bc",
        r"\b[]\d",
        r"\b[]\p{RGI_Emoji}",
        r"\b[\q{ab}]c",
        r"\b[a&&\q{a}]b",
        r"\b[a]\u{D800}\u{DC00}",
    ] {
        let source = JsString::from(text);
        let mut seen = Vec::new();
        assert!(
            recording(&source, false, false, &mut seen, None)
                .unwrap()
                .is_none(),
            "{text}"
        );
        assert_eq!(seen, [source.len().saturating_mul(3)], "{text}");
        let mut aborted = Vec::new();
        assert!(matches!(
            recording(&source, false, false, &mut aborted, Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
        assert_eq!(aborted, seen);
    }
    for (source, input, range) in [
        (r"\b[a]b", "abcx", Some(0..2)),
        (r"\b[a]b", "!ab", Some(1..3)),
        (r"\b[a]b", "xab", None),
        (r"[a]b\b", "xab", Some(1..3)),
        (r"[a]b\b", "abx", None),
        (r"\b[a]\$\B", "a$\n", Some(0..2)),
        (r"\b[a]\$\B$", "a$\n", None),
        (r"\b[a]\\\B$", "a\\", Some(0..2)),
        (r"\b[a]\u{000062}\b", "ab", Some(0..2)),
        (r"\B[😀]😁\B", "😀😁", Some(0..4)),
        (r"[\P{Ll}]a\b", "!a!", Some(0..2)),
    ] {
        assert_eq!(
            compile(source, true, false)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            range,
            "{source}"
        );
    }
    let overlap = compile(r"\B[a]a\b", false, false).unwrap();
    let input = JsString::from("aaa!");
    assert_eq!(overlap.find(&input, 0, false), Some(1..3));
    assert_eq!(overlap.find(&input, 0, true), None);
    assert_eq!(overlap.find(&input, 1, true), Some(1..3));
    let mut entry = Vec::new();
    assert_eq!(
        overlap.find_with_work(&input, 0, false, |n| {
            entry.push(n);
            Err::<(), usize>(77)
        }),
        Err(77)
    );
    assert_eq!(entry, [1]);
    let mut entry = Vec::new();
    assert_eq!(
        overlap.find_with_work(&input, 0, false, |n| {
            entry.push(n);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..3))
    );
    assert_eq!(entry, [1]);
    let pair = compile(r"\b[\p{Any}]\u{1f601}\B", true, false).unwrap();
    assert_eq!(pair.find(&JsString::from("A😀😁!"), 2, true), Some(1..5));
    // Both consuming predicates are fold-invariant here. Only the original
    // endpoint WordCharacters depend on the caller's actual i flag.
    for input in ["K!!", "ſ!!"] {
        let input = JsString::from(input);
        assert_eq!(
            compile(r"\b[\p{Any}]!\B", true, false)
                .unwrap()
                .find(&input, 0, false),
            Some(0..2)
        );
        assert_eq!(
            compile(r"\b[\p{Any}]!\B", false, false)
                .unwrap()
                .find(&input, 0, false),
            None
        );
    }
}
