use super::*;

fn compile_wrapped(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeRepeatedAtomMatcher> {
    RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

fn last_character(input: &JsString, range: &Range<usize>) -> Option<Range<usize>> {
    if range.is_empty() {
        return None;
    }
    let units = input.code_units();
    let mut at = range.end - 1;
    if at > range.start
        && (0xdc00..=0xdfff).contains(&units[at])
        && (0xd800..=0xdbff).contains(&units[at - 1])
    {
        at -= 1
    }
    Some(at..range.end)
}

#[test]
fn complete_wrapped_character_results_match_independent_repetition_and_final_iteration_ranges() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate); 6] = [
        (r"[\q{a|ab}&&\p{Any}]", |p| p == 97),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[[\q{ab}&&a]b]", |p| p == 98),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
    ];
    for (class, contains) in classes {
        for (open, close, captures) in [
            ("(", ")", 1),
            ("((", "))", 2),
            ("(?:((", ")))", 2),
            ("(?:(?:", "))", 0),
        ] {
            for (quantifier, minimum, maximum, greedy) in [
                ("*", 0, None, true),
                ("*?", 0, None, false),
                ("+", 1, None, true),
                ("+?", 1, None, false),
                ("{0}", 0, Some(0), true),
                ("{1,3}", 1, Some(3), true),
                ("{1,3}?", 1, Some(3), false),
            ] {
                for (prefix, suffix, anchors, words) in [
                    ("", "", false, [None, None]),
                    (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                ] {
                    for i in [false, true] {
                        for m in [false, true] {
                            let source =
                                format!("{prefix}{open}{class}{close}{quantifier}{suffix}");
                            let plan = compile_wrapped(&source, i, m).unwrap();
                            assert_eq!(plan.capture_count(), captures);
                            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_v_set_with_single_character_strings_and_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap().is_none());
                            for input in [
                                JsString::from(""),
                                JsString::from("aaab!aba"),
                                JsString::from("A😀😀aa"),
                                JsString::from("!\r\naaa\u{2028}!"),
                                JsString::from("ſK"),
                                JsString::from("𐐀𐐨!"),
                                JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                            ] {
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = oracle(
                                            &input,
                                            start,
                                            sticky,
                                            contains,
                                            Bounds {
                                                minimum,
                                                maximum,
                                                greedy,
                                            },
                                            Gates {
                                                anchors,
                                                words,
                                                i,
                                                m,
                                            },
                                        );
                                        let expected_atom = if captures == 0 {
                                            None
                                        } else {
                                            expected
                                                .as_ref()
                                                .and_then(|range| last_character(&input, range))
                                        };
                                        let found = plan
                                            .find_with_captures_and_work(
                                                &input,
                                                start,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        assert_eq!(
                                            found.as_ref().map(|m| m.range.clone()),
                                            expected,
                                            "{source} i={i} m={m} {input:?} start={start} sticky={sticky}"
                                        );
                                        assert_eq!(
                                            found.and_then(|m| m.atom_capture),
                                            expected_atom,
                                            "{source} capture at start={start} sticky={sticky}"
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
fn wrapped_projection_keeps_zero_iteration_captures_undefined_and_complete_class_scope() {
    for source in [
        r"[\q{a|ab}&&a]+",
        r"(?<each>[\q{a|ab}&&a])+",
        r"(\b[\q{a|ab}&&a])+",
        r"([\q{a|ab}&&a]\b)+",
        r"([\q{a|ab}&&a]|b)+",
        r"([\q{ab}&&\q{cd}]){0}",
        r"([\q{ab}--\q{ab}])*",
        r"([\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}",
        r"([\q{a|ab}&&a])+[b]",
        r"\b^([\q{a|ab}&&a])+",
    ] {
        assert!(compile_wrapped(source, true, true).is_none(), "{source}");
    }
    for source in [r"(([\q{ab}&&a]))*", r"(([\q{a|ab}&&a])){0}"] {
        let plan = compile_wrapped(source, false, false).unwrap();
        assert_eq!(plan.capture_count(), 2);
        let found = plan
            .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.atom_capture, None);
    }
    let huge = "9".repeat(100);
    let source = format!(r"(([\q{{a|ab}}&&a])){{{huge}}}");
    let plan = compile_wrapped(&source, false, false).unwrap();
    assert_eq!(plan.minimum, None);
    assert_eq!(
        plan.find_with_captures_and_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
    let plan = compile_wrapped(r"(([\q{😀|😀x}&&\p{Any}]))+", false, false).unwrap();
    let found = plan
        .find_with_captures_and_work(&JsString::from("😀😀x"), 1, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..4);
    assert_eq!(found.atom_capture, Some(2..4));
}

#[test]
fn wrapped_character_result_construction_checks_callbacks_and_flat_deep_capture_storage() {
    for source in [r"^\b((?:([\q{a|ab}&&a]))){1,3}?\b$", r"(([\q{ab}&&a])){0}"] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
            &source,
            true,
            true,
            |n| {
                complete.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 16);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
                    &source,
                    true,
                    true,
                    |n| {
                        seen.push(n);
                        if seen.len() == fail {
                            Err(fail)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(
                matches!(result,Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at==fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
            &source,
            true,
            true,
            |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    let source = JsString::from(
        format!(
            r"{}[\q{{a|ab}}&&a]{}+",
            "(".repeat(20_000),
            ")".repeat(20_000)
        )
        .as_str(),
    );
    let mut work = 0usize;
    let original =
        RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
            &source,
            false,
            false,
            |n| {
                work += n;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
    assert!(
        work < source.len() * 32 + 1024,
        "wrapper work must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.capture_count(), 20_000);
    let found = plan
        .find_with_captures_and_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..3);
    assert_eq!(found.atom_capture, Some(2..3));
    let source =
        JsString::from(format!(r"([\q{{{}}}&&\p{{Any}}]){{0}}", "x".repeat(20_000)).as_str());
    let mut work = 0usize;
    let plan = RegExpUnicodeRepeatedAtomMatcher::compile_wrapped_character_result_v_set_with_work(
        &source,
        false,
        false,
        |n| {
            work += n;
            Ok::<(), ()>(())
        },
    )
    .unwrap()
    .unwrap();
    assert!(
        work < source.len() * 32 + 1024,
        "discarded words must stay compact: {work}"
    );
    let found = plan
        .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..0);
    assert_eq!(found.atom_capture, None);
}
