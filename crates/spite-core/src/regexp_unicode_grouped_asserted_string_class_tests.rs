#[test]
fn grouped_asserted_string_class_choices_follow_independent_ordered_branch_and_capture_oracle() {
    let classes: [(&str, &[&str], Predicate); 32] = [
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
        (r"[\q{a|\n|\na}]", &["a", "\n", "\na"], |_| false),
        (r"[\q{|a|\n|\na}]", &["", "a", "\n", "\na"], |_| false),
        (r"[\n\q{a|\na}]", &["a", "\na"], |p| p == 10),
        (r"[\q{|a|a\n}--\q{a}]", &["", "a\n"], |_| false),
        (r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]", &["\n", "\na"], |_| false),
    ];

    for (class, words, contains) in classes {
        for (cp, cs, ep, es) in [
            ("^", "$", "", ""),
            (r"\b", r"\b", r"\B", ""),
            (r"\B", r"\B", r"\b", ""),
            ("$", "^", "^", "$"),
            ("^", r"\B", "", "$"),
            ("", "", r"\B", ""),
            (r"\b\B", "", "^", ""),
            ("^", "$", r"\b\B", ""),
            ("$", "", "$", "^"),
            ("", "", "^", ""),
            ("^", "", "", "$"),
            ("", r"\B", r"\b", ""),
        ] {
            for layout in 0..6 {
                for empty_first in [false, true] {
                    for frame in 0..3 {
                        let mut tree = root_nullable_tree(layout, class, empty_first);
                        let prefix = if frame == 2 { 2 } else { 1 };
                        let mut count = prefix;
                        let BoundaryTree::Main(_, character, empty) = &mut tree else {
                            unreachable!()
                        };
                        let (character, empty) = if empty_first {
                            let e = empty.source(&mut count);
                            let c = character.source(&mut count);
                            (c, e)
                        } else {
                            (character.source(&mut count), empty.source(&mut count))
                        };
                        let c = format!("{cp}{character}{cs}");
                        let e = format!("{ep}{empty}{es}");
                        let source = if empty_first {
                            format!("{e}|{c}")
                        } else {
                            format!("{c}|{e}")
                        };
                        let source = match frame {
                            0 => format!("({source})"),
                            1 => format!("(?:({source}))"),
                            _ => format!("((?:({source})))"),
                        };
                        for index in (0..prefix).rev() {
                            tree = BoundaryTree::Capture(index, Box::new(tree));
                        }
                        for i in [false, true] {
                            for m in [false, true] {
                                let plan=super::RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_grouped_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap().unwrap();
                                assert_eq!(plan.capture_count(), count);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("!abcabx!"),
                                    JsString::from("!ABCa!"),
                                    JsString::from("!😀a😀!"),
                                    JsString::from("!ſaKa"),
                                    JsString::from("!𐐀a𐐨!"),
                                    JsString::from("!\r\nabc"),
                                    JsString::from("\na!"),
                                    JsString::from("a\nx"),
                                    JsString::from("\n\na\n"),
                                    JsString::from("\u{2028}a\u{2028}"),
                                    JsString::from_code_units(vec![0xd800, 97, 98, 0xdc00]),
                                ] {
                                    let oracle = Oracle {
                                        input: &input,
                                        contains,
                                        minimum: 1,
                                        maximum: Some(1),
                                        greedy: true,
                                        empty_first,
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
                                                    let c =
                                                        asserted_root_predicates(cp, &oracle, at)
                                                            .then(|| {
                                                                asserted_root_class_end(
                                                                    &oracle, at, words, cs,
                                                                )
                                                            })
                                                            .flatten();
                                                    let e =
                                                        asserted_root_predicates(ep, &oracle, at)
                                                            && asserted_root_predicates(
                                                                es, &oracle, at,
                                                            );
                                                    let selected = if empty_first && e {
                                                        Some((at, false))
                                                    } else if let Some(end) = c {
                                                        Some((end, true))
                                                    } else {
                                                        e.then_some((at, false))
                                                    };
                                                    if let Some((end, class_selected)) = selected {
                                                        let mut captures = vec![None; count];
                                                        assert_eq!(
                                                            eval_word_root_tree(
                                                                &tree,
                                                                class_selected,
                                                                at,
                                                                Some(end),
                                                                &mut captures
                                                            ),
                                                            Some(end)
                                                        );
                                                        break Some((
                                                            Expected {
                                                                range: at..end,
                                                                captures,
                                                            },
                                                            class_selected,
                                                        ));
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
                                            let actual = found.map(|found| {
                                                (
                                                    Expected {
                                                        captures: (0..count)
                                                            .map(|n| plan.capture_range(n, &found))
                                                            .collect(),
                                                        range: found.range.clone(),
                                                    },
                                                    found.class_selected,
                                                )
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
#[test]
fn grouped_asserted_string_class_prefix_captures_stay_defined_for_empty_branch_and_flat_at_depth() {
    use super::RegExpUnicodeAssertedStringClassAlternativeMatcher as Plan;
    let source = JsString::from(
        format!(
            "{}([\\q{{|a|ab}}])\\B|(^){}",
            "(".repeat(20_000),
            ")".repeat(20_000)
        )
        .as_str(),
    );
    let original = Plan::compile_grouped_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.capture_count(), 20_002);
    for (input, range, selected) in [
        ("ab!", 0..1, true),
        ("a", 0..0, false),
        ("!ab", 0..0, true),
        ("", 0..0, true),
    ] {
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.class_selected, selected);
        for n in 0..20_000 {
            assert_eq!(plan.capture_range(n, &found), Some(range.clone()));
        }
        assert_eq!(
            plan.capture_range(20_000, &found),
            selected.then_some(range.clone())
        );
        assert_eq!(
            plan.capture_range(20_001, &found),
            (!selected).then_some(range)
        );
        assert_eq!(plan.capture_range(20_002, &found), None);
    }
    for (source, input, range, selected, captures) in [
        (
            r"(?:^([\q{a|ab}])$|(\b))",
            "ab!",
            0..0,
            false,
            vec![None, Some(0..0)],
        ),
        (
            r"((?:^([\q{a|ab}])$|(\b)))",
            "ab!",
            0..0,
            false,
            vec![Some(0..0), None, Some(0..0)],
        ),
        (
            r"((^([\q{a|ab}])$|(\b)))",
            "ab",
            0..2,
            true,
            vec![Some(0..2), Some(0..2), Some(0..2), None],
        ),
        (
            r"((([\q{\||\|a}])\B|(^)))",
            "|ab",
            0..2,
            true,
            vec![Some(0..2), Some(0..2), Some(0..2), None],
        ),
    ] {
        let source = JsString::from(source);
        assert!(
            Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let plan = Plan::compile_grouped_with_work(&source, false, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range, "{source:?}");
        assert_eq!(found.class_selected, selected, "{source:?}");
        assert_eq!(
            (0..plan.capture_count())
                .map(|n| plan.capture_range(n, &found))
                .collect::<Vec<_>>(),
            captures,
            "{source:?}"
        );
    }
}
#[test]
fn grouped_asserted_string_class_cancellation_old_domains_and_partial_wrappers_stay_distinct() {
    use super::RegExpUnicodeAssertedStringClassAlternativeMatcher as Plan;
    for source in [
        r"((?:^()([\q{a|ab}])$|(\b)))",
        r"((\b)|()([\q{|a|ab}])()\B)",
        r"((?:\b\B([\q{|a|ab}])|(^)))",
    ] {
        let source = JsString::from(source);
        let mut full = Vec::new();
        let plan = Plan::compile_grouped_with_work(&source, true, true, |n| {
            full.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=full.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(Plan::compile_grouped_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n)) if n==fail)
            );
            assert_eq!(seen, full[..fail]);
        }
        let mut recovered = Vec::new();
        assert!(
            Plan::compile_grouped_with_work(&source, true, true, |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(recovered, full);
        for input in [
            JsString::from("ab!"),
            JsString::from("!"),
            JsString::from(""),
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
        r"^([\q{a|ab}])$|(\b)",
        r"(([\q{|a|ab}])|())",
        r"^((?:([\q{a|ab}])|()))",
        r"((^([\q{a|ab}])|()))$",
        r"(^([\q{a|ab}])|())()",
        r"()(^([\q{a|ab}])|())",
        r"(^([\q{a|ab}])|())?",
        r"(^([\q{a|ab}])|()|())",
        r"(^([\q{a|ab}])|([\q{}]))",
        r"(^([\q{a|ab}])|(\b)?)",
        r"(^([\q{a|ab}])|\1)",
        r"(\b\B([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|(^))",
        r"(^([\q{a|ab}])+|(^))",
    ] {
        assert!(
            Plan::compile_grouped_with_work(
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
