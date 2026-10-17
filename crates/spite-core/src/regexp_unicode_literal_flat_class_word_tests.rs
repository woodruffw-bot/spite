use super::*;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    Any,
    NotK,
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

// Identical selected simple/common-fold facts from the earlier finite oracle.
// The oracle never calls production data, membership or boundary helpers.
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
        let p = input.points[at + 1];
        let member = match case.set {
            Set::Point(value) => canonical(p) == canonical(value),
            Set::Any => true,
            Set::NotK => canonical(p) != canonical(0x4b),
            Set::Empty => false,
        };
        let begin = input.boundaries[at];
        let end = input.boundaries[at + 2];
        if member
            && canonical(input.points[at]) == canonical(case.literal)
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
    RegExpUnicodeClassLiteralMatcher::compile_literal_then_flat_v_set_with_word_boundaries_and_work(
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
fn literal_flat_class_word_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, Empty, NotK, Point};
    let inputs = [
        JsString::from(""),
        JsString::from("aaa!"),
        JsString::from("abcx !ab xab abx"),
        JsString::from("ſK sK sk"),
        JsString::from("aaK aaſ α"),
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
    for (source, literal, set, i, words) in [
        (
            r"\Ba[a]\b",
            0x61,
            Point(0x61),
            false,
            [Some(false), Some(true)],
        ),
        (r"\ba[b]", 0x61, Point(0x62), false, [Some(true), None]),
        (r"a[b]\b", 0x61, Point(0x62), false, [None, Some(true)]),
        (
            r"\bs[K]\b",
            0x73,
            Point(0x4b),
            true,
            [Some(true), Some(true)],
        ),
        (
            r"\bs[K]\b",
            0x73,
            Point(0x4b),
            false,
            [Some(true), Some(true)],
        ),
        (r"\Bs[^K]\b", 0x73, NotK, true, [Some(false), Some(true)]),
        (
            r"\Ba[\p{Any}]\b",
            0x61,
            Any,
            true,
            [Some(false), Some(true)],
        ),
        (
            r"\Ba[\p{Any}]\b",
            0x61,
            Any,
            false,
            [Some(false), Some(true)],
        ),
        (r"\Ba[]\b", 0x61, Empty, true, [Some(false), Some(true)]),
        (
            r"\B\u{1f600}[\u{1f601}]\B",
            0x1f600,
            Point(0x1f601),
            false,
            [Some(false), Some(false)],
        ),
        (
            r"\B\uD800[a]\b",
            0xd800,
            Point(0x61),
            false,
            [Some(false), Some(true)],
        ),
        (
            r"\ba[\uD800]\B",
            0x61,
            Point(0xd800),
            false,
            [Some(true), Some(false)],
        ),
        (
            r"\B\uDC00[a]\b",
            0xdc00,
            Point(0x61),
            false,
            [Some(false), Some(true)],
        ),
    ] {
        queries += check_case(
            &Case {
                source,
                literal,
                set,
                ignore_case: i,
                multiline: false,
                anchors: [false, false],
                words,
            },
            &inputs,
        );
    }
    // Eight nonempty word choices and four line frames use only three inputs.
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
            let source = format!("{prefix}{left}a[b]{right}{suffix}");
            for i in [false, true] {
                for m in [false, true] {
                    queries += check_case(
                        &Case {
                            source: &source,
                            literal: 0x61,
                            set: Point(0x62),
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
    let semantic_per_case = inputs.iter().map(|s| 2 * (s.text.len() + 2)).sum::<usize>();
    let frame_per_case = frame_inputs
        .iter()
        .map(|s| 2 * (s.text.len() + 2))
        .sum::<usize>();
    assert_eq!(
        queries,
        13 * semantic_per_case + 8 * 4 * 2 * 2 * frame_per_case
    );
    assert_eq!(queries, 7_876);
}

// Explicit bare slices supply the established literal-first/class-second work.
// All outside gates belong to the separate complete-pair assertion frame.
fn expected_work(
    source: &JsString,
    literal: &str,
    class: &str,
    i: bool,
    m: bool,
    words: [Option<bool>; 2],
    anchors: [bool; 2],
) -> (Vec<usize>, Option<usize>) {
    let mut work = vec![source.len().saturating_mul(3)];
    let literal_source = JsString::from(literal);
    let class_source = JsString::from(class);
    let Some(literal) = RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
        literal_source.code_units(),
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
    let class = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
        class_source.code_units(),
        i,
        m,
        |n| {
            work.push(n);
            Ok::<(), usize>(())
        },
    )
    .unwrap();
    let passes = class.map(|class| {
        literal
            .search_passes()
            .saturating_add(class.search_passes())
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
fn literal_flat_class_word_domains_full_proofs_and_work_prefixes() {
    let long_literal = format!(r"\u{{{}61}}", "0".repeat(64));
    let long_source = format!(r"\B{long_literal}[]\b");
    for (text, literal, class, i, m, words, anchors, supported) in [
        (
            r"\ba[b]",
            "a",
            "[b]",
            false,
            false,
            [Some(true), None],
            [false, false],
            true,
        ),
        (
            r"^\bs[K]\b$",
            "s",
            "[K]",
            true,
            true,
            [Some(true), Some(true)],
            [true, true],
            true,
        ),
        (
            r"\B\uD83D\uDE00[\p{Any}]\B",
            r"\uD83D\uDE00",
            r"[\p{Any}]",
            true,
            false,
            [Some(false), Some(false)],
            [false, false],
            true,
        ),
        (
            r"\B\uD800[a]\b",
            r"\uD800",
            "[a]",
            false,
            false,
            [Some(false), Some(true)],
            [false, false],
            true,
        ),
        (
            long_source.as_str(),
            long_literal.as_str(),
            "[]",
            true,
            false,
            [Some(false), Some(true)],
            [false, false],
            true,
        ),
        (
            r"^\B\$[\p{Any}]\b$",
            r"\$",
            r"[\p{Any}]",
            true,
            false,
            [Some(false), Some(true)],
            [true, true],
            true,
        ),
        (
            r"\ba[[b]]\b",
            "a",
            "[[b]]",
            false,
            false,
            [Some(true), Some(true)],
            [false, false],
            false,
        ),
        (
            r"\ba[b--b]\b",
            "a",
            "[b--b]",
            false,
            false,
            [Some(true), Some(true)],
            [false, false],
            false,
        ),
        (
            r"\Ba[^\p{Any}\p{RGI_Emoji}]\B",
            "a",
            r"[^\p{Any}\p{RGI_Emoji}]",
            false,
            false,
            [Some(false), Some(false)],
            [false, false],
            false,
        ),
    ] {
        let source = JsString::from(text);
        let (expected, passes) = expected_work(&source, literal, class, i, m, words, anchors);
        assert_eq!(passes.is_some(), supported, "{text}");
        let mut seen = Vec::new();
        let plan = recording(&source, i, m, &mut seen, None).unwrap();
        assert_eq!(seen, expected, "{text}");
        assert_eq!(plan.is_some(), supported, "{text}");
        if let Some(plan) = plan {
            assert_eq!(Some(plan.search_passes()), passes);
        }
        for fail in 1..=expected.len() {
            let mut aborted = Vec::new();
            assert!(
                matches!(recording(&source, i, m, &mut aborted, Some(fail)), Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(aborted, expected[..fail], "{text} fail={fail}");
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
            let accept: fn(usize) -> Result<(), usize> = |_| Ok(());
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(&source, i, m, accept)
                    .unwrap()
                    .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
                    &source, i, m, accept
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                    &source, i, m, accept
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_work(
                    &source, i, m, accept
                )
                .unwrap()
                .is_none()
            );
            assert!(RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_word_boundaries_and_work(&source, i, m, accept).unwrap().is_none());
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_literal_then_flat_v_set_with_work(
                    &source, i, m, accept
                )
                .unwrap()
                .is_none()
            );
        }
    }
    for text in [
        "",
        "a[b]",
        "^a[b]$",
        r"\baa",
        r"\ba",
        r"\b[a]b",
        r"\ba[b][c]",
        r"\bab[c]",
        r"\ba[b]c",
        r"\b.[b]",
        r"\b\d[b]",
        r"\b\p{Any}[b]",
        r"\ba^[b]",
        r"\ba$[b]",
        r"\b^a[b]",
        r"\ba[b]$\b",
        r"\b\ba[b]",
        r"\ba[b]\b\B",
        r"\ba[b]$$",
        r"^\ba[b]\b$^",
        r"\b(a[b])",
        r"\b(a)[b]",
        r"\ba([b])",
        r"\ba+[b]",
        r"\ba[b]?",
        r"\b(?:a[b])+",
        r"\b[]bc",
        r"\ba[\q{ab}]",
        r"\ba[b&&\q{b}]",
        r"\b\u{D800}\u{DC00}[a]",
        r"\ba[b]\$",
        r"\ba\b[b]",
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
        (r"\ba[b]", "abcx", Some(0..2)),
        (r"\ba[b]", "!ab", Some(1..3)),
        (r"\ba[b]", "xab", None),
        (r"a[b]\b", "xab", Some(1..3)),
        (r"a[b]\b", "abx", None),
        (r"\B\$[\]]\B", "$]", Some(0..2)),
        (r"\B\\[a]\b$", "\\a", Some(0..2)),
        (r"\b\u{000061}[b]\b", "ab", Some(0..2)),
        (r"\B😀[😁]\B", "😀😁", Some(0..4)),
    ] {
        assert_eq!(
            compile(source, true, false)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            range,
            "{source}"
        );
    }
    let overlap = compile(r"\Ba[a]\b", false, false).unwrap();
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
    let pair = compile(r"\b\u{1f600}[\p{Any}]\B", true, false).unwrap();
    assert_eq!(pair.find(&JsString::from("A😀😁!"), 2, true), Some(1..5));
    // Both consuming predicates are fold-invariant. Only the actual caller i
    // changes the final WordCharacters gate.
    for input in ["$K", "$ſ"] {
        let input = JsString::from(input);
        assert_eq!(
            compile(r"^\B\$[\p{Any}]\b$", true, false)
                .unwrap()
                .find(&input, 0, true),
            Some(0..2)
        );
        assert_eq!(
            compile(r"^\B\$[\p{Any}]\b$", false, false)
                .unwrap()
                .find(&input, 0, true),
            None
        );
    }
}
