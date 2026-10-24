#[test]
fn exact_single_class_spine_plain_captures_follow_independent_oracle() {
    let classes: [(&str, &[&str], Predicate); 27] = [
        (r"[\q{a|ab|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[\q{ab}\q{a|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[x\q{a|ab}]", &["a", "ab"], |p| p == 120),
        (r"[a-z\q{ab|abc}]", &["ab", "abc"], |p| {
            (97..=122).contains(&p)
        }),
        (r"[[\q{a|ab}][x\q{abc}]]", &["a", "ab", "abc"], |p| p == 120),
        (r"[\q{a|ab|abc}--\q{ab}]", &["a", "abc"], |_| false),
        (r"[[\q{a|ab}x]&&[\q{ab|x}]]", &["ab", "x"], |_| false),
        (r"[\q{a|b|ab}--[b-z]]", &["a", "ab"], |_| false),
        (r"[\q{😀|😀a|a}]", &["😀", "😀a", "a"], |_| false),
        (r"[\q{\u{10428}|\u{10428}a}]", &["𐐨", "𐐨a"], |_| false),
        (r"[\q{s|sa|k|ka}]", &["s", "sa", "k", "ka"], |_| false),
        (r"[\q{ab}--\q{ab}]", &[], |_| false),
        (r"[\q{|a|ab}--\q{}]", &["a", "ab"], |_| false),
        (r"[\q{a|ab}&&\q{ab}]", &["ab"], |_| false),
        (r"[\q{|a|ab}]", &["", "a", "ab"], |_| false),
        (r"[\q{ab}\q{}]", &["", "ab"], |_| false),
        (r"[x\q{|ab}]", &["", "ab"], |p| p == 120),
        (r"[a-z\q{|ab}]", &["", "ab"], |p| (97..=122).contains(&p)),
        (r"[[\q{|ab}][x\q{a}]]", &["", "ab", "a"], |p| p == 120),
        (r"[\q{|a|ab}--\q{a}]", &["", "ab"], |_| false),
        (r"[[\q{|a|ab}x]&&[\q{|ab|x}]]", &["", "ab", "x"], |_| false),
        (r"[\q{|a|ab}--[a-z]]", &["", "ab"], |_| false),
        (r"[\q{|😀|😀a}]", &["", "😀", "😀a"], |_| false),
        (r"[\q{|\u{10428}|\u{10428}a}]", &["", "𐐨", "𐐨a"], |_| false),
        (r"[\q{|s|sa|k|ka}]", &["", "s", "sa", "k", "ka"], |_| false),
        (r"[\q{}]", &[""], |_| false),
        (r"[\q{|ab}--\q{ab}]", &[""], |_| false),
    ];
    for (class, words, contains) in classes {
        for layout in 0..6 {
            let mut tree = single_string_capture_tree(layout, class);
            let mut count = 0;
            let source = tree.source(&mut count);
            let source = source.replacen(class, &format!("{class}{{0001,1}}?"), 1);
            let source = format!("(?:{source}){{1}}");
            for i in [false, true] {
                for m in [false, true] {
                    let plan =
                        super::RegExpUnicodeSingleStringClassCaptureMatcher::compile_exact_single_counts_with_work(
                            &JsString::from(source.as_str()),
                            i,
                            m,
                            |_| Ok::<(), ()>(()),
                        )
                        .unwrap()
                        .unwrap();
                    assert_eq!(plan.capture_count(), count);
                    for input in [
                        JsString::from(""),
                        JsString::from("!abcabx!"),
                        JsString::from("!ABCa!"),
                        JsString::from("!😀a😀!"),
                        JsString::from("!ſaKa"),
                        JsString::from("!𐐀a𐐨!"),
                        JsString::from("!\r\nabc"),
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
                            captures: count,
                            empty_slots: &[],
                            never_slots: &[],
                            i,
                            m,
                            anchors: false,
                            words: [None, None],
                        };
                        for requested in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                let expected = if requested > input.len() {
                                    None
                                } else {
                                    let units = input.code_units();
                                    let mut at = if requested > 0
                                        && units
                                            .get(requested)
                                            .is_some_and(|u| (0xdc00..=0xdfff).contains(u))
                                        && (0xd800..=0xdbff).contains(&units[requested - 1])
                                    {
                                        requested - 1
                                    } else {
                                        requested
                                    };
                                    loop {
                                        if let Some(end) = word_root_end(&oracle, at, words) {
                                            let mut captures = vec![None; count];
                                            assert_eq!(
                                                eval_word_root_tree(
                                                    &tree,
                                                    true,
                                                    at,
                                                    Some(end),
                                                    &mut captures
                                                ),
                                                Some(end)
                                            );
                                            break Some(Expected {
                                                range: at..end,
                                                captures,
                                            });
                                        }
                                        if sticky {
                                            break None;
                                        }
                                        let Some((_, next)) = oracle.point(at) else {
                                            break None;
                                        };
                                        at = next;
                                    }
                                };
                                let found = plan
                                    .find_with_captures_and_work(&input, requested, sticky, |_| {
                                        Ok::<(), ()>(())
                                    })
                                    .unwrap();
                                let actual = found.map(|range| Expected {
                                    captures: (0..count)
                                        .map(|slot| plan.capture_range(slot, &range))
                                        .collect(),
                                    range,
                                });
                                assert_eq!(
                                    actual, expected,
                                    "{source:?}, {input:?}, {requested}, sticky={sticky}, i={i}, m={m}"
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
fn exact_single_class_spine_canonical_captures_follow_independent_oracle() {
    let classes: [(&str, &[&str], Predicate); 27] = [
        (r"[\q{a|ab|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[\q{ab}\q{a|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[x\q{a|ab}]", &["a", "ab"], |p| p == 120),
        (r"[a-z\q{ab|abc}]", &["ab", "abc"], |p| {
            (97..=122).contains(&p)
        }),
        (r"[[\q{a|ab}][x\q{abc}]]", &["a", "ab", "abc"], |p| p == 120),
        (r"[\q{a|ab|abc}--\q{ab}]", &["a", "abc"], |_| false),
        (r"[[\q{a|ab}x]&&[\q{ab|x}]]", &["ab", "x"], |_| false),
        (r"[\q{a|b|ab}--[b-z]]", &["a", "ab"], |_| false),
        (r"[\q{😀|😀a|a}]", &["😀", "😀a", "a"], |_| false),
        (r"[\q{\u{10428}|\u{10428}a}]", &["𐐨", "𐐨a"], |_| false),
        (r"[\q{s|sa|k|ka}]", &["s", "sa", "k", "ka"], |_| false),
        (r"[\q{ab}--\q{ab}]", &[], |_| false),
        (r"[\q{|a|ab}--\q{}]", &["a", "ab"], |_| false),
        (r"[\q{a|ab}&&\q{ab}]", &["ab"], |_| false),
        (r"[\q{|a|ab}]", &["", "a", "ab"], |_| false),
        (r"[\q{ab}\q{}]", &["", "ab"], |_| false),
        (r"[x\q{|ab}]", &["", "ab"], |p| p == 120),
        (r"[a-z\q{|ab}]", &["", "ab"], |p| (97..=122).contains(&p)),
        (r"[[\q{|ab}][x\q{a}]]", &["", "ab", "a"], |p| p == 120),
        (r"[\q{|a|ab}--\q{a}]", &["", "ab"], |_| false),
        (r"[[\q{|a|ab}x]&&[\q{|ab|x}]]", &["", "ab", "x"], |_| false),
        (r"[\q{|a|ab}--[a-z]]", &["", "ab"], |_| false),
        (r"[\q{|😀|😀a}]", &["", "😀", "😀a"], |_| false),
        (r"[\q{|\u{10428}|\u{10428}a}]", &["", "𐐨", "𐐨a"], |_| false),
        (r"[\q{|s|sa|k|ka}]", &["", "s", "sa", "k", "ka"], |_| false),
        (r"[\q{}]", &[""], |_| false),
        (r"[\q{|ab}--\q{ab}]", &[""], |_| false),
    ];
    for (class, words, contains) in classes {
        for (prefix, suffix, start_anchor, end_anchor, start_word, end_word) in [
            ("^", "", true, false, None, None),
            ("", "$", false, true, None, None),
            ("^", "$", true, true, None, None),
            (r"\b", "", false, false, Some(true), None),
            ("", r"\b", false, false, None, Some(true)),
            ("", r"\B", false, false, None, Some(false)),
            (r"\b", r"\B", false, false, Some(true), Some(false)),
            (r"\B", r"\b", false, false, Some(false), Some(true)),
            (r"^\b", r"\b$", true, true, Some(true), Some(true)),
            (r"^\B", r"\B$", true, true, Some(false), Some(false)),
        ] {
            for framing in 0..3 {
                for layout in 0..6 {
                    let middle = single_string_capture_tree(layout, class);
                    let root = BoundaryTree::Sequence(vec![
                        BoundaryTree::empty_capture(),
                        middle,
                        BoundaryTree::empty_capture(),
                    ]);
                    let mut tree =
                        match framing {
                            0 => root,
                            1 => BoundaryTree::capture(root),
                            _ => BoundaryTree::capture(BoundaryTree::First(vec![
                                BoundaryTree::capture(root),
                            ])),
                        };
                    let mut count = 0;
                    let wrapped = tree.source(&mut count);
                    let (begin, closing) = match framing {
                        0 => (0, 0),
                        1 => (1, 1),
                        _ => (5, 3),
                    };
                    let end = wrapped.len() - closing;
                    let source = format!(
                        "{}({prefix}){}({suffix}){}",
                        &wrapped[..begin],
                        &wrapped[begin + 2..end - 2],
                        &wrapped[end..]
                    );
                    let source = source.replacen(class, &format!("{class}{{0001,1}}?"), 1);
                    let source = format!("(?:{source}){{1}}");
                    for i in [false, true] {
                        for m in [false, true] {
                            let plan =
                        super::RegExpUnicodeSingleStringClassCaptureMatcher::compile_exact_single_counts_with_work(
                            &JsString::from(source.as_str()),
                            i,
                            m,
                            |_| Ok::<(), ()>(()),
                        )
                        .unwrap()
                        .unwrap();
                            assert_eq!(plan.capture_count(), count);
                            for input in [
                                JsString::from(""),
                                JsString::from("!abcabx!"),
                                JsString::from("!ABCa!"),
                                JsString::from("!😀a😀!"),
                                JsString::from("!ſaKa"),
                                JsString::from("!𐐀a𐐨!"),
                                JsString::from("!\r\nabc"),
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
                                    captures: count,
                                    empty_slots: &[],
                                    never_slots: &[],
                                    i,
                                    m,
                                    anchors: end_anchor,
                                    words: [start_word, end_word],
                                };
                                for requested in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = if requested > input.len() {
                                            None
                                        } else {
                                            let units = input.code_units();
                                            let mut at = if requested > 0
                                                && units
                                                    .get(requested)
                                                    .is_some_and(|u| (0xdc00..=0xdfff).contains(u))
                                                && (0xd800..=0xdbff).contains(&units[requested - 1])
                                            {
                                                requested - 1
                                            } else {
                                                requested
                                            };
                                            loop {
                                                let at_line = !start_anchor
                                                    || at == 0
                                                    || (m
                                                        && Oracle::line(
                                                            input.code_units()[at - 1],
                                                        ));
                                                let at_word = start_word
                                                    .is_none_or(|word| oracle.boundary(at) == word);
                                                if let Some(end) = (at_line && at_word)
                                                    .then(|| word_root_end(&oracle, at, words))
                                                    .flatten()
                                                {
                                                    let mut captures = vec![None; count];
                                                    assert_eq!(
                                                        eval_word_root_tree(
                                                            &tree,
                                                            true,
                                                            at,
                                                            Some(end),
                                                            &mut captures
                                                        ),
                                                        Some(end)
                                                    );
                                                    break Some(Expected {
                                                        range: at..end,
                                                        captures,
                                                    });
                                                }
                                                if sticky {
                                                    break None;
                                                }
                                                let Some((_, next)) = oracle.point(at) else {
                                                    break None;
                                                };
                                                at = next;
                                            }
                                        };
                                        let found = plan
                                            .find_with_captures_and_work(
                                                &input,
                                                requested,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = found.map(|range| Expected {
                                            captures: (0..count)
                                                .map(|slot| plan.capture_range(slot, &range))
                                                .collect(),
                                            range,
                                        });
                                        assert_eq!(
                                            actual, expected,
                                            "{source:?}, {input:?}, {requested}, sticky={sticky}, i={i}, m={m}"
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

#[test]
fn exact_single_class_spine_reversed_captures_follow_independent_oracle() {
    let classes: [(&str, &[&str], Predicate); 32] = [
        (r"[\q{a|ab|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[\q{ab}\q{a|abc}]", &["a", "ab", "abc"], |_| false),
        (r"[x\q{a|ab}]", &["a", "ab"], |p| p == 120),
        (r"[a-z\q{ab|abc}]", &["ab", "abc"], |p| {
            (97..=122).contains(&p)
        }),
        (r"[[\q{a|ab}][x\q{abc}]]", &["a", "ab", "abc"], |p| p == 120),
        (r"[\q{a|ab|abc}--\q{ab}]", &["a", "abc"], |_| false),
        (r"[[\q{a|ab}x]&&[\q{ab|x}]]", &["ab", "x"], |_| false),
        (r"[\q{a|b|ab}--[b-z]]", &["a", "ab"], |_| false),
        (r"[\q{😀|😀a|a}]", &["😀", "😀a", "a"], |_| false),
        (r"[\q{\u{10428}|\u{10428}a}]", &["𐐨", "𐐨a"], |_| false),
        (r"[\q{s|sa|k|ka}]", &["s", "sa", "k", "ka"], |_| false),
        (r"[\q{ab}--\q{ab}]", &[], |_| false),
        (r"[\q{|a|ab}--\q{}]", &["a", "ab"], |_| false),
        (r"[\q{a|ab}&&\q{ab}]", &["ab"], |_| false),
        (r"[\q{|a|ab}]", &["", "a", "ab"], |_| false),
        (r"[\q{ab}\q{}]", &["", "ab"], |_| false),
        (r"[x\q{|ab}]", &["", "ab"], |p| p == 120),
        (r"[a-z\q{|ab}]", &["", "ab"], |p| (97..=122).contains(&p)),
        (r"[[\q{|ab}][x\q{a}]]", &["", "ab", "a"], |p| p == 120),
        (r"[\q{|a|ab}--\q{a}]", &["", "ab"], |_| false),
        (r"[[\q{|a|ab}x]&&[\q{|ab|x}]]", &["", "ab", "x"], |_| false),
        (r"[\q{|a|ab}--[a-z]]", &["", "ab"], |_| false),
        (r"[\q{|😀|😀a}]", &["", "😀", "😀a"], |_| false),
        (r"[\q{|\u{10428}|\u{10428}a}]", &["", "𐐨", "𐐨a"], |_| false),
        (r"[\q{|s|sa|k|ka}]", &["", "s", "sa", "k", "ka"], |_| false),
        (r"[\q{}]", &[""], |_| false),
        (r"[\q{|ab}--\q{ab}]", &[""], |_| false),
        (r"[\q{a|\n|\na}]", &["a", "\n", "\na"], |_| false),
        (r"[\q{|a|\n|\na}]", &["", "a", "\n", "\na"], |_| false),
        (r"[\n\q{a|\na}]", &["a", "\na"], |p| p == 10),
        (r"[\q{|a|a\n}--\q{a}]", &["", "a\n"], |_| false),
        (r"[[\q{a|\na}\n]&&[\q{|\n|\na}]]", &["\n", "\na"], |_| false),
    ];
    for (class, words, contains) in classes {
        for (
            prefix,
            suffix,
            start_anchor,
            end_anchor,
            start_end_anchor,
            end_start_anchor,
            start_word,
            end_word,
        ) in [
            ("$", "", false, false, true, false, None, None),
            ("", "^", false, false, false, true, None, None),
            ("$", "^", false, false, true, true, None, None),
            ("^$", "^$", true, true, true, true, None, None),
            (r"$\b", "", false, false, true, false, Some(true), None),
            (r"$\B", "", false, false, true, false, Some(false), None),
            ("", r"^\b", false, false, false, true, None, Some(true)),
            ("", r"^\B", false, false, false, true, None, Some(false)),
            ("$", r"^\B", false, false, true, true, None, Some(false)),
            (
                r"^$\b",
                r"^\B$",
                true,
                true,
                true,
                true,
                Some(true),
                Some(false),
            ),
            (
                r"$\B",
                r"^\b",
                false,
                false,
                true,
                true,
                Some(false),
                Some(true),
            ),
            (r"\b$", "^", false, false, true, true, Some(true), None),
            (r"$\B\B", "^^", false, false, true, true, Some(false), None),
            ("$", "^$", false, true, true, true, None, None),
            ("^$", "^", true, false, true, true, None, None),
            ("", r"$^", false, true, false, true, None, None),
            ("$^", "", true, false, true, false, None, None),
            (
                r"$\b\b",
                r"^\b\b",
                false,
                false,
                true,
                true,
                Some(true),
                Some(true),
            ),
        ] {
            for framing in 0..3 {
                for layout in 0..6 {
                    let middle = single_string_capture_tree(layout, class);
                    let root = BoundaryTree::Sequence(vec![
                        BoundaryTree::empty_capture(),
                        middle,
                        BoundaryTree::empty_capture(),
                    ]);
                    let mut tree =
                        match framing {
                            0 => root,
                            1 => BoundaryTree::capture(root),
                            _ => BoundaryTree::capture(BoundaryTree::First(vec![
                                BoundaryTree::capture(root),
                            ])),
                        };
                    let mut count = 0;
                    let wrapped = tree.source(&mut count);
                    let (begin, closing) = match framing {
                        0 => (0, 0),
                        1 => (1, 1),
                        _ => (5, 3),
                    };
                    let end = wrapped.len() - closing;
                    let source = format!(
                        "{}({prefix}){}({suffix}){}",
                        &wrapped[..begin],
                        &wrapped[begin + 2..end - 2],
                        &wrapped[end..]
                    );
                    let source = source.replacen(class, &format!("{class}{{0001,1}}?"), 1);
                    let source = format!("(?:{source}){{1}}");
                    for i in [false, true] {
                        for m in [false, true] {
                            let plan =
                        super::RegExpUnicodeSingleStringClassCaptureMatcher::compile_exact_single_counts_with_work(
                            &JsString::from(source.as_str()),
                            i,
                            m,
                            |_| Ok::<(), ()>(()),
                        )
                        .unwrap()
                        .unwrap();
                            assert_eq!(plan.capture_count(), count);
                            for input in [
                                JsString::from(""),
                                JsString::from("!abcabx!"),
                                JsString::from("!ABCa!"),
                                JsString::from("!😀a😀!"),
                                JsString::from("!ſaKa"),
                                JsString::from("!𐐀a𐐨!"),
                                JsString::from("!\r\nabc"),
                                JsString::from("\na!"),
                                JsString::from("a\nx"),
                                JsString::from("\n\na\n"),
                                JsString::from("\u{2028}a\u{2028}"),
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
                                    captures: count,
                                    empty_slots: &[],
                                    never_slots: &[],
                                    i,
                                    m,
                                    anchors: end_anchor,
                                    words: [start_word, end_word],
                                };
                                for requested in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = if requested > input.len() {
                                            None
                                        } else {
                                            let units = input.code_units();
                                            let mut at = if requested > 0
                                                && units
                                                    .get(requested)
                                                    .is_some_and(|u| (0xdc00..=0xdfff).contains(u))
                                                && (0xd800..=0xdbff).contains(&units[requested - 1])
                                            {
                                                requested - 1
                                            } else {
                                                requested
                                            };
                                            loop {
                                                let at_line = !start_anchor
                                                    || at == 0
                                                    || (m
                                                        && Oracle::line(
                                                            input.code_units()[at - 1],
                                                        ));
                                                let at_line = at_line
                                                    && (!start_end_anchor
                                                        || at == input.len()
                                                        || (m
                                                            && input
                                                                .code_units()
                                                                .get(at)
                                                                .is_some_and(|u| {
                                                                    Oracle::line(*u)
                                                                })));
                                                let at_word = start_word
                                                    .is_none_or(|word| oracle.boundary(at) == word);
                                                if let Some(end) = (at_line && at_word)
                                                    .then(|| {
                                                        reversed_line_word_end(
                                                            &oracle,
                                                            at,
                                                            words,
                                                            end_start_anchor,
                                                        )
                                                    })
                                                    .flatten()
                                                {
                                                    let mut captures = vec![None; count];
                                                    assert_eq!(
                                                        eval_word_root_tree(
                                                            &tree,
                                                            true,
                                                            at,
                                                            Some(end),
                                                            &mut captures
                                                        ),
                                                        Some(end)
                                                    );
                                                    break Some(Expected {
                                                        range: at..end,
                                                        captures,
                                                    });
                                                }
                                                if sticky {
                                                    break None;
                                                }
                                                let Some((_, next)) = oracle.point(at) else {
                                                    break None;
                                                };
                                                at = next;
                                            }
                                        };
                                        let found = plan
                                            .find_with_captures_and_work(
                                                &input,
                                                requested,
                                                sticky,
                                                |_| Ok::<(), ()>(()),
                                            )
                                            .unwrap();
                                        let actual = found.map(|range| Expected {
                                            captures: (0..count)
                                                .map(|slot| plan.capture_range(slot, &range))
                                                .collect(),
                                            range,
                                        });
                                        assert_eq!(
                                            actual, expected,
                                            "{source:?}, {input:?}, {requested}, sticky={sticky}, i={i}, m={m}"
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
#[test]
fn exact_single_class_spine_fixed_ranges_deep_counts_and_cancellation_keep_full_proofs() {
    use super::RegExpUnicodeSingleStringClassCaptureMatcher as Plan;
    for (source, input, i, m, start, range, captures) in [
        (
            r"()([\q{|a|ab}]){1}()",
            "!",
            false,
            false,
            0,
            0..0,
            vec![Some(0..0), Some(0..0), Some(0..0)],
        ),
        (
            r"(\b)([\q{a|ab}]{1})(\B)",
            "ab!",
            false,
            false,
            0,
            0..1,
            vec![Some(0..0), Some(0..1), Some(1..1)],
        ),
        (
            r"(^)([\q{|a|ab}]){0001,1}?($)",
            "ab",
            false,
            false,
            0,
            0..2,
            vec![Some(0..0), Some(0..2), Some(2..2)],
        ),
        (
            r"($)([\q{\n|\na}]){1}(^)",
            "x\nab",
            false,
            true,
            1,
            1..2,
            vec![Some(1..1), Some(1..2), Some(2..2)],
        ),
        (
            r"(\B)([\q{😀|😀\n}]{1})(^)",
            "😀\nx",
            false,
            true,
            1,
            0..3,
            vec![Some(0..0), Some(0..3), Some(3..3)],
        ),
        (
            r"(\b)([\q{s|sa|k|ka}]{1})(\B)",
            "ſa!",
            true,
            false,
            0,
            0..1,
            vec![Some(0..0), Some(0..1), Some(1..1)],
        ),
        (
            r"(){0}()([\q{a|ab}]{1})()?(())",
            "ab!",
            false,
            false,
            0,
            0..2,
            vec![None, Some(0..0), Some(0..2), None, Some(2..2), Some(2..2)],
        ),
    ] {
        let source = JsString::from(source);
        assert!(
            Plan::compile_with_work(&source, i, m, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        assert!(
            Plan::compile_unconditional_assertions_with_work(&source, i, m, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        assert!(
            Plan::compile_reversed_line_assertions_with_work(&source, i, m, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none()
        );
        let plan = Plan::compile_exact_single_counts_with_work(&source, i, m, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), start, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found, range, "{source:?}");
        assert_eq!(plan.capture_count(), captures.len());
        assert_eq!(
            (0..captures.len())
                .map(|n| plan.capture_range(n, &found))
                .collect::<Vec<_>>(),
            captures,
            "{source:?}"
        );
    }
    let source = JsString::from(
        format!(
            r"{}(^)([\q{{|a|ab}}])($){}",
            "(".repeat(20_000),
            "){1}".repeat(20_000)
        )
        .as_str(),
    );
    let original =
        Plan::compile_exact_single_counts_with_work(&source, false, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.capture_count(), 20_003);
    for (input, end) in [("ab", 2), ("", 0)] {
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found, 0..end);
        for n in 0..20_000 {
            assert_eq!(plan.capture_range(n, &found), Some(0..end));
        }
        assert_eq!(plan.capture_range(20_000, &found), Some(0..0));
        assert_eq!(plan.capture_range(20_001, &found), Some(0..end));
        assert_eq!(plan.capture_range(20_002, &found), Some(end..end));
        assert_eq!(plan.capture_range(20_003, &found), None);
    }
    let zeros = "0".repeat(20_000);
    let source = JsString::from(format!(r"()([\q{{a|ab}}]){{{zeros}1,{zeros}1}}?()").as_str());
    let plan =
        Plan::compile_exact_single_counts_with_work(&source, false, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
    let found = plan
        .find_with_captures_and_work(&JsString::from("ab!"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found, 0..2);
    assert_eq!(plan.capture_range(0, &found), Some(0..0));
    assert_eq!(plan.capture_range(1, &found), Some(0..2));
    assert_eq!(plan.capture_range(2, &found), Some(2..2));
    for source in [
        r"()([\q{a|ab}]{1})()",
        r"(^){2}([\q{|a|ab}]){1}($){2}",
        r"($)([\q{\n|\na}]{1})(^)",
        r"(\b\B)([\q{a|ab}]){1}",
    ] {
        let source = JsString::from(source);
        let mut full = Vec::new();
        let plan = Plan::compile_exact_single_counts_with_work(&source, true, true, |n| {
            full.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=full.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(Plan::compile_exact_single_counts_with_work(&source,true,true,|n|{seen.push(n);if seen.len()==fail{Err(fail)}else{Ok(())}}),Err(crate::RegExpUnicodeSetCompileError::Work(n))if n==fail)
            );
            assert_eq!(seen, full[..fail]);
        }
        let mut recovered = Vec::new();
        assert!(
            Plan::compile_exact_single_counts_with_work(&source, true, true, |n| {
                recovered.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(recovered, full);
        for input in [
            JsString::from("ab!"),
            JsString::from("\na"),
            JsString::from(""),
        ] {
            for sticky in [false, true] {
                let mut full = Vec::new();
                let expected = plan
                    .find_with_captures_and_work(&input, 0, sticky, |n| {
                        full.push(n);
                        Ok::<(), usize>(())
                    })
                    .unwrap();
                for fail in 1..=full.len() {
                    let mut seen = Vec::new();
                    assert_eq!(
                        plan.find_with_captures_and_work(&input, 0, sticky, |n| {
                            seen.push(n);
                            if seen.len() == fail {
                                Err(fail)
                            } else {
                                Ok(())
                            }
                        }),
                        Err(fail)
                    );
                    assert_eq!(seen, full[..fail]);
                }
                assert_eq!(
                    plan.find_with_captures_and_work(&input, 0, sticky, |_| Ok::<(), usize>(())),
                    Ok(expected)
                );
            }
        }
    }
    for source in [
        r"()([\q{a|ab}])()",
        r"([\q{a|ab}]){0}",
        r"([\q{a|ab}]){2}",
        r"([\q{a|ab}]){1,}",
        r"([\q{a|ab}]){1,2}",
        r"([\q{a|ab}])?",
        r"([\q{a|ab}])+",
        r"([\q{a|ab}])*",
        r"([\q{a|ab}]{2}){1}",
        r"[\q{a|ab}]{1}[\q{}]",
        r"\b\B[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]{1}",
        r"([\q{a|ab}]){1}\1",
        r"(\b)?[\q{a|ab}]{1}",
        r"((\b)|())[\q{a|ab}]{1}",
        r"[\q{a|ab}]{1}a",
        r"[\q{a|ab}]{1}(?=a)",
        r"[\q{a|ab}]{1}|()",
        r"([\q{a|ab}]|()){1}",
        r"(?:^[\q{ab}&&\p{RGI_Emoji}]){0}",
    ] {
        assert!(
            Plan::compile_exact_single_counts_with_work(
                &JsString::from(source),
                false,
                false,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}
