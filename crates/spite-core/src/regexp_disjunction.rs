//! Top-level fixed Disjunction compilation and matching (22.2.2.3).

use crate::{JsString, RegExpAnchoredMatcher, RegExpLiteralMatcher, RegExpSequenceMatcher};
use std::{ops::Range, sync::Arc};

/// Immutable top-level alternatives of fixed ordinary sequences and outer anchors.
///
/// The Pattern must already be validated without `u` or `v`. Every alternative
/// must compile as a literal, fixed class sequence or outer-anchored sequence.
/// Unsupported alternatives reject the entire plan. Nested alternatives and
/// quantified groups remain unsupported; compilation never expands combinations
/// or uses native recursion.
#[derive(Clone, Debug)]
pub struct RegExpDisjunctionMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    alternatives: Vec<Alternative>,
    capture_offsets: Vec<usize>,
    capture_count: usize,
}

#[derive(Debug)]
enum Alternative {
    Literal(RegExpLiteralMatcher),
    Sequence(RegExpSequenceMatcher),
    Anchored(RegExpAnchoredMatcher),
}

impl Alternative {
    fn capture_ranges(&self) -> &[Range<usize>] {
        match self {
            Self::Literal(m) => m.capture_ranges(),
            Self::Sequence(m) => m.capture_ranges(),
            Self::Anchored(m) => m.capture_ranges(),
        }
    }
    fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        match self {
            Self::Literal(m) => m.find(input, start, sticky),
            Self::Sequence(m) => m.find(input, start, sticky),
            Self::Anchored(m) => m.find(input, start, sticky),
        }
    }
    fn search_passes(&self, sticky: bool) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::Sequence(m) => {
                if sticky {
                    1
                } else {
                    m.atom_count().max(1)
                }
            }
            Self::Anchored(m) => m.search_passes(sticky),
        }
    }
}

impl RegExpDisjunctionMatcher {
    /// Compiles two or more top-level alternatives, including empty alternatives.
    pub fn compile(source: &JsString, ignore_case: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, false, false, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles with explicit multiline/DotAll and optional set preparation work.
    ///
    /// Top-level separators exclude escapes, classes and groups. Branch matching
    /// retains source-order capture slots and the complete input's boundaries.
    /// Rejected subsets and independent host charge failures remain distinct.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(ranges) = alternative_ranges(source.code_units()) else {
            return Ok(None);
        };
        let units = source.code_units();
        let mut alternatives = Vec::with_capacity(ranges.len());
        let mut capture_offsets = Vec::with_capacity(ranges.len());
        let mut capture_count = 0usize;
        for range in ranges {
            let source = JsString::from_code_units(units[range].to_vec());
            let matcher = if let Some(m) = RegExpLiteralMatcher::compile(&source, ignore_case) {
                Alternative::Literal(m)
            } else if let Some(m) = RegExpSequenceMatcher::compile_with_work(
                &source,
                ignore_case,
                dot_all,
                &mut charge,
            )? {
                Alternative::Sequence(m)
            } else if let Some(m) = RegExpAnchoredMatcher::compile_with_work(
                &source,
                ignore_case,
                multiline,
                dot_all,
                &mut charge,
            )? {
                Alternative::Anchored(m)
            } else {
                return Ok(None);
            };
            capture_offsets.push(capture_count);
            let Some(count) = capture_count.checked_add(matcher.capture_ranges().len()) else {
                return Ok(None);
            };
            capture_count = count;
            alternatives.push(matcher);
        }
        Ok(Some(Self(Arc::new(Program {
            alternatives,
            capture_offsets,
            capture_count,
        }))))
    }

    /// Conservative optional search passes across every branch's implementation.
    pub fn search_passes(&self, sticky: bool) -> usize {
        self.0.alternatives.iter().fold(0usize, |total, branch| {
            total.saturating_add(branch.search_passes(sticky))
        })
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
    /// Sticky matching considers only `start`. Literal branches retain linear
    /// search; fixed class branches retain their input/atom candidate bound.
    /// Search allocates nothing.
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
                if best
                    .as_ref()
                    .is_none_or(|(_, current)| found.start < current.start)
                {
                    best = Some((branch, found));
                }
            }
        }
        best
    }
}

fn alternative_ranges(units: &[u16]) -> Option<Vec<Range<usize>>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    let mut index = 0;
    let mut depth = 0usize;
    let mut in_class = false;
    while let Some(&unit) = units.get(index) {
        index += 1;
        if in_class {
            if unit == 0x5c {
                units.get(index)?;
                index += 1;
            } else if unit == 0x5d {
                in_class = false;
            }
            continue;
        }
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
            0x5b => in_class = true,
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
    if depth != 0 || in_class || ranges.is_empty() {
        return None;
    }
    ranges.push(start..units.len());
    Some(ranges)
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn fixed_class_and_anchored_alternative_snapshot() {
        let mut rows = String::new();
        for source in [
            "([ab])([a])|([a])",
            "([a])|([ab])([a])",
            "[a|b]|c",
            "[(|)]|a",
            r"[\]]|a",
            "[[a]|b",
            "[]|a",
            "[^]|a",
            ".|[a]",
            r"\d|[a]",
            "^([a])$|([b])$",
            "^a|b$",
            "^([a])|([b])$",
            "()|([a])",
            "([a])|()",
            "[a]+|b",
            "([a]|b)",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
            ] {
                let matcher = RegExpDisjunctionMatcher::compile_with_work(
                    &JsString::from(source),
                    i,
                    m,
                    s,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap();
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(matcher) = matcher {
                    write!(
                        rows,
                        " captures={} branches={}",
                        matcher.capture_count(),
                        matcher.alternative_count()
                    )
                    .unwrap();
                    for input in [
                        "", "a", "ba", "aba", "A", "b\n", "x\na\nyb", "|", "(", "]", "[", "1", "\n",
                    ] {
                        let input = JsString::from(input);
                        let found = matcher.find_branch(&input, 0, false);
                        write!(
                            rows,
                            " {input:?}:{found:?}/{:?}",
                            matcher.find_branch(&input, 1, true)
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
    fn class_branches_preserve_position_first_source_order_and_global_capture_offsets() {
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from("([ab])([a])|([a])"), false).unwrap();
        assert_eq!(matcher.capture_count(), 3);
        assert_eq!(matcher.branch_captures(0), Some((0, &[0..1, 1..2][..])));
        let (offset, ranges) = matcher.branch_captures(1).unwrap();
        assert_eq!(offset, 2);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], 0..1);
        let alphabet = [97, 98, 65, 10, 0xd800];
        for length in 0..=5u32 {
            for mut encoded in 0..alphabet.len().pow(length) {
                let mut units = Vec::new();
                for _ in 0..length {
                    units.push(alphabet[encoded % alphabet.len()]);
                    encoded /= alphabet.len();
                }
                let input = JsString::from_code_units(units.clone());
                for start in 0..=units.len() + 1 {
                    for sticky in [false, true] {
                        let expected = (start..=units.len()).find_map(|offset| {
                            if sticky && offset != start {
                                return None;
                            }
                            if let Some(pair) = units.get(offset..offset + 2) {
                                if matches!(pair[0], 97 | 98) && pair[1] == 97 {
                                    return Some((0, offset..offset + 2));
                                }
                            }
                            (units.get(offset) == Some(&97)).then_some((1, offset..offset + 1))
                        });
                        assert_eq!(
                            matcher.find_branch(&input, start, sticky),
                            expected,
                            "{units:?} start={start} sticky={sticky}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn class_alternative_construction_work_remains_distinct_from_matching_failure() {
        let result = RegExpDisjunctionMatcher::compile_with_work(
            &JsString::from(r"\d|[a]"),
            false,
            false,
            false,
            |work| {
                if work == 65_536 {
                    Err("host abort")
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(result, Err("host abort")));
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from("[a]a|^b$|c"), false).unwrap();
        assert_eq!(matcher.search_passes(false), 5);
        assert_eq!(matcher.search_passes(true), 4);
        assert_eq!(matcher.find(&JsString::from("xx"), 0, false), None);
    }

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
