use super::*;

fn compile(source: &str, flags: &str) -> RegExpProgram {
    RegExpProgram::compile_validated(&JsString::from(source), &JsString::from(flags))
        .unwrap_or_else(|error| panic!("{source} /{flags}: {error}"))
}
fn found(
    program: &RegExpProgram,
    input: &JsString,
    start: usize,
    sticky: bool,
) -> Option<RegExpProgramMatch> {
    match program.find(input, start, sticky, None).0 {
        RegExpProgramResult::Match(found) => Some(found),
        RegExpProgramResult::NoMatch => None,
        RegExpProgramResult::WorkLimit => panic!("unlimited execution interrupted"),
    }
}

#[test]
fn shared_program_handles_compositions_captures_references_and_reverse_priority() {
    for (source, flags, input, range, captures) in [
        (r"(a+)(b)", "", "!aaab", 1..5, vec![Some(1..4), Some(4..5)]),
        (r"(a|ab)\1", "", "abab", 0..4, vec![Some(0..2)]),
        (r"(?<x>a)\k<x>", "", "aa", 0..2, vec![Some(0..1)]),
        (
            r"(?<=([ab]+)([bc]+))$",
            "u",
            "abc",
            3..3,
            vec![Some(0..1), Some(1..3)],
        ),
        (r"(a|(b))+", "", "aba", 0..3, vec![Some(2..3), None]),
        (r"(?i:a)(?-i:b)", "", "Ab", 0..2, vec![]),
        (r"(?!(a))", "u", "!a", 0..0, vec![None]),
    ] {
        let program = compile(source, flags);
        assert_eq!(program.capture_count(), captures.len());
        let actual = found(&program, &JsString::from(input), 0, false).unwrap();
        assert_eq!(
            (actual.range, actual.captures),
            (range, captures),
            "{source}"
        );
    }
}

#[test]
fn program_retains_utf16_lone_surrogates_sticky_offsets_and_original_context() {
    let emoji = JsString::from("!😀a");
    for flags in ["", "u", "v"] {
        assert_eq!(
            found(
                &compile(r"(?<\u{10400}>a)\k<\u{10400}>", flags),
                &JsString::from("aa"),
                0,
                true
            )
            .unwrap()
            .captures,
            vec![Some(0..1)]
        );
        assert_eq!(
            found(&compile("𠮷", flags), &JsString::from("𠮷a"), 0, true)
                .unwrap()
                .range,
            0..2
        );
        assert_eq!(
            found(
                &compile(r"(?<𐐀>a)\k<𐐀>", flags),
                &JsString::from("aa"),
                0,
                true
            )
            .unwrap()
            .captures,
            vec![Some(0..1)]
        );
    }

    assert_eq!(
        found(&compile(".", ""), &emoji, 2, true).unwrap().range,
        2..3
    );
    for flags in ["u", "v"] {
        assert_eq!(
            found(&compile(".", flags), &emoji, 2, true).unwrap().range,
            1..3
        );
        assert_eq!(
            found(&compile("(?!😀)", flags), &JsString::from("😀"), 0, false)
                .unwrap()
                .range,
            2..2
        );
        assert_eq!(found(&compile("a", flags), &emoji, 1, true), None);
        assert_eq!(
            found(&compile("a", flags), &emoji, 1, false).unwrap().range,
            3..4
        );
        assert_eq!(found(&compile("", flags), &emoji, 5, false), None);
    }
    for source in [r"\B[\b]\B", r"\B[[\b]--[a]]\B"] {
        assert_eq!(
            found(&compile(source, "v"), &JsString::from("\u{8}"), 0, true)
                .unwrap()
                .range,
            0..1
        );
    }
    let lone = JsString::from_code_units(vec![0xd800]);
    let program = RegExpProgram::compile_validated(&lone, &JsString::from("u")).unwrap();
    assert_eq!(found(&program, &lone, 0, true).unwrap().range, 0..1);
    assert_eq!(
        found(&compile(r"(?<=!)😀(?=a)", "u"), &emoji, 0, false)
            .unwrap()
            .range,
        1..3
    );
    assert_eq!(
        found(&compile(r"^a$", "m"), &JsString::from("\na\n"), 0, false)
            .unwrap()
            .range,
        1..2
    );
}

#[test]
fn program_preserves_unicode_set_empty_strings_folded_operations_and_complements() {
    for (source, input, range, capture) in [
        (r"(?=([\d\q{ab|}]))", "!ab", 0..0, 0..0),
        (r"(?=([\d\q{ab|}]))", "ab!", 0..0, 0..2),
        (r"(?=([a-z\q{aaa|aa}]\B))", "aaa!", 0..0, 0..2),
        (r"(?=([\p{ASCII}\q{😀a|}]))", "ſK", 0..0, 0..1),
        (r"(?=([\P{ASCII}\q{ab}]))", "ſK😀", 2..2, 2..4),
        (r"(?=([\W\q{ab}]))", "ſK!", 2..2, 2..3),
        (r"(?=([[a-z]&&\q{A}]))", "A!", 0..0, 0..1),
        (r"(?=([\q{AB}&&\q{ab}]))", "Ab!", 0..0, 0..2),
    ] {
        let actual = found(&compile(source, "iv"), &JsString::from(input), 0, false).unwrap();
        assert_eq!(
            (actual.range, actual.captures),
            (range, vec![Some(capture)]),
            "{source}"
        );
    }
    for source in [
        r"[\q{A}--\q{a}]",
        r"[\q{AB}--\q{ab}]",
        r"[\d\q{\u{D83D}\u{DE00}}]",
    ] {
        assert_eq!(
            found(
                &compile(source, "iv"),
                &JsString::from("AaABab😀"),
                0,
                false
            ),
            None,
            "{source}"
        );
    }
}

#[test]
fn work_limits_interrupt_search_and_negation_without_poisoning_shared_programs() {
    for source in [r"^(a+)+$", r"(?!^(a+)+$)", r"(?=^(a+)+$)", r"a+$"] {
        let program = compile(source, "u");
        let input = JsString::from(("a".repeat(200) + "!").as_str());
        let (result, used) = program.find(&input, 0, false, Some(10_000));
        assert_eq!(result, RegExpProgramResult::WorkLimit, "{source}");
        assert!(used <= 10_000);
        assert!(matches!(
            program.find(&JsString::from("aaaa"), 0, false, None).0,
            RegExpProgramResult::Match(_) | RegExpProgramResult::NoMatch
        ));
    }
    let program = compile("(a+)", "u");
    let input = JsString::from("aaa");
    assert_eq!(
        program.find(&input, 0, true, Some(0)).0,
        RegExpProgramResult::WorkLimit
    );
    assert_eq!(
        program.find(&input, 0, true, None).0,
        program.clone().find(&input, 0, true, None).0
    );
}

#[test]
fn flat_disjunctions_keep_branch_order_without_recursive_linear_depth() {
    let source = "(b)|".repeat(10_000) + r"(?<x>a)\k<x>";
    let program = compile(&source, "d");
    let result = found(&program, &JsString::from("aa"), 0, false).unwrap();
    assert_eq!(result.range, 0..2);
    assert_eq!(result.captures.len(), 10_001);
    assert!(result.captures[..10_000].iter().all(Option::is_none));
    assert_eq!(result.captures[10_000], Some(0..1));
    assert_eq!(
        found(
            &compile("a|ab|abc|abcd|abcde", ""),
            &JsString::from("abcde"),
            0,
            true
        )
        .unwrap()
        .range,
        0..1
    );
    let source = "(".repeat(512) + "a" + &")".repeat(512);
    let error =
        RegExpProgram::compile_validated(&JsString::from(source.as_str()), &JsString::default())
            .unwrap_err();
    assert!(error.contains("too deeply nested"));
}

#[test]
fn unrepresentable_repetition_counts_report_capacity_without_saturating() {
    let optional = found(
        &compile(r"(){0,999999999999999999999999999999}", "d"),
        &JsString::from("a"),
        0,
        true,
    )
    .unwrap();
    assert_eq!(optional.range, 0..0);
    assert_eq!(optional.captures, vec![None]);

    let error = RegExpProgram::compile_validated(
        &JsString::from("(a?){888888888888888888888888888888}"),
        &JsString::default(),
    )
    .unwrap_err();
    assert!(error.contains("exceeds backend representation"));
    assert!(
        RegExpProgram::compile_validated(
            &JsString::from("a{888888888888888888888888888888x"),
            &JsString::default()
        )
        .is_ok()
    );
}

#[test]
fn unicode_eighteen_scripts_and_aliases_use_the_pinned_backend_tables() {
    // Unicode 18 Scripts.txt: Jurchen U+18E00, Proto_Cuneiform U+125A8,
    // Seal U+3D000. These were unassigned in the upstream Unicode 17 tables.
    for (name, point) in [
        ("Jurchen", 0x18e00),
        ("Jurc", 0x18e00),
        ("Proto_Cuneiform", 0x125a8),
        ("Pcun", 0x125a8),
        ("Seal", 0x3d000),
    ] {
        let input = JsString::from(char::from_u32(point).unwrap().to_string().as_str());
        for flags in ["u", "v"] {
            let program = compile(&format!(r"\p{{Script={name}}}"), flags);
            assert_eq!(found(&program, &input, 0, true).unwrap().range, 0..2);
            assert_eq!(found(&program, &JsString::from("a"), 0, false), None);
        }
    }
}

#[test]
fn ordinary_and_unicode_case_matching_follow_independent_canonicalize_rules() {
    let points = [
        's', 'S', 'ſ', 'k', 'K', 'K', 'σ', 'ς', 'Σ', 'µ', 'μ', 'Μ', 'ß', 'ẞ', 'ﬃ', 'ᾀ', 'ᾈ',
    ];
    for flags in ["i", "iu", "iv"] {
        let unicode = flags != "i";
        for source in points {
            let literal = compile(&source.to_string(), flags);
            let class = compile(&format!("[{source}]"), flags);
            let inverse = compile(&format!("[^{source}]"), flags);
            for input in points {
                let input_string = JsString::from(input.to_string().as_str());
                let expected = crate::regexp_canonicalize_character(source as u32, true, unicode)
                    == crate::regexp_canonicalize_character(input as u32, true, unicode);
                assert_eq!(
                    found(&literal, &input_string, 0, true).is_some(),
                    expected,
                    "literal {source} /{flags} input {input}"
                );
                assert_eq!(
                    found(&class, &input_string, 0, true).is_some(),
                    expected,
                    "class {source} /{flags} input {input}"
                );
                assert_eq!(
                    found(&inverse, &input_string, 0, true).is_some(),
                    !expected,
                    "inverse {source} /{flags} input {input}"
                );
            }
        }
    }
}

#[test]
fn word_class_complements_use_the_selected_canonicalization_mode() {
    for flags in ["i", "iu", "iv"] {
        for source in [r"\w", r"[\w]", r"\W", r"[\W]"] {
            let program = compile(source, flags);
            for input in ['s', 'ſ', 'K', 'K', 'σ', '_', '1', '!'] {
                let canonical =
                    crate::regexp_canonicalize_character(input as u32, true, flags != "i");
                let word = matches!(canonical, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a);
                let expected = word != source.contains('W');
                assert_eq!(
                    found(
                        &program,
                        &JsString::from(input.to_string().as_str()),
                        0,
                        true
                    )
                    .is_some(),
                    expected,
                    "{source} /{flags} input {input}"
                );
            }
        }
    }
    let input = JsString::from_code_units(vec![0x78, 0xd800, 0xdc00]);
    let actual = found(&compile(r"(\uD800\uDC00)", "d"), &input, 0, false).unwrap();
    assert_eq!(actual.range, 1..3);
    assert_eq!(actual.captures, vec![Some(1..3)]);
}

#[test]
fn class_string_backspace_and_empty_combined_script_properties() {
    assert_eq!(
        found(
            &compile(r"[\q{\ba|x}]", "v"),
            &JsString::from("\u{8}a"),
            0,
            true
        )
        .unwrap()
        .range,
        0..2
    );
    for flags in ["u", "iu", "v", "iv"] {
        for name in ["Hrkt", "Katakana_Or_Hiragana"] {
            for property in ["Script", "Script_Extensions"] {
                assert_eq!(
                    found(
                        &compile(&format!(r"\p{{{property}={name}}}"), flags),
                        &JsString::from("あアa"),
                        0,
                        false
                    ),
                    None
                );
                assert_eq!(
                    found(
                        &compile(&format!(r"\P{{{property}={name}}}"), flags),
                        &JsString::from("あ"),
                        0,
                        true
                    )
                    .unwrap()
                    .range,
                    0..1
                );
            }
        }
    }
}

#[test]
fn unicode_surrogate_escape_forms_only_join_two_fixed_hex_escapes() {
    for flags in ["u", "v"] {
        for source in [r"\uD83D\u{DE00}", r"\u{D83D}\uDE00", r"\u{D83D}\u{DE00}"] {
            assert_eq!(
                found(&compile(source, flags), &JsString::from("😀"), 0, true),
                None
            );
        }
        assert!(
            found(
                &compile(r"\uD83D\uDE00", flags),
                &JsString::from("😀"),
                0,
                true
            )
            .is_some()
        );
        for source in [r"[\u{D800}\u{DC00}]", r"[\uD800\u{DC00}]"] {
            for point in [0xd800, 0xdc00] {
                assert_eq!(
                    found(
                        &compile(source, flags),
                        &JsString::from_code_units(vec![point]),
                        0,
                        true
                    )
                    .unwrap()
                    .range,
                    0..1
                );
            }
        }
    }
}

#[test]
fn unicode_set_punctuation_and_single_character_string_folding_follow_the_grammar() {
    for source in [r"[!#]", r"[,+]", r"[\q{!#|x}]"] {
        assert!(found(&compile(source, "v"), &JsString::from("!#,+x"), 0, false).is_some());
    }
    for source in [r"[\q{k}]", r"[\q{s|sa|k|ka}]"] {
        // Edition-17 Canonicalize closes one-character q members under folding.
        // V8 24.19 currently misses Kelvin sign in these forms.
        assert_eq!(
            found(&compile(source, "iv"), &JsString::from("K"), 0, true)
                .unwrap()
                .range,
            0..1
        );
    }
}
