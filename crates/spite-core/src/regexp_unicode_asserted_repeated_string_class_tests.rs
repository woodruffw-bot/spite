fn asserted_repeat_test_ends(oracle: &Oracle<'_>, at: usize, words: &[&str]) -> Vec<usize> {
    let mut ends = Vec::new();
    if let Some((point, end)) = oracle.point(at) {
        if (oracle.contains)(if oracle.i {
            word_root_fold(point)
        } else {
            point
        }) {
            ends.push(end);
        }
    }
    for word in words {
        let mut end = at;
        let mut matches = true;
        for ch in word.chars() {
            let Some((point, next)) = oracle.point(end) else {
                matches = false;
                break;
            };
            let (point, expected) = if oracle.i {
                (word_root_fold(point), word_root_fold(u32::from(ch)))
            } else {
                (point, u32::from(ch))
            };
            if point != expected {
                matches = false;
                break;
            }
            end = next;
        }
        if matches && end > at {
            ends.push(end);
        }
    }
    ends.sort_unstable_by(|a, b| b.cmp(a));
    ends.dedup();
    ends
}

fn asserted_repeat_test_predicates(bits: u8, oracle: &Oracle<'_>, at: usize) -> bool {
    let units = oracle.input.code_units();
    (bits & 1 == 0 || at == 0 || (oracle.m && Oracle::line(units[at - 1])))
        && (bits & 2 == 0 || at == units.len() || (oracle.m && Oracle::line(units[at])))
        && (bits & 4 == 0 || oracle.boundary(at))
        && (bits & 8 == 0 || !oracle.boundary(at))
}
#[allow(clippy::too_many_arguments)]
fn asserted_repeat_test_walk(
    oracle: &Oracle<'_>,
    words: &[&str],
    at: usize,
    minimum: usize,
    maximum: Option<usize>,
    greedy: bool,
    suffix: u8,
    depth: usize,
    last: Option<Range<usize>>,
) -> Option<(usize, Option<Range<usize>>)> {
    let stop = depth >= minimum && asserted_repeat_test_predicates(suffix, oracle, at);
    if stop && !greedy {
        return Some((at, last));
    }
    if maximum.is_none_or(|maximum| depth < maximum) {
        for end in asserted_repeat_test_ends(oracle, at, words) {
            if let Some(found) = asserted_repeat_test_walk(
                oracle,
                words,
                end,
                minimum,
                maximum,
                greedy,
                suffix,
                depth + 1,
                Some(at..end),
            ) {
                return Some(found);
            }
        }
    }
    if stop { Some((at, last)) } else { None }
}
#[test]
fn asserted_repeated_string_classes_follow_independent_continuation_and_capture_oracle() {
    let classes: [(&str, &[&str], Predicate); 9] = [
        (r"[\q{aa|aaa}]", &["aa", "aaa"], |_| false),
        (r"[\q{ab}&&\q{cd}]", &[], |_| false),
        (r"[\q{ab|abc|c}]", &["ab", "abc", "c"], |_| false),
        (r"[\q{A|ab}]", &["A", "ab"], |_| false),
        (r"[\p{Any}\q{ab}]", &["ab"], |_| true),
        (r"[a-z\q{ab}]", &["ab"], |p| (97..=122).contains(&p)),
        (r"[\q{😀|😀a|aX}]", &["😀", "😀a", "aX"], |_| false),
        (r"[\q{\n|\na|aX}]", &["\n", "\na", "aX"], |_| false),
        (r"[\q{S|sa|aX}]", &["S", "sa", "aX"], |_| false),
    ];
    for (class, words, contains) in classes {
        for layout in 0..6 {
            for atomic in [false, true] {
                let mut tree = single_string_capture_tree(layout, class);
                let mut slots = 0;
                let body = tree.source(&mut slots);
                for (op, minimum, maximum, greedy) in [
                    ("*", 0, None, true),
                    ("*?", 0, None, false),
                    ("+", 1, None, true),
                    ("+?", 1, None, false),
                    ("?", 0, Some(1), true),
                    ("??", 0, Some(1), false),
                    ("{0}", 0, Some(0), true),
                    ("{0,2}?", 0, Some(2), false),
                    ("{1,3}", 1, Some(3), true),
                    ("{2}", 2, Some(2), true),
                    ("{3,4}?", 3, Some(4), false),
                    ("{2,}", 2, None, true),
                ] {
                    for (prefix, suffix, prefix_bits, suffix_bits) in [
                        ("^", "$", 1, 2),
                        (r"\b", r"\b", 4, 4),
                        (r"\B", r"\B", 8, 8),
                        ("^", r"\b", 1, 4),
                        (r"\b", "$", 4, 2),
                        ("$", "^", 2, 1),
                        (r"\b\B", "", 12, 0),
                        ("", "^$", 0, 3),
                    ] {
                        let source = if atomic {
                            format!(
                                "({prefix})({})({suffix})",
                                body.replacen(class, &format!("{class}{op}"), 1)
                            )
                        } else {
                            format!("({prefix})(({body}){op})({suffix})")
                        };
                        for i in [false, true] {
                            for m in [false, true] {
                                let plan=crate::RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher::compile_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap().unwrap_or_else(||panic!("{source}"));
                                assert_eq!(
                                    plan.capture_count(),
                                    slots + if atomic { 3 } else { 4 }
                                );
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaaaaa"),
                                    JsString::from("!abcab!"),
                                    JsString::from("😀aX😀!"),
                                    JsString::from("ſaXſ!"),
                                    JsString::from("\n\naX\n"),
                                    JsString::from("ab"),
                                    JsString::from_code_units(vec![0xd800, 97, 98, 0xdc00]),
                                ] {
                                    let oracle = Oracle {
                                        input: &input,
                                        contains,
                                        minimum: 1,
                                        maximum: Some(1),
                                        greedy: true,
                                        empty_first: false,
                                        roots: 0,
                                        captures: slots,
                                        empty_slots: &[],
                                        never_slots: &[],
                                        i,
                                        m,
                                        anchors: false,
                                        words: [None, None],
                                    };
                                    for requested in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            let expected = (|| {
                                                if requested > input.len() {
                                                    return None;
                                                }
                                                let units = input.code_units();
                                                let mut start = if requested > 0
                                                    && units.get(requested).is_some_and(|u| {
                                                        (0xdc00..=0xdfff).contains(u)
                                                    })
                                                    && (0xd800..=0xdbff)
                                                        .contains(&units[requested - 1])
                                                {
                                                    requested - 1
                                                } else {
                                                    requested
                                                };
                                                loop {
                                                    if asserted_repeat_test_predicates(
                                                        prefix_bits,
                                                        &oracle,
                                                        start,
                                                    ) {
                                                        if let Some((end, last)) =
                                                            asserted_repeat_test_walk(
                                                                &oracle,
                                                                words,
                                                                start,
                                                                minimum,
                                                                maximum,
                                                                greedy,
                                                                suffix_bits,
                                                                0,
                                                                None,
                                                            )
                                                        {
                                                            let range = start..end;
                                                            let mut values = vec![None; slots];
                                                            if atomic {
                                                                assert_eq!(
                                                                    eval_word_root_tree(
                                                                        &tree,
                                                                        true,
                                                                        start,
                                                                        Some(end),
                                                                        &mut values
                                                                    ),
                                                                    Some(end)
                                                                );
                                                            } else if let Some(last) = last.as_ref()
                                                            {
                                                                assert_eq!(
                                                                    eval_word_root_tree(
                                                                        &tree,
                                                                        true,
                                                                        last.start,
                                                                        Some(last.end),
                                                                        &mut values
                                                                    ),
                                                                    Some(last.end)
                                                                );
                                                            }
                                                            let mut captures = vec![
                                                                Some(start..start),
                                                                Some(range.clone()),
                                                            ];
                                                            if !atomic {
                                                                captures.push(last.clone());
                                                            }
                                                            captures.extend(values);
                                                            captures.push(Some(end..end));
                                                            return Some((
                                                                range,
                                                                last.clone(),
                                                                last.is_some(),
                                                                captures,
                                                            ));
                                                        }
                                                    }
                                                    if sticky {
                                                        return None;
                                                    }
                                                    start = oracle.point(start)?.1;
                                                }
                                            })(
                                            );
                                            let found = plan
                                                .find_with_captures_and_work(
                                                    &input,
                                                    requested,
                                                    sticky,
                                                    |_| Ok::<(), ()>(()),
                                                )
                                                .unwrap();
                                            let actual = found.map(|found| {
                                                let caps = (0..plan.capture_count())
                                                    .map(|n| plan.capture_range(n, &found))
                                                    .collect::<Vec<_>>();
                                                (
                                                    found.range,
                                                    found.last_iteration,
                                                    found.class_selected,
                                                    caps,
                                                )
                                            });
                                            assert_eq!(
                                                actual, expected,
                                                "{source:?}, {input:?}, {requested}, sticky={sticky},i={i},m={m}"
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
    }
}

#[test]
fn asserted_repeated_string_classes_match_native_assertion_continuation_and_capture_tables() {
    for (source, input, i, m, sticky, start, expected, last, selected, captures) in [
        (
            r"(^)((([\q{aa|aaa}]))*)($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(^)((([\q{aa|aaa}]))*?)($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(^)((([\q{aa|aaa}]))+)($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(^)((([\q{aa|aaa}]))+?)($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(^)((([\q{aa|aaa}])){3})($)",
            "aaaaaa",
            false,
            false,
            false,
            0,
            Some(0..6),
            Some(4..6),
            true,
            vec![Some(0..0), Some(0..6), Some(4..6), Some(4..6), Some(6..6)],
        ),
        (
            r"(^)((([\q{aa|aaa}])){3,4}?)($)",
            "aaaaaa",
            false,
            false,
            false,
            0,
            Some(0..6),
            Some(4..6),
            true,
            vec![Some(0..0), Some(0..6), Some(4..6), Some(4..6), Some(6..6)],
        ),
        (
            r"(^)((([\q{aa|aaa}])){0})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            None,
            false,
            vec![Some(0..0), Some(0..0), None, None, Some(0..0)],
        ),
        (
            r"(^)((([\q{aa|aaa}])){0})($)",
            "a",
            false,
            false,
            false,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"(^)((([\q{aa|aaa}])){0,2}?)($)",
            "aaa",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(0..3),
            true,
            vec![Some(0..0), Some(0..3), Some(0..3), Some(0..3), Some(3..3)],
        ),
        (
            r"(^)((([\q{aa|aaa}]))?)($)",
            "aaa",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(0..3),
            true,
            vec![Some(0..0), Some(0..3), Some(0..3), Some(0..3), Some(3..3)],
        ),
        (
            r"(^)((([\q{aa|aaa}]))??)($)",
            "aaa",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(0..3),
            true,
            vec![Some(0..0), Some(0..3), Some(0..3), Some(0..3), Some(3..3)],
        ),
        (
            r"(^)((([\q{aa|aaa}])){2,})($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(\b)((([\q{a|aa}]))*)(\b)",
            "aaaa!",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(\b)((([\q{a|aa}]))*?)(\b)",
            "aaaa!",
            false,
            false,
            false,
            0,
            Some(0..0),
            None,
            false,
            vec![Some(0..0), Some(0..0), None, None, Some(0..0)],
        ),
        (
            r"(^)((([\q{ab|abc|c}])){2})($)",
            "abc",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![Some(0..0), Some(0..3), Some(2..3), Some(2..3), Some(3..3)],
        ),
        (
            r"(^)((([\p{Any}\q{ab}])){2})($)",
            "ab",
            false,
            false,
            false,
            0,
            Some(0..2),
            Some(1..2),
            true,
            vec![Some(0..0), Some(0..2), Some(1..2), Some(1..2), Some(2..2)],
        ),
        (
            r"(^)((([a-z\q{ab}])){2})($)",
            "ab",
            false,
            false,
            false,
            0,
            Some(0..2),
            Some(1..2),
            true,
            vec![Some(0..0), Some(0..2), Some(1..2), Some(1..2), Some(2..2)],
        ),
        (
            r"(^)((([\q{😀|😀a|aX}])){2})($)",
            "😀aX",
            false,
            false,
            true,
            1,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(^)((([\q{S|sa|aX}])){2})($)",
            "ſaX",
            true,
            false,
            false,
            0,
            Some(0..3),
            Some(1..3),
            true,
            vec![Some(0..0), Some(0..3), Some(1..3), Some(1..3), Some(3..3)],
        ),
        (
            r"(^)((([\q{a|aa}]))+?)($)",
            "\naaaa\n",
            false,
            true,
            false,
            0,
            Some(1..5),
            Some(3..5),
            true,
            vec![Some(1..1), Some(1..5), Some(3..5), Some(3..5), Some(5..5)],
        ),
        (
            r"($)((([\q{\n|\na|aX}]))*)(^)",
            "\n\naX\n",
            false,
            true,
            false,
            0,
            Some(0..5),
            Some(4..5),
            true,
            vec![Some(0..0), Some(0..5), Some(4..5), Some(4..5), Some(5..5)],
        ),
        (
            r"(\b)((([\q{a|aa}]))*)(\b)",
            "!aaaa!",
            false,
            false,
            false,
            0,
            Some(1..5),
            Some(3..5),
            true,
            vec![Some(1..1), Some(1..5), Some(3..5), Some(3..5), Some(5..5)],
        ),
        (
            r"(\b)((([\q{a|aa}]))*)(\b)",
            "!aaaa!",
            false,
            false,
            true,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"(^)((([\q{a|aa}]))*?)(\b)",
            "x",
            false,
            false,
            false,
            0,
            Some(0..0),
            None,
            false,
            vec![Some(0..0), Some(0..0), None, None, Some(0..0)],
        ),
        (
            r"(\B)((([\q{a|aa}]))*)(\B)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            None,
            false,
            vec![Some(0..0), Some(0..0), None, None, Some(0..0)],
        ),
        (
            r"(\b\B)((([\q{a|aa}]))*)()",
            "a",
            false,
            false,
            false,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"(^)(([\q{ab|abc|c}]*?))($)",
            "abcab",
            false,
            false,
            false,
            0,
            Some(0..5),
            Some(3..5),
            true,
            vec![Some(0..0), Some(0..5), Some(0..5), Some(5..5)],
        ),
        (
            r"(^)((([\q{a|aa}])){2,999999999999999999999999})($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"(^)((([\q{aa|aaa}])){999999999999999999999999})($)",
            "aaaa",
            false,
            false,
            false,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"(^)((([\q{ab}&&\q{cd}]))*)($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            None,
            false,
            vec![Some(0..0), Some(0..0), None, None, Some(0..0)],
        ),
        (
            r"(^)((([\q{ab}&&\q{cd}]))+)($)",
            "",
            false,
            false,
            false,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"(^)((([\q{ab}&&\q{cd}])){0})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            None,
            false,
            vec![Some(0..0), Some(0..0), None, None, Some(0..0)],
        ),
    ] {
        let plan =
            crate::RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher::compile_with_work(
                &JsString::from(source),
                i,
                m,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
        assert_eq!(plan.capture_count(), captures.len());
        let found =
            plan.find_with_captures_and_work(&JsString::from(input), start, sticky, |_| {
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(
            found.as_ref().map(|f| f.range.clone()),
            expected,
            "{source}, {input}"
        );
        if let Some(found) = found {
            assert_eq!(found.last_iteration, last, "{source}");
            assert_eq!(found.class_selected, selected, "{source}");
            assert_eq!(
                (0..plan.capture_count())
                    .map(|n| plan.capture_range(n, &found))
                    .collect::<Vec<_>>(),
                captures,
                "{source}"
            );
        }
    }
}

#[test]
fn asserted_repeated_string_class_deep_capture_and_continuation_storage_stays_flat() {
    use crate::RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher as Plan;
    let source = JsString::from(
        format!(
            r"^{}([\q{{aa|aaa}}]){{3,4}}{}$",
            "(".repeat(20_000),
            "){1}".repeat(20_000)
        )
        .as_str(),
    );
    let original = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let plan = original.clone();
    drop(original);
    let found = plan
        .find_with_captures_and_work(&JsString::from("aaaaaa"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_count(), 20_001);
    assert_eq!(found.range, 0..6);
    assert_eq!(found.last_iteration, Some(4..6));
    for n in 0..20_000 {
        assert_eq!(plan.capture_range(n, &found), Some(0..6));
    }
    assert_eq!(plan.capture_range(20_000, &found), Some(4..6));
    let source = JsString::from(r"^()([\q{a|aa}])*()$");
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(
            &JsString::from("a".repeat(40_000).as_str()),
            0,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..40000);
    assert_eq!(found.last_iteration, Some(39998..40000));
    let word = "x".repeat(20_000);
    let source = JsString::from(format!(r"^()([\q{{{word}|{word}x}}]){{3}}()$").as_str());
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(
            &JsString::from("x".repeat(60_000).as_str()),
            0,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..60000);
    assert_eq!(found.last_iteration, Some(40000..60000));
    let source = JsString::from(r"^()([\q{a|aa}])*?()\b");
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let mut work = Vec::new();
    let found = plan
        .find_with_captures_and_work(&JsString::from("a".repeat(40_000).as_str()), 0, true, |n| {
            work.push(n);
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..0);
    assert_eq!(found.last_iteration, None);
    assert!(!found.class_selected);
    assert!(work.iter().sum::<usize>() < 1000);
    let huge = "9".repeat(20_000);
    let source = JsString::from(format!(r"^([\q{{a|aa}}]){{{huge}}}$").as_str());
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let mut work = Vec::new();
    assert!(
        plan.find_with_captures_and_work(&JsString::from("aaaa"), 0, false, |n| {
            work.push(n);
            Ok::<(), ()>(())
        })
        .unwrap()
        .is_none()
    );
    assert_eq!(work, vec![4]);
}
#[test]
fn asserted_repeated_string_class_cancellations_and_complete_leaf_proofs_remain_separate() {
    use crate::{
        RegExpUnicodeAssertedRepeatedStringClassCaptureMatcher as Plan,
        RegExpUnicodeMatchError as FindFailure, RegExpUnicodeSetCompileError as CompileFailure,
    };
    for source in [
        r"^()([\q{aa|aaa}])*()$",
        r"(\b)(([\p{Any}\q{ab}]){2,3}?)(\b)",
        r"($)(([\q{\n|\na|aX}]){3,})(^)",
        r"(^){1}((([\q{A|ab}]){1}){0,2}?)(\b){1}",
    ] {
        let source = JsString::from(source);
        let mut work = Vec::new();
        let plan = Plan::compile_with_work(&source, true, true, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(Plan::compile_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(CompileFailure::Work(n))if n==fail)
            );
            assert_eq!(seen, work[..fail]);
        }
        let mut recovered = Vec::new();
        assert!(
            Plan::compile_with_work(&source, true, true, |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(recovered, work);
        for input in ["aaaaaa", "!abcab!", "\n\naX\n", "ab", ""] {
            for sticky in [false, true] {
                let input = JsString::from(input);
                let mut work = Vec::new();
                let expected = plan
                    .find_with_captures_and_work(&input, 0, sticky, |n| {
                        work.push(n);
                        Ok::<(), usize>(())
                    })
                    .unwrap();
                for fail in 1..=work.len() {
                    let mut seen = Vec::new();
                    assert!(
                        matches!(plan.find_with_captures_and_work(&input,0,sticky,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(FindFailure::Work(n))if n==fail)
                    );
                    assert_eq!(seen, work[..fail]);
                }
                assert_eq!(
                    plan.find_with_captures_and_work(&input, 0, sticky, |_| Ok::<(), usize>(()))
                        .unwrap(),
                    expected
                );
            }
        }
    }
    for source in [
        r"([\q{aa|aaa}])*",
        r"^([\q{|aa|aaa}])*$",
        r"^(([\q{aa|aaa}])|())*$",
        r"(^([\q{aa|aaa}]))*$",
        r"(^|)([\q{aa|aaa}])*$",
        r"(^)*([\q{aa|aaa}])*$",
        r"^([\q{aa|aaa}])*(\b|)",
        r"^([\q{aa|aaa}])*\1",
        r"a^([\q{aa|aaa}])*$",
        r"^([\q{aa|aaa}])*a$",
        r"^(([\q{aa|aaa}])*){2}$",
        r"^([\q{aa|aaa}])*[\q{ab}]$",
        r"^([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}$",
    ] {
        assert!(
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
