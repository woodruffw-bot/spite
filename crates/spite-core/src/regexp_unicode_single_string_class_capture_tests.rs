fn single_string_capture_tree(layout: usize, class: &'static str) -> BoundaryTree {
    use BoundaryTree::{Class, First, Repeat, Sequence};
    match layout {
        0 => Class(class),
        1 => BoundaryTree::capture(Class(class)),
        2 => Sequence(vec![
            BoundaryTree::empty_capture(),
            BoundaryTree::capture(Class(class)),
            BoundaryTree::empty_capture(),
        ]),
        3 => BoundaryTree::capture(Sequence(vec![
            BoundaryTree::capture(First(vec![
                BoundaryTree::empty_capture(),
                BoundaryTree::empty_capture(),
            ])),
            BoundaryTree::capture(Class(class)),
            Repeat(Box::new(BoundaryTree::empty_capture()), 0, "?".into()),
            BoundaryTree::empty_capture(),
        ])),
        4 => Sequence(vec![
            Repeat(
                Box::new(BoundaryTree::empty_capture()),
                1,
                "{999999999999999999999999999999}".into(),
            ),
            BoundaryTree::capture(BoundaryTree::capture(Class(class))),
            Repeat(
                Box::new(BoundaryTree::empty_capture()),
                0,
                "{0,999999999999999999999999999999}?".into(),
            ),
            BoundaryTree::capture(First(vec![
                BoundaryTree::empty_capture(),
                BoundaryTree::empty_capture(),
            ])),
        ]),
        _ => BoundaryTree::capture(Sequence(vec![
            First(vec![
                Repeat(Box::new(BoundaryTree::empty_capture()), 0, "*".into()),
                BoundaryTree::empty_capture(),
            ]),
            BoundaryTree::capture(Class(class)),
            BoundaryTree::capture(Sequence(vec![
                BoundaryTree::empty_capture(),
                BoundaryTree::empty_capture(),
            ])),
        ])),
    }
}

#[test]
fn single_string_class_captures_follow_independent_word_search_and_capture_tree_oracle() {
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
        for layout in 0..6 {
            let mut tree = single_string_capture_tree(layout, class);
            let mut count = 0;
            let source = tree.source(&mut count);
            for i in [false, true] {
                for m in [false, true] {
                    let plan =
                        super::RegExpUnicodeSingleStringClassCaptureMatcher::compile_with_work(
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
                            anchors: false,
                            words: [None, None],
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
                                        if let Some(end) = word_root_end(&oracle, at, words) {
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
                                    .find_with_captures_and_work(&input, requested, sticky, |_| {
                                        Ok::<(), ()>(())
                                    })
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

#[test]
fn single_string_class_captures_charge_before_construction_search_and_recover_after_abort() {
    use super::RegExpUnicodeSingleStringClassCaptureMatcher as Single;
    for class in [r"[\q{|a|ab}]", r"[\q{a|ab}]"] {
        let source = JsString::from(format!("()((())?({class})(()|())(?:){{1}})()").as_str());
        let mut full = Vec::new();
        let plan = Single::compile_with_work(&source, true, true, |n| {
            full.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=full.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(Single::compile_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n)) if n==fail)
            );
            assert_eq!(seen, full[..fail]);
        }
        let mut recovered = Vec::new();
        assert!(
            Single::compile_with_work(&source, true, true, |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(recovered, full);
        for input in [JsString::from("!ab!"), JsString::from("!")] {
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
}

#[test]
fn single_string_class_captures_keep_deep_storage_flat_and_other_grammar_proofs_separate() {
    use super::RegExpUnicodeSingleStringClassCaptureMatcher as Single;
    let source = JsString::from(
        format!("{}[\\q{{|a|ab}}]{}", "(".repeat(20_000), ")".repeat(20_000)).as_str(),
    );
    let original = Single::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.capture_count(), 20_000);
    for (input, end) in [(JsString::from("ab!"), 2), (JsString::from("!"), 0)] {
        let range = plan
            .find_with_captures_and_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(range, 0..end);
        for slot in 0..20_000 {
            assert_eq!(plan.capture_range(slot, &range), Some(0..end));
        }
        assert_eq!(plan.capture_range(20_000, &range), None);
    }
    let word = "x".repeat(20_000);
    let source = JsString::from(format!("()([\\q{{{word}}}])()").as_str());
    let plan = Single::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let input = JsString::from(format!("!!{word}!").as_str());
    let range = plan
        .find_with_captures_and_work(&input, 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(range, 2..20_002);
    assert_eq!(plan.capture_range(0, &range), Some(2..2));
    assert_eq!(plan.capture_range(1, &range), Some(range.clone()));
    assert_eq!(plan.capture_range(2, &range), Some(20_002..20_002));
    for source in [
        r"([\q{a|ab}])|()",
        r"([\q{a|ab}]|())",
        r"()([\q{a|ab}])+()",
        r"^()([\q{a|ab}])",
        r"()([\q{a|ab}])$",
        r"()([\q{a|ab}])\1",
        r"()([\q{a|ab}])x",
        r"()([\q{a|ab}])([\q{cd}])",
        r"()([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])()",
    ] {
        assert!(
            Single::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
