#[test]
fn unasserted_zero_count_single_string_classes_preserve_skipped_slots_and_required_class_identity()
{
    let classes: [(&str, &[&str], Predicate); 6] = [
        (r"[\q{|a|ab}]", &["", "a", "ab"], |_| false),
        (r"[\q{ab|abc|c}]", &["ab", "abc", "c"], |_| false),
        (r"[\p{Any}\q{|ab}]", &["", "ab"], |_| true),
        (r"[a-z\q{|S|sa|aX}]", &["", "S", "sa", "aX"], |p| {
            (97..=122).contains(&p)
        }),
        (r"[\q{ab}&&\q{cd}]", &[], |_| false),
        (r"[\p{Any}\q{ab|abc}]", &["ab", "abc"], |_| true),
    ];
    for (zero_before, before_slots, zero_after, after_slots) in [
        (r"((^)|(\b\B)){0}", 3, "", 0),
        ("", 0, r"((\B)*|(\b)+){0}?", 3),
        (r"((\b){1,}|(\B)?){0}", 3, r"(($)|(^)){000,000}?", 3),
        (
            r"(((\b){0})|((\B))){0}",
            5,
            r"((^){99999999999999999999999}){0}",
            2,
        ),
    ] {
        for (class, words, contains) in classes {
            for layout in 0..6 {
                for mode in [0] {
                    for atomic in [false, true] {
                        if atomic && mode != 0 {
                            continue;
                        }
                        let mut tree = if mode == 0 {
                            single_string_capture_tree(layout, class)
                        } else {
                            root_nullable_tree(layout, class, mode == 2)
                        };
                        let mut slots = 0;
                        let body = tree.source(&mut slots);
                        // Huge decimal references use input units + 1 required copies,
                        // preserving the exact upper-minus-lower difference.
                        for (op, minimum, delta, greedy) in [
                            ("", 1, Some(0), true),
                            ("{1}", 1, Some(0), true),
                            ("{1}?", 1, Some(0), false),
                            ("{1,1}", 1, Some(0), true),
                            ("{0001,0001}?", 1, Some(0), false),
                        ] {
                            for (prefix, suffix, _prefix_bits, _suffix_bits) in [
                                ("^", "$", 1, 2),
                                (r"\b", r"\b", 4, 4),
                                (r"\B", r"\B", 8, 8),
                                ("^", r"\b", 1, 4),
                                (r"\b", "$", 4, 2),
                                ("$", "^", 2, 1),
                                (r"\b\B", "", 12, 0),
                                ("", "^$", 0, 3),
                            ] {
                                let source = if atomic {
                                    format!(
                                        "({prefix}){{0}}{zero_before}({}){zero_after}({suffix}){{0}}",
                                        body.replacen(class, &format!("{class}{op}"), 1)
                                    )
                                } else {
                                    format!(
                                        "({prefix}){{0}}{zero_before}(({body}){op}){zero_after}({suffix}){{0}}"
                                    )
                                };
                                for i in [false, true] {
                                    for m in [false, true] {
                                        let plan=crate::RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher::compile_with_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap();
                                        let plan = plan.unwrap_or_else(|| panic!("{source}"));
                                        assert_eq!(
                                            plan.capture_count(),
                                            slots
                                                + before_slots
                                                + after_slots
                                                + if atomic { 3 } else { 4 }
                                        );
                                        for input in [
                                            JsString::from(""),
                                            JsString::from("aaaa"),
                                            JsString::from("!abcab!"),
                                            JsString::from("😀aX😀!"),
                                            JsString::from("ſaXſ!"),
                                            JsString::from("\n\nab\n"),
                                            JsString::from("a"),
                                            JsString::from_code_units(vec![0xd800, 97, 98, 0xdc00]),
                                        ] {
                                            let oracle = Oracle {
                                                input: &input,
                                                contains,
                                                minimum: 1,
                                                maximum: Some(1),
                                                greedy: true,
                                                empty_first: false,
                                                roots: 0,
                                                captures: slots,
                                                empty_slots: &[],
                                                never_slots: &[],
                                                i,
                                                m,
                                                anchors: false,
                                                words: [None, None],
                                            };
                                            let minimum = minimum.min(input.len() + 1);
                                            let maximum = delta.map(|extra| minimum + extra);
                                            let model = AssertedCountedNullableOracle {
                                                oracle: &oracle,
                                                words,
                                                minimum,
                                                maximum,
                                                greedy,
                                                mode,
                                                suffix: 0,
                                            };
                                            for requested in 0..=input.len() + 1 {
                                                for sticky in [false, true] {
                                                    let expected = (|| {
                                                        if requested > input.len() {
                                                            return None;
                                                        }
                                                        let units = input.code_units();
                                                        let mut start = if requested > 0
                                                            && units.get(requested).is_some_and(
                                                                |u| (0xdc00..=0xdfff).contains(u),
                                                            )
                                                            && (0xd800..=0xdbff)
                                                                .contains(&units[requested - 1])
                                                        {
                                                            requested - 1
                                                        } else {
                                                            requested
                                                        };
                                                        loop {
                                                            if asserted_repeat_test_predicates(
                                                                0, &oracle, start,
                                                            ) {
                                                                if let Some((end, last, selected)) =
                                                                    model
                                                                        .walk(start, 0, None, false)
                                                                {
                                                                    let range = start..end;
                                                                    let mut values =
                                                                        vec![None; slots];
                                                                    if atomic {
                                                                        assert_eq!(
                                                                            eval_word_root_tree(
                                                                                &tree,
                                                                                true,
                                                                                start,
                                                                                Some(end),
                                                                                &mut values
                                                                            ),
                                                                            Some(end)
                                                                        );
                                                                    } else if let Some(last) =
                                                                        last.as_ref()
                                                                    {
                                                                        assert_eq!(
                                                                            eval_word_root_tree(
                                                                                &tree,
                                                                                selected,
                                                                                last.start,
                                                                                Some(last.end),
                                                                                &mut values
                                                                            ),
                                                                            Some(last.end)
                                                                        );
                                                                    }
                                                                    let mut captures = vec![None];
                                                                    captures.extend(
                                                                        std::iter::repeat_n(
                                                                            None,
                                                                            before_slots,
                                                                        ),
                                                                    );
                                                                    captures
                                                                        .push(Some(range.clone()));
                                                                    if !atomic {
                                                                        captures.push(last.clone());
                                                                    }
                                                                    captures.extend(values);
                                                                    captures.extend(
                                                                        std::iter::repeat_n(
                                                                            None,
                                                                            after_slots,
                                                                        ),
                                                                    );
                                                                    captures.push(None);
                                                                    return Some((
                                                                        range,
                                                                        last.clone(),
                                                                        selected,
                                                                        captures,
                                                                    ));
                                                                }
                                                            }
                                                            if sticky {
                                                                return None;
                                                            }
                                                            start = oracle.point(start)?.1;
                                                        }
                                                    })(
                                                    );
                                                    let found = plan
                                                        .find_with_captures_and_work(
                                                            &input,
                                                            requested,
                                                            sticky,
                                                            |_| Ok::<(), ()>(()),
                                                        )
                                                        .unwrap();
                                                    let actual = found.map(|found| {
                                                        let caps = (0..plan.capture_count())
                                                            .map(|n| plan.capture_range(n, &found))
                                                            .collect::<Vec<_>>();
                                                        (
                                                            found.range,
                                                            found.last_iteration,
                                                            found.class_selected,
                                                            caps,
                                                        )
                                                    });
                                                    assert_eq!(
                                                        actual, expected,
                                                        "{source:?}, {input:?}, {requested}, sticky={sticky},i={i},m={m}"
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
            }
        }
    }
}

#[test]
fn unasserted_zero_count_single_string_classes_match_native_skipped_predicate_and_capture_tables() {
    for (source, input, start, sticky, i, m, expected, last, selected, captures) in [
        (
            r"(^){0}((^)|(\b\B)){0}((([\q{|a|ab}])))($){0}",
            "",
            0,
            true,
            false,
            true,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
            ],
        ),
        (
            r"(^){0}((^)|(\b\B)){0}((([\q{a|ab}])){1})($){0}",
            "aba",
            0,
            false,
            false,
            false,
            Some(0..2),
            Some(0..2),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..2),
                Some(0..2),
                Some(0..2),
                None,
            ],
        ),
        (
            r"(^){0}((^)|(\b\B)){0}((([\p{Any}\q{|ab}])){1}?)($){0}",
            "aaaa",
            0,
            false,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
            ],
        ),
        (
            r"(^){0}((([a-z\q{|S|sa|aX}])){1,1})((\B)*|(\b)+){0}?($){0}",
            "😀aX",
            1,
            true,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}((([\q{|a|ab}])){0001,0001}?)((\B)*|(\b)+){0}?(\b){0}",
            "ſaX",
            0,
            false,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}((([\q{a|ab}])))((\B)*|(\b)+){0}?(\b){0}",
            "\n\nab\n",
            0,
            false,
            false,
            true,
            Some(2..4),
            Some(2..4),
            true,
            vec![
                None,
                Some(2..4),
                Some(2..4),
                Some(2..4),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}((\b){1,}|(\B)?){0}((([\p{Any}\q{|ab}])){1})(($)|(^)){000,000}?(\b){0}",
            "a",
            0,
            true,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}((\b){1,}|(\B)?){0}((([a-z\q{|S|sa|aX}])){1}?)(($)|(^)){000,000}?(\b){0}",
            "!ab!",
            0,
            false,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\B){0}((\b){1,}|(\B)?){0}((([\q{|a|ab}])){1,1})(($)|(^)){000,000}?(\B){0}",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\B){0}(((\b){0})|((\B))){0}((([\q{a|ab}])){0001,0001}?)((^){99999999999999999999999}){0}(\B){0}",
            "aba",
            0,
            true,
            false,
            false,
            Some(0..2),
            Some(0..2),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..2),
                Some(0..2),
                Some(0..2),
                None,
                None,
                None,
            ],
        ),
        (
            r"(\B){0}(((\b){0})|((\B))){0}((([\p{Any}\q{|ab}])))((^){99999999999999999999999}){0}(\B){0}",
            "aaaa",
            0,
            false,
            false,
            true,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
                None,
                None,
            ],
        ),
        (
            r"(\B){0}(((\b){0})|((\B))){0}((([a-z\q{|S|sa|aX}])){1})((^){99999999999999999999999}){0}(\B){0}",
            "😀aX",
            1,
            false,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((^)|(\b\B)){0}((([\q{|a|ab}])){1}?)(\b){0}",
            "ſaX",
            0,
            true,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
            ],
        ),
        (
            r"(^){0}((^)|(\b\B)){0}((([\q{a|ab}])){1,1})(\b){0}",
            "\n\nab\n",
            0,
            false,
            false,
            false,
            Some(2..4),
            Some(2..4),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(2..4),
                Some(2..4),
                Some(2..4),
                None,
            ],
        ),
        (
            r"(^){0}((^)|(\b\B)){0}((([\p{Any}\q{|ab}])){0001,0001}?)(\b){0}",
            "a",
            0,
            false,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
            ],
        ),
        (
            r"(^){0}((([a-z\q{|S|sa|aX}])))((\B)*|(\b)+){0}?(\b){0}",
            "!ab!",
            0,
            true,
            true,
            true,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((([\q{|a|ab}])){1})((\B)*|(\b)+){0}?($){0}",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((([\q{a|ab}])){1}?)((\B)*|(\b)+){0}?($){0}",
            "aba",
            0,
            false,
            false,
            false,
            Some(0..2),
            Some(0..2),
            true,
            vec![
                None,
                Some(0..2),
                Some(0..2),
                Some(0..2),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((\b){1,}|(\B)?){0}((([\p{Any}\q{|ab}])){1,1})(($)|(^)){000,000}?($){0}",
            "aaaa",
            0,
            true,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((\b){1,}|(\B)?){0}((([a-z\q{|S|sa|aX}])){0001,0001}?)(($)|(^)){000,000}?($){0}",
            "😀aX",
            1,
            false,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}((\b){1,}|(\B)?){0}((([\q{|a|ab}])))(($)|(^)){000,000}?(\b){0}",
            "ſaX",
            0,
            false,
            false,
            true,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}(((\b){0})|((\B))){0}((([\q{a|ab}])){1})((^){99999999999999999999999}){0}(\b){0}",
            "\n\nab\n",
            0,
            true,
            false,
            false,
            None,
            None,
            false,
            vec![
                None, None, None, None, None, None, None, None, None, None, None, None,
            ],
        ),
        (
            r"(\b){0}(((\b){0})|((\B))){0}((([\p{Any}\q{|ab}])){1}?)((^){99999999999999999999999}){0}(\b){0}",
            "a",
            0,
            false,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}(((\b){0})|((\B))){0}((([a-z\q{|S|sa|aX}])){1,1})((^){99999999999999999999999}){0}(\b){0}",
            "!ab!",
            0,
            false,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
            ],
        ),
        (
            r"(\B){0}((^)|(\b\B)){0}((([\q{|a|ab}])){0001,0001}?)(\B){0}",
            "",
            0,
            true,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
            ],
        ),
        (
            r"(\B){0}((^)|(\b\B)){0}((([\q{a|ab}])))(\B){0}",
            "aba",
            0,
            false,
            false,
            true,
            Some(0..2),
            Some(0..2),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..2),
                Some(0..2),
                Some(0..2),
                None,
            ],
        ),
        (
            r"(\B){0}((^)|(\b\B)){0}((([\p{Any}\q{|ab}])){1})(\B){0}",
            "aaaa",
            0,
            false,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
            ],
        ),
        (
            r"(\B){0}((([a-z\q{|S|sa|aX}])){1}?)((\B)*|(\b)+){0}?(\B){0}",
            "😀aX",
            1,
            true,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((([\q{|a|ab}])){1,1})((\B)*|(\b)+){0}?(\b){0}",
            "ſaX",
            0,
            false,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((([\q{a|ab}])){0001,0001}?)((\B)*|(\b)+){0}?(\b){0}",
            "\n\nab\n",
            0,
            false,
            false,
            false,
            Some(2..4),
            Some(2..4),
            true,
            vec![
                None,
                Some(2..4),
                Some(2..4),
                Some(2..4),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((\b){1,}|(\B)?){0}((([\p{Any}\q{|ab}])))(($)|(^)){000,000}?(\b){0}",
            "a",
            0,
            true,
            false,
            true,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((\b){1,}|(\B)?){0}((([a-z\q{|S|sa|aX}])){1})(($)|(^)){000,000}?(\b){0}",
            "!ab!",
            0,
            false,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}((\b){1,}|(\B)?){0}((([\q{|a|ab}])){1}?)(($)|(^)){000,000}?($){0}",
            "",
            0,
            false,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}(((\b){0})|((\B))){0}((([\q{a|ab}])){1,1})((^){99999999999999999999999}){0}($){0}",
            "aba",
            0,
            true,
            false,
            false,
            Some(0..2),
            Some(0..2),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..2),
                Some(0..2),
                Some(0..2),
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}(((\b){0})|((\B))){0}((([\p{Any}\q{|ab}])){0001,0001}?)((^){99999999999999999999999}){0}($){0}",
            "aaaa",
            0,
            false,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
                None,
                None,
            ],
        ),
        (
            r"(^){0}(((\b){0})|((\B))){0}((([a-z\q{|S|sa|aX}])))((^){99999999999999999999999}){0}($){0}",
            "😀aX",
            1,
            false,
            true,
            true,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
            ],
        ),
        (
            r"(\b){0}((^)|(\b\B)){0}((([\q{|a|ab}])){1})(\b){0}",
            "ſaX",
            0,
            true,
            false,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
            ],
        ),
        (
            r"(\b){0}((^)|(\b\B)){0}((([\q{a|ab}])){1}?)(\b){0}",
            "\n\nab\n",
            0,
            false,
            false,
            false,
            Some(2..4),
            Some(2..4),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(2..4),
                Some(2..4),
                Some(2..4),
                None,
            ],
        ),
        (
            r"(\b){0}((^)|(\b\B)){0}((([\p{Any}\q{|ab}])){1,1})(\b){0}",
            "a",
            0,
            false,
            false,
            false,
            Some(0..1),
            Some(0..1),
            true,
            vec![
                None,
                None,
                None,
                None,
                Some(0..1),
                Some(0..1),
                Some(0..1),
                None,
            ],
        ),
        (
            r"(\b){0}((([a-z\q{|S|sa|aX}])){0001,0001}?)((\B)*|(\b)+){0}?(\b){0}",
            "!ab!",
            0,
            true,
            true,
            false,
            Some(0..0),
            Some(0..0),
            true,
            vec![
                None,
                Some(0..0),
                Some(0..0),
                Some(0..0),
                None,
                None,
                None,
                None,
            ],
        ),
    ] {
        let p=crate::RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher::compile_with_work(&JsString::from(source),i,m,|_|Ok::<(),()>(())).unwrap().unwrap_or_else(||panic!("{source}"));
        assert_eq!(p.capture_count(), captures.len());
        let f =
            p.find_with_captures_and_work(&JsString::from(input), start, sticky, |_| {
                Ok::<(), ()>(())
            })
            .unwrap();
        assert_eq!(
            f.as_ref().map(|f| f.range.clone()),
            expected,
            "{source}, {input}"
        );
        if let Some(f) = f {
            assert_eq!(f.last_iteration, last, "{source}, {input}");
            assert_eq!(f.class_selected, selected, "{source}, {input}");
            assert_eq!(
                (0..p.capture_count())
                    .map(|n| p.capture_range(n, &f))
                    .collect::<Vec<_>>(),
                captures,
                "{source}, {input}"
            );
        }
    }
}

#[test]
fn unasserted_zero_count_single_classes_keep_required_empty_and_outer_point_captures_flat() {
    use crate::RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher as P;
    let source = JsString::from(
        format!(
            r"{}\b{}([\q{{}}]){{1}}",
            "(".repeat(20_000),
            "){0}".repeat(20_000)
        )
        .as_str(),
    );
    let original = P::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    let p = original.clone();
    drop(original);
    let f = p
        .find_with_captures_and_work(&JsString::from(""), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(p.capture_count(), 20_001);
    assert_eq!(f.range, 0..0);
    assert_eq!(f.last_iteration, Some(0..0));
    assert!(f.class_selected);
    for n in 0..20_000 {
        assert_eq!(p.capture_range(n, &f), None);
    }
    assert_eq!(p.capture_range(20_000, &f), Some(0..0));
    let p = P::compile_with_work(
        &JsString::from(r"((\b){0})([\q{|a|ab}])((^){0})"),
        false,
        false,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
    .unwrap();
    for (text, end) in [("ab!", 2), ("x", 0)] {
        let f = p
            .find_with_captures_and_work(&JsString::from(text), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(f.range, 0..end);
        assert_eq!(f.last_iteration, Some(0..end));
        assert!(f.class_selected);
        assert_eq!(
            (0..5).map(|n| p.capture_range(n, &f)).collect::<Vec<_>>(),
            vec![Some(0..0), None, Some(0..end), Some(end..end), None]
        );
    }
    let word = "x".repeat(20_000);
    let source = JsString::from(
        format!(r"((\b)|(\B)){{0}}()([\p{{Any}}\q{{{word}|{word}x}}]){{1,1}}()").as_str(),
    );
    let p = P::compile_with_work(&source, false, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    for (text, end) in [("x".repeat(40_000), 20_001), ("x".to_owned(), 1)] {
        let f = p
            .find_with_captures_and_work(&JsString::from(text.as_str()), 0, true, |_| {
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(f.range, 0..end);
        assert_eq!(p.capture_range(4, &f), Some(0..end));
        assert_eq!(p.capture_range(5, &f), Some(end..end));
    }
}
#[test]
fn unasserted_zero_count_single_class_callbacks_recover_and_unproved_paths_stay_closed() {
    use crate::{
        RegExpUnicodeMatchError as FindError, RegExpUnicodeSetCompileError as CompileError,
        RegExpUnicodeUnassertedZeroCountSingleStringClassCaptureMatcher as P,
    };
    for source in [
        r"((\b)|(\B)){0}([\q{|a|ab}])",
        r"((\b)+|(\B)*){0}()(([\q{|😀|😀a|aX}])){0001,0001}?",
        r"([\p{Any}\q{ab}]){1}((^)|($)){0}",
        r"((\b){0})([a-z\q{|S|sa|aX}])((^){0})",
    ] {
        let source = JsString::from(source);
        let mut work = Vec::new();
        let p = P::compile_with_work(&source, true, true, |n| {
            work.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(P::compile_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(CompileError::Work(n)) if n==fail)
            );
            assert_eq!(seen, work[..fail]);
        }
        let mut recovered = Vec::new();
        assert!(
            P::compile_with_work(&source, true, true, |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(recovered, work);
        for text in ["", "aba", "!ab!", "😀aX", "ſaX"] {
            for sticky in [false, true] {
                let input = JsString::from(text);
                let mut work = Vec::new();
                let expected = p
                    .find_with_captures_and_work(&input, 0, sticky, |n| {
                        work.push(n);
                        Ok::<(), usize>(())
                    })
                    .unwrap();
                for fail in 1..=work.len() {
                    let mut seen = Vec::new();
                    assert!(
                        matches!(p.find_with_captures_and_work(&input,0,sticky,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(FindError::Work(n)) if n==fail)
                    );
                    assert_eq!(seen, work[..fail]);
                }
                assert_eq!(
                    p.find_with_captures_and_work(&input, 0, sticky, |_| Ok::<(), usize>(()))
                        .unwrap(),
                    expected
                );
            }
        }
    }
    for source in [
        r"([\q{|a|ab}])",
        r"^((\b)|(\B)){0}([\q{|a|ab}])",
        r"((\b)|(\B)){0}([\q{|a|ab}])$",
        r"((\b)|(\B)){0}(^([\q{|a|ab}])){1}",
        r"((\b)|(\B)){0}([\q{|a|ab}]){0}",
        r"((\b)|(\B)){0}([\q{|a|ab}])*",
        r"((\b)|(\B)){0}([\q{|a|ab}]){2}",
        r"((([\q{|a|ab}]){1})){0}",
        r"((\b)|(\B)){0}([\q{|a|ab}])\1",
        r"((?=a)){0}([\q{|a|ab}])",
        r"((\b)|(\B)){0}([\q{ab}&&\p{RGI_Emoji}]){1}",
        r"([\p{RGI_Emoji}]){0}([\q{|a|ab}])",
    ] {
        assert!(
            P::compile_with_work(&JsString::from(source), false, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
