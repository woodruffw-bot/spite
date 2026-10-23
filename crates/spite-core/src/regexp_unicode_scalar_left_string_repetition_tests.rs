use super::*;

#[path = "regexp_unicode_character_result_repetition_tests.rs"]
mod character_result;

#[derive(Clone, Copy, Default)]
struct Gates {
    anchors: bool,
    words: [Option<bool>; 2],
    i: bool,
    m: bool,
}

#[derive(Clone, Copy)]
struct Bounds {
    minimum: usize,
    maximum: Option<usize>,
    greedy: bool,
}

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeRepeatedAtomMatcher> {
    RegExpUnicodeRepeatedAtomMatcher::compile_scalar_left_string_operations_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

fn fold(point: u32) -> u32 {
    match point {
        65..=90 => point + 32,
        0x17f => 115,
        0x212a => 107,
        _ => point,
    }
}

// Enumerate legal repetition endpoints over a tiny independent UTF-16 decoder.
fn oracle(
    input: &JsString,
    start: usize,
    sticky: bool,
    contains: fn(u32) -> bool,
    bounds: Bounds,
    gates: Gates,
) -> Option<Range<usize>> {
    let units = input.code_units();
    if start > units.len() {
        return None;
    }
    let mut points = Vec::new();
    let mut offsets = vec![0];
    let mut at = 0;
    while at < units.len() {
        let first = units[at];
        let (point, width) = if (0xd800..=0xdbff).contains(&first)
            && units
                .get(at + 1)
                .is_some_and(|last| (0xdc00..=0xdfff).contains(last))
        {
            (
                0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(units[at + 1]) - 0xdc00,
                2,
            )
        } else {
            (u32::from(first), 1)
        };
        points.push(point);
        at += width;
        offsets.push(at);
    }
    let first = offsets.partition_point(|&offset| offset <= start) - 1;
    let line = |p| matches!(p, 10 | 13 | 0x2028 | 0x2029);
    let word = |p| {
        let p = if gates.i { fold(p) } else { p };
        (48..=57).contains(&p) || (65..=90).contains(&p) || (97..=122).contains(&p) || p == 95
    };
    let boundary = |at: usize| {
        at.checked_sub(1).is_some_and(|i| word(points[i]))
            != points.get(at).is_some_and(|&p| word(p))
    };
    for begin in first..=points.len() {
        if sticky && begin != first {
            break;
        }
        let mut run = 0;
        while points
            .get(begin + run)
            .is_some_and(|&p| contains(if gates.i { fold(p) } else { p }))
        {
            run += 1
        }
        let limit = bounds.maximum.unwrap_or(run).min(run);
        if bounds.minimum > limit {
            continue;
        }
        for offset in 0..=limit - bounds.minimum {
            let count = if bounds.greedy {
                limit - offset
            } else {
                bounds.minimum + offset
            };
            let end = begin + count;
            if gates.anchors && !(begin == 0 || gates.m && line(points[begin - 1])) {
                continue;
            }
            if gates.anchors && !(end == points.len() || gates.m && line(points[end])) {
                continue;
            }
            if gates.words[0].is_some_and(|expected| boundary(begin) != expected)
                || gates.words[1].is_some_and(|expected| boundary(end) != expected)
            {
                continue;
            }
            return Some(offsets[begin]..offsets[end]);
        }
    }
    None
}

#[test]
fn projected_repetition_matches_independent_counts_priorities_assertions_and_utf16_ranges() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate); 5] = [
        (r"[\p{Any}&&\q{a|ab|}]", |p| p == 97),
        (r"[\p{Any}--\q{a|ab|}]", |p| p != 97),
        (r"[[a-z]&&[a-z\q{ab}]]", |p| (97..=122).contains(&p)),
        (r"[\p{Any}&&\q{😀|😀x|}]", |p| p == 0x1f600),
        (r"[\p{Any}&&\q{ab|}]", |_| false),
    ];
    let quantifiers = [
        ("*", 0, None, true),
        ("*?", 0, None, false),
        ("+", 1, None, true),
        ("+?", 1, None, false),
        ("?", 0, Some(1), true),
        ("??", 0, Some(1), false),
        ("{0}", 0, Some(0), true),
        ("{2}", 2, Some(2), true),
        ("{1,3}", 1, Some(3), true),
        ("{1,3}?", 1, Some(3), false),
    ];
    for (class, contains) in classes {
        for (quantifier, minimum, maximum, greedy) in quantifiers {
            for (prefix, suffix, anchors, words) in [
                ("", "", false, [None, None]),
                ("^", "$", true, [None, None]),
                (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                (r"\B", r"\b", false, [Some(false), Some(true)]),
            ] {
                for i in [false, true] {
                    for m in [false, true] {
                        let source = format!("{prefix}{class}{quantifier}{suffix}");
                        let plan = compile(&source, i, m).unwrap();
                        assert_eq!(plan.capture_count(), 0);
                        for input in [
                            JsString::from(""),
                            JsString::from("aaab!aba"),
                            JsString::from("A😀😀aa"),
                            JsString::from("!\r\naaa\u{2028}!"),
                            JsString::from("ſK"),
                            JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let found = plan
                                        .find_with_captures_and_work(&input, start, sticky, |_| {
                                            Ok::<(), ()>(())
                                        })
                                        .unwrap();
                                    assert!(
                                        found
                                            .as_ref()
                                            .is_none_or(|found| found.atom_capture.is_none())
                                    );
                                    assert_eq!(
                                        found.map(|found| found.range),
                                        oracle(
                                            &input,
                                            start,
                                            sticky,
                                            contains,
                                            Bounds {
                                                minimum,
                                                maximum,
                                                greedy
                                            },
                                            Gates {
                                                anchors,
                                                words,
                                                i,
                                                m
                                            }
                                        ),
                                        "{source} i={i} m={m} {input:?} start={start} sticky={sticky}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn projected_repetition_proves_zero_and_huge_bounds_and_keeps_other_shapes_separate() {
    for source in [
        r"[\p{Any}--\q{ab}]",
        r"[\q{ab}--\p{Any}]+",
        r"[[a\q{ab}]&&[a-z]]+",
        r"[\p{Any}\q{ab}]+",
        r"[^\p{Any}--\q{ab}]+",
        r"([\p{Any}--\q{ab}])+",
        r"[\p{Any}--\q{ab}]+[a]",
        r"\b^[\p{Any}--\q{ab}]+",
        r"[[]&&\q{ab}&&\p{RGI_Emoji}]{0}",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    for source in [r"[\p{Any}&&\q{ab|}]*", r"[\p{Any}--\q{ab|}]{0}"] {
        let plan = compile(source, false, false).unwrap();
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            Some(0..0)
        );
    }
    let huge = "9".repeat(100);
    let source = format!(r"[\p{{Any}}--\q{{ab}}]{{{huge}}}");
    let plan = compile(&source, false, false).unwrap();
    assert_eq!(plan.minimum, None);
    assert_eq!(
        plan.find_with_work(&JsString::from("😀ab"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
    let source = format!(r"[\p{{Any}}&&\q{{ab|}}]{{0,{huge}}}");
    let plan = compile(&source, false, false).unwrap();
    assert_eq!(
        plan.find_with_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..0)
    );
    let plan = compile(r"[\p{Any}&&\q{ab|}]+", false, false).unwrap();
    assert_eq!(
        plan.find_with_work(&JsString::from("😀ab"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
}

#[test]
fn projected_repetition_checks_every_constructor_callback_and_retains_flat_long_source_work() {
    for source in [r"^\b[[a-z]&&[a\q{ab}]]{1,3}?\b$", r"[[]&&\q{ab}]{0}"] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        RegExpUnicodeRepeatedAtomMatcher::compile_scalar_left_string_operations_with_work(
            &source,
            true,
            true,
            |work| {
                complete.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len());
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                RegExpUnicodeRepeatedAtomMatcher::compile_scalar_left_string_operations_with_work(
                    &source,
                    true,
                    true,
                    |work| {
                        seen.push(work);
                        if seen.len() == fail {
                            Err(fail)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(
                matches!(result,Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at==fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        RegExpUnicodeRepeatedAtomMatcher::compile_scalar_left_string_operations_with_work(
            &source,
            true,
            true,
            |work| {
                recovered.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    let source = JsString::from(
        format!(
            r"[\p{{Any}}&&{}\q{{a|ab}}{}]+",
            "[".repeat(20_000),
            "]".repeat(20_000)
        )
        .as_str(),
    );
    let mut work = 0usize;
    let original =
        RegExpUnicodeRepeatedAtomMatcher::compile_scalar_left_string_operations_with_work(
            &source,
            false,
            false,
            |n| {
                work += n;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
    assert!(
        work < source.len() * 64 + 1024,
        "checked deep work must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(
        plan.find_with_work(&JsString::from("aaab"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..3)
    );
    let source =
        JsString::from(format!(r"[\p{{Any}}--\q{{{}}}]{{2}}", "x".repeat(20_000)).as_str());
    let mut work = 0usize;
    let plan = RegExpUnicodeRepeatedAtomMatcher::compile_scalar_left_string_operations_with_work(
        &source,
        false,
        false,
        |n| {
            work += n;
            Ok::<(), ()>(())
        },
    )
    .unwrap()
    .unwrap();
    assert!(
        work < source.len() * 24 + 1024,
        "long words must not expand: {work}"
    );
    assert_eq!(
        plan.find_with_work(&JsString::from("😀a"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..3)
    );
}
