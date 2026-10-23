use super::*;

fn compile_nullable(source: &str, i: bool, m: bool) -> Option<RegExpUnicodeRepeatedAtomMatcher> {
    RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(
        &JsString::from(source),
        i,
        m,
        |_| Ok::<(), ()>(()),
    )
    .unwrap()
}

struct Oracle {
    points: Vec<u32>,
    offsets: Vec<usize>,
    contains: fn(u32) -> bool,
    bounds: Bounds,
    gates: Gates,
    empty_first: bool,
    captures: usize,
}

impl Oracle {
    fn new(
        input: &JsString,
        contains: fn(u32) -> bool,
        bounds: Bounds,
        gates: Gates,
        empty_first: bool,
        captures: usize,
    ) -> Self {
        let units = input.code_units();
        let mut points = Vec::new();
        let mut offsets = vec![0];
        let mut at = 0;
        while at < units.len() {
            let first = units[at];
            let (point, width) = if (0xd800..=0xdbff).contains(&first)
                && units
                    .get(at + 1)
                    .is_some_and(|last| (0xdc00..=0xdfff).contains(last))
            {
                (
                    0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(units[at + 1])
                        - 0xdc00,
                    2,
                )
            } else {
                (u32::from(first), 1)
            };
            points.push(point);
            at += width;
            offsets.push(at);
        }
        Self {
            points,
            offsets,
            contains,
            bounds,
            gates,
            empty_first,
            captures,
        }
    }

    fn boundary(&self, at: usize) -> bool {
        let word = |p| {
            let p = if self.gates.i { fold(p) } else { p };
            (48..=57).contains(&p) || (65..=90).contains(&p) || (97..=122).contains(&p) || p == 95
        };
        at.checked_sub(1)
            .is_some_and(|before| word(self.points[before]))
            != self.points.get(at).is_some_and(|&p| word(p))
    }

    fn end_allowed(&self, at: usize) -> bool {
        (!self.gates.anchors
            || at == self.points.len()
            || self.gates.m && matches!(self.points[at], 10 | 13 | 0x2028 | 0x2029))
            && self.gates.words[1].is_none_or(|expected| self.boundary(at) == expected)
    }

    // Small-input specification backtracking oracle, unlike production's linear
    // scan. RepeatMatcher rejects optional empty iterations before continuation.
    // https://262.ecma-international.org/17.0/#sec-repeatmatcher
    fn repeat(
        &self,
        at: usize,
        minimum: usize,
        maximum: Option<usize>,
        capture: Option<Range<usize>>,
    ) -> Option<(usize, Option<Range<usize>>)> {
        if maximum == Some(0) {
            return self.end_allowed(at).then_some((at, capture));
        }
        if minimum == 0 && !self.bounds.greedy && self.end_allowed(at) {
            return Some((at, capture));
        }
        for empty in [self.empty_first, !self.empty_first] {
            let end = if empty {
                at
            } else if self
                .points
                .get(at)
                .is_some_and(|&p| (self.contains)(if self.gates.i { fold(p) } else { p }))
            {
                at + 1
            } else {
                continue;
            };
            if minimum == 0 && at == end {
                continue;
            }
            let selected = (self.captures != 0).then_some(self.offsets[at]..self.offsets[end]);
            if let Some(result) = self.repeat(
                end,
                minimum.saturating_sub(1),
                maximum.map(|n| n - 1),
                selected,
            ) {
                return Some(result);
            }
        }
        if minimum == 0 && self.bounds.greedy && self.end_allowed(at) {
            Some((at, capture))
        } else {
            None
        }
    }

    fn find(&self, start: usize, sticky: bool) -> Option<RegExpUnicodeRepeatedAtomMatch> {
        if start > *self.offsets.last().unwrap() {
            return None;
        }
        let first = self.offsets.partition_point(|&offset| offset <= start) - 1;
        for begin in first..=self.points.len() {
            if sticky && begin != first {
                break;
            }
            if self.gates.anchors
                && !(begin == 0
                    || self.gates.m && matches!(self.points[begin - 1], 10 | 13 | 0x2028 | 0x2029))
            {
                continue;
            }
            if self.gates.words[0].is_some_and(|expected| self.boundary(begin) != expected) {
                continue;
            }
            if let Some((end, atom_capture)) =
                self.repeat(begin, self.bounds.minimum, self.bounds.maximum, None)
            {
                return Some(RegExpUnicodeRepeatedAtomMatch {
                    range: self.offsets[begin]..self.offsets[end],
                    atom_capture,
                });
            }
        }
        None
    }
}

#[test]
fn nullable_character_results_follow_independent_branch_priority_progress_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate); 5] = [
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
    ];
    for (class, contains) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, minimum, maximum, greedy) in [
                    ("*", 0, None, true),
                    ("*?", 0, None, false),
                    ("+", 1, None, true),
                    ("+?", 1, None, false),
                    ("?", 0, Some(1), true),
                    ("??", 0, Some(1), false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_nullable(&source, i, m).unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                assert!(RegExpUnicodeRepeatedAtomMatcher::compile_nullable_v_set_with_single_character_strings_and_work(&JsString::from(source.as_str()),i,m,|_|Ok::<(),()>(())).unwrap().is_none());
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            minimum,
                                            maximum,
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn raw_empty_branches_keep_required_empty_captures_distinct_from_zero_iterations_and_q_members() {
    for (source, capture) in [
        (r"(([\q{ab}&&a]|))+", Some(0..0)),
        (r"((|[\q{ab}&&a]))+?", Some(0..0)),
        (r"(([\q{ab}&&a]|))*", None),
        (r"((|[\q{ab}&&a]))??", None),
    ] {
        let plan = compile_nullable(source, false, false).unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(found.atom_capture, capture);
        assert_eq!(plan.capture_count(), 2);
    }
    for source in [
        r"([\q{ab}&&\q{cd}]|)*",
        r"(|[\q{ab}--\q{ab}])+",
        r"([\q{a|ab}&&a]|){0}",
        r"([\q{a|ab}&&a]|){1,2}",
        r"([\q{a|ab}&&a]|b)+",
        r"(([\q{a|ab}&&a])|)+",
        r"([\q{a|ab}&&a]|())+",
        r"(\b[\q{a|ab}&&a]|)+",
        r"(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}])+",
        r"([\q{a|ab}&&a]|)+[b]",
        r"\b^([\q{a|ab}&&a]|)+",
    ] {
        assert!(compile_nullable(source, true, true).is_none(), "{source}");
    }
}

#[test]
fn nullable_character_result_construction_checks_callbacks_and_flat_deep_and_long_storage() {
    for source in [
        r"^\b((|[\q{a|ab}&&a]))+?\b$",
        r"(([\q{ab}&&a]|))*",
        r"(?:[^\q{ab}&&a]|)+",
    ] {
        let source = JsString::from(source);
        let mut complete = Vec::new();
        RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(
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
        assert_eq!(complete[0], source.len() * 10);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(
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
        RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(
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
        format!(
            r"{}[\q{{a|ab}}&&a]|{}+",
            "(".repeat(20_000),
            ")".repeat(20_000)
        )
        .as_str(),
    );
    let mut work = 0usize;
    let original =
        RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(
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
        work < source.len() * 24 + 1024,
        "nullable wrapper work must stay linear: {work}"
    );
    let plan = original.clone();
    drop(original);
    assert_eq!(plan.capture_count(), 20_000);
    let found = plan
        .find_with_captures_and_work(&JsString::from("aaa"), 0, false, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..3);
    assert_eq!(found.atom_capture, Some(2..3));
    let source = JsString::from(format!(r"(|[\q{{{}}}&&a])+", "x".repeat(20_000)).as_str());
    let mut work = 0usize;
    let plan = RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(
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
        work < source.len() * 24 + 1024,
        "discarded nullable words must stay compact: {work}"
    );
    let found = plan
        .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..0);
    assert_eq!(found.atom_capture, Some(0..0));
}

fn compile_zero_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_zero_minimum_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn zero_minimum_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, maximum, greedy) in [
                    ("{0}", Some(0), true),
                    ("{00,0}?", Some(0), false),
                    ("{0,1}", Some(1), true),
                    ("{0,1}?", Some(1), false),
                    ("{00,0003}", Some(3), true),
                    ("{0,3}?", Some(3), false),
                    ("{0,}", None, true),
                    ("{000,}?", None, false),
                    ("{0,999999999999999999999999999999999999999}", None, true),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_zero_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            minimum: 0,
                                            maximum,
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn zero_minimum_string_braces_keep_complete_leaf_proofs_and_closed_constructor_domains() {
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|ab}&&a]")] {
        for q in [
            "{0}",
            "{00,00}?",
            "{0,3}",
            "{0,}",
            "{0,999999999999999999999999999}",
        ] {
            let source = JsString::from(format!("(({class}|)){q}").as_str());
            let plan = compile_zero_string_with_work(&source, false, false, singleton, |_| {
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
            let found = plan
                .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..0);
            assert_eq!(found.atom_capture, None);
            assert_eq!(plan.capture_count(), 2);
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_nullable_v_set_with_single_character_strings_and_work(&source, false, false, |_| Ok::<(), ()>(())).unwrap().is_none());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(&source, false, false, |_| Ok::<(), ()>(())).unwrap().is_none());
        }
        for tail in ["*", "+?", "?", "{1}", "{1,}", "{1,2}"] {
            let source = JsString::from(format!("({class}|){tail}").as_str());
            assert!(
                compile_zero_string_with_work(&source, false, false, singleton, |_| Ok::<(), ()>(
                    ()
                ))
                .unwrap()
                .is_none()
            );
        }
        for source in [
            r"([\q{a|b}]|()){0}",
            r"([\q{a|ab}&&a]|()){0}",
            r"(([\q{a|ab}&&a])|){0}",
            r"([\q{a|ab}&&a]||){0}",
            r"(\b[\q{a|ab}&&a]|){0}",
            r"\b^([\q{a|ab}&&a]|){0}",
            r"([\q{a|ab}&&a]|){0}[b]",
            r"(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){0}",
            r"([\q{ab}&&\q{cd}]|){0}",
            r"([\q{}]|){0}",
        ] {
            assert!(
                compile_zero_string_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
    for (singleton, source) in [(true, r"([\q{a|ab}&&a]|){0}"), (false, r"([\q{a|b}]|){0}")] {
        assert!(
            compile_zero_string_with_work(
                &JsString::from(source),
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

#[test]
fn zero_minimum_string_braces_charge_every_scan_and_keep_deep_wrappers_and_counts_flat() {
    for (singleton, pattern) in [
        (true, r"^\b((|[\q{a|b}])){0,3}?\b$"),
        (false, r"(([\q{ab}&&a]|)){0}"),
        (
            false,
            r"(?:[^\q{ab}&&a]|){0,999999999999999999999999999999999999999}",
        ),
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        compile_zero_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = compile_zero_string_with_work(&source, true, true, singleton, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_zero_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{0,3}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..3,
            Some(2..3),
        ),
        (
            format!(r"(|[\q{{{}}}&&a]){{0,}}", "x".repeat(20_000)),
            1,
            0..0,
            None,
        ),
        (
            format!(r"([\q{{a|ab}}&&a]|){{0,{}}}", "9".repeat(20_000)),
            1,
            0..3,
            Some(2..3),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original = compile_zero_string_with_work(&source, false, false, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 32 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, capture);
    }
}

fn compile_one_open_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_one_minimum_open_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn one_minimum_open_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, maximum, greedy) in [
                    ("{1,}", None, true),
                    ("{01,}", None, true),
                    ("{1,}?", None, false),
                    ("{0001,}?", None, false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_one_open_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            minimum: 1,
                                            maximum,
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn one_minimum_open_string_braces_charge_every_scan_and_keep_deep_wrappers_and_counts_flat() {
    for (singleton, pattern) in [
        (true, r"^\b((|[\q{a|b}])){1,}?\b$"),
        (false, r"(([\q{ab}&&a]|)){1,}"),
        (false, r"(?:[^\q{ab}&&a]|){01,}"),
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        compile_one_open_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = compile_one_open_string_with_work(&source, true, true, singleton, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_one_open_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{1,}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..3,
            Some(2..3),
        ),
        (
            format!(r"(|[\q{{{}}}&&a]){{1,}}?", "x".repeat(20_000)),
            1,
            0..0,
            Some(0..0),
        ),
        (
            format!(r"([\q{{a|ab}}&&a]|){{{}1,}}", "0".repeat(20_000)),
            1,
            0..3,
            Some(2..3),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original = compile_one_open_string_with_work(&source, false, false, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 32 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, capture);
    }
}

#[test]
fn one_minimum_open_string_braces_keep_required_empty_captures_and_complete_proof_domains() {
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|ab}&&a]")] {
        for q in ["{1,}", "{01,}", "{1,}?", "{0001,}?"] {
            let source = JsString::from(format!("(({class}|)){q}").as_str());
            let plan = compile_one_open_string_with_work(&source, false, false, singleton, |_| {
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
            let found = plan
                .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..0);
            assert_eq!(found.atom_capture, Some(0..0));
            assert_eq!(plan.capture_count(), 2);
            assert!(
                compile_zero_string_with_work(&source, false, false, singleton, |_| Ok::<(), ()>(
                    ()
                ))
                .unwrap()
                .is_none()
            );
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_nullable_v_set_with_single_character_strings_and_work(&source, false, false, |_| Ok::<(), ()>(())).unwrap().is_none());
            assert!(RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(&source, false, false, |_| Ok::<(), ()>(())).unwrap().is_none());
        }
        for tail in [
            "*",
            "+?",
            "?",
            "{0,}",
            "{1}",
            "{1,1}",
            "{1,2}",
            "{2,}",
            "{1,999999999999999999999999999999999999999}",
        ] {
            let source = JsString::from(format!("({class}|){tail}").as_str());
            assert!(
                compile_one_open_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        for source in [
            r"([\q{a|b}]|()){1,}",
            r"([\q{a|ab}&&a]|()){1,}",
            r"(([\q{a|ab}&&a])|){1,}",
            r"([\q{a|ab}&&a]||){1,}",
            r"(\b[\q{a|ab}&&a]|){1,}",
            r"\b^([\q{a|ab}&&a]|){1,}",
            r"([\q{a|ab}&&a]|){1,}[b]",
            r"(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){1,}?",
            r"([\q{ab}&&\q{cd}]|){1,}",
            r"([\q{}]|){1,}",
        ] {
            assert!(
                compile_one_open_string_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
    for (singleton, source) in [
        (true, r"([\q{a|ab}&&a]|){1,}"),
        (false, r"([\q{a|b}]|){1,}"),
    ] {
        assert!(
            compile_one_open_string_with_work(
                &JsString::from(source),
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

fn compile_one_iteration_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_one_iteration_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn one_iteration_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, maximum, greedy) in [
                    ("{1}", Some(1), true),
                    ("{1}?", Some(1), false),
                    ("{01}", Some(1), true),
                    ("{0001}?", Some(1), false),
                    ("{1,1}", Some(1), true),
                    ("{1,1}?", Some(1), false),
                    ("{0001,01}", Some(1), true),
                    ("{01,0001}?", Some(1), false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_one_iteration_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            minimum: 1,
                                            maximum,
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn one_iteration_string_braces_charge_every_scan_and_keep_deep_wrappers_and_counts_flat() {
    for (singleton, pattern) in [
        (true, r"^\b((|[\q{a|b}])){1,1}?\b$"),
        (false, r"(([\q{ab}&&a]|)){1}"),
        (false, r"(?:[^\q{ab}&&a]|){01,001}"),
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        compile_one_iteration_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                compile_one_iteration_string_with_work(&source, true, true, singleton, |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_one_iteration_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{1}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..1,
            Some(0..1),
        ),
        (
            format!(r"(|[\q{{{}}}&&a]){{1}}?", "x".repeat(20_000)),
            1,
            0..0,
            Some(0..0),
        ),
        (
            format!(r"([\q{{a|ab}}&&a]|){{{}1}}", "0".repeat(20_000)),
            1,
            0..1,
            Some(0..1),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original = compile_one_iteration_string_with_work(&source, false, false, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 32 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, capture);
    }
}

#[test]
fn one_iteration_string_braces_keep_mandatory_branch_order_and_complete_proof_domains() {
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|ab}&&a]")] {
        for q in ["{1}", "{01}?", "{1,1}", "{0001,01}?"] {
            for (empty_first, range) in [(true, 0..0), (false, 0..1)] {
                let inner = if empty_first {
                    format!("|{class}")
                } else {
                    format!("{class}|")
                };
                let source = JsString::from(format!("(({inner})){q}").as_str());
                let plan = compile_one_iteration_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
                let found = plan
                    .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| {
                        Ok::<(), ()>(())
                    })
                    .unwrap()
                    .unwrap();
                assert_eq!(found.range, range);
                assert_eq!(found.atom_capture, Some(range));
                assert_eq!(plan.capture_count(), 2);
                assert!(
                    compile_zero_string_with_work(
                        &source,
                        false,
                        false,
                        singleton,
                        |_| Ok::<(), ()>(())
                    )
                    .unwrap()
                    .is_none()
                );
                assert!(
                    compile_one_open_string_with_work(&source, false, false, singleton, |_| Ok::<
                        (),
                        (),
                    >(
                        ()
                    ))
                    .unwrap()
                    .is_none()
                );
                assert!(RegExpUnicodeRepeatedAtomMatcher::compile_nullable_v_set_with_single_character_strings_and_work(&source, false, false, |_| Ok::<(), ()>(())).unwrap().is_none());
                assert!(RegExpUnicodeRepeatedAtomMatcher::compile_nullable_character_result_v_set_with_work(&source, false, false, |_| Ok::<(), ()>(())).unwrap().is_none());
            }
        }
        for tail in [
            "*",
            "+?",
            "?",
            "{0}",
            "{0,1}",
            "{1,}",
            "{1,2}",
            "{2}",
            "{2,2}",
            "{1,999999999999999999999999999999999999999}",
        ] {
            let source = JsString::from(format!("({class}|){tail}").as_str());
            assert!(
                compile_one_iteration_string_with_work(&source, false, false, singleton, |_| Ok::<
                    (),
                    (),
                >(
                    ()
                ))
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        for source in [
            r"([\q{a|b}]|()){1}",
            r"([\q{a|ab}&&a]|()){1}",
            r"(([\q{a|ab}&&a])|){1}",
            r"([\q{a|ab}&&a]||){1}",
            r"(\b[\q{a|ab}&&a]|){1}",
            r"\b^([\q{a|ab}&&a]|){1}",
            r"([\q{a|ab}&&a]|){1}[b]",
            r"(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){1}?",
            r"([\q{ab}&&\q{cd}]|){1}",
            r"([\q{}]|){1}",
        ] {
            assert!(
                compile_one_iteration_string_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
    for (singleton, source) in [(true, r"([\q{a|ab}&&a]|){1}"), (false, r"([\q{a|b}]|){1}")] {
        assert!(
            compile_one_iteration_string_with_work(
                &JsString::from(source),
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

fn compile_positive_open_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_open_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn positive_open_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, minimum, greedy) in [
                    ("{1,}", Some(1), true),
                    ("{2,}", Some(2), true),
                    ("{2,}?", Some(2), false),
                    ("{003,}", Some(3), true),
                    ("{03,}?", Some(3), false),
                    ("{999999999999999999999999999999999999999,}", None, true),
                    ("{999999999999999999999999999999999999999,}?", None, false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_positive_open_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            // A count above the whole input character count
                                            // has the same mandatory choices on these inputs.
                                            minimum: minimum.unwrap_or(input.len() + 1),
                                            maximum: None,
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn positive_open_string_braces_charge_every_scan_and_keep_deep_wrappers_and_counts_flat() {
    for (singleton, pattern) in [
        (true, r"^\b((|[\q{a|b}])){2,}?\b$"),
        (false, r"(([\q{ab}&&a]|)){3,}"),
        (
            false,
            r"(?:[^\q{ab}&&a]|){999999999999999999999999999999999999999,}",
        ),
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        compile_positive_open_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                compile_positive_open_string_with_work(&source, true, true, singleton, |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_positive_open_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{2,}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..3,
            Some(2..3),
        ),
        (
            format!(r"(|[\q{{{}}}&&a]){{3,}}?", "x".repeat(20_000)),
            1,
            0..0,
            Some(0..0),
        ),
        (
            format!(r"([\q{{a|ab}}&&a]|){{{},}}", "9".repeat(20_000)),
            1,
            0..3,
            Some(3..3),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original = compile_positive_open_string_with_work(&source, false, false, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 32 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, capture);
    }
}

#[test]
fn positive_open_string_braces_keep_mandatory_empty_capture_priority_and_complete_domains() {
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|ab}&&a]")] {
        for q in [
            "{1,}",
            "{2,}",
            "{003,}",
            "{2,}?",
            "{999999999999999999999999999999999999999,}",
        ] {
            let source = JsString::from(format!("(({class}|)){q}").as_str());
            let plan =
                compile_positive_open_string_with_work(&source, false, false, singleton, |_| {
                    Ok::<(), ()>(())
                })
                .unwrap()
                .unwrap();
            let found = plan
                .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..0);
            assert_eq!(found.atom_capture, Some(0..0));
            assert_eq!(plan.capture_count(), 2);
            assert!(
                compile_zero_string_with_work(&source, false, false, singleton, |_| Ok::<(), ()>(
                    ()
                ))
                .unwrap()
                .is_none()
            );
            assert!(
                compile_one_iteration_string_with_work(&source, false, false, singleton, |_| Ok::<
                    (),
                    (),
                >(
                    ()
                ))
                .unwrap()
                .is_none()
            );
            assert_eq!(
                compile_one_open_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_some(),
                q == "{1,}"
            );
        }
        for tail in [
            "*",
            "+?",
            "?",
            "{0,}",
            "{0}",
            "{1}",
            "{2}",
            "{2,2}",
            "{2,3}",
            "{2,999999999999999999999999999999999999999}",
        ] {
            let source = JsString::from(format!("({class}|){tail}").as_str());
            assert!(
                compile_positive_open_string_with_work(&source, false, false, singleton, |_| Ok::<
                    (),
                    (),
                >(
                    ()
                ))
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        for source in [
            r"([\q{a|b}]|()){2,}",
            r"([\q{a|ab}&&a]|()){2,}",
            r"(([\q{a|ab}&&a])|){2,}",
            r"([\q{a|ab}&&a]||){2,}",
            r"(\b[\q{a|ab}&&a]|){2,}",
            r"\b^([\q{a|ab}&&a]|){2,}",
            r"([\q{a|ab}&&a]|){2,}[b]",
            r"(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){2,}?",
            r"([\q{ab}&&\q{cd}]|){2,}",
            r"([\q{}]|){2,}",
        ] {
            assert!(
                compile_positive_open_string_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }
    for (source, input, range, capture) in [
        (r"([\q{a|ab}&&a]|){3,}", "aab", 0..2, 2..2),
        (r"(|[\q{a|ab}&&a]){3,}", "aab", 0..2, 1..2),
        (r"([\q{a|ab}&&a]|){3,}?", "aab", 0..2, 2..2),
        (r"(|[\q{a|ab}&&a]){3,}?", "aab", 0..0, 0..0),
        (r"([\q{😀|😀x}&&\p{Any}]|){2,}", "😀x", 0..2, 2..2),
        (r"(|[\q{😀|😀x}&&\p{Any}]){2,}", "😀x", 0..2, 0..2),
    ] {
        let plan = compile_positive_open_string_with_work(
            &JsString::from(source),
            false,
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range, "{source}");
        assert_eq!(found.atom_capture, Some(capture), "{source}");
    }
    for (singleton, source) in [
        (true, r"([\q{a|ab}&&a]|){2,}"),
        (false, r"([\q{a|b}]|){2,}"),
    ] {
        assert!(
            compile_positive_open_string_with_work(
                &JsString::from(source),
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

fn compile_positive_exact_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_exact_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn positive_exact_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, minimum, greedy) in [
                    ("{2}", 2, true),
                    ("{2}?", 2, false),
                    ("{02,0002}", 2, true),
                    ("{2,2}?", 2, false),
                    ("{3}", 3, true),
                    ("{003}?", 3, false),
                    ("{3,03}", 3, true),
                    ("{03,3}?", 3, false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_positive_exact_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            minimum,
                                            maximum: Some(minimum),
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn positive_exact_string_braces_charge_every_scan_and_keep_deep_wrappers_and_counts_flat() {
    for (singleton, pattern) in [
        (true, r"^\b((|[\q{a|b}])){2,2}?\b$"),
        (false, r"(([\q{ab}&&a]|)){3}"),
        (false, r"(?:[^\q{ab}&&a]|){03,0003}"),
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        compile_positive_exact_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                compile_positive_exact_string_with_work(&source, true, true, singleton, |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_positive_exact_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{2}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..2,
            Some(1..2),
        ),
        (
            format!(r"(|[\q{{{}}}&&a]){{2}}?", "x".repeat(20_000)),
            1,
            0..0,
            Some(0..0),
        ),
        (
            format!(r"([\q{{a|ab}}&&a]|){{{}2}}", "0".repeat(20_000)),
            1,
            0..2,
            Some(1..2),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original = compile_positive_exact_string_with_work(&source, false, false, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 32 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, capture);
    }
}

#[test]
fn positive_exact_string_braces_keep_mandatory_order_decoded_counts_and_closed_domains() {
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|ab}&&a]")] {
        for (q, character_capture) in [
            ("{2}", 1..2),
            ("{02,0002}?", 1..2),
            ("{3}", 2..2),
            ("{003,3}?", 2..2),
        ] {
            for (empty_first, range, capture) in
                [(true, 0..0, 0..0), (false, 0..2, character_capture)]
            {
                let inner = if empty_first {
                    format!("|{class}")
                } else {
                    format!("{class}|")
                };
                let source = JsString::from(format!("(({inner})){q}").as_str());
                let plan = compile_positive_exact_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
                let found = plan
                    .find_with_captures_and_work(&JsString::from("aa"), 0, true, |_| {
                        Ok::<(), ()>(())
                    })
                    .unwrap()
                    .unwrap();
                assert_eq!(found.range, range);
                assert_eq!(found.atom_capture, Some(capture));
                assert_eq!(plan.capture_count(), 2);
                assert!(
                    compile_one_iteration_string_with_work(
                        &source,
                        false,
                        false,
                        singleton,
                        |_| Ok::<(), ()>(())
                    )
                    .unwrap()
                    .is_none()
                );
                assert!(
                    compile_positive_open_string_with_work(
                        &source,
                        false,
                        false,
                        singleton,
                        |_| Ok::<(), ()>(())
                    )
                    .unwrap()
                    .is_none()
                );
            }
        }
        for tail in [
            "*",
            "+?",
            "?",
            "{0}",
            "{1}",
            "{1,1}",
            "{0,3}",
            "{2,}",
            "{2,3}",
            "{1,2}",
            "{999999999999999999999999999999999999999}",
        ] {
            let source = JsString::from(format!("({class}|){tail}").as_str());
            assert!(
                compile_positive_exact_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        for source in [
            r"([\q{a|b}]|()){2}",
            r"([\q{a|ab}&&a]|()){2}",
            r"(([\q{a|ab}&&a])|){2}",
            r"([\q{a|ab}&&a]||){2}",
            r"(\b[\q{a|ab}&&a]|){2}",
            r"\b^([\q{a|ab}&&a]|){2}",
            r"([\q{a|ab}&&a]|){2}[b]",
            r"(|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]){2}?",
            r"([\q{ab}&&\q{cd}]|){2}",
            r"([\q{}]|){2}",
        ] {
            assert!(
                compile_positive_exact_string_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        for empty_first in [false, true] {
            let inner = if empty_first {
                format!("|{class}")
            } else {
                format!("{class}|")
            };
            let source = JsString::from(format!("({inner}){{{}}}", usize::MAX).as_str());
            let plan =
                compile_positive_exact_string_with_work(&source, false, false, singleton, |_| {
                    Ok::<(), ()>(())
                })
                .unwrap()
                .unwrap();
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            let range = if empty_first { 0..0 } else { 0..3 };
            assert_eq!(found.range, range);
            assert_eq!(found.atom_capture, Some(range.end..range.end));
        }
    }
    for (source, input, range, capture) in [
        (r"([\q{😀|😀x}&&\p{Any}]|){3}$", "😀", 0..2, 2..2),
        (r"(|[\q{😀|😀x}&&\p{Any}]){3}$", "😀", 0..2, 0..2),
        (r"([\q{😀|😀x}&&\p{Any}]|){3}?$", "😀😀😀😀", 2..8, 6..8),
        (r"(|[\q{😀|😀x}&&\p{Any}]){3}$", "😀😀😀😀", 2..8, 6..8),
    ] {
        let plan = compile_positive_exact_string_with_work(
            &JsString::from(source),
            false,
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range, "{source}");
        assert_eq!(found.atom_capture, Some(capture), "{source}");
    }
    for (singleton, source) in [(true, r"([\q{a|ab}&&a]|){2}"), (false, r"([\q{a|b}]|){2}")] {
        assert!(
            compile_positive_exact_string_with_work(
                &JsString::from(source),
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

fn compile_unrepresentable_exact_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_unrepresentable_exact_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn unrepresentable_exact_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, greedy) in [
                    ("{9999999999999999999999999999999999999999}", true),
                    ("{9999999999999999999999999999999999999999}?", false),
                    (
                        "{9999999999999999999999999999999999999999,9999999999999999999999999999999999999999}",
                        true,
                    ),
                    (
                        "{0009999999999999999999999999999999999999999,09999999999999999999999999999999999999999}?",
                        false,
                    ),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_unrepresentable_exact_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            // Above every possible character count on this input,
                                            // this finite exact count has the same mandatory choices.
                                            minimum: input.len() + 1,
                                            maximum: Some(input.len() + 1),
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn unrepresentable_exact_string_braces_keep_numeric_equality_and_closed_proof_domains() {
    let huge = "9".repeat(40);
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|b|ab}&&\p{Any}]")] {
        for q in [
            format!("{{{huge}}}"),
            format!("{{{huge}}}?"),
            format!("{{{huge},{huge}}}"),
            format!("{{000{huge},0{huge}}}?"),
        ] {
            for empty_first in [false, true] {
                let inner = if empty_first {
                    format!("|{class}")
                } else {
                    format!("{class}|")
                };
                let source = JsString::from(format!("(({inner})){q}").as_str());
                let plan = compile_unrepresentable_exact_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
                let found = plan
                    .find_with_captures_and_work(&JsString::from("abX"), 0, true, |_| {
                        Ok::<(), ()>(())
                    })
                    .unwrap()
                    .unwrap();
                let range = if empty_first { 0..0 } else { 0..2 };
                assert_eq!(found.range, range);
                assert_eq!(found.atom_capture, Some(range.end..range.end));
                assert_eq!(plan.capture_count(), 2);
                assert!(
                    compile_positive_exact_string_with_work(
                        &source,
                        false,
                        false,
                        singleton,
                        |_| Ok::<(), ()>(())
                    )
                    .unwrap()
                    .is_none()
                );
                assert!(
                    compile_positive_open_string_with_work(
                        &source,
                        false,
                        false,
                        singleton,
                        |_| Ok::<(), ()>(())
                    )
                    .unwrap()
                    .is_none()
                );
            }
        }
        for q in [
            "*".to_owned(),
            "+?".to_owned(),
            "{0}".to_owned(),
            "{2}".to_owned(),
            format!("{{{}}}", usize::MAX),
            format!("{{{huge},}}"),
            format!("{{{huge},{huge}0}}"),
            format!("{{{},{}9}}", "8".repeat(40), "8".repeat(39)),
        ] {
            let source = JsString::from(format!("({class}|){q}").as_str());
            assert!(
                compile_unrepresentable_exact_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        for inner in [
            format!("({class})|"),
            format!("{class}|()"),
            format!("{class}||"),
            format!(r"\b{class}|"),
            format!("{class}|[b]"),
            r"|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]".to_owned(),
            r"[\q{ab}&&\q{cd}]|".to_owned(),
            r"[\q{}]|".to_owned(),
        ] {
            let source = JsString::from(format!("({inner}){{{huge}}}").as_str());
            assert!(
                compile_unrepresentable_exact_string_with_work(
                    &source,
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
    }
    for (singleton, class) in [(true, r"[\q{a|ab}&&a]"), (false, r"[\q{a|b}]")] {
        let source = JsString::from(format!("({class}|){{{huge}}}").as_str());
        assert!(
            compile_unrepresentable_exact_string_with_work(
                &source,
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

#[test]
fn unrepresentable_exact_string_braces_charge_every_scan_and_keep_all_storage_flat() {
    let huge = "9".repeat(40);
    for (singleton, pattern) in [
        (true, format!(r"^\b((|[\q{{a|b}}])){{{huge},{huge}}}?\b$")),
        (false, format!(r"(([\q{{ab}}&&a]|)){{{huge}}}")),
        (false, format!(r"(?:[^\q{{ab}}&&a]|){{000{huge},0{huge}}}?")),
    ] {
        let source = JsString::from(pattern.as_str());
        let mut complete = Vec::new();
        compile_unrepresentable_exact_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        assert!(complete[2] > 0 && complete[2] <= source.len() * 3);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = compile_unrepresentable_exact_string_with_work(
                &source,
                true,
                true,
                singleton,
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
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_unrepresentable_exact_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{{huge}}}?",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..3,
            3..3,
        ),
        (
            format!(r"(|[\q{{{}}}&&a]){{{huge}}}", "x".repeat(20_000)),
            1,
            0..0,
            0..0,
        ),
        (
            format!(r"([\q{{a|ab}}&&a]|){{{}}}", "9".repeat(20_000)),
            1,
            0..3,
            3..3,
        ),
        (
            format!(
                r"(|[\q{{a|ab}}&&a]){{000{huge},{}{huge}}}",
                "0".repeat(20_000)
            ),
            1,
            0..0,
            0..0,
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original =
            compile_unrepresentable_exact_string_with_work(&source, false, false, false, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(
            work < source.len() * 32 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, Some(capture));
    }
}

fn compile_positive_finite_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_positive_finite_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn positive_finite_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                for (quantifier, minimum, maximum, greedy) in [
                    ("{1,2}", 1, 2, true),
                    ("{1,2}?", 1, 2, false),
                    ("{2,3}", 2, 3, true),
                    ("{02,0003}?", 2, 3, false),
                    ("{2,4}", 2, 4, true),
                    ("{3,4}?", 3, 4, false),
                    ("{3,5}", 3, 5, true),
                    ("{0003,0005}?", 3, 5, false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_positive_finite_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            minimum,
                                            maximum: Some(maximum),
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn positive_finite_string_braces_charge_every_scan_and_keep_deep_wrappers_and_counts_flat() {
    for (singleton, pattern) in [
        (true, r"^\b((|[\q{a|b}])){1,2}?\b$"),
        (false, r"(([\q{ab}&&a]|)){2,3}"),
        (false, r"(?:[^\q{ab}&&a]|){02,0003}"),
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        compile_positive_finite_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                compile_positive_finite_string_with_work(&source, true, true, singleton, |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_positive_finite_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{2,3}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..3,
            Some(2..3),
        ),
        (
            format!(r"(|[\q{{{}}}&&a]){{2,3}}?", "x".repeat(20_000)),
            1,
            0..0,
            Some(0..0),
        ),
        (
            format!(r"([\q{{a|ab}}&&a]|){{{}2,4}}", "0".repeat(20_000)),
            1,
            0..3,
            Some(2..3),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original =
            compile_positive_finite_string_with_work(&source, false, false, false, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(
            work < source.len() * 32 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, capture);
    }
}

#[test]
fn positive_finite_string_braces_keep_exact_optional_budgets_and_closed_domains() {
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|b|ab}&&\p{Any}]")] {
        for q in ["{1,2}", "{02,0003}?", "{2,4}", "{003,5}"] {
            let source = JsString::from(format!("(({class}|)){q}").as_str());
            let plan =
                compile_positive_finite_string_with_work(&source, false, false, singleton, |_| {
                    Ok::<(), ()>(())
                })
                .unwrap()
                .unwrap();
            let found = plan
                .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..0);
            assert_eq!(found.atom_capture, Some(0..0));
            assert_eq!(plan.capture_count(), 2);
            assert!(
                compile_positive_exact_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                compile_positive_open_string_with_work(&source, false, false, singleton, |_| Ok::<
                    (),
                    (),
                >(
                    ()
                ))
                .unwrap()
                .is_none()
            );
        }
        for tail in [
            "*",
            "+?",
            "?",
            "{0,3}",
            "{1}",
            "{2}",
            "{2,2}",
            "{2,}",
            "{2,999999999999999999999999999999999999999}",
            "{999999999999999999999999999999999999999,9999999999999999999999999999999999999999}",
        ] {
            let source = JsString::from(format!("({class}|){tail}").as_str());
            assert!(
                compile_positive_finite_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        for inner in [
            format!("({class})|"),
            format!("{class}|()"),
            format!("{class}||"),
            format!(r"\b{class}|"),
            format!("{class}|[b]"),
            r"|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]".to_owned(),
            r"[\q{ab}&&\q{cd}]|".to_owned(),
            r"[\q{}]|".to_owned(),
        ] {
            let source = JsString::from(format!("({inner}){{2,3}}").as_str());
            assert!(
                compile_positive_finite_string_with_work(&source, true, true, singleton, |_| Ok::<
                    (),
                    (),
                >(
                    ()
                ))
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        let source =
            JsString::from(format!("(|{class}){{{},{}}}", usize::MAX - 1, usize::MAX).as_str());
        let plan =
            compile_positive_finite_string_with_work(&source, false, false, singleton, |_| {
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..1);
        assert_eq!(found.atom_capture, Some(0..1));
    }
    for (source, input, range, capture) in [
        (r"(|[\q{a|ab}&&a]){2,3}", "aab", 0..1, 0..1),
        (r"(|[\q{a|ab}&&a]){2,3}$", "aa", 0..2, 1..2),
        (r"([\q{a|ab}&&a]|){3,4}", "aab", 0..2, 2..2),
        (r"(|[\q{a|ab}&&a]){3,4}?", "aab", 0..0, 0..0),
        (r"(|[\q{😀|😀x}&&\p{Any}]){2,3}$", "😀😀", 0..4, 2..4),
        (r"([\q{😀|😀x}&&\p{Any}]|){3,4}", "😀x", 0..2, 2..2),
    ] {
        let plan = compile_positive_finite_string_with_work(
            &JsString::from(source),
            false,
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range, "{source}");
        assert_eq!(found.atom_capture, Some(capture), "{source}");
    }
    for (singleton, source) in [
        (true, r"([\q{a|ab}&&a]|){2,3}"),
        (false, r"([\q{a|b}]|){2,3}"),
    ] {
        assert!(
            compile_positive_finite_string_with_work(
                &JsString::from(source),
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}

fn compile_overflowing_maximum_string_with_work<E>(
    source: &JsString,
    i: bool,
    m: bool,
    singleton: bool,
    charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<RegExpUnicodeRepeatedAtomMatcher>, crate::RegExpUnicodeSetCompileError<E>> {
    if singleton {
        RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_v_set_with_single_character_strings_and_work(source, i, m, charge)
    } else {
        RegExpUnicodeRepeatedAtomMatcher::compile_overflowing_maximum_nullable_character_result_v_set_with_work(source, i, m, charge)
    }
}

#[test]
fn overflowing_maximum_string_braces_follow_independent_progress_branch_and_capture_oracle() {
    type Predicate = fn(u32) -> bool;
    let classes: [(&str, Predicate, bool); 6] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98, true),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98, false),
        (r"[\q{ab}&&a]", |_| false, false),
        (r"[^\q{ab}&&a]", |_| true, false),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600, false),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428, false),
    ];
    for (class, contains, singleton) in classes {
        for (open, close, captures) in [("(", ")", 1), ("((", "))", 2), ("(?:(?:", "))", 0)] {
            for empty_first in [false, true] {
                let limit = usize::MAX as u128;
                for (quantifier, represented_minimum, optional, greedy) in [
                    (format!("{{1,{}}}", limit + 1), Some(1), None, true),
                    (format!("{{01,000{}}}?", limit + 1), Some(1), None, false),
                    (format!("{{3,{}}}", limit * 2 + 3), Some(3), None, true),
                    (format!("{{0003,{}}}?", limit * 2 + 3), Some(3), None, false),
                    (format!("{{{limit},{}}}", limit + 1), None, Some(1), true),
                    (format!("{{{limit},{}}}?", limit + 1), None, Some(1), false),
                    (format!("{{0{limit},00{}}}", limit + 3), None, Some(3), true),
                    (format!("{{{limit},{}}}?", limit + 3), None, Some(3), false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let inner = if empty_first {
                                    format!("|{class}")
                                } else {
                                    format!("{class}|")
                                };
                                let source =
                                    format!("{prefix}{open}{inner}{close}{quantifier}{suffix}");
                                let plan = compile_overflowing_maximum_string_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    singleton,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    // A minimum above all input characters keeps
                                    // every consuming choice mandatory. An upper
                                    // bound above all possible consumed characters
                                    // preserves the otherwise unreachable limit.
                                    let minimum = represented_minimum.unwrap_or(input.len() + 1);
                                    let maximum =
                                        optional.map_or(minimum + input.len() + 1, |n| minimum + n);
                                    let expected = Oracle::new(
                                        &input,
                                        contains,
                                        Bounds {
                                            minimum,
                                            maximum: Some(maximum),
                                            greedy,
                                        },
                                        Gates {
                                            anchors,
                                            words,
                                            i,
                                            m,
                                        },
                                        empty_first,
                                        captures,
                                    );
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            assert_eq!(
                                                plan.find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(())
                                                )
                                                .unwrap(),
                                                expected.find(start, sticky),
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
    }
}

#[test]
fn overflowing_maximum_string_braces_charge_every_scan_and_keep_deep_wrappers_and_counts_flat() {
    for (singleton, pattern) in [
        (
            true,
            r"^\b((|[\q{a|b}])){1,999999999999999999999999999999999999999}?\b$",
        ),
        (
            false,
            r"(([\q{ab}&&a]|)){2,999999999999999999999999999999999999999}",
        ),
        (
            false,
            r"(?:[^\q{ab}&&a]|){02,000999999999999999999999999999999999999999}",
        ),
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        compile_overflowing_maximum_string_with_work(&source, true, true, singleton, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[..2], [source.len() * 10, source.len() * 8]);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                compile_overflowing_maximum_string_with_work(&source, true, true, singleton, |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(
                matches!(result, Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at == fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        compile_overflowing_maximum_string_with_work(&source, true, true, singleton, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, captures, range, capture) in [
        (
            format!(
                r"{}[\q{{a|ab}}&&a]|{}{{2,999999999999999999999999999999999999999}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..3,
            Some(2..3),
        ),
        (
            format!(
                r"(|[\q{{{}}}&&a]){{2,999999999999999999999999999999999999999}}?",
                "x".repeat(20_000)
            ),
            1,
            0..0,
            Some(0..0),
        ),
        (
            format!(
                r"([\q{{a|ab}}&&a]|){{{}2,999999999999999999999999999999999999999}}",
                "0".repeat(20_000)
            ),
            1,
            0..3,
            Some(2..3),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0usize;
        let original =
            compile_overflowing_maximum_string_with_work(&source, false, false, false, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert!(
            work < source.len() * 38 + 1024,
            "linear construction work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), captures);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        assert_eq!(found.atom_capture, capture);
    }
}

#[test]
fn overflowing_maximum_string_braces_keep_exact_differences_and_closed_domains() {
    let limit = usize::MAX as u128;
    for (singleton, class) in [(true, r"[\q{a|b}]"), (false, r"[\q{a|b|ab}&&\p{Any}]")] {
        for (minimum, maximum, empty_first, lazy, input, range, capture) in [
            (limit, limit + 1, true, false, "aaa", 0..1, 0..1),
            (limit, limit + 1, false, false, "aaa", 0..3, 3..3),
            (limit, limit + 1, false, true, "aaa", 0..3, 3..3),
            (limit, limit + 1, true, true, "aaa", 0..0, 0..0),
            (limit - 1, limit + 1, true, false, "aaa", 0..2, 1..2),
            (limit, limit + 3, true, false, "aaaa", 0..3, 2..3),
            (1, limit + 1, true, false, "aaa", 0..3, 2..3),
            (3, limit * 2 + 3, false, true, "abX", 0..2, 2..2),
        ] {
            let inner = if empty_first {
                format!("|{class}")
            } else {
                format!("{class}|")
            };
            let source = JsString::from(
                format!(
                    "(({inner})){{000{minimum},00{maximum}}}{}",
                    if lazy { "?" } else { "" }
                )
                .as_str(),
            );
            let plan = compile_overflowing_maximum_string_with_work(
                &source,
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
            assert_eq!(plan.capture_count(), 2);
            let found = plan
                .find_with_captures_and_work(&JsString::from(input), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, range, "{source:?}");
            assert_eq!(found.atom_capture, Some(capture), "{source:?}");
            assert!(
                compile_positive_finite_string_with_work(
                    &source,
                    false,
                    false,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
            assert!(
                compile_positive_open_string_with_work(&source, false, false, singleton, |_| Ok::<
                    (),
                    (),
                >(
                    ()
                ))
                .unwrap()
                .is_none()
            );
        }
        let source = JsString::from(format!("(|{class}){{{limit},{}}}$", limit + 1).as_str());
        let plan =
            compile_overflowing_maximum_string_with_work(&source, false, false, singleton, |_| {
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from("aa"), 0, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..2); // Higher phase starts earlier than phase one.
        assert_eq!(found.atom_capture, Some(1..2));
        for tail in [
            "*",
            "+?",
            "?",
            "{0,999999999999999999999999999999999999999}",
            "{1}",
            "{2,3}",
            "{3,}",
            "{999999999999999999999999999999999999999}",
            "{999999999999999999999999999999999999999,9999999999999999999999999999999999999999}",
        ] {
            let source = JsString::from(format!("({class}|){tail}").as_str());
            assert!(
                compile_overflowing_maximum_string_with_work(
                    &source,
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
        for inner in [
            format!("({class})|"),
            format!("{class}|()"),
            format!("{class}||"),
            format!(r"\b{class}|"),
            format!("{class}|[b]"),
            r"|[\q{ab}&&\p{Any}&&\p{RGI_Emoji}]".to_owned(),
            r"[\q{ab}&&\q{cd}]|".to_owned(),
            r"[\q{}]|".to_owned(),
        ] {
            let source = JsString::from(format!("({inner}){{2,{}}}", limit + 1).as_str());
            assert!(
                compile_overflowing_maximum_string_with_work(
                    &source,
                    true,
                    true,
                    singleton,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source:?}"
            );
        }
    }
    let source =
        JsString::from(format!(r"(|[\q{{😀|😀x}}&&\p{{Any}}]){{{limit},{}}}$", limit + 1).as_str());
    let plan = compile_overflowing_maximum_string_with_work(&source, false, false, false, |_| {
        Ok::<(), ()>(())
    })
    .unwrap()
    .unwrap();
    let found = plan
        .find_with_captures_and_work(&JsString::from("😀😀"), 1, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..4);
    assert_eq!(found.atom_capture, Some(2..4));
    for (singleton, class) in [(true, r"[\q{a|ab}&&a]"), (false, r"[\q{a|b}]")] {
        let source = JsString::from(format!("({class}|){{2,{}}}", limit + 1).as_str());
        assert!(
            compile_overflowing_maximum_string_with_work(
                &source,
                false,
                false,
                singleton,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}
