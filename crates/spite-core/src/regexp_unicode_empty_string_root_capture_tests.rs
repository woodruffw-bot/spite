#[test]
fn empty_string_root_captures_follow_independent_finite_word_and_tagged_capture_oracle() {
    let classes: [(&str, &[&str], Predicate); 13] = [
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
            for empty_first in [false, true] {
                let mut tree = root_nullable_tree(layout, class, empty_first);
                let mut count = 0;
                let wrapped = tree.source(&mut count);
                let source = &wrapped[3..wrapped.len() - 1];
                for i in [false, true] {
                    for m in [false, true] {
                        let plan =
                            super::RegExpUnicodeEmptyStringClassCaptureMatcher::compile_with_work(
                                &JsString::from(source),
                                i,
                                m,
                                |_| Ok::<(), ()>(()),
                            )
                            .unwrap()
                            .unwrap();
                        assert_eq!(plan.capture_count(), count);
                        for input in [
                            JsString::from(""),
                            JsString::from("abcabx!"),
                            JsString::from("ABCa!"),
                            JsString::from("😀a😀!"),
                            JsString::from("ſaKa"),
                            JsString::from("𐐀a𐐨!"),
                            JsString::from("!\r\nabc"),
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
                                        let start = if requested > 0
                                            && units
                                                .get(requested)
                                                .is_some_and(|u| (0xdc00..=0xdfff).contains(u))
                                            && (0xd800..=0xdbff).contains(&units[requested - 1])
                                        {
                                            requested - 1
                                        } else {
                                            requested
                                        };
                                        let class_end = word_root_end(&oracle, start, words);
                                        let consuming = !empty_first && class_end.is_some();
                                        let mut captures = vec![None; count];
                                        let end = eval_word_root_tree(
                                            &tree,
                                            consuming,
                                            start,
                                            class_end,
                                            &mut captures,
                                        )
                                        .unwrap();
                                        Some((
                                            Expected {
                                                range: start..end,
                                                captures,
                                            },
                                            consuming,
                                        ))
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
                                                    .map(|slot| {
                                                        plan.capture_range(
                                                            slot,
                                                            &found.range,
                                                            found.class_selected,
                                                        )
                                                    })
                                                    .collect(),
                                                range: found.range,
                                            },
                                            found.class_selected,
                                        )
                                    });
                                    assert_eq!(
                                        actual, expected,
                                        "{source} i={i} m={m} {input:?} start={requested} sticky={sticky}"
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
fn empty_string_root_work_cancels_and_zero_width_class_captures_remain_defined() {
    use super::RegExpUnicodeEmptyStringClassCaptureMatcher as EmptyWords;
    for pattern in [
        r"()([\q{|a|ab}])()|()",
        r"()?|[\q{}]",
        r"([\q{|ab}--\q{ab}])|()",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        EmptyWords::compile_with_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 96 + 256);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = EmptyWords::compile_with_work(&source, true, true, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(
                matches!(result,Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at==fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        let plan = EmptyWords::compile_with_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
        for input in [JsString::from("ab!"), JsString::from("!")] {
            let mut full = Vec::new();
            let expected = plan
                .find_with_captures_and_work(&input, 0, true, |n| {
                    full.push(n);
                    Ok::<(), usize>(())
                })
                .unwrap();
            for fail in 1..=full.len() {
                let mut seen = Vec::new();
                assert_eq!(
                    plan.find_with_captures_and_work(&input, 0, true, |n| {
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
                plan.find_with_captures_and_work(&input, 0, true, |_| Ok::<(), usize>(()))
                    .unwrap(),
                expected
            );
        }
    }
    for empty_first in [false, true] {
        let character = format!(
            "{}[\\q{{|ab|abc}}]{}",
            "(".repeat(20_000),
            ")".repeat(20_000)
        );
        let source = JsString::from(
            if empty_first {
                format!("()|{character}")
            } else {
                format!("{character}|()")
            }
            .as_str(),
        );
        let original = EmptyWords::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), 20_001);
        for input in [JsString::from("abc!"), JsString::from("!")] {
            let found = plan
                .find_with_captures_and_work(&input, 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.class_selected, !empty_first);
            let end = if !empty_first && input.len() > 1 {
                3
            } else {
                0
            };
            assert_eq!(found.range, 0..end);
            for slot in 0..20_001 {
                let empty_slot = if empty_first { 0 } else { 20_000 };
                let expected = if slot == empty_slot {
                    empty_first.then_some(0..0)
                } else {
                    (!empty_first).then_some(0..end)
                };
                assert_eq!(
                    plan.capture_range(slot, &found.range, found.class_selected),
                    expected
                );
            }
            assert_eq!(
                plan.capture_range(20_001, &found.range, found.class_selected),
                None
            );
        }
    }
    let word = "x".repeat(20_000);
    let source = JsString::from(format!("([\\q{{|{word}}}])|()").as_str());
    let plan = EmptyWords::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let input = JsString::from(format!("{word}!").as_str());
    let found = plan
        .find_with_captures_and_work(&input, 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert!(found.class_selected);
    assert_eq!(found.range, 0..20_000);
    let empty = plan
        .find_with_captures_and_work(&JsString::from("!"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert!(empty.class_selected);
    assert_eq!(empty.range, 0..0);
    assert_eq!(
        plan.capture_range(0, &empty.range, empty.class_selected),
        Some(0..0)
    );
    assert_eq!(
        plan.capture_range(1, &empty.range, empty.class_selected),
        None
    );
}

#[test]
fn empty_string_root_proof_keeps_nonempty_results_assertions_references_and_unknown_terms_closed() {
    use super::RegExpUnicodeEmptyStringClassCaptureMatcher as EmptyWords;
    for pattern in [
        r"[\q{a|ab}]|()",
        r"[\q{|a|ab}--\q{}]|()",
        r"[\q{ab}--\q{ab}]|()",
        r"^([\q{|ab}])|()",
        r"([\q{|ab}])|()$",
        r"\b([\q{|ab}])|()",
        r"([\q{|ab}])|()\1",
        r"(?:([\q{|ab}])|())",
        r"([\q{|ab}])x|()",
        r"([\q{|ab}])?|()",
        r"([\q{|ab}])|([\q{|cd}])",
        r"([\q{|ab}])|()|()",
        r"([\q{|ab}&&\p{Any}&&\p{RGI_Emoji}])|()",
        r"([\q{a|b}])|()",
    ] {
        assert!(
            EmptyWords::compile_with_work(&JsString::from(pattern), true, true, |_| Ok::<(), ()>(
                ()
            ))
            .unwrap()
            .is_none(),
            "{pattern}"
        );
    }
}
