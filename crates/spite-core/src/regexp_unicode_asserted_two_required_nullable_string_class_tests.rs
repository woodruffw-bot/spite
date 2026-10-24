struct AssertedTwoRequiredNullableOracle<'a> {
    oracle: &'a Oracle<'a>,
    words: &'a [&'a str],
    minimum: usize,
    maximum: Option<usize>,
    greedy: bool,
    mode: usize,
    suffix: u8,
}
impl AssertedTwoRequiredNullableOracle<'_> {
    fn walk(
        &self,
        at: usize,
        depth: usize,
        last: Option<Range<usize>>,
        selected: bool,
    ) -> Option<(usize, Option<Range<usize>>, bool)> {
        let stop =
            depth >= self.minimum && asserted_repeat_test_predicates(self.suffix, self.oracle, at);
        if stop && !self.greedy {
            return Some((at, last, selected));
        }
        if self.maximum.is_none_or(|maximum| depth < maximum) {
            let mut choices = asserted_repeat_test_ends(self.oracle, at, self.words)
                .into_iter()
                .map(|end| (end, true))
                .collect::<Vec<_>>();
            if depth < self.minimum {
                if self.words.contains(&"") {
                    choices.push((at, true));
                }
                if self.mode == 2 {
                    choices.insert(0, (at, false));
                } else if self.mode == 1 {
                    choices.push((at, false));
                }
            }
            for (end, selected) in choices {
                if let Some(found) = self.walk(end, depth + 1, Some(at..end), selected) {
                    return Some(found);
                }
            }
        }
        if stop {
            Some((at, last, selected))
        } else {
            None
        }
    }
}
#[test]
fn asserted_two_required_nullable_string_classes_follow_independent_required_empty_continuations_and_capture_oracle()
 {
    let classes: [(&str, &[&str], Predicate); 10] = [
        (r"[\q{|a|ab}]", &["", "a", "ab"], |_| false),
        (r"[\q{|aa|aaa}]", &["", "aa", "aaa"], |_| false),
        (r"[\q{|ab|abc|c}]", &["", "ab", "abc", "c"], |_| false),
        (r"[\q{ab|abc|c}]", &["ab", "abc", "c"], |_| false),
        (r"[\p{Any}\q{|ab}]", &["", "ab"], |_| true),
        (r"[a-z\q{|ab}]", &["", "ab"], |p| (97..=122).contains(&p)),
        (r"[\q{|😀|😀a|aX}]", &["", "😀", "😀a", "aX"], |_| false),
        (r"[\q{|S|sa|aX}]", &["", "S", "sa", "aX"], |_| false),
        (r"[\q{ab}&&\q{cd}]", &[], |_| false),
        (r"[\q{}]", &[""], |_| false),
    ];
    for (class, words, contains) in classes {
        for layout in 0..6 {
            for mode in 0..3 {
                for atomic in [false, true] {
                    if atomic && mode != 0 {
                        continue;
                    }
                    let mut tree = if mode == 0 {
                        single_string_capture_tree(layout, class)
                    } else {
                        root_nullable_tree(layout, class, mode == 2)
                    };
                    let mut slots = 0;
                    let body = tree.source(&mut slots);
                    for (op, minimum, maximum, greedy) in [
                        ("{2}", 2, Some(2), true),
                        ("{2}?", 2, Some(2), false),
                        ("{2,3}", 2, Some(3), true),
                        ("{2,3}?", 2, Some(3), false),
                        ("{2,4}", 2, Some(4), true),
                        ("{2,4}?", 2, Some(4), false),
                        ("{0002,0003}", 2, Some(3), true),
                        ("{0002,0003}?", 2, Some(3), false),
                        ("{2,}", 2, None, true),
                        ("{2,}?", 2, None, false),
                        ("{2,999999999999999999999999999999}", 2, None, true),
                        ("{2,999999999999999999999999999999}?", 2, None, false),
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
                                    let plan=crate::RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher::compile_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap();
                                    if mode == 0 && !words.contains(&"") {
                                        assert!(plan.is_none(), "{source}");
                                        continue;
                                    }
                                    let plan = plan.unwrap_or_else(|| panic!("{source}"));
                                    assert_eq!(
                                        plan.capture_count(),
                                        slots + if atomic { 3 } else { 4 }
                                    );
                                    for input in [
                                        JsString::from(""),
                                        JsString::from("aaaa"),
                                        JsString::from("!abcab!"),
                                        JsString::from("😀aX😀!"),
                                        JsString::from("ſaXſ!"),
                                        JsString::from("\n\nab\n"),
                                        JsString::from("a"),
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
                                        let model = AssertedTwoRequiredNullableOracle {
                                            oracle: &oracle,
                                            words,
                                            minimum,
                                            maximum,
                                            greedy,
                                            mode,
                                            suffix: suffix_bits,
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
                                                            if let Some((end, last, selected)) =
                                                                model.walk(start, 0, None, false)
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
                                                                } else if let Some(last) =
                                                                    last.as_ref()
                                                                {
                                                                    assert_eq!(
                                                                        eval_word_root_tree(
                                                                            &tree,
                                                                            selected,
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
                                                                    selected,
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
}

#[test]
fn asserted_two_required_nullable_string_classes_match_native_required_empty_and_branch_priority_tables()
 {
    for (source, input, i, m, sticky, start, expected, last, selected, captures) in [
        (
            r"(^)((([\q{|a|ab}])){2})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            true,
            vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
        ),
        (
            r"(^)((([\q{|a|ab}])){2}?)($)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..1),
            Some(1..1),
            true,
            vec![Some(0..0), Some(0..1), Some(1..1), Some(1..1), Some(1..1)],
        ),
        (
            r"(^)((([\q{|a|ab}])){2})($)",
            "ab",
            false,
            false,
            false,
            0,
            Some(0..2),
            Some(2..2),
            true,
            vec![Some(0..0), Some(0..2), Some(2..2), Some(2..2), Some(2..2)],
        ),
        (
            r"(^)((([\q{|a|ab}])){2,3})($)",
            "aba",
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
            r"(^)((([\q{|a|ab}])){2,3}?)($)",
            "aba",
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
            r"(^)((([\q{|a|ab}])){2,})($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(3..4),
            true,
            vec![Some(0..0), Some(0..4), Some(3..4), Some(3..4), Some(4..4)],
        ),
        (
            r"(^)((([\q{|a|ab}])){2,999999999999999999999999})($)",
            "aba",
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
            r"(^)((([\q{|a|ab}])|()){2})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                Some(0..0),
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..0),
            ],
        ),
        (
            r"(^)((([\q{|a|ab}])|()){2}?)($)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..1),
            Some(1..1),
            true,
            vec![
                Some(0..0),
                Some(0..1),
                Some(1..1),
                Some(1..1),
                None,
                Some(1..1),
            ],
        ),
        (
            r"(^)((([\q{|a|ab}])|()){2})($)",
            "ab",
            false,
            false,
            false,
            0,
            Some(0..2),
            Some(2..2),
            true,
            vec![
                Some(0..0),
                Some(0..2),
                Some(2..2),
                Some(2..2),
                None,
                Some(2..2),
            ],
        ),
        (
            r"(^)((([\q{|a|ab}])|()){2,3})($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                Some(2..3),
                None,
                Some(3..3),
            ],
        ),
        (
            r"(^)((([\q{|a|ab}])|()){2,3}?)($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                Some(2..3),
                None,
                Some(3..3),
            ],
        ),
        (
            r"(^)((([\q{|a|ab}])|()){2,})($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(3..4),
            true,
            vec![
                Some(0..0),
                Some(0..4),
                Some(3..4),
                Some(3..4),
                None,
                Some(4..4),
            ],
        ),
        (
            r"(^)((([\q{|a|ab}])|()){2,999999999999999999999999})($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                Some(2..3),
                None,
                Some(3..3),
            ],
        ),
        (
            r"(^)((()|([\q{|a|ab}])){2})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            false,
            vec![
                Some(0..0),
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..0),
            ],
        ),
        (
            r"(^)((()|([\q{|a|ab}])){2}?)($)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                Some(0..0),
                Some(0..1),
                Some(0..1),
                None,
                Some(0..1),
                Some(1..1),
            ],
        ),
        (
            r"(^)((()|([\q{|a|ab}])){2})($)",
            "ab",
            false,
            false,
            false,
            0,
            Some(0..2),
            Some(0..2),
            true,
            vec![
                Some(0..0),
                Some(0..2),
                Some(0..2),
                None,
                Some(0..2),
                Some(2..2),
            ],
        ),
        (
            r"(^)((()|([\q{|a|ab}])){2,3})($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                None,
                Some(2..3),
                Some(3..3),
            ],
        ),
        (
            r"(^)((()|([\q{|a|ab}])){2,3}?)($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                None,
                Some(2..3),
                Some(3..3),
            ],
        ),
        (
            r"(^)((()|([\q{|a|ab}])){2,})($)",
            "aaaa",
            false,
            false,
            false,
            0,
            Some(0..4),
            Some(3..4),
            true,
            vec![
                Some(0..0),
                Some(0..4),
                Some(3..4),
                None,
                Some(3..4),
                Some(4..4),
            ],
        ),
        (
            r"(^)((()|([\q{|a|ab}])){2,999999999999999999999999})($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                None,
                Some(2..3),
                Some(3..3),
            ],
        ),
        (
            r"(^)((([\q{a|ab}])|()){2})($)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..1),
            Some(1..1),
            false,
            vec![
                Some(0..0),
                Some(0..1),
                Some(1..1),
                None,
                Some(1..1),
                Some(1..1),
            ],
        ),
        (
            r"(^)((([\q{a|ab}])|()){2})($)",
            "ab",
            false,
            false,
            false,
            0,
            Some(0..2),
            Some(2..2),
            false,
            vec![
                Some(0..0),
                Some(0..2),
                Some(2..2),
                None,
                Some(2..2),
                Some(2..2),
            ],
        ),
        (
            r"(^)((([\q{a|ab}])|()){2,3})($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                Some(2..3),
                None,
                Some(3..3),
            ],
        ),
        (
            r"(^)((()|([\q{a|ab}])){2})($)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                Some(0..0),
                Some(0..1),
                Some(0..1),
                None,
                Some(0..1),
                Some(1..1),
            ],
        ),
        (
            r"(^)((()|([\q{a|ab}])){2})($)",
            "ab",
            false,
            false,
            false,
            0,
            Some(0..2),
            Some(0..2),
            true,
            vec![
                Some(0..0),
                Some(0..2),
                Some(0..2),
                None,
                Some(0..2),
                Some(2..2),
            ],
        ),
        (
            r"(^)((()|([\q{a|ab}])){2,3})($)",
            "aba",
            false,
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(2..3),
                None,
                Some(2..3),
                Some(3..3),
            ],
        ),
        (
            r"(^)((()|([\q{|😀|😀a|aX}])){2})($)",
            "😀aX",
            false,
            false,
            true,
            1,
            Some(0..4),
            Some(2..4),
            true,
            vec![
                Some(0..0),
                Some(0..4),
                Some(2..4),
                None,
                Some(2..4),
                Some(4..4),
            ],
        ),
        (
            r"(^)((()|([\q{|S|sa|aX}])){2})($)",
            "ſaX",
            true,
            false,
            false,
            0,
            Some(0..3),
            Some(1..3),
            true,
            vec![
                Some(0..0),
                Some(0..3),
                Some(1..3),
                None,
                Some(1..3),
                Some(3..3),
            ],
        ),
        (
            r"(^)((([\p{Any}\q{|ab}])|()){2})($)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..1),
            Some(1..1),
            true,
            vec![
                Some(0..0),
                Some(0..1),
                Some(1..1),
                Some(1..1),
                None,
                Some(1..1),
            ],
        ),
        (
            r"(^)((()|([\p{Any}\q{|ab}])){2})($)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                Some(0..0),
                Some(0..1),
                Some(0..1),
                None,
                Some(0..1),
                Some(1..1),
            ],
        ),
        (
            r"(^)((([\q{ab}&&\q{cd}])|()){2})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            false,
            vec![
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..0),
                Some(0..0),
            ],
        ),
        (
            r"(^)((([\q{}])){2})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            true,
            vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
        ),
        (
            r"(\b)((([\q{}])){2}?)(\b)",
            "a",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            true,
            vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
        ),
        (
            r"(^)((()|([\q{}])){2})($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            false,
            vec![
                Some(0..0),
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                Some(0..0),
            ],
        ),
        (
            r"(^)(([\q{|a|ab}]{2}))($)",
            "",
            false,
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            true,
            vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
        ),
    ] {
        let plan=crate::RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher::compile_with_work(&JsString::from(source),i,m,|_|Ok::<(),()>(())).unwrap().unwrap_or_else(||panic!("{source}"));
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
fn asserted_two_required_nullable_string_class_deep_scopes_empty_identity_and_continuations_remain_flat()
 {
    use crate::RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher as Plan;
    let source = JsString::from(
        format!(
            r"^{}(()|([\q{{a|ab}}])){{2}}{}$",
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
        .find_with_captures_and_work(&JsString::from(""), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_count(), 20_003);
    assert_eq!(found.range, 0..0);
    assert_eq!(found.last_iteration, Some(0..0));
    assert!(!found.class_selected);
    for n in 0..20_002 {
        assert_eq!(plan.capture_range(n, &found), Some(0..0));
    }
    assert_eq!(plan.capture_range(20_002, &found), None);
    let source = JsString::from(r"^()(()|([\q{a|aa}])){2,}()$");
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(
            &JsString::from("a".repeat(40_000).as_str()),
            0,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..40000);
    assert_eq!(found.last_iteration, Some(39998..40000));
    assert!(found.class_selected);
    assert_eq!(plan.capture_range(2, &found), None);
    assert_eq!(plan.capture_range(3, &found), Some(39998..40000));
    for (source, selected) in [
        (r"^()([\q{}]){2}?()\b", true),
        (r"^()(()|([\q{}])){2}?()\b", false),
    ] {
        let plan =
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
        let mut work = Vec::new();
        let found = plan
            .find_with_captures_and_work(
                &JsString::from("a".repeat(40_000).as_str()),
                0,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.last_iteration, Some(0..0));
        assert_eq!(found.class_selected, selected);
        assert!(work.iter().sum::<usize>() < 1000);
    }
    let upper = "9".repeat(20_000);
    let source = JsString::from(format!(r"^()(()|([\q{{a|ab}}])){{2,{upper}}}?()\b").as_str());
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
    assert_eq!(found.last_iteration, Some(0..0));
    assert!(!found.class_selected);
    assert_eq!(plan.capture_range(2, &found), Some(0..0));
    assert_eq!(plan.capture_range(3, &found), None);
    assert!(work.iter().sum::<usize>() < 1000);
}
#[test]
fn asserted_two_required_nullable_string_class_cancellation_recovery_and_complete_leaf_proofs_preserve_domains()
 {
    use crate::{
        RegExpUnicodeAssertedTwoRequiredNullableStringClassCaptureMatcher as Plan,
        RegExpUnicodeMatchError as FindFailure, RegExpUnicodeSetCompileError as CompileFailure,
    };
    for source in [
        r"^()([\q{|a|ab}]){2,3}()$",
        r"^()(()|([\q{a|ab}])){2}()$",
        r"(\b)((([\q{|a|ab}])|()){2,3}?)(\b)",
        r"(^){1}((()|([\q{|😀|😀a|aX}])){2,3})(\b){1}",
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
        for input in ["aba", "!ab!", "😀aX", "x", ""] {
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
        r"^([\q{|a|ab}]){3}$",
        r"^(()|([\q{a|ab}])){3}$",
        r"^([\q{a|ab}]){2}$",
        r"^([\q{|a|ab}])*$",
        r"([\q{|a|ab}]){2}",
        r"(^([\q{|a|ab}])){2}$",
        r"(^|)([\q{|a|ab}]){2}$",
        r"(^)*([\q{|a|ab}]){2}$",
        r"^([\q{|a|ab}]){2}(\b|)",
        r"^([\q{|a|ab}]){2}\1",
        r"^([\q{|a|ab}]){2}a$",
        r"^(([\q{|a|ab}]){2}){2}$",
        r"^([\q{|a|ab}]){2}[\q{ab}]$",
        r"^(()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){2}$",
    ] {
        assert!(
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
