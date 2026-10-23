use super::*;

fn compile_finite_operations(
    source: &str,
    i: bool,
    m: bool,
) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn finite_operands_match_independent_character_and_string_sets() {
    type Word<'a> = &'a [u32];
    type Words<'a> = &'a [Word<'a>];
    struct Case<'a> {
        source: &'a str,
        left: Words<'a>,
        right: Words<'a>,
        subtract: bool,
        gates: Gates,
    }
    let plain = Gates::default();
    let cases = [
        Case {
            source: r"[[\q{ab|cd}]&&[\q{cd|xy}]]",
            left: &[&[97, 98], &[99, 100]],
            right: &[&[99, 100], &[120, 121]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{a|ab}&&[a\q{cd}]]",
            left: &[&[97], &[97, 98]],
            right: &[&[97], &[99, 100]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[[a\q{ab|abc}]--[a\q{abc}]]",
            left: &[&[97], &[97, 98], &[97, 98, 99]],
            right: &[&[97], &[97, 98, 99]],
            subtract: true,
            gates: plain,
        },
        Case {
            source: r"[a&&\q{ab}]",
            left: &[&[97]],
            right: &[&[97, 98]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{ab}--a]",
            left: &[&[97, 98]],
            right: &[&[97]],
            subtract: true,
            gates: plain,
        },
        Case {
            source: r"[[]&&\q{ab|}]",
            left: &[],
            right: &[&[97, 98], &[]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{}--[]]",
            left: &[&[]],
            right: &[],
            subtract: true,
            gates: plain,
        },
        Case {
            source: r"[[K\q{sK|}]&&[k\q{ſK|}]]",
            left: &[&[75], &[115, 75], &[]],
            right: &[&[107], &[383, 8490], &[]],
            subtract: false,
            gates: Gates { i: true, ..plain },
        },
        Case {
            source: r"^\b[[[\q{ab}]a]--[\q{abc}]]\b$",
            left: &[&[97, 98], &[97]],
            right: &[&[97, 98, 99]],
            subtract: true,
            gates: Gates {
                m: true,
                anchors: [true, true],
                words: [Some(true), Some(true)],
                ..plain
            },
        },
        Case {
            source: r"[[[\q{a\]b}]]&&[\q{a\]b}]]",
            left: &[&[97, 93, 98]],
            right: &[&[97, 93, 98]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[[\q{😀x}🚀]--[\q{😀x}]]",
            left: &[&[0x1f600, 120], &[0x1f680]],
            right: &[&[0x1f600, 120]],
            subtract: true,
            gates: plain,
        },
    ];
    for case in cases {
        assert!(
            compile_operations(case.source, case.gates.i, case.gates.m).is_none(),
            "{}",
            case.source
        );
        let words = case
            .left
            .iter()
            .copied()
            .filter(|left| {
                let present = case.right.iter().any(|right| {
                    left.len() == right.len()
                        && left.iter().zip(*right).all(|(&a, &b)| {
                            if case.gates.i {
                                fold(a) == fold(b)
                            } else {
                                a == b
                            }
                        })
                });
                present != case.subtract
            })
            .collect::<Vec<_>>();
        let plan = compile_finite_operations(case.source, case.gates.i, case.gates.m).unwrap();
        for input in [
            JsString::from(""),
            JsString::from("xabcd!abc!"),
            JsString::from("a!a]b["),
            JsString::from("A😀x🚀"),
            JsString::from("ſK"),
            JsString::from("ſKẞA"),
            JsString::from("!\r\nab\u{2028}!"),
            JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
        ] {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        plan.find(&input, start, sticky),
                        oracle(&input, start, sticky, &words, case.gates),
                        "{} {input:?} start={start} sticky={sticky}",
                        case.source
                    );
                }
            }
        }
    }
}

#[test]
fn finite_operation_scope_keeps_ranges_properties_negation_and_inner_operators_separate() {
    for source in [
        r"[\q{ab}&&\q{ab}]",
        r"[\q{ab}--\q{cd}]",
        r"[[\q{ab}]]",
        r"[[a]&&[a]]",
        r"[[a-z]&&\q{ab}]",
        r"[\q{ab}&&[\d]]",
        r"[\q{ab}&&\p{Any}]",
        r"[[^a]--\q{ab}]",
        r"[[\q{ab}&&\q{ab}]--\q{cd}]",
        r"[[]&&\q{ab}&&[\p{RGI_Emoji}]]",
        r"[[\q{ab}]--[\q{ab}]]+",
        r"[[\q{ab}]--[\q{ab}]][a]",
        r"\b^[[\q{ab}]--[\q{ab}]]",
    ] {
        assert!(
            compile_finite_operations(source, true, true).is_none(),
            "{source}"
        );
    }
    let empty = compile_finite_operations(r"[[]&&\q{ab|}]", false, false).unwrap();
    assert_eq!(empty.max_match_code_units(), 0);
    assert_eq!(empty.find(&JsString::from("😀"), 1, true), None);
    let zero = compile_finite_operations(r"[\q{}--[]]", false, false).unwrap();
    assert_eq!(zero.max_match_code_units(), 0);
    assert_eq!(zero.find(&JsString::from("😀"), 1, true), Some(0..0));
}

#[test]
fn finite_operation_callbacks_cover_empty_left_sets_and_deep_operand_streams() {
    for source in [
        r"[[a\q{ab|abc}]&&[a\q{abc}]]",
        r"[[]&&[\q{ab|}]&&[\q{cd}]]",
        r"[[K\q{sK|}]--[\q{ſK}]]",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
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
            let result = RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
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
        let retry = RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
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
        for text in ["", "ab", "ABC", "ſK", "😀"] {
            let input = JsString::from(text);
            assert_eq!(plan.find(&input, 0, false), retry.find(&input, 0, false));
        }
    }
    let source = JsString::from(
        format!(
            r"[{}\q{{ab}}{}&&[\q{{ab}}]]",
            "[".repeat(20_000),
            "]".repeat(20_000)
        )
        .as_str(),
    );
    let mut work = 0usize;
    let original = RegExpUnicodeStringSetMatcher::compile_finite_string_operations_with_work(
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
        work < source.len() * 6 + 128,
        "finite operand streaming must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.max_match_code_units(), 4);
    assert_eq!(plan.find(&JsString::from("xab"), 0, false), Some(1..3));
}
