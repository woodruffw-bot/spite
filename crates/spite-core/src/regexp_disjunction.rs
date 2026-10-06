//! Top-level literal Disjunction compilation and matching (22.2.2.3).

use crate::{JsString, RegExpLiteralMatcher};
use std::{ops::Range, sync::Arc};

/// Immutable top-level alternatives of ordinary literal sequences.
///
/// The Pattern must already be validated without `u` or `v`. Every alternative
/// must compile as a literal sequence. Unsupported alternatives reject the
/// entire plan. Nested alternatives and quantified groups remain
/// unsupported; compilation never expands combinations or uses native recursion.
#[derive(Clone, Debug)]
pub struct RegExpDisjunctionMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    alternatives: Vec<RegExpLiteralMatcher>,
    capture_offsets: Vec<usize>,
    capture_count: usize,
}

impl RegExpDisjunctionMatcher {
    /// Compiles two or more top-level alternatives, including empty alternatives.
    pub fn compile(source: &JsString, ignore_case: bool) -> Option<Self> {
        let units = source.code_units();
        let mut ranges = Vec::new();
        let mut start = 0;
        let mut index = 0;
        let mut depth = 0usize;
        while let Some(&unit) = units.get(index) {
            index += 1;
            match unit {
                0x5c => {
                    units.get(index)?;
                    index += 1;
                }
                0x28 => {
                    if units.get(index) == Some(&u16::from(b'?')) {
                        if units.get(index..index + 2)? != [u16::from(b'?'), u16::from(b':')] {
                            return None;
                        }
                        index += 2;
                    }
                    depth = depth.checked_add(1)?;
                }
                0x29 => depth = depth.checked_sub(1)?,
                0x5b => return None,
                0x7c => {
                    if depth != 0 {
                        return None;
                    }
                    ranges.push(start..index - 1);
                    start = index;
                }
                _ => {}
            }
        }
        if depth != 0 || ranges.is_empty() {
            return None;
        }
        ranges.push(start..units.len());
        let mut alternatives = Vec::with_capacity(ranges.len());
        let mut capture_offsets = Vec::with_capacity(ranges.len());
        let mut capture_count = 0usize;
        for range in ranges {
            let source = JsString::from_code_units(units[range].to_vec());
            let matcher = RegExpLiteralMatcher::compile(&source, ignore_case)?;
            capture_offsets.push(capture_count);
            capture_count = capture_count.checked_add(matcher.capture_ranges().len())?;
            alternatives.push(matcher);
        }
        Some(Self(Arc::new(Program {
            alternatives,
            capture_offsets,
            capture_count,
        })))
    }

    /// Number of alternatives, for optional worst-case work accounting.
    pub fn alternative_count(&self) -> usize {
        self.0.alternatives.len()
    }

    /// Total capturing groups in source order, including unselected branches.
    pub fn capture_count(&self) -> usize {
        self.0.capture_count
    }

    /// First capture slot and relative ranges for a selected branch.
    ///
    /// Returns `None` for an invalid branch index. Groups in other branches do
    /// not participate; consumers must represent those slots as undefined.
    pub fn branch_captures(&self, branch: usize) -> Option<(usize, &[Range<usize>])> {
        Some((
            *self.0.capture_offsets.get(branch)?,
            self.0.alternatives.get(branch)?.capture_ranges(),
        ))
    }

    /// Finds the earliest match, choosing source order for equal start offsets.
    ///
    /// Sticky matching considers only `start`. Each branch uses linear literal
    /// search; worst-case work is proportional to the number of alternatives
    /// times the input suffix length. Search allocates nothing.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.find_branch(input, start, sticky)
            .map(|(_, range)| range)
    }

    /// Finds a match and its source-order branch index without allocating.
    pub fn find_branch(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
    ) -> Option<(usize, Range<usize>)> {
        let mut best: Option<(usize, Range<usize>)> = None;
        for (branch, alternative) in self.0.alternatives.iter().enumerate() {
            if let Some(found) = alternative.find(input, start, sticky) {
                if found.start == start {
                    return Some((branch, found));
                }
                if best
                    .as_ref()
                    .is_none_or(|(_, best)| found.start < best.start)
                {
                    best = Some((branch, found));
                }
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn top_level_literal_disjunction_snapshot() {
        let mut rows = String::new();
        for source in [
            "a|ab",
            "ab|a",
            "2|12",
            "a|b",
            "|a",
            "a|",
            "||",
            "a||b",
            "aba|ba",
            "(?:ab)|(?:c)",
            r"a\|b|c",
            r"\uD800|\uDC00",
            "µ|s",
            "σ|k",
            "(a)|b",
            "(?:a|b)|c",
            "a|[b]",
            "a|b*",
            "a|.",
            "a|(?i:b)",
            "(?:a|b",
        ] {
            for ignore_case in [false, true] {
                let source = JsString::from(source);
                write!(rows, "{source:?} i={ignore_case}").unwrap();
                if let Some(matcher) = RegExpDisjunctionMatcher::compile(&source, ignore_case) {
                    write!(rows, " alternatives={}", matcher.alternative_count()).unwrap();
                    for input in ["", "ab", "xaba", "1.012", "a|b", "ς", "ſ", "x𐀀y"] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            matcher.find(&input, 0, false),
                            matcher.find(&input, 0, true)
                        )
                        .unwrap();
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
    fn branch_choices_agree_with_independent_position_then_source_order_oracle() {
        let alphabet = [u16::from(b'a'), u16::from(b'A'), u16::from(b'b')];
        let mut inputs = vec![Vec::new()];
        let mut previous = vec![Vec::new()];
        for _ in 0..5 {
            let mut next = Vec::new();
            for word in previous {
                for unit in alphabet {
                    let mut word = word.clone();
                    word.push(unit);
                    inputs.push(word.clone());
                    next.push(word);
                }
            }
            previous = next;
        }
        let fold = |unit: u16, ignore_case: bool| {
            if ignore_case && (0x61..=0x7a).contains(&unit) {
                unit - 32
            } else {
                unit
            }
        };
        for left in ["", "a", "A", "ab", "ba", "aa", "aba"] {
            for right in ["", "a", "A", "ab", "ba", "aa", "aba"] {
                let source = JsString::from(format!("({left})|({right})").as_str());
                for ignore_case in [false, true] {
                    let matcher = RegExpDisjunctionMatcher::compile(&source, ignore_case).unwrap();
                    for units in &inputs {
                        let input = JsString::from_code_units(units.clone());
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                let expected = (start..=input.len()).find_map(|offset| {
                                    if sticky && offset != start {
                                        return None;
                                    }
                                    [left, right].into_iter().enumerate().find_map(
                                        |(branch, word)| {
                                            let end = offset.checked_add(word.len())?;
                                            units
                                                .get(offset..end)?
                                                .iter()
                                                .copied()
                                                .zip(word.bytes().map(u16::from))
                                                .all(|(a, b)| {
                                                    fold(a, ignore_case) == fold(b, ignore_case)
                                                })
                                                .then_some((branch, offset..end))
                                        },
                                    )
                                });
                                assert_eq!(
                                    matcher.find_branch(&input, start, sticky),
                                    expected,
                                    "{source:?} {input:?} start={start} i={ignore_case} y={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn large_prefixes_many_branches_and_cloned_plans_have_no_default_cap() {
        let source = format!("{}b|{}c", "a".repeat(60_000), "a".repeat(60_000));
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        let input = JsString::from(format!("{}c", "a".repeat(120_000)).as_str());
        assert_eq!(matcher.find(&input, 0, false), Some(60_000..120_001));
        assert_eq!(matcher.find(&input, 0, true), None);
        let source = format!("{}b", "a|".repeat(100_000));
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        assert_eq!(matcher.alternative_count(), 100_001);
        let clone = matcher.clone();
        assert!(Arc::ptr_eq(&matcher.0, &clone.0));
        drop(matcher);
        assert_eq!(clone.find(&JsString::from("ab"), 0, false), Some(0..1));
        drop(clone);
    }

    #[test]
    fn alternative_capture_participation_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a)|b",
            "a|(b)",
            "(a)|(ab)",
            "(ab)|(a)",
            "()|()",
            "((a)())|c(d)",
            r"(\uD800)|c(\uDC00)",
            "µ|(σ)",
        ] {
            for ignore_case in [false, true] {
                let matcher =
                    RegExpDisjunctionMatcher::compile(&JsString::from(source), ignore_case)
                        .unwrap();
                for input in ["", "a", "ab", "xcd", "xς", "x𐀀"] {
                    let input = JsString::from(input);
                    write!(
                        rows,
                        "{source:?} i={ignore_case} input={input:?} count={}",
                        matcher.capture_count()
                    )
                    .unwrap();
                    if let Some((branch, range)) = matcher.find_branch(&input, 0, false) {
                        let (offset, captures) = matcher.branch_captures(branch).unwrap();
                        let slots: Vec<_> = (0..matcher.capture_count())
                            .map(|index| {
                                let capture = captures.get(index.checked_sub(offset)?)?;
                                Some(range.start + capture.start..range.start + capture.end)
                            })
                            .collect();
                        write!(rows, " branch={branch} match={range:?} captures={slots:?}")
                            .unwrap();
                    } else {
                        rows.push_str(" no-match");
                    }
                    rows.push('\n');
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn selected_branch_capture_offsets_are_linear_storage_and_iterative() {
        let source = format!("{}(b)", "(a)|".repeat(100_000));
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        assert_eq!(matcher.capture_count(), 100_001);
        assert_eq!(
            matcher.find_branch(&JsString::from("b"), 0, false),
            Some((100_000, 0..1))
        );
        assert_eq!(
            matcher.branch_captures(100_000),
            Some((100_000, std::slice::from_ref(&(0..1))))
        );
        assert_eq!(matcher.branch_captures(100_001), None);
        let clone = matcher.clone();
        assert!(Arc::ptr_eq(&matcher.0, &clone.0));
        drop(matcher);
        drop(clone);
        let source = format!("{}a{}|(b)", "(".repeat(100_000), ")".repeat(100_000));
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        assert_eq!(matcher.capture_count(), 100_001);
        assert_eq!(
            matcher.branch_captures(1),
            Some((100_000, std::slice::from_ref(&(0..1))))
        );
        assert_eq!(
            matcher.find_branch(&JsString::from("b"), 0, false),
            Some((1, 0..1))
        );
    }
}
