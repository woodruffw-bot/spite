//! Literal Unicode concatenations and boundary assertions (22.2.2.4, 22.2.2.7.3).
use crate::{
    JsString, RegExpLiteralMatcher,
    case_data::SIMPLE_CASE_FOLD,
    regexp_canonicalize_character,
    regexp_literal::unicode_start,
    regexp_outer_group_body,
    regexp_unicode_character::{unicode_assertions_match, unicode_input_character},
};
use std::{ops::Range, sync::Arc};

/// A flat, immutable nonempty Unicode literal program with optional simple folding.
///
/// The complete Pattern must be validated in u/v mode. Ordinary mandatory
/// capturing/noncapturing groups and empty groups are admitted. Named wrappers
/// must first be normalized to their source-order capture slots. Optional outer
/// boundary assertions have a separate entry point. References, alternatives,
/// classes and quantifiers are excluded. The existing literal proof also excludes
/// distinct surrogate atoms that would flatten into one pair, which no Unicode
/// input can match as two separate characters.
/// Compilation and search add no native recursion or input-sized storage.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeFoldedLiteralMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    points: Vec<u32>,
    failure: Vec<usize>,
    captures: Vec<Range<usize>>,
    start_anchor: bool,
    end_anchor: bool,
    multiline: bool,
    ignore_case: bool,
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
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_impl(source, false, true, false, charge)
    }

    /// Compiles optional leading ^ and trailing $ around the literal body.
    ///
    /// Complete ordinary enclosing groups may include these assertions. Other
    /// assertion positions remain excluded. The Pattern must be validated and
    /// any named groups normalized before calling this method.
    pub fn compile_with_assertions(source: &JsString, multiline: bool) -> Option<Self> {
        Self::compile_with_assertions_and_work(source, multiline, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the assertion subset with fallible opt-in construction work.
    pub fn compile_with_assertions_and_work<E>(
        source: &JsString,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_impl(source, true, true, multiline, charge)
    }

    /// Compiles the literal boundary subset with explicit Unicode i and m flags.
    ///
    /// With i disabled, complete code points are compared exactly. The validated
    /// Pattern and normalized named-wrapper requirements remain unchanged.
    pub fn compile_with_flags(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
    ) -> Option<Self> {
        Self::compile_with_flags_and_work(source, ignore_case, multiline, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles explicit Unicode flags with fallible opt-in construction work.
    pub fn compile_with_flags_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_impl(source, true, ignore_case, multiline, charge)
    }

    fn compile_impl<E>(
        source: &JsString,
        assertions: bool,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        // Covers the existing literal compiler, flat vectors and prefix passes,
        // including allocation moves. A rejected body is charged before scanning.
        charge(
            source
                .len()
                .saturating_mul(if assertions { 48 } else { 32 }),
        )?;
        let units = source.code_units();
        let outer = assertions
            .then(|| regexp_outer_group_body(source))
            .flatten();
        let wrappers = outer.as_ref().map_or(0, |group| group.captures);
        let mut body = outer.map_or(0..units.len(), |group| group.body);
        let start_anchor = assertions && units.get(body.start) == Some(&0x5e);
        body.start += usize::from(start_anchor);
        let end_anchor = assertions
            && body.end > body.start
            && units[body.end - 1] == 0x24
            && units[body.start..body.end - 1]
                .iter()
                .rev()
                .take_while(|&&unit| unit == 0x5c)
                .count()
                % 2
                == 0;
        body.end -= usize::from(end_anchor);
        // The source charge precedes this copy and the delimiter scans. Complete
        // literal compilation excludes Unicode set/string syntax those scans do
        // not interpret, as required by the outer-group helper's contract.
        let retained = assertions.then(|| JsString::from_code_units(units[body].to_vec()));
        let Some(literal) =
            RegExpLiteralMatcher::compile_unicode_code_points(retained.as_ref().unwrap_or(source))
        else {
            return Ok(None);
        };
        let units = literal.matched_units();
        let mut boundaries = vec![0];
        let mut points = Vec::new();
        let mut cursor = 0;
        while let Some(&first) = units.get(cursor) {
            let (value, width) = unicode_input_character(units, cursor, first);
            let point = if ignore_case {
                charge(fold_work())?;
                regexp_canonicalize_character(value, true, true)
            } else {
                value
            };
            points.push(point);
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
        let mut captures = Vec::with_capacity(wrappers + literal.capture_ranges().len());
        captures.extend((0..wrappers).map(|_| 0..points.len()));
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
            start_anchor,
            end_anchor,
            multiline,
            ignore_case,
        }))))
    }

    /// Number of mandatory captures, excluding the complete match.
    pub fn capture_count(&self) -> usize {
        self.0.captures.len()
    }

    /// Searches complete input characters with original widths and capture bounds.
    ///
    /// KMP compares complete code points, canonicalized with i, so width changes cannot
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
        if self.0.start_anchor && !self.0.multiline && cursor != 0 {
            return Ok(None);
        }
        let sticky = sticky || (self.0.start_anchor && !self.0.multiline);
        let initial = cursor;
        // A multiline start assertion needs each candidate's original start,
        // including rejected overlapping candidates. A source-sized ring keeps
        // that lookup constant-time instead of rescanning each candidate.
        let mut starts = if self.0.start_anchor && self.0.multiline {
            if units.len() - cursor < self.0.points.len() {
                return Ok(None);
            }
            charge(self.0.points.len().saturating_add(1))?;
            let mut ring = vec![0; self.0.points.len() + 1];
            ring[0] = cursor;
            ring
        } else {
            Vec::new()
        };
        let mut consumed = 0;
        let mut matched = 0;
        while let Some(&first) = units.get(cursor) {
            charge(2 + if self.0.ignore_case { fold_work() } else { 0 })?;
            let (value, width) = unicode_input_character(units, cursor, first);
            let value = if self.0.ignore_case {
                regexp_canonicalize_character(value, true, true)
            } else {
                value
            };
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
            if !starts.is_empty() {
                charge(1)?;
                consumed += 1;
                let slot = consumed % starts.len();
                starts[slot] = cursor;
            }
            if matched == self.0.points.len() {
                let end = cursor;
                let known_start = if !starts.is_empty() {
                    Some(starts[(consumed - matched) % starts.len()])
                } else {
                    sticky.then_some(initial)
                };
                charge(2)?;
                // Check an end assertion before recovering a start. Rejected
                // suffixes must not turn a linear KMP scan into repeated walks.
                if !unicode_assertions_match(
                    units,
                    known_start.unwrap_or(0),
                    end,
                    self.0.start_anchor,
                    self.0.end_anchor,
                    self.0.multiline,
                ) {
                    if sticky {
                        return Ok(None);
                    }
                    matched = self.0.failure[matched - 1];
                    continue;
                }
                // Recover the original start without keeping all scanned offsets.
                if let Some(start) = known_start {
                    cursor = start;
                } else {
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

    #[test]
    fn folded_unicode_literal_boundary_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            "^(a)A(b)$",
            "((^a(a)$))",
            "(aA$)",
            "^a(a)",
            r"^(\u{10400})(\u{10428})$",
            r"^(\$)(a)\$$",
            "^(())a(a)$",
            "^a(a)$",
            "a(a)$",
            "^a(a)",
        ] {
            for multiline in [false, true] {
                let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions(
                    &JsString::from(source),
                    multiline,
                )
                .unwrap();
                for input in [
                    "aaab\nAAB\r\nAA\u{2028}aa\u{2029}",
                    "\u{10428}\u{10400}\n\u{10400}\u{10428}",
                    "$A$\n$A$",
                    "AA\n",
                    "",
                ] {
                    let input = JsString::from(input);
                    for start in 0..=input.len() + 1 {
                        for sticky in [false, true] {
                            let found = plan
                                .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                                .unwrap()
                                .map(|m| (m.range, m.captures));
                            writeln!(rows,"{:?} m={multiline} input={input:?} start={start} sticky={sticky} {found:?}",JsString::from(source)).unwrap();
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn folded_unicode_literal_boundaries_independent_candidates() {
        let plans = [
            (
                "^(a)A$",
                vec![0x61, 0x61],
                std::iter::once(0..1).collect(),
                true,
                true,
            ),
            (
                "((^a(a)$))",
                vec![0x61, 0x61],
                vec![0..2, 0..2, 1..2],
                true,
                true,
            ),
            ("(a(a)$)", vec![0x61, 0x61], vec![0..2, 1..2], false, true),
            (
                "^a(a)",
                vec![0x61, 0x61],
                std::iter::once(1..2).collect(),
                true,
                false,
            ),
            (
                "^(())a(a)$",
                vec![0x61, 0x61],
                vec![0..0, 0..0, 1..2],
                true,
                true,
            ),
            (
                r"^(\uD800)(a)$",
                vec![0xd800, 0x61],
                vec![0..1, 1..2],
                true,
                true,
            ),
            (
                "^a(ſ)$",
                vec![0x61, 0x73],
                std::iter::once(1..2).collect(),
                true,
                true,
            ),
        ];
        let alphabet = [0x41u16, 0x61, 0x17f, 0xd800, 0xdc00, 10, 13, 0x2028, 0x2029];
        for length in 0..=4 {
            for encoded in 0..alphabet.len().pow(length) {
                let mut value = encoded;
                let raw: Vec<_> = (0..length)
                    .map(|_| {
                        let unit = alphabet[value % alphabet.len()];
                        value /= alphabet.len();
                        unit
                    })
                    .collect();
                let input = JsString::from_code_units(raw.clone());
                let (points, bounds) = decoded(&raw);
                for (source, pattern, caps, begin, end) in &plans {
                    for multiline in [false, true] {
                        let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions(
                            &JsString::from(*source),
                            multiline,
                        )
                        .unwrap();
                        for start in 0..=raw.len() + 1 {
                            for sticky in [false, true] {
                                let normalized = if start > 0
                                    && start < raw.len()
                                    && (0xdc00..=0xdfff).contains(&raw[start])
                                    && (0xd800..=0xdbff).contains(&raw[start - 1])
                                {
                                    start - 1
                                } else {
                                    start
                                };
                                let mut expected = None;
                                for (offset, &first) in bounds.iter().enumerate() {
                                    if first < normalized || (sticky && first != normalized) {
                                        continue;
                                    }
                                    let last_point = offset + pattern.len();
                                    if points.get(offset..last_point) != Some(pattern.as_slice()) {
                                        continue;
                                    }
                                    let last = bounds[last_point];
                                    let lt = |unit| matches!(unit, 10 | 13 | 0x2028 | 0x2029);
                                    if (*begin && first != 0 && !(multiline && lt(raw[first - 1])))
                                        || (*end
                                            && last != raw.len()
                                            && !(multiline && lt(raw[last])))
                                    {
                                        continue;
                                    }
                                    expected = Some((
                                        first..last,
                                        caps.iter()
                                            .map(|cap| {
                                                Some(
                                                    bounds[offset + cap.start]
                                                        ..bounds[offset + cap.end],
                                                )
                                            })
                                            .collect::<Box<[_]>>(),
                                    ));
                                    break;
                                }
                                let actual = plan
                                    .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                                    .unwrap()
                                    .map(|m| (m.range, m.captures));
                                assert_eq!(
                                    actual, expected,
                                    "{source:?} m={multiline} raw={raw:?} start={start} sticky={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn folded_unicode_literal_boundaries_linear_rejections_and_work() {
        for source in [
            format!("^{}$", "a".repeat(5000)),
            format!("{}$", "a".repeat(5000)),
        ] {
            let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions(
                &JsString::from(source.as_str()),
                true,
            )
            .unwrap();
            let input = JsString::from(("a".repeat(100000) + "b").as_str());
            let mut work = 0;
            assert!(
                plan.find_with_work(&input, 0, false, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap()
                .is_none()
            );
            assert!(work < 25 * input.len() + 10000, "work={work}");
        }
        let source = JsString::from("((^(a)a$))");
        let mut calls = 0;
        let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions_and_work(
            &source,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions_and_work(
                    &source,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let input = JsString::from("xxAA\nAA");
        let mut calls = 0;
        let found = plan
            .clone()
            .find_with_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 5..7);
        assert_eq!(&*found.captures, &[Some(5..7), Some(5..7), Some(5..6)]);
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                plan.find_with_work(&input, 0, false, |_| {
                    n += 1;
                    if n == fail { Err(()) } else { Ok(()) }
                })
                .is_err()
            );
        }
        let ring = RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions(
            &JsString::from("^aa$"),
            true,
        )
        .unwrap();
        let mut calls = 0;
        assert!(
            ring.find_with_work(&input, 0, false, |_| {
                calls += 1;
                if calls == 2 { Err(()) } else { Ok(()) }
            })
            .is_err()
        );
        assert_eq!(calls, 2);
    }

    #[test]
    fn folded_unicode_literal_boundary_contracts_and_original_widths() {
        for source in [
            "^ab", "ab$", "(^ab$)", "^$", "^(a$)b", "a^b", "a$b", "^(ab)+$", "^[ab]$", "^(a|b)$",
        ] {
            assert!(RegExpUnicodeFoldedLiteralMatcher::compile(&JsString::from(source)).is_none());
        }
        for source in [
            "^$",
            "^(a$)b",
            "a^b",
            "a$b",
            "^(ab)+$",
            "^[ab]$",
            "^(a|b)$",
            r"^[\q{a\)b}]$",
        ] {
            assert!(
                RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions(
                    &JsString::from(source),
                    true
                )
                .is_none()
            );
        }
        for (source, input) in [
            (r"^(\$)(a)\$$", "$A$"),
            (r"^(a)\\$", "A\\"),
            (r"^(a)\$$", "A$"),
        ] {
            let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions(
                &JsString::from(source),
                false,
            )
            .unwrap();
            assert_eq!(
                plan.find_with_work(&JsString::from(input), 0, false, |_| Ok::<(), ()>(()))
                    .unwrap()
                    .unwrap()
                    .range,
                0..input.encode_utf16().count()
            );
        }
        for (source, input, last) in [
            ("((^(ß)(s)()$))", "\u{1df95}S", 3),
            (r"^(\u{1df95})(s)()$", "ßS", 2),
        ] {
            let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_assertions(
                &JsString::from(source),
                false,
            )
            .unwrap();
            let found = plan
                .find_with_work(&JsString::from(input), 1, false, |_| Ok::<(), ()>(()))
                .unwrap();
            if last == 3 {
                let found = found.unwrap();
                assert_eq!(found.range, 0..3);
                assert_eq!(
                    &*found.captures,
                    &[Some(0..3), Some(0..3), Some(0..2), Some(2..3), Some(3..3)]
                );
            } else {
                assert!(found.is_none());
            }
        }
    }

    #[test]
    fn exact_unicode_literal_boundary_ranges_snapshot() {
        let mut rows = String::new();
        for source in [
            "^(A)(b)$",
            "((^A(b)$))",
            "(Ab$)",
            "^A(b)",
            r"^(\u{10400})(\u{10428})$",
            r"^(\$)(A)\$$",
            "^(())A(b)$",
            "A(b)$",
        ] {
            for multiline in [false, true] {
                let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_flags(
                    &JsString::from(source),
                    false,
                    multiline,
                )
                .unwrap();
                for input in [
                    "Ab\naB\r\nAb\u{2028}AB\u{2029}",
                    "\u{10400}\u{10428}\n\u{10428}\u{10400}",
                    "$A$\n$a$",
                    "Ab\n",
                    "",
                ] {
                    let input = JsString::from(input);
                    for start in 0..=input.len() + 1 {
                        for sticky in [false, true] {
                            let found = plan
                                .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                                .unwrap()
                                .map(|m| (m.range, m.captures));
                            writeln!(rows,"{:?} m={multiline} input={input:?} start={start} sticky={sticky} {found:?}",JsString::from(source)).unwrap();
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn exact_unicode_literal_boundaries_independent_raw_candidates() {
        let plans = [
            ("^(A)(b)$", vec![0x41, 0x62], vec![0..1, 1..2], true, true),
            (
                "((^A(b)$))",
                vec![0x41, 0x62],
                vec![0..2, 0..2, 1..2],
                true,
                true,
            ),
            (
                "(Ab$)",
                vec![0x41, 0x62],
                std::iter::once(0..2).collect(),
                false,
                true,
            ),
            (
                "^A(b)",
                vec![0x41, 0x62],
                std::iter::once(1..2).collect(),
                true,
                false,
            ),
            (
                "^(())A(b)$",
                vec![0x41, 0x62],
                vec![0..0, 0..0, 1..2],
                true,
                true,
            ),
            (
                r"^(\u{10000})(A)$",
                vec![0x10000, 0x41],
                vec![0..1, 1..2],
                true,
                true,
            ),
            (
                r"^(\uD800)(A)$",
                vec![0xd800, 0x41],
                vec![0..1, 1..2],
                true,
                true,
            ),
        ];
        let alphabet = [
            0x41u16, 0x61, 0x42, 0x62, 0xd800, 0xdc00, 10, 13, 0x2028, 0x2029,
        ];
        for length in 0..=4 {
            for encoded in 0..alphabet.len().pow(length) {
                let mut value = encoded;
                let raw: Vec<_> = (0..length)
                    .map(|_| {
                        let unit = alphabet[value % alphabet.len()];
                        value /= alphabet.len();
                        unit
                    })
                    .collect();
                let input = JsString::from_code_units(raw.clone());
                let mut points = Vec::new();
                let mut bounds = vec![0];
                let mut index = 0;
                while index < raw.len() {
                    let first = u32::from(raw[index]);
                    index += 1;
                    let point = if (0xd800..=0xdbff).contains(&first)
                        && index < raw.len()
                        && (0xdc00..=0xdfff).contains(&raw[index])
                    {
                        let second = u32::from(raw[index]);
                        index += 1;
                        (first - 0xd800) * 1024 + second - 0xdc00 + 65536
                    } else {
                        first
                    };
                    points.push(point);
                    bounds.push(index);
                }
                for (source, pattern, caps, begin, end) in &plans {
                    for multiline in [false, true] {
                        let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_flags(
                            &JsString::from(*source),
                            false,
                            multiline,
                        )
                        .unwrap();
                        for start in 0..=raw.len() + 1 {
                            for sticky in [false, true] {
                                let normalized = if start > 0
                                    && start < raw.len()
                                    && (0xdc00..=0xdfff).contains(&raw[start])
                                    && (0xd800..=0xdbff).contains(&raw[start - 1])
                                {
                                    start - 1
                                } else {
                                    start
                                };
                                let mut expected = None;
                                for (offset, &first) in bounds.iter().enumerate() {
                                    if first < normalized || (sticky && first != normalized) {
                                        continue;
                                    }
                                    let last_point = offset + pattern.len();
                                    if points.get(offset..last_point) != Some(pattern.as_slice()) {
                                        continue;
                                    }
                                    let last = bounds[last_point];
                                    let lt = |unit| matches!(unit, 10 | 13 | 0x2028 | 0x2029);
                                    if (*begin && first != 0 && !(multiline && lt(raw[first - 1])))
                                        || (*end
                                            && last != raw.len()
                                            && !(multiline && lt(raw[last])))
                                    {
                                        continue;
                                    }
                                    expected = Some((
                                        first..last,
                                        caps.iter()
                                            .map(|cap| {
                                                Some(
                                                    bounds[offset + cap.start]
                                                        ..bounds[offset + cap.end],
                                                )
                                            })
                                            .collect::<Box<[_]>>(),
                                    ));
                                    break;
                                }
                                let actual = plan
                                    .find_with_work(&input, start, sticky, |_| Ok::<(), ()>(()))
                                    .unwrap()
                                    .map(|m| (m.range, m.captures));
                                assert_eq!(
                                    actual, expected,
                                    "{source} m={multiline} raw={raw:?} start={start} sticky={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn exact_unicode_literal_flags_work_clones_and_original_boundaries() {
        let source = JsString::from("((^(A)b$))");
        let input = JsString::from("ab\nAb");
        let mut calls = 0;
        let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_flags_and_work(
            &source,
            false,
            true,
            |_| {
                calls += 1;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                RegExpUnicodeFoldedLiteralMatcher::compile_with_flags_and_work(
                    &source,
                    false,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let mut calls = 0;
        let found = plan
            .clone()
            .find_with_work(&input, 0, false, |_| {
                calls += 1;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(found.range, 3..5);
        assert_eq!(&*found.captures, &[Some(3..5), Some(3..5), Some(3..4)]);
        for fail in 1..=calls {
            let mut n = 0;
            assert!(
                plan.find_with_work(&input, 0, false, |_| {
                    n += 1;
                    if n == fail { Err(()) } else { Ok(()) }
                })
                .is_err()
            );
        }
        for ignore_case in [false, true] {
            let plan = RegExpUnicodeFoldedLiteralMatcher::compile_with_flags(
                &JsString::from("^(ß)(S)$"),
                ignore_case,
                false,
            )
            .unwrap();
            let found = plan
                .find_with_work(
                    &JsString::from("\u{1df95}s"),
                    1,
                    false,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap();
            assert_eq!(found.is_some(), ignore_case);
            if let Some(found) = found {
                assert_eq!(found.range, 0..3);
                assert_eq!(&*found.captures, &[Some(0..2), Some(2..3)]);
            }
        }
        let bare = RegExpUnicodeFoldedLiteralMatcher::compile(&JsString::from("Ab")).unwrap();
        assert!(
            bare.find_with_work(&JsString::from("aB"), 0, false, |_| Ok::<(), ()>(()))
                .unwrap()
                .is_some()
        );
        for source in ["^$", "a^b", "a$b", "^(A$)b", "^(Ab)+$", "^[Ab]$", "^A|b$"] {
            assert!(
                RegExpUnicodeFoldedLiteralMatcher::compile_with_flags(
                    &JsString::from(source),
                    false,
                    true
                )
                .is_none()
            );
        }
    }
}
