use super::*;

fn compile_result(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeClassMatcher> {
    RegExpUnicodeClassMatcher::compile_character_result_v_set_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn static_character_results_follow_independent_nested_set_predicates_and_utf16_ranges() {
    type Predicate = fn(u32) -> bool;
    let cases: [(&str, Predicate); 15] = [
        (r"[\q{a|ab|}&&\p{Any}]", |p| p == 97),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[[\q{ab}&&a]b]", |p| p == 98),
        (r"[[^\q{ab}&&a]&&\q{b|bc|}]", |p| p == 98),
        (r"[\q{a|ab}&&[[a-z]&&\q{a|ab}]]", |p| p == 97),
        (r"[[^\q{ab}&&a]--\q{a|ab}]", |p| p != 97),
        (r"[\q{\uFEFF|\u0085|ab}&&\s]", |p| p == 0xfeff),
        (r"[^\q{k|sK|}&&\p{ASCII}]", |p| p != 107),
        (r"[[\q{😀x|😀}&&\p{Any}]a]", |p| p == 97 || p == 0x1f600),
        (r"[^\q{}&&\p{Any}]", |_| true),
        (r"[\q{a|ab}&&[\q{a}\p{Any}]]", |p| p == 97),
        (r"[[\q{ab}&&\p{Any}][\q{a|ab}&&[a-z]]]", |p| p == 97),
        (r"[[\q{ab}--\q{ab}]&&\p{Any}]", |_| false),
        (r"[\q{a|ab}&&\q{a}]", |p| p == 97),
    ];
    let points = [
        0, 8, 10, 13, 32, 33, 48, 53, 65, 75, 97, 98, 99, 100, 107, 115, 120, 0xdf, 0x17f, 0x85,
        0x1e9e, 0x2028, 0x212a, 0xd800, 0xdc00, 0xfeff, 0x10000, 0x1f600,
    ];
    for (source, contains) in cases {
        for i in [false, true] {
            for m in [false, true] {
                let plan = compile_result(source, i, m).unwrap();
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
                    JsString::from("ſKẞA"),
                    JsString::from("A😀ab𐀀"),
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

#[test]
fn character_result_proof_is_conservative_and_retains_gates_folding_and_empty_set_identity() {
    for source in [
        r"[\q{ab}]",
        r"[\q{ab}&&\q{cd}]",
        r"[\q{ab}--\q{ab}]",
        r"[\q{}--\q{}]",
        r"[\q{a|ab}&&[a-z\q{ab}]]",
        r"[\q{ab}--\p{Any}]",
        r"[\q{a|ab}a]",
        r"[\q{a}&&a]",
        r"[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]",
        r"[^\q{ab}--\q{ab}]",
        r"[\q{ab}&&a]+",
        r"[\q{ab}&&a][b]",
        r"\b^[\q{ab}&&a]",
        r"[\q{ab}&&a]$\b",
    ] {
        assert!(compile_result(source, true, true).is_none(), "{source}");
    }
    let plan = compile_result(r"^\b[[\q{ab}&&a]b]\b$", true, true).unwrap();
    assert_eq!(
        plan.find(&JsString::from("!\r\nB\u{2028}!"), 0, false),
        Some(3..4)
    );
    assert_eq!(plan.find(&JsString::from("!\r\nB\u{2028}!"), 0, true), None);
    let plan = compile_result(r"[\q{ab|}&&\p{Any}]", false, false).unwrap();
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), None);
    assert_eq!(plan.find(&JsString::from(""), 0, true), None);
    let plan = compile_result(r"[^\q{ab|}&&\p{Any}]", false, false).unwrap();
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), Some(0..2));
    let plan = compile_result(r"[\q{μ|ab}&&\p{Script=Greek}]", true, false).unwrap();
    assert_eq!(plan.find(&JsString::from("µ"), 0, true), Some(0..1));
    let plan = compile_result(r"[\q{\u{D800}\u{DC00}|ab}&&\p{Any}]", false, false).unwrap();
    assert_eq!(plan.find(&JsString::from("𐀀ab"), 0, false), None);
}

#[test]
fn containment_projection_checks_every_callback_and_keeps_deep_and_long_words_flat() {
    for source in [
        r"[^\q{k|sK|}&&\p{ASCII}]",
        r"[[\q{ab}&&\p{Any}][\q{a|ab}&&[a-z]]]",
        r"[[]&&\q{ab}]",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        RegExpUnicodeClassMatcher::compile_character_result_v_set_with_work(
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
        assert_eq!(complete[0], source.len() * 4);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = RegExpUnicodeClassMatcher::compile_character_result_v_set_with_work(
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
            assert!(matches!(result,Err(RegExpUnicodeSetCompileError::Work(at)) if at==fail));
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        RegExpUnicodeClassMatcher::compile_character_result_v_set_with_work(
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
        format!(r"{}\q{{ab}}&&a{}", "[".repeat(20_000), "]".repeat(20_000)).as_str(),
    );
    let mut work = 0usize;
    let original = RegExpUnicodeClassMatcher::compile_character_result_v_set_with_work(
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
        "containment nesting must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.find(&JsString::from("ab"), 0, false), None);
    let source = JsString::from(format!(r"[\q{{{}}}&&\p{{Any}}]", "x".repeat(20_000)).as_str());
    let mut work = 0usize;
    let plan = RegExpUnicodeClassMatcher::compile_character_result_v_set_with_work(
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
        "discarded words must not expand: {work}"
    );
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), None);
}
