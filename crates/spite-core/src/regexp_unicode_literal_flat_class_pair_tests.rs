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
    literal: u32,
    sets: [Set; 2],
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

// Identical selected C/S and WordCharacters facts, independent of production
// decoders, folding tables, class predicates and outside-assertion helpers.
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
    // Keep complete original characters at the approved initial low-pair offset.
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
        if canonical(input.points[at]) == canonical(case.literal)
            && contains(case.sets[0], input.points[at + 1], case.ignore_case)
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

// A single concrete new-compiler callback site serves all fresh abort prefixes.
fn recording(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeLiteralClassPairWordMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeLiteralClassPairWordMatcher::compile_flat_v_set_with_work(source, i, m, |n| {
        work.push(n);
        if fail == Some(work.len()) {
            Err(work.len())
        } else {
            Ok(())
        }
    })
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeLiteralClassPairWordMatcher> {
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
fn literal_flat_class_pair_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, Empty, NotK, Point};
    let inputs = [
        JsString::from(""),
        JsString::from("aaaa!"),
        JsString::from("aaab! cabx abc"),
        JsString::from("Kſk KsK a!s aKs"),
        JsString::from("$!K"),
        JsString::from("$!ſ"),
        JsString::from("😀😁😀!"),
        JsString::from("x😀😁a"),
        JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0xd800, 0x21]),
        JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
        JsString::from("a\r\nabc\u{2028}ABC\u{2029}abc\rc"),
        JsString::from("abc"),
        JsString::from("[/$"),
    ]
    .map(Input::new);
    let mut queries = 0;
    for (source, literal, sets, i, m, anchors, words) in [
        (
            "a[a][b]",
            0x61,
            [Point(0x61), Point(0x62)],
            false,
            false,
            [false; 2],
            [None; 2],
        ),
        (
            r"\bc[a][b]",
            0x63,
            [Point(0x61), Point(0x62)],
            false,
            false,
            [false; 2],
            [Some(true), None],
        ),
        (
            r"a[a][a]\b",
            0x61,
            [Point(0x61); 2],
            false,
            false,
            [false; 2],
            [None, Some(true)],
        ),
        (
            r"\bK[s][K]\b",
            0x4b,
            [Point(0x73), Point(0x4b)],
            true,
            false,
            [false; 2],
            [Some(true); 2],
        ),
        (
            r"\bK[s][K]\b",
            0x4b,
            [Point(0x73), Point(0x4b)],
            false,
            false,
            [false; 2],
            [Some(true); 2],
        ),
        (
            r"\ba[^K][s]\b",
            0x61,
            [NotK, Point(0x73)],
            true,
            false,
            [false; 2],
            [Some(true); 2],
        ),
        (
            r"^\B\$[\p{Any}][\p{Any}]\b$",
            0x24,
            [Any; 2],
            true,
            false,
            [true; 2],
            [Some(false), Some(true)],
        ),
        (
            r"^\B\$[\p{Any}][\p{Any}]\b$",
            0x24,
            [Any; 2],
            false,
            false,
            [true; 2],
            [Some(false), Some(true)],
        ),
        (
            "a[][b]",
            0x61,
            [Empty, Point(0x62)],
            false,
            false,
            [false; 2],
            [None; 2],
        ),
        (
            "a[a][]",
            0x61,
            [Point(0x61), Empty],
            true,
            false,
            [false; 2],
            [None; 2],
        ),
        (
            r"\B\u{1f600}[\u{1f601}][\u{1f600}]\B",
            0x1f600,
            [Point(0x1f601), Point(0x1f600)],
            false,
            false,
            [false; 2],
            [Some(false); 2],
        ),
        (
            r"\b\uD800[\uD800][\uD800]\B",
            0xd800,
            [Point(0xd800); 2],
            false,
            false,
            [false; 2],
            [Some(true), Some(false)],
        ),
        (
            r"^\ba[b][c]\b$",
            0x61,
            [Point(0x62), Point(0x63)],
            false,
            true,
            [true; 2],
            [Some(true); 2],
        ),
        (
            r"^\ba[b][c]\b$",
            0x61,
            [Point(0x62), Point(0x63)],
            false,
            false,
            [true; 2],
            [Some(true); 2],
        ),
        (
            r"\B\[[\/][\$]\B$",
            0x5b,
            [Point(0x2f), Point(0x24)],
            false,
            false,
            [false, true],
            [Some(false); 2],
        ),
    ] {
        queries += check_case(
            &Case {
                source,
                literal,
                sets,
                ignore_case: i,
                multiline: m,
                anchors,
                words,
            },
            &inputs,
        );
    }
    // Nine word frames include bare; line frames and actual i/m vary separately.
    let frame_inputs = ["abc", "x\nabc\nx"].map(|s| Input::new(JsString::from(s)));
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
            let source = format!("{prefix}{left}a[b][c]{right}{suffix}");
            for i in [false, true] {
                for m in [false, true] {
                    queries += check_case(
                        &Case {
                            source: source.as_str(),
                            literal: 0x61,
                            sets: [Point(0x62), Point(0x63)],
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
    assert_eq!(queries, 7_302);
}

#[test]
fn literal_flat_class_pair_domains_full_proofs_and_work_prefixes() {
    // Complete framing fails before any component proof, including q behind [].
    for source in [
        "",
        "^",
        r"\b",
        "a",
        "a[b]",
        "[a][b]c",
        "ab[b][c]",
        "a[b][c]x",
        "a[b][c][d]",
        "a[b]+[c]",
        "a[b][c]?",
        "(a[b][c])",
        "(?:a[b][c])",
        "a([b])[c]",
        "a[b]([c])",
        "a^[b][c]",
        "a[b]$[c]",
        r"a\b[b][c]",
        r"a[b]\B[c]",
        r"\b^a[b][c]",
        r"a[b][c]$\b",
        r"\b\ba[b][c]",
        r"a[b][c]\b\B",
        r"a[b][c]\\b",
        r"a[b][c]\$",
        r"\\ba[b][c]",
        r"\w[b][c]",
        r"\p{Any}[b][c]",
        r"a[][\q{b}]",
        r"a[\q{b}][c]",
        "a[b][c]|d",
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
        assert_eq!(failed, [entry]);
    }
    let long_literal = format!(r"\u{{{}61}}", "0".repeat(16));
    let long_source = format!(r"\B{long_literal}[a][b]\b");
    // Explicit component spellings reconstruct literal -> first -> last work.
    // Class proof None stops later work; a successful empty class does not.
    for (source, literal, first, last, i, some) in [
        ("a[a][b]", "a", "[a]", "[b]", false, true),
        (r"^\ba[b][c]\b$", "a", "[b]", "[c]", false, true),
        ("K[K][s]", "K", "[K]", "[s]", true, true),
        ("a[][b]", "a", "[]", "[b]", false, true),
        ("a[b][]", "a", "[b]", "[]", false, true),
        (
            long_source.as_str(),
            long_literal.as_str(),
            "[a]",
            "[b]",
            false,
            true,
        ),
        (r"\B\[[\/][\$]\B$", r"\[", r"[\/]", r"[\$]", false, true),
        ("a[[a]][b]", "a", "[[a]]", "[b]", false, false),
        ("a[a][[b]]", "a", "[a]", "[[b]]", false, false),
        (
            r"a[][\p{RGI_Emoji}]",
            "a",
            "[]",
            r"[\p{RGI_Emoji}]",
            false,
            false,
        ),
    ] {
        let source = JsString::from(source);
        let mut observed = Vec::new();
        let matcher = recording(&source, i, false, &mut observed, None).unwrap();
        assert_eq!(matcher.is_some(), some, "{source:?}");
        let mut expected = vec![source.len().saturating_mul(4)];
        let literal = JsString::from(literal);
        let mut complete =
            RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
                literal.code_units(),
                i,
                false,
                |n| {
                    expected.push(n);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .is_some();
        for part in [first, last] {
            if complete {
                complete = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
                    JsString::from(part).code_units(),
                    i,
                    false,
                    |n| {
                        expected.push(n);
                        Ok::<(), usize>(())
                    },
                )
                .unwrap()
                .is_some();
            }
        }
        assert_eq!(complete, some);
        assert_eq!(observed, expected, "{source:?}");
        for fail in 1..=observed.len() {
            let mut failed = Vec::new();
            assert!(matches!(
                recording(&source, i, false, &mut failed, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail
            ));
            assert_eq!(failed, observed[..fail]);
        }
    }
    // Old checked constructors retain both their bare and outside-word domains.
    for source in ["a[b][c]", r"\ba[b][c]", r"a[b][c]\b", r"^\ba[b][c]\b$"] {
        let source = JsString::from(source);
        assert!(
            RegExpUnicodeLiteralClassPairMatcher::compile_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), usize>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeLiteralClassPairWordMatcher::compile_v_set_with_work(
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
        (r"\B\[[\/][\$]\B$", "[/$"),
        (r"\B\/[a][b]\b", "/ab"),
        (r"\B\\[a][b]\b", r"\ab"),
        (r"\B\$[a][b]\b", "$ab"),
        (r"\b\x61[\x62][\x63]\b", "abc"),
        (r"\b\u{61}[\u0062][\u{63}]\b", "abc"),
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from(input), 0, true),
            Some(0..3)
        );
    }
    for source in [
        r"\B\u{1f600}[\u{1f601}][\u{1f600}]\B",
        r"\B\uD83D\uDE00[\u{1f601}][\u{1f600}]\B",
        r"\B😀[😁][😀]\B",
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from("!😀😁😀!"), 2, true),
            Some(1..7)
        );
    }
    assert_eq!(
        compile(r"\B\uD800[\uD800][\uD800]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0x21, 0xd800, 0xd800, 0xd800, 0x21]),
                1,
                true
            ),
        Some(1..4)
    );
    assert_eq!(
        compile(r"\B\uD800[\uD800][\uDC00]\B", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
                0,
                false
            ),
        None
    );
    for input in ["$!K", "$!ſ"] {
        let input = JsString::from(input);
        let source = r"^\B\$[\p{Any}][\p{Any}]\b$";
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&input, 0, false),
            None
        );
        assert_eq!(
            compile(source, true, false).unwrap().find(&input, 0, false),
            Some(0..3)
        );
    }
    assert_eq!(
        compile(r"\bc[a][b]", false, false)
            .unwrap()
            .find(&JsString::from("cabx"), 0, false),
        Some(0..3)
    );
    assert_eq!(
        compile(r"^\ba[b][c]\b$", false, true).unwrap().find(
            &JsString::from("x\r\nabc\u{2028}x"),
            0,
            false
        ),
        Some(3..6)
    );
    for source in ["a[a][b]", r"a[a][a]\b"] {
        let matcher = compile(source, false, false).unwrap();
        let input = JsString::from(if source == "a[a][b]" {
            "aaab!"
        } else {
            "aaaa!"
        });
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
