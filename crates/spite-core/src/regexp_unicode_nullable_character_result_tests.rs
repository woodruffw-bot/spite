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
