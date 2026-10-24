fn required_test_ends(oracle: &Oracle<'_>, at: usize, words: &[&str]) -> Vec<usize> {
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
// Small-input recursive reference; every accepted member consumes a point.
fn required_test_tail(
    oracle: &Oracle<'_>,
    at: usize,
    words: &[&str],
    left: usize,
) -> Option<Range<usize>> {
    for end in required_test_ends(oracle, at, words) {
        if left == 1 {
            return Some(at..end);
        }
        if let Some(last) = required_test_tail(oracle, end, words, left - 1) {
            return Some(last);
        }
    }
    None
}
#[test]
fn required_string_classes_follow_independent_word_iteration_and_capture_oracle() {
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
                ("{3}", 3, Some(3), true),
                ("{3}?", 3, Some(3), false),
                ("{3,4}", 3, Some(4), true),
                ("{3,4}?", 3, Some(4), false),
                ("{4,5}", 4, Some(5), true),
                ("{4,5}?", 4, Some(5), false),
                ("{0003,0004}", 3, Some(4), true),
                ("{0003,0004}?", 3, Some(4), false),
                ("{3,}", 3, None, true),
                ("{3,}?", 3, None, false),
                ("{3,999999999999999999999999999999}", 3, None, true),
                ("{5}", 5, Some(5), true),
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
                                let plan=crate::RegExpUnicodeRequiredStringClassCaptureMatcher::compile_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap();
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
                                    JsString::from("!aaaaaaX"),
                                    JsString::from("abcabc"),
                                    JsString::from("😀a😀b😀a!"),
                                    JsString::from("ſaXſ!"),
                                    JsString::from("𐐀a𐐨a𐐀a!"),
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
                                                    let last = required_test_tail(
                                                        &oracle, start, words, minimum,
                                                    );
                                                    let complete = last.is_some();
                                                    let (mut at, mut last) =
                                                        if let Some(last) = last {
                                                            (last.end, Some(last))
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
fn required_string_classes_match_native_last_iteration_and_capture_tables() {
    for (source, input, i, sticky, start, expected, last, selected, captures) in [
        (
            r"()((([\q{aa|aaa}])){3})()",
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
            r"()((([\q{aa|aaa}])){3,4}?)()",
            "aaaaaaaa",
            false,
            false,
            0,
            Some(0..8),
            Some(6..8),
            true,
            vec![Some(0..0), Some(0..8), Some(6..8), Some(6..8), Some(8..8)],
        ),
        (
            r"()((([\q{a|ab}])){3})()",
            "ababa",
            false,
            false,
            0,
            Some(0..5),
            Some(4..5),
            true,
            vec![Some(0..0), Some(0..5), Some(4..5), Some(4..5), Some(5..5)],
        ),
        (
            r"()((([\q{a|ab}])){3})()",
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
            r"()((([\q{ab|abc|c}])){3})()",
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
            r"()((([\q{ab|abc|c}])){3,4})()",
            "abcabcab",
            false,
            false,
            0,
            Some(0..8),
            Some(6..8),
            true,
            vec![Some(0..0), Some(0..8), Some(6..8), Some(6..8), Some(8..8)],
        ),
        (
            r"()((([\q{a|aa}])){3,4})()",
            "aaaaaaaa",
            false,
            false,
            0,
            Some(0..8),
            Some(6..8),
            true,
            vec![Some(0..0), Some(0..8), Some(6..8), Some(6..8), Some(8..8)],
        ),
        (
            r"()((([\q{a|aa}])){3,4}?)()",
            "aaaaaaaa",
            false,
            false,
            0,
            Some(0..6),
            Some(4..6),
            true,
            vec![Some(0..0), Some(0..6), Some(4..6), Some(4..6), Some(6..6)],
        ),
        (
            r"()((([\q{aa|aaa}])){3})()",
            "!aaaaaa",
            false,
            false,
            0,
            Some(1..7),
            Some(5..7),
            true,
            vec![Some(1..1), Some(1..7), Some(5..7), Some(5..7), Some(7..7)],
        ),
        (
            r"()((([\q{aa|aaa}])){3})()",
            "!aaaaaa",
            false,
            true,
            0,
            None,
            None,
            false,
            vec![None, None, None, None, None],
        ),
        (
            r"()((([\q{😀|😀a|aX}])){3})()",
            "😀aX😀",
            false,
            true,
            1,
            Some(0..6),
            Some(4..6),
            true,
            vec![Some(0..0), Some(0..6), Some(4..6), Some(4..6), Some(6..6)],
        ),
        (
            r"()((([\q{S|sa|aX}])){3})()",
            "ſaXſ",
            true,
            false,
            0,
            Some(0..4),
            Some(3..4),
            true,
            vec![Some(0..0), Some(0..4), Some(3..4), Some(3..4), Some(4..4)],
        ),
        (
            r"()((([\p{Any}\q{ab}])){3})()",
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
            r"()((([\p{Any}\q{ab}])){3}?)()",
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
            r"()((([a-z\q{ab}])){3})()",
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
            r"()((([\p{Any}\q{ab}])){3})()",
            "😀aba",
            false,
            false,
            0,
            Some(0..5),
            Some(4..5),
            true,
            vec![Some(0..0), Some(0..5), Some(4..5), Some(4..5), Some(5..5)],
        ),
        (
            r"()((([\q{aa|aaa}])){3,999999999999999999999999999999})()",
            "aaaaaab",
            false,
            false,
            0,
            Some(0..6),
            Some(4..6),
            true,
            vec![Some(0..0), Some(0..6), Some(4..6), Some(4..6), Some(6..6)],
        ),
        (
            r"()(([\q{aa|aaa}]{3}))()",
            "aaaaaa",
            false,
            false,
            0,
            Some(0..6),
            Some(4..6),
            true,
            vec![Some(0..0), Some(0..6), Some(0..6), Some(6..6)],
        ),
        (
            r"()((([\q{ab|aba}])){3})()",
            "abababa",
            false,
            false,
            0,
            Some(0..7),
            Some(4..7),
            true,
            vec![Some(0..0), Some(0..7), Some(4..7), Some(4..7), Some(7..7)],
        ),
        (
            r"()((([\q{\(|\(a|aX}])){3})()",
            "(aX(",
            false,
            false,
            0,
            Some(0..4),
            Some(3..4),
            true,
            vec![Some(0..0), Some(0..4), Some(3..4), Some(3..4), Some(4..4)],
        ),
    ] {
        let plan = crate::RegExpUnicodeRequiredStringClassCaptureMatcher::compile_with_work(
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
fn required_string_class_deep_scopes_required_stacks_and_shared_prefixes_remain_flat() {
    use crate::RegExpUnicodeRequiredStringClassCaptureMatcher as Plan;
    let source = JsString::from(
        format!(
            r"{}([\q{{aa|aaa}}]){{3,4}}{}",
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
        .find_with_captures_and_work(&JsString::from("aaaaaa"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_count(), 20_001);
    assert_eq!(found.range, 0..6);
    assert_eq!(found.last_iteration, Some(4..6));
    for n in 0..20_000 {
        assert_eq!(plan.capture_range(n, &found), Some(0..6));
    }
    assert_eq!(plan.capture_range(20_000, &found), Some(4..6));
    assert_eq!(plan.capture_range(20_001, &found), None);
    let word = "x".repeat(20_000);
    let source = JsString::from(format!(r"()([\q{{{word}|{word}x}}]){{3}}()").as_str());
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(
            &JsString::from("x".repeat(60_000).as_str()),
            0,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..60000);
    assert_eq!(found.last_iteration, Some(40000..60000));
    let source = JsString::from(r"()([\q{aa|aaa}]){20000}()");
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let found = plan
        .find_with_captures_and_work(
            &JsString::from("a".repeat(60_000).as_str()),
            0,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..60000);
    assert_eq!(found.last_iteration, Some(59997..60000));
    let huge = "9".repeat(20_000);
    let source = JsString::from(format!(r"()([\q{{aa|aaa}}]){{{huge}}}()").as_str());
    let plan = Plan::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let mut work = Vec::new();
    assert!(
        plan.find_with_captures_and_work(&JsString::from("aaaaaa"), 0, false, |n| {
            work.push(n);
            Ok::<(), ()>(())
        })
        .unwrap()
        .is_none()
    );
    assert_eq!(work, vec![4]);
}
#[test]
fn required_string_class_cancellation_recovery_and_closed_proofs_keep_host_errors_separate() {
    use crate::{
        RegExpUnicodeMatchError as FindFailure,
        RegExpUnicodeRequiredStringClassCaptureMatcher as Plan,
        RegExpUnicodeSetCompileError as Failure,
    };
    for source in [
        r"()([\q{aa|aaa}]){3,4}()",
        r"()([\q{ab|abc|c}]){3}()",
        r"()([\p{Any}\q{ab}]{3,4}?)()",
        r"()(([\q{A|ab}]){1}){3,4}()",
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
                matches!(Plan::compile_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(Failure::Work(n))if n==fail)
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
        for input in ["aaaaaa", "abcab", "!abcab", "ab", "aababa", ""] {
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
        r"([\q{aa|aaa}]){2}",
        r"([\q{aa|aaa}]){0,3}",
        r"([\q{aa|aaa}])+",
        r"([\q{|aa|aaa}]){3}",
        r"(([\q{aa|aaa}])|()){3}",
        r"^([\q{aa|aaa}]){3}",
        r"([\q{aa|aaa}]){3}$",
        r"([\q{aa|aaa}]){3}\1",
        r"a([\q{aa|aaa}]){3}",
        r"([\q{aa|aaa}]){3}a",
        r"(([\q{aa|aaa}]){3}){3}",
        r"([\q{aa|aaa}]){3}[\q{ef}]",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){3}",
    ] {
        assert!(
            Plan::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
