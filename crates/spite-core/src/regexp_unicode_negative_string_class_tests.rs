use super::*;

fn compile(source: &str, i: bool, m: bool) -> RegExpUnicodeNegativeStringClassMatcher {
    RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_validated_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn negative_string_classes_keep_empty_member_failure_and_undefined_operand_slots() {
    for (class, text, expected) in [
        (r"[\q{a|aaa|aa}]", "aaa!", Some(3..3)),
        (r"[\q{ab}]", "a", Some(0..0)),
        (r"[\q{ab|a|}]", "!ab", None),
        (r"[\q{}]", "", None),
        (r"[\q{a|aaa|aa}]$", "aaa", Some(3..3)),
        (r"[\q{a|aaa|aa}]$", "aaa!", Some(0..0)),
    ] {
        let plan = compile(&format!("((?!(({class}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(found, expected, "{class}");
        assert_eq!(plan.capture_count(), 3);
        if let Some(range) = found {
            assert_eq!(plan.capture_range(0, &range), Some(range.clone()));
            for index in 1..=3 {
                assert_eq!(plan.capture_range(index, &range), None);
            }
        }
    }
}

#[test]
fn negative_string_classes_keep_original_line_fold_and_complete_surrogate_context() {
    for (source, i, m, text, start, sticky, expected) in [
        (
            r"(?!(^[\q{ab|a|}]$))",
            true,
            true,
            "\nA\n",
            0,
            false,
            Some(2..2),
        ),
        (
            r"(?!(^[\q{ab|a|}]$))",
            false,
            true,
            "\nA\n",
            0,
            false,
            Some(1..1),
        ),
        (
            r"(?!([\q{ſa|S|Kk}]))",
            true,
            false,
            "SA!",
            0,
            false,
            Some(1..1),
        ),
        (
            r"(?!([\q{😀a|😀}]))",
            false,
            false,
            "😀😀a!",
            0,
            false,
            Some(4..4),
        ),
        (r"(?!([\q{😀a|😀}]))", false, false, "!😀a", 2, true, None),
        (
            r"(?!([\q{\u{D83D}\u{DE00}}]))",
            false,
            false,
            "😀",
            0,
            true,
            Some(0..0),
        ),
        (r"(?!([\q{ab}]))", false, false, "", 1, false, None),
    ] {
        assert_eq!(
            compile(source, i, m)
                .find_with_work(&JsString::from(text), start, sticky, |_| Ok::<(), ()>(()))
                .unwrap(),
            expected,
            "{source}"
        );
    }
    let plan = compile(r"(?!([\q{\uD800x|\uD800}]))", false, false);
    assert_eq!(
        plan.find_with_work(
            &JsString::from_code_units(vec![0xd800, 0x78]),
            0,
            false,
            |_| Ok::<(), ()>(())
        )
        .unwrap(),
        Some(1..1)
    );
    assert_eq!(
        plan.find_with_work(&JsString::from("😀"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..0)
    );
}

#[test]
fn negative_string_class_proof_declines_other_widths_directions_scopes_and_binding_roles() {
    for source in [
        r"(?!([\q{a|b}]))",
        r"(?!([a-z]))",
        r"(?!([\q{ab}&&a]))",
        r"(?!([a\q{ab}]))",
        r"(?!([\q{ab}]{1}))",
        r"(?!([\q{ab}]?))",
        r"(?!([\q{ab}]a))",
        r"(?!([\q{ab}]|a))",
        r"(?<!([\q{ab}]))",
        r"(?=([\q{ab}]))",
        r"(?!([\q{ab}]\b))",
        r"(?!(\B[\q{ab}]))",
        r"(?!(?i:[\q{ab}]))",
        r"(?!([\q{ab}])\1)",
        r"(?!([\q{ab}])$)",
        r"(?!([\p{RGI_Emoji}]))",
    ] {
        assert!(RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_validated_with_work(
            &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
        ).unwrap().is_none(), "{source}");
    }
}

#[test]
fn negative_string_classes_keep_deep_slots_and_recover_from_every_work_abort() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?!([\q{ab}]))" + &")".repeat(depth);
    let plan = compile(&source, false, false);
    assert_eq!(plan.capture_count(), depth + 1);
    let found = plan
        .find_with_work(&JsString::from("a"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_range(depth - 1, &found), Some(0..0));
    assert_eq!(plan.capture_range(depth, &found), None);
    let source = JsString::from(r"((?!(([\q{a|aaa|aa}]))))");
    let mut construction = Vec::new();
    let plan =
        RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_validated_with_work(
            &source,
            false,
            false,
            |n| {
                construction.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
    for stop in 0..construction.len() {
        let mut seen = Vec::new();
        assert!(
            matches!(RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_validated_with_work(
            &source, false, false, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from("aaaa!");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected, Some(4..4));
    for stop in 0..work.len() {
        let mut seen = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |n| {
                seen.push(n);
                if seen.len() == stop + 1 {
                    Err(stop)
                } else {
                    Ok(())
                }
            }),
            Err(stop)
        );
        assert_eq!(seen, work[..=stop]);
        assert_eq!(
            plan.clone()
                .find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    let input = JsString::from(("a".repeat(20_000) + "!").as_str());
    let mut total = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, false, |n| {
            total += n;
            Ok::<(), ()>(())
        })
        .unwrap(),
        Some(20_000..20_000)
    );
    assert!(total <= input.len() * 500 + 128, "{total}");
    let word = "a".repeat(100_000) + "b";
    let plan = compile(&format!(r"(?!([\q{{{word}|}}]))"), false, false);
    let input = JsString::from(("a".repeat(100_000) + "c").as_str());
    let mut calls = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, true, |_| {
            calls += 1;
            if calls == 17 { Err(()) } else { Ok(()) }
        }),
        Err(())
    );
    assert_eq!(calls, 17);
    assert_eq!(
        plan.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
}

fn word_compile(source: &str, i: bool, m: bool) -> RegExpUnicodeNegativeStringClassMatcher {
    RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
        &JsString::from(source), i, m, |_| Ok::<(), ()>(()),
    ).unwrap().unwrap()
}
#[test]
fn negative_word_string_classes_negate_the_whole_endpoint_conjunction_and_keep_undefined_slots() {
    for (body, text, expected) in [
        (r"\b[\q{ab|a|}]\b", "ab!", Some(1..1)),
        (r"\b[\q{ab|a|}]\B", "ab!", Some(1..1)),
        (r"\B[\q{ab|a|}]\b", "ab!", Some(0..0)),
        (r"\B[\q{ab|a|}]\B", "ab!", Some(0..0)),
        (r"[\q{}]\b", "aa!", Some(1..1)),
        (r"[\q{}]\b", "a", None),
        (r"\b[\q{ab|a|}]\b", "", Some(0..0)),
        (r"\B[\q{}]\B", "", None),
    ] {
        let plan = word_compile(&format!("((?!(({body}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(found, expected, "{body}");
        assert_eq!(plan.capture_count(), 3);
        if let Some(range) = found {
            assert_eq!(plan.capture_range(0, &range), Some(range.clone()));
            for index in 1..=3 {
                assert_eq!(plan.capture_range(index, &range), None);
            }
        }
    }
}
#[test]
fn negative_word_string_classes_keep_original_line_fold_backspace_and_pair_context() {
    for (source, i, m, text, start, sticky, expected) in [
        (
            r"(?!(^\b[\q{ab|a|}]\b$))",
            true,
            true,
            "\nA\n",
            1,
            true,
            None,
        ),
        (
            r"(?!(^\b[\q{ab|a|}]\b$))",
            true,
            true,
            "!A!",
            1,
            true,
            Some(1..1),
        ),
        (
            r"(?!(\b[\q{ſa|S|Kk|}]\B))",
            true,
            false,
            "SA!",
            0,
            false,
            Some(1..1),
        ),
        (
            r"(?!(\b[\q{ab|a|}]\b))",
            false,
            false,
            "\u{8}",
            0,
            true,
            Some(0..0),
        ),
        (
            r"(?!(\B[\q{😀a|😀}]\B))",
            false,
            false,
            "😀😀",
            0,
            false,
            Some(4..4),
        ),
        (
            r"(?!(\B[\q{😀a|😀}]\B))",
            false,
            false,
            "😀😀",
            1,
            true,
            None,
        ),
        (r"(?!(\b[\q{ab}]\b))", false, false, "", 1, false, None),
    ] {
        assert_eq!(
            word_compile(source, i, m)
                .find_with_work(&JsString::from(text), start, sticky, |_| Ok::<(), ()>(()))
                .unwrap(),
            expected,
            "{source}"
        );
    }
}
#[test]
fn negative_word_string_class_proof_requires_predicates_and_excludes_other_binding_roles() {
    for source in [
        r"(?!([\q{ab}]))",
        r"(?!(^[\q{ab}]$))",
        r"(?!(\b[\q{a|b}]))",
        r"(?!(\b[a-z]))",
        r"(?!(\b[\q{ab}&&a]))",
        r"(?!(\b[a\q{ab}]))",
        r"(?!(\b[\q{ab}]{1}))",
        r"(?!(\b[\q{ab}]a))",
        r"(?!(\b[\q{ab}]|a))",
        r"(?<!(\b[\q{ab}]))",
        r"(?=(\b[\q{ab}]))",
        r"(?!(?i:\b[\q{ab}]))",
        r"(?!(\b[\q{ab}])\1)",
        r"(?!(\b[\q{ab}])$)",
        r"(?!(\b[\p{RGI_Emoji}]))",
    ] {
        assert!(RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
            &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
        ).unwrap().is_none(), "{source}");
    }
}
#[test]
fn negative_word_string_classes_keep_deep_scopes_and_recover_from_every_work_abort() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?!(\b[\q{ab|a|}]\B))" + &")".repeat(depth);
    let plan = word_compile(&source, false, false);
    let found = plan
        .find_with_work(&JsString::from("ab!"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found, 1..1);
    assert_eq!(plan.capture_count(), depth + 1);
    assert_eq!(plan.capture_range(depth - 1, &found), Some(1..1));
    assert_eq!(plan.capture_range(depth, &found), None);
    let source = JsString::from(r"((?!(([\q{a|aaa|aa}]\B))))");
    let mut construction = Vec::new();
    let plan = RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
        &source, false, false, |n| { construction.push(n); Ok::<(), usize>(()) },
    ).unwrap().unwrap();
    for stop in 0..construction.len() {
        let mut seen = Vec::new();
        assert!(
            matches!(RegExpUnicodeNegativeStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
            &source, false, false, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from("aaaa!");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected, Some(3..3));
    for stop in 0..work.len() {
        let mut seen = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |n| {
                seen.push(n);
                if seen.len() == stop + 1 {
                    Err(stop)
                } else {
                    Ok(())
                }
            }),
            Err(stop)
        );
        assert_eq!(seen, work[..=stop]);
        assert_eq!(
            plan.clone()
                .find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    let input = JsString::from(("a".repeat(20_000) + "!").as_str());
    let mut total = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, false, |n| {
            total += n;
            Ok::<(), ()>(())
        })
        .unwrap(),
        Some(19_999..19_999)
    );
    assert!(total <= input.len() * 500 + 128, "{total}");
}

fn negative_union_compile(
    source: &str,
    i: bool,
    m: bool,
) -> RegExpUnicodeNegativeStringClassMatcher {
    RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_validated_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn negative_q_unions_keep_member_existence_empty_failure_and_undefined_operand_slots() {
    for (body, text, expected) in [
        (r"[\q{a|aa}\q{aaa|a}]", "aaa!", Some(3..3)),
        (r"[\q{ab}\q{abc}]", "abc!", Some(1..1)),
        (r"[\q{ab|a}\q{abc|}]", "!abc", None),
        (r"[\q{}\q{}]", "", None),
        (r"[\q{a|aa}\q{aaa|a}]$", "aaa", Some(3..3)),
        (r"[\q{a|aa}\q{aaa|a}]$", "aaa!", Some(0..0)),
    ] {
        let plan = negative_union_compile(&format!("((?!(({body}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(found, expected, "{body}");
        assert_eq!(plan.capture_count(), 3);
        if let Some(range) = found {
            assert_eq!(plan.capture_range(0, &range), Some(range.clone()));
            for index in 1..=3 {
                assert_eq!(plan.capture_range(index, &range), None);
            }
        }
    }
}

#[test]
fn negative_q_unions_keep_original_line_fold_and_complete_surrogate_context() {
    for (source, i, m, text, start, sticky, expected) in [
        (
            r"(?!(^[\q{a|aa}\q{aaa|a}]$))",
            true,
            true,
            "\nA\n",
            1,
            true,
            None,
        ),
        (
            r"(?!(^[\q{a|aa}\q{aaa|a}]$))",
            true,
            true,
            "!A!",
            1,
            true,
            Some(1..1),
        ),
        (
            r"(?!([\q{ſa|S}\q{Kk}]))",
            true,
            false,
            "sa!",
            0,
            false,
            Some(1..1),
        ),
        (
            r"(?!([\q{ſa|S}\q{Kk}]))",
            false,
            false,
            "sa!",
            0,
            false,
            Some(0..0),
        ),
        (
            r"(?!([\q{😀a}\q{😀}]))",
            false,
            false,
            "😀😀a!",
            0,
            false,
            Some(4..4),
        ),
        (
            r"(?!([\q{😀a}\q{😀}]))",
            false,
            false,
            "!😀a",
            2,
            true,
            None,
        ),
        (
            r"(?!([\q{\u{D83D}\u{DE00}}\q{ab}]))",
            false,
            false,
            "😀",
            0,
            true,
            Some(0..0),
        ),
    ] {
        assert_eq!(
            negative_union_compile(source, i, m)
                .find_with_work(&JsString::from(text), start, sticky, |_| Ok::<(), ()>(()))
                .unwrap(),
            expected,
            "{source}"
        );
    }
    let plan = negative_union_compile(r"(?!([\q{\uD800x}\q{\uD800}]))", false, false);
    assert_eq!(
        plan.find_with_work(
            &JsString::from_code_units(vec![0xd800, 0x78]),
            0,
            false,
            |_| Ok::<(), ()>(())
        )
        .unwrap(),
        Some(1..1)
    );
    assert_eq!(
        plan.find_with_work(&JsString::from("😀"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..0)
    );
    assert_eq!(
        plan.find_with_work(&JsString::from(""), 1, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
}

#[test]
fn negative_q_union_proof_keeps_other_domains_directions_scopes_and_binding_roles_separate() {
    for source in [
        r"(?!([\q{ab}]))",
        r"(?!([\q{a}\q{b}]))",
        r"(?!([a\q{ab}\q{}]))",
        r"(?!([\q{ab}&&\q{ab}]))",
        r"(?!([[\q{ab}]\q{}]))",
        r"(?!([\q{ab}\q{}]{1}))",
        r"(?!([\q{ab}\q{}]?))",
        r"(?!([\q{ab}\q{}]a))",
        r"(?!([\q{ab}\q{}]|a))",
        r"(?<!([\q{ab}\q{}]))",
        r"(?=([\q{ab}\q{}]))",
        r"(?!(\b[\q{ab}\q{}]))",
        r"(?!(?i:[\q{ab}\q{}]))",
        r"(?!([\q{ab}\q{}])\1)",
        r"(?!([\q{ab}\q{}])$)",
    ] {
        assert!(
            RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_validated_with_work(
                &JsString::from(source),
                false,
                false,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}

#[test]
fn negative_q_unions_keep_deep_scopes_bounded_retries_and_work_abort_recovery() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?!([\q{ab}\q{abc}]))" + &")".repeat(depth);
    let plan = negative_union_compile(&source, false, false);
    assert_eq!(plan.capture_count(), depth + 1);
    let found = plan
        .find_with_work(&JsString::from("a"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_range(depth - 1, &found), Some(0..0));
    assert_eq!(plan.capture_range(depth, &found), None);
    let source = JsString::from(r"((?!(([\q{a|aa}\q{aaa|a}]))))");
    let mut construction = Vec::new();
    let plan =
        RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_validated_with_work(
            &source,
            false,
            false,
            |n| {
                construction.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
    for stop in 0..construction.len() {
        let mut seen = Vec::new();
        assert!(
            matches!(RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_validated_with_work(
            &source, false, false, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from("aaaa!");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected, Some(4..4));
    for stop in 0..work.len() {
        let mut seen = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |n| {
                seen.push(n);
                if seen.len() == stop + 1 {
                    Err(stop)
                } else {
                    Ok(())
                }
            }),
            Err(stop)
        );
        assert_eq!(seen, work[..=stop]);
        assert_eq!(
            plan.clone()
                .find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    let input = JsString::from(("a".repeat(20_000) + "!").as_str());
    let mut total = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, false, |n| {
            total += n;
            Ok::<(), ()>(())
        })
        .unwrap(),
        Some(20_000..20_000)
    );
    assert!(total <= input.len() * 500 + 128, "{total}");
    let word = "a".repeat(100_000) + "b";
    let plan = negative_union_compile(&format!(r"(?!([\q{{{word}}}\q{{}}]))"), false, false);
    let input = JsString::from(("a".repeat(100_000) + "c").as_str());
    let mut calls = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, true, |_| {
            calls += 1;
            if calls == 17 { Err(()) } else { Ok(()) }
        }),
        Err(())
    );
    assert_eq!(calls, 17);
    assert_eq!(
        plan.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
}

fn negative_word_union_compile(
    source: &str,
    i: bool,
    m: bool,
) -> RegExpUnicodeNegativeStringClassMatcher {
    RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_with_word_boundaries_validated_with_work(
        &JsString::from(source), i, m, |_| Ok::<(), ()>(()),
    ).unwrap().unwrap()
}
#[test]
fn negative_word_q_union_classes_negate_the_whole_endpoint_conjunction_and_keep_undefined_slots() {
    for (body, text, expected) in [
        (r"\b[\q{ab|a}\q{abc|}]\b", "ab!", Some(1..1)),
        (r"\b[\q{ab|a}\q{abc|}]\B", "ab!", Some(1..1)),
        (r"\B[\q{ab|a}\q{abc|}]\b", "ab!", Some(0..0)),
        (r"\B[\q{ab|a}\q{abc|}]\B", "ab!", Some(0..0)),
        (r"[\q{}\q{}]\b", "aa!", Some(1..1)),
        (r"[\q{}\q{}]\b", "a", None),
        (r"\b[\q{ab|a}\q{abc|}]\b", "", Some(0..0)),
        (r"\B[\q{}\q{}]\B", "", None),
    ] {
        let plan = negative_word_union_compile(&format!("((?!(({body}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(found, expected, "{body}");
        assert_eq!(plan.capture_count(), 3);
        if let Some(range) = found {
            assert_eq!(plan.capture_range(0, &range), Some(range.clone()));
            for index in 1..=3 {
                assert_eq!(plan.capture_range(index, &range), None);
            }
        }
    }
}
#[test]
fn negative_word_q_union_classes_keep_original_line_fold_backspace_and_pair_context() {
    for (source, i, m, text, start, sticky, expected) in [
        (
            r"(?!(^\b[\q{ab|a}\q{abc|}]\b$))",
            true,
            true,
            "\nA\n",
            1,
            true,
            None,
        ),
        (
            r"(?!(^\b[\q{ab|a}\q{abc|}]\b$))",
            true,
            true,
            "!A!",
            1,
            true,
            Some(1..1),
        ),
        (
            r"(?!(\b[\q{ſa|S}\q{Kk|}]\B))",
            true,
            false,
            "sa!",
            0,
            false,
            Some(1..1),
        ),
        (
            r"(?!(\b[\q{ab|a}\q{abc|}]\b))",
            false,
            false,
            "\u{8}",
            0,
            true,
            Some(0..0),
        ),
        (
            r"(?!(\B[\q{😀a}\q{😀}]\B))",
            false,
            false,
            "😀😀",
            0,
            false,
            Some(4..4),
        ),
        (
            r"(?!(\B[\q{😀a}\q{😀}]\B))",
            false,
            false,
            "😀😀",
            1,
            true,
            None,
        ),
        (
            r"(?!(\b[\q{ab}\q{abc}]\b))",
            false,
            false,
            "",
            1,
            false,
            None,
        ),
    ] {
        assert_eq!(
            negative_word_union_compile(source, i, m)
                .find_with_work(&JsString::from(text), start, sticky, |_| Ok::<(), ()>(()))
                .unwrap(),
            expected,
            "{source}"
        );
    }
}
#[test]
fn negative_word_q_union_class_proof_requires_predicates_and_excludes_other_binding_roles() {
    for source in [
        r"(?!(\b[\q{ab}]))",
        r"(?!([\q{ab}\q{abc}]))",
        r"(?!(^[\q{ab}\q{abc}]$))",
        r"(?!(\b[\q{a}\q{b}]))",
        r"(?!(\b[a-z]))",
        r"(?!(\b[\q{ab}&&a]))",
        r"(?!(\b[a\q{ab}]))",
        r"(?!(\b[\q{ab}\q{abc}]{1}))",
        r"(?!(\b[\q{ab}\q{abc}]a))",
        r"(?!(\b[\q{ab}\q{abc}]|a))",
        r"(?<!(\b[\q{ab}\q{abc}]))",
        r"(?=(\b[\q{ab}\q{abc}]))",
        r"(?!(?i:\b[\q{ab}\q{abc}]))",
        r"(?!(\b[\q{ab}\q{abc}])\1)",
        r"(?!(\b[\q{ab}\q{abc}])$)",
        r"(?!(\b[\p{RGI_Emoji}]))",
    ] {
        assert!(RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_with_word_boundaries_validated_with_work(
            &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
        ).unwrap().is_none(), "{source}");
    }
}
#[test]
fn negative_word_q_union_classes_keep_deep_scopes_and_recover_from_every_work_abort() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?!(\b[\q{ab|a}\q{abc|}]\B))" + &")".repeat(depth);
    let plan = negative_word_union_compile(&source, false, false);
    let found = plan
        .find_with_work(&JsString::from("ab!"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found, 1..1);
    assert_eq!(plan.capture_count(), depth + 1);
    assert_eq!(plan.capture_range(depth - 1, &found), Some(1..1));
    assert_eq!(plan.capture_range(depth, &found), None);
    let source = JsString::from(r"((?!(([\q{a|aa}\q{aaa|a}]\B))))");
    let mut construction = Vec::new();
    let plan = RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_with_word_boundaries_validated_with_work(
        &source, false, false, |n| { construction.push(n); Ok::<(), usize>(()) },
    ).unwrap().unwrap();
    for stop in 0..construction.len() {
        let mut seen = Vec::new();
        assert!(
            matches!(RegExpUnicodeNegativeStringClassMatcher::compile_q_union_lookahead_with_word_boundaries_validated_with_work(
            &source, false, false, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from("aaaa!");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected, Some(3..3));
    for stop in 0..work.len() {
        let mut seen = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |n| {
                seen.push(n);
                if seen.len() == stop + 1 {
                    Err(stop)
                } else {
                    Ok(())
                }
            }),
            Err(stop)
        );
        assert_eq!(seen, work[..=stop]);
        assert_eq!(
            plan.clone()
                .find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    let input = JsString::from(("a".repeat(20_000) + "!").as_str());
    let mut total = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, false, |n| {
            total += n;
            Ok::<(), ()>(())
        })
        .unwrap(),
        Some(19_999..19_999)
    );
    assert!(total <= input.len() * 500 + 128, "{total}");
}

fn negative_character_union_compile(
    source: &str,
    i: bool,
    m: bool,
) -> RegExpUnicodeNegativeStringClassMatcher {
    RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_validated_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn negative_character_unions_keep_literal_members_empty_failure_and_undefined_operand_slots() {
    for (body, text, expected) in [
        (r"[x\q{ab}]", "x!", Some(1..1)),
        (r"[x\q{ab}]", "!ab", Some(0..0)),
        (r"[a\q{ab|abc}]", "aaa!", Some(3..3)),
        (r"[x\q{ab|}]", "!ab", None),
        (r"[x\q{}]", "", None),
        (r"[a\q{aaa|aa}]$", "aaa", Some(3..3)),
        (r"[a\q{aaa|aa}]$", "aaa!", Some(0..0)),
    ] {
        let plan = negative_character_union_compile(&format!("((?!(({body}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(found, expected, "{body}");
        assert_eq!(plan.capture_count(), 3);
        if let Some(range) = found {
            assert_eq!(plan.capture_range(0, &range), Some(range.clone()));
            for index in 1..=3 {
                assert_eq!(plan.capture_range(index, &range), None);
            }
        }
    }
}

#[test]
fn negative_character_unions_keep_original_line_fold_and_complete_surrogate_context() {
    for (source, i, m, text, start, sticky, expected) in [
        (r"(?!(^[a\q{ab|abc}]$))", true, true, "\nA\n", 1, true, None),
        (
            r"(?!(^[a\q{ab|abc}]$))",
            true,
            true,
            "!A!",
            1,
            true,
            Some(1..1),
        ),
        (
            r"(?!([ſ\q{Sa|Kk}]))",
            true,
            false,
            "sx",
            0,
            false,
            Some(1..1),
        ),
        (
            r"(?!([ſ\q{Sa|Kk}]))",
            false,
            false,
            "sx",
            0,
            false,
            Some(0..0),
        ),
        (
            r"(?!([😀\q{😀a}]))",
            false,
            false,
            "😀😀a!",
            0,
            false,
            Some(4..4),
        ),
        (r"(?!([😀\q{😀a}]))", false, false, "!😀a", 2, true, None),
        (
            r"(?!([x\q{\u{D83D}\u{DE00}}]))",
            false,
            false,
            "😀",
            0,
            true,
            Some(0..0),
        ),
    ] {
        assert_eq!(
            negative_character_union_compile(source, i, m)
                .find_with_work(&JsString::from(text), start, sticky, |_| Ok::<(), ()>(()))
                .unwrap(),
            expected,
            "{source}"
        );
    }
    let plan = negative_character_union_compile(r"(?!([\uD800\q{\uD800x}]))", false, false);
    assert_eq!(
        plan.find_with_work(
            &JsString::from_code_units(vec![0xd800, 0x78]),
            0,
            false,
            |_| Ok::<(), ()>(())
        )
        .unwrap(),
        Some(1..1)
    );
    assert_eq!(
        plan.find_with_work(&JsString::from("😀"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..0)
    );
    assert_eq!(
        plan.find_with_work(&JsString::from(""), 1, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
}

#[test]
fn negative_character_union_proof_keeps_other_domains_counts_scopes_and_bindings_separate() {
    for source in [
        r"(?!([abc]))",
        r"(?!([\q{ab}]))",
        r"(?!([\q{ab}\q{}]))",
        r"(?!([a\q{a}]))",
        r"(?!([\d\q{ab}]))",
        r"(?!([a-z\q{ab}]))",
        r"(?!([[a]\q{ab}]))",
        r"(?!([x&&\q{ab}]))",
        r"(?!([x\q{ab}]{1}))",
        r"(?!([x\q{ab}]?))",
        r"(?!([x\q{ab}]a))",
        r"(?!([x\q{ab}]|a))",
        r"(?<!([x\q{ab}]))",
        r"(?=([x\q{ab}]))",
        r"(?!(\b[x\q{ab}]))",
        r"(?!(?i:[x\q{ab}]))",
        r"(?!([x\q{ab}])\1)",
        r"(?!([x\q{ab}])$)",
    ] {
        assert!(RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_validated_with_work(
            &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
        ).unwrap().is_none(), "{source}");
    }
}

#[test]
fn negative_character_unions_keep_deep_scopes_bounded_retries_and_work_abort_recovery() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?!([x\q{ab}]))" + &")".repeat(depth);
    let plan = negative_character_union_compile(&source, false, false);
    assert_eq!(plan.capture_count(), depth + 1);
    let found = plan
        .find_with_work(&JsString::from("a"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_range(depth - 1, &found), Some(0..0));
    assert_eq!(plan.capture_range(depth, &found), None);
    let source = JsString::from(r"((?!(([a\q{aaa|aa}]))))");
    let mut construction = Vec::new();
    let plan = RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_validated_with_work(
        &source, false, false, |n| { construction.push(n); Ok::<(), usize>(()) },
    ).unwrap().unwrap();
    for stop in 0..construction.len() {
        let mut seen = Vec::new();
        assert!(
            matches!(RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_validated_with_work(
            &source, false, false, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from("aaaa!");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected, Some(4..4));
    for stop in 0..work.len() {
        let mut seen = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |n| {
                seen.push(n);
                if seen.len() == stop + 1 {
                    Err(stop)
                } else {
                    Ok(())
                }
            }),
            Err(stop)
        );
        assert_eq!(seen, work[..=stop]);
        assert_eq!(
            plan.clone()
                .find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    let input = JsString::from(("a".repeat(20_000) + "!").as_str());
    let mut total = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, false, |n| {
            total += n;
            Ok::<(), ()>(())
        })
        .unwrap(),
        Some(20_000..20_000)
    );
    assert!(total <= input.len() * 500 + 128, "{total}");
    let word = "a".repeat(100_000) + "b";
    let plan = negative_character_union_compile(&format!(r"(?!([x\q{{{word}|}}]))"), false, false);
    let input = JsString::from(("a".repeat(100_000) + "c").as_str());
    let mut calls = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, true, |_| {
            calls += 1;
            if calls == 17 { Err(()) } else { Ok(()) }
        }),
        Err(())
    );
    assert_eq!(calls, 17);
    assert_eq!(
        plan.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
}

fn negative_word_character_union_compile(
    source: &str,
    i: bool,
    m: bool,
) -> RegExpUnicodeNegativeStringClassMatcher {
    RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_with_word_boundaries_validated_with_work(
        &JsString::from(source), i, m, |_| Ok::<(), ()>(()),
    ).unwrap().unwrap()
}
#[test]
fn negative_word_character_union_classes_negate_the_whole_endpoint_conjunction_and_keep_undefined_slots()
 {
    for (body, text, expected) in [
        (r"\b[a\q{ab|abc|}]\b", "ab!", Some(1..1)),
        (r"\b[a\q{ab|abc|}]\B", "ab!", Some(1..1)),
        (r"\B[a\q{ab|abc|}]\b", "ab!", Some(0..0)),
        (r"\B[a\q{ab|abc|}]\B", "ab!", Some(0..0)),
        (r"[x\q{}]\b", "aa!", Some(1..1)),
        (r"[x\q{}]\b", "a", None),
        (r"\b[a\q{ab|abc|}]\b", "", Some(0..0)),
        (r"\B[x\q{}]\B", "", None),
    ] {
        let plan =
            negative_word_character_union_compile(&format!("((?!(({body}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap();
        assert_eq!(found, expected, "{body}");
        assert_eq!(plan.capture_count(), 3);
        if let Some(range) = found {
            assert_eq!(plan.capture_range(0, &range), Some(range.clone()));
            for index in 1..=3 {
                assert_eq!(plan.capture_range(index, &range), None);
            }
        }
    }
}
#[test]
fn negative_word_character_union_classes_keep_original_line_fold_backspace_and_pair_context() {
    for (source, i, m, text, start, sticky, expected) in [
        (
            r"(?!(^\b[a\q{ab|abc|}]\b$))",
            true,
            true,
            "\nA\n",
            1,
            true,
            None,
        ),
        (
            r"(?!(^\b[a\q{ab|abc|}]\b$))",
            true,
            true,
            "!A!",
            1,
            true,
            Some(1..1),
        ),
        (
            r"(?!(\b[ſ\q{Sa|Kk|}]\B))",
            true,
            false,
            "sa!",
            0,
            false,
            Some(1..1),
        ),
        (
            r"(?!(\b[a\q{ab|abc|}]\b))",
            false,
            false,
            "\u{8}",
            0,
            true,
            Some(0..0),
        ),
        (
            r"(?!(\B[😀\q{😀a}]\B))",
            false,
            false,
            "😀😀",
            0,
            false,
            Some(4..4),
        ),
        (
            r"(?!(\B[😀\q{😀a}]\B))",
            false,
            false,
            "😀😀",
            1,
            true,
            None,
        ),
        (r"(?!(\b[x\q{ab}]\b))", false, false, "", 1, false, None),
    ] {
        assert_eq!(
            negative_word_character_union_compile(source, i, m)
                .find_with_work(&JsString::from(text), start, sticky, |_| Ok::<(), ()>(()))
                .unwrap(),
            expected,
            "{source}"
        );
    }
}
#[test]
fn negative_word_character_union_class_proof_requires_predicates_and_excludes_other_binding_roles()
{
    for source in [
        r"(?!(\b[\q{ab}]))",
        r"(?!([x\q{ab}]))",
        r"(?!(^[x\q{ab}]$))",
        r"(?!(\b[\q{a}\q{b}]))",
        r"(?!(\b[a-z]))",
        r"(?!(\b[\q{ab}&&a]))",
        r"(?!(\b[\d\q{ab}]))",
        r"(?!(\b[x\q{ab}]{1}))",
        r"(?!(\b[x\q{ab}]a))",
        r"(?!(\b[x\q{ab}]|a))",
        r"(?<!(\b[x\q{ab}]))",
        r"(?=(\b[x\q{ab}]))",
        r"(?!(?i:\b[x\q{ab}]))",
        r"(?!(\b[x\q{ab}])\1)",
        r"(?!(\b[x\q{ab}])$)",
        r"(?!(\b[\p{RGI_Emoji}]))",
    ] {
        assert!(RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_with_word_boundaries_validated_with_work(
            &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
        ).unwrap().is_none(), "{source}");
    }
}
#[test]
fn negative_word_character_union_classes_keep_deep_scopes_and_recover_from_every_work_abort() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?!(\b[a\q{ab|abc|}]\B))" + &")".repeat(depth);
    let plan = negative_word_character_union_compile(&source, false, false);
    let found = plan
        .find_with_work(&JsString::from("ab!"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found, 1..1);
    assert_eq!(plan.capture_count(), depth + 1);
    assert_eq!(plan.capture_range(depth - 1, &found), Some(1..1));
    assert_eq!(plan.capture_range(depth, &found), None);
    let source = JsString::from(r"((?!(([a\q{aaa|aa}]\B))))");
    let mut construction = Vec::new();
    let plan = RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_with_word_boundaries_validated_with_work(
        &source, false, false, |n| { construction.push(n); Ok::<(), usize>(()) },
    ).unwrap().unwrap();
    for stop in 0..construction.len() {
        let mut seen = Vec::new();
        assert!(
            matches!(RegExpUnicodeNegativeStringClassMatcher::compile_character_union_lookahead_with_word_boundaries_validated_with_work(
            &source, false, false, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from("aaaa!");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected, Some(3..3));
    for stop in 0..work.len() {
        let mut seen = Vec::new();
        assert_eq!(
            plan.find_with_work(&input, 0, false, |n| {
                seen.push(n);
                if seen.len() == stop + 1 {
                    Err(stop)
                } else {
                    Ok(())
                }
            }),
            Err(stop)
        );
        assert_eq!(seen, work[..=stop]);
        assert_eq!(
            plan.clone()
                .find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    let input = JsString::from(("a".repeat(20_000) + "!").as_str());
    let mut total = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, false, |n| {
            total += n;
            Ok::<(), ()>(())
        })
        .unwrap(),
        Some(19_999..19_999)
    );
    assert!(total <= input.len() * 500 + 128, "{total}");
}
