#[test]
fn counted_nullable_string_classes_follow_independent_word_iteration_and_capture_oracle() {
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
        for layout in 0..6 {
            for (op, minimum, maximum, greedy) in [
                ("{2}", 2, Some(2), true),
                ("{2}?", 2, Some(2), false),
                ("{2,3}", 2, Some(3), true),
                ("{2,3}?", 2, Some(3), false),
                ("{3,4}", 3, Some(4), true),
                ("{3,4}?", 3, Some(4), false),
                ("{0002,0003}", 2, Some(3), true),
                ("{0002,0003}?", 2, Some(3), false),
                ("{2,}", 2, None, true),
                ("{3,}?", 3, None, false),
                ("{2,999999999999999999999999999999}", 2, None, true),
                ("{3,999999999999999999999999999999}?", 3, None, false),
            ] {
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
                        let source = if atomic {
                            format!("()({})()", body.replacen(class, &format!("{class}{op}"), 1))
                        } else {
                            format!("()(({body}){op})()")
                        };
                        for i in [false, true] {
                            for m in [false, true] {
                                let plan=crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher::compile_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap();
                                if mode == 0 && !words.contains(&"") {
                                    assert!(plan.is_none(), "{source}");
                                    continue;
                                }
                                let plan = plan.unwrap();
                                assert_eq!(
                                    plan.capture_count(),
                                    slots + if atomic { 3 } else { 4 }
                                );
                                for input in [
                                    JsString::from(""),
                                    JsString::from("!ababaaX"),
                                    JsString::from("aaaaa"),
                                    JsString::from("😀a😀!"),
                                    JsString::from("ſaKa"),
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
                                                let start = if requested > 0
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
                                                let mut at = start;
                                                let mut last = None;
                                                let mut selected = false;
                                                if mode == 2 {
                                                    last = Some(at..at);
                                                } else {
                                                    for _ in 0..minimum {
                                                        if let Some(end) =
                                                            word_root_end(&oracle, at, words)
                                                        {
                                                            last = Some(at..end);
                                                            selected = true;
                                                            if end == at {
                                                                break;
                                                            }
                                                            at = end;
                                                        } else if mode != 0 {
                                                            last = Some(at..at);
                                                            selected = false;
                                                            break;
                                                        } else {
                                                            return None;
                                                        }
                                                    }
                                                }
                                                if greedy {
                                                    let mut optional = 0;
                                                    loop {
                                                        if maximum.is_some_and(|max| {
                                                            minimum + optional >= max
                                                        }) {
                                                            break;
                                                        }
                                                        let Some(end) =
                                                            word_root_end(&oracle, at, words)
                                                        else {
                                                            break;
                                                        };
                                                        if end == at {
                                                            break;
                                                        }
                                                        last = Some(at..end);
                                                        at = end;
                                                        selected = true;
                                                        optional += 1;
                                                    }
                                                }
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
fn counted_nullable_string_classes_match_native_last_iteration_and_capture_tables() {
    for (source, input, i, sticky, start, expected, last, selected, captures) in [
        (
            r"()((([\q{a|ab}])|()){2})()",
            "ababaX",
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![
                Some(0..0),
                Some(0..4),
                Some(2..4),
                Some(2..4),
                None,
                Some(4..4),
            ],
        ),
        (
            r"()((()|([\q{a|ab}])){2,3})()",
            "ababaX",
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
            r"()((()|([\q{a|ab}])){2,3}?)()",
            "ab",
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
            r"()((([\q{a|ab}])|()){2,3}?)()",
            "ab",
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
            r"()((([\q{|a|ab}])|()){2})()",
            "ab",
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
            r"()((([\q{|a|ab}])){2})()",
            "ab",
            false,
            false,
            0,
            Some(0..2),
            Some(2..2),
            true,
            vec![Some(0..0), Some(0..2), Some(2..2), Some(2..2), Some(2..2)],
        ),
        (
            r"()((([\q{|a|ab}])){2,3})()",
            "ababaX",
            false,
            false,
            0,
            Some(0..5),
            Some(4..5),
            true,
            vec![Some(0..0), Some(0..5), Some(4..5), Some(4..5), Some(5..5)],
        ),
        (
            r"()((([\q{|a|ab}])){3}?)()",
            "ababaX",
            false,
            false,
            0,
            Some(0..5),
            Some(4..5),
            true,
            vec![Some(0..0), Some(0..5), Some(4..5), Some(4..5), Some(5..5)],
        ),
        (
            r"()((([\q{aa|aaa}])|()){2})()",
            "aaaa",
            false,
            false,
            0,
            Some(0..3),
            Some(3..3),
            false,
            vec![
                Some(0..0),
                Some(0..3),
                Some(3..3),
                None,
                Some(3..3),
                Some(3..3),
            ],
        ),
        (
            r"()((([\q{a|ab}])|()){2})()",
            "!ababX",
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
            r"()((()|([\q{a|ab}])){2,4})()",
            "ababaX",
            false,
            false,
            0,
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
            r"()((([\q{😀|😀a}])|()){2})()",
            "😀a😀!",
            false,
            true,
            1,
            Some(0..5),
            Some(3..5),
            true,
            vec![
                Some(0..0),
                Some(0..5),
                Some(3..5),
                Some(3..5),
                None,
                Some(5..5),
            ],
        ),
        (
            r"()((([\q{s|sa|k|ka}])|()){2})()",
            "ſaK!",
            true,
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
            r"()((([\q{ab}])|()){2})()",
            "!",
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
            r"()(([\q{}]{2}))()",
            "!",
            false,
            false,
            0,
            Some(0..0),
            Some(0..0),
            true,
            vec![Some(0..0), Some(0..0), Some(0..0), Some(0..0)],
        ),
        (
            r"()(([\q{|a|ab}]{2}))()",
            "ab",
            false,
            false,
            0,
            Some(0..2),
            Some(2..2),
            true,
            vec![Some(0..0), Some(0..2), Some(0..2), Some(2..2)],
        ),
        (
            r"()((([\q{|a|ab}])){2})()",
            "ab",
            false,
            false,
            0,
            Some(0..2),
            Some(2..2),
            true,
            vec![Some(0..0), Some(0..2), Some(2..2), Some(2..2), Some(2..2)],
        ),
        (
            r"()(([\q{|a|ab}]{2,3}?))()",
            "ababaX",
            false,
            false,
            0,
            Some(0..4),
            Some(2..4),
            true,
            vec![Some(0..0), Some(0..4), Some(0..4), Some(4..4)],
        ),
        (
            r"()((([\q{\n|\na}])|()){2})()",
            "\na",
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
            r"()((([\q{ab}&&\q{cd}])|()){2})()",
            "!",
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
            r"()((()|([\q{a|ab}])){99999999999999999999,100000000000000000000})()",
            "ababaX",
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
            r"()((([\q{a|ab}])|()){99999999999999999999,100000000000000000000})()",
            "ababaX",
            false,
            false,
            0,
            Some(0..5),
            Some(5..5),
            false,
            vec![
                Some(0..0),
                Some(0..5),
                Some(5..5),
                None,
                Some(5..5),
                Some(5..5),
            ],
        ),
        (
            r"()((([\q{|a|ab}])){99999999999999999999,100000000000000000000})()",
            "ababaX",
            false,
            false,
            0,
            Some(0..5),
            Some(5..5),
            true,
            vec![Some(0..0), Some(0..5), Some(5..5), Some(5..5), Some(5..5)],
        ),
        (
            r"()((()|([\q{a|ab}])){99999999999999999999})()",
            "ababaX",
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
            r"()((()|([\q{a|ab}])){000099999999999999999999,0000100000000000000000001})()",
            "ababaX",
            false,
            false,
            0,
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
    ] {
        let plan = crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher::compile_with_work(
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
fn counted_nullable_string_class_spines_keep_deep_scopes_and_callback_cancellation_flat() {
    use crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher as Plan;
    let source = JsString::from(
        format!(
            r"{}(([\q{{a|ab}}])|()){{3,4}}{}",
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
    assert_eq!(plan.capture_count(), 20_003);
    let found = plan
        .find_with_captures_and_work(&JsString::from("ababX"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..4);
    assert_eq!(found.last_iteration, Some(4..4));
    assert!(!found.class_selected);
    for n in 0..20_000 {
        assert_eq!(plan.capture_range(n, &found), Some(0..4));
    }
    assert_eq!(plan.capture_range(20_000, &found), Some(4..4));
    assert_eq!(plan.capture_range(20_001, &found), None);
    assert_eq!(plan.capture_range(20_002, &found), Some(4..4));
    assert_eq!(plan.capture_range(20_003, &found), None);
    for source in [
        r"()(([\q{a|ab}])|()){2,3}()",
        r"()(()|([\q{|a|ab}])){3,4}?()",
        r"()([\q{|a|ab}]{2,3})()",
        r"()(?:[\q{}]){3}()",
        r"()(()|([\q{a|ab}])){99999999999999999999,100000000000000000000}()",
        r"()(([\q{a|ab}])|()){99999999999999999999,100000000000000000000}()",
        r"()([\q{|a|ab}]){99999999999999999999,100000000000000000001}()",
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
        for input in [
            JsString::from("ababaX"),
            JsString::from("Xababa"),
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
                    plan.find_with_captures_and_work(&input, 0, sticky, |_| Ok::<(), usize>(())),
                    Ok(expected)
                );
            }
        }
    }
    for source in [
        r"()(([\q{a|ab}])|())()",
        r"(([\q{a|ab}])|()){1}",
        r"(([\q{a|ab}])|()){0,2}",
        r"(([\q{a|ab}])|())+",
        r"([\q{aa|aaa}]){2}",
        r"[\q{ab}&&\q{cd}]{2,3}",
        r"(([\q{a|ab}])|()){2,3}a",
        r"a(([\q{a|ab}])|()){2,3}",
        r"^(([\q{a|ab}])|()){2,3}",
        r"(([\q{a|ab}])|()){2,3}$",
        r"(([\q{a|ab}])|()){2,3}\1",
        r"(?=a)(([\q{a|ab}])|()){2,3}",
        r"(?:(\b)(([\q{a|ab}])|())){2}",
        r"(([\q{a|ab}])|()){2,3}|()",
        r"((([\q{a|ab}])|()){2,3}){3}",
        r"(([\q{a|ab}])|()){2,3}([\q{}])",
        r"(([\q{a|ab}])+|()){2,3}",
        r"(([\q{a|ab}])|()|()){2,3}",
        r"(([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()){2}",
        r"(()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){99999999999999999999}?",
    ] {
        assert!(
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}

#[test]
fn counted_nullable_string_class_huge_close_bounds_required_eps_and_zero_query_work() {
    use crate::RegExpUnicodeCountedNullableStringClassCaptureMatcher as Plan;
    let lower = "9".repeat(20_000);
    let upper = format!("1{}", "0".repeat(20_000));
    let source = JsString::from(format!(r"(()|([\q{{a|ab}}])){{{lower},{upper}}}").as_str());
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(&JsString::from("ababaX"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..2);
    assert_eq!(found.last_iteration, Some(0..2));
    assert!(found.class_selected);
    assert_eq!(plan.capture_range(0, &found), Some(0..2));
    assert_eq!(plan.capture_range(1, &found), None);
    assert_eq!(plan.capture_range(2, &found), Some(0..2));
    let source = JsString::from(format!(r"(()|([\q{{a|ab}}])){{{lower}}}").as_str());
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let input = JsString::from("x".repeat(100_000).as_str());
    let mut work = Vec::new();
    let found = plan
        .find_with_captures_and_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..0);
    assert_eq!(found.last_iteration, Some(0..0));
    assert!(!found.class_selected);
    assert_eq!(work, vec![4]);
    let source = JsString::from(r"()((((([\q{a|ab}]){1}|()){1}){3,4}){1})()");
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(&JsString::from("abX"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..2);
    assert_eq!(found.last_iteration, Some(2..2));
    assert!(!found.class_selected);
}
