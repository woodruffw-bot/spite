use super::*;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Set {
    A,
    B,
    K,
    S,
    Any,
    Empty,
    High,
    Low,
    Face,
    NextFace,
    Bang,
    Question,
}

struct Case<'a> {
    source: &'a str,
    sets: [Set; 2],
    ignore_case: bool,
    multiline: bool,
    anchored: bool,
    start_word: Option<bool>,
    end_word: Option<bool>,
}

fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
}

fn contains(set: Set, point: u32, ignore_case: bool) -> bool {
    let value = if ignore_case { fold(point) } else { point };
    match set {
        Set::A => value == 0x61,
        Set::B => value == 0x62,
        Set::K => value == if ignore_case { 0x6b } else { 0x4b },
        Set::S => value == 0x73,
        Set::Any => true,
        Set::Empty => false,
        Set::High => point == 0xd800,
        Set::Low => point == 0xdc00,
        Set::Face => point == 0x1f600,
        Set::NextFace => point == 0x1f601,
        Set::Bang => point == 0x21,
        Set::Question => point == 0x3f,
    }
}

fn word(point: u32, ignore_case: bool) -> bool {
    let point = if ignore_case { fold(point) } else { point };
    matches!(point, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
}

// Independently decoded points and their neighboring word membership determine
// boundaries. No production matcher, decoder or word helper computes this oracle.
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
    let first = boundaries.partition_point(|&at| at <= start) - 1;
    let boundary = |at: usize| {
        let before = at
            .checked_sub(1)
            .is_some_and(|i| word(points[i], case.ignore_case));
        let after = points.get(at).is_some_and(|&p| word(p, case.ignore_case));
        before != after
    };
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        let begin = boundaries[at];
        let end = boundaries[at + 2];
        if contains(case.sets[0], points[at], case.ignore_case)
            && contains(case.sets[1], points[at + 1], case.ignore_case)
            && case.start_word.is_none_or(|wanted| boundary(at) == wanted)
            && case
                .end_word
                .is_none_or(|wanted| boundary(at + 2) == wanted)
            && (!case.anchored
                || begin == 0
                || (case.multiline && line(input.code_units()[begin - 1])))
            && (!case.anchored
                || end == input.len()
                || (case.multiline && line(input.code_units()[end])))
        {
            return Some(begin..end);
        }
    }
    None
}

fn compile(
    source: &str,
    ignore_case: bool,
    multiline: bool,
) -> Option<RegExpUnicodeClassPairMatcher> {
    RegExpUnicodeClassPairMatcher::compile_flat_pair_with_word_boundaries_and_work(
        &JsString::from(source),
        ignore_case,
        multiline,
        |_| Ok::<(), Infallible>(()),
    )
    .unwrap()
}

#[test]
fn flat_v_set_pair_word_ranges_match_sparse_reference() {
    use Set::*;
    let cases = [
        (
            r"\B[a][a]\b",
            [A, A],
            false,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (r"\b[a][b]", [A, B], false, false, false, Some(true), None),
        (r"[a][b]\b", [A, B], false, false, false, None, Some(true)),
        (
            r"^\b[a][b]\b$",
            [A, B],
            false,
            true,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"^\b[a][b]\b$",
            [A, B],
            false,
            false,
            true,
            Some(true),
            Some(true),
        ),
        (
            r"\b[K][s]\b",
            [K, S],
            true,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\b[K][s]\b",
            [K, S],
            false,
            false,
            false,
            Some(true),
            Some(true),
        ),
        (
            r"\B[\p{Any}][a]\b",
            [Any, A],
            false,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\B[\p{Any}][a]\b",
            [Any, A],
            true,
            false,
            false,
            Some(false),
            Some(true),
        ),
        (
            r"\b[a][\p{Any}]\B",
            [A, Any],
            false,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\b[a][\p{Any}]\B",
            [A, Any],
            true,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B[!][\?]\B",
            [Bang, Question],
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\b[][b]",
            [Empty, B],
            false,
            false,
            false,
            Some(true),
            None,
        ),
        (
            r"\b[\uD800][\uD800]\B",
            [High, High],
            false,
            false,
            false,
            Some(true),
            Some(false),
        ),
        (
            r"\B[\uD800][\uDC00]\B",
            [High, Low],
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
        (
            r"\B[\u{1f600}][\u{1f601}]\B",
            [Face, NextFace],
            false,
            false,
            false,
            Some(false),
            Some(false),
        ),
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("a"),
        JsString::from("aa"),
        JsString::from("aaa!"),
        JsString::from("ab!"),
        JsString::from("xab!"),
        JsString::from("aa ab!"),
        JsString::from("Kſ!"),
        JsString::from("Kſ!"),
        JsString::from("as!"),
        JsString::from("αa!"),
        JsString::from("!?"),
        JsString::from("aK!"),
        JsString::from("aſ!"),
        JsString::from("x\r\nab\u{2028}cd"),
        JsString::from("😀a!"),
        JsString::from("!😀😁!"),
        JsString::from_code_units(vec![0x41, 0xd800, 0xd800, 0x21]),
        JsString::from_code_units(vec![0x41, 0xd800, 0xdc00, 0xdc00]),
        JsString::from("a$!"),
    ];
    let mut queries = 0;
    for (source, sets, ignore_case, multiline, anchored, start_word, end_word) in cases {
        let case = Case {
            source,
            sets,
            ignore_case,
            multiline,
            anchored,
            start_word,
            end_word,
        };
        let matcher = compile(source, ignore_case, multiline).unwrap();
        for input in &inputs {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(input, start, sticky),
                        expected(&case, input, start, sticky),
                        "source={} input={input:?} start={start} sticky={sticky}",
                        case.source
                    );
                    queries += 1;
                }
            }
        }
    }
    assert_eq!(queries, 3488);
}

#[test]
fn flat_v_set_pair_word_domains_and_work_prefixes() {
    for text in [
        "",
        "[a][b]",
        "^[a][b]$",
        r"\b[[a]][b]",
        r"\b[a][[b]]",
        r"\b[a--b][c]",
        r"\b[a][b--c]",
        r"\b[a]",
        r"\b[a][b][c]",
        r"\b[a]c",
        r"\bc[a]",
        r"\b^[a][b]",
        r"[a][b]$\b",
        r"\b\b[a][b]",
        r"[a]\b[b]",
        r"\b[a]^[b]",
        r"\b[a][b]$$",
        r"\b([a])[b]",
        r"\b[a]([b])",
        r"(\b[a][b])",
        r"\b[a]+[b]",
        r"\b[a][b]?",
        r"\b[a][b]c",
        r"\b[][[b]]",
        r"\b[][\q{b}]",
        r"\b[][\p{RGI_Emoji}]",
        r"\b[\q{a}][]",
        r"\b[\p{RGI_Emoji}][]",
    ] {
        assert!(compile(text, false, false).is_none(), "{text}");
    }
    for text in [r"\b[a][b]", r"[a][b]\b", r"^\b[K][s]\b$", r"\b[][b]"] {
        let source = JsString::from(text);
        assert!(
            RegExpUnicodeClassPairMatcher::compile_v_set_with_work(&source, true, false, |_| Ok::<
                (),
                Infallible,
            >(
                ()
            ),)
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassPairMatcher::compile_v_set_with_word_boundaries_and_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassPairMatcher::compile_v_set_then_flat_with_word_boundaries_and_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassPairMatcher::compile_flat_then_v_set_with_word_boundaries_and_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeClassPairMatcher::compile_flat_pair_with_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap()
            .is_none()
        );
    }
    for (text, first, second, ignore_case) in [
        (r"\b[a][b]", r"[a]", r"[b]", false),
        (r"^\b[K][s]\b$", r"[K]", r"[s]", true),
        (r"\b[][b]", r"[]", r"[b]", false),
        (r"\b[a][]", r"[a]", r"[]", false),
    ] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeClassPairMatcher::compile_flat_pair_with_word_boundaries_and_work(
            &source,
            ignore_case,
            false,
            |work| {
                complete.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert!(plan.assertions.is_some());
        let mut expected_work = vec![source.len().saturating_mul(3)];
        for component in [first, second] {
            RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &JsString::from(component),
                true,
                ignore_case,
                false,
                |work| {
                    expected_work.push(work);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .unwrap();
        }
        assert_eq!(
            complete, expected_work,
            "source-order word flat then flat work: {text}"
        );
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                RegExpUnicodeClassPairMatcher::compile_flat_pair_with_word_boundaries_and_work(
                    &source,
                    ignore_case,
                    false,
                    |work| {
                        seen.push(work);
                        if seen.len() == fail {
                            Err(fail)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(matches!(result, Err(RegExpUnicodeSetCompileError::Work(at)) if at == fail));
            assert_eq!(seen, complete[..fail]);
        }
    }
    let plan = compile(r"\B[a][a]\b", false, false).unwrap();
    let input = JsString::from("aaa!");
    let mut charges = Vec::new();
    assert_eq!(
        plan.find_with_work(&input, 0, false, |work| {
            charges.push(work);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..3))
    );
    assert_eq!(charges, [1]);
    assert_eq!(plan.find_with_work(&input, 0, false, |_| Err(23)), Err(23));
    assert_eq!(plan.find(&input, 0, true), None);
    assert_eq!(plan.find(&input, 1, true), Some(1..3));
    let bare = RegExpUnicodeClassPairMatcher::compile_flat_pair_with_work(
        &JsString::from("[a][b]"),
        false,
        false,
        |_| Ok::<(), Infallible>(()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        compile(r"\b[a][b]", false, false).unwrap().search_passes(),
        bare.search_passes() + 8
    );
    assert_eq!(
        compile(r"^\b[a][b]\b$", false, false)
            .unwrap()
            .search_passes(),
        bare.search_passes() + 18
    );
    assert_eq!(
        compile(r"\B[\p{Any}][a]\b", false, false)
            .unwrap()
            .find(&JsString::from("😀a!"), 1, true),
        Some(0..3)
    );
    assert_eq!(
        compile(r"\B[\uD800][\uDC00]\B", false, false)
            .unwrap()
            .find(&JsString::from_code_units(vec![0xd800, 0xdc00]), 0, false),
        None
    );
    assert_eq!(
        compile(r"\b[\P{Ll}][s]\b", false, false)
            .unwrap()
            .find(&JsString::from("As!"), 0, false),
        Some(0..2)
    );
    assert_eq!(
        compile(r"\b[\P{Ll}][s]\b", true, false)
            .unwrap()
            .find(&JsString::from("As!"), 0, false),
        None
    );
}
