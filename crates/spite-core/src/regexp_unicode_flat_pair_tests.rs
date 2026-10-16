use super::*;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Set {
    A,
    B,
    C,
    K,
    S,
    NotK,
    UpperA,
    NotA,
    Any,
    Empty,
    Abc,
    Ascii,
    High,
    Low,
    Face,
    NextFace,
}
struct Case<'a> {
    source: &'a str,
    sets: [Set; 2],
    ignore_case: bool,
    multiline: bool,
    anchored: bool,
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
        Set::C => value == 0x63,
        Set::K => value == if ignore_case { 0x6b } else { 0x4b },
        Set::S => value == 0x73,
        Set::NotK => value != if ignore_case { 0x6b } else { 0x4b },
        Set::UpperA => value == if ignore_case { 0x61 } else { 0x41 },
        Set::NotA => value != 0x61,
        Set::Any => true,
        Set::Empty => false,
        Set::Abc => (0x61..=0x63).contains(&value),
        Set::Ascii => point <= 0x7f,
        Set::High => point == 0xd800,
        Set::Low => point == 0xdc00,
        Set::Face => point == 0x1f600,
        Set::NextFace => point == 0x1f601,
    }
}

// Independent finite original-character oracle: standard UTF-16 decoding,
// manually selected memberships/fold classes, and original line context.
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
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..points.len().saturating_sub(1) {
        if sticky && at != first {
            break;
        }
        let begin = boundaries[at];
        let end = boundaries[at + 2];
        if contains(case.sets[0], points[at], case.ignore_case)
            && contains(case.sets[1], points[at + 1], case.ignore_case)
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
    RegExpUnicodeClassPairMatcher::compile_flat_pair_with_work(
        &JsString::from(source),
        ignore_case,
        multiline,
        |_| Ok::<(), Infallible>(()),
    )
    .unwrap()
}

#[test]
fn flat_v_set_pair_ranges_match_sparse_reference() {
    use Set::*;
    let cases = [
        (r"[a][c]", [A, C], false, false, false),
        (r"^[a][b]$", [A, B], false, false, true),
        (r"^[a][b]$", [A, B], false, true, true),
        (r"[K][s]", [K, S], true, false, false),
        (r"[K][s]", [K, S], false, false, false),
        (r"[^K][s]", [NotK, S], true, false, false),
        (r"[A][^a]", [UpperA, NotA], false, false, false),
        (r"[A][^a]", [UpperA, NotA], true, false, false),
        (r"[\p{Any}][a]", [Any, A], false, false, false),
        (r"[][b]", [Empty, B], false, false, false),
        (r"[a][^]", [A, Any], false, false, false),
        (r"[a-c][b]", [Abc, B], true, false, false),
        (r"[\p{ASCII}][a]", [Ascii, A], false, false, false),
        (r"[\uD800][\uD800]", [High, High], false, false, false),
        (r"[\uD800][\uDC00]", [High, Low], false, false, false),
        (
            r"[\u{1f600}][\u{1f601}]",
            [Face, NextFace],
            false,
            false,
            false,
        ),
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("a"),
        JsString::from("b"),
        JsString::from("ab"),
        JsString::from("aaab"),
        JsString::from("aac"),
        JsString::from("aaac"),
        JsString::from("x\nab\nab"),
        JsString::from("ab\n"),
        JsString::from("x\r\nab\u{2028}cd"),
        JsString::from("KſKSks"),
        JsString::from("AAas"),
        JsString::from("Aſ"),
        JsString::from("😀😁"),
        JsString::from("x😀😁y"),
        JsString::from("][]["),
        JsString::from("!"),
        JsString::from_code_units(vec![0xd800, 0xd800]),
        JsString::from_code_units(vec![0x41, 0xd800, 0xdc00, 0xdc00]),
        JsString::from_code_units(vec![0xd83d, 0xde00, 0x61]),
    ];
    let mut queries = 0;
    for (source, sets, ignore_case, multiline, anchored) in cases {
        let case = Case {
            source,
            sets,
            ignore_case,
            multiline,
            anchored,
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
fn flat_v_set_pair_domains_and_work_prefixes() {
    for text in [
        "",
        "[a]",
        "[a][b][c]",
        "c[a][b]",
        "[a]c[b]",
        "[a]^[b]",
        "[a][b]$$",
        r"\b[a][b]",
        r"[a][b]\b",
        "([a])[b]",
        "[a]([b])",
        "[a]+[b]",
        "[a][b]?",
        "[[a]][b]",
        "[a--b][c]",
        "[a][[b]]",
        "[a][b--c]",
        "[][[b]]",
        r"[][\q{b}]",
        r"[][\p{RGI_Emoji}]",
        r"[\q{a}][b]",
        r"[\p{RGI_Emoji}][b]",
    ] {
        assert!(compile(text, false, false).is_none(), "{text}");
    }
    for text in [r"[a][c]", r"^[K][s]$", r"[][b]"] {
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
    }
    for (text, first, second, ignore_case) in [
        (r"[a][c]", r"[a]", r"[c]", false),
        (r"^[K][s]$", r"^[K]", r"[s]$", true),
        (r"[][b]", r"[]", r"[b]", false),
        (r"[a][]", r"[a]", r"[]", false),
    ] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeClassPairMatcher::compile_flat_pair_with_work(
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
        assert!(plan.assertions.is_none());
        let mut expected_work = vec![source.len().saturating_mul(2)];
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
            "source-order flat then flat work: {text}"
        );
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassPairMatcher::compile_flat_pair_with_work(
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
    let plan = compile("[a][c]", false, false).unwrap();
    let input = JsString::from("aac");
    let mut charges = Vec::new();
    assert_eq!(
        plan.find_with_work(&input, 0, false, |work| {
            charges.push(work);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..3))
    );
    assert_eq!(charges, [1]);
    assert_eq!(plan.find_with_work(&input, 0, false, |_| Err(29)), Err(29));
    assert_eq!(plan.find(&input, 0, true), None);
    assert_eq!(plan.find(&input, 1, true), Some(1..3));
    assert_eq!(
        plan.search_passes(),
        plan.classes[0]
            .search_passes()
            .saturating_add(plan.classes[1].search_passes())
            .saturating_add(2)
    );
    assert_eq!(
        compile(r"[\p{Any}][a]", false, false)
            .unwrap()
            .find(&JsString::from("😀a"), 1, true),
        Some(0..3)
    );
    assert_eq!(
        compile(r"[\uD800][\uDC00]", false, false).unwrap().find(
            &JsString::from_code_units(vec![0xd800, 0xdc00]),
            0,
            false
        ),
        None
    );
    assert_eq!(
        compile(r"^[\^][\$]$", false, false)
            .unwrap()
            .find(&JsString::from("^$"), 0, true),
        Some(0..2)
    );
    assert!(compile("[][]", false, false).is_some());
    assert_eq!(
        compile(r"[\P{Ll}][s]", false, false)
            .unwrap()
            .find(&JsString::from("As!"), 0, false),
        Some(0..2)
    );
    assert_eq!(
        compile(r"[\P{Ll}][s]", true, false)
            .unwrap()
            .find(&JsString::from("As!"), 0, false),
        None
    );
}
