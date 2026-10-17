use super::*;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    Not(u32),
    Any,
    Empty,
}

struct Case<'a> {
    source: &'a str,
    literal: u32,
    set: Set,
    ignore_case: bool,
    multiline: bool,
    anchors: [bool; 2],
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
        // Standard-library decoding is independent of matcher boundaries.
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

// Only identical selected simple/common-fold facts from the earlier oracle.
// Membership and boundaries do not call production predicates or tables.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
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
    // An initial low-pair position selects its original complete character.
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let canonical = |point| if case.ignore_case { fold(point) } else { point };
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..input.points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        let point = input.points[at + 1];
        let member = match case.set {
            Set::Point(value) => canonical(point) == canonical(value),
            Set::Not(value) => canonical(point) != canonical(value),
            Set::Any => true,
            Set::Empty => false,
        };
        let begin = input.boundaries[at];
        let end = input.boundaries[at + 2];
        if canonical(input.points[at]) == canonical(case.literal)
            && member
            && (!case.anchors[0]
                || begin == 0
                || (case.multiline && line(input.input.code_units()[begin - 1])))
            && (!case.anchors[1]
                || end == input.input.len()
                || (case.multiline && line(input.input.code_units()[end])))
        {
            return Some(begin..end);
        }
    }
    None
}

// One concrete new-constructor callback type for all complete/aborted compiles.
fn compile_recording_work(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassLiteralMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassLiteralMatcher::compile_literal_then_flat_v_set_with_work(
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
    compile_recording_work(&JsString::from(source), i, m, &mut Vec::new(), None).unwrap()
}

fn check_case(case: &Case<'_>, inputs: &[OracleInput]) -> usize {
    let original = compile(case.source, case.ignore_case, case.multiline).unwrap();
    let matcher = original.clone();
    drop(original);
    let mut queries = 0;
    for input in inputs {
        for start in 0..=input.input.len() + 1 {
            for sticky in [false, true] {
                assert_eq!(
                    matcher.find(&input.input, start, sticky),
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
fn literal_flat_class_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, Empty, Not, Point};
    let inputs = [
        JsString::from(""),
        JsString::from("aab!"),
        JsString::from("ſKsK!ks"),
        JsString::from("Ksſs!"),
        JsString::from("αA!Aa"),
        JsString::from("A😀😁!"),
        JsString::from("x😀😁!"),
        JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21, 0x61, 0xd800]),
        JsString::from("x\r\nab\u{2028}ac"),
        JsString::from("ab"),
        JsString::from("$]"),
    ]
    .map(OracleInput::new);
    let cases = [
        (r"a[b]", 0x61, Point(0x62), false),
        (r"s[K]", 0x73, Point(0x4b), true),
        (r"s[K]", 0x73, Point(0x4b), false),
        (r"s[^K]", 0x73, Not(0x4b), true),
        (r"s[^K]", 0x73, Not(0x4b), false),
        (r"a[\p{Any}]", 0x61, Any, true),
        (r"a[]", 0x61, Empty, false),
        (r"\u{1f600}[\u{1f601}]", 0x1f600, Point(0x1f601), false),
        (r"\uD800[a]", 0xd800, Point(0x61), false),
        (r"a[\uD800]", 0x61, Point(0xd800), false),
        (r"a[^b]", 0x61, Not(0x62), false),
        (r"\$[\]]", 0x24, Point(0x5d), false),
    ];
    let mut queries = 0;
    for (source, literal, set, ignore_case) in cases {
        queries += check_case(
            &Case {
                source,
                literal,
                set,
                ignore_case,
                multiline: false,
                anchors: [false, false],
            },
            &inputs,
        );
    }
    for (source, anchors) in [
        ("a[b]", [false, false]),
        ("^a[b]", [true, false]),
        ("a[b]$", [false, true]),
        ("^a[b]$", [true, true]),
    ] {
        for multiline in [false, true] {
            queries += check_case(
                &Case {
                    source,
                    literal: 0x61,
                    set: Point(0x62),
                    ignore_case: false,
                    multiline,
                    anchors,
                },
                &inputs,
            );
        }
    }
    // Eleven inputs have51 original code units: 2*(51+2*11)=146 per case.
    assert_eq!(queries, (12 + 4 * 2) * 146);
}

// Explicit component slices avoid rediscovering boundaries using new framing.
// The literal keeps leading^ and the flat class keeps final$.
fn expected_work(
    source: &JsString,
    literal: &str,
    class: &str,
    i: bool,
    m: bool,
) -> (Vec<usize>, Option<usize>) {
    let literal = JsString::from(literal);
    let class = JsString::from(class);
    let mut work = vec![source.len().saturating_mul(3)];
    let Some(literal) = RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
        literal.code_units(),
        i,
        m,
        |amount| {
            work.push(amount);
            Ok::<(), usize>(())
        },
    )
    .unwrap() else {
        return (work, None);
    };
    let class = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
        class.code_units(),
        i,
        m,
        |amount| {
            work.push(amount);
            Ok::<(), usize>(())
        },
    )
    .unwrap();
    let passes = class.map(|class| {
        literal
            .search_passes()
            .saturating_add(class.search_passes())
            .saturating_add(2)
    });
    (work, passes)
}

#[test]
fn literal_flat_class_domains_full_proofs_and_work_prefixes() {
    let long_literal = format!(r"^\u{{{}61}}", "0".repeat(64));
    let long_source = format!("{long_literal}[]$");
    for (text, literal, class, i, m, supported) in [
        ("a[b]", "a", "[b]", false, false, true),
        ("^ſ[K]$", "^ſ", "[K]$", true, true, true),
        ("s[^K]", "s", "[^K]", true, false, true),
        (
            r"\uD83D\uDE00[\p{Any}]",
            r"\uD83D\uDE00",
            r"[\p{Any}]",
            false,
            false,
            true,
        ),
        (r"\uD800[a]", r"\uD800", "[a]", false, false, true),
        (
            long_source.as_str(),
            long_literal.as_str(),
            "[]$",
            true,
            false,
            true,
        ),
        ("a[[b]]", "a", "[[b]]", false, false, false),
        ("a[b--b]", "a", "[b--b]", false, false, false),
        (
            r"a[^\p{Any}\p{RGI_Emoji}]",
            "a",
            r"[^\p{Any}\p{RGI_Emoji}]",
            false,
            false,
            false,
        ),
    ] {
        let source = JsString::from(text);
        let (expected, passes) = expected_work(&source, literal, class, i, m);
        assert_eq!(passes.is_some(), supported, "{text}");
        let mut work = Vec::new();
        let matcher = compile_recording_work(&source, i, m, &mut work, None).unwrap();
        assert_eq!(work, expected, "{text}");
        assert_eq!(matcher.is_some(), supported, "{text}");
        if let Some(matcher) = matcher {
            assert_eq!(Some(matcher.search_passes()), passes);
        }
        for fail in 1..=expected.len() {
            let mut work = Vec::new();
            assert!(
                matches!(compile_recording_work(&source, i, m, &mut work, Some(fail)), Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(work, expected[..fail], "{text} fail={fail}");
            let mut fresh_work = Vec::new();
            let fresh = compile_recording_work(&source, i, m, &mut fresh_work, None).unwrap();
            assert_eq!(fresh_work, expected);
            assert_eq!(fresh.is_some(), supported);
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
        }
    }

    for text in [
        "",
        "a",
        "[a]b",
        "a[b][c]",
        "ab[c]",
        "a[b]c",
        "a[b].",
        r"a\d",
        r"a\p{Any}",
        "a^[b]",
        "a[b]$c",
        "^^a[b]",
        "a[b]$$",
        r"\ba[b]",
        r"a[b]\b",
        r"a\b[b]",
        "(a[b])",
        "(a)[b]",
        "a([b])",
        "a+[b]",
        "a[b]?",
        "(?:a[b])+",
        r"a[\q{ab}]",
        r"a[b&&\q{b}]",
        r"\u{D800}\u{DC00}[a]",
        r"a[b]\$",
    ] {
        let source = JsString::from(text);
        let mut work = Vec::new();
        assert!(
            compile_recording_work(&source, false, false, &mut work, None)
                .unwrap()
                .is_none()
        );
        assert_eq!(work, [source.len().saturating_mul(3)], "{text}");
        let mut abort = Vec::new();
        assert!(matches!(
            compile_recording_work(&source, false, false, &mut abort, Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
        assert_eq!(abort, work);
    }
    for (source, input, range) in [
        (r"\$[\]]", JsString::from("$]"), Some(0..2)),
        (r"\][a]", JsString::from("]a"), Some(0..2)),
        (r"\/[a]", JsString::from("/a"), Some(0..2)),
        (r"\\[a]", JsString::from("\\a"), Some(0..2)),
        (r"\u0061[b]", JsString::from("ab"), Some(0..2)),
        (r"\u{000061}[b]", JsString::from("ab"), Some(0..2)),
        (
            r"\uD83D\uDE00[\u{1f601}]",
            JsString::from("😀😁"),
            Some(0..4),
        ),
        ("😀[😁]", JsString::from("😀😁"), Some(0..4)),
        (r"\u{1f600}[\u{1f601}]", JsString::from("😀😁"), Some(0..4)),
        (
            r"\uD800[\uDC00]",
            JsString::from_code_units(vec![0xd800, 0xdc00]),
            None,
        ),
        (
            r"\uD800[a]",
            JsString::from_code_units(vec![0xd800, 0x61]),
            Some(0..2),
        ),
        (
            r"a[\uD800]",
            JsString::from_code_units(vec![0x61, 0xd800]),
            Some(0..2),
        ),
        (r"^\$[a]$", JsString::from("$a"), Some(0..2)),
        (r"\u0024[a]$", JsString::from("$a"), Some(0..2)),
    ] {
        assert_eq!(
            compile(source, false, false).unwrap().find(&input, 0, true),
            range,
            "{source}"
        );
    }
    let matcher = compile("a[b]", false, false).unwrap();
    let input = JsString::from("aab!");
    assert_eq!(matcher.find(&input, 0, false), Some(1..3));
    assert_eq!(matcher.find(&input, 0, true), None);
    assert_eq!(matcher.find(&input, 1, true), Some(1..3));
    let mut entry = Vec::new();
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |n| {
            entry.push(n);
            Err::<(), usize>(77)
        }),
        Err(77)
    );
    assert_eq!(entry, [1]);
    let mut entry = Vec::new();
    assert_eq!(
        matcher
            .find_with_work(&input, 0, false, |n| {
                entry.push(n);
                Ok::<(), usize>(())
            })
            .unwrap(),
        Some(1..3)
    );
    assert_eq!(entry, [1]);
    let pair = compile(r"\u{1f600}[\p{Any}]", false, false).unwrap();
    assert_eq!(pair.find(&JsString::from("A😀😁!"), 2, true), Some(1..5));
}
