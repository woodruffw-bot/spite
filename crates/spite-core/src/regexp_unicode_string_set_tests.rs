use super::*;

fn compile(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeStringSetMatcher> {
    RegExpUnicodeStringSetMatcher::compile_with_work(&JsString::from(source), i, m, |_| {
        Ok::<(), ()>(())
    })
    .unwrap()
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
