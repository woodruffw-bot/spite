use super::*;

#[test]
fn metered_string_sets_preserve_longest_members_empty_choices_original_context_and_pairs() {
    for (source, i, m, text, start, sticky, expected) in [
        (
            r"[\q{a|aaa|aa}]",
            false,
            false,
            "aaa!",
            0,
            false,
            Some(0..3),
        ),
        (r"[\q{a|ab}]", false, false, "!ab", 0, false, Some(1..3)),
        (
            r"\b[\q{ab|a|}]\b",
            false,
            false,
            "ac!",
            0,
            false,
            Some(0..0),
        ),
        (r"^[\q{ab|a|}]$", true, true, "\nA\n", 0, false, Some(0..0)),
        (
            r"^\b[\q{ab|a|}]\b$",
            true,
            true,
            "\nA\n",
            0,
            false,
            Some(1..2),
        ),
        (r"[\q{😀a|😀|}]", false, false, "!😀a", 2, true, Some(1..4)),
        (r"[\q{}]", false, false, "a😀", 2, true, Some(1..1)),
        (r"\B[\q{ab|}]\B", false, false, "ab!", 0, false, Some(1..1)),
        (r"[\q{ſa|S|}]", true, false, "SA!", 0, true, Some(0..2)),
        (r"[\q{ab}]", false, false, "z", 0, false, None),
        (r"[\q{ab}]", false, false, "ab", 3, false, None),
    ] {
        let plan =
            RegExpUnicodeStringSetMatcher::compile_with_work(&JsString::from(source), i, m, |_| {
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        let input = JsString::from(text);
        let mut used = 0;
        let actual = plan
            .find_with_work(&input, start, sticky, |n| {
                assert!(n > 0);
                used += n;
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(actual, expected, "{source} {text} start={start}");
        assert!(used > 0);
    }
    let plan = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
        &JsString::from(r"[\p{ASCII}\q{😀a|}]"),
        true,
        false,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap();
    assert_eq!(
        plan.find_with_work(&JsString::from("ſK"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..1)
    );
}

#[test]
fn metered_string_sets_cancel_every_step_recover_and_interrupt_long_nonterminal_prefixes() {
    let plan = RegExpUnicodeStringSetMatcher::compile_with_work(
        &JsString::from(r"^\b[\q{a|aa|aaa|}]\b$"),
        true,
        true,
        |_| Ok::<(), usize>(()),
    )
    .unwrap()
    .unwrap();
    let input = JsString::from("!!aa\nAa\n");
    let mut work = Vec::new();
    let expected = plan
        .find_with_work(&input, 0, false, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap();
    assert_eq!(expected, Some(5..7));
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
            plan.find_with_work(&input, 0, false, |_| Ok::<(), usize>(()))
                .unwrap(),
            expected
        );
    }
    let word = "a".repeat(100_000) + "b";
    let source = JsString::from(format!(r"[\q{{{word}|}}]").as_str());
    let plan = RegExpUnicodeStringSetMatcher::compile_with_work(&source, false, false, |_| {
        Ok::<(), ()>(())
    })
    .unwrap()
    .unwrap();
    let input = JsString::from(("a".repeat(100_000) + "c").as_str());
    let mut calls = 0;
    assert_eq!(
        plan.clone().find_with_work(&input, 0, true, |_| {
            calls += 1;
            if calls == 17 { Err(()) } else { Ok(()) }
        }),
        Err(())
    );
    assert_eq!(calls, 17);
    assert_eq!(
        plan.find_with_work(&input, 0, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..0)
    );
}
