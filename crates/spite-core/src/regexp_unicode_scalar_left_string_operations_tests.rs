use super::*;

fn compile_scalar_left(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassMatcher> {
    RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn scalar_left_results_follow_independent_character_projections_of_nested_word_sets() {
    type Predicate = fn(u32) -> bool;
    let cases: [(&str, Predicate); 12] = [
        (r"[\p{Any}&&\q{a|ab|}]", |p| p == 97),
        (r"[\p{Any}--\q{a|ab|}]", |p| p != 97),
        (r"[[a-z]--[\q{b|bc|}]]", |p| {
            (97..=122).contains(&p) && p != 98
        }),
        (r"[\p{ASCII}&&[a-z\q{ab}]]", |p| (97..=122).contains(&p)),
        (r"[[a-z]&&[\q{ab|a}--\q{ab}]]", |p| p == 97),
        (r"[[^]--[[\q{a|ab}]&&[\q{b|ab}]]]", |_| true),
        (r"[\p{Any}--[^\q{ab}&&\p{Any}]]", |_| false),
        (r"[\p{Any}&&[\q{ab}--\q{ab}]]", |_| false),
        (r"[\p{Any}--[\q{}--\q{ab}]]", |_| true),
        (r"[[]--[\q{|ab}]]", |_| false),
        (r"[\p{Any}&&[\q{a|ab}\d]&&[^5]]", |p| {
            p == 97 || (48..=57).contains(&p) && p != 53
        }),
        (r"[\p{Any}&&[\q{\u{D800}\u{DC00}|\uD800a|😀}]]", |p| {
            p == 0x1f600
        }),
    ];
    let points = [
        0, 8, 10, 13, 32, 33, 48, 53, 65, 75, 97, 98, 99, 100, 107, 115, 120, 0xdf, 0x17f, 0x85,
        0x1e9e, 0x2028, 0x212a, 0xd800, 0xdc00, 0x10000, 0x1f600,
    ];
    for (source, contains) in cases {
        for i in [false, true] {
            for m in [false, true] {
                let plan = compile_scalar_left(source, i, m).unwrap();
                assert!(RegExpUnicodeClassMatcher::compile_v_set_with_single_character_strings_and_work(&JsString::from(source),i,m,|_|Ok::<(),()>(())).unwrap().is_none(),"{source}");
                let members = points
                    .iter()
                    .copied()
                    .filter(|&p| contains(if i { fold(p) } else { p }))
                    .map(|p| [p])
                    .collect::<Vec<_>>();
                let words = members.iter().map(|p| p.as_slice()).collect::<Vec<_>>();
                for input in [
                    JsString::from(""),
                    JsString::from("xabcd!abc!"),
                    JsString::from("ſKẞ5!"),
                    JsString::from("A😀ab𐀀"),
                    JsString::from("!\r\na\u{2028}!"),
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

#[test]
fn scalar_left_scope_requires_a_scalar_first_operand_and_complete_ignored_branches() {
    for source in [
        r"[\q{a|ab}&&\p{Any}]",
        r"[\q{ab}--\p{Any}]",
        r"[[a\q{ab}]&&\p{Any}]",
        r"[\p{Any}\q{ab}]",
        r"[^\p{Any}--\q{ab}]",
        r"[[a-z]&&\q{a}]",
        r"[\p{Any}&&\p{RGI_Emoji}&&\q{ab}]",
        r"[[]&&[\q{ab}]&&\p{RGI_Emoji}]",
        r"[\p{Any}&&[\q{ab}]]+",
        r"[\p{Any}&&[\q{ab}]][a]",
        r"\b^[\p{Any}&&[\q{ab}]]",
    ] {
        assert!(
            compile_scalar_left(source, true, true).is_none(),
            "{source}"
        );
    }
    let source = r"^\b[[a-z]&&[\q{a|ab}]]\b$";
    let plan = compile_scalar_left(source, true, true).unwrap();
    let input = JsString::from("!\r\nA\u{2028}!");
    assert_eq!(plan.find(&input, 0, false), Some(3..4));
    assert_eq!(plan.find(&input, 0, true), None);
    let plan = compile_scalar_left(r"[\p{Any}&&\q{ab|}]", false, false).unwrap();
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), None);
    let plan = compile_scalar_left(r"[\p{Any}--\q{ab|}]", false, false).unwrap();
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), Some(0..2));
    let plan = compile_scalar_left(r"[\p{ASCII}--[\q{k|sK|}]]", true, false).unwrap();
    assert_eq!(plan.find(&JsString::from("Kſ"), 0, false), Some(1..2));
    assert_eq!(plan.find(&JsString::from("ſK"), 0, true), Some(0..1));
}

#[test]
fn scalar_projection_checks_every_callback_and_keeps_deep_and_long_operands_flat() {
    for source in [
        r"[\p{ASCII}--[\q{k|sK|}]]",
        r"[[]&&[\q{ab}]]",
        r"[[a-z]&&[a\q{ab}]&&[^b]]",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_with_work(
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
            let result = RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_with_work(
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
        RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_with_work(
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
            r"[\p{{Any}}&&{}\q{{a|ab}}{}]",
            "[".repeat(20_000),
            "]".repeat(20_000)
        )
        .as_str(),
    );
    let mut work = 0usize;
    let original = RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_with_work(
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
        "projected nesting must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.find(&JsString::from("xab"), 0, false), Some(1..2));
    let source = JsString::from(format!(r"[\p{{Any}}--\q{{{}}}]", "x".repeat(20_000)).as_str());
    let mut work = 0usize;
    let plan = RegExpUnicodeClassMatcher::compile_scalar_left_string_operations_with_work(
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
        work < source.len() * 12 + 512,
        "ignored words must stay compact: {work}"
    );
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), Some(0..2));
}
