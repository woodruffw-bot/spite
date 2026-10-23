#[test]
fn asserted_single_string_class_captures_follow_filtered_word_and_capture_tree_oracle() {
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
            for layout in 0..6 {
                let mut tree = single_string_capture_tree(layout, class);
                let mut count = 0;
                let source = format!("{prefix}{}{suffix}", tree.source(&mut count));
                for i in [false, true] {
                    for m in [false, true] {
                        let plan =
                        super::RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_outer_assertions_and_work(
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
                                anchors: end_anchor,
                                words: [start_word, end_word],
                            };
                            for requested in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = if requested > input.len() {
                                        None
                                    } else {
                                        let units = input.code_units();
                                        let mut at = if requested > 0
                                            && units
                                                .get(requested)
                                                .is_some_and(|u| (0xdc00..=0xdfff).contains(u))
                                            && (0xd800..=0xdbff).contains(&units[requested - 1])
                                        {
                                            requested - 1
                                        } else {
                                            requested
                                        };
                                        loop {
                                            let at_line = !start_anchor
                                                || at == 0
                                                || (m && Oracle::line(input.code_units()[at - 1]));
                                            let at_word = start_word
                                                .is_none_or(|word| oracle.boundary(at) == word);
                                            if let Some(end) = (at_line && at_word)
                                                .then(|| word_root_end(&oracle, at, words))
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
                                            let Some((_, next)) = oracle.point(at) else {
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
                                            .map(|slot| plan.capture_range(slot, &range))
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

#[test]
fn asserted_single_string_class_retries_shorter_members_and_searches_after_rejected_empty_members()
{
    use super::RegExpUnicodeSingleStringClassCaptureMatcher as Single;
    for (source, input, expected) in [
        (r"()([\q{a|ab}])()\B", "ab!", Some(0..1)),
        (r"()([\q{|ab}])()$", "!ab", Some(1..3)),
        (r"()([\q{}])()$", "!ab", Some(3..3)),
        (r"^()([\q{|ab}])()$", "!ab", None),
        (r"^()([\q{|ab}])()$", "ab", Some(0..2)),
    ] {
        let plan = Single::compile_with_outer_assertions_and_work(
            &JsString::from(source),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(found, expected, "{source} {input:?}");
        if let Some(range) = found {
            assert_eq!(
                plan.capture_range(0, &range),
                Some(range.start..range.start)
            );
            assert_eq!(plan.capture_range(1, &range), Some(range.clone()));
            assert_eq!(plan.capture_range(2, &range), Some(range.end..range.end));
        }
    }
    // RegExpBuiltinExec advances failed Unicode starts by code point (22.2.7.2,
    // 22.2.7.3). V8 incorrectly finds an empty \B match inside the second pair.
    let plan = Single::compile_with_outer_assertions_and_work(
        &JsString::from(r"()([\q{|a|ab}])()\B"),
        false,
        false,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap();
    let input = JsString::from("😀a😀!");
    for start in [2, 3, 4] {
        for sticky in [false, true] {
            let found = plan
                .find_with_captures_and_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                .unwrap();
            assert_eq!(found, (!sticky).then_some(5..5));
            if let Some(range) = found {
                for slot in 0..3 {
                    assert_eq!(plan.capture_range(slot, &range), Some(5..5));
                }
            }
        }
    }
}

#[test]
fn asserted_single_string_class_cancellation_and_scope_controls_preserve_prior_domains() {
    use super::RegExpUnicodeSingleStringClassCaptureMatcher as Single;
    for source in [
        r"^()([\q{|a|ab}])()$",
        r"()([\q{|a|ab}])()$",
        r"\b()([\q{a|ab}])()\B",
    ] {
        let source = JsString::from(source);
        let mut full = Vec::new();
        let plan = Single::compile_with_outer_assertions_and_work(&source, true, true, |n| {
            full.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=full.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(Single::compile_with_outer_assertions_and_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n)) if n==fail)
            );
            assert_eq!(seen, full[..fail]);
        }
        let mut recovered = Vec::new();
        assert!(
            Single::compile_with_outer_assertions_and_work(&source, true, true, |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(recovered, full);
        for input in [
            JsString::from("!ab"),
            JsString::from("ab!"),
            JsString::from("!"),
        ] {
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
        r"()([\q{a|ab}])()",
        r"^()([\q{a|ab}])|()",
        r"()[\q{a|ab}]|()$",
        r"^()\b([\q{a|ab}])$",
        r"^()([\q{a|ab}])\1$",
        r"^()([\q{a|ab}])+()$",
        r"^()([\q{a|ab}])x$",
        r"^()([\q{a|ab}])([\q{cd}])$",
        r"^^()([\q{a|ab}])$",
        r"^()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()$",
    ] {
        assert!(
            Single::compile_with_outer_assertions_and_work(
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
