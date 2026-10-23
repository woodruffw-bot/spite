use super::RegExpUnicodeNullableClassCaptureMatcher as Matcher;
use crate::JsString;
use std::ops::Range;

type Predicate = fn(u32) -> bool;
#[derive(Clone, Debug, PartialEq, Eq)]
struct Expected {
    range: Range<usize>,
    captures: Vec<Option<Range<usize>>>,
}

// Independent small-input RepeatMatcher: reset captures before evaluating each
// alternative, write only groups actually entered, then invoke the continuation.
struct Oracle<'a> {
    input: &'a JsString,
    contains: Predicate,
    minimum: usize,
    maximum: Option<usize>,
    greedy: bool,
    empty_first: bool,
    roots: usize,
    captures: usize,
    empty_slots: &'a [usize],
    never_slots: &'a [usize],
    i: bool,
    m: bool,
    anchors: bool,
    words: [Option<bool>; 2],
}
impl Oracle<'_> {
    fn point(&self, at: usize) -> Option<(u32, usize)> {
        let units = self.input.code_units();
        let first = *units.get(at)?;
        if (0xd800..=0xdbff).contains(&first) {
            if let Some(&second) = units
                .get(at + 1)
                .filter(|&&p| (0xdc00..=0xdfff).contains(&p))
            {
                return Some((
                    0x10000 + (u32::from(first) - 0xd800) * 0x400 + u32::from(second) - 0xdc00,
                    at + 2,
                ));
            }
        }
        Some((u32::from(first), at + 1))
    }
    fn word(&self, p: u32) -> bool {
        matches!(p,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a)
            || (self.i && matches!(p, 0x17f | 0x212a))
    }
    fn boundary(&self, at: usize) -> bool {
        let mut before = false;
        let mut cursor = 0;
        while cursor < at {
            let (p, next) = self.point(cursor).unwrap();
            before = self.word(p);
            cursor = next;
        }
        before != self.point(at).is_some_and(|(p, _)| self.word(p))
    }
    fn line(p: u16) -> bool {
        matches!(p, 0xa | 0xd | 0x2028 | 0x2029)
    }
    fn end(&self, at: usize) -> bool {
        (!self.anchors
            || at == self.input.len()
            || (self.m
                && self
                    .input
                    .code_units()
                    .get(at)
                    .is_some_and(|&p| Self::line(p))))
            && self.words[1].is_none_or(|word| self.boundary(at) == word)
    }
    fn accept(
        &self,
        at: usize,
        captures: &[Option<Range<usize>>],
    ) -> Option<(usize, Vec<Option<Range<usize>>>)> {
        self.end(at).then(|| (at, captures.to_vec()))
    }
    fn repeat(
        &self,
        at: usize,
        min: usize,
        max: Option<usize>,
        captures: &[Option<Range<usize>>],
    ) -> Option<(usize, Vec<Option<Range<usize>>>)> {
        if max == Some(0) {
            return (min == 0).then(|| self.accept(at, captures)).flatten();
        }
        if min != 0 {
            return self.step(at, min, max);
        }
        if self.greedy {
            self.step(at, min, max)
                .or_else(|| self.accept(at, captures))
        } else {
            self.accept(at, captures)
                .or_else(|| self.step(at, min, max))
        }
    }
    fn step(
        &self,
        at: usize,
        min: usize,
        max: Option<usize>,
    ) -> Option<(usize, Vec<Option<Range<usize>>>)> {
        for consuming in if self.empty_first {
            [false, true]
        } else {
            [true, false]
        } {
            let next = if consuming {
                let Some((mut p, next)) = self.point(at) else {
                    continue;
                };
                if self.i {
                    p = match p {
                        0x41..=0x5a => p + 32,
                        0x17f => 0x73,
                        0x212a => 0x6b,
                        0x10400..=0x10427 => p + 0x28,
                        _ => p,
                    };
                }
                if !(self.contains)(p) {
                    continue;
                }
                next
            } else {
                if min == 0 {
                    continue;
                }
                at
            };
            let mut fresh = vec![None; self.captures];
            for slot in fresh.iter_mut().take(self.roots) {
                *slot = Some(at..next);
            }
            if consuming {
                for (index, slot) in fresh.iter_mut().enumerate().skip(self.roots) {
                    if !self.empty_slots.contains(&index) && !self.never_slots.contains(&index) {
                        *slot = Some(at..next);
                    }
                }
            } else {
                for &index in self.empty_slots {
                    fresh[index] = Some(at..at);
                }
            }
            if let Some(found) =
                self.repeat(next, min.saturating_sub(1), max.map(|n| n - 1), &fresh)
            {
                return Some(found);
            }
        }
        None
    }
    fn find(&self, requested: usize, sticky: bool) -> Option<Expected> {
        let units = self.input.code_units();
        if requested > units.len() {
            return None;
        }
        let mut start = requested;
        // The narrowly approved leading-boundary behavior for an interior start.
        if start > 0
            && start < units.len()
            && (0xdc00..=0xdfff).contains(&units[start])
            && (0xd800..=0xdbff).contains(&units[start - 1])
        {
            start -= 1;
        }
        loop {
            let head = (!self.anchors
                || start == 0
                || (self.m && units.get(start - 1).is_some_and(|&p| Self::line(p))))
                && self.words[0].is_none_or(|word| self.boundary(start) == word);
            if head {
                if let Some((end, captures)) = self.repeat(
                    start,
                    self.minimum,
                    self.maximum,
                    &vec![None; self.captures],
                ) {
                    return Some(Expected {
                        range: start..end,
                        captures,
                    });
                }
            }
            if sticky || start == units.len() {
                return None;
            }
            start = self.point(start).unwrap().1;
        }
    }
}

#[test]
fn partial_class_captures_follow_independent_capture_reset_and_branch_oracle() {
    let classes: [(&str, Predicate); 8] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
        (r"[\q{s|k}]", |p| p == 0x73 || p == 0x6b),
        (r"[\q{\(|\)|a}]", |p| p == 0x28 || p == 0x29 || p == 97),
    ];
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for (class, contains) in classes {
        for (open, close, roots, inner_open, inner_close, inner_captures) in [
            ("(", ")", 1, "(", ")", 1),
            ("(?:", ")", 0, "((", "))", 2),
            ("((?:", "))", 1, "(?:((", ")))", 2),
        ] {
            for empty_first in [false, true] {
                for (count, minimum, optional, greedy) in [
                    ("*".to_owned(), Some(0), None, true),
                    ("*?".to_owned(), Some(0), None, false),
                    ("+".to_owned(), Some(1), None, true),
                    ("+?".to_owned(), Some(1), None, false),
                    ("?".to_owned(), Some(0), Some(1), true),
                    ("??".to_owned(), Some(0), Some(1), false),
                    ("{0}".to_owned(), Some(0), Some(0), true),
                    ("{0,3}?".to_owned(), Some(0), Some(3), false),
                    ("{01,001}?".to_owned(), Some(1), Some(0), false),
                    ("{1,}".to_owned(), Some(1), None, true),
                    ("{2,2}?".to_owned(), Some(2), Some(0), false),
                    ("{2,3}".to_owned(), Some(2), Some(1), true),
                    ("{2,3}?".to_owned(), Some(2), Some(1), false),
                    ("{3,}?".to_owned(), Some(3), None, false),
                    (format!("{{0,{huge}}}"), Some(0), None, true),
                    (format!("{{1,{huge}}}?"), Some(1), None, false),
                    (format!("{{{limit},{}}}", limit + 1), None, Some(1), true),
                    (format!("{{{limit},{}}}?", limit + 1), None, Some(1), false),
                    (format!("{{{huge}}}?"), None, Some(0), false),
                    (format!("{{00{huge},0{huge}}}"), None, Some(0), true),
                    (format!("{{{huge},}}?"), None, None, false),
                    (format!("{{{huge},{}}}", huge + 1), None, Some(1), true),
                    (format!("{{{huge},{}}}?", huge + 1), None, Some(1), false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let branch = format!("{inner_open}{class}{inner_close}");
                                let branch = if empty_first {
                                    format!("|{branch}")
                                } else {
                                    format!("{branch}|")
                                };
                                let source =
                                    format!("{prefix}{open}{branch}{close}{count}{suffix}");
                                let plan = Matcher::compile_with_work(
                                    &JsString::from(source.as_str()),
                                    i,
                                    m,
                                    |_| Ok::<(), ()>(()),
                                )
                                .unwrap()
                                .unwrap();
                                assert_eq!(plan.capture_count(), roots + inner_captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from("(a))!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let minimum = minimum.unwrap_or(input.len() + 1);
                                    let oracle = Oracle {
                                        input: &input,
                                        contains,
                                        minimum,
                                        maximum: optional.map(|n| minimum + n),
                                        greedy,
                                        empty_first,
                                        roots,
                                        captures: roots + inner_captures,
                                        empty_slots: &[],
                                        never_slots: &[],
                                        i,
                                        m,
                                        anchors,
                                        words,
                                    };
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            let found = plan
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(()),
                                                )
                                                .unwrap();
                                            let projected = found.map(|found| Expected {
                                                captures: (0..plan.capture_count())
                                                    .map(|slot| {
                                                        plan.capture_range(
                                                            slot,
                                                            found.atom_capture.as_ref(),
                                                        )
                                                    })
                                                    .collect(),
                                                range: found.range,
                                            });
                                            assert_eq!(
                                                projected,
                                                oracle.find(start, sticky),
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
fn partial_class_capture_construction_cancels_each_callback_and_keeps_large_layouts_flat() {
    for pattern in [
        r"^\b(([\q{a|b}])|){2,3}?\b$",
        r"(?:(?:|(([\q{ab}&&a])))){0}",
        r"((|([\q{😀|😀x}&&\p{Any}]))){999999999999999999999999999999,1000000000000000000000000000000}?",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Matcher::compile_with_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 16);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = Matcher::compile_with_work(&source, true, true, |n| {
                seen.push(n);
                if seen.len() == fail {
                    Err(fail)
                } else {
                    Ok(())
                }
            });
            assert!(
                matches!(result,Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at==fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        Matcher::compile_with_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for (source, count, range, iteration, character) in [
        (
            format!(
                r"({}[\q{{a|ab}}&&a]{}|){{999999999999999999999999999999,1000000000000000000000000000000}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_001,
            0..3,
            Some(3..3),
            None,
        ),
        (
            format!(
                r"(?:|{}[\q{{a|b}}]{}){{2,3}}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            ),
            20_000,
            0..1,
            None,
            Some(0..1),
        ),
        (
            format!(r"(([\q{{{}}}&&a])|)+", "x".repeat(20_000)),
            2,
            0..0,
            Some(0..0),
            None,
        ),
        (
            format!(r"(([\q{{a|b}}])|){{{}2,3}}", "0".repeat(20_000)),
            2,
            0..3,
            Some(2..3),
            Some(2..3),
        ),
    ] {
        let source = JsString::from(source.as_str());
        let mut work = 0;
        let original = Matcher::compile_with_work(&source, false, false, |n| {
            work += n;
            Ok::<(), ()>(())
        })
        .unwrap()
        .unwrap();
        assert!(
            work < source.len() * 144 + 1024,
            "linear checked work: {work}"
        );
        let plan = original.clone();
        drop(original);
        assert_eq!(plan.capture_count(), count);
        let found = plan
            .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, range);
        if let Some(iteration) = iteration {
            assert_eq!(
                plan.capture_range(0, found.atom_capture.as_ref()),
                Some(iteration)
            );
        }
        assert_eq!(
            plan.capture_range(count - 1, found.atom_capture.as_ref()),
            character
        );
        assert_eq!(plan.capture_range(count, found.atom_capture.as_ref()), None);
    }
}

#[test]
fn partial_class_capture_proofs_reject_other_capture_effect_and_leaf_shapes() {
    for source in [
        r"([\q{a|b}]|)+",
        r"(([\q{a|b}])|())+",
        r"(([\q{a|b}])|(?:))+",
        r"(([\q{a|b}])||)+",
        r"((\b[\q{a|b}])|)+",
        r"(([\q{a|b}])+|)+",
        r"(([\q{ab}&&\q{cd}])|){0}",
        r"(|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
        r"(([\q{a|b}])|)+[b]",
        r"(([\q{a|b}])|)",
        r"(([\q{a|b}])|)+(?:)",
    ] {
        assert!(
            Matcher::compile_with_work(&JsString::from(source), true, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_none(),
            "{source}"
        );
    }
}
#[test]
fn empty_branch_captures_follow_independent_capture_reset_and_branch_oracle() {
    let classes: [(&str, Predicate); 8] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
        (r"[\q{s|k}]", |p| p == 0x73 || p == 0x6b),
        (r"[\q{\(|\)|a}]", |p| p == 0x28 || p == 0x29 || p == 97),
    ];
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for (class, contains) in classes {
        for (open, close, roots, inner_open, inner_close, inner_captures) in [
            ("(", ")", 1, "(", ")", 1),
            ("(?:", ")", 0, "((", "))", 2),
            ("((?:", "))", 1, "(?:((", ")))", 2),
        ] {
            for empty_first in [false, true] {
                for (empty, empty_count) in [("()", 1), ("(?:(())())", 3), ("(?:())(?:())", 2)] {
                    for (count, minimum, optional, greedy) in [
                        ("*".to_owned(), Some(0), None, true),
                        ("*?".to_owned(), Some(0), None, false),
                        ("+".to_owned(), Some(1), None, true),
                        ("+?".to_owned(), Some(1), None, false),
                        ("?".to_owned(), Some(0), Some(1), true),
                        ("??".to_owned(), Some(0), Some(1), false),
                        ("{0}".to_owned(), Some(0), Some(0), true),
                        ("{0,3}?".to_owned(), Some(0), Some(3), false),
                        ("{01,001}?".to_owned(), Some(1), Some(0), false),
                        ("{1,}".to_owned(), Some(1), None, true),
                        ("{2,2}?".to_owned(), Some(2), Some(0), false),
                        ("{2,3}".to_owned(), Some(2), Some(1), true),
                        ("{2,3}?".to_owned(), Some(2), Some(1), false),
                        ("{3,}?".to_owned(), Some(3), None, false),
                        (format!("{{0,{huge}}}"), Some(0), None, true),
                        (format!("{{1,{huge}}}?"), Some(1), None, false),
                        (format!("{{{limit},{}}}", limit + 1), None, Some(1), true),
                        (format!("{{{limit},{}}}?", limit + 1), None, Some(1), false),
                        (format!("{{{huge}}}?"), None, Some(0), false),
                        (format!("{{00{huge},0{huge}}}"), None, Some(0), true),
                        (format!("{{{huge},}}?"), None, None, false),
                        (format!("{{{huge},{}}}", huge + 1), None, Some(1), true),
                        (format!("{{{huge},{}}}?", huge + 1), None, Some(1), false),
                    ] {
                        for (prefix, suffix, anchors, words) in [
                            ("", "", false, [None, None]),
                            (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                            (r"\B", r"\b", false, [Some(false), Some(true)]),
                        ] {
                            for i in [false, true] {
                                for m in [false, true] {
                                    let branch = format!("{inner_open}{class}{inner_close}");
                                    let branch = if empty_first {
                                        format!("{empty}|{branch}")
                                    } else {
                                        format!("{branch}|{empty}")
                                    };
                                    let source =
                                        format!("{prefix}{open}{branch}{close}{count}{suffix}");
                                    let plan =
                                        Matcher::compile_with_empty_branch_captures_and_work(
                                            &JsString::from(source.as_str()),
                                            i,
                                            m,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap()
                                        .unwrap();
                                    assert_eq!(
                                        plan.capture_count(),
                                        roots + inner_captures + empty_count
                                    );
                                    let empty_start =
                                        roots + if empty_first { 0 } else { inner_captures };
                                    let empty_slots = (empty_start..empty_start + empty_count)
                                        .collect::<Vec<_>>();
                                    for input in [
                                        JsString::from(""),
                                        JsString::from("aaab!aba"),
                                        JsString::from("A😀😀aa"),
                                        JsString::from("!\r\naaa\u{2028}!"),
                                        JsString::from("ſK"),
                                        JsString::from("𐐀𐐨!"),
                                        JsString::from("(a))!"),
                                        JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                    ] {
                                        let minimum = minimum.unwrap_or(input.len() + 1);
                                        let oracle = Oracle {
                                            input: &input,
                                            contains,
                                            minimum,
                                            maximum: optional.map(|n| minimum + n),
                                            greedy,
                                            empty_first,
                                            roots,
                                            captures: roots + inner_captures + empty_count,
                                            empty_slots: &empty_slots,
                                            never_slots: &[],
                                            i,
                                            m,
                                            anchors,
                                            words,
                                        };
                                        for start in 0..=input.len() + 1 {
                                            for sticky in [false, true] {
                                                let found = plan
                                                    .find_with_captures_and_work(
                                                        &input,
                                                        start,
                                                        sticky,
                                                        |_| Ok::<(), ()>(()),
                                                    )
                                                    .unwrap();
                                                let projected = found.map(|found| Expected {
                                                    captures: (0..plan.capture_count())
                                                        .map(|slot| {
                                                            plan.capture_range(
                                                                slot,
                                                                found.atom_capture.as_ref(),
                                                            )
                                                        })
                                                        .collect(),
                                                    range: found.range,
                                                });
                                                assert_eq!(
                                                    projected,
                                                    oracle.find(start, sticky),
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
}

#[test]
fn empty_branch_capture_proofs_cancel_each_callback_and_keep_large_layouts_flat() {
    for pattern in [
        r"^\b(([\q{a|b}])|(())){2,3}?\b$",
        r"(?:()|(([\q{ab}&&a]))){0}",
        r"((()|([\q{😀|😀x}&&\p{Any}]))){999999999999999999999999999999,1000000000000000000000000000000}?",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Matcher::compile_with_empty_branch_captures_and_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                Matcher::compile_with_empty_branch_captures_and_work(&source, true, true, |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(
                matches!(result,Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at==fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        Matcher::compile_with_empty_branch_captures_and_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for empty_first in [false, true] {
        for deep in [false, true] {
            let empty = if deep {
                format!("{}{}", "(".repeat(20_000), ")".repeat(20_000))
            } else {
                "()".repeat(20_000)
            };
            let branch = if empty_first {
                format!(r"{empty}|([\q{{a|b}}])")
            } else {
                format!(r"([\q{{a|b}}])|{empty}")
            };
            let source = format!(
                "({branch}){{999999999999999999999999999999,1000000000000000000000000000000}}"
            );
            let source = JsString::from(source.as_str());
            let mut work = 0;
            let original =
                Matcher::compile_with_empty_branch_captures_and_work(&source, false, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap()
                .unwrap();
            assert!(work < source.len() * 144 + 1024, "{work}");
            let plan = original.clone();
            drop(original);
            assert_eq!(plan.capture_count(), 20_002);
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            let end = if empty_first { 1 } else { 3 };
            assert_eq!(found.range, 0..end);
            assert_eq!(
                plan.capture_range(0, found.atom_capture.as_ref()),
                Some(if empty_first { 0..1 } else { 3..3 })
            );
            let char_slot = if empty_first { 20_001 } else { 1 };
            assert_eq!(
                plan.capture_range(char_slot, found.atom_capture.as_ref()),
                if empty_first { Some(0..1) } else { None }
            );
            for slot in 1..20_002 {
                if slot != char_slot {
                    assert_eq!(
                        plan.capture_range(slot, found.atom_capture.as_ref()),
                        if empty_first { None } else { Some(3..3) }
                    );
                }
            }
            assert_eq!(
                plan.capture_range(20_002, found.atom_capture.as_ref()),
                None
            );
        }
    }
    {
        let source = format!(r"(([\q{{{}}}&&a])|())+", "x".repeat(20_000));
        let source = JsString::from(source.as_str());
        let plan =
            Matcher::compile_with_empty_branch_captures_and_work(&source, false, false, |_| {
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from("😀"), 1, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 0..0);
        assert_eq!(
            plan.capture_range(0, found.atom_capture.as_ref()),
            Some(0..0)
        );
        assert_eq!(plan.capture_range(1, found.atom_capture.as_ref()), None);
        assert_eq!(
            plan.capture_range(2, found.atom_capture.as_ref()),
            Some(0..0)
        );
    }
}

#[test]
fn empty_branch_capture_proofs_keep_other_effects_and_leaf_domains_closed() {
    for source in [
        r"([\q{a|b}]|)+",
        r"(([\q{a|b}])|)+",
        r"(([\q{a|b}])|(?:))+",
        r"(([\q{a|b}])|()*)+",
        r"(([\q{a|b}])|(()|))+",
        r"(([\q{a|b}])|())?()+",
        r"(()([\q{a|b}])|())+",
        r"(([\q{a|b}])()|())+",
        r"(([\q{a|b}])||())+",
        r"((\b[\q{a|b}])|())+",
        r"(([\q{a|b}])+|())+",
        r"(([\q{a|b}])|\b())+",
        r"(([\q{ab}&&\q{cd}])|()){0}",
        r"(()|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
        r"(([\q{a|b}])|())+[b]",
        r"(([\q{a|b}])|())",
        r"(([\q{a|b}])|())+(?:)",
    ] {
        assert!(
            Matcher::compile_with_empty_branch_captures_and_work(
                &JsString::from(source),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}
#[test]
fn partial_captures_beside_noncapturing_empty_sequences_follow_independent_branch_oracle() {
    let classes: [(&str, Predicate); 8] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
        (r"[\q{s|k}]", |p| p == 0x73 || p == 0x6b),
        (r"[\q{\(|\)|a}]", |p| p == 0x28 || p == 0x29 || p == 97),
    ];
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for (class, contains) in classes {
        for (open, close, roots, inner_open, inner_close, inner_captures) in [
            ("(", ")", 1, "(", ")", 1),
            ("(?:", ")", 0, "((", "))", 2),
            ("((?:", "))", 1, "(?:((", ")))", 2),
        ] {
            for empty_first in [false, true] {
                for (count, minimum, optional, greedy) in [
                    ("*".to_owned(), Some(0), None, true),
                    ("*?".to_owned(), Some(0), None, false),
                    ("+".to_owned(), Some(1), None, true),
                    ("+?".to_owned(), Some(1), None, false),
                    ("?".to_owned(), Some(0), Some(1), true),
                    ("??".to_owned(), Some(0), Some(1), false),
                    ("{0}".to_owned(), Some(0), Some(0), true),
                    ("{0,3}?".to_owned(), Some(0), Some(3), false),
                    ("{01,001}?".to_owned(), Some(1), Some(0), false),
                    ("{1,}".to_owned(), Some(1), None, true),
                    ("{2,2}?".to_owned(), Some(2), Some(0), false),
                    ("{2,3}".to_owned(), Some(2), Some(1), true),
                    ("{2,3}?".to_owned(), Some(2), Some(1), false),
                    ("{3,}?".to_owned(), Some(3), None, false),
                    (format!("{{0,{huge}}}"), Some(0), None, true),
                    (format!("{{1,{huge}}}?"), Some(1), None, false),
                    (format!("{{{limit},{}}}", limit + 1), None, Some(1), true),
                    (format!("{{{limit},{}}}?", limit + 1), None, Some(1), false),
                    (format!("{{{huge}}}?"), None, Some(0), false),
                    (format!("{{00{huge},0{huge}}}"), None, Some(0), true),
                    (format!("{{{huge},}}?"), None, None, false),
                    (format!("{{{huge},{}}}", huge + 1), None, Some(1), true),
                    (format!("{{{huge},{}}}?", huge + 1), None, Some(1), false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let branch = format!("{inner_open}{class}{inner_close}");
                                let branch = if empty_first {
                                    format!("(?:(?:))(?:)|{branch}")
                                } else {
                                    format!("{branch}|(?:(?:))(?:)")
                                };
                                let source =
                                    format!("{prefix}{open}{branch}{close}{count}{suffix}");
                                let plan =
                                    Matcher::compile_with_noncapturing_empty_branch_and_work(
                                        &JsString::from(source.as_str()),
                                        i,
                                        m,
                                        |_| Ok::<(), ()>(()),
                                    )
                                    .unwrap()
                                    .unwrap();
                                assert_eq!(plan.capture_count(), roots + inner_captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from("(a))!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let minimum = minimum.unwrap_or(input.len() + 1);
                                    let oracle = Oracle {
                                        input: &input,
                                        contains,
                                        minimum,
                                        maximum: optional.map(|n| minimum + n),
                                        greedy,
                                        empty_first,
                                        roots,
                                        captures: roots + inner_captures,
                                        empty_slots: &[],
                                        never_slots: &[],
                                        i,
                                        m,
                                        anchors,
                                        words,
                                    };
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            let found = plan
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(()),
                                                )
                                                .unwrap();
                                            let projected = found.map(|found| Expected {
                                                captures: (0..plan.capture_count())
                                                    .map(|slot| {
                                                        plan.capture_range(
                                                            slot,
                                                            found.atom_capture.as_ref(),
                                                        )
                                                    })
                                                    .collect(),
                                                range: found.range,
                                            });
                                            assert_eq!(
                                                projected,
                                                oracle.find(start, sticky),
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
fn partial_captures_beside_empty_sequences_cancel_each_callback_and_keep_large_sources_flat() {
    for pattern in [
        r"^\b(([\q{a|b}])|(?:(?:))(?:)){2,3}?\b$",
        r"(?:(?:)|(([\q{ab}&&a]))){0}",
        r"(((?:(?:))(?:)|([\q{😀|😀x}&&\p{Any}]))){999999999999999999999999999999,1000000000000000000000000000000}?",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Matcher::compile_with_noncapturing_empty_branch_and_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = Matcher::compile_with_noncapturing_empty_branch_and_work(
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
        Matcher::compile_with_noncapturing_empty_branch_and_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete, recovered);
    }
    for empty_first in [false, true] {
        for deep in [false, true] {
            let empty = if deep {
                format!("{}{}", "(?:".repeat(20_000), ")".repeat(20_000))
            } else {
                "(?:)".repeat(20_000)
            };
            let character = format!(
                r"{}[\q{{a|b|ab}}&&\p{{Any}}]{}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            );
            let branch = if empty_first {
                format!("{empty}|{character}")
            } else {
                format!("{character}|{empty}")
            };
            let source = format!(
                "({branch}){{999999999999999999999999999999,1000000000000000000000000000000}}"
            );
            let source = JsString::from(source.as_str());
            let mut work = 0;
            let original = Matcher::compile_with_noncapturing_empty_branch_and_work(
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
            assert!(work < source.len() * 144 + 1024, "{work}");
            let plan = original.clone();
            drop(original);
            assert_eq!(plan.capture_count(), 20_001);
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..if empty_first { 1 } else { 3 });
            assert_eq!(
                plan.capture_range(0, found.atom_capture.as_ref()),
                Some(if empty_first { 0..1 } else { 3..3 })
            );
            for slot in 1..=20_000 {
                assert_eq!(
                    plan.capture_range(slot, found.atom_capture.as_ref()),
                    if empty_first { Some(0..1) } else { None }
                );
            }
            assert_eq!(
                plan.capture_range(20_001, found.atom_capture.as_ref()),
                None
            );
        }
    }
}

#[test]
fn partial_captures_beside_empty_sequences_keep_other_capture_effect_and_leaf_domains_closed() {
    for source in [
        r"([\q{a|b}]|(?:))+",
        r"(([\q{a|b}])|)+",
        r"(([\q{a|b}])|())+",
        r"(([\q{a|b}])|(?:)*)+",
        r"(([\q{a|b}])|(?:|))+",
        r"(([\q{a|b}])|(?:))?()+",
        r"((?:)([\q{a|b}])|(?:))+",
        r"(([\q{a|b}])(?:)|(?:))+",
        r"(([\q{a|b}])||(?:))+",
        r"((\b[\q{a|b}])|(?:))+",
        r"(([\q{a|b}])+|(?:))+",
        r"(([\q{a|b}])|\b(?:))+",
        r"(([\q{ab}&&\q{cd}])|(?:)){0}",
        r"((?:)|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
        r"(([\q{a|b}])|(?:))+[b]",
        r"(([\q{a|b}])|(?:))",
        r"(([\q{a|b}])|(?:))+(?:)",
    ] {
        assert!(
            Matcher::compile_with_noncapturing_empty_branch_and_work(
                &JsString::from(source),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}
#[test]
fn partial_captures_beside_quantified_noncapturing_empty_sequences_follow_independent_branch_oracle()
 {
    let classes: [(&str, Predicate); 8] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
        (r"[\q{s|k}]", |p| p == 0x73 || p == 0x6b),
        (r"[\q{\(|\)|a}]", |p| p == 0x28 || p == 0x29 || p == 97),
    ];
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for (class, contains) in classes {
        for (open, close, roots, inner_open, inner_close, inner_captures) in [
            ("(", ")", 1, "(", ")", 1),
            ("(?:", ")", 0, "((", "))", 2),
            ("((?:", "))", 1, "(?:((", ")))", 2),
        ] {
            for empty_first in [false, true] {
                for (count, minimum, optional, greedy) in [
                    ("*".to_owned(), Some(0), None, true),
                    ("*?".to_owned(), Some(0), None, false),
                    ("+".to_owned(), Some(1), None, true),
                    ("+?".to_owned(), Some(1), None, false),
                    ("?".to_owned(), Some(0), Some(1), true),
                    ("??".to_owned(), Some(0), Some(1), false),
                    ("{0}".to_owned(), Some(0), Some(0), true),
                    ("{0,3}?".to_owned(), Some(0), Some(3), false),
                    ("{01,001}?".to_owned(), Some(1), Some(0), false),
                    ("{1,}".to_owned(), Some(1), None, true),
                    ("{2,2}?".to_owned(), Some(2), Some(0), false),
                    ("{2,3}".to_owned(), Some(2), Some(1), true),
                    ("{2,3}?".to_owned(), Some(2), Some(1), false),
                    ("{3,}?".to_owned(), Some(3), None, false),
                    (format!("{{0,{huge}}}"), Some(0), None, true),
                    (format!("{{1,{huge}}}?"), Some(1), None, false),
                    (format!("{{{limit},{}}}", limit + 1), None, Some(1), true),
                    (format!("{{{limit},{}}}?", limit + 1), None, Some(1), false),
                    (format!("{{{huge}}}?"), None, Some(0), false),
                    (format!("{{00{huge},0{huge}}}"), None, Some(0), true),
                    (format!("{{{huge},}}?"), None, None, false),
                    (format!("{{{huge},{}}}", huge + 1), None, Some(1), true),
                    (format!("{{{huge},{}}}?", huge + 1), None, Some(1), false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let branch = format!("{inner_open}{class}{inner_close}");
                                let branch = if empty_first {
                                    format!("(?:(?:)*?)(?:)+?|{branch}")
                                } else {
                                    format!("{branch}|(?:(?:)*?)(?:)+?")
                                };
                                let source =
                                    format!("{prefix}{open}{branch}{close}{count}{suffix}");
                                let plan =
                                    Matcher::compile_with_quantified_noncapturing_empty_branch_and_work(
                                        &JsString::from(source.as_str()),
                                        i,
                                        m,
                                        |_| Ok::<(), ()>(()),
                                    )
                                    .unwrap()
                                    .unwrap();
                                assert_eq!(plan.capture_count(), roots + inner_captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from("(a))!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let minimum = minimum.unwrap_or(input.len() + 1);
                                    let oracle = Oracle {
                                        input: &input,
                                        contains,
                                        minimum,
                                        maximum: optional.map(|n| minimum + n),
                                        greedy,
                                        empty_first,
                                        roots,
                                        captures: roots + inner_captures,
                                        empty_slots: &[],
                                        never_slots: &[],
                                        i,
                                        m,
                                        anchors,
                                        words,
                                    };
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            let found = plan
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(()),
                                                )
                                                .unwrap();
                                            let projected = found.map(|found| Expected {
                                                captures: (0..plan.capture_count())
                                                    .map(|slot| {
                                                        plan.capture_range(
                                                            slot,
                                                            found.atom_capture.as_ref(),
                                                        )
                                                    })
                                                    .collect(),
                                                range: found.range,
                                            });
                                            assert_eq!(
                                                projected,
                                                oracle.find(start, sticky),
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
fn partial_captures_beside_quantified_empty_sequences_cancel_each_callback_and_keep_large_counts_flat()
 {
    for pattern in [
        r"^\b(([\q{a|b}])|(?:(?:)*?)(?:){0,999999999999999999999999999999999}?){2,3}?\b$",
        r"(?:(?:)*?|(([\q{ab}&&a]))){0}",
        r"(((?:(?:)*?)(?:){0,999999999999999999999999999999999}?|([\q{😀|😀x}&&\p{Any}]))){999999999999999999999999999999,1000000000000000000000000000000}?",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Matcher::compile_with_quantified_noncapturing_empty_branch_and_work(
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
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = Matcher::compile_with_quantified_noncapturing_empty_branch_and_work(
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
        Matcher::compile_with_quantified_noncapturing_empty_branch_and_work(
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
        assert_eq!(complete, recovered);
    }
    for empty_first in [false, true] {
        for deep in [false, true] {
            let empty = if deep {
                format!("{}{}", "(?:".repeat(20_000), ")".repeat(20_000))
            } else {
                "(?:)".repeat(20_000)
            };
            let empty = format!("(?:{empty}){{0,{}}}?", "9".repeat(20_000));
            let character = format!(
                r"{}[\q{{a|b|ab}}&&\p{{Any}}]{}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            );
            let branch = if empty_first {
                format!("{empty}|{character}")
            } else {
                format!("{character}|{empty}")
            };
            let source = format!(
                "({branch}){{999999999999999999999999999999,1000000000000000000000000000000}}"
            );
            let source = JsString::from(source.as_str());
            let mut work = 0;
            let original = Matcher::compile_with_quantified_noncapturing_empty_branch_and_work(
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
            assert!(work < source.len() * 144 + 1024, "{work}");
            let plan = original.clone();
            drop(original);
            assert_eq!(plan.capture_count(), 20_001);
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..if empty_first { 1 } else { 3 });
            assert_eq!(
                plan.capture_range(0, found.atom_capture.as_ref()),
                Some(if empty_first { 0..1 } else { 3..3 })
            );
            for slot in 1..=20_000 {
                assert_eq!(
                    plan.capture_range(slot, found.atom_capture.as_ref()),
                    if empty_first { Some(0..1) } else { None }
                );
            }
            assert_eq!(
                plan.capture_range(20_001, found.atom_capture.as_ref()),
                None
            );
        }
    }
}

#[test]
fn partial_captures_beside_quantified_empty_sequences_keep_other_capture_effect_and_leaf_domains_closed()
 {
    for source in [
        r"([\q{a|b}]|(?:))+",
        r"(([\q{a|b}])|)+",
        r"(([\q{a|b}])|())+",
        r"(([\q{a|b}])|(?:|)*)+",
        r"(([\q{a|b}])|(?:|))+",
        r"(([\q{a|b}])|(?:))?()+",
        r"((?:)([\q{a|b}])|(?:))+",
        r"(([\q{a|b}])(?:)|(?:))+",
        r"(([\q{a|b}])||(?:))+",
        r"((\b[\q{a|b}])|(?:))+",
        r"(([\q{a|b}])+|(?:))+",
        r"(([\q{a|b}])|\b(?:))+",
        r"(([\q{ab}&&\q{cd}])|(?:)*?){0}",
        r"((?:)*?|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
        r"(([\q{a|b}])|(?:))+[b]",
        r"(([\q{a|b}])|(?:))",
        r"(([\q{a|b}])|(?:))+(?:)",
    ] {
        assert!(
            Matcher::compile_with_quantified_noncapturing_empty_branch_and_work(
                &JsString::from(source),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}
#[test]
fn partial_captures_beside_noncapturing_empty_choices_follow_independent_branch_oracle() {
    let classes: [(&str, Predicate); 8] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
        (r"[\q{s|k}]", |p| p == 0x73 || p == 0x6b),
        (r"[\q{\(|\)|a}]", |p| p == 0x28 || p == 0x29 || p == 97),
    ];
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for (class, contains) in classes {
        for (open, close, roots, inner_open, inner_close, inner_captures) in [
            ("(", ")", 1, "(", ")", 1),
            ("(?:", ")", 0, "((", "))", 2),
            ("((?:", "))", 1, "(?:((", ")))", 2),
        ] {
            for empty_first in [false, true] {
                for (count, minimum, optional, greedy) in [
                    ("*".to_owned(), Some(0), None, true),
                    ("*?".to_owned(), Some(0), None, false),
                    ("+".to_owned(), Some(1), None, true),
                    ("+?".to_owned(), Some(1), None, false),
                    ("?".to_owned(), Some(0), Some(1), true),
                    ("??".to_owned(), Some(0), Some(1), false),
                    ("{0}".to_owned(), Some(0), Some(0), true),
                    ("{0,3}?".to_owned(), Some(0), Some(3), false),
                    ("{01,001}?".to_owned(), Some(1), Some(0), false),
                    ("{1,}".to_owned(), Some(1), None, true),
                    ("{2,2}?".to_owned(), Some(2), Some(0), false),
                    ("{2,3}".to_owned(), Some(2), Some(1), true),
                    ("{2,3}?".to_owned(), Some(2), Some(1), false),
                    ("{3,}?".to_owned(), Some(3), None, false),
                    (format!("{{0,{huge}}}"), Some(0), None, true),
                    (format!("{{1,{huge}}}?"), Some(1), None, false),
                    (format!("{{{limit},{}}}", limit + 1), None, Some(1), true),
                    (format!("{{{limit},{}}}?", limit + 1), None, Some(1), false),
                    (format!("{{{huge}}}?"), None, Some(0), false),
                    (format!("{{00{huge},0{huge}}}"), None, Some(0), true),
                    (format!("{{{huge},}}?"), None, None, false),
                    (format!("{{{huge},{}}}", huge + 1), None, Some(1), true),
                    (format!("{{{huge},{}}}?", huge + 1), None, Some(1), false),
                ] {
                    for (prefix, suffix, anchors, words) in [
                        ("", "", false, [None, None]),
                        (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                        (r"\B", r"\b", false, [Some(false), Some(true)]),
                    ] {
                        for i in [false, true] {
                            for m in [false, true] {
                                let branch = format!("{inner_open}{class}{inner_close}");
                                let branch = if empty_first {
                                    format!("(?:(?:|)*?)(?:|)+?|{branch}")
                                } else {
                                    format!("{branch}|(?:(?:|)*?)(?:|)+?")
                                };
                                let source =
                                    format!("{prefix}{open}{branch}{close}{count}{suffix}");
                                let plan =
                                    Matcher::compile_with_noncapturing_empty_choices_and_work(
                                        &JsString::from(source.as_str()),
                                        i,
                                        m,
                                        |_| Ok::<(), ()>(()),
                                    )
                                    .unwrap()
                                    .unwrap();
                                assert_eq!(plan.capture_count(), roots + inner_captures);
                                for input in [
                                    JsString::from(""),
                                    JsString::from("aaab!aba"),
                                    JsString::from("A😀😀aa"),
                                    JsString::from("!\r\naaa\u{2028}!"),
                                    JsString::from("ſK"),
                                    JsString::from("𐐀𐐨!"),
                                    JsString::from("(a))!"),
                                    JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                ] {
                                    let minimum = minimum.unwrap_or(input.len() + 1);
                                    let oracle = Oracle {
                                        input: &input,
                                        contains,
                                        minimum,
                                        maximum: optional.map(|n| minimum + n),
                                        greedy,
                                        empty_first,
                                        roots,
                                        captures: roots + inner_captures,
                                        empty_slots: &[],
                                        never_slots: &[],
                                        i,
                                        m,
                                        anchors,
                                        words,
                                    };
                                    for start in 0..=input.len() + 1 {
                                        for sticky in [false, true] {
                                            let found = plan
                                                .find_with_captures_and_work(
                                                    &input,
                                                    start,
                                                    sticky,
                                                    |_| Ok::<(), ()>(()),
                                                )
                                                .unwrap();
                                            let projected = found.map(|found| Expected {
                                                captures: (0..plan.capture_count())
                                                    .map(|slot| {
                                                        plan.capture_range(
                                                            slot,
                                                            found.atom_capture.as_ref(),
                                                        )
                                                    })
                                                    .collect(),
                                                range: found.range,
                                            });
                                            assert_eq!(
                                                projected,
                                                oracle.find(start, sticky),
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
fn partial_captures_beside_empty_choices_cancel_each_callback_and_keep_large_sources_flat() {
    for pattern in [
        r"^\b(([\q{a|b}])|(?:(?:|)*?)(?:){0,999999999999999999999999999999999}?){2,3}?\b$",
        r"(?:(?:|)*?|(([\q{ab}&&a]))){0}",
        r"(((?:(?:|)*?)(?:){0,999999999999999999999999999999999}?|([\q{😀|😀x}&&\p{Any}]))){999999999999999999999999999999,1000000000000000000000000000000}?",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Matcher::compile_with_noncapturing_empty_choices_and_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result = Matcher::compile_with_noncapturing_empty_choices_and_work(
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
        Matcher::compile_with_noncapturing_empty_choices_and_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete, recovered);
    }
    for empty_first in [false, true] {
        for deep in [false, true] {
            let empty = if deep {
                format!("{}{}", "(?:".repeat(20_000), ")".repeat(20_000))
            } else {
                "(?:)".repeat(20_000)
            };
            let empty = format!("(?:(?:|){empty}){{0,{}}}?", "9".repeat(20_000));
            let character = format!(
                r"{}[\q{{a|b|ab}}&&\p{{Any}}]{}",
                "(".repeat(20_000),
                ")".repeat(20_000)
            );
            let branch = if empty_first {
                format!("{empty}|{character}")
            } else {
                format!("{character}|{empty}")
            };
            let source = format!(
                "({branch}){{999999999999999999999999999999,1000000000000000000000000000000}}"
            );
            let source = JsString::from(source.as_str());
            let mut work = 0;
            let original = Matcher::compile_with_noncapturing_empty_choices_and_work(
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
            assert!(work < source.len() * 144 + 1024, "{work}");
            let plan = original.clone();
            drop(original);
            assert_eq!(plan.capture_count(), 20_001);
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..if empty_first { 1 } else { 3 });
            assert_eq!(
                plan.capture_range(0, found.atom_capture.as_ref()),
                Some(if empty_first { 0..1 } else { 3..3 })
            );
            for slot in 1..=20_000 {
                assert_eq!(
                    plan.capture_range(slot, found.atom_capture.as_ref()),
                    if empty_first { Some(0..1) } else { None }
                );
            }
            assert_eq!(
                plan.capture_range(20_001, found.atom_capture.as_ref()),
                None
            );
        }
    }
}

#[test]
fn partial_captures_beside_empty_choices_keep_capture_effect_and_class_branch_domains_closed() {
    for source in [
        r"((?:|)([\q{a|b}])|(?:|))+",
        r"(([\q{a|b}])|(?:|)|(?:|))+",
        r"([\q{a|b}]|(?:))+",
        r"(([\q{a|b}])|)+",
        r"(([\q{a|b}])|())+",
        r"(([\q{a|b}])|(()|)*)+",
        r"(([\q{a|b}])|(()|))+",
        r"(([\q{a|b}])|(?:))?()+",
        r"((?:)([\q{a|b}])|(?:))+",
        r"(([\q{a|b}])(?:)|(?:))+",
        r"(([\q{a|b}])||(?:))+",
        r"((\b[\q{a|b}])|(?:))+",
        r"(([\q{a|b}])+|(?:))+",
        r"(([\q{a|b}])|\b(?:))+",
        r"(([\q{ab}&&\q{cd}])|(?:|)*?){0}",
        r"((?:|)*?|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
        r"(([\q{a|b}])|(?:))+[b]",
        r"(([\q{a|b}])|(?:))",
        r"(([\q{a|b}])|(?:))+(?:)",
    ] {
        assert!(
            Matcher::compile_with_noncapturing_empty_choices_and_work(
                &JsString::from(source),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}
#[test]
fn empty_capture_effects_follow_independent_participation_and_capture_reset_oracle() {
    let classes: [(&str, Predicate); 8] = [
        (r"[\q{a|b}]", |p| p == 97 || p == 98),
        (r"[\q{a|b|ab}&&\p{Any}]", |p| p == 97 || p == 98),
        (r"[\q{ab}&&a]", |_| false),
        (r"[^\q{ab}&&a]", |_| true),
        (r"[\q{😀|😀x}&&\p{Any}]", |p| p == 0x1f600),
        (r"[\q{\u{10428}|ab}&&\p{Any}]", |p| p == 0x10428),
        (r"[\q{s|k}]", |p| p == 0x73 || p == 0x6b),
        (r"[\q{\(|\)|a}]", |p| p == 0x28 || p == 0x29 || p == 97),
    ];
    let limit = usize::MAX as u128;
    let huge = 10u128.pow(30);
    for (class, contains) in classes {
        for (open, close, roots, inner_open, inner_close, inner_captures) in [
            ("(", ")", 1, "(", ")", 1),
            ("(?:", ")", 0, "((", "))", 2),
            ("((?:", "))", 1, "(?:((", ")))", 2),
        ] {
            for empty_first in [false, true] {
                for (empty, empty_count, defined) in [
                    ("(())?", 2, &[][..]),
                    ("(?:(())?())", 3, &[2][..]),
                    ("(?:(())+?())", 3, &[0, 1, 2][..]),
                    ("(?:(()|())())", 4, &[0, 1, 3][..]),
                    ("(?:(()|())*?())", 4, &[3][..]),
                    ("(?:(()|()){2}())", 4, &[0, 1, 3][..]),
                ] {
                    for (count, minimum, optional, greedy) in [
                        ("*".to_owned(), Some(0), None, true),
                        ("*?".to_owned(), Some(0), None, false),
                        ("+".to_owned(), Some(1), None, true),
                        ("+?".to_owned(), Some(1), None, false),
                        ("?".to_owned(), Some(0), Some(1), true),
                        ("??".to_owned(), Some(0), Some(1), false),
                        ("{0}".to_owned(), Some(0), Some(0), true),
                        ("{0,3}?".to_owned(), Some(0), Some(3), false),
                        ("{01,001}?".to_owned(), Some(1), Some(0), false),
                        ("{1,}".to_owned(), Some(1), None, true),
                        ("{2,2}?".to_owned(), Some(2), Some(0), false),
                        ("{2,3}".to_owned(), Some(2), Some(1), true),
                        ("{2,3}?".to_owned(), Some(2), Some(1), false),
                        ("{3,}?".to_owned(), Some(3), None, false),
                        (format!("{{0,{huge}}}"), Some(0), None, true),
                        (format!("{{1,{huge}}}?"), Some(1), None, false),
                        (format!("{{{limit},{}}}", limit + 1), None, Some(1), true),
                        (format!("{{{limit},{}}}?", limit + 1), None, Some(1), false),
                        (format!("{{{huge}}}?"), None, Some(0), false),
                        (format!("{{00{huge},0{huge}}}"), None, Some(0), true),
                        (format!("{{{huge},}}?"), None, None, false),
                        (format!("{{{huge},{}}}", huge + 1), None, Some(1), true),
                        (format!("{{{huge},{}}}?", huge + 1), None, Some(1), false),
                    ] {
                        for (prefix, suffix, anchors, words) in [
                            ("", "", false, [None, None]),
                            (r"^\b", r"\b$", true, [Some(true), Some(true)]),
                            (r"\B", r"\b", false, [Some(false), Some(true)]),
                        ] {
                            for i in [false, true] {
                                for m in [false, true] {
                                    let branch = format!("{inner_open}{class}{inner_close}");
                                    let branch = if empty_first {
                                        format!("{empty}|{branch}")
                                    } else {
                                        format!("{branch}|{empty}")
                                    };
                                    let source =
                                        format!("{prefix}{open}{branch}{close}{count}{suffix}");
                                    let plan =
                                        Matcher::compile_with_empty_capture_effects_and_work(
                                            &JsString::from(source.as_str()),
                                            i,
                                            m,
                                            |_| Ok::<(), ()>(()),
                                        )
                                        .unwrap()
                                        .unwrap();
                                    assert_eq!(
                                        plan.capture_count(),
                                        roots + inner_captures + empty_count
                                    );
                                    let empty_start =
                                        roots + if empty_first { 0 } else { inner_captures };
                                    let empty_slots =
                                        defined.iter().map(|n| empty_start + n).collect::<Vec<_>>();
                                    let never_slots = (0..empty_count)
                                        .filter(|n| !defined.contains(n))
                                        .map(|n| empty_start + n)
                                        .collect::<Vec<_>>();
                                    for input in [
                                        JsString::from(""),
                                        JsString::from("aaab!aba"),
                                        JsString::from("A😀😀aa"),
                                        JsString::from("!\r\naaa\u{2028}!"),
                                        JsString::from("ſK"),
                                        JsString::from("𐐀𐐨!"),
                                        JsString::from("(a))!"),
                                        JsString::from_code_units(vec![0xd800, 97, 0xdc00]),
                                    ] {
                                        let minimum = minimum.unwrap_or(input.len() + 1);
                                        let oracle = Oracle {
                                            input: &input,
                                            contains,
                                            minimum,
                                            maximum: optional.map(|n| minimum + n),
                                            greedy,
                                            empty_first,
                                            roots,
                                            captures: roots + inner_captures + empty_count,
                                            empty_slots: &empty_slots,
                                            never_slots: &never_slots,
                                            i,
                                            m,
                                            anchors,
                                            words,
                                        };
                                        for start in 0..=input.len() + 1 {
                                            for sticky in [false, true] {
                                                let found = plan
                                                    .find_with_captures_and_work(
                                                        &input,
                                                        start,
                                                        sticky,
                                                        |_| Ok::<(), ()>(()),
                                                    )
                                                    .unwrap();
                                                let projected = found.map(|found| Expected {
                                                    captures: (0..plan.capture_count())
                                                        .map(|slot| {
                                                            plan.capture_range(
                                                                slot,
                                                                found.atom_capture.as_ref(),
                                                            )
                                                        })
                                                        .collect(),
                                                    range: found.range,
                                                });
                                                assert_eq!(
                                                    projected,
                                                    oracle.find(start, sticky),
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
}

#[test]
fn empty_capture_effects_cancel_each_callback_and_keep_large_masks_counts_and_clones_flat() {
    for pattern in [
        r"^\b(([\q{a|b}])|(?:(())?())){2,3}?\b$",
        r"(?:(?:(())*?)|(([\q{ab}&&a]))){0}",
        r"((?:(()|())+?())|([\q{😀|😀x}&&\p{Any}])){999999999999999999999999999999,1000000000000000000000000000000}?",
    ] {
        let source = JsString::from(pattern);
        let mut complete = Vec::new();
        Matcher::compile_with_empty_capture_effects_and_work(&source, true, true, |n| {
            complete.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(complete[0], source.len() * 64);
        for fail in 1..=complete.len() {
            let mut seen = Vec::new();
            let result =
                Matcher::compile_with_empty_capture_effects_and_work(&source, true, true, |n| {
                    seen.push(n);
                    if seen.len() == fail {
                        Err(fail)
                    } else {
                        Ok(())
                    }
                });
            assert!(
                matches!(result,Err(crate::RegExpUnicodeSetCompileError::Work(at)) if at==fail)
            );
            assert_eq!(seen, complete[..fail]);
        }
        let mut recovered = Vec::new();
        Matcher::compile_with_empty_capture_effects_and_work(&source, true, true, |n| {
            recovered.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(recovered, complete);
    }
    for empty_first in [false, true] {
        for mode in 0..3 {
            let empty = match mode {
                0 => format!("{}{}?", "(".repeat(20_000), ")".repeat(20_000)),
                1 => format!("(?:|{})", "()".repeat(20_000)),
                _ => format!("(?:{})+?", "()".repeat(20_000)),
            };
            let branch = if empty_first {
                format!(r"{empty}|([\q{{a|b}}])")
            } else {
                format!(r"([\q{{a|b}}])|{empty}")
            };
            let source = format!(
                "({branch}){{999999999999999999999999999999,1000000000000000000000000000000}}"
            );
            let source = JsString::from(source.as_str());
            let mut work = 0;
            let original =
                Matcher::compile_with_empty_capture_effects_and_work(&source, false, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap()
                .unwrap();
            assert!(work < source.len() * 256 + 1024, "{work}");
            let plan = original.clone();
            drop(original);
            assert_eq!(plan.capture_count(), 20_002);
            let found = plan
                .find_with_captures_and_work(&JsString::from("aaa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, 0..if empty_first { 1 } else { 3 });
            assert_eq!(
                plan.capture_range(0, found.atom_capture.as_ref()),
                Some(if empty_first { 0..1 } else { 3..3 })
            );
            let character = if empty_first { 20_001 } else { 1 };
            assert_eq!(
                plan.capture_range(character, found.atom_capture.as_ref()),
                if empty_first { Some(0..1) } else { None }
            );
            for slot in 1..20_002 {
                if slot != character {
                    assert_eq!(
                        plan.capture_range(slot, found.atom_capture.as_ref()),
                        if !empty_first && mode == 2 {
                            Some(3..3)
                        } else {
                            None
                        }
                    );
                }
            }
            assert_eq!(
                plan.capture_range(20_002, found.atom_capture.as_ref()),
                None
            );
        }
    }
    let empty = format!("(()){{{}}}", "9".repeat(20_000));
    let source = format!(r"(([\q{{a|b}}])|{empty}){{3}}");
    let source = JsString::from(source.as_str());
    let mut work = 0;
    let plan = Matcher::compile_with_empty_capture_effects_and_work(&source, false, false, |n| {
        work += n;
        Ok::<(), ()>(())
    })
    .unwrap()
    .unwrap();
    assert!(work < source.len() * 256 + 1024, "{work}");
    let found = plan
        .find_with_captures_and_work(&JsString::from("ab"), 0, true, |_| Ok::<(), ()>(()))
        .unwrap()
        .unwrap();
    assert_eq!(found.range, 0..2);
    assert_eq!(plan.capture_count(), 4);
    assert_eq!(
        plan.capture_range(0, found.atom_capture.as_ref()),
        Some(2..2)
    );
    assert_eq!(plan.capture_range(1, found.atom_capture.as_ref()), None);
    assert_eq!(
        plan.capture_range(2, found.atom_capture.as_ref()),
        Some(2..2)
    );
    assert_eq!(
        plan.capture_range(3, found.atom_capture.as_ref()),
        Some(2..2)
    );
}

#[test]
fn empty_capture_effects_keep_assertions_references_character_effects_and_leaves_closed() {
    for source in [
        r"(([\q{a|b}])|())+",
        r"(([\q{a|b}])|(?:)*?)+",
        r"(([\q{a|b}])|(?:|))+",
        r"(()([\q{a|b}])|()*)+",
        r"(([\q{a|b}])()|()*)+",
        r"(([\q{a|b}])|()*|())+",
        r"((\b[\q{a|b}])|()*)+",
        r"(([\q{a|b}])+|()*)+",
        r"(([\q{a|b}])|\b()*)+",
        r"(([\q{a|b}])|\1()*)+",
        r"(([\q{a|b}])|x()*)+",
        r"(([\q{a|b}])|(?=())*)+",
        r"(([\q{ab}&&\q{cd}])|()*){0}",
        r"(()*|([\q{ab}&&\p{Any}&&\p{RGI_Emoji}])){0}",
        r"(([\q{a|b}])|()*)+[b]",
        r"(([\q{a|b}])|()*)",
        r"(([\q{a|b}])|()*)+(?:)",
    ] {
        assert!(
            Matcher::compile_with_empty_capture_effects_and_work(
                &JsString::from(source),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none(),
            "{source}"
        );
    }
}

#[test]
fn nullable_capture_proofs_keep_observable_empty_captures_inside_the_alternative_owner() {
    macro_rules! rejects {
        ($source:expr; $($factory:ident),+ $(,)?) => {
            $(assert!(Matcher::$factory(&$source, false, false, |_| Ok::<(), ()>(())).unwrap().is_none(),
                "{} {:?}", stringify!($factory), $source);)+
        };
    }
    for source in [
        r"(?:(?:[\q{a}]|)())+",
        r"(?:()(?:|[\q{a}]))+",
        r"(?:(?:([\q{a}])|)(?:(())?))+",
        r"(?:(?:(())+?)(?:|([\q{a}]))){2,3}",
    ] {
        let source = JsString::from(source);
        rejects!(source;
            compile_with_work,
            compile_with_empty_branch_captures_and_work,
            compile_with_noncapturing_empty_branch_and_work,
            compile_with_quantified_noncapturing_empty_branch_and_work,
            compile_with_noncapturing_empty_choices_and_work,
            compile_with_empty_capture_effects_and_work,
        );
    }
    for (source, whole, character, empty) in [
        (r"(?:(?:([\q{a}])|())){3}", 0..2, None, Some(2..2)),
        (r"(?:(?:()|([\q{a}]))){2,3}", 0..1, Some(0..1), None),
    ] {
        let plan = Matcher::compile_with_empty_branch_captures_and_work(
            &JsString::from(source),
            false,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        let found = plan
            .find_with_captures_and_work(&JsString::from("aa"), 0, true, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, whole);
        let character_slot = usize::from(source.contains("()|"));
        assert_eq!(
            plan.capture_range(character_slot, found.atom_capture.as_ref()),
            character
        );
        assert_eq!(
            plan.capture_range(1 - character_slot, found.atom_capture.as_ref()),
            empty
        );
    }
    macro_rules! admits_transparent {
        ($factory:ident, $source:literal, $whole:expr, $character:expr) => {
            let plan =
                Matcher::$factory(&JsString::from($source), false, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
            let found = plan
                .find_with_captures_and_work(&JsString::from("aa"), 0, true, |_| Ok::<(), ()>(()))
                .unwrap()
                .unwrap();
            assert_eq!(found.range, $whole);
            assert_eq!(
                plan.capture_range(0, found.atom_capture.as_ref()),
                Some($character)
            );
        };
    }
    admits_transparent!(
        compile_with_noncapturing_empty_branch_and_work,
        r"(?:(?:([\q{a}])|)(?:))+",
        0..2,
        1..2
    );
    admits_transparent!(
        compile_with_quantified_noncapturing_empty_branch_and_work,
        r"(?:(?:([\q{a}])|)(?:)*?)+",
        0..2,
        1..2
    );
    admits_transparent!(
        compile_with_noncapturing_empty_choices_and_work,
        r"(?:(?:|)*?(?:|([\q{a}]))){2,3}",
        0..1,
        0..1
    );
}
