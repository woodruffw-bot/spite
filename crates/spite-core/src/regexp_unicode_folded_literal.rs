//! Literal Unicode concatenations with simple/common folding (22.2.2.7.3).
use crate::{
    JsString, RegExpLiteralMatcher, case_data::SIMPLE_CASE_FOLD, regexp_canonicalize_character,
    regexp_literal::unicode_start, regexp_unicode_character::unicode_input_character,
};
use std::{ops::Range, sync::Arc};

/// A flat, immutable nonempty Unicode literal program with simple/common folding.
///
/// The complete Pattern must be validated in u/v mode. Ordinary mandatory
/// capturing/noncapturing groups and empty groups are admitted. Named wrappers
/// must first be normalized to their source-order capture slots. Assertions,
/// references, alternatives, classes and quantifiers are excluded. The existing\n/// literal proof also excludes distinct surrogate atoms that would flatten into\n/// one pair, which no Unicode input can match as two separate characters.
/// Compilation and search add no native recursion or input-sized storage.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeFoldedLiteralMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    points: Vec<u32>,
    failure: Vec<usize>,
    captures: Vec<Range<usize>>,
}

/// A successful match's original absolute UTF-16 boundaries.
#[derive(Debug)]
pub struct RegExpUnicodeFoldedLiteralMatch {
    /// The complete original input substring.
    pub range: Range<usize>,
    /// Mandatory captures in opening-parenthesis order, including empty groups.
    pub captures: Box<[Option<Range<usize>>]>,
}

impl RegExpUnicodeFoldedLiteralMatcher {
    /// Compiles the validated nonempty literal subset with Unicode i.
    pub fn compile(source: &JsString) -> Option<Self> {
        Self::compile_with_work(source, |_| Ok::<(), std::convert::Infallible>(()))
            .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same subset with fallible opt-in construction work.
    ///
    /// Source work is charged before literal decoding and temporary flat storage.
    /// Each pinned-table lookup and capture-boundary lookup is also bounded.
    pub fn compile_with_work<E>(
        source: &JsString,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        // Covers the existing literal compiler, flat vectors and prefix passes,
        // including allocation moves. A rejected body is charged before scanning.
        charge(source.len().saturating_mul(32))?;
        let Some(literal) = RegExpLiteralMatcher::compile_unicode_code_points(source) else {
            return Ok(None);
        };
        let units = literal.matched_units();
        let mut boundaries = vec![0];
        let mut points = Vec::new();
        let mut cursor = 0;
        while let Some(&first) = units.get(cursor) {
            let (value, width) = unicode_input_character(units, cursor, first);
            charge(fold_work())?;
            points.push(regexp_canonicalize_character(value, true, true));
            cursor += width;
            boundaries.push(cursor);
        }
        debug_assert!(!points.is_empty());
        let levels = (usize::BITS - boundaries.len().leading_zeros()) as usize;
        charge(
            literal
                .capture_ranges()
                .len()
                .saturating_mul(2usize.saturating_mul(2 + levels)),
        )?;
        let mut captures = Vec::with_capacity(literal.capture_ranges().len());
        for range in literal.capture_ranges() {
            let (Ok(start), Ok(end)) = (
                boundaries.binary_search(&range.start),
                boundaries.binary_search(&range.end),
            ) else {
                return Ok(None);
            };
            captures.push(start..end);
        }
        let mut failure = vec![0; points.len()];
        let mut matched = 0;
        for index in 1..points.len() {
            while matched > 0 && points[index] != points[matched] {
                matched = failure[matched - 1];
            }
            if points[index] == points[matched] {
                matched += 1;
            }
            failure[index] = matched;
        }
        Ok(Some(Self(Arc::new(Program {
            points,
            failure,
            captures,
        }))))
    }

    /// Number of mandatory captures, excluding the complete match.
    pub fn capture_count(&self) -> usize {
        self.0.captures.len()
    }

    /// Searches complete input characters with original widths and capture bounds.
    ///
    /// KMP compares canonical code points, so changes of UTF-16 width cannot
    /// affect candidate order. A successful match is decoded once more to resolve
    /// capture boundaries. Temporary storage is bounded by the compiled source.
    /// Initial offsets inside pairs use the approved Node/V8 leading boundary.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<RegExpUnicodeFoldedLiteralMatch>, E> {
        charge(1)?;
        let Some(mut cursor) = unicode_start(input, start) else {
            return Ok(None);
        };
        let units = input.code_units();
        let mut matched = 0;
        while let Some(&first) = units.get(cursor) {
            charge(2 + fold_work())?;
            let (value, width) = unicode_input_character(units, cursor, first);
            let value = regexp_canonicalize_character(value, true, true);
            while matched > 0 && value != self.0.points[matched] {
                charge(2)?;
                if sticky {
                    return Ok(None);
                }
                matched = self.0.failure[matched - 1];
            }
            charge(1)?;
            if value == self.0.points[matched] {
                matched += 1;
            } else if sticky {
                return Ok(None);
            }
            cursor += width;
            if matched == self.0.points.len() {
                let end = cursor;
                // Recover the original start without keeping all scanned offsets.
                for _ in 0..matched {
                    charge(2)?;
                    cursor -= 1;
                    if cursor > 0
                        && (0xdc00..=0xdfff).contains(&units[cursor])
                        && (0xd800..=0xdbff).contains(&units[cursor - 1])
                    {
                        cursor -= 1;
                    }
                }
                let range = cursor..end;
                let captures = self.resolve_captures(units, &range, &mut charge)?;
                return Ok(Some(RegExpUnicodeFoldedLiteralMatch { range, captures }));
            }
        }
        Ok(None)
    }

    fn resolve_captures<E>(
        &self,
        units: &[u16],
        range: &Range<usize>,
        charge: &mut impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Box<[Option<Range<usize>>]>, E> {
        if self.0.captures.is_empty() {
            return Ok(Box::default());
        }
        charge(
            self.0
                .points
                .len()
                .saturating_add(1)
                .saturating_add(self.0.captures.len().saturating_mul(3)),
        )?;
        let mut boundaries = vec![0; self.0.points.len() + 1];
        boundaries[0] = range.start;
        let mut cursor = range.start;
        for boundary in boundaries.iter_mut().skip(1) {
            charge(2)?;
            let (_, width) = unicode_input_character(units, cursor, units[cursor]);
            cursor += width;
            *boundary = cursor;
        }
        debug_assert_eq!(cursor, range.end);
        Ok(self
            .0
            .captures
            .iter()
            .map(|range| Some(boundaries[range.start]..boundaries[range.end]))
            .collect())
    }
}

fn fold_work() -> usize {
    2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn folded_unicode_literal_capture_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a)A(b)",
            "(())a(a)",
            "(ß)(s)()",
            r"(\u{10400})(\u{10428})",
            r"(\uD800)(a)",
            r"(\$)(\^)",
            "((ſ)(K))",
            "(a)(A)",
            "(a)a(a)",
        ] {
            let matcher =
                RegExpUnicodeFoldedLiteralMatcher::compile(&JsString::from(source)).unwrap();
            for input in [
                JsString::from("aaabſK😀ßS"),
                JsString::from("\u{10428}\u{10400}"),
                JsString::from_code_units(vec![0xd800, 0x41, 0xdc00, 0x61]),
                JsString::from("$^"),
                JsString::from(""),
            ] {
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        let found = matcher
                            .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                            .unwrap()
                            .map(|m| (m.range, m.captures));
                        writeln!(
                            rows,
                            "{:?} input={input:?} start={start} sticky={sticky} {found:?}",
                            JsString::from(source)
                        )
                        .unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    // Independently pair raw units, then enumerate candidate point substrings.
    fn decoded(units: &[u16]) -> (Vec<u32>, Vec<usize>) {
        let mut values = Vec::new();
        let mut offsets = vec![0];
        let mut i = 0;
        while i < units.len() {
            let first = u32::from(units[i]);
            i += 1;
            let value = if (0xd800..=0xdbff).contains(&first)
                && i < units.len()
                && (0xdc00..=0xdfff).contains(&units[i])
            {
                let second = u32::from(units[i]);
                i += 1;
                (first - 0xd800) * 1024 + (second - 0xdc00) + 65536
            } else {
                first
            };
            values.push(match value {
                0x41 => 0x61,
                0x53 | 0x17f => 0x73,
                _ => value,
            });
            offsets.push(i);
        }
        (values, offsets)
    }

    #[test]
    fn folded_unicode_literal_candidates_and_captures_agree_with_point_oracle() {
        type Case = (&'static str, &'static [u32], &'static [(usize, usize)]);
        let cases: &[Case] = &[
            ("aA", &[0x61, 0x61], &[]),
            ("(a)(s)", &[0x61, 0x73], &[(0, 1), (1, 2)]),
            ("(a())(a)", &[0x61, 0x61], &[(0, 1), (1, 1), (1, 2)]),
            ("(())a(a)", &[0x61, 0x61], &[(0, 0), (0, 0), (1, 2)]),
            (r"(\uD800)(a)", &[0xd800, 0x61], &[(0, 1), (1, 2)]),
            (r"(\u{1f600})(a)", &[0x1f600, 0x61], &[(0, 1), (1, 2)]),
            (r"(a)(\u{1f600})", &[0x61, 0x1f600], &[(0, 1), (1, 2)]),
            ("((a)a(a))", &[0x61, 0x61, 0x61], &[(0, 3), (0, 1), (2, 3)]),
            ("(a)A(b)", &[0x61, 0x61, 0x62], &[(0, 1), (2, 3)]),
        ];
        let plans: Vec<_> = cases
            .iter()
            .map(|(source, _, _)| {
                RegExpUnicodeFoldedLiteralMatcher::compile(&JsString::from(*source)).unwrap()
            })
            .collect();
        let alphabet = [
            0x41, 0x61, 0x53, 0x17f, 0x62, 0xd83d, 0xde00, 0xd800, 0xdc00,
        ];
        for len in 0..=4 {
            for mut code in 0..alphabet.len().pow(len as u32) {
                let mut units = vec![0; len];
                for unit in &mut units {
                    *unit = alphabet[code % alphabet.len()];
                    code /= alphabet.len();
                }
                let (points, offsets) = decoded(&units);
                let input = JsString::from_code_units(units);
                for ((source, wanted, captures), plan) in cases.iter().zip(&plans) {
                    for start in 0..=input.len() + 1 {
                        for sticky in [false, true] {
                            let expected = if start > input.len() {
                                None
                            } else {
                                let first = offsets.partition_point(|&n| n <= start) - 1;
                                let end = if sticky { first + 1 } else { points.len() };
                                (first..end).find_map(|i| {
                                    if points.get(i..i + wanted.len()) != Some(*wanted) {
                                        return None;
                                    }
                                    Some((
                                        offsets[i]..offsets[i + wanted.len()],
                                        captures
                                            .iter()
                                            .map(|&(a, b)| Some(offsets[i + a]..offsets[i + b]))
                                            .collect::<Vec<_>>(),
                                    ))
                                })
                            };
                            let actual = plan
                                .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                                .unwrap()
                                .map(|m| (m.range, m.captures.into_vec()));
                            assert_eq!(
                                actual, expected,
                                "{source} {input:?} start={start} sticky={sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn folded_unicode_literal_cross_plane_captures_keep_original_widths() {
        for (source, input, ranges) in [
            (
                "(ß)(s)()",
                "\u{1df95}S",
                vec![Some(0..2), Some(2..3), Some(3..3)],
            ),
            (
                r"(\u{1df95})(s)()",
                "ßſ",
                vec![Some(0..1), Some(1..2), Some(2..2)],
            ),
            (
                "((ß)())s",
                "\u{1df95}S",
                vec![Some(0..2), Some(0..2), Some(2..2)],
            ),
        ] {
            let plan = RegExpUnicodeFoldedLiteralMatcher::compile(&JsString::from(source)).unwrap();
            let input = JsString::from(input);
            for start in [0, usize::from(input.len() == 3)] {
                let found = plan
                    .find_with_work(&input, start, true, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap();
                assert_eq!(found.range, 0..input.len());
                assert_eq!(found.captures.as_ref(), ranges.as_slice());
            }
        }
    }

    #[test]
    fn folded_unicode_literal_contracts_clones_and_every_fallible_charge() {
        for source in [
            "",
            "()",
            "a+",
            "a|b",
            "[a]b",
            "^ab",
            "ab$",
            r"(a)\1",
            r"(\uD800)(\uDC00)",
            r"\u{D800}\u{DC00}",
        ] {
            assert!(
                RegExpUnicodeFoldedLiteralMatcher::compile(&JsString::from(source)).is_none(),
                "{source}"
            );
        }
        let source = JsString::from("(a())A(b)");
        let mut charges = Vec::new();
        let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_work(&source, |n| {
            charges.push(n);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(charges[0], 32 * source.len());
        assert_eq!(plan.capture_count(), 3);
        for failure in 0..charges.len() {
            let mut calls = 0;
            let result = RegExpUnicodeFoldedLiteralMatcher::compile_with_work(&source, |_| {
                let index = calls;
                calls += 1;
                if index == failure { Err(17) } else { Ok(()) }
            });
            assert!(matches!(result, Err(17)));
            assert_eq!(calls, failure + 1);
        }
        let input = JsString::from("aaab");
        let mut charges = Vec::new();
        let found = plan
            .clone()
            .find_with_work(&input, 0, false, |n| {
                charges.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 1..4);
        for failure in 0..charges.len() {
            let mut calls = 0;
            let result = plan.find_with_work(&input, 0, false, |_| {
                let index = calls;
                calls += 1;
                if index == failure { Err(17) } else { Ok(()) }
            });
            assert!(matches!(result, Err(17)));
            assert_eq!(calls, failure + 1);
        }
        let source = JsString::from(format!("{}aA{}", "(".repeat(5000), ")".repeat(5000)).as_str());
        let plan = RegExpUnicodeFoldedLiteralMatcher::compile(&source).unwrap();
        assert_eq!(plan.capture_count(), 5000);
        let input = JsString::from(format!("{}Aa", "😀".repeat(100000)).as_str());
        let found = plan
            .clone()
            .find_with_work(&input, 1, false, |_| Ok::<(), ()>(()))
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 200000..200002);
        assert!(found.captures.iter().all(|c| c == &Some(200000..200002)));
    }
}
