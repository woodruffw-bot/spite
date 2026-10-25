use super::*;

fn compile(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveStringClassMatcher {
    RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_validated_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap()
}

#[test]
fn positive_string_classes_keep_longest_members_earliest_empty_choices_and_capture_scopes() {
    for (class, text, root, operand) in [
        (r"[\q{ab|a|}]", "ab!", 0..0, 0..2),
        (r"[\q{|ab}]", "!ab", 0..0, 0..0),
        (r"[\q{a|ab}]", "!ab", 1..1, 1..3),
        (r"[\q{😀a|😀|}]", "😀a!", 0..0, 0..3),
        (r"[\q{}]", "", 0..0, 0..0),
    ] {
        let plan = compile(&format!("((?=(({class}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, root);
        assert_eq!(found.operand, operand);
        assert_eq!(plan.capture_count(), 3);
        assert_eq!(plan.capture_range(0, &found), Some(root));
        assert_eq!(plan.capture_range(1, &found), Some(operand.clone()));
        assert_eq!(plan.capture_range(2, &found), Some(operand));
        assert_eq!(plan.capture_range(3, &found), None);
    }
}

#[test]
fn positive_string_classes_keep_original_context_folding_and_complete_unicode_candidates() {
    for (source, i, m, text, start, root, operand) in [
        (r"(?=(^[\q{ab|a}]$))", true, true, "\nA\n", 0, 1..1, 1..2),
        (r"(?=([\q{a\nb|a}]$))", false, true, "a\nbc", 0, 0..0, 0..1),
        (r"(?=([\q{ſa|S|}]))", true, false, "SA!", 0, 0..0, 0..2),
        (r"(?=([\q{😀a|😀|}]))", false, false, "!😀a", 2, 1..1, 1..4),
        (
            r"(?=([\q{\u{D83D}\u{DE00}|}]))",
            false,
            false,
            "😀",
            0,
            0..0,
            0..0,
        ),
    ] {
        let plan = compile(source, i, m);
        let found = plan
            .find_with_work(&JsString::from(text), start, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!((found.range, found.operand), (root, operand), "{source}");
    }
    let plan = compile(r"(?=([\q{\uD800x|\uD800|}]))", false, false);
    for (input, expected) in [
        (JsString::from_code_units(vec![0xd800, 0x78]), 0..2),
        (JsString::from("😀"), 0..0),
    ] {
        let found = plan
            .find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.operand, expected);
    }
    assert_eq!(
        plan.find_with_work(&JsString::from(""), 1, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        None
    );
}

#[test]
fn positive_string_class_proof_declines_other_grammars_and_binding_roles() {
    for source in [
        r"(?=([\q{a|b}]))",
        r"(?=([a-z]))",
        r"(?=([\q{ab}&&a]))",
        r"(?=([a\q{ab}]))",
        r"(?=([\q{ab}]?))",
        r"(?=([\q{ab}]{1}))",
        r"(?=([\q{ab}]a))",
        r"(?=([\q{ab}]|a))",
        r"(?<=([\q{ab}]))",
        r"(?!([\q{ab}]))",
        r"(?=(\b[\q{ab}]))",
        r"(?=([\q{ab}]\B))",
        r"(?=(?i:[\q{ab}]))",
        r"(?=([\q{ab}])\1)",
        r"(?=([\q{ab}])$)",
        r"(?=([\p{RGI_Emoji}]))",
        r"(?=([[\q{ab}]]))",
    ] {
        assert!(RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_validated_with_work(
            &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
        ).unwrap().is_none(), "{source}");
    }
}

#[test]
fn positive_string_classes_keep_deep_scopes_linear_failed_endpoints_and_cancellation_recovery() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?=([\q{ab|a|}]))" + &")".repeat(depth);
    let plan = compile(&source, false, false);
    assert_eq!(plan.capture_count(), depth + 1);
    let input = JsString::from("ab");
    let found = plan
        .find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_range(depth - 1, &found), Some(0..0));
    assert_eq!(plan.capture_range(depth, &found), Some(0..2));
    let source = JsString::from(r"((?=(([\q{aa|a}]$))))");
    let mut construction = Vec::new();
    let plan =
        RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_validated_with_work(
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
            matches!(RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_validated_with_work(
            &source, false, false, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from(("a".repeat(20_000) + "!").as_str());
    let mut total = 0;
    assert_eq!(
        plan.find_with_work(&input, 0, false, |n| {
            total += n;
            Ok::<(), ()>(())
        })
        .unwrap(),
        None
    );
    assert!(total <= input.len() * 250 + 128, "{total}");
    let input = JsString::from("!!aaa");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected.as_ref().map(|m| m.operand.clone()), Some(3..5));
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
}

fn word_compile(source: &str, i: bool, m: bool) -> RegExpUnicodePositiveStringClassMatcher {
    RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
        &JsString::from(source), i, m, |_| Ok::<(), ()>(()),
    ).unwrap().unwrap()
}

#[test]
fn word_string_classes_keep_endpoint_retries_empty_members_and_independent_captures() {
    for (body, text, root, operand) in [
        (r"\b[\q{ab|a|}]\b", "ac!", 0..0, 0..0),
        (r"\b[\q{ab|a}]\b", "ab!", 0..0, 0..2),
        (r"\b[\q{ab|a|}]\B", "ab!", 0..0, 0..1),
        (r"\B[\q{ab|a|}]\b", "xab!", 1..1, 1..3),
        (r"\B[\q{ab|a|}]\B", "ab!", 1..1, 1..1),
        (r"[\q{}]\b", "a", 0..0, 0..0),
    ] {
        let plan = word_compile(&format!("((?=(({body}))))"), false, false);
        let found = plan
            .find_with_work(&JsString::from(text), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, root, "{body}");
        assert_eq!(found.operand, operand, "{body}");
        assert_eq!(plan.capture_count(), 3);
        assert_eq!(plan.capture_range(0, &found), Some(root));
        assert_eq!(plan.capture_range(1, &found), Some(operand.clone()));
        assert_eq!(plan.capture_range(2, &found), Some(operand));
        assert_eq!(plan.capture_range(3, &found), None);
    }
}

#[test]
fn word_string_classes_keep_line_fold_backspace_and_surrogate_context() {
    for (source, i, m, text, start, root, operand) in [
        (
            r"(?=(^\b[\q{ab|a|}]\b$))",
            true,
            true,
            "\nA\n",
            0,
            1..1,
            1..2,
        ),
        (r"(?=(\b[\q{ſa|S|}]\B))", true, false, "SA!", 0, 0..0, 0..1),
        (
            r"(?=(\B[\q{ab|a|}]\B))",
            false,
            false,
            "\u{8}",
            0,
            0..0,
            0..0,
        ),
        (
            r"(?=(\B[\q{😀a|😀|}]\b))",
            false,
            false,
            "!😀ab",
            2,
            1..1,
            1..3,
        ),
    ] {
        let plan = word_compile(source, i, m);
        let found = plan
            .find_with_work(&JsString::from(text), start, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!((found.range, found.operand), (root, operand), "{source}");
    }
    let plan = word_compile(r"(?=(\B[\q{\uD800x|\uD800|}]\B))", false, false);
    let found = plan
        .find_with_work(&JsString::from("😀a"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..0);
    assert_eq!(found.operand, 0..0);
}

#[test]
fn word_string_class_proof_requires_predicates_and_declines_other_grammar_and_binding_roles() {
    for source in [
        r"(?=([\q{ab}]))",
        r"(?=(^[\q{ab}]$))",
        r"(?=(\b[\q{a|b}]\b))",
        r"(?=(\b[a-z]\b))",
        r"(?=(\b[\q{ab}&&a]))",
        r"(?=(\b[a\q{ab}]))",
        r"(?=(\b[\q{ab}]{1}))",
        r"(?=(\b[\q{ab}]a))",
        r"(?=(\b[\q{ab}]|a))",
        r"(?<=(\b[\q{ab}]))",
        r"(?!(\b[\q{ab}]))",
        r"(?=(?i:\b[\q{ab}]))",
        r"(?=(\b[\q{ab}])\1)",
        r"(?=(\b[\q{ab}])$)",
        r"(?=(\b[\p{RGI_Emoji}]))",
    ] {
        assert!(RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
            &JsString::from(source), false, false, |_| Ok::<(), ()>(()),
        ).unwrap().is_none(), "{source}");
    }
}

#[test]
fn word_string_classes_keep_deep_scopes_and_recover_from_every_work_abort() {
    let depth = 100_000;
    let source = "(".repeat(depth) + r"(?=(\b[\q{ab|a|}]\B))" + &")".repeat(depth);
    let plan = word_compile(&source, false, false);
    assert_eq!(plan.capture_count(), depth + 1);
    let found = plan
        .find_with_work(&JsString::from("ab!"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(plan.capture_range(depth - 1, &found), Some(0..0));
    assert_eq!(plan.capture_range(depth, &found), Some(0..1));
    let source = JsString::from(r"((?=((^\b[\q{aa|a|}]\b$))))");
    let mut construction = Vec::new();
    let plan = RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
        &source, true, true, |n| { construction.push(n); Ok::<(), usize>(()) },
    ).unwrap().unwrap();
    for stop in 0..construction.len() {
        let mut seen = Vec::new();
        assert!(
            matches!(RegExpUnicodePositiveStringClassMatcher::compile_single_q_lookahead_with_word_boundaries_validated_with_work(
            &source, true, true, |n| { seen.push(n); if seen.len() == stop + 1 { Err(stop) } else { Ok(()) } },
        ), Err(RegExpUnicodeSetCompileError::Work(n)) if n == stop)
        );
        assert_eq!(seen, construction[..=stop]);
    }
    let input = JsString::from("!!aa\nAa\n");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected.as_ref().map(|m| m.operand.clone()), Some(5..7));
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
        None
    );
    assert!(total <= input.len() * 250 + 128, "{total}");
}
