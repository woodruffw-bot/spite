use super::*;

fn compile_result(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeRepeatedAtomMatcher> {
    RegExpUnicodeRepeatedAtomMatcher::compile_character_result_v_set_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn character_result_repetition_matches_independent_counts_assertions_and_utf16_endpoints() {
    type Predicate = fn(u32) -> bool;
    let cases: [(&str, Predicate); 7] = [
        (r"[\q{a|ab|}&&\p{Any}]", |p| p == 97),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[[\q{ab}&&a]b]", |p| p == 98),
        (r"[[^\q{ab}&&a]&&\q{b|bc|}]", |p| p == 98),
        (r"[\q{a|ab}&&[[a-z]&&\q{a|ab}]]", |p| p == 97),
        (r"[[^\q{ab}&&a]--\q{a|ab}]", |p| p != 97),
        (r"[\q{ab}&&a]", |_| false),
    ];
    for (class, contains) in cases {
        for (quantifier, minimum, maximum, greedy) in [
            ("*", 0, None, true),
            ("*?", 0, None, false),
            ("+", 1, None, true),
            ("+?", 1, None, false),
            ("{0}", 0, Some(0), true),
            ("{1,3}?", 1, Some(3), false),
        ] {
            for (prefix, suffix, anchors, words) in [
                ("", "", false, [None, None]),
                (r"^\b", r"\b$", true, [Some(true), Some(true)]),
            ] {
                for i in [false, true] {
                    for m in [false, true] {
                        let source = format!("{prefix}{class}{quantifier}{suffix}");
                        let plan = compile_result(&source, i, m).unwrap();
                        assert_eq!(plan.capture_count(), 0);
                        for input in [
                            JsString::from(""),
                            JsString::from("aaab!aba"),
                            JsString::from("A😀😀aa"),
                            JsString::from("!\r\naaa\u{2028}!"),
                            JsString::from("ſK"),
                            JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                        ] {
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    let found = plan
                                        .find_with_captures_and_work(&input, start, sticky, |_| {
                                            Ok::<(), ()>(())
                                        })
                                        .unwrap();
                                    assert!(
                                        found.as_ref().is_none_or(|m| m.atom_capture.is_none())
                                    );
                                    assert_eq!(
                                        found.map(|m| m.range),
                                        oracle(
                                            &input,
                                            start,
                                            sticky,
                                            contains,
                                            Bounds {
                                                minimum,
                                                maximum,
                                                greedy
                                            },
                                            Gates {
                                                anchors,
                                                words,
                                                i,
                                                m
                                            }
                                        ),
                                        "{source} i={i} m={m} {input:?} start={start} sticky={sticky}"
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
fn character_result_repetition_keeps_conservative_domains_and_exact_zero_and_huge_bounds() {
    for source in [
        r"[\q{ab}&&a]",
        r"[\q{ab}&&\q{cd}]{0}",
        r"[\q{ab}--\q{ab}]*",
        r"[\q{}--\q{}]+",
        r"[\q{a|ab}&&[a-z\q{ab}]]+",
        r"[\q{ab}--\p{Any}]+",
        r"([\q{ab}&&a])+",
        r"[\q{ab}&&a]+[b]",
        r"\b^[\q{ab}&&a]+",
        r"[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]{0}",
    ] {
        assert!(compile_result(source, true, true).is_none(), "{source}");
    }
    let huge = "9".repeat(100);
    for (source, expected) in [
        (r"[\q{ab}&&a]*".to_owned(), Some(0..0)),
        (r"[^\q{ab}&&a]{0}".to_owned(), Some(0..0)),
        (format!(r"[\q{{a|ab}}&&\p{{Any}}]{{{huge}}}"), None),
        (format!(r"[\q{{ab}}&&a]{{0,{huge}}}"), Some(0..0)),
    ] {
        let plan = compile_result(&source, false, false).unwrap();
        assert_eq!(
            plan.find_with_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap(),
            expected,
            "{source}"
        );
    }
}

#[test]
fn quantified_character_projection_checks_callbacks_and_flat_deep_and_long_operands() {
    for source in [
        r"^\b[\q{a|ab}&&\p{Any}]{1,3}?\b$",
        r"[^\q{ab}&&a]*",
        r"[\q{ab}&&a]{0}",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        RegExpUnicodeRepeatedAtomMatcher::compile_character_result_v_set_with_work(
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
        assert_eq!(complete[0], source.len());
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeRepeatedAtomMatcher::compile_character_result_v_set_with_work(
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
        RegExpUnicodeRepeatedAtomMatcher::compile_character_result_v_set_with_work(
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
        format!(r"{}\q{{ab}}&&a{}*", "[".repeat(20_000), "]".repeat(20_000)).as_str(),
    );
    let mut work = 0usize;
    let original = RegExpUnicodeRepeatedAtomMatcher::compile_character_result_v_set_with_work(
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
        work < source.len() * 48 + 512,
        "deep projection work must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(
        plan.find_with_work(&JsString::from("ab"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..0)
    );
    let source =
        JsString::from(format!(r"[\q{{{}}}&&\p{{Any}}]{{0}}", "x".repeat(20_000)).as_str());
    let mut work = 0usize;
    let plan = RegExpUnicodeRepeatedAtomMatcher::compile_character_result_v_set_with_work(
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
        work < source.len() * 24 + 512,
        "ignored words must not expand: {work}"
    );
    assert_eq!(
        plan.find_with_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap(),
        Some(0..0)
    );
}
