//! Top-level literal Disjunction compilation and matching (22.2.2.3).

use crate::{JsString, RegExpLiteralMatcher};
use std::{ops::Range, sync::Arc};

/// Immutable top-level alternatives of noncapturing ordinary literal sequences.
///
/// The Pattern must already be validated without `u` or `v`. Every alternative
/// must compile as a literal sequence without captures. Unsupported alternatives
/// reject the entire plan. Nested alternatives and quantified groups remain
/// unsupported; compilation never expands combinations or uses native recursion.
#[derive(Clone, Debug)]
pub struct RegExpDisjunctionMatcher(Arc<[RegExpLiteralMatcher]>);

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
                    if units.get(index..index + 2)? != [u16::from(b'?'), u16::from(b':')] {
                        return None;
                    }
                    index += 2;
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
        for range in ranges {
            let source = JsString::from_code_units(units[range].to_vec());
            let matcher = RegExpLiteralMatcher::compile(&source, ignore_case)?;
            if !matcher.capture_ranges().is_empty() {
                return None;
            }
            alternatives.push(matcher);
        }
        Some(Self(alternatives.into()))
    }

    /// Number of alternatives, for optional worst-case work accounting.
    pub fn alternative_count(&self) -> usize {
        self.0.len()
    }

    /// Finds the earliest match, choosing source order for equal start offsets.
    ///
    /// Sticky matching considers only `start`. Each branch uses linear literal
    /// search; worst-case work is proportional to the number of alternatives
    /// times the input suffix length. Search allocates nothing.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut best: Option<Range<usize>> = None;
        for alternative in self.0.iter() {
            if let Some(found) = alternative.find(input, start, sticky) {
                if found.start == start {
                    return Some(found);
                }
                if best.as_ref().is_none_or(|best| found.start < best.start) {
                    best = Some(found);
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
    fn alternatives_agree_with_independent_position_then_source_order_oracle() {
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
                let source = JsString::from(format!("{left}|{right}").as_str());
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
                                    [left, right].into_iter().find_map(|word| {
                                        let end = offset.checked_add(word.len())?;
                                        units
                                            .get(offset..end)?
                                            .iter()
                                            .copied()
                                            .zip(word.bytes().map(u16::from))
                                            .all(|(a, b)| {
                                                fold(a, ignore_case) == fold(b, ignore_case)
                                            })
                                            .then_some(offset..end)
                                    })
                                });
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
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
}
