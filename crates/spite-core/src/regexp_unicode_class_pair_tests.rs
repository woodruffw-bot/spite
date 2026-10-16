use super::*;
use crate::RegExpUnicodeClassLiteralMatcher;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Set {
    A,
    B,
    C,
    K,
    S,
    NotK,
    CapitalWithoutLower,
    Any,
    Empty,
    High,
    Low,
    Face,
    NextFace,
    Close,
    Open,
}

struct Case<'a> {
    source: &'a str,
    sets: [Set; 2],
    ignore_case: bool,
    multiline: bool,
    anchored: bool,
}

// Independent finite facts: ASCII common/simple folds and the two selected
// non-ASCII preimages. These tests never enumerate Unicode or use a production
// matcher/decoder to compute the expected range.
fn fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 0x20,
        0x17f => 0x73,
        0x212a => 0x6b,
        _ => point,
    }
}

fn contains(set: Set, point: u32, ignore_case: bool) -> bool {
    let canonical = if ignore_case { fold(point) } else { point };
    match set {
        Set::A => canonical == 0x61,
        Set::B => canonical == 0x62,
        Set::C => canonical == 0x63,
        Set::K => canonical == if ignore_case { 0x6b } else { 0x4b },
        Set::S => canonical == 0x73,
        Set::NotK => canonical != if ignore_case { 0x6b } else { 0x4b },
        // In vi, P{Ll} complements folded membership and therefore excludes A.
        Set::CapitalWithoutLower => !ignore_case && point == 0x41,
        Set::Any => true,
        Set::Empty => false,
        Set::High => point == 0xd800,
        Set::Low => point == 0xdc00,
        Set::Face => point == 0x1f600,
        Set::NextFace => point == 0x1f601,
        Set::Close => point == 0x5d,
        Set::Open => point == 0x5b,
    }
}

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
    // The narrowly approved initial-low-surrogate behavior checks the leading
    // whole-character boundary, never the interior of its UTF-16 encoding.
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
    RegExpUnicodeClassPairMatcher::compile_v_set_with_work(
        &JsString::from(source),
        ignore_case,
        multiline,
        |_| Ok::<(), Infallible>(()),
    )
    .unwrap()
}

#[test]
fn checked_v_set_class_pair_ranges_match_sparse_reference() {
    use Set::*;
    let cases = [
        (r"[[a]][[b]]", [A, B], false, false, false),
        (r"[a--b][[c]]", [A, C], false, false, false),
        (r"^[[a]][[b]]$", [A, B], false, false, true),
        (r"^[[a]][[b]]$", [A, B], false, true, true),
        (r"[[K]][[s]]", [K, S], true, false, false),
        (r"[[K]][[s]]", [K, S], false, false, false),
        (r"[[^K]][[s]]", [NotK, S], true, false, false),
        (
            r"[\P{Ll}&&A][[s]]",
            [CapitalWithoutLower, S],
            false,
            false,
            false,
        ),
        (
            r"[\P{Ll}&&A][[s]]",
            [CapitalWithoutLower, S],
            true,
            false,
            false,
        ),
        (r"[[\p{Any}]][[a]]", [Any, A], false, false, false),
        (r"[a--a][[b]]", [Empty, B], false, false, false),
        (r"[[a]][b--b]", [A, Empty], false, false, false),
        (r"[[\uD800]][[\uD800]]", [High, High], false, false, false),
        (r"[[\uD800]][[\uDC00]]", [High, Low], false, false, false),
        (
            r"[[\u{1f600}]][[\u{1f601}]]",
            [Face, NextFace],
            false,
            false,
            false,
        ),
        (r"[[\]]][[\[]]", [Close, Open], false, false, false),
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
        JsString::from("Asas"),
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
fn checked_v_set_class_pair_domains_and_work_prefixes() {
    for source in [
        "",
        "ab",
        "[a][b]",
        "[a--b][c]",
        "[a][b--c]",
        "[[a]]",
        "[[a]][[b]][[c]]",
        "[[a]]b[[c]]",
        "a[[b]][[c]]",
        "[[a]][[b]]c",
        "[[a]]^[[b]]",
        "[[a]]$[[b]]",
        "^^[[a]][[b]]",
        "[[a]][[b]]$$",
        r"\b[[a]][[b]]",
        r"[[a]][[b]]\b",
        "([[a]])[[b]]",
        "([[a]][[b]])",
        "[[a]]+[[b]]",
        "[[a]][[b]]?",
        "([[a]][[b]])+",
        r"[[\q{ab}]][[c]]",
        r"[a--a][[\q{b}]]",
        r"[[\q{a}]][b--b]",
        r"[a--a][[\p{RGI_Emoji}]]",
        r"[[\p{RGI_Emoji}]][b--b]",
    ] {
        assert!(compile(source, false, false).is_none(), "{source}");
    }
    for source in [
        r"[[a]][[b]]",
        r"^[[K]][[s]]$",
        r"[[\]]][[\[]]",
        r"[a--a][[b]]",
    ] {
        let source = JsString::from(source);
        for old in [
            RegExpUnicodeClassLiteralMatcher::compile_v_set_with_work(&source, true, false, |_| {
                Ok::<(), Infallible>(())
            })
            .unwrap(),
            RegExpUnicodeClassLiteralMatcher::compile_literal_then_v_set_with_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap(),
            RegExpUnicodeClassLiteralMatcher::compile_v_set_with_word_boundaries_and_work(
                &source,
                true,
                false,
                |_| Ok::<(), Infallible>(()),
            )
            .unwrap(),
        ] {
            assert!(old.is_none(), "{source:?}");
        }
    }
    for (text, ignore_case) in [(r"[a--b][[c]]", false), (r"^[[K]][[s]]$", true)] {
        let source = JsString::from(text);
        let mut complete = Vec::new();
        assert!(
            RegExpUnicodeClassPairMatcher::compile_v_set_with_work(
                &source,
                ignore_case,
                false,
                |amount| {
                    complete.push(amount);
                    Ok::<(), usize>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(complete.first(), Some(&source.len().saturating_mul(2)));
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassPairMatcher::compile_v_set_with_work(
                &source,
                ignore_case,
                false,
                |amount| {
                    seen.push(amount);
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
    let matcher = compile("[a--b][[c]]", false, false).unwrap();
    let input = JsString::from("aac");
    let mut charges = Vec::new();
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |amount| {
            charges.push(amount);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..3))
    );
    assert_eq!(charges, [1]);
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |_| Err(17)),
        Err(17)
    );
    assert_eq!(matcher.find(&input, 0, true), None);
    assert_eq!(matcher.find(&input, 1, true), Some(1..3));
    assert_eq!(
        matcher.search_passes(),
        matcher.classes[0]
            .search_passes()
            .saturating_add(matcher.classes[1].search_passes())
            .saturating_add(2)
    );
    let emoji = compile(r"[[\u{1f600}]][[\u{1f601}]]", false, false).unwrap();
    assert_eq!(emoji.find(&JsString::from("😀😁"), 1, true), Some(0..4));
    assert_eq!(
        compile(r"[[\uD800]][[\uDC00]]", false, false)
            .unwrap()
            .find(&JsString::from_code_units(vec![0xd800, 0xdc00]), 0, false),
        None
    );
}
