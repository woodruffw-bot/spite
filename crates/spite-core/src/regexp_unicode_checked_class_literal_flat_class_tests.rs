use super::*;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    NotK,
    Any,
    Empty,
}

struct Case<'a> {
    source: &'a str,
    sets: [Set; 2],
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

// Reuses identical selected common/simple-fold and WordCharacters facts only.
// No production decoder, folding table, set predicate or word helper is an oracle.
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

fn contains(set: Set, point: u32, i: bool) -> bool {
    let canonical = |p| if i { fold(p) } else { p };
    match set {
        Set::Point(value) => canonical(point) == canonical(value),
        Set::NotK => canonical(point) != canonical(0x4b),
        Set::Any => true,
        Set::Empty => false,
    }
}

fn expected(case: &Case<'_>, input: &Input, start: usize, sticky: bool) -> Option<Range<usize>> {
    if start > input.text.len() {
        return None;
    }
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
    for at in first..input.points.len().saturating_sub(2) {
        if sticky && at != first {
            break;
        }
        let begin = input.boundaries[at];
        let end = input.boundaries[at + 3];
        if contains(case.sets[0], input.points[at], case.ignore_case)
            && canonical(input.points[at + 1]) == canonical(case.literal)
            && contains(case.sets[1], input.points[at + 2], case.ignore_case)
            && (!case.anchors[0]
                || begin == 0
                || (case.multiline && line(input.text.code_units()[begin - 1])))
            && (!case.anchors[1]
                || end == input.text.len()
                || (case.multiline && line(input.text.code_units()[end])))
            && case.words[0].is_none_or(|wanted| boundary(at) == wanted)
            && case.words[1].is_none_or(|wanted| boundary(at + 3) == wanted)
        {
            return Some(begin..end);
        }
    }
    None
}

// One concrete new-compiler callback site serves every fresh abort transcript.
fn recording(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassLiteralClassWordMatcher>, RegExpUnicodeSetCompileError<usize>>
{
    RegExpUnicodeClassLiteralClassWordMatcher::compile_checked_then_flat_v_set_with_work(
        source,
        i,
        m,
        |n| {
            work.push(n);
            if fail == Some(work.len()) {
                Err(work.len())
            } else {
                Ok(())
            }
        },
    )
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassLiteralClassWordMatcher> {
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
fn checked_class_literal_flat_class_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, Empty, NotK, Point};
    let inputs = [
        JsString::from(""),
        JsString::from("aaaa!"),
        JsString::from("abcx !abc xabc abcx"),
        JsString::from("Kaſ! KaS! ſaK!"),
        JsString::from("!$K !$ſ !$K"),
        JsString::from("!$K"),
        JsString::from("!$ſ"),
        JsString::from("😀😁😀!"),
        JsString::from("x😀😁a"),
        JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
        JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
        JsString::from("a\r\nabc\u{2028}ABC\u{2029}abc\rc"),
        JsString::from("ab\n"),
        JsString::from("0a _a !a!"),
    ]
    .map(Input::new);
    let mut queries = 0;
    for (source, sets, literal, i, m, anchors, words) in [
        (
            r"[[a]]b[c]",
            [Point(0x61), Point(0x63)],
            0x62,
            false,
            false,
            [false; 2],
            [None; 2],
        ),
        (
            r"\b[[a]]b[c]",
            [Point(0x61), Point(0x63)],
            0x62,
            false,
            false,
            [false; 2],
            [Some(true), None],
        ),
        (
            r"[[a]]b[c]\b",
            [Point(0x61), Point(0x63)],
            0x62,
            false,
            false,
            [false; 2],
            [None, Some(true)],
        ),
        (
            r"\B[[a]]a[a]\b",
            [Point(0x61); 2],
            0x61,
            false,
            false,
            [false; 2],
            [Some(false), Some(true)],
        ),
        (
            r"\b[K&&K]a[s]\b",
            [Point(0x4b), Point(0x73)],
            0x61,
            true,
            false,
            [false; 2],
            [Some(true); 2],
        ),
        (
            r"\b[K&&K]a[s]\b",
            [Point(0x4b), Point(0x73)],
            0x61,
            false,
            false,
            [false; 2],
            [Some(true); 2],
        ),
        (
            r"\B[[^K]]a[s]\b",
            [NotK, Point(0x73)],
            0x61,
            true,
            false,
            [false; 2],
            [Some(false), Some(true)],
        ),
        (
            r"^\B[[\p{Any}]]\$[\p{Any}]\b$",
            [Any; 2],
            0x24,
            true,
            false,
            [true; 2],
            [Some(false), Some(true)],
        ),
        (
            r"^\B[[\p{Any}]]\$[\p{Any}]\b$",
            [Any; 2],
            0x24,
            false,
            false,
            [true; 2],
            [Some(false), Some(true)],
        ),
        (
            r"[a--a]b[c]",
            [Empty, Point(0x63)],
            0x62,
            true,
            false,
            [false; 2],
            [None; 2],
        ),
        (
            r"[[a]]b[]",
            [Point(0x61), Empty],
            0x62,
            false,
            false,
            [false; 2],
            [None; 2],
        ),
        (
            r"\B[[\u{1f600}]]\u{1f601}[\u{1f600}]\B",
            [Point(0x1f600), Point(0x1f600)],
            0x1f601,
            false,
            false,
            [false; 2],
            [Some(false); 2],
        ),
        (
            r"\b[[\uD800]]\uD800[\uD800]\B",
            [Point(0xd800); 2],
            0xd800,
            false,
            false,
            [false; 2],
            [Some(true), Some(false)],
        ),
        (
            r"^\b[[a]]b[c]\b$",
            [Point(0x61), Point(0x63)],
            0x62,
            false,
            true,
            [true; 2],
            [Some(true); 2],
        ),
        (
            r"^\b[[a]]b[c]\b$",
            [Point(0x61), Point(0x63)],
            0x62,
            false,
            false,
            [true; 2],
            [Some(true); 2],
        ),
    ] {
        queries += check_case(
            &Case {
                source,
                sets,
                literal,
                ignore_case: i,
                multiline: m,
                anchors,
                words,
            },
            &inputs,
        );
    }
    // All nine optional-word choices, four line frames and both actual i/m flags.
    let frame_inputs = ["abcx", "x\nabc\nx"].map(|s| Input::new(JsString::from(s)));
    for (left, right, words) in [
        ("", "", [None, None]),
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
            let source = format!("{prefix}{left}[[a]]b[c]{right}{suffix}");
            for i in [false, true] {
                for m in [false, true] {
                    queries += check_case(
                        &Case {
                            source: source.as_str(),
                            sets: [Point(0x61), Point(0x63)],
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
    let semantic_per_case = inputs.iter().map(|s| 2 * (s.text.len() + 2)).sum::<usize>();
    let frame_per_case = frame_inputs
        .iter()
        .map(|s| 2 * (s.text.len() + 2))
        .sum::<usize>();
    assert_eq!(
        queries,
        15 * semantic_per_case + 9 * 4 * 2 * 2 * frame_per_case
    );
    assert_eq!(queries, 8_280);
}

#[test]
fn checked_class_literal_flat_class_domains_full_proofs_and_work_prefixes() {
    // Full framing, including q rejection, precedes every component callback.
    // A later flat string property is instead probed behind a checked empty.
    for source in [
        "",
        "^",
        "$",
        "^$",
        "[[a]]",
        "[[a]]b",
        "[[a]][c]",
        "a[[b]][c]",
        "[[a]]bc[c]",
        "[[a]]b[c]d",
        "x[[a]]b[c]",
        "[[a]]b[c][d]",
        r"[[a]].[c]",
        r"[[a]]\d[c]",
        r"[[a]]\p{Any}[c]",
        "[[a]]^[c]",
        "[[a]]$[c]",
        r"[[a]]\b[c]",
        r"[[a]]b\B[c]",
        r"\b^[[a]]b[c]",
        r"[[a]]b[c]$\b",
        r"\b\b[[a]]b[c]",
        r"[[a]]b[c]\b\B",
        "[[a]]b[c]$$",
        "([[a]])b[c]",
        "[[a]](b)[c]",
        "[[a]]b([c])",
        "([[a]]b[c])",
        "[[a]]+b[c]",
        "[[a]]b?[c]",
        "[[a]]b[c]+",
        "(?:[[a]]b[c])+",
        r"[a--a]b[\q{c}]",
        r"[\q{a}&&a]b[]",
        r"[[a]]b[c]\$",
    ] {
        let source = JsString::from(source);
        let entry = source.len().saturating_mul(4);
        let mut work = Vec::new();
        assert!(
            recording(&source, false, false, &mut work, None)
                .unwrap()
                .is_none()
        );
        assert_eq!(work, [entry], "{source:?}");
        let mut failed = Vec::new();
        assert!(matches!(
            recording(&source, false, false, &mut failed, Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
        assert_eq!(failed, work);
    }
    let long_literal = format!(r"\u{{{}62}}", "0".repeat(16));
    let long_source = format!("[a--a]{long_literal}[]");
    // Independent explicit slices require checked -> literal -> flat callbacks,
    // including both full later proofs after successful checked emptiness.
    for (text, checked, literal, flat, i, m, some) in [
        ("[[a]]b[c]", "[[a]]", "b", "[c]", false, false, true),
        (r"^\b[K&&K]a[s]\b$", "[K&&K]", "a", "[s]", true, true, true),
        ("[a--a]b[c]", "[a--a]", "b", "[c]", false, false, true),
        ("[[a]]b[]", "[[a]]", "b", "[]", true, false, true),
        ("[a--a]b[]", "[a--a]", "b", "[]", false, false, true),
        (
            long_source.as_str(),
            "[a--a]",
            long_literal.as_str(),
            "[]",
            false,
            false,
            true,
        ),
        (
            r"^\B[[\p{Any}]]\$[\p{Any}]\b$",
            r"[[\p{Any}]]",
            r"\$",
            r"[\p{Any}]",
            true,
            true,
            true,
        ),
        (
            r"\B[[\u{1f600}]]\uD83D\uDE01[\u{1f600}]\B",
            r"[[\u{1f600}]]",
            r"\uD83D\uDE01",
            r"[\u{1f600}]",
            false,
            false,
            true,
        ),
        ("[a]b[c]", "[a]", "b", "[c]", false, false, false),
        ("[[a]]b[[c]]", "[[a]]", "b", "[[c]]", false, false, false),
        ("[a]b[[c]]", "[a]", "b", "[[c]]", false, false, false),
        ("[[a]]b[c--c]", "[[a]]", "b", "[c--c]", false, false, false),
        (
            r"[a--a]b[\p{RGI_Emoji}]",
            "[a--a]",
            "b",
            r"[\p{RGI_Emoji}]",
            false,
            false,
            false,
        ),
        (
            r"[\p{RGI_Emoji}&&a]b[c]",
            r"[\p{RGI_Emoji}&&a]",
            "b",
            "[c]",
            false,
            false,
            false,
        ),
    ] {
        let source = JsString::from(text);
        let mut observed = Vec::new();
        assert_eq!(
            recording(&source, i, m, &mut observed, None)
                .unwrap()
                .is_some(),
            some,
            "{text}"
        );
        let mut expected = vec![source.len().saturating_mul(4)];
        let checked = JsString::from(checked);
        let mut complete = RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            checked.code_units(),
            i,
            m,
            |n| {
                expected.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .is_some();
        if complete {
            let literal = JsString::from(literal);
            complete = RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                literal.code_units(),
                i,
                m,
                |n| {
                    expected.push(n);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .is_some();
        }
        if complete {
            let flat = JsString::from(flat);
            complete = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
                flat.code_units(),
                i,
                m,
                |n| {
                    expected.push(n);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .is_some();
        }
        assert_eq!(complete, some, "{text}");
        assert_eq!(observed, expected, "{text}");
        for fail in 1..=observed.len() {
            let mut failed = Vec::new();
            assert!(matches!(recording(&source, i, m, &mut failed, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail));
            assert_eq!(failed, expected[..fail], "{text} fail={fail}");
            let mut fresh = Vec::new();
            assert_eq!(
                recording(&source, i, m, &mut fresh, None)
                    .unwrap()
                    .is_some(),
                some
            );
            assert_eq!(fresh, expected);
        }
    }
    // Prior checked/checked bare, checked/checked word and flat/flat domains
    // remain exact; all capture recognition is still a separate caller proof.
    for text in [
        "[[a]]b[c]",
        r"\b[[a]]b[c]",
        r"[[a]]b[c]\b",
        r"^\b[[a]]b[c]\b$",
    ] {
        let source = JsString::from(text);
        assert!(
            RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), usize>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassLiteralClassWordMatcher::compile_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), usize>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassLiteralClassWordMatcher::compile_flat_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), usize>(()),
            )
            .unwrap()
            .is_none()
        );
    }
    for (source, input) in [
        (r"[[a]]\[[b]", "a[b"),
        (r"[[a]]\$[b]", "a$b"),
        (r"^[[a]]\$[b]$", "a$b"),
        (r"[[!]]\\[!]", "!\\!"),
        (r"[[\]]]a[\[]", "]a["),
        (r"[[\x61]]\x62[\x63]", "abc"),
        (r"[[\u0061]]\u{000062}[\u{63}]", "abc"),
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from(input), 0, true),
            Some(0..3)
        );
    }
    for source in [
        r"\B[[\u{1f600}]]\u{1f601}[\u{1f600}]\B",
        r"\B[[\u{1f600}]]\uD83D\uDE01[\u{1f600}]\B",
        r"\B[[😀]]😁[😀]\B",
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from("!😀😁😀!"), 2, true),
            Some(1..7)
        );
    }
    assert_eq!(
        compile(r"\b[[\p{Any}]]\u{1f601}[a]\b", false, false)
            .unwrap()
            .find(&JsString::from("A😀😁a!"), 2, true,),
        Some(1..6)
    );
    assert_eq!(
        compile(r"\b[[\uD800]]\uD800[\uD800]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
                1,
                true,
            ),
        Some(1..4)
    );
    assert_eq!(
        compile(r"\B[[\uD800]]\uD800[\uDC00]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
                0,
                false,
            ),
        None
    );
    for i in [false, true] {
        for text in ["!$K", "!$ſ"] {
            assert_eq!(
                compile(r"^\B[[\p{Any}]]\$[\p{Any}]\b$", i, false)
                    .unwrap()
                    .find(&JsString::from(text), 0, true,),
                if i { Some(0..3) } else { None }
            );
        }
        for m in [false, true] {
            assert_eq!(
                compile(r"^\B[[\p{Any}]]\$[\p{Any}]\b$", i, m)
                    .unwrap()
                    .find(&JsString::from("x\r\n!$K\u{2028}x"), 0, false,),
                if i && m { Some(3..6) } else { None }
            );
        }
    }
    assert_eq!(
        compile(r"\b[[a]]b[c]", false, false)
            .unwrap()
            .find(&JsString::from("abcx"), 0, false),
        Some(0..3)
    );
    assert_eq!(
        compile(r"^\b[[a]]b[c]\b$", false, true).unwrap().find(
            &JsString::from("x\r\nabc\u{2028}x"),
            0,
            false,
        ),
        Some(3..6)
    );
    for (source, text) in [
        ("[[a]]b[c]", "aabc"),
        ("[[a]]a[b]", "aaab!"),
        (r"[[a]]a[a]\b", "aaaa!"),
    ] {
        let matcher = compile(source, false, false).unwrap();
        let input = JsString::from(text);
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
            matcher.find_with_work(&input, 0, false, |_| Err(73)),
            Err(73)
        );
        assert_eq!(matcher.find(&input, 0, true), None);
        assert_eq!(matcher.find(&input, 1, true), Some(1..4));
    }
}
