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
        // Decode original UTF-16 with std, independently of matcher helpers.
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

// Complete simple/common fold facts for the selected points only. The oracle
// does not read production folding, class membership, or character boundaries.
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
    // The approved initial-low-surrogate behavior selects its complete point.
    let first = input.boundaries.partition_point(|&at| at <= start) - 1;
    let canonical = |point| if case.ignore_case { fold(point) } else { point };
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..input.points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        let point = input.points[at];
        let member = match case.set {
            Set::Point(value) => canonical(point) == canonical(value),
            Set::Any => true,
            Set::NotK => canonical(point) != canonical(0x4b),
            Set::Empty => false,
        };
        let begin = input.boundaries[at];
        let end = input.boundaries[at + 2];
        if member
            && canonical(input.points[at + 1]) == canonical(case.literal)
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

// One concrete callback instantiation for every new constructor transcript.
fn compile_recording_work(
    source: &JsString,
    i: bool,
    m: bool,
    work: &mut Vec<usize>,
    fail: Option<usize>,
) -> Result<Option<RegExpUnicodeClassLiteralMatcher>, RegExpUnicodeSetCompileError<usize>> {
    RegExpUnicodeClassLiteralMatcher::compile_flat_v_set_then_literal_with_work(
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
                    input.input,
                );
                queries += 1;
            }
        }
    }
    queries
}

#[test]
fn flat_class_literal_matches_independent_tiny_utf16_oracle() {
    use Set::{Any, Empty, NotK, Point};
    let inputs = [
        JsString::from(""),
        JsString::from("aaab!"),
        JsString::from("KſKs!ks"),
        JsString::from("Ksſs!"),
        JsString::from("αA!Aa"),
        JsString::from("😀😁a"),
        JsString::from("x😀😁!"),
        JsString::from_code_units(vec![0xd800, 0x61, 0x61, 0xd800, 0x21]),
        JsString::from("a\r\nab\u{2028}b"),
        JsString::from("ab"),
    ]
    .map(OracleInput::new);
    let cases = [
        (r"[a]b", Point(0x61), 0x62, false),
        (r"[K]s", Point(0x4b), 0x73, true),
        (r"[K]s", Point(0x4b), 0x73, false),
        (r"[^K]s", NotK, 0x73, true),
        (r"[^K]s", NotK, 0x73, false),
        (r"[\p{Any}]a", Any, 0x61, true),
        (r"[]a", Empty, 0x61, false),
        (r"[\u{1f600}]\u{1f601}", Point(0x1f600), 0x1f601, false),
        (r"[\uD800]a", Point(0xd800), 0x61, false),
        (r"[a]\uD800", Point(0x61), 0xd800, false),
    ];
    let mut queries = 0;
    for (source, set, literal, ignore_case) in cases {
        queries += check_case(
            &Case {
                source,
                set,
                literal,
                ignore_case,
                multiline: false,
                anchors: [false, false],
            },
            &inputs,
        );
    }
    for (source, anchors) in [
        ("[a]b", [false, false]),
        ("^[a]b", [true, false]),
        ("[a]b$", [false, true]),
        ("^[a]b$", [true, true]),
    ] {
        for multiline in [false, true] {
            queries += check_case(
                &Case {
                    source,
                    set: Point(0x61),
                    literal: 0x62,
                    ignore_case: false,
                    multiline,
                    anchors,
                },
                &inputs,
            );
        }
    }
    // Ten inputs have 47 code units: 2*(47+2*10)=134 queries per case.
    assert_eq!(queries, (10 + 4 * 2) * 134);
}

// Component slices are supplied explicitly, not rediscovered using production
// framing. Preserve ^ in the class and $ in the literal. A successful empty
// class still continues through the complete literal proof and its work.
fn expected_work(
    source: &JsString,
    class: &str,
    literal: &str,
    i: bool,
    m: bool,
) -> (Vec<usize>, Option<usize>) {
    let class = JsString::from(class);
    let literal = JsString::from(literal);
    let mut work = vec![source.len().saturating_mul(3)];
    let Some(class) = RegExpUnicodeClassMatcher::compile_flat_v_set_units_with_work(
        class.code_units(),
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
    let literal = RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
        literal.code_units(),
        i,
        m,
        |amount| {
            work.push(amount);
            Ok::<(), usize>(())
        },
    )
    .unwrap();
    let passes = literal.map(|literal| {
        class
            .search_passes()
            .saturating_add(literal.search_passes())
            .saturating_add(2)
    });
    (work, passes)
}

#[test]
fn flat_class_literal_domains_full_proofs_and_work_prefixes() {
    let long_literal = format!(r"\u{{{}61}}", "0".repeat(64));
    let long_source = format!("[]{long_literal}");
    for (text, class, literal, i, m, supported) in [
        ("[a]b", "[a]", "b", false, false, true),
        ("^[K]ſ$", "^[K]", "ſ$", true, true, true),
        ("[^K]s", "[^K]", "s", true, false, true),
        (
            r"[\p{Any}]\uD83D\uDE01",
            r"[\p{Any}]",
            r"\uD83D\uDE01",
            false,
            false,
            true,
        ),
        (r"[\uD800]a", r"[\uD800]", "a", false, false, true),
        (
            long_source.as_str(),
            "[]",
            long_literal.as_str(),
            true,
            false,
            true,
        ),
        ("[[a]]b", "[[a]]", "b", false, false, false),
        ("[a--a]b", "[a--a]", "b", false, false, false),
        (
            r"[\p{RGI_Emoji}]a",
            r"[\p{RGI_Emoji}]",
            "a",
            false,
            false,
            false,
        ),
    ] {
        let source = JsString::from(text);
        let (expected, passes) = expected_work(&source, class, literal, i, m);
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
            assert!(matches!(
                compile_recording_work(&source, i, m, &mut work, Some(fail)),
                Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail
            ));
            assert_eq!(work, expected[..fail], "{text} fail={fail}");
            // A fresh compile must not retain any aborted work or partial plan.
            let mut fresh_work = Vec::new();
            let fresh = compile_recording_work(&source, i, m, &mut fresh_work, None).unwrap();
            assert_eq!(fresh_work, expected);
            assert_eq!(fresh.is_some(), supported);
        }
        if supported {
            // Earlier checked-only entry contracts and callbacks are untouched.
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(&source, i, m, |_| Ok::<
                    (),
                    usize,
                >(
                    ()
                ),)
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
                    &source,
                    i,
                    m,
                    |_| Ok::<(), usize>(()),
                )
                .unwrap()
                .is_none()
            );
            assert!(
                RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                    &source,
                    i,
                    m,
                    |_| Ok::<(), usize>(()),
                )
                .unwrap()
                .is_none()
            );
        }
    }

    for source in [
        "c[a]",
        "[a][b]",
        "[a]bc",
        "[a].",
        r"[a]\d",
        r"[a]\p{Any}",
        "[a]^b",
        "[a]b$c",
        "^^[a]b",
        "[a]b$$",
        r"\b[a]b",
        r"[a]b\b",
        "([a]b)",
        "([a])b",
        "[a](b)",
        "[a]+b",
        "[a]b?",
        "(?:[a]b)+",
        "[]bc",
        r"[]\d",
        r"[]\p{Any}",
        r"[\q{ab}]c",
        r"[a&&\q{a}]b",
        r"[a]\u{D800}\u{DC00}",
    ] {
        let source = JsString::from(source);
        let mut work = Vec::new();
        assert!(
            compile_recording_work(&source, false, false, &mut work, None)
                .unwrap()
                .is_none()
        );
        // These neighbors fail complete framing, before either component proof.
        assert_eq!(work, [source.len().saturating_mul(3)]);
        let mut abort = Vec::new();
        assert!(matches!(
            compile_recording_work(&source, false, false, &mut abort, Some(1)),
            Err(RegExpUnicodeSetCompileError::Work(1))
        ));
        assert_eq!(abort, work);
    }

    for (source, input, range) in [
        (r"[a]\$", JsString::from("a$"), Some(0..2)),
        (r"[a]\]", JsString::from("a]"), Some(0..2)),
        (r"[a]\/", JsString::from("a/"), Some(0..2)),
        (r"[a]\\", JsString::from("a\\"), Some(0..2)),
        (r"[a]\u0062", JsString::from("ab"), Some(0..2)),
        (r"[a]\u{000062}", JsString::from("ab"), Some(0..2)),
        (
            r"[\u{1f600}]\uD83D\uDE01",
            JsString::from("😀😁"),
            Some(0..4),
        ),
        ("[😀]😁", JsString::from("😀😁"), Some(0..4)),
        (r"[\u{1f600}]\u{1f601}", JsString::from("😀😁"), Some(0..4)),
        (
            r"[\uD800]\uDC00",
            JsString::from_code_units(vec![0xd800, 0xdc00]),
            None,
        ),
        (
            r"[\uD800]a",
            JsString::from_code_units(vec![0xd800, 0x61]),
            Some(0..2),
        ),
        (
            r"[a]\uD800",
            JsString::from_code_units(vec![0x61, 0xd800]),
            Some(0..2),
        ),
        (r"^[a]\$$", JsString::from("a$"), Some(0..2)),
        (r"[a]\u0024$", JsString::from("a$"), Some(0..2)),
    ] {
        assert_eq!(
            compile(source, false, false).unwrap().find(&input, 0, true),
            range,
            "{source}"
        );
    }

    let matcher = compile("[a]b", false, false).unwrap();
    let input = JsString::from("aaab!");
    assert_eq!(matcher.find(&input, 0, false), Some(2..4));
    assert_eq!(matcher.find(&input, 0, true), None);
    assert_eq!(matcher.find(&input, 2, true), Some(2..4));
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
        Some(2..4)
    );
    assert_eq!(entry, [1]);
    let pair = compile(r"[\u{1f600}]\u{1f601}", false, false).unwrap();
    assert_eq!(pair.find(&JsString::from("😀😁"), 1, true), Some(0..4));
}
