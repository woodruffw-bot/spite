fn two_required_test_ends(oracle: &Oracle<'_>, at: usize, words: &[&str]) -> Vec<usize> {
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
#[test]
fn two_required_string_classes_follow_independent_word_iteration_and_capture_oracle() {
    let classes: [(&str, &[&str], Predicate); 19] = [
        (r"[\q{ab|cd}]", &["ab", "cd"], |_| false),
        (r"[\q{ab|cde}]", &["ab", "cde"], |_| false),
        (r"[\q{ab|ac}]", &["ab", "ac"], |_| false),
        (r"[\q{ab}\q{cd}]", &["ab", "cd"], |_| false),
        (r"[[\q{ab}][\q{cd}]]", &["ab", "cd"], |_| false),
        (r"[\q{ab|abc}--\q{abc}]", &["ab"], |_| false),
        (r"[\q{ab|abc}--\q{ab}]", &["abc"], |_| false),
        (r"[\q{ab|cde}&&\q{ab|cde|z}]", &["ab", "cde"], |_| false),
        (r"[\q{😀a|😀b}]", &["😀a", "😀b"], |_| false),
        (r"[\q{\u{10428}a|\u{10428}b}]", &["𐐨a", "𐐨b"], |_| {
            false
        }),
        (r"[\q{Sx|k}]", &["Sx", "k"], |_| false),
        (r"[\q{abcd|bc|da}]", &["abcd", "bc", "da"], |_| false),
        (r"[\q{a|bc}]", &["a", "bc"], |_| false),
        (r"[\q{A|ab}]", &["A", "ab"], |_| false),
        (r"[\q{S|sa|AX}]", &["S", "sa", "AX"], |_| false),
        (r"[\q{aa|aaa}]", &["aa", "aaa"], |_| false),
        (r"[\q{|ab|cd}]", &["", "ab", "cd"], |_| false),
        (r"[\p{Any}\q{ab}]", &["ab"], |_| true),
        (r"[a-z\q{ab}]", &["ab"], |p| (97..=122).contains(&p)),
    ];
    for (class, words, contains) in classes {
        for layout in 0..6 {
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
                for mode in 0..1 {
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
                        let source = if atomic {
                            format!("()({})()", body.replacen(class, &format!("{class}{op}"), 1))
                        } else {
                            format!("()(({body}){op})()")
                        };
                        for i in [false, true] {
                            for m in [false, true] {
                                let plan=crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher::compile_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap();
                                if words.contains(&"") {
                                    assert!(plan.is_none(), "{source}");
                                    continue;
                                }
                                let plan = plan.unwrap_or_else(|| panic!("{source}, i={i}, m={m}"));
                                assert_eq!(
                                    plan.capture_count(),
                                    slots + if atomic { 3 } else { 4 }
                                );
                                for input in [
                                    JsString::from(""),
                                    JsString::from("!abcdabX"),
                                    JsString::from("abcda"),
                                    JsString::from("😀a😀b!"),
                                    JsString::from("ſxK!"),
                                    JsString::from("𐐀a𐐨!"),
                                    JsString::from("\na!"),
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
                                                let (at, last) = loop {
                                                    if minimum > input.len() - start {
                                                        return None;
                                                    }
                                                    let pair = two_required_test_ends(
                                                        &oracle, start, words,
                                                    )
                                                    .into_iter()
                                                    .find_map(|first_end| {
                                                        word_root_end(&oracle, first_end, words)
                                                            .map(|end| (first_end, end))
                                                    });
                                                    let complete = pair.is_some();
                                                    let (mut at, mut last) =
                                                        if let Some((first_end, end)) = pair {
                                                            (end, Some(first_end..end))
                                                        } else {
                                                            (start, None)
                                                        };
                                                    if complete {
                                                        let mut extra = 0;
                                                        while greedy
                                                            && maximum.is_none_or(|upper| {
                                                                minimum + extra < upper
                                                            })
                                                        {
                                                            let Some(end) =
                                                                word_root_end(&oracle, at, words)
                                                            else {
                                                                break;
                                                            };
                                                            last = Some(at..end);
                                                            at = end;
                                                            extra += 1;
                                                        }
                                                        break (at, last);
                                                    }
                                                    if sticky {
                                                        return None;
                                                    }
                                                    let (_, next) = oracle.point(start)?;
                                                    start = next;
                                                };
                                                let selected = true;
                                                let range = start..at;
                                                let mut values = vec![None; slots];
                                                if atomic {
                                                    assert_eq!(
                                                        eval_word_root_tree(
                                                            &tree,
                                                            true,
                                                            start,
                                                            Some(at),
                                                            &mut values
                                                        ),
                                                        Some(at)
                                                    );
                                                } else if let Some(last) = last.as_ref() {
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
                                                let mut captures =
                                                    vec![Some(start..start), Some(range.clone())];
                                                if !atomic {
                                                    captures.push(last.clone());
                                                }
                                                captures.extend(values);
                                                captures.push(Some(at..at));
                                                Some((range, last, selected, captures))
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
                                                "{source:?}, {input:?}, {requested}, sticky={sticky}, i={i},m={m}"
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
fn two_required_string_classes_match_native_last_iteration_and_capture_tables() {
    for (source, input, i, sticky, start, expected, last, selected, captures) in [
        (
            r"()((([\q{aa|aaa}])){2})()",
            "aaaa",
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"()((([\q{aa|aaa}])){2,3}?)()",
            "aaaaa",
            false,
            false,
            0,
            Some(0..5),
            Some(3..5),
            true,
            vec![Some(0..0), Some(0..5), Some(3..5), Some(3..5), Some(5..5)],
        ),
        (
            r"()((([\q{a|ab}])){2})()",
            "aba",
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![Some(0..0), Some(0..3), Some(2..3), Some(2..3), Some(3..3)],
        ),
        (
            r"()((([\q{a|ab}])){2})()",
            "ab",
            false,
            false,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"()((([\q{ab|abc|c}])){2})()",
            "abc",
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![Some(0..0), Some(0..3), Some(2..3), Some(2..3), Some(3..3)],
        ),
        (
            r"()((([\q{ab|abc|c}])){2,3})()",
            "abcab",
            false,
            false,
            0,
            Some(0..5),
            Some(3..5),
            true,
            vec![Some(0..0), Some(0..5), Some(3..5), Some(3..5), Some(5..5)],
        ),
        (
            r"()((([\q{a|aa}])){2,3})()",
            "aaaaaa",
            false,
            false,
            0,
            Some(0..6),
            Some(4..6),
            true,
            vec![Some(0..0), Some(0..6), Some(4..6), Some(4..6), Some(6..6)],
        ),
        (
            r"()((([\q{a|aa}])){2,3}?)()",
            "aaaaaa",
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"()((([\q{aa|aaa}])){2})()",
            "!aaaa",
            false,
            false,
            0,
            Some(1..5),
            Some(3..5),
            true,
            vec![Some(1..1), Some(1..5), Some(3..5), Some(3..5), Some(5..5)],
        ),
        (
            r"()((([\q{aa|aaa}])){2})()",
            "!aaaa",
            false,
            true,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"()((([\q{😀|😀a|aX}])){2})()",
            "😀aX",
            false,
            true,
            1,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"()((([\q{S|sa|aX}])){2})()",
            "ſaX",
            true,
            false,
            0,
            Some(0..3),
            Some(1..3),
            true,
            vec![Some(0..0), Some(0..3), Some(1..3), Some(1..3), Some(3..3)],
        ),
        (
            r"()((([\p{Any}\q{ab}])){2})()",
            "ab",
            false,
            false,
            0,
            Some(0..2),
            Some(1..2),
            true,
            vec![Some(0..0), Some(0..2), Some(1..2), Some(1..2), Some(2..2)],
        ),
        (
            r"()((([\p{Any}\q{ab}])){2}?)()",
            "abc",
            false,
            false,
            0,
            Some(0..3),
            Some(2..3),
            true,
            vec![Some(0..0), Some(0..3), Some(2..3), Some(2..3), Some(3..3)],
        ),
        (
            r"()((([a-z\q{ab}])){2})()",
            "ab",
            false,
            false,
            0,
            Some(0..2),
            Some(1..2),
            true,
            vec![Some(0..0), Some(0..2), Some(1..2), Some(1..2), Some(2..2)],
        ),
        (
            r"()((([\p{Any}\q{ab}])){2})()",
            "😀ab",
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(2..4), Some(2..4), Some(4..4)],
        ),
        (
            r"()((([\q{aa|aaa}])){2,999999999999999999999999999999})()",
            "aaaaab",
            false,
            false,
            0,
            Some(0..5),
            Some(3..5),
            true,
            vec![Some(0..0), Some(0..5), Some(3..5), Some(3..5), Some(5..5)],
        ),
        (
            r"()(([\q{aa|aaa}]{2}))()",
            "aaaa",
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(0..4), Some(4..4)],
        ),
        (
            r"()((([\q{ab|aba}])){2})()",
            "ababa",
            false,
            false,
            0,
            Some(0..5),
            Some(2..5),
            true,
            vec![Some(0..0), Some(0..5), Some(2..5), Some(2..5), Some(5..5)],
        ),
        (
            r"()((([\q{\(|\(a|aX}])){2})()",
            "(aX",
            false,
            false,
            0,
            Some(0..3),
            Some(1..3),
            true,
            vec![Some(0..0), Some(0..3), Some(1..3), Some(1..3), Some(3..3)],
        ),
    ] {
        let plan = crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher::compile_with_work(
            &JsString::from(source),
            i,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.capture_count(), captures.len(), "{source}");
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
fn two_required_string_class_deep_last_scopes_long_words_and_huge_failures_remain_flat() {
    use crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher as Plan;
    let source = JsString::from(
        format!(
            r"{}([\q{{aa|aaa}}]){{2,3}}{}",
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
        .find_with_captures_and_work(&JsString::from("aaaa"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_count(), 20_001);
    assert_eq!(found.range, 0..4);
    assert_eq!(found.last_iteration, Some(2..4));
    assert!(found.class_selected);
    for n in 0..20_000 {
        assert_eq!(plan.capture_range(n, &found), Some(0..4));
    }
    assert_eq!(plan.capture_range(20_000, &found), Some(2..4));
    assert_eq!(plan.capture_range(20_001, &found), None);
    let word = "x".repeat(20_000);
    let source = JsString::from(format!(r"()([\q{{{word}|{word}x}}]){{2}}()").as_str());
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(
            &JsString::from("x".repeat(40_000).as_str()),
            0,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..40000);
    assert_eq!(found.last_iteration, Some(20000..40000));
    assert_eq!(plan.capture_range(0, &found), Some(0..0));
    assert_eq!(plan.capture_range(1, &found), Some(20000..40000));
    assert_eq!(plan.capture_range(2, &found), Some(40000..40000));
}

#[test]
fn two_required_string_class_cancellation_recovery_and_nonprefix_languages_keep_their_domains() {
    use crate::RegExpUnicodeTwoRequiredStringClassCaptureMatcher as Plan;
    for source in [
        r"()([\q{aa|aaa}]){2,3}()",
        r"()([\q{ab|abc|c}]){2}()",
        r"()([\p{Any}\q{ab}]{2,3}?)()",
        r"()(([\q{A|ab}]){1}){2,3}()",
    ] {
        let source = JsString::from(source);
        let mut full = Vec::new();
        let plan = Plan::compile_with_work(&source, true, true, |n| {
            full.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=full.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(Plan::compile_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==fail)
            );
            assert_eq!(seen, full[..fail]);
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
        assert_eq!(recovered, full);
        for input in ["abcdab!", "abcda", "!abcd", "ab", ""] {
            for sticky in [false, true] {
                let input = JsString::from(input);
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
                    plan.find_with_captures_and_work(&input, 0, sticky, |_| Ok::<(), usize>(())),
                    Ok(expected)
                );
            }
        }
    }
    for source in [
        r"([\q{aa|aaa}]){3}",
        r"([\q{aa|aaa}]){0,2}",
        r"([\q{aa|aaa}]){1}",
        r"([\q{aa|aaa}])+",
        r"([\q{|aa|aaa}]){2}",
        r"(([\q{aa|aaa}])|()){2}",
        r"^([\q{aa|aaa}]){2}",
        r"([\q{aa|aaa}]){2}$",
        r"([\q{aa|aaa}]){2}\1",
        r"a([\q{aa|aaa}]){2}",
        r"([\q{aa|aaa}]){2}a",
        r"(([\q{aa|aaa}]){2}){3}",
        r"([\q{aa|aaa}]){2}[\q{ef}]",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){2}",
    ] {
        assert!(
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
    assert!(
        Plan::compile_with_work(&JsString::from(r"([\q{A|ab}]){2}"), true, false, |_| Ok::<
            (),
            (),
        >(
            ()
        ))
        .unwrap()
        .is_some()
    );
}
