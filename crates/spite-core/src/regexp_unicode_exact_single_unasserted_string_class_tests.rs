#[test]
fn exact_single_unasserted_nonempty_string_root_captures_follow_independent_finite_word_and_capture_tree_oracle()
 {
    let classes: [(&str, &[&str], Predicate); 14] = [
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
    ];
    for (class, words, contains) in classes {
        for layout in 0..6 {
            for empty_first in [false, true] {
                let mut tree = root_nullable_tree(layout, class, empty_first);
                let mut count = 0;
                let wrapped = tree.source(&mut count);
                let counted = wrapped[3..wrapped.len() - 1].replacen(
                    class,
                    &format!("(?:{class}{{0001,01}}?){{1,1}}"),
                    1,
                );
                let source = counted.as_str();
                for i in [false, true] {
                    for m in [false, true] {
                        let plan=super::RegExpUnicodeNullableStringClassCaptureMatcher::compile_exact_single_counts_with_work(&JsString::from(source),i,m,|_|Ok::<(),()>(())).unwrap().unwrap();
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
                                        Some(Expected {
                                            range: start..end,
                                            captures,
                                        })
                                    };
                                    let found = plan
                                        .find_with_captures_and_work(
                                            &input,
                                            requested,
                                            sticky,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap();
                                    let actual = found.map(|found| Expected {
                                        captures: (0..count)
                                            .map(|slot| {
                                                plan.capture_range(
                                                    slot,
                                                    found.atom_capture.as_ref(),
                                                )
                                            })
                                            .collect(),
                                        range: found.range,
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
fn exact_single_unasserted_empty_string_root_captures_follow_independent_finite_word_and_tagged_capture_oracle()
 {
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
                let counted = wrapped[3..wrapped.len() - 1].replacen(
                    class,
                    &format!("(?:{class}{{0001,01}}?){{1,1}}"),
                    1,
                );
                let source = counted.as_str();
                for i in [false, true] {
                    for m in [false, true] {
                        let plan =
                            super::RegExpUnicodeEmptyStringClassCaptureMatcher::compile_exact_single_counts_with_work(
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
fn exact_single_unasserted_nonempty_string_grouped_captures_follow_independent_finite_word_and_capture_tree_oracle()
 {
    let classes: [(&str, &[&str], Predicate); 14] = [
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
    ];
    for (class, words, contains) in classes {
        for layout in 0..6 {
            for framing in 0..4 {
                for empty_first in [false, true] {
                    let mut tree =
                        grouped_word_tree(root_nullable_tree(layout, class, empty_first), framing);
                    let mut count = 0;
                    let wrapped = tree.source(&mut count);
                    let counted = (wrapped.as_str()).replacen(
                        class,
                        &format!("(?:{class}{{0001,01}}?){{1,1}}"),
                        1,
                    );
                    let counted = format!("(?:{counted}){{1}}");
                    let source = counted.as_str();
                    for i in [false, true] {
                        for m in [false, true] {
                            let plan=super::RegExpUnicodeNullableStringClassCaptureMatcher::compile_exact_single_counts_with_work(&JsString::from(source),i,m,|_|Ok::<(),()>(())).unwrap().unwrap();
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
                                            Some(Expected {
                                                range: start..end,
                                                captures,
                                            })
                                        };
                                        let found = plan
                                            .find_with_captures_and_work(
                                                &input,
                                                requested,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = found.map(|found| Expected {
                                            captures: (0..count)
                                                .map(|slot| {
                                                    plan.capture_range(
                                                        slot,
                                                        found.atom_capture.as_ref(),
                                                    )
                                                })
                                                .collect(),
                                            range: found.range,
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
}

#[test]
fn exact_single_unasserted_empty_string_grouped_captures_follow_independent_finite_word_and_tagged_capture_oracle()
 {
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
            for framing in 0..4 {
                for empty_first in [false, true] {
                    let mut tree =
                        grouped_word_tree(root_nullable_tree(layout, class, empty_first), framing);
                    let mut count = 0;
                    let wrapped = tree.source(&mut count);
                    let counted = (wrapped.as_str()).replacen(
                        class,
                        &format!("(?:{class}{{0001,01}}?){{1,1}}"),
                        1,
                    );
                    let counted = format!("(?:{counted}){{1}}");
                    let source = counted.as_str();
                    for i in [false, true] {
                        for m in [false, true] {
                            let plan =
                            super::RegExpUnicodeEmptyStringClassCaptureMatcher::compile_exact_single_counts_with_work(
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
}

#[test]
fn exact_single_unasserted_string_class_deep_counts_callbacks_and_closed_domains_remain_flat() {
    macro_rules! check {
        ($matcher:ty, $class:expr) => {{
            let source = JsString::from(format!("(?:()(((){})()|((())|())?)(?:){{999999999999999999999999}}()){{1}}",$class).as_str());
            let mut full=Vec::new();
            let plan=<$matcher>::compile_exact_single_counts_with_work(&source,true,true,|n|{full.push(n);Ok::<(),usize>(())}).unwrap().unwrap();
            for fail in 1..=full.len() {
                let mut seen=Vec::new();
                assert!(matches!(<$matcher>::compile_exact_single_counts_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n)) if n==fail));
                assert_eq!(seen,full[..fail]);
            }
            let mut recovered=Vec::new();
            assert!(<$matcher>::compile_exact_single_counts_with_work(&source,true,true,|n|{recovered.push(n);Ok::<(),usize>(())}).unwrap().is_some());
            assert_eq!(recovered,full);
            for input in [JsString::from("ab!"),JsString::from("!")] {
                let mut full=Vec::new();
                let expected=plan.find_with_captures_and_work(&input,0,true,|n|{full.push(n);Ok::<(),usize>(())}).unwrap();
                for fail in 1..=full.len() {
                    let mut seen=Vec::new();
                    assert_eq!(plan.find_with_captures_and_work(&input,0,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(fail));
                    assert_eq!(seen,full[..fail]);
                }
                assert_eq!(plan.find_with_captures_and_work(&input,0,true,|_|Ok::<(),usize>(())).unwrap(),expected);
            }
            let source=JsString::from(format!("{}(?:{}|()){}","(".repeat(20_000),$class,"){1}".repeat(20_000)).as_str());
            let original=<$matcher>::compile_exact_single_counts_with_work(&source,false,false,|_|Ok::<(),()>(())).unwrap().unwrap();
            let plan=original.clone();drop(original);
            assert_eq!(plan.capture_count(),20_001);
            assert_eq!(plan.find_with_captures_and_work(&JsString::from("ab!"),0,true,|_|Ok::<(),()>(())).unwrap().unwrap().range,0..2);
            assert_eq!(plan.find_with_captures_and_work(&JsString::from("!"),0,true,|_|Ok::<(),()>(())).unwrap().unwrap().range,0..0);
            let zeros="0".repeat(20_000);
            let source=JsString::from(format!("(?:({}|())){{{zeros}1,{zeros}1}}?",$class).as_str());
            let plan=<$matcher>::compile_exact_single_counts_with_work(&source,false,false,|_|Ok::<(),()>(())).unwrap().unwrap();
            assert_eq!(plan.find_with_captures_and_work(&JsString::from("ab!"),0,true,|_|Ok::<(),()>(())).unwrap().unwrap().range,0..2);
            for source in [
                format!("({}|())",$class),
                format!("({}|()){{0}}",$class),format!("({}|()){{2}}",$class),format!("({}|()){{1,}}",$class),format!("({}|()){{1,2}}",$class),
                format!("({}|())+",$class),format!("({}|())?",$class),format!("({}|())*",$class),
                format!("(?:^({}|())){{1}}",$class),format!("(?:({}|())$){{1}}",$class),format!("(?:({}|())\\1){{1}}",$class),
                format!("(?:({}x|())){{1}}",$class),format!("(?:({}|()|())){{1}}",$class),format!("(?:({}|[\\q{{cd}}])){{1}}",$class),
                format!("(?:((\\b)?{}|())){{1}}",$class),format!("(?:({}{{2}}|())){{1}}",$class),
            ] {
                assert!(<$matcher>::compile_exact_single_counts_with_work(&JsString::from(source.as_str()),false,false,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
            }
            for source in [r"()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){1}",r"(?:([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()){1}"] {
                assert!(<$matcher>::compile_exact_single_counts_with_work(&JsString::from(source),false,false,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
            }

        }};
    }
    check!(
        super::RegExpUnicodeNullableStringClassCaptureMatcher,
        r"[\q{a|ab}]"
    );
    check!(
        super::RegExpUnicodeEmptyStringClassCaptureMatcher,
        r"[\q{|a|ab}]"
    );
    assert!(super::RegExpUnicodeNullableStringClassCaptureMatcher::compile_exact_single_counts_with_work(&JsString::from(r"(([\q{|a|ab}])|()){1}"),false,false,|_|Ok::<(),()>(())).unwrap().is_none());
    assert!(
        super::RegExpUnicodeEmptyStringClassCaptureMatcher::compile_exact_single_counts_with_work(
            &JsString::from(r"(([\q{a|ab}])|()){1}"),
            false,
            false,
            |_| Ok::<(), ()>(())
        )
        .unwrap()
        .is_none()
    );
}
#[test]
fn exact_single_unasserted_string_class_fixed_ranges_keep_class_empty_branch_identity() {
    use super::{
        RegExpUnicodeEmptyStringClassCaptureMatcher as Empty,
        RegExpUnicodeNullableStringClassCaptureMatcher as Nonempty,
    };
    for (source, input, i, start, range, captures) in [
        (
            r"()(([\q{a|ab}])|()){1}()",
            "ab!",
            false,
            0,
            0..2,
            vec![Some(0..0), Some(0..2), Some(0..2), None, Some(2..2)],
        ),
        (
            r"()(([\q{a|ab}])|()){1}()",
            "!",
            false,
            0,
            0..0,
            vec![Some(0..0), Some(0..0), None, Some(0..0), Some(0..0)],
        ),
        (
            r"()([\q{a|ab}]){0001,1}?()|()",
            "ab!",
            false,
            0,
            0..2,
            vec![Some(0..0), Some(0..2), Some(2..2), None],
        ),
        (
            r"()(([\q{s|sa|k|ka}]{1})|())()",
            "ſ!",
            true,
            0,
            0..1,
            vec![Some(0..0), Some(0..1), Some(0..1), None, Some(1..1)],
        ),
    ] {
        let source = JsString::from(source);
        assert!(
            Nonempty::compile_with_work(&source, i, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        assert!(
            Nonempty::compile_grouped_with_work(&source, i, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let plan =
            Nonempty::compile_exact_single_counts_with_work(
                &source,
                i,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), start, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range, "{source:?}");
        assert_eq!(plan.capture_count(), captures.len());
        assert_eq!(
            (0..captures.len())
                .map(|n| plan.capture_range(n, found.atom_capture.as_ref()))
                .collect::<Vec<_>>(),
            captures,
            "{source:?}"
        );
    }
    for (source, input, start, range, selected, captures) in [
        (
            r"()(([\q{|a|ab}])|()){1}()",
            "!",
            0,
            0..0,
            true,
            vec![Some(0..0), Some(0..0), Some(0..0), None, Some(0..0)],
        ),
        (
            r"()|()([\q{|a|ab}]){1}()",
            "ab!",
            0,
            0..0,
            false,
            vec![Some(0..0), None, None, None],
        ),
        (
            r"(){0}()(([\q{|😀|😀\n}])|()){1}()?",
            "😀\n!",
            1,
            0..3,
            true,
            vec![None, Some(0..0), Some(0..3), Some(0..3), None, None],
        ),
        (
            r"()([\q{}]{1})()|()",
            "x",
            1,
            1..1,
            true,
            vec![Some(1..1), Some(1..1), Some(1..1), None],
        ),
    ] {
        let source = JsString::from(source);
        assert!(
            Empty::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        assert!(
            Empty::compile_grouped_with_work(&source, false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let plan = Empty::compile_exact_single_counts_with_work(&source, false, false, |_| {
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), start, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range, "{source:?}");
        assert_eq!(found.class_selected, selected, "{source:?}");
        assert_eq!(plan.capture_count(), captures.len());
        assert_eq!(
            (0..captures.len())
                .map(|n| plan.capture_range(n, &found.range, found.class_selected))
                .collect::<Vec<_>>(),
            captures,
            "{source:?}"
        );
    }
}
