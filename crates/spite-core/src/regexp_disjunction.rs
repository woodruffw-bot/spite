//! Top-level ordinary Disjunction compilation and matching (22.2.2.3).

use crate::{
    JsString, RegExpAnchoredMatcher, RegExpLiteralMatcher, RegExpPrefixedMatcher,
    RegExpQuantifiedContinuationMatcher, RegExpQuantifiedMatcher, RegExpRepeatedLiteralMatcher,
    RegExpRepeatedSequenceMatcher, RegExpSequenceMatcher, regexp_outer_group_body,
};
use std::{ops::Range, sync::Arc};

/// Immutable top-level alternatives of supported ordinary consuming sequences.
///
/// The Pattern must already be validated without `u` or `v`. Every alternative
/// must compile as a literal, fixed class sequence, outer-anchored sequence,
/// quantified atom, quantified prefix with a fixed continuation, or repeated
/// literal group.
/// Complete ordinary capturing/noncapturing wrappers can enclose each body;
/// captured wrappers precede that branch's retained capture slots.
/// Unsupported alternatives reject the entire plan. Nested alternatives and
/// multiple quantifiers remain unsupported; compilation never expands
/// combinations or uses native recursion.
#[derive(Clone, Debug)]
pub struct RegExpDisjunctionMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    alternatives: Vec<Branch>,
    capture_offsets: Vec<usize>,
    capture_count: usize,
    full_suffix: bool,
}

#[derive(Debug)]
enum Alternative {
    Literal(RegExpLiteralMatcher),
    Sequence(RegExpSequenceMatcher),
    Anchored(RegExpAnchoredMatcher),
    Quantified(RegExpQuantifiedMatcher),
    QuantifiedContinuation(RegExpQuantifiedContinuationMatcher),
    Prefixed(RegExpPrefixedMatcher),
    RepeatedLiteral(RegExpRepeatedLiteralMatcher),
    RepeatedSequence(RegExpRepeatedSequenceMatcher),
}

impl Alternative {
    fn capture_count(&self) -> usize {
        match self {
            Self::Prefixed(matcher) => matcher.capture_count(),
            Self::RepeatedLiteral(matcher) => matcher.capture_count(),
            Self::RepeatedSequence(matcher) => matcher.capture_count(),
            Self::Quantified(matcher) => matcher.capture_count(),
            Self::QuantifiedContinuation(matcher) => matcher.capture_count(),
            Self::Anchored(matcher) => matcher.capture_count(),
            _ => self.capture_ranges().len(),
        }
    }

    fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        match self {
            Self::Prefixed(matcher) => matcher.capture_range(index, matched),
            Self::RepeatedLiteral(matcher) => matcher.capture_range(index, matched),
            Self::RepeatedSequence(matcher) => matcher.capture_range(index, matched),
            Self::Quantified(matcher) => matcher.capture_range(index, matched),
            Self::QuantifiedContinuation(matcher) => matcher.capture_range(index, matched),
            Self::Anchored(matcher) => matcher.capture_range(index, matched),
            _ => {
                let relative = self.capture_ranges().get(index)?;
                Some(
                    matched.start.checked_add(relative.start)?
                        ..matched.start.checked_add(relative.end)?,
                )
            }
        }
    }

    fn capture_ranges(&self) -> &[Range<usize>] {
        match self {
            Self::Literal(m) => m.capture_ranges(),
            Self::Sequence(m) => m.capture_ranges(),
            Self::Anchored(m) => m.capture_ranges(),
            Self::Quantified(_)
            | Self::QuantifiedContinuation(_)
            | Self::Prefixed(_)
            | Self::RepeatedLiteral(_)
            | Self::RepeatedSequence(_) => &[],
        }
    }
    fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        match self {
            Self::Literal(m) => m.find(input, start, sticky),
            Self::Sequence(m) => m.find(input, start, sticky),
            Self::Anchored(m) => m.find(input, start, sticky),
            Self::Prefixed(m) => m.find(input, start, sticky),
            Self::RepeatedLiteral(m) => m.find(input, start, sticky),
            Self::RepeatedSequence(m) => m.find(input, start, sticky),
            Self::Quantified(m) => m.find(input, start, sticky),
            Self::QuantifiedContinuation(m) => m.find(input, start, sticky),
        }
    }
    fn search_passes(&self, sticky: bool) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::Sequence(m) => m.search_passes(sticky),
            Self::Anchored(m) => m.search_passes(sticky),
            Self::Prefixed(m) => m.search_passes(sticky),
            Self::RepeatedLiteral(m) => m.search_passes(),
            Self::RepeatedSequence(m) => m.search_passes(sticky),
            Self::Quantified(_) => 1,
            Self::QuantifiedContinuation(m) => m.search_passes(),
        }
    }
}

/// An existing body plan with a scalar prefix of whole-branch captures.
#[derive(Debug)]
struct Branch {
    matcher: Alternative,
    enclosing_captures: usize,
    capture_count: usize,
}

impl Branch {
    fn new(matcher: Alternative, enclosing_captures: usize) -> Option<Self> {
        let capture_count = enclosing_captures.checked_add(matcher.capture_count())?;
        Some(Self {
            matcher,
            enclosing_captures,
            capture_count,
        })
    }
    fn capture_count(&self) -> usize {
        self.capture_count
    }
    fn capture_ranges(&self) -> &[Range<usize>] {
        if self.enclosing_captures == 0 {
            self.matcher.capture_ranges()
        } else {
            &[]
        }
    }
    fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        if index >= self.capture_count || matched.start > matched.end {
            return None;
        }
        if index < self.enclosing_captures {
            return Some(matched.clone());
        }
        self.matcher
            .capture_range(index - self.enclosing_captures, matched)
    }
    fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.matcher.find(input, start, sticky)
    }
    fn search_passes(&self, sticky: bool) -> usize {
        self.matcher.search_passes(sticky)
    }
}

fn compile_alternative<E>(
    source: &JsString,
    ignore_case: bool,
    multiline: bool,
    dot_all: bool,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<Alternative>, E> {
    let matcher = if let Some(m) = RegExpLiteralMatcher::compile(source, ignore_case) {
        Alternative::Literal(m)
    } else if let Some(m) =
        RegExpSequenceMatcher::compile_with_work(source, ignore_case, dot_all, &mut *charge)?
    {
        Alternative::Sequence(m)
    } else if let Some(m) = RegExpAnchoredMatcher::compile_with_work(
        source,
        ignore_case,
        multiline,
        dot_all,
        &mut *charge,
    )? {
        Alternative::Anchored(m)
    } else if let Some(m) =
        RegExpQuantifiedMatcher::compile_with_work(source, ignore_case, dot_all, &mut *charge)?
    {
        Alternative::Quantified(m)
    } else if let Some(m) = RegExpQuantifiedContinuationMatcher::compile_with_work(
        source,
        ignore_case,
        dot_all,
        &mut *charge,
    )? {
        Alternative::QuantifiedContinuation(m)
    } else if let Some(m) =
        RegExpPrefixedMatcher::compile_with_work(source, ignore_case, dot_all, &mut *charge)?
    {
        Alternative::Prefixed(m)
    } else if let Some(m) = RegExpSequenceMatcher::compile_with_assertions_and_work(
        source,
        ignore_case,
        multiline,
        dot_all,
        &mut *charge,
    )? {
        Alternative::Sequence(m)
    } else if let Some(m) = RegExpQuantifiedContinuationMatcher::compile_with_assertions_and_work(
        source,
        ignore_case,
        multiline,
        dot_all,
        &mut *charge,
    )? {
        Alternative::QuantifiedContinuation(m)
    } else if let Some(m) = RegExpPrefixedMatcher::compile_with_assertions_and_work(
        source,
        ignore_case,
        multiline,
        dot_all,
        &mut *charge,
    )? {
        Alternative::Prefixed(m)
    } else if let Some(m) =
        RegExpRepeatedLiteralMatcher::compile_with_work(source, ignore_case, &mut *charge)?
    {
        Alternative::RepeatedLiteral(m)
    } else if let Some(m) = RegExpRepeatedSequenceMatcher::compile_with_work(
        source,
        ignore_case,
        dot_all,
        &mut *charge,
    )? {
        Alternative::RepeatedSequence(m)
    } else {
        return Ok(None);
    };
    Ok(Some(matcher))
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
            // Keep existing fixed layouts before trying complete group wrappers.
            let matcher = if let Some(matcher) =
                compile_alternative(&source, ignore_case, multiline, dot_all, &mut charge)?
            {
                Branch::new(matcher, 0)
            } else if source.code_units().first() == Some(&40) {
                charge(source.len())?;
                charge(source.len())?;
                charge(source.len())?;
                let Some(group) = regexp_outer_group_body(&source) else {
                    return Ok(None);
                };
                charge(group.body.len())?;
                let body = JsString::from_code_units(source.code_units()[group.body].to_vec());
                charge(body.len())?;
                charge(body.len())?;
                let Some(matcher) =
                    compile_alternative(&body, ignore_case, multiline, dot_all, &mut charge)?
                else {
                    return Ok(None);
                };
                Branch::new(matcher, group.captures)
            } else {
                return Ok(None);
            };
            let Some(matcher) = matcher else {
                return Ok(None);
            };
            capture_offsets.push(capture_count);
            let Some(count) = capture_count.checked_add(matcher.capture_count()) else {
                return Ok(None);
            };
            capture_count = count;
            alternatives.push(matcher);
        }
        charge(alternatives.len())?;
        let full_suffix = alternatives.iter().any(|branch| match &branch.matcher {
            Alternative::Quantified(_)
            | Alternative::QuantifiedContinuation(_)
            | Alternative::Prefixed(_)
            | Alternative::RepeatedLiteral(_)
            | Alternative::RepeatedSequence(_) => true,
            Alternative::Anchored(matcher) => matcher.requires_full_suffix(),
            _ => false,
        });
        Ok(Some(Self(Arc::new(Program {
            alternatives,
            capture_offsets,
            capture_count,
            full_suffix,
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

    /// Quantified branches may inspect the entire suffix even at one sticky start.
    pub fn requires_full_suffix(&self) -> bool {
        self.0.full_suffix
    }

    /// Total capturing groups in source order, including unselected branches.
    pub fn capture_count(&self) -> usize {
        self.0.capture_count
    }

    /// First capture slot and relative ranges for a selected branch.
    ///
    /// Returns `None` for an invalid index or a branch with dynamic captures.
    /// Use `capture_range` to resolve all supported captures. Groups in other
    /// branches do not participate; consumers represent those slots as undefined.
    pub fn branch_captures(&self, branch: usize) -> Option<(usize, &[Range<usize>])> {
        let alternative = self.0.alternatives.get(branch)?;
        if alternative.capture_count() != alternative.capture_ranges().len() {
            return None;
        }
        Some((
            *self.0.capture_offsets.get(branch)?,
            self.0.alternatives.get(branch)?.capture_ranges(),
        ))
    }

    /// Absolute capture range for a source-order slot in a successful branch match.
    /// Unselected groups and zero-iteration inner groups return `None`.
    pub fn capture_range(
        &self,
        branch: usize,
        index: usize,
        matched: &Range<usize>,
    ) -> Option<Range<usize>> {
        let local = index.checked_sub(*self.0.capture_offsets.get(branch)?)?;
        self.0
            .alternatives
            .get(branch)?
            .capture_range(local, matched)
    }

    /// Finds the earliest match, choosing source order for equal start offsets.
    ///
    /// Sticky matching considers only `start`. Literal/quantified branches retain
    /// linear search; fixed class branches retain their input/atom candidate bound.
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
    fn quantified_alternative_snapshot() {
        let mut rows = String::new();
        for source in [
            "a+|a",
            "a|a+",
            "a+?|a+",
            "a+|a+?",
            "[ab]*ab|[ab]*?ab",
            "[ab]*?ab|[ab]*ab",
            "([x])|[ab]+",
            "([x])()|[ab]+|([y])",
            "[ab]+|([x])",
            "a*|b",
            "a|b*",
            "[]+|a",
            "[^]*x|([a])",
            "^([a])$|b+",
            "(?:a)+|([b])",
            r"\d+x|([a])",
            ".+|([a])",
            "a+[b]|a",
            "(a)+|b",
            "(a+|b)",
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
                        " captures={} branches={} full_suffix={}",
                        matcher.capture_count(),
                        matcher.alternative_count(),
                        matcher.requires_full_suffix()
                    )
                    .unwrap();
                    for input in [
                        "", "a", "aaaa", "baaa", "abab", "x", "y", "ABab", "12x", "a\nb", "\n",
                        "x\na\n",
                    ] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            matcher.find_branch(&input, 0, false),
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
    fn quantified_branches_keep_position_first_source_order_and_capture_offsets() {
        let m = RegExpDisjunctionMatcher::compile(&JsString::from("[ab]*?ab|([a])|[ab]+"), false)
            .unwrap();
        assert_eq!(m.capture_count(), 1);
        assert_eq!(m.branch_captures(0), Some((0, &[][..])));
        assert_eq!(m.branch_captures(2), Some((1, &[][..])));
        for len in 0..=6u32 {
            for mut encoded in 0..3usize.pow(len) {
                let mut units = Vec::new();
                for _ in 0..len {
                    units.push([97, 98, 120][encoded % 3]);
                    encoded /= 3;
                }
                let input = JsString::from_code_units(units.clone());
                for start in 0..=units.len() + 1 {
                    for sticky in [false, true] {
                        let expected = (start..=units.len())
                            .take(if sticky { 1 } else { usize::MAX })
                            .find_map(|candidate| {
                                for count in 0..=units.len() - candidate {
                                    if units[candidate..candidate + count]
                                        .iter()
                                        .all(|u| matches!(u, 97 | 98))
                                        && units.get(candidate + count..candidate + count + 2)
                                            == Some(&[97, 98])
                                    {
                                        return Some((0, candidate..candidate + count + 2));
                                    }
                                }
                                if units.get(candidate) == Some(&97) {
                                    return Some((1, candidate..candidate + 1));
                                }
                                let count = units[candidate..]
                                    .iter()
                                    .take_while(|u| matches!(u, 97 | 98))
                                    .count();
                                (count > 0).then_some((2, candidate..candidate + count))
                            });
                        assert_eq!(
                            m.find_branch(&input, start, sticky),
                            expected,
                            "{input:?} {start} {sticky}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn quantified_alternative_bounds_and_optional_host_charges_remain_distinct() {
        let m = RegExpDisjunctionMatcher::compile(&JsString::from("a+|a*ab|([b])"), false).unwrap();
        assert!(m.requires_full_suffix());
        assert_eq!(m.search_passes(false), 4);
        assert_eq!(m.search_passes(true), 4);
        let fixed = RegExpDisjunctionMatcher::compile(&JsString::from("[ab]a|^b$"), false).unwrap();
        assert!(!fixed.requires_full_suffix());
        let abort = RegExpDisjunctionMatcher::compile_with_work(
            &JsString::from(r"\d+|[a]"),
            false,
            false,
            false,
            |n| {
                if n == 65_536 {
                    Err("host work")
                } else {
                    Ok(())
                }
            },
        );
        assert!(matches!(abort, Err("host work")));
    }

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
    #[test]
    fn complete_branch_group_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a+b)|x",
            "x|(a+b)",
            "((a+b))|x",
            "(?:(a+b))|x",
            "(?:a+b)|x",
            "(a+?aa)|(a+aa)",
            "((a)+b)|(b+)",
            "(^a+$)|x",
            "(^a+?aa$)|(b+)",
            "x|(^((a)+)(b)$)",
            "(a$)|(^b)",
            "((a)*())|x",
            "x|((a*)())",
            "(?:[ab]+ab)|x",
            "([x])|((a)+(b))|([y])",
            "((?:a)*x)|([y])",
            "([ab](c))|x",
            "((ab))|x",
            "()|(a+b)",
            "(a+b)|()",
            r"(\d+x)|([a])",
            "(.+x)|([a])",
            "(µ+x)|([a])",
            r"(\uD800+x)|([a])",
            "(a+[b])|x",
            "(a+b)*|x",
            "a(a+b)|x",
            "(a|b)|x",
            "(?:(a|b))|x",
            "(?<n>a)|x",
            "(a+b)|x*",
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
                        " captures={} branches={} full_suffix={}",
                        matcher.capture_count(),
                        matcher.alternative_count(),
                        matcher.requires_full_suffix()
                    )
                    .unwrap();
                    for text in [
                        "",
                        "a",
                        "aaaa",
                        "aaab",
                        "baaa",
                        "abab",
                        "x",
                        "y",
                        "ABab",
                        "12x",
                        "a\nb",
                        "x\naaab\ny",
                    ] {
                        let input = JsString::from(text);
                        for (start, sticky) in [(0, false), (1, true)] {
                            let result =
                                matcher
                                    .find_branch(&input, start, sticky)
                                    .map(|(branch, r)| {
                                        let captures = (0..matcher.capture_count())
                                            .map(|slot| matcher.capture_range(branch, slot, &r))
                                            .collect::<Vec<_>>();
                                        (branch, r, captures)
                                    });
                            write!(rows, " {text:?}@{start}/{sticky}:{result:?}").unwrap();
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
    fn wrapped_branch_captures_follow_independent_position_and_repetition_order() {
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from("((a)+b)|(b+)"), false).unwrap();
        assert_eq!(matcher.capture_count(), 3);
        for len in 0..=5u32 {
            for mut encoded in 0..3usize.pow(len) {
                let mut units = Vec::new();
                for _ in 0..len {
                    units.push([97, 98, 120][encoded % 3]);
                    encoded /= 3;
                }
                let input = JsString::from_code_units(units.clone());
                for start in 0..=units.len() + 1 {
                    for sticky in [false, true] {
                        let expected = (start..=units.len())
                            .take(if sticky { 1 } else { usize::MAX })
                            .find_map(|candidate| {
                                let mut end = candidate;
                                while units.get(end) == Some(&97) {
                                    end += 1;
                                }
                                if end > candidate && units.get(end) == Some(&98) {
                                    return Some((
                                        0,
                                        candidate..end + 1,
                                        vec![Some(candidate..end + 1), Some(end - 1..end), None],
                                    ));
                                }
                                end = candidate;
                                while units.get(end) == Some(&98) {
                                    end += 1;
                                }
                                (end > candidate).then(|| {
                                    (1, candidate..end, vec![None, None, Some(candidate..end)])
                                })
                            });
                        let actual =
                            matcher
                                .find_branch(&input, start, sticky)
                                .map(|(branch, r)| {
                                    let caps = (0..3)
                                        .map(|i| matcher.capture_range(branch, i, &r))
                                        .collect::<Vec<_>>();
                                    (branch, r, caps)
                                });
                        assert_eq!(actual, expected, "{units:?} {start} {sticky}");
                    }
                }
            }
        }
    }

    #[test]
    fn deep_branch_wrappers_static_layouts_and_optional_work_stay_distinct() {
        let source = format!("{}a+b{}|x", "(".repeat(100_000), ")".repeat(100_000));
        let matcher =
            RegExpDisjunctionMatcher::compile(&JsString::from(source.as_str()), false).unwrap();
        let copy = matcher.clone();
        let input = JsString::from("aaab");
        let (branch, r) = copy.find_branch(&input, 0, false).unwrap();
        assert_eq!(copy.capture_count(), 100_000);
        assert_eq!(copy.capture_range(branch, 0, &r), Some(0..4));
        assert_eq!(copy.capture_range(branch, 99_999, &r), Some(0..4));
        assert_eq!(copy.capture_range(branch, 100_000, &r), None);
        assert_eq!(copy.branch_captures(branch), None);
        let fixed = RegExpDisjunctionMatcher::compile(&JsString::from("((ab))|x"), false).unwrap();
        assert_eq!(fixed.branch_captures(0), Some((0, &[0..2, 0..2][..])));
        assert_eq!(
            RegExpDisjunctionMatcher::compile_with_work(
                &JsString::from("(^[a])|x"),
                false,
                false,
                false,
                |_| Err("abort")
            )
            .unwrap_err(),
            "abort"
        );
        assert!(
            RegExpDisjunctionMatcher::compile_with_work(
                &JsString::from("(a|b)|x"),
                false,
                false,
                false,
                |_| Err::<(), _>("unexpected charge")
            )
            .unwrap()
            .is_none()
        );
    }
}
