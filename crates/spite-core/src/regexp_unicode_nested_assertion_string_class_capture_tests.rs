#[test]
fn nested_assertion_string_class_captures_follow_independent_layer_conjunction_and_capture_oracle()
{
    let classes: [(&str, &[&str], Predicate); 27] = [
        (r"[\q{a|ab|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[\q{ab}\q{a|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[x\q{a|ab}]", &["a", "ab"], |p| p == 120),
        (r"[a-z\q{ab|abc}]", &["ab", "abc"], |p| {
            (97..=122).contains(&p)
        }),
        (r"[[\q{a|ab}][x\q{abc}]]", &["a", "ab", "abc"], |p| p == 120),
        (r"[\q{a|ab|abc}--\q{ab}]", &["a", "abc"], |_| false),
        (r"[[\q{a|ab}x]&&[\q{ab|x}]]", &["ab", "x"], |_| false),
        (r"[\q{a|b|ab}--[b-z]]", &["a", "ab"], |_| false),
        (r"[\q{😀|😀a|a}]", &["😀", "😀a", "a"], |_| false),
        (r"[\q{\u{10428}|\u{10428}a}]", &["𐐨", "𐐨a"], |_| false),
        (r"[\q{s|sa|k|ka}]", &["s", "sa", "k", "ka"], |_| false),
        (r"[\q{ab}--\q{ab}]", &[], |_| false),
        (r"[\q{|a|ab}--\q{}]", &["a", "ab"], |_| false),
        (r"[\q{a|ab}&&\q{ab}]", &["ab"], |_| false),
        (r"[\q{|a|ab}]", &["", "a", "ab"], |_| false),
        (r"[\q{ab}\q{}]", &["", "ab"], |_| false),
        (r"[x\q{|ab}]", &["", "ab"], |p| p == 120),
        (r"[a-z\q{|ab}]", &["", "ab"], |p| (97..=122).contains(&p)),
        (r"[[\q{|ab}][x\q{a}]]", &["", "ab", "a"], |p| p == 120),
        (r"[\q{|a|ab}--\q{a}]", &["", "ab"], |_| false),
        (r"[[\q{|a|ab}x]&&[\q{|ab|x}]]", &["", "ab", "x"], |_| false),
        (r"[\q{|a|ab}--[a-z]]", &["", "ab"], |_| false),
        (r"[\q{|😀|😀a}]", &["", "😀", "😀a"], |_| false),
        (r"[\q{|\u{10428}|\u{10428}a}]", &["", "𐐨", "𐐨a"], |_| false),
        (r"[\q{|s|sa|k|ka}]", &["", "s", "sa", "k", "ka"], |_| false),
        (r"[\q{}]", &[""], |_| false),
        (r"[\q{|ab}--\q{ab}]", &[""], |_| false),
    ];
    for (class, words, contains) in classes {
        for (
            outer_prefix,
            outer_suffix,
            outer_start,
            outer_end,
            outer_start_word,
            outer_end_word,
        ) in [
            (r"\b", "", false, false, Some(true), None),
            (r"\B", "", false, false, Some(false), None),
            ("", r"\b", false, false, None, Some(true)),
            ("", r"\B", false, false, None, Some(false)),
            (r"\b", r"\b", false, false, Some(true), Some(true)),
            (r"\B", r"\B", false, false, Some(false), Some(false)),
            (r"^\b", "", true, false, Some(true), None),
            (r"^\B", r"\b$", true, true, Some(false), Some(true)),
            (r"^\b", r"\B$", true, true, Some(true), Some(false)),
            (r"^\B", r"\B$", true, true, Some(false), Some(false)),
        ] {
            for (prefix, suffix, start_anchor, end_anchor, start_word, end_word) in [
                ("^", "", true, false, None, None),
                ("", "$", false, true, None, None),
                ("^", "$", true, true, None, None),
                (r"\b", "", false, false, Some(true), None),
                ("", r"\b", false, false, None, Some(true)),
                ("", r"\B", false, false, None, Some(false)),
                (r"\b", r"\B", false, false, Some(true), Some(false)),
                (r"\B", r"\b", false, false, Some(false), Some(true)),
                (r"^\b", r"\b$", true, true, Some(true), Some(true)),
                (r"^\B", r"\B$", true, true, Some(false), Some(false)),
            ] {
                for (
                    middle_prefix,
                    middle_suffix,
                    middle_start,
                    middle_end,
                    middle_start_word,
                    middle_end_word,
                ) in [
                    ("^", "$", true, true, None, None),
                    (r"\b", r"\b", false, false, Some(true), Some(true)),
                    (r"\B", r"\B", false, false, Some(false), Some(false)),
                ] {
                    let head_words = [outer_start_word, start_word, middle_start_word];
                    let tail_words = [outer_end_word, end_word, middle_end_word];
                    let conflict = (head_words.contains(&Some(true))
                        && head_words.contains(&Some(false)))
                        || (tail_words.contains(&Some(true)) && tail_words.contains(&Some(false)));
                    for frame in [
                        &[true][..],
                        &[false, true][..],
                        &[true, false, true, true][..],
                    ] {
                        for layout in 0..6 {
                            let mut tree = single_string_capture_tree(layout, class);
                            for &capture in frame.iter().rev() {
                                tree = if capture {
                                    BoundaryTree::capture(tree)
                                } else {
                                    BoundaryTree::First(vec![tree])
                                };
                            }
                            tree = BoundaryTree::capture(tree);
                            let mut count = 0;
                            let wrapped = tree.source(&mut count);
                            let begin = 1 + frame
                                .iter()
                                .map(|&capture| if capture { 1 } else { 3 })
                                .sum::<usize>();
                            let end = wrapped.len() - frame.len() - 1;
                            let source = format!(
                                "{outer_prefix}{}{middle_prefix}{}{prefix}{}{suffix}{}{middle_suffix}{}{outer_suffix}",
                                &wrapped[..1],
                                &wrapped[1..begin],
                                &wrapped[begin..end],
                                &wrapped[end..wrapped.len() - 1],
                                &wrapped[wrapped.len() - 1..]
                            );
                            for i in [false, true] {
                                for m in [false, true] {
                                    let plan =
                        super::RegExpUnicodeSingleStringClassCaptureMatcher::compile_nested_assertion_wrappers_with_work(
                            &JsString::from(source.as_str()),
                            i,
                            m,
                            |_| Ok::<(), ()>(()),
                        )
                        .unwrap()
                        .unwrap();
                                    assert_eq!(plan.capture_count(), count);
                                    for input in [
                                        JsString::from(""),
                                        JsString::from("!abcabx!"),
                                        JsString::from("!ABCa!"),
                                        JsString::from("!😀a😀!"),
                                        JsString::from("!ſaKa"),
                                        JsString::from("!𐐀a𐐨!"),
                                        JsString::from("!\r\nabc"),
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
                                            captures: count,
                                            empty_slots: &[],
                                            never_slots: &[],
                                            i,
                                            m,
                                            anchors: end_anchor || outer_end || middle_end,
                                            words: [
                                                outer_start_word
                                                    .or(middle_start_word)
                                                    .or(start_word),
                                                outer_end_word.or(middle_end_word).or(end_word),
                                            ],
                                        };
                                        for requested in 0..=input.len() + 1 {
                                            for sticky in [false, true] {
                                                let expected = if requested > input.len()
                                                    || conflict
                                                {
                                                    None
                                                } else {
                                                    let units = input.code_units();
                                                    let mut at = if requested > 0
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
                                                        let at_line = !(start_anchor
                                                            || outer_start
                                                            || middle_start)
                                                            || at == 0
                                                            || (m
                                                                && Oracle::line(
                                                                    input.code_units()[at - 1],
                                                                ));
                                                        let at_word = outer_start_word
                                                            .or(middle_start_word)
                                                            .or(start_word)
                                                            .is_none_or(|word| {
                                                                oracle.boundary(at) == word
                                                            });
                                                        if let Some(end) = (at_line && at_word)
                                                            .then(|| {
                                                                word_root_end(&oracle, at, words)
                                                            })
                                                            .flatten()
                                                        {
                                                            let mut captures = vec![None; count];
                                                            assert_eq!(
                                                                eval_word_root_tree(
                                                                    &tree,
                                                                    true,
                                                                    at,
                                                                    Some(end),
                                                                    &mut captures
                                                                ),
                                                                Some(end)
                                                            );
                                                            break Some(Expected {
                                                                range: at..end,
                                                                captures,
                                                            });
                                                        }
                                                        if sticky {
                                                            break None;
                                                        }
                                                        let Some((_, next)) = oracle.point(at)
                                                        else {
                                                            break None;
                                                        };
                                                        at = next;
                                                    }
                                                };
                                                let found = plan
                                                    .find_with_captures_and_work(
                                                        &input,
                                                        requested,
                                                        sticky,
                                                        |_| Ok::<(), ()>(()),
                                                    )
                                                    .unwrap();
                                                let actual = found.map(|range| Expected {
                                                    captures: (0..count)
                                                        .map(|slot| {
                                                            plan.capture_range(slot, &range)
                                                        })
                                                        .collect(),
                                                    range,
                                                });
                                                assert_eq!(
                                                    actual, expected,
                                                    "{source:?}, {input:?}, {requested}, sticky={sticky}, i={i}, m={m}"
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
fn nested_assertion_string_class_captures_recover_many_gated_layers_and_class_delimiters() {
    use super::RegExpUnicodeSingleStringClassCaptureMatcher as Single;
    let source = JsString::from(
        format!(
            "{}()([\\q{{|a|ab}}])(){}",
            "(^".repeat(20_000),
            "$)".repeat(20_000)
        )
        .as_str(),
    );
    let original =
        Single::compile_nested_assertion_wrappers_with_work(&source, false, false, |_| {
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.capture_count(), 20_003);
    for (input, end) in [(JsString::from("ab"), 2), (JsString::from(""), 0)] {
        let range = plan
            .find_with_captures_and_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(range, 0..end);
        for slot in 0..20_000 {
            assert_eq!(plan.capture_range(slot, &range), Some(range.clone()));
        }
        assert_eq!(plan.capture_range(20_000, &range), Some(0..0));
        assert_eq!(plan.capture_range(20_001, &range), Some(range.clone()));
        assert_eq!(plan.capture_range(20_002, &range), Some(end..end));
    }
    for (source, input, expected) in [
        (r"\b(^(\b()([\q{a|ab}])()\b)$)\b", "ab", Some(0..2)),
        (r"\b(^(\B()([\q{|a|ab}])())$)\b", "ab", None),
        (r"\B(^(\B()([\q{}])()\B)$)\B", "", Some(0..0)),
        (r"(^(\b()([\q{a|ab}])()\B))\B", "ab!", Some(0..1)),
        (r"(^(()([[\q{\(|\(a}][x\q{ab}]])())$)", "(a", Some(0..2)),
    ] {
        let plan = Single::compile_nested_assertion_wrappers_with_work(
            &JsString::from(source),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.capture_count(), 5);
        assert_eq!(
            plan.find_with_captures_and_work(
                &JsString::from(input),
                0,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn nested_assertion_string_class_cancellation_and_unknown_leaves_remain_complete() {
    use super::RegExpUnicodeSingleStringClassCaptureMatcher as Single;
    for source in [
        r"\b(^(\b()([\q{a|ab}])()\b)$)\b",
        r"\b(^(\B()([\q{|a|ab}])())$)\b",
    ] {
        let source = JsString::from(source);
        let mut full = Vec::new();
        let plan = Single::compile_nested_assertion_wrappers_with_work(&source, true, true, |n| {
            full.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=full.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(Single::compile_nested_assertion_wrappers_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n)) if n==fail)
            );
            assert_eq!(seen, full[..fail]);
        }
        let mut recovered = Vec::new();
        assert!(
            Single::compile_nested_assertion_wrappers_with_work(&source, true, true, |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(recovered, full);
        for input in [JsString::from("ab"), JsString::from("!")] {
            for sticky in [false, true] {
                let mut full = Vec::new();
                let expected = plan
                    .find_with_captures_and_work(&input, 0, sticky, |n| {
                        full.push(n);
                        Ok::<(), usize>(())
                    })
                    .unwrap();
                for fail in 1..=full.len() {
                    let mut seen = Vec::new();
                    assert_eq!(
                        plan.find_with_captures_and_work(&input, 0, sticky, |n| {
                            seen.push(n);
                            if seen.len() == fail {
                                Err(fail)
                            } else {
                                Ok(())
                            }
                        }),
                        Err(fail)
                    );
                    assert_eq!(seen, full[..fail]);
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
        r"((()([\q{a|ab}])()))",
        r"^()([\q{a|ab}])()$",
        r"^()[\q{a|ab}]|()$",
        r"\b(^(\B()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])())$)\b",
        r"\b(^(\B()([\q{a|ab}])\1)$)\b",
        r"\b(^(\B()([\q{a|ab}])+())$)\b",
        r"\b(^(\B()[\q{a|ab}]|())$)\b",
        r"(^()([\q{a|ab}])()$)()",
        r"^\b\B((()([\q{a|ab}])()))$",
    ] {
        assert!(
            Single::compile_nested_assertion_wrappers_with_work(
                &JsString::from(source),
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}
