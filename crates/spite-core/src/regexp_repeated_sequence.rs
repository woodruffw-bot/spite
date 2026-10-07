//! Repetition of fixed consuming ordinary groups (22.2.2.3.1, 22.2.2.5–7).

use crate::{JsString, RegExpSequenceMatcher};
use std::{ops::Range, sync::Arc};

/// Immutable matcher for a fixed consuming group and one complete quantifier.
///
/// The Pattern must already be validated without `u` or `v`. Each iteration
/// consumes at least two UTF-16 units, with ordinary literals, character sets,
/// dots and nested capturing/noncapturing groups. Assertions, alternatives,
/// surrounding terms and further quantifiers remain unsupported. Neither
/// compilation nor matching expands repetition counts or uses native recursion.
#[derive(Clone, Debug)]
pub struct RegExpRepeatedSequenceMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    atom: RegExpSequenceMatcher,
    min: Option<usize>,
    max: Option<usize>,
    greedy: bool,
}

#[derive(Clone, Copy, Default)]
struct Run {
    end: usize,
    count: usize,
}

impl RegExpRepeatedSequenceMatcher {
    /// Compiles a complete quantified fixed group, or returns `None`.
    pub fn compile(source: &JsString, ignore_case: bool, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Accepts the complete subset before charging optional set construction.
    ///
    /// Exact decimal bounds preserve oversized minimum/maximum semantics. The
    /// fixed group compiler shares identical immutable character sets. Platform
    /// capacity checks bound the phase storage needed by a nonsticky search;
    /// resource quotas remain opt-in through the supplied callback.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let Some(end) = crate::regexp_repeated_literal::group_end(units) else {
            return Ok(None);
        };
        let Some((min, max, greedy)) = crate::regexp_quantified::complete_quantifier(&units[end..])
        else {
            return Ok(None);
        };
        let source = JsString::from_code_units(units[..end].to_vec());
        let Some(width) = RegExpSequenceMatcher::repeated_atom_width(&source, dot_all) else {
            return Ok(None);
        };
        if width > isize::MAX as usize / std::mem::size_of::<Run>() {
            return Ok(None);
        }
        charge(units.len())?;
        let Some(atom) =
            RegExpSequenceMatcher::compile_with_work(&source, ignore_case, dot_all, charge)?
        else {
            return Ok(None);
        };
        Ok(Some(Self(Arc::new(Program {
            atom,
            min,
            max,
            greedy,
        }))))
    }

    /// Finds the earliest start and its greedy or lazy complete iterations.
    ///
    /// Nonsticky minimum search visits each fixed-group candidate once, retaining
    /// one run counter for each offset modulo the group's width. This storage is
    /// proportional to the group, independent of input length and bounds. Search
    /// costs at most input length times group width. Sticky checks and greedy
    /// extension check consecutive groups directly and allocate no phase storage.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let program = &self.0;
        let units = input.code_units();
        let remaining = units.get(start..)?.len();
        let width = program.atom.atom_count();
        let min = program.min?;
        if min > remaining / width {
            return None;
        }
        let mut range = if min == 0 {
            start..start
        } else if sticky {
            let end = start + min * width;
            let mut cursor = start;
            while cursor < end {
                cursor = program.atom.find(input, cursor, true)?.end;
            }
            start..end
        } else if min == 1 {
            program.atom.find(input, start, false)?
        } else {
            program.find_minimum(units, start, min)?
        };
        let mut count = min;
        if !program.greedy || program.max == Some(count) {
            return Some(range);
        }
        while let Some(next) = program.atom.find(input, range.end, true) {
            range.end = next.end;
            count += 1;
            if program.max == Some(count) {
                break;
            }
        }
        Some(range)
    }

    /// Number of capturing groups inside the repeated atom, in source order.
    pub fn capture_count(&self) -> usize {
        self.0.atom.capture_ranges().len()
    }

    /// Last-iteration capture range, or `None` when no iteration participated.
    ///
    /// Every group in this fixed consuming subset participates in each iteration.
    /// Empty groups retain their final empty range (RepeatMatcher, 22.2.2.3.1).
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        let relative = self.0.atom.capture_ranges().get(index)?;
        let last = matched.end.checked_sub(self.0.atom.atom_count())?;
        if last < matched.start {
            return None;
        }
        Some(last.checked_add(relative.start)?..last.checked_add(relative.end)?)
    }

    /// Conservative optional work passes, including nonsticky phase setup.
    pub fn search_passes(&self, sticky: bool) -> usize {
        if sticky {
            2
        } else {
            self.0.atom.search_passes(false).saturating_add(1)
        }
    }
}

impl Program {
    fn find_minimum(&self, input: &[u16], start: usize, min: usize) -> Option<Range<usize>> {
        let width = self.atom.atom_count();
        let mut runs = vec![Run::default(); width];
        for matched in self.atom.matches_from(input, start)? {
            let run = &mut runs[matched.start % width];
            run.count = if run.end == matched.start {
                run.count + 1
            } else {
                1
            };
            run.end = matched.end;
            if run.count == min {
                // All minimum matches have the same consuming width, so the
                // first endpoint also identifies the earliest candidate start.
                return Some(matched.end - min * width..matched.end);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn repeated_fixed_groups_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a[b])*",
            "(a[b])*?",
            "(a[b])+",
            "(a[b])+?",
            "(a[b])?",
            "(a[b])??",
            "(a[b]){0}",
            "(a[b]){0,2}",
            "(a[b]){0,2}?",
            "(a[b]){2,3}",
            "(a[b]){2,3}?",
            "(a[b]){999999999999999999999999999999}",
            "(a[b]){1,999999999999999999999999999999}",
            "(?:a[b])+",
            "((a)([bc])())+",
            "((a)([bc])())*?",
            "((a)([bc])()){2}",
            "([ab]a)+",
            "(a[ab])+",
            "([ab][ab]){2,3}",
            "([ab][ab]){2,3}?",
            "([ab][ab]){3}",
            "(.a)+",
            "(a.)+",
            "(..){2,3}",
            "(a[^])+",
            "([]a)*",
            "([]a)+",
            "([µ]b)+",
            r"(\d\w)+",
            r"(\s\S)+",
            r"(\D\W)+",
            r"([\uD800]a)+",
            r"([\uD83D][\uDCA9])+",
            r"(\x61[b])+",
            "(ab)+",
            "([ab])+",
            "()*",
            "([ab]a$)+",
            r"(a\bb)+",
            "^(a[b])+$",
            "(a[b])+c",
            "c(a[b])+",
            "(a[b]|cd)+",
            "((a[b])+)",
            "(a[b])+(cd)+",
            "(?<n>a[b])+",
            r"(a[b])\1",
            "(a[b])+|x",
            "(a[b]+)+",
        ] {
            for (i, s) in [(false, false), (true, false), (false, true)] {
                write!(rows, "{source:?} i={i} s={s}").unwrap();
                if let Some(matcher) =
                    RegExpRepeatedSequenceMatcher::compile(&JsString::from(source), i, s)
                {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for text in [
                        "", "ab", "xabababz", "aababab", "abacacab", "ABab", "aaaaaaa", "abaaba",
                        "Μbµb", "1a2b", " a\nb", "a\na\n", "💩💩",
                    ] {
                        let input = JsString::from(text);
                        for (start, sticky) in [(0, false), (1, true)] {
                            let range = matcher.find(&input, start, sticky);
                            let captures = range.as_ref().map(|range| {
                                (0..matcher.capture_count())
                                    .map(|index| matcher.capture_range(index, range))
                                    .collect::<Vec<_>>()
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{range:?}:{captures:?}")
                                .unwrap();
                        }
                    }
                } else {
                    rows.push_str(" unsupported");
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn fixed_group_phase_runs_and_captures_agree_with_candidate_oracle() {
        for (first, second, kind) in [
            ("[ab]", "a", 0),
            ("a", "[ab]", 1),
            ("[ab]", "[ab]", 2),
            (".", "[ab]", 3),
            ("[]", "a", 4),
        ] {
            for (min, max) in [
                (0, None),
                (1, None),
                (2, Some(3)),
                (3, Some(4)),
                (0, Some(0)),
                (0, Some(2)),
            ] {
                for greedy in [false, true] {
                    let bounds =
                        max.map_or_else(|| format!("{{{min},}}"), |max| format!("{{{min},{max}}}"));
                    let source = format!(
                        "(({first})({second})()){bounds}{}",
                        if greedy { "" } else { "?" }
                    );
                    for (i, s) in [(false, false), (true, false), (false, true)] {
                        let matcher = RegExpRepeatedSequenceMatcher::compile(
                            &JsString::from(source.as_str()),
                            i,
                            s,
                        )
                        .unwrap();
                        for length in 0..=5 {
                            for mut number in 0..4usize.pow(length) {
                                let mut input = vec![0; length as usize];
                                for unit in &mut input {
                                    *unit = [u16::from(b'a'), u16::from(b'b'), u16::from(b'A'), 10]
                                        [number % 4];
                                    number /= 4;
                                }
                                let text = JsString::from_code_units(input.clone());
                                for start in 0..=input.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=input.len())
                                            .take(if sticky { 1 } else { usize::MAX })
                                            .find_map(|candidate| {
                                                let limit = max
                                                    .unwrap_or((input.len() - candidate) / 2)
                                                    .min((input.len() - candidate) / 2);
                                                let mut count = 0;
                                                while count < limit {
                                                    let a = input[candidate + 2 * count];
                                                    let b = input[candidate + 2 * count + 1];
                                                    let a = if i && a == 65 { 97 } else { a };
                                                    let b = if i && b == 65 { 97 } else { b };
                                                    let matches = match kind {
                                                        0 => matches!(a, 97 | 98) && b == 97,
                                                        1 => a == 97 && matches!(b, 97 | 98),
                                                        2 => {
                                                            matches!(a, 97 | 98)
                                                                && matches!(b, 97 | 98)
                                                        }
                                                        3 => (s || a != 10) && matches!(b, 97 | 98),
                                                        _ => false,
                                                    };
                                                    if !matches {
                                                        break;
                                                    }
                                                    count += 1;
                                                }
                                                (count >= min).then_some(
                                                    candidate
                                                        ..candidate
                                                            + if greedy { count } else { min } * 2,
                                                )
                                            });
                                        let actual = matcher.find(&text, start, sticky);
                                        assert_eq!(
                                            actual, expected,
                                            "{source} i={i} s={s} {input:?} @{start}/{sticky}"
                                        );
                                        if let Some(range) = actual {
                                            let last =
                                                (range.start < range.end).then(|| range.end - 2);
                                            let expected = [
                                                last.map(|last| last..range.end),
                                                last.map(|last| last..last + 1),
                                                last.map(|last| last + 1..range.end),
                                                last.map(|_| range.end..range.end),
                                            ];
                                            for (index, expected) in
                                                expected.into_iter().enumerate()
                                            {
                                                assert_eq!(
                                                    matcher.capture_range(index, &range),
                                                    expected
                                                );
                                            }
                                            assert_eq!(matcher.capture_range(4, &range), None);
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
    fn large_minimums_shared_sets_nested_captures_and_optional_work() {
        let source = JsString::from("((a)([bc])()){100000}");
        let matcher = RegExpRepeatedSequenceMatcher::compile(&source, false, false).unwrap();
        let copy = matcher.clone();
        drop(matcher);
        let input =
            JsString::from(format!("{}x{}", "ab".repeat(99_999), "ac".repeat(100_000)).as_str());
        let range = copy.find(&input, 0, false).unwrap();
        assert_eq!(range, 199_999..399_999);
        assert_eq!(copy.capture_range(0, &range), Some(399_997..399_999));
        assert_eq!(copy.capture_range(2, &range), Some(399_998..399_999));
        assert_eq!(copy.capture_range(3, &range), Some(399_999..399_999));
        assert_eq!(copy.find(&input, 0, true), None);
        assert_eq!(copy.find(&input, usize::MAX, false), None);
        assert_eq!(copy.search_passes(false), 3);
        assert_eq!(copy.search_passes(true), 2);
        let deep = format!("{}a[b]{}+", "(".repeat(100_000), ")".repeat(100_000));
        let deep =
            RegExpRepeatedSequenceMatcher::compile(&JsString::from(deep.as_str()), false, false)
                .unwrap();
        assert_eq!(deep.capture_count(), 100_000);
        let range = deep.find(&JsString::from("abab"), 0, true).unwrap();
        assert_eq!(range, 0..4);
        assert_eq!(deep.capture_range(99_999, &range), Some(2..4));
        let wide = format!("([ab]{}){{2}}", "[ab]".repeat(99_999));
        let wide =
            RegExpRepeatedSequenceMatcher::compile(&JsString::from(wide.as_str()), false, false)
                .unwrap();
        assert_eq!(wide.search_passes(false), 100_001);
        let input = JsString::from("a".repeat(200_000).as_str());
        assert_eq!(wide.find(&input, 0, true), Some(0..200_000));
        let mut charges = Vec::new();
        let aborted =
            RegExpRepeatedSequenceMatcher::compile_with_work(&source, false, false, |work| {
                charges.push(work);
                if charges.len() == 3 {
                    Err("host")
                } else {
                    Ok(())
                }
            });
        assert!(matches!(aborted, Err("host")));
        assert_eq!(charges.first(), Some(&source.len()));
        assert!(
            RegExpRepeatedSequenceMatcher::compile_with_work(
                &JsString::from("(a[bc]$)+"),
                false,
                false,
                |_| Err("unexpected")
            )
            .unwrap()
            .is_none()
        );
    }
}
