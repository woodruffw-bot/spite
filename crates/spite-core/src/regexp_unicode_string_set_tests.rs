use super::*;

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_with_work(&JsString::from(source), i, m, |_| {
        Ok::<(), ()>(())
    })
    .unwrap()
}

fn compile_operations(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn q_only_operations_match_independent_canonical_word_sets() {
    type Word<'a> = &'a [u32];
    type Words<'a> = &'a [Word<'a>];
    type Sets<'a> = &'a [Words<'a>];
    struct Case<'a> {
        source: &'a str,
        left: Words<'a>,
        right: Sets<'a>,
        subtract: bool,
        gates: Gates,
    }
    let plain = Gates::default();
    let base: Words<'_> = &[&[97], &[97, 98], &[97, 98, 99], &[]];
    let cases = [
        Case {
            source: r"[\q{a|ab|abc|}&&\q{ab|}]",
            left: base,
            right: &[&[&[97, 98], &[]]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{a|ab|abc|}--\q{ab|}]",
            left: base,
            right: &[&[&[97, 98], &[]]],
            subtract: true,
            gates: plain,
        },
        Case {
            source: r"^[\q{K|sK|}&&\q{k|ſK|}]\b$",
            left: &[&[75], &[115, 75], &[]],
            right: &[&[&[107], &[383, 8490], &[]]],
            subtract: false,
            gates: Gates {
                i: true,
                m: true,
                anchors: [true, true],
                words: [None, Some(true)],
            },
        },
        Case {
            source: r"[\q{a|ab}&&\q{abc}]",
            left: &[&[97], &[97, 98]],
            right: &[&[&[97, 98, 99]]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{|ab}--\q{ab}]",
            left: &[&[], &[97, 98]],
            right: &[&[&[97, 98]]],
            subtract: true,
            gates: plain,
        },
        Case {
            source: r"[\q{a|ab|abc}&&\q{a|ab}&&\q{ab}]",
            left: &base[..3],
            right: &[&[&[97], &[97, 98]], &[&[97, 98]]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{a|ab|abc|abcd}--\q{a}--\q{abc}]",
            left: &[&[97], &[97, 98], &[97, 98, 99], &[97, 98, 99, 100]],
            right: &[&[&[97]], &[&[97, 98, 99]]],
            subtract: true,
            gates: plain,
        },
        Case {
            source: r"[\q{\uD83D\uDE00x|🚀|}&&\q{😀x|}]",
            left: &[&[0x1f600, 120], &[0x1f680], &[]],
            right: &[&[&[0x1f600, 120], &[]]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{\u{D800}\u{DC00}|ab}&&\q{\uD800\uDC00|ab}]",
            left: &[&[0xd800, 0xdc00], &[97, 98]],
            right: &[&[&[0x10000], &[97, 98]]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{a\|b|ab}&&\q{a\|b}]",
            left: &[&[97, 124, 98], &[97, 98]],
            right: &[&[&[97, 124, 98]]],
            subtract: false,
            gates: plain,
        },
        Case {
            source: r"[\q{a!|a}&&\q{a!|a}]\b",
            left: &[&[97, 33], &[97]],
            right: &[&[&[97, 33], &[97]]],
            subtract: false,
            gates: Gates {
                words: [None, Some(true)],
                ..plain
            },
        },
        Case {
            source: r"[\q{sK|K|}--\q{ſK}]",
            left: &[&[115, 75], &[75], &[]],
            right: &[&[&[383, 8490]]],
            subtract: true,
            gates: Gates { i: true, ..plain },
        },
    ];
    for case in cases {
        let mut words = case.left.to_vec();
        for right in case.right {
            words.retain(|left| {
                let present = right.iter().any(|right| {
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
            });
        }
        let plan = compile_operations(case.source, case.gates.i, case.gates.m).unwrap();
        for input in [
            JsString::from(""),
            JsString::from("xabcd!abc!"),
            JsString::from("a!a|b"),
            JsString::from("A😀x🚀"),
            JsString::from("ſK"),
            JsString::from("ſKẞA"),
            JsString::from("!\r\nab\u{2028}!"),
            JsString::from("𐀀ab"),
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
fn q_operation_scope_and_empty_intermediates_require_every_complete_operand() {
    for source in [
        r"[\q{ab}]",
        r"[\q{ab}\q{cd}]",
        r"[\q{a}&&\q{a}]",
        r"[\q{ab}&&[\q{ab}]]",
        r"[\q{ab}&&\p{Any}]",
        r"[[\q{ab}]--\q{ab}]",
        r"[\q{ab}&&\q{cd}&&\p{RGI_Emoji}]",
        r"[\q{ab}&&\q{ab}--\q{ab}]",
        r"[^\q{ab}&&\q{a}]",
        r"[\q{ab}&&\q{ab}]+",
        r"[\q{ab}--\q{ab}][a]",
        r"\b^[\q{ab}&&\q{ab}]",
    ] {
        assert!(compile_operations(source, true, true).is_none(), "{source}");
    }
    let empty = compile_operations(r"[\q{ab}&&\q{cd}]", false, false).unwrap();
    assert_eq!(empty.max_match_code_units(), 0);
    assert_eq!(empty.find(&JsString::from("abcd"), 0, false), None);
    let zero = compile_operations(r"[\q{|ab}--\q{ab}]", false, false).unwrap();
    assert_eq!(zero.max_match_code_units(), 0);
    assert_eq!(zero.find(&JsString::from("😀"), 1, true), Some(0..0));
}

#[test]
fn q_operations_cancel_every_phase_and_avoid_retaining_rhs_only_words() {
    for source in [
        r"[\q{ab|abc|}&&\q{ab|}&&\q{}]",
        r"[\q{sK|K|}--\q{ſK}--\q{k}]",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
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
            let result = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
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
        let retry = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
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
    let right = "x".repeat(60_000);
    let source = JsString::from(format!(r"[\q{{ab}}--\q{{{right}}}]").as_str());
    let mut work = 0usize;
    let plan = RegExpUnicodeStringSetMatcher::compile_string_operations_with_work(
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
        work < source.len() * 8 + 256,
        "right-only words must be probed without node insertion: {work}"
    );
    assert_eq!(plan.max_match_code_units(), 4);
    assert_eq!(plan.find(&JsString::from("ab"), 0, true), Some(0..2));
    let left = "a".repeat(20_000);
    let plan =
        compile_operations(&format!(r"[\q{{a|{left}}}--\q{{{left}}}]"), false, false).unwrap();
    assert_eq!(plan.max_match_code_units(), 2);
    assert_eq!(
        plan.find(&JsString::from(left.as_str()), 0, true),
        Some(0..1)
    );
}

fn compile_scalars(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

fn compile_nested(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn positive_nested_unions_match_independent_flat_member_sets() {
    let plain = Gates::default();
    type Case<'a> = (&'a str, &'a [&'a [u32]], Option<ScalarOracle>, Gates);
    let cases: [Case<'_>; 10] = [
        (r"[[\q{ab}]]", &[&[97, 98]], None, plain),
        (r"[[\q{}][]]", &[&[]], None, plain),
        (
            r"[[\q{a}][\q{ab|abc}]]",
            &[&[97], &[97, 98], &[97, 98, 99]],
            None,
            plain,
        ),
        (
            r"[a[[\q{ab}]]b]",
            &[&[97, 98]],
            Some(ScalarOracle::Range(97, 98)),
            plain,
        ),
        (
            r"[[a-z][\q{ab|a!}]]\b",
            &[&[97, 98], &[97, 33]],
            Some(ScalarOracle::Range(97, 122)),
            Gates {
                i: true,
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"^\b[[a-z][\q{ab}]]\b$",
            &[&[97, 98]],
            Some(ScalarOracle::Range(97, 122)),
            Gates {
                m: true,
                anchors: [true, true],
                words: [Some(true), Some(true)],
                ..plain
            },
        ),
        (
            r"[[\q{a\]b}][\[]]",
            &[&[97, 93, 98]],
            Some(ScalarOracle::Range(91, 91)),
            plain,
        ),
        (
            r"[[\q{}][\p{Any}]]",
            &[&[]],
            Some(ScalarOracle::Range(0, 0x10ffff)),
            plain,
        ),
        (
            r"[[\p{ASCII}][\q{ab}]]",
            &[&[97, 98]],
            Some(ScalarOracle::Ascii(true)),
            Gates { i: true, ..plain },
        ),
        (
            r"[[\u{D7FF}-\u{E000}][\q{ab}]]",
            &[&[97, 98]],
            Some(ScalarOracle::Range(0xd7ff, 0xe000)),
            plain,
        ),
    ];
    for (source, choices, scalar, gates) in cases {
        assert!(compile(source, gates.i, gates.m).is_none(), "{source}");
        assert!(
            compile_union(source, gates.i, gates.m).is_none(),
            "{source}"
        );
        assert!(
            compile_character_union(source, gates.i, gates.m).is_none(),
            "{source}"
        );
        assert!(
            compile_scalars(source, gates.i, gates.m).is_none(),
            "{source}"
        );
        let plan = compile_nested(source, gates.i, gates.m).unwrap();
        for input in [
            JsString::from(""),
            JsString::from("xabc!ab"),
            JsString::from("a!a]b["),
            JsString::from("A😀x🚀"),
            JsString::from("ſKẞA"),
            JsString::from("!\r\nab\u{2028}!"),
            JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
        ] {
            let mut words = choices.iter().map(|s| s.to_vec()).collect::<Vec<_>>();
            if let Some(scalar) = scalar {
                for c in char::decode_utf16(input.code_units().iter().copied()) {
                    let p = match c {
                        Ok(c) => u32::from(c),
                        Err(e) => u32::from(e.unpaired_surrogate()),
                    };
                    if scalar.contains(p, gates.i) {
                        words.push(vec![p]);
                    }
                }
            }
            let choices = words.iter().map(Vec::as_slice).collect::<Vec<_>>();
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        plan.find(&input, start, sticky),
                        oracle(&input, start, sticky, &choices, gates),
                        "{source} {input:?} start={start} sticky={sticky}"
                    );
                }
            }
        }
    }
}

#[test]
fn nested_string_union_proof_requires_positive_nesting_and_a_variable_q_member() {
    for source in [
        r"[\q{ab}]",
        r"[a\q{ab}]",
        r"[\d\q{ab}]",
        r"[[a-z]]",
        r"[[\q{a}]]",
        r"[[^a]\q{ab}]",
        r"[[\q{ab}]&&[a-z]]",
        r"[[\q{ab}]--[a-z]]",
        r"[[\q{ab}][\p{RGI_Emoji}]]",
        r"[[\q{ab}]]+",
        r"[[\q{ab}]]x",
        r"\b^[[\q{ab}]]",
    ] {
        assert!(compile_nested(source, true, true).is_none(), "{source}");
    }
    let empty = compile_nested(r"[[\q{}][\P{Any}]]", false, false).unwrap();
    assert_eq!(empty.max_match_code_units(), 0);
    assert_eq!(empty.find(&JsString::from("😀"), 1, true), Some(0..0));
    let any = compile_nested(r"[[\q{}][\p{Any}]]", false, false).unwrap();
    assert_eq!(any.max_match_code_units(), 2);
    assert_eq!(any.find(&JsString::from("😀"), 1, true), Some(0..2));
}

#[test]
fn positive_nested_string_unions_cancel_every_phase_and_keep_deep_storage_flat() {
    for source in [
        r"[[\q{}][a-z][\q{abc}]]",
        r"^\b[[\p{Script=Greek}][\q{ab}]]\b$",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
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
            let result = RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
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
        let retry = RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
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
        for text in ["", "ab", "ABC", "µ", "!\r\nab\u{2028}!"] {
            let input = JsString::from(text);
            assert_eq!(plan.find(&input, 0, false), retry.find(&input, 0, false));
        }
    }
    let source =
        JsString::from(format!(r"[{}\q{{ab}}{}]", "[".repeat(20_000), "]".repeat(20_000)).as_str());
    let mut work = 0usize;
    let original = RegExpUnicodeStringSetMatcher::compile_positive_nested_union_with_work(
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
        work < source.len() * 8 + 128,
        "nested union construction must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.max_match_code_units(), 4);
    assert_eq!(plan.find(&JsString::from("xab"), 0, false), Some(1..3));
}

#[derive(Clone, Copy)]
enum ScalarOracle {
    Range(u32, u32),
    Digit(bool),
    Word,
    Ascii(bool),
    Greek,
    Space(bool),
}

impl ScalarOracle {
    // Handwritten membership for the input points used here. Whole Unicode
    // property tables are independently covered by the class compiler tests.
    fn contains(self, p: u32, i: bool) -> bool {
        match self {
            Self::Range(a, b) => (a..=b).contains(&p) || (i && (a..=b).contains(&fold(p))),
            Self::Digit(positive) => (48..=57).contains(&p) == positive,
            Self::Word => {
                matches!(p,48..=57|65..=90|95|97..=122) || (i && matches!(p, 0x17f | 0x212a))
            }
            Self::Ascii(positive) => (p < 128 || (i && matches!(p, 0x17f | 0x212a))) == positive,
            Self::Greek => matches!(p, 0x391 | 0x39c | 0x3b1 | 0x3bc) || (i && p == 0xb5),
            Self::Space(property) => {
                matches!(p,9..=13|32|0xa0|0x1680|0x2000..=0x200a|0x2028|0x2029|0x202f|0x205f|0x3000)
                    || if property { p == 0x85 } else { p == 0xfeff }
            }
        }
    }
}

#[test]
fn scalar_ranges_escapes_and_properties_union_with_strings_using_independent_membership() {
    use ScalarOracle as S;
    let plain = Gates::default();
    let folded = Gates { i: true, ..plain };
    let ab: &[&[u32]] = &[&[97, 98]];
    let cases: [(&str, &[&[u32]], S, Gates); 15] = [
        (
            r"[a-z\q{ab|a!}]",
            &[&[97, 98], &[97, 33]],
            S::Range(97, 122),
            folded,
        ),
        (r"[\q{ab}\d]", ab, S::Digit(true), plain),
        (r"[\D\q{ab}]", ab, S::Digit(false), plain),
        (r"[\w\q{sK}]", &[&[115, 75]], S::Word, folded),
        (r"[\P{ASCII}\q{ab}]", ab, S::Ascii(false), folded),
        (r"[\q{ab}\p{ASCII}]", ab, S::Ascii(true), plain),
        (r"[\q{ab}\p{Any}]", ab, S::Range(0, 0x10ffff), plain),
        (
            r"[\q{a!}\p{Any}]\b",
            &[&[97, 33]],
            S::Range(0, 0x10ffff),
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"^\b[\q{ab}a-z]\b$",
            ab,
            S::Range(97, 122),
            Gates {
                m: true,
                anchors: [true, true],
                words: [Some(true), Some(true)],
                ..plain
            },
        ),
        (
            r"[\q{ab}\u{D7FF}-\u{E000}]",
            ab,
            S::Range(0xd7ff, 0xe000),
            plain,
        ),
        (
            r"[\u{0}-\u{10FFFF}\q{ab}]",
            ab,
            S::Range(0, 0x10ffff),
            folded,
        ),
        (r"[\q{}\p{Any}]", &[&[]], S::Range(0, 0x10ffff), plain),
        (r"[\q{}\P{Any}]", &[&[]], S::Range(1, 0), plain),
        (r"[\p{Script=Greek}\q{ab}]", ab, S::Greek, folded),
        (r"[\q{ab}\s]", ab, S::Space(false), plain),
    ];
    for (source, choices, scalar, gates) in cases {
        assert!(
            compile_character_union(source, gates.i, gates.m).is_none(),
            "{source}"
        );
        let plan = compile_scalars(source, gates.i, gates.m).unwrap();
        for input in [
            JsString::from(""),
            JsString::from("xab!ab0_"),
            JsString::from("A😀x🚀"),
            JsString::from("ſKẞA"),
            JsString::from("Μμµα"),
            JsString::from("x\r\nab\u{2028}z"),
            JsString::from("\t\u{85}\u{a0}\u{feff}"),
            JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
        ] {
            let mut words = choices.iter().map(|s| s.to_vec()).collect::<Vec<_>>();
            for c in char::decode_utf16(input.code_units().iter().copied()) {
                let p = match c {
                    Ok(c) => u32::from(c),
                    Err(e) => u32::from(e.unpaired_surrogate()),
                };
                if scalar.contains(p, gates.i) {
                    words.push(vec![p]);
                }
            }
            let choices = words.iter().map(Vec::as_slice).collect::<Vec<_>>();
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        plan.find(&input, start, sticky),
                        oracle(&input, start, sticky, &choices, gates),
                        "{source} {input:?} start={start} sticky={sticky}"
                    );
                }
            }
        }
    }
    let es = compile_scalars(r"[\q{ab}\s]", false, false).unwrap();
    let ucd = compile_scalars(r"[\q{ab}\p{White_Space}]", false, false).unwrap();
    assert_eq!(es.find(&JsString::from("\u{feff}"), 0, true), Some(0..1));
    assert_eq!(ucd.find(&JsString::from("\u{feff}"), 0, true), None);
    assert_eq!(es.find(&JsString::from("\u{85}"), 0, true), None);
    assert_eq!(ucd.find(&JsString::from("\u{85}"), 0, true), Some(0..1));
}

#[test]
fn scalar_string_union_keeps_other_proofs_and_algebra_separate() {
    for source in [
        r"[\q{ab}]",
        r"[\q{ab}a]",
        r"[\q{ab}\q{cd}]",
        r"[a-z]",
        r"[\d\q{a}]",
        r"[^\q{ab}\d]",
        r"[[a-z]\q{ab}]",
        r"[a-z&&\q{ab}]",
        r"[\q{ab}--\d]",
        r"[\q{ab}\p{RGI_Emoji}]",
        r"[\q{ab}\d]+",
        r"[\q{ab}\d]x",
        r"\b^[\q{ab}\d]",
    ] {
        assert!(compile_scalars(source, true, true).is_none(), "{source}");
    }
    let plan = compile_scalars(r"[\q{}\p{Any}]", false, false).unwrap();
    assert_eq!(plan.max_match_code_units(), 2);
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), Some(0..2));
    assert_eq!(plan.find(&JsString::from(""), 0, true), Some(0..0));
}

#[test]
fn scalar_string_union_checks_every_reservation_and_keeps_full_ranges_compact() {
    for source in [r"[\q{}a-z\q{abc}]", r"[\p{Script=Greek}\q{ab}]"] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
            &source,
            true,
            false,
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
            let result = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
                &source,
                true,
                false,
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
        let retry = RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(
            &source,
            true,
            false,
            |work| {
                recovered.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
        for text in ["", "ab", "ABC", "µ", "😀"] {
            let input = JsString::from(text);
            assert_eq!(plan.find(&input, 0, false), retry.find(&input, 0, false));
        }
    }
    let source = JsString::from(r"[\q{ab}\u{0}-\u{10FFFF}]");
    let mut work = 0usize;
    let plan =
        RegExpUnicodeStringSetMatcher::compile_scalar_union_with_work(&source, true, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
    assert!(
        work < 1_000_000,
        "a full Unicode range must retain interval storage: {work}"
    );
    assert_eq!(plan.max_match_code_units(), 4);
    assert_eq!(plan.find(&JsString::from("😀"), 1, true), Some(0..2));
}

fn compile_union(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_string_union_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

fn compile_character_union(
    source: &str,
    i: bool,
    m: bool,
) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_character_union_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

#[test]
fn character_and_string_operands_match_independent_decoded_sequences_in_any_order() {
    let plain = Gates::default();
    let cases: [(&str, &[&[u32]], Gates); 9] = [
        (r"[a\q{ab}]", &[&[97], &[97, 98]], plain),
        (r"[\q{ab}a]", &[&[97, 98], &[97]], plain),
        (
            r"[x\q{ab}a\q{abc}\q{}z]",
            &[&[120], &[97, 98], &[97], &[97, 98, 99], &[], &[122]],
            plain,
        ),
        (
            r"[\q{a\|b}\|\}\]]",
            &[&[97, 124, 98], &[124], &[125], &[93]],
            plain,
        ),
        (
            r"[\q{a!}a]\b",
            &[&[97, 33], &[97]],
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"^\b[k\q{sK}\q{ßa}]\b$",
            &[&[107], &[115, 75], &[223, 97]],
            Gates {
                i: true,
                m: true,
                anchors: [true, true],
                words: [Some(true), Some(true)],
            },
        ),
        (
            r"[😀\q{😀x}🚀]",
            &[&[0x1f600], &[0x1f600, 120], &[0x1f680]],
            plain,
        ),
        (
            r"[\q{\u{D800}\u{DC00}}\u{D800}\u{DC00}]",
            &[&[0xd800, 0xdc00], &[0xd800], &[0xdc00]],
            plain,
        ),
        (
            r"[\q{ab}\uD83D\uDE00\b]",
            &[&[97, 98], &[0x1f600], &[8]],
            plain,
        ),
    ];
    for (source, choices, gates) in cases {
        assert!(compile(source, gates.i, gates.m).is_none(), "{source}");
        assert!(
            compile_union(source, gates.i, gates.m).is_none(),
            "{source}"
        );
        let original = compile_character_union(source, gates.i, gates.m).unwrap();
        let matcher = original.clone();
        drop(original);
        let inputs = [
            JsString::from(""),
            JsString::from("xabc!ab"),
            JsString::from("a|b}!]"),
            JsString::from("a!"),
            JsString::from("x\r\nſK\u{2028}z"),
            JsString::from("A😀x🚀"),
            JsString::from("𐀀"),
            JsString::from("\u{8}"),
            JsString::from_code_units(vec![0xd800, 0x61, 0xdc00]),
        ];
        for input in inputs {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(&input, start, sticky),
                        oracle(&input, start, sticky, choices, gates),
                        "{source} {input:?} start={start} sticky={sticky}"
                    );
                }
            }
        }
    }
}

#[test]
fn literal_string_union_proof_excludes_other_algebra_and_keeps_earlier_domains() {
    for source in [
        r"[\q{ab}]",
        r"[\q{}]",
        r"[\q{ab}\q{cd}]",
        r"[a\q{b}]",
        r"[\q{b}a]",
        r"[ab]",
        r"[^a\q{ab}]",
        r"[a-z\q{ab}]",
        r"[a\d\q{ab}]",
        r"[\q{ab}\p{Any}]",
        r"[[a]\q{ab}]",
        r"[a&&\q{ab}]",
        r"[\q{ab}&&a]",
        r"[\q{ab}--a]",
        r"[a\q{ab}]+",
        r"[a\q{ab}]x",
        r"\b^[a\q{ab}]",
    ] {
        assert!(
            compile_character_union(source, true, true).is_none(),
            "{source}"
        );
    }
}

#[test]
fn mixed_union_construction_cancels_after_every_operand_and_handles_flat_large_classes() {
    for source in [r"[a\q{ab}b]", r"[\q{}a\q{sK}k]", r"^\b[k\q{sK}\q{ßa}]\b$"] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        let plan = RegExpUnicodeStringSetMatcher::compile_character_union_with_work(
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
            let result = RegExpUnicodeStringSetMatcher::compile_character_union_with_work(
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
        let retry = RegExpUnicodeStringSetMatcher::compile_character_union_with_work(
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
        for text in ["", "ab", "ſK", "x\r\nßA\u{2028}z"] {
            let input = JsString::from(text);
            assert_eq!(plan.find(&input, 0, false), retry.find(&input, 0, false));
        }
    }
    let source = format!(r"[\q{{}}{}\q{{bc}}]", "a".repeat(20_000));
    let matcher = compile_character_union(&source, false, false).unwrap();
    assert_eq!(matcher.max_match_code_units(), 4);
    assert_eq!(matcher.find(&JsString::from("bc"), 0, true), Some(0..2));
}

#[test]
fn adjacent_string_operands_share_longest_priority_and_independent_membership() {
    let plain = Gates::default();
    let cases: [(&str, &[&[u32]], Gates); 7] = [
        (
            r"[\q{a}\q{ab|abc}]",
            &[&[97], &[97, 98], &[97, 98, 99]],
            plain,
        ),
        (r"[\q{|}\q{ab|a}]", &[&[], &[], &[97, 98], &[97]], plain),
        (
            r"^\b[\q{a!}\q{a|}]\b$",
            &[&[97, 33], &[97], &[]],
            Gates {
                m: true,
                anchors: [true, true],
                words: [Some(true), Some(true)],
                ..plain
            },
        ),
        (r"[\q{a\}}\q{\{b}]", &[&[97, 125], &[123, 98]], plain),
        (
            r"[\q{\uD83D\uDE00x}\q{🚀}]",
            &[&[0x1f600, 120], &[0x1f680]],
            plain,
        ),
        (
            r"[\q{sK}\q{k|ßa}]",
            &[&[115, 75], &[107], &[223, 97]],
            Gates { i: true, ..plain },
        ),
        (
            r"[\q{\u{D800}\u{DC00}}\q{ab}]",
            &[&[0xd800, 0xdc00], &[97, 98]],
            plain,
        ),
    ];
    for (source, choices, gates) in cases {
        assert!(compile(source, gates.i, gates.m).is_none(), "{source}");
        let matcher = compile_union(source, gates.i, gates.m).unwrap();
        for text in [
            "",
            "xabc!ab",
            "a!",
            "x\r\na\u{2028}z",
            "a}{b",
            "A😀x🚀",
            "ſKẞA",
            "𐀀ab",
        ] {
            let input = JsString::from(text);
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(&input, start, sticky),
                        oracle(&input, start, sticky, choices, gates),
                        "{source} {text:?} start={start} sticky={sticky}"
                    );
                }
            }
        }
    }
}

#[test]
fn string_union_domain_requires_multiple_q_operands_and_a_variable_member() {
    for source in [
        r"[\q{ab}]",
        r"[\q{}]",
        r"[\q{a}\q{b}]",
        r"[^\q{ab}\q{cd}]",
        r"[[\q{ab}\q{cd}]]",
        r"[\q{ab}a\q{cd}]",
        r"[\q{ab}&&\q{cd}]",
        r"[\q{ab}--\q{cd}]",
        r"[\q{ab}\p{Any}]",
        r"[\q{ab}\q{cd}]+",
        r"[\q{ab}\q{cd}]x",
        r"\b^[\q{ab}\q{cd}]",
    ] {
        assert!(compile_union(source, true, true).is_none(), "{source}");
    }
    let matcher = compile_union(r"[\q{}\q{}]", false, false).unwrap();
    assert_eq!(matcher.max_match_code_units(), 0);
    assert_eq!(matcher.find(&JsString::from("😀"), 1, true), Some(0..0));
}

#[test]
fn string_union_checks_later_operands_after_empty_and_recovers_from_every_abort() {
    let source = JsString::from(r"^\b[\q{}\q{ab|abc}\q{sK|ſk}]\b$");
    let mut complete = Vec::new();
    let matcher = RegExpUnicodeStringSetMatcher::compile_string_union_with_work(
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
    for fail in 1..=complete.len() {
        let mut seen = Vec::new();
        let result = RegExpUnicodeStringSetMatcher::compile_string_union_with_work(
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
    let mut rebuilt = Vec::new();
    let recovered = RegExpUnicodeStringSetMatcher::compile_string_union_with_work(
        &source,
        true,
        true,
        |work| {
            rebuilt.push(work);
            Ok::<(), usize>(())
        },
    )
    .unwrap()
    .unwrap();
    assert_eq!(rebuilt, complete);
    assert_eq!(matcher.max_match_code_units(), 6);
    for text in ["", "ab", "abc", "ſK", "x\r\nab\u{2028}y"] {
        let input = JsString::from(text);
        assert_eq!(
            matcher.find(&input, 0, false),
            recovered.find(&input, 0, false)
        );
    }
    let source = format!("[{}]", r"\q{ab}".repeat(20_000));
    let matcher = compile_union(&source, false, false).unwrap();
    assert_eq!(matcher.max_match_code_units(), 4);
    assert_eq!(matcher.find(&JsString::from("ab"), 0, true), Some(0..2));
}

#[derive(Clone, Copy, Default)]
struct Gates {
    i: bool,
    m: bool,
    anchors: [bool; 2],
    words: [Option<bool>; 2],
}

// A small handwritten preimage for these independent examples, rather than the
// production folding table or ClassString decoder.
fn fold(point: u32) -> u32 {
    match point {
        65..=90 => point + 32,
        0x17f => 115,
        0x212a => 107,
        0x1e9e => 0xdf,
        0x10400..=0x10427 => point + 0x28,
        _ => point,
    }
}

fn oracle(
    input: &JsString,
    start: usize,
    sticky: bool,
    choices: &[&[u32]],
    gates: Gates,
) -> Option<Range<usize>> {
    if start > input.len() {
        return None;
    }
    let mut points = Vec::new();
    let mut offsets = vec![0];
    for character in char::decode_utf16(input.code_units().iter().copied()) {
        let (point, width) = match character {
            Ok(c) => (u32::from(c), c.len_utf16()),
            Err(e) => (u32::from(e.unpaired_surrogate()), 1),
        };
        points.push(point);
        offsets.push(offsets.last().unwrap() + width);
    }
    let first = offsets.partition_point(|&at| at <= start) - 1;
    let word = |point| {
        matches!(point,48..=57|65..=90|95|97..=122) || (gates.i && matches!(point, 0x17f | 0x212a))
    };
    let boundary = |at: usize| {
        points.get(at).is_some_and(|&p| word(p))
            != at
                .checked_sub(1)
                .and_then(|at| points.get(at))
                .is_some_and(|&p| word(p))
    };
    let line = |point| matches!(point, 10 | 13 | 0x2028 | 0x2029);
    for begin in first..=points.len() {
        if sticky && begin != first {
            break;
        }
        let end = choices
            .iter()
            .filter_map(|choice| {
                let end = begin + choice.len();
                let actual = points.get(begin..end)?;
                let equal = actual
                    .iter()
                    .zip(*choice)
                    .all(|(&a, &b)| if gates.i { fold(a) == fold(b) } else { a == b });
                (equal
                    && (!gates.anchors[0] || begin == 0 || (gates.m && line(points[begin - 1])))
                    && (!gates.anchors[1] || end == points.len() || (gates.m && line(points[end])))
                    && gates.words[0].is_none_or(|wanted| boundary(begin) == wanted)
                    && gates.words[1].is_none_or(|wanted| boundary(end) == wanted))
                .then_some(end)
            })
            .max();
        if let Some(end) = end {
            return Some(offsets[begin]..offsets[end]);
        }
    }
    None
}

#[test]
fn variable_and_empty_members_follow_an_independent_character_list_oracle() {
    let inputs = [
        JsString::from(""),
        JsString::from("xabca!ab!"),
        JsString::from("aaa!"),
        JsString::from("a|b"),
        JsString::from("ſKSx"),
        JsString::from("A😀x🚀"),
        JsString::from("x\r\nab\u{2028}a!"),
        JsString::from("\u{10428}A!"),
        JsString::from_code_units(vec![0xd800, 92, 0xdc00, 0xdc00]),
    ];
    let plain = Gates::default();
    let folded = Gates { i: true, ..plain };
    let cases: [(&str, &[&[u32]], Gates); 12] = [
        (r"[\q{a|ab|abc}]", &[&[97], &[97, 98], &[97, 98, 99]], plain),
        (
            r"[\q{abc|a|ab|abc}]",
            &[&[97, 98, 99], &[97], &[97, 98], &[97, 98, 99]],
            plain,
        ),
        (r"[\q{}]", &[&[]], plain),
        (r"[\q{|a|ab|}]", &[&[], &[97], &[97, 98], &[]], plain),
        (
            r"[\q{ab|a!|a}]\b",
            &[&[97, 98], &[97, 33], &[97]],
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (
            r"[\q{a!|}]\b",
            &[&[97, 33], &[]],
            Gates {
                words: [None, Some(true)],
                ..plain
            },
        ),
        (r"[\q{a\|b|a}]", &[&[97, 124, 98], &[97]], plain),
        (r"[\q{sK|k}]", &[&[115, 75], &[107]], folded),
        (
            r"[\q{\uD83D\uDE00x|🚀}]",
            &[&[0x1f600, 120], &[0x1f680]],
            plain,
        ),
        (r"[\q{\uD800\\|x}]", &[&[0xd800, 92], &[120]], plain),
        (r"[\q{\u{10400}a|b}]", &[&[0x10400, 97], &[98]], folded),
        (
            r"[\q{\u{D800}\u{DC00}|ab}]",
            &[&[0xd800, 0xdc00], &[97, 98]],
            plain,
        ),
    ];
    for (source, choices, gates) in cases {
        let original = compile(source, gates.i, gates.m).unwrap();
        let matcher = original.clone();
        drop(original);
        for input in &inputs {
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    assert_eq!(
                        matcher.find(input, start, sticky),
                        oracle(input, start, sticky, choices, gates),
                        "{source} {input:?} start={start} sticky={sticky}"
                    );
                }
            }
        }
    }
}

#[test]
fn longer_members_precede_shorter_members_and_empty_members_at_the_same_start() {
    for (source, input, wanted) in [
        (r"[\q{a|ab|abc}]", "abc", Some(0..3)),
        (r"[\q{abc|ab|a}]", "abx", Some(0..2)),
        (r"[\q{|ab|a}]", "ab", Some(0..2)),
        (r"[\q{|ab|a}]", "xab", Some(0..0)),
        (r"[\q{ab|a!|a}]\b", "a!", Some(0..1)),
        (r"[\q{a!|}]\b", "a!", Some(0..0)),
        (r"[\q{ab|a}]$", "abx", None),
        (r"[\q{ab|a}]$", "aa", Some(1..2)),
        (r"[\q{a!|a}]\B", "a!", Some(0..2)),
    ] {
        assert_eq!(
            compile(source, false, false)
                .unwrap()
                .find(&JsString::from(input), 0, false),
            wanted,
            "{source}"
        );
    }
}

#[test]
fn line_and_word_assertions_filter_each_possible_member_in_original_context() {
    let choices: &[&[u32]] = &[&[], &[97], &[97, 97]];
    let inputs = [
        "",
        "aaa!",
        "xaa!",
        "!aax",
        "x\raa\ry",
        "x\naa\ny",
        "x\r\naa\r\ny",
        "x\u{2028}aa\u{2029}y",
    ];
    for m in [false, true] {
        for anchors in [[false, false], [true, false], [false, true], [true, true]] {
            for left in [None, Some(false), Some(true)] {
                for right in [None, Some(false), Some(true)] {
                    let word = |gate| match gate {
                        Some(true) => r"\b",
                        Some(false) => r"\B",
                        None => "",
                    };
                    let source = format!(
                        "{}{}[\\q{{|a|aa}}]{}{}",
                        if anchors[0] { "^" } else { "" },
                        word(left),
                        word(right),
                        if anchors[1] { "$" } else { "" }
                    );
                    let matcher = compile(&source, false, m).unwrap();
                    let gates = Gates {
                        m,
                        anchors,
                        words: [left, right],
                        ..Gates::default()
                    };
                    for text in inputs {
                        let input = JsString::from(text);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
                                    oracle(&input, start, sticky, choices, gates),
                                    "{source} m={m} {text:?} start={start} sticky={sticky}"
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
fn string_decoding_preserves_escapes_surrogates_and_simple_folding_without_expansion() {
    assert_eq!(
        compile(r"[\q{\uD83D\uDE00x|a}]", false, false)
            .unwrap()
            .find(&JsString::from("A😀x!"), 2, true),
        Some(1..4)
    );
    assert_eq!(
        compile(r"[\q{}]", false, false)
            .unwrap()
            .find(&JsString::from("😀"), 1, true),
        Some(0..0)
    );
    assert_eq!(
        compile(r"[\q{\u{D800}\u{DC00}|x}]", false, false)
            .unwrap()
            .find(&JsString::from("𐀀"), 0, false),
        None
    );
    assert_eq!(
        compile(r"[\q{ßa|x}]", true, false)
            .unwrap()
            .find(&JsString::from("ẞA"), 0, true),
        Some(0..2)
    );
    assert_eq!(
        compile(r"[\q{ßa|x}]", true, false)
            .unwrap()
            .find(&JsString::from("SSa"), 0, true),
        None
    );
    for (source, input) in [
        (r"[\q{\ba|x}]", "\u{8}a"),
        (r"[\q{\0a|x}]", "\0a"),
        ("[\\q{a\n|b}]", "a\n"),
        (r"[\q{\]\}|x}]", "]}"),
    ] {
        let input = JsString::from(input);
        assert_eq!(
            compile(source, false, false).unwrap().find(&input, 0, true),
            Some(0..input.len()),
            "{source}"
        );
    }
}

#[test]
fn domain_declines_other_class_grammar_and_keeps_single_character_proofs_separate() {
    for source in [
        "",
        "^$",
        r"\b",
        r"[\q{a|b}]",
        r"[\q{\uD83D\uDE00|a}]",
        r"[^\q{ab}]",
        r"[[\q{ab}]]",
        r"[\q{ab}&&[a-z]]",
        r"[\q{ab}--\q{cd}]",
        r"[\q{ab}a]",
        r"[\q{ab}\q{cd}]",
        r"[\p{RGI_Emoji}]",
        r"[\q{\d|ab}]",
        r"([\q{ab}])",
        r"[\q{ab}]+",
        r"[\q{ab}]x",
        r"x[\q{ab}]",
        r"\b^[\q{ab}]",
        r"[\q{ab}]$\b",
        r"[\q{ab}]\b\b",
        r"[\q{ab}]$$",
    ] {
        assert!(compile(source, true, true).is_none(), "{source}");
    }
    for source in [r"[\q{}]", r"[\q{a|ab}]"] {
        let source = JsString::from(source);
        assert!(
            crate::RegExpUnicodeClassMatcher::compile_single_character_strings_v_set_with_work(
                &source,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
        assert!(
            crate::RegExpUnicodeClassMatcher::compile_v_set_with_single_character_strings_and_work(
                &source,
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

#[test]
fn every_checked_construction_callback_can_abort_and_a_fresh_build_recovers() {
    for source in [r"[\q{}]", r"^\b[\q{ab|abc|a|}]\b$", r"[\q{sK|ſk|ab}]"] {
        let source = JsString::from(source);
        let mut full = Vec::new();
        let plan = RegExpUnicodeStringSetMatcher::compile_with_work(&source, true, true, |work| {
            full.push(work);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(full[0], source.len() * 4);
        for fail in 1..=full.len() {
            let mut seen = Vec::new();
            let result =
                RegExpUnicodeStringSetMatcher::compile_with_work(&source, true, true, |work| {
                    seen.push(work);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(matches!(result,Err(RegExpUnicodeSetCompileError::Work(at)) if at==fail));
            assert_eq!(seen, full[..fail]);
        }
        let mut rebuilt = Vec::new();
        let recovered =
            RegExpUnicodeStringSetMatcher::compile_with_work(&source, true, true, |work| {
                rebuilt.push(work);
                Ok::<(), usize>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(rebuilt, full);
        for text in ["", "ab", "abc", "ſK", "xxx"] {
            let text = JsString::from(text);
            assert_eq!(plan.find(&text, 0, false), recovered.find(&text, 0, false));
        }
    }
}

#[test]
fn long_members_compile_clone_match_and_drop_without_recursive_storage() {
    let word = "a".repeat(100_000);
    let source = format!(r"[\q{{{word}|a}}]");
    let original = compile(&source, false, false).unwrap();
    let matcher = original.clone();
    drop(original);
    assert_eq!(matcher.max_match_code_units(), 200_000);
    assert_eq!(
        matcher.find(&JsString::from(word.as_str()), 0, true),
        Some(0..100_000)
    );
    assert_eq!(
        compile(r"[\q{}]", false, false)
            .unwrap()
            .max_match_code_units(),
        0
    );
    assert!(matcher.search_passes(false) > matcher.search_passes(true));
}
