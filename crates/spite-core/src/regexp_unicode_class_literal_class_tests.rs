use super::*;
use std::convert::Infallible;

#[derive(Clone, Copy)]
enum Set {
    Point(u32),
    NotK,
    CapitalWithoutLower,
    Any,
    Empty,
}

struct Case<'a> {
    source: &'a str,
    literal: u32,
    sets: [Set; 2],
    ignore_case: bool,
    multiline: bool,
    anchored: bool,
}

// Independent finite facts only: ASCII common/simple folds, long s and Kelvin.
// Full/Turkic folds and production decoders/matchers are not oracle inputs.
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
        Set::Point(value) => canonical == if ignore_case { fold(value) } else { value },
        Set::NotK => canonical != if ignore_case { 0x6b } else { 0x4b },
        // v+i complements the folded Ll set, which contains folded A.
        Set::CapitalWithoutLower => !ignore_case && point == 0x41,
        Set::Any => true,
        Set::Empty => false,
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
    // Only the initial inside-pair offset uses the documented leading boundary.
    let first = boundaries.partition_point(|&at| at <= start) - 1;
    let line = |unit| matches!(unit, 0xa | 0xd | 0x2028 | 0x2029);
    for at in first..points.len().saturating_sub(2) {
        if sticky && at != first {
            break;
        }
        let begin = boundaries[at];
        let end = boundaries[at + 3];
        let literal = if case.ignore_case {
            fold(points[at + 1])
        } else {
            points[at + 1]
        };
        let required = if case.ignore_case {
            fold(case.literal)
        } else {
            case.literal
        };
        if literal == required
            && contains(case.sets[0], points[at], case.ignore_case)
            && contains(case.sets[1], points[at + 2], case.ignore_case)
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
) -> Option<RegExpUnicodeClassLiteralClassMatcher> {
    RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
        &JsString::from(source),
        ignore_case,
        multiline,
        |_| Ok::<(), Infallible>(()),
    )
    .unwrap()
}

#[test]
fn checked_literal_checked_matches_independent_tiny_utf16_oracle() {
    let cases = [
        Case {
            source: "[[a]]b[[a]]",
            literal: 0x62,
            sets: [Set::Point(0x61), Set::Point(0x61)],
            ignore_case: false,
            multiline: false,
            anchored: false,
        },
        Case {
            source: "[[K]]K[[s]]",
            literal: 0x4b,
            sets: [Set::Point(0x4b), Set::Point(0x73)],
            ignore_case: true,
            multiline: false,
            anchored: false,
        },
        Case {
            source: r"[\P{Ll}&&A]c[[s]]",
            literal: 0x63,
            sets: [Set::CapitalWithoutLower, Set::Point(0x73)],
            ignore_case: false,
            multiline: false,
            anchored: false,
        },
        Case {
            source: r"[\P{Ll}&&A]c[[s]]",
            literal: 0x63,
            sets: [Set::CapitalWithoutLower, Set::Point(0x73)],
            ignore_case: true,
            multiline: false,
            anchored: false,
        },
        Case {
            source: r"[[\u{1f600}]]\u{1f601}[[\u{1f602}]]",
            literal: 0x1f601,
            sets: [Set::Point(0x1f600), Set::Point(0x1f602)],
            ignore_case: false,
            multiline: false,
            anchored: false,
        },
        Case {
            source: r"[[\uD800]]\uD800[[\uD800]]",
            literal: 0xd800,
            sets: [Set::Point(0xd800), Set::Point(0xd800)],
            ignore_case: false,
            multiline: false,
            anchored: false,
        },
        Case {
            source: r"^[[a]]b[[\p{Any}]]$",
            literal: 0x62,
            sets: [Set::Point(0x61), Set::Any],
            ignore_case: false,
            multiline: true,
            anchored: true,
        },
        Case {
            source: r"^[[a]]b[[\p{Any}]]$",
            literal: 0x62,
            sets: [Set::Point(0x61), Set::Any],
            ignore_case: false,
            multiline: false,
            anchored: true,
        },
        Case {
            source: "[[a]]s[[^K]]",
            literal: 0x73,
            sets: [Set::Point(0x61), Set::NotK],
            ignore_case: true,
            multiline: false,
            anchored: false,
        },
        Case {
            source: "[a--a]a[[b]]",
            literal: 0x61,
            sets: [Set::Empty, Set::Point(0x62)],
            ignore_case: false,
            multiline: false,
            anchored: false,
        },
    ];
    let inputs = [
        JsString::from(""),
        JsString::from("aba"),
        JsString::from("aaba"),
        JsString::from("aac"),
        JsString::from("Kkſ"),
        JsString::from("KKS"),
        JsString::from("Acs"),
        JsString::from("asα"),
        JsString::from("asK"),
        JsString::from("😀😁😂"),
        JsString::from("x😀😁😂!"),
        JsString::from("ab😀"),
        JsString::from("x\r\nab😀\u{2028}x"),
        JsString::from("aba\n"),
        JsString::from_code_units(vec![0xd800, 0xd800, 0xd800]),
        JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
    ];
    let mut queries = 0;
    for case in &cases {
        let matcher = compile(case.source, case.ignore_case, case.multiline).unwrap();
        for input in &inputs {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(input, start, sticky),
                        expected(case, input, start, sticky),
                        "{} input={input:?} start={start} sticky={sticky}",
                        case.source
                    );
                    queries += 1;
                }
            }
        }
    }
    assert_eq!(queries, 1880);
}

#[test]
fn checked_literal_checked_domains_full_proofs_and_work_prefixes() {
    for source in [
        "",
        "a",
        "[[a]]b",
        "[[a]][[b]]",
        "c[[a]][[b]]",
        "[[a]][[b]]c",
        "[a]b[[c]]",
        "[[a]]b[c]",
        "[a]b[c]",
        "[[a]][[b]][[c]]",
        "c[[a]]x[[b]]",
        "[[a]]x[[b]]c",
        "[[a]]bc[[d]]",
        "[[a]]+b[[c]]",
        "[[a]]b?[[c]]",
        "[[a]]b[[c]]+",
        "[[a]]^[[b]]",
        "[[a]]b^[[c]]",
        "([[a]])b[[c]]",
        "[[a]](b)[[c]]",
        "([[a]]b[[c]])",
        r"\b[[a]]b[[c]]",
        r"[[a]]b[[c]]\b",
        r"[[a]]\b[[c]]",
        r"[[a]].[[c]]",
        r"[[a]]\d[[c]]",
        r"[[a]]\p{Any}[[c]]",
        r"[[\q{a}]]b[[c]]",
        r"[a--a]b[[\p{RGI_Emoji}]]",
    ] {
        assert!(compile(source, false, false).is_none(), "{source}");
    }
    for (source, ignore_case) in [
        ("^[[a]]b[[a]]$".to_owned(), false),
        ("[[K]]K[[s]]".to_owned(), true),
        ("[a--a]a[[b]]".to_owned(), false),
        ("[[a]]K[b--b]".to_owned(), true),
        (format!(r"[[a]]\u{{{}61}}[[b]]", "0".repeat(32)), false),
    ] {
        let source = JsString::from(source.as_str());
        let mut complete = Vec::new();
        let matcher = RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
            &source,
            ignore_case,
            false,
            |amount| {
                complete.push(amount);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len().saturating_mul(3));
        let units = source.code_units();
        let begin = usize::from(units.first() == Some(&0x5e));
        let split = class_end(units, begin).unwrap();
        let mut literal_end = split + 1;
        unicode_literal_atom(units, &mut literal_end, units[split]).unwrap();
        let mut components = vec![source.len().saturating_mul(3)];
        RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[..split],
            ignore_case,
            false,
            |n| {
                components.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
            &units[split..literal_end],
            ignore_case,
            false,
            |n| {
                components.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[literal_end..],
            ignore_case,
            false,
            |n| {
                components.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(complete, components);
        assert_eq!(
            matcher.search_passes(),
            matcher.classes[0]
                .search_passes()
                .saturating_add(matcher.literal.search_passes())
                .saturating_add(matcher.classes[1].search_passes())
                .saturating_add(3)
        );
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
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
    // The literal and full second-class proof run even after an empty first set.
    let source = JsString::from(r"[a--a]c[[\p{RGI_Emoji}]]");
    let units = source.code_units();
    let split = class_end(units, 0).unwrap();
    let literal_end = split + 1;
    let mut observed = Vec::new();
    assert!(
        RegExpUnicodeClassLiteralClassMatcher::compile_v_set_with_work(
            &source,
            false,
            false,
            |n| {
                observed.push(n);
                Ok::<(), Infallible>(())
            },
        )
        .unwrap()
        .is_none()
    );
    let mut expected_work = vec![source.len().saturating_mul(3)];
    RegExpUnicodeClassMatcher::compile_v_set_units_with_work(&units[..split], false, false, |n| {
        expected_work.push(n);
        Ok::<(), Infallible>(())
    })
    .unwrap()
    .unwrap();
    RegExpUnicodeCharacterMatcher::compile_literal_units_with_flags_and_work(
        &units[split..literal_end],
        false,
        false,
        |n| {
            expected_work.push(n);
            Ok::<(), Infallible>(())
        },
    )
    .unwrap()
    .unwrap();
    assert!(
        RegExpUnicodeClassMatcher::compile_v_set_units_with_work(
            &units[literal_end..],
            false,
            false,
            |n| {
                expected_work.push(n);
                Ok::<(), Infallible>(())
            },
        )
        .unwrap()
        .is_none()
    );
    assert_eq!(observed, expected_work);
    let matcher = compile("[[a]]b[[a]]", false, false).unwrap();
    let input = JsString::from("aaba");
    let mut charges = Vec::new();
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |n| {
            charges.push(n);
            Ok::<(), usize>(())
        }),
        Ok(Some(1..4))
    );
    assert_eq!(charges, [1]);
    assert_eq!(
        matcher.find_with_work(&input, 0, false, |_| Err(31)),
        Err(31)
    );
    assert_eq!(matcher.find(&input, 0, true), None);
    assert_eq!(matcher.find(&input, 1, true), Some(1..4));
    assert_eq!(
        compile("[[a]]a[[c]]", false, false)
            .unwrap()
            .find(&JsString::from("aaac"), 0, false),
        Some(1..4)
    );
    let astral = compile(r"[[\u{1f600}]]\u{1f601}[[\u{1f602}]]", false, false).unwrap();
    assert_eq!(astral.find(&JsString::from("😀😁😂"), 1, true), Some(0..6));
    assert_eq!(
        compile(r"^[[a]]\$[[b]]$", false, false)
            .unwrap()
            .find(&JsString::from("a$b"), 0, true),
        Some(0..3)
    );
    assert_eq!(
        compile(r"[[\uD800]]\uD800[[\uDC00]]", false, false)
            .unwrap()
            .find(
                &JsString::from_code_units(vec![0xd800, 0xd800, 0xdc00]),
                0,
                false
            ),
        None
    );
}
