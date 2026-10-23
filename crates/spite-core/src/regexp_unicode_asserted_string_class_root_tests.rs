fn asserted_root_predicates(source: &str, oracle: &Oracle<'_>, at: usize) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let pass = match byte {
            b'^' => at == 0 || (oracle.m && Oracle::line(oracle.input.code_units()[at - 1])),
            b'$' => {
                at == oracle.input.len()
                    || (oracle.m
                        && oracle
                            .input
                            .code_units()
                            .get(at)
                            .is_some_and(|u| Oracle::line(*u)))
            }
            b'\\' => {
                index += 1;
                match bytes[index] {
                    b'b' => oracle.boundary(at),
                    b'B' => !oracle.boundary(at),
                    _ => panic!("oracle predicate"),
                }
            }
            _ => panic!("oracle predicate"),
        };
        if !pass {
            return false;
        }
        index += 1;
    }
    true
}
fn asserted_root_class_end(
    oracle: &Oracle<'_>,
    at: usize,
    words: &[&str],
    suffix: &str,
) -> Option<usize> {
    let mut best = oracle.point(at).and_then(|(mut point, end)| {
        if oracle.i {
            point = word_root_fold(point)
        }
        ((oracle.contains)(point) && asserted_root_predicates(suffix, oracle, end)).then_some(end)
    });
    for word in words {
        let mut end = at;
        let mut matched = true;
        for ch in word.chars() {
            let Some((mut point, next)) = oracle.point(end) else {
                matched = false;
                break;
            };
            let mut expected = u32::from(ch);
            if oracle.i {
                point = word_root_fold(point);
                expected = word_root_fold(expected)
            }
            if point != expected {
                matched = false;
                break;
            }
            end = next;
        }
        if matched && asserted_root_predicates(suffix, oracle, end) {
            best = Some(best.map_or(end, |old| old.max(end)));
        }
    }
    best
}
#[test]
fn asserted_string_class_root_choices_follow_independent_ordered_branch_and_capture_oracle() {
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
                    let mut tree = root_nullable_tree(layout, class, empty_first);
                    let mut count = 0;
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
                    for i in [false, true] {
                        for m in [false, true] {
                            let plan=super::RegExpUnicodeAssertedStringClassAlternativeMatcher::compile_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap().unwrap();
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
                                                let c = asserted_root_predicates(cp, &oracle, at)
                                                    .then(|| {
                                                        asserted_root_class_end(
                                                            &oracle, at, words, cs,
                                                        )
                                                    })
                                                    .flatten();
                                                let e = asserted_root_predicates(ep, &oracle, at)
                                                    && asserted_root_predicates(es, &oracle, at);
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
#[test]
fn asserted_string_class_root_captures_keep_epsilon_branch_identity_and_deep_lexical_slots_flat() {
    use super::RegExpUnicodeAssertedStringClassAlternativeMatcher as Plan;
    let source = JsString::from(
        format!(
            "{}([\\q{{|a|ab}}]){}\\B|(^)",
            "(".repeat(20_000),
            ")".repeat(20_000)
        )
        .as_str(),
    );
    let original = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.capture_count(), 20_002);
    for (input, range, class_selected) in [
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
        assert_eq!(found.class_selected, class_selected);
        for n in 0..20_001 {
            assert_eq!(
                plan.capture_range(n, &found),
                class_selected.then_some(range.clone())
            );
        }
        assert_eq!(
            plan.capture_range(20_001, &found),
            (!class_selected).then_some(range)
        );
        assert_eq!(plan.capture_range(20_002, &found), None);
    }
    let input = JsString::from("x".repeat(40_000).as_str());
    for (source, selected) in [
        (r"|(\b\B[\q{a|ab}])", false),
        (r"([\q{|a|ab}])|\b\B()", true),
    ] {
        let plan =
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
        let mut work = 0usize;
        let found = plan
            .find_with_captures_and_work(&input, 0, false, |n| {
                work += n;
                if work > 100 { Err(()) } else { Ok(()) }
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.class_selected, selected);
        assert!(work <= 100);
    }
    for (source, input, start, range, selected, captures) in [
        (
            r"(^)|()([\q{|a|ab}])()",
            "ab!",
            0,
            0..0,
            false,
            vec![Some(0..0), None, None, None],
        ),
        (
            r"(^)|()([\q{|a|ab}])()",
            "ab!",
            1,
            1..1,
            true,
            vec![None, Some(1..1), Some(1..1), Some(1..1)],
        ),
        (
            r"^([\q{a|ab}])$|(\b)",
            "ab!",
            0,
            0..0,
            false,
            vec![None, Some(0..0)],
        ),
        (
            r"^([\q{a|ab}])$|(\b)",
            "!ab",
            0,
            1..1,
            false,
            vec![None, Some(1..1)],
        ),
        (
            r"^([\q{a|ab}])$|(\b)",
            "ab",
            0,
            0..2,
            true,
            vec![Some(0..2), None],
        ),
        (
            r"([\q{\||\|a}])\B|(^)",
            "|ab",
            0,
            0..2,
            true,
            vec![Some(0..2), None],
        ),
    ] {
        let plan =
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), start, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range, "{source}");
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
#[test]
fn asserted_string_class_root_cancellation_scope_and_full_leaf_proofs_stay_distinct() {
    use super::RegExpUnicodeAssertedStringClassAlternativeMatcher as Plan;
    for source in [
        r"^()([\q{a|ab}])$|(\b)",
        r"(\b)|()([\q{|a|ab}])()\B",
        r"\b\B([\q{|a|ab}])|(^)",
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
                matches!(Plan::compile_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n)) if n==fail)
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
            JsString::from("ab!"),
            JsString::from("!"),
            JsString::from(""),
        ] {
            for sticky in [false, true] {
                let mut full = Vec::new();
                let found = plan
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
                    found
                );
            }
        }
    }
    for source in [
        r"([\q{|a|ab}])|()",
        r"^([\q{a|ab}]|())",
        r"^([\q{a|ab}])+|()",
        r"^([\q{a|ab}])|()|()",
        r"^([\q{a|ab}])|([\q{}])",
        r"^([\q{a|ab}])|(\b)?",
        r"^([\q{a|ab}])|(\b|)",
        r"^([\q{a|ab}])|\1",
        r"\b\B([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|(^)",
        r"^([\q{a|ab}])x|(^)",
        r"|(\b\B[\q{ab}&&\p{Any}&&\p{RGI_Emoji}])",
    ] {
        assert!(
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
