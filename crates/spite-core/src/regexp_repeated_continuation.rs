//! A repeated fixed group followed by a fixed sequel (22.2.2.3.1).

use crate::regexp_assertion::Assertions;
use crate::{JsString, RegExpRepeatedSequenceMatcher, RegExpSequenceMatcher};
use std::ops::Range;

/// A complete ordinary fixed repeated group followed by a fixed sequel.
///
/// The Pattern must already be validated without `u` or `v`. Both components
/// support nested ordinary captures and word/input/line assertions with explicit
/// flags. The sequel may be empty but contain no further quantifier or nested choice. Counts
/// remain compact and matching uses the fixed-sequence candidate bound.
#[derive(Clone, Debug)]
pub struct RegExpRepeatedContinuationMatcher {
    repeated: RegExpRepeatedSequenceMatcher,
    sequel: RegExpSequenceMatcher,
    capture_count: usize,
}

impl RegExpRepeatedContinuationMatcher {
    /// Compiles the complete group/quantifier/sequel layout with explicit flags.
    pub fn compile(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
    ) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, multiline, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Accepts both complete components before charging optional construction work.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let Some(end) = crate::regexp_repeated_literal::group_end(units) else {
            return Ok(None);
        };
        let Some((_, width)) = crate::regexp_quantified::quantifier(&units[end..]) else {
            return Ok(None);
        };
        let split = end + width;
        if split == units.len() {
            return Ok(None);
        }
        let atom = JsString::from_code_units(units[..end].to_vec());
        let sequel = JsString::from_code_units(units[split..].to_vec());
        if RegExpSequenceMatcher::repeated_atom_width(&atom, dot_all, true).is_none()
            || RegExpSequenceMatcher::repeated_atom_width(&sequel, dot_all, true).is_none()
        {
            return Ok(None);
        }
        charge(units.len())?;
        let repeated = JsString::from_code_units(units[..split].to_vec());
        let Some(repeated) = RegExpRepeatedSequenceMatcher::compile_with_assertions_and_work(
            &repeated,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let Some(sequel) = RegExpSequenceMatcher::compile_with_assertions_and_work(
            &sequel,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        let Some(capture_count) = repeated
            .capture_count()
            .checked_add(sequel.capture_ranges().len())
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            repeated,
            sequel,
            capture_count,
        }))
    }

    /// Source-order captures across the repeated group and fixed sequel.
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }

    /// Absolute final-iteration, undefined or fixed-sequel capture range.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        if index >= self.capture_count {
            return None;
        }
        let end = matched.end.checked_sub(self.sequel.atom_count())?;
        if matched.start > end {
            return None;
        }
        if index < self.repeated.capture_count() {
            return self.repeated.capture_range(index, &(matched.start..end));
        }
        let relative = self
            .sequel
            .capture_ranges()
            .get(index - self.repeated.capture_count())?;
        Some(end.checked_add(relative.start)?..end.checked_add(relative.end)?)
    }

    /// Earliest start and greedy/lazy repetition length that satisfies the sequel.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.find_asserted(
            input,
            start,
            sticky,
            Assertions::default(),
            Assertions::default(),
            false,
        )
    }

    pub(crate) fn find_asserted(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        leading: Assertions,
        trailing: Assertions,
        multiline: bool,
    ) -> Option<Range<usize>> {
        let units = input.code_units();
        let repeated = self.repeated.find_asserted(
            input,
            start,
            sticky,
            |position| leading.accepts(units, position, multiline),
            |position| {
                self.sequel
                    .find(input, position, true)
                    .is_some_and(|range| trailing.accepts(units, range.end, multiline))
            },
        )?;
        Some(repeated.start..repeated.end.checked_add(self.sequel.atom_count())?)
    }

    /// Conservative passes for group candidates, endpoint selection and fixed sequels.
    pub fn search_passes(&self, sticky: bool) -> usize {
        self.repeated
            .search_passes(sticky)
            .saturating_add(2)
            .saturating_add(self.sequel.search_passes(false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn repeated_fixed_sequel_snapshot() {
        let mut rows = String::new();
        for source in [
            "(ab)+ab",
            "(ab)+?ab",
            "(ab)*(ab)",
            "(ab)*?(ab)",
            "(ab){1,2}(ab)",
            "(ab){1,2}?(ab)",
            "([ab][ab])+([ab]b)",
            "([ab][ab]){0,1}([ab]b)",
            "(([ab])([ab])()){2}([ab]b)()",
            "(a())+b",
            "(a())+($)(\\n)",
            r"(^a$\n){2}(a)",
            r"(a\B)+b",
            r"(\Ba)+b",
            r"(ab)+\b",
            r"(ab)+$",
            r"(ab)+^x",
            "(ab)+()",
            "(ab){0}([ab])",
            "(ab){999999999999999999999999999999}c",
            "()*(a)",
            "()+(a)",
            "(){999999999999999999999999999999}(a)",
            r"(^){2}(a)",
            r"(\b\B)*(a)",
            r"(\b\B)+(a)",
            "(?:ab)+(?:[ab])",
            "(.a)+([ab])",
            "([µ][µ])+(x)",
            r"(\uD83D\uDCA9)+(\uDCA9)",
            "(ab)+[.$]",
            "(ab)+c",
            "(ab)+",
            "^(ab)+c$",
            "x(ab)+c",
            "(ab|a)+c",
            "(ab)+(a)+",
            "(ab)+a+",
            "(a*)+b",
            "(?<n>ab)+c",
            r"(ab)+\1",
            "(ab)+(?=c)",
            "((ab)+)c",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (false, true, false),
                (true, true, false),
                (false, true, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(matcher) =
                    RegExpRepeatedContinuationMatcher::compile(&JsString::from(source), i, m, s)
                {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for text in [
                        "",
                        "a",
                        "b",
                        "ab",
                        "abab",
                        "ababab",
                        "abababa",
                        "xababab",
                        "aaaaab",
                        "aaaaaab",
                        "ABab",
                        "aaab",
                        "aaa\n",
                        "x\na\na\na\n",
                        "a\r\n",
                        "\na\nab",
                        "Μµx",
                        "💩💩",
                    ] {
                        let input = JsString::from(text);
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let result = matcher.find(&input, start, sticky).map(|range| {
                                let captures = (0..matcher.capture_count())
                                    .map(|slot| matcher.capture_range(slot, &range))
                                    .collect::<Vec<_>>();
                                (range, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{result:?}").unwrap();
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
    fn fixed_sequels_agree_with_independent_candidate_repetition_and_capture_order() {
        let alphabet = [97, 98, 10];
        for (quantifier, min, max) in [
            ("*", 0, None),
            ("+", 1, None),
            ("{0,2}", 0, Some(2)),
            ("{1,2}", 1, Some(2)),
            ("{2}", 2, Some(2)),
            ("{0}", 0, Some(0)),
        ] {
            for greedy in [false, true] {
                let source = format!(
                    "(([ab])([ab])()){quantifier}{}([ab]b)()",
                    if greedy { "" } else { "?" }
                );
                for anchored in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpRepeatedContinuationMatcher::compile(
                            &JsString::from(source.as_str()),
                            false,
                            multiline,
                            false,
                        )
                        .unwrap();
                        let outer = crate::RegExpAnchoredMatcher::compile(
                            &JsString::from(format!("^({source})$").as_str()),
                            false,
                            multiline,
                        )
                        .unwrap();
                        for length in 0..=6u32 {
                            for mut encoded in 0..alphabet.len().pow(length) {
                                let mut units = Vec::new();
                                for _ in 0..length {
                                    units.push(alphabet[encoded % alphabet.len()]);
                                    encoded /= alphabet.len();
                                }
                                let input = JsString::from_code_units(units.clone());
                                for start in 0..=units.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected =
                                            (start..=units.len()).find_map(|candidate| {
                                                if sticky && candidate != start
                                                    || anchored
                                                        && candidate != 0
                                                        && !(multiline
                                                            && [10, 13, 0x2028, 0x2029]
                                                                .contains(&units[candidate - 1]))
                                                {
                                                    return None;
                                                }
                                                let limit = max
                                                    .unwrap_or((units.len() - candidate) / 2)
                                                    .min((units.len() - candidate) / 2);
                                                let mut counts: Vec<_> = (min..=limit).collect();
                                                if greedy {
                                                    counts.reverse();
                                                }
                                                counts.into_iter().find_map(|count| {
                                                    let tail = candidate + count * 2;
                                                    let finish = tail + 2;
                                                    let suffix = units.get(tail..finish)?;
                                                    if !units[candidate..tail]
                                                        .iter()
                                                        .all(|u| [97, 98].contains(u))
                                                        || ![97, 98].contains(&suffix[0])
                                                        || suffix[1] != 98
                                                    {
                                                        return None;
                                                    }
                                                    if anchored
                                                        && finish != units.len()
                                                        && !(multiline
                                                            && [10, 13, 0x2028, 0x2029]
                                                                .contains(&units[finish]))
                                                    {
                                                        return None;
                                                    }
                                                    let mut captures = vec![
                                                        (count > 0).then(|| tail - 2..tail),
                                                        (count > 0).then(|| tail - 2..tail - 1),
                                                        (count > 0).then(|| tail - 1..tail),
                                                        (count > 0).then_some(tail..tail),
                                                        Some(tail..finish),
                                                        Some(finish..finish),
                                                    ];
                                                    if anchored {
                                                        captures.insert(0, Some(candidate..finish));
                                                    }
                                                    Some((candidate..finish, captures))
                                                })
                                            });
                                        let actual = if anchored {
                                            outer.find(&input, start, sticky).map(|range| {
                                                let captures = (0..outer.capture_count())
                                                    .map(|slot| outer.capture_range(slot, &range))
                                                    .collect::<Vec<_>>();
                                                (range, captures)
                                            })
                                        } else {
                                            matcher.find(&input, start, sticky).map(|range| {
                                                let captures = (0..matcher.capture_count())
                                                    .map(|slot| matcher.capture_range(slot, &range))
                                                    .collect::<Vec<_>>();
                                                (range, captures)
                                            })
                                        };
                                        assert_eq!(
                                            actual, expected,
                                            "{source} {units:?} {start} sticky={sticky} anchors={anchored} m={multiline}"
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

    #[test]
    fn long_fixed_sequel_runs_counts_copies_and_optional_construction_stay_checked() {
        let input = JsString::from(format!("{}b", "a".repeat(200_000)).as_str());
        let matcher = RegExpRepeatedContinuationMatcher::compile(
            &JsString::from("(aa)+(ab)"),
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(matcher.clone().find(&input, 0, false), Some(1..200_001));
        assert_eq!(matcher.find(&input, 0, true), None);
        assert_eq!(
            matcher.capture_range(0, &(1..200_001)),
            Some(199_997..199_999)
        );
        assert_eq!(
            matcher.capture_range(1, &(1..200_001)),
            Some(199_999..200_001)
        );
        assert_eq!(matcher.search_passes(true), 6);
        assert_eq!(matcher.search_passes(false), 7);
        assert_eq!(matcher.find(&input, usize::MAX, false), None);
        let source = format!("({}a{})+(b)", "(".repeat(100_000), ")".repeat(100_000));
        let deep = RegExpRepeatedContinuationMatcher::compile(
            &JsString::from(source.as_str()),
            false,
            false,
            false,
        )
        .unwrap();
        let matched = deep.find(&input, 0, false).unwrap();
        assert_eq!(deep.capture_count(), 100_002);
        assert_eq!(
            deep.capture_range(100_000, &matched),
            Some(199_999..200_000)
        );
        assert_eq!(
            deep.capture_range(100_001, &matched),
            Some(200_000..200_001)
        );
        assert_eq!(
            RegExpRepeatedContinuationMatcher::compile_with_work(
                &JsString::from(r"(ab)+(\d)"),
                false,
                false,
                false,
                |work| if work == 65536 { Err("host") } else { Ok(()) }
            )
            .unwrap_err(),
            "host"
        );
        assert!(
            RegExpRepeatedContinuationMatcher::compile_with_work(
                &JsString::from("([ab][ab])+(a)+"),
                false,
                false,
                false,
                |_| Err("unexpected")
            )
            .unwrap()
            .is_none()
        );
    }
}
