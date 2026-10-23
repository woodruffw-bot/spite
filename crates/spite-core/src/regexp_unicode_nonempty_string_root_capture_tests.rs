fn word_root_fold(point: u32) -> u32 {
    match point {
        0x41..=0x5a => point + 32,
        0x17f => 0x73,
        0x212a => 0x6b,
        0x10400..=0x10427 => point + 0x28,
        _ => point,
    }
}
fn word_root_end(oracle: &Oracle<'_>, at: usize, words: &[&str]) -> Option<usize> {
    let mut best = oracle.point(at).and_then(|(mut point, end)| {
        if oracle.i {
            point = word_root_fold(point)
        }
        ((oracle.contains)(point) && oracle.end(end)).then_some(end)
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
        if matched && oracle.end(end) {
            best = Some(best.map_or(end, |old| old.max(end)));
        }
    }
    best
}
fn eval_word_root_tree(
    tree: &BoundaryTree,
    consuming: bool,
    at: usize,
    end: Option<usize>,
    captures: &mut [Option<Range<usize>>],
) -> Option<usize> {
    use BoundaryTree::{Capture, Class, Empty, First, Main, Repeat, Sequence};
    match tree {
        Empty => Some(at),
        Class(_) => {
            if consuming {
                end
            } else {
                None
            }
        }
        Capture(index, child) => {
            let next = eval_word_root_tree(child, consuming, at, end, captures)?;
            captures[*index] = Some(at..next);
            Some(next)
        }
        Sequence(nodes) => {
            let mut next = at;
            for child in nodes {
                next = eval_word_root_tree(child, consuming, next, end, captures)?;
            }
            Some(next)
        }
        Main(_, character, empty) => eval_word_root_tree(
            if consuming { character } else { empty },
            consuming,
            at,
            end,
            captures,
        ),
        First(nodes) => eval_word_root_tree(nodes.first().unwrap(), consuming, at, end, captures),
        Repeat(child, minimum, _) => {
            for _ in 0..*minimum {
                child.clear(captures);
                assert_eq!(
                    eval_word_root_tree(child, consuming, at, end, captures),
                    Some(at)
                );
            }
            Some(at)
        }
    }
}

#[test]
fn nonempty_string_root_captures_follow_independent_finite_word_and_capture_tree_oracle() {
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
                let source = &wrapped[3..wrapped.len() - 1];
                for i in [false, true] {
                    for m in [false, true] {
                        let plan=super::RegExpUnicodeNullableStringClassCaptureMatcher::compile_with_work(&JsString::from(source),i,m,|_|Ok::<(),()>(())).unwrap().unwrap();
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
fn nonempty_string_root_work_cancels_and_long_words_and_capture_tables_stay_flat() {
    use super::RegExpUnicodeNullableStringClassCaptureMatcher as Words;
    for pattern in [
        r"()([\q{a|ab|abc}])()|()",
        r"()?|[\q{ab}--\q{ab}]",
        r"([\q{|a|ab}--\q{}])|()",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Words::compile_with_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 96 + 256);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = Words::compile_with_work(&source, true, true, |n| {
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
        let plan = Words::compile_with_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
        for input in [JsString::from("abc!"), JsString::from("!")] {
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
    let source = JsString::from(
        format!(
            "{}[\\q{{ab|abc}}]{}|()",
            "(".repeat(20_000),
            ")".repeat(20_000)
        )
        .as_str(),
    );
    let original = Words::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let plan = original.clone();
    drop(original);
    let found = plan
        .find_with_captures_and_work(&JsString::from("abc!"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..3);
    assert_eq!(plan.capture_count(), 20_001);
    for slot in 0..20_000 {
        assert_eq!(
            plan.capture_range(slot, found.atom_capture.as_ref()),
            Some(0..3)
        );
    }
    assert_eq!(
        plan.capture_range(20_000, found.atom_capture.as_ref()),
        None
    );
    let word = "x".repeat(20_000);
    let source = JsString::from(format!("([\\q{{{word}}}])|()").as_str());
    let plan = Words::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let input = JsString::from(format!("{word}!").as_str());
    let mut work = 0;
    let found = plan
        .find_with_captures_and_work(&input, 0, true, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..20_000);
    assert_eq!(
        plan.capture_range(0, found.atom_capture.as_ref()),
        Some(0..20_000)
    );
    assert!(work < 200_000, "{work}");
}

#[test]
fn nonempty_string_root_proof_declines_empty_class_results_assertions_references_and_unknown_terms()
{
    use super::RegExpUnicodeNullableStringClassCaptureMatcher as Words;
    for pattern in [
        r"[\q{|a|ab}]|()",
        r"()|[\q{|ab}]",
        r"[\q{|a|ab}&&\q{|a|ab}]|()",
        r"^([\q{ab}])|()",
        r"([\q{ab}])|()$",
        r"\b([\q{ab}])|()",
        r"([\q{ab}])|()\1",
        r"(?:([\q{ab}])|())",
        r"([\q{ab}])x|()",
        r"([\q{ab}])?|()",
        r"([\q{ab}])|([\q{cd}])",
        r"([\q{ab}])|()|()",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])|()",
        r"([\q{a|b}])|()",
    ] {
        assert!(
            Words::compile_with_work(&JsString::from(pattern), true, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{pattern}"
        );
    }
}
