use super::*;

fn compile_scalar_rhs(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn scalar_operands_filter_independent_single_character_members_without_expanding_words() {
    type Predicate = fn(u32) -> bool;
    let operands: [(&str, Predicate); 12] = [
        (r"[a-z]", |p| (97..=122).contains(&p)),
        (r"\d", |p| (48..=57).contains(&p)),
        (r"\D", |p| !(48..=57).contains(&p)),
        (r"\w", |p| {
            (48..=57).contains(&p) || (65..=90).contains(&p) || (97..=122).contains(&p) || p == 95
        }),
        (r"\s", |p| matches!(p, 32 | 0xfeff)),
        (r"\p{White_Space}", |p| matches!(p, 32 | 0x85)),
        (r"\p{ASCII}", |p| p <= 127),
        (r"\P{ASCII}", |p| p > 127),
        (r"\p{Any}", |_| true),
        (r"[[a-z]--[aeiou]]", |p| {
            (97..=122).contains(&p) && ![97, 101, 105, 111, 117].contains(&p)
        }),
        (r"[^a-z]", |p| !(97..=122).contains(&p)),
        (r"[\u{0}-\u{10FFFF}]", |_| true),
    ];
    let members: &[&[u32]] = &[
        &[],
        &[97],
        &[97, 98],
        &[97, 98, 99],
        &[75],
        &[115, 75],
        &[53],
        &[32],
        &[0xfeff],
        &[0x85],
        &[0x1f600],
        &[0xd800],
        &[0xd800, 97],
        &[0xd800, 0xdc00],
    ];
    for (rhs, contains) in operands {
        for i in [false, true] {
            for m in [false, true] {
                for subtract in [false, true] {
                    let source = format!(
                        r"[\q{{|a|ab|abc|K|sK|5| |\uFEFF|\u0085|😀|\uD800|\uD800a|\u{{D800}}\u{{DC00}}}}{}{}]",
                        if subtract { "--" } else { "&&" },
                        rhs
                    );
                    let words = members
                        .iter()
                        .copied()
                        .filter(|word| {
                            let present = word.len() == 1
                                && contains(if i { fold(word[0]) } else { word[0] });
                            present != subtract
                        })
                        .collect::<Vec<_>>();
                    let plan = compile_scalar_rhs(&source, i, m).unwrap();
                    assert!(
                        RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
                            &JsString::from(source.as_str()),
                            i,
                            m,
                            |_| Ok::<(), ()>(()),
                        )
                        .unwrap()
                        .is_none()
                    );
                    for input in [
                        JsString::from(""),
                        JsString::from("xabcd!abc!"),
                        JsString::from("ſK5!"),
                        JsString::from("A😀ab"),
                        JsString::from("!\r\na\u{2028}!"),
                        JsString::from(" \u{feff}\u{85}"),
                        JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                    ] {
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                assert_eq!(
                                    plan.find(&input, start, sticky),
                                    oracle(
                                        &input,
                                        start,
                                        sticky,
                                        &words,
                                        Gates {
                                            i,
                                            m,
                                            ..Gates::default()
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

#[test]
fn scalar_rhs_chains_keep_empty_words_distinct_and_require_every_complete_operand() {
    for (source, words) in [
        (r"[[a\q{ab|abc}]--[a-z]--[\q{abc}]]", vec![&[97, 98][..]]),
        (r"[\q{a|b|ab}&&[a-z]&&\q{b|ab}]", vec![&[98][..]]),
        (r"[\q{a|ab|}--[a-z]--[]]", vec![&[97, 98][..], &[][..]]),
        (r"[\q{a|ab|}&&\p{Any}&&[^a]]", vec![]),
        (r"[[]&&\p{Any}&&\q{ab}]", vec![]),
    ] {
        let plan = compile_scalar_rhs(source, false, false).unwrap();
        for input in [
            JsString::from(""),
            JsString::from("xab!"),
            JsString::from("A😀ab"),
        ] {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        plan.find(&input, start, sticky),
                        oracle(&input, start, sticky, &words, Gates::default()),
                        "{source}"
                    );
                }
            }
        }
    }
    for source in [
        r"[\q{ab}&&\q{ab}]",
        r"[[\q{ab}]&&[\q{ab}]]",
        r"[a&&[a-z]]",
        r"[[a-z]&&\q{ab}]",
        r"[\q{a|ab}&&[a-z\q{ab}]]",
        r"[[\q{ab}&&\q{ab}]--[a-z]]",
        r"[\q{ab}&&[\p{RGI_Emoji}]]",
        r"[[]&&\p{Any}&&\p{RGI_Emoji}&&\q{ab}]",
        r"[\q{ab}--\d&&[a-z]]",
        r"[\q{ab}--\d]+",
        r"[\q{ab}--\d][a]",
        r"\b^[\q{ab}--\d]",
    ] {
        assert!(compile_scalar_rhs(source, true, true).is_none(), "{source}");
    }
    let empty = compile_scalar_rhs(r"[\q{ab|}&&\p{Any}]", false, false).unwrap();
    assert_eq!(empty.max_match_code_units(), 0);
    assert_eq!(empty.find(&JsString::from("😀"), 1, true), None);
    let zero = compile_scalar_rhs(r"[\q{}--\p{Any}]", false, false).unwrap();
    assert_eq!(zero.max_match_code_units(), 0);
    assert_eq!(zero.find(&JsString::from("😀"), 1, true), Some(0..0));
    assert_eq!(
        compile_scalar_rhs(r"[\q{a|ab}&&\p{Any}]", false, false)
            .unwrap()
            .max_match_code_units(),
        2
    );
}

#[test]
fn scalar_rhs_construction_checks_every_callback_keeps_full_ranges_compact_and_nesting_flat() {
    for source in [
        r"[[K\q{sK|a|ab|}]--\P{ASCII}--[a-z]]",
        r"[[]&&\p{Any}&&\q{ab}]",
        r"[\q{a|ab}&&[[a-z]--[aeiou]]]",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
            &source,
            true,
            true,
            |work| {
                complete.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 4);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
                    &source,
                    true,
                    true,
                    |work| {
                        seen.push(work);
                        if seen.len() == fail {
                            Err(fail)
                        } else {
                            Ok(())
                        }
                    },
                );
            assert!(matches!(result,Err(RegExpUnicodeSetCompileError::Work(at)) if at==fail));
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
            &source,
            true,
            true,
            |work| {
                recovered.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    let source = JsString::from(
        format!(
            r"[\q{{a|ab}}&&{}a-z{}]",
            "[".repeat(20_000),
            "]".repeat(20_000)
        )
        .as_str(),
    );
    let mut work = 0usize;
    let original = RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
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
        work < source.len() * 32 + 512,
        "checked nesting must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.find(&JsString::from("xab"), 0, false), Some(1..2));
    let mut work = 0usize;
    let plan = RegExpUnicodeStringSetMatcher::compile_scalar_rhs_string_operations_with_work(
        &JsString::from(r"[\q{a|ab|😀}&&[\u{0}-\u{10FFFF}]]"),
        false,
        false,
        |n| {
            work += n;
            Ok::<(), ()>(())
        },
    )
    .unwrap()
    .unwrap();
    assert!(work < 4096, "full ranges must remain compact: {work}");
    assert_eq!(plan.find(&JsString::from("😀ab"), 0, false), Some(0..2));
}
