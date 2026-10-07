//! Fixed prefixes around repeated fixed groups (22.2.2.3.1–3).

use crate::regexp_assertion::Assertions;
use crate::{
    JsString, RegExpRepeatedContinuationMatcher, RegExpRepeatedSequenceMatcher,
    RegExpSequenceMatcher,
};
use std::ops::Range;

/// An ordinary fixed prefix, one repeated fixed group and an optional fixed sequel.
///
/// Patterns must already be validated without `u` or `v`. Every fixed component
/// supports ordinary captures and word/input/line assertions with explicit flags.
/// Matching retains the fixed candidate bound without expanding counts or recursion.
#[derive(Clone, Debug)]
pub struct RegExpRepeatedPrefixedMatcher {
    prefix: RegExpSequenceMatcher,
    body: Body,
    capture_count: usize,
}

#[derive(Clone, Debug)]
enum Body {
    Repeated(RegExpRepeatedSequenceMatcher),
    Continuation(RegExpRepeatedContinuationMatcher),
}

impl Body {
    fn capture_count(&self) -> usize {
        match self {
            Self::Repeated(m) => m.capture_count(),
            Self::Continuation(m) => m.capture_count(),
        }
    }
    fn capture_range(&self, index: usize, range: &Range<usize>) -> Option<Range<usize>> {
        match self {
            Self::Repeated(m) => m.capture_range(index, range),
            Self::Continuation(m) => m.capture_range(index, range),
        }
    }
    fn find_filtered(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        accept_start: impl FnMut(usize) -> bool,
        accept_end: impl FnMut(usize) -> bool,
    ) -> Option<Range<usize>> {
        match self {
            Self::Repeated(m) => m.find_asserted(input, start, sticky, accept_start, accept_end),
            Self::Continuation(m) => {
                m.find_filtered(input, start, sticky, accept_start, accept_end)
            }
        }
    }
    fn search_passes(&self, sticky: bool) -> usize {
        match self {
            Self::Repeated(m) => m.search_passes(sticky).saturating_add(2),
            Self::Continuation(m) => m.search_passes(sticky),
        }
    }
}

impl RegExpRepeatedPrefixedMatcher {
    /// Compiles a complete fixed prefix/group/quantifier/sequel layout.
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

    /// Rejects unsupported component syntax before constructing the prefix's sets.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let Some(split) = repeated_start(units) else {
            return Ok(None);
        };
        let prefix = JsString::from_code_units(units[..split].to_vec());
        if RegExpSequenceMatcher::repeated_atom_width(&prefix, dot_all, true).is_none() {
            return Ok(None);
        }
        let body = JsString::from_code_units(units[split..].to_vec());
        let body = if let Some(m) = RegExpRepeatedSequenceMatcher::compile_with_assertions_and_work(
            &body,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )? {
            Body::Repeated(m)
        } else if let Some(m) = RegExpRepeatedContinuationMatcher::compile_with_work(
            &body,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )? {
            Body::Continuation(m)
        } else {
            return Ok(None);
        };
        charge(units.len())?;
        let Some(prefix) = RegExpSequenceMatcher::compile_with_assertions_and_work(
            &prefix,
            ignore_case,
            multiline,
            dot_all,
            charge,
        )?
        else {
            return Ok(None);
        };
        let Some(capture_count) = prefix
            .capture_ranges()
            .len()
            .checked_add(body.capture_count())
        else {
            return Ok(None);
        };
        Ok(Some(Self {
            prefix,
            body,
            capture_count,
        }))
    }

    /// Source-order captures across the prefix, repeated group and sequel.
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }

    /// Absolute fixed-prefix, final-iteration, undefined or sequel capture range.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        if index >= self.capture_count {
            return None;
        }
        let body_start = matched.start.checked_add(self.prefix.atom_count())?;
        if body_start > matched.end {
            return None;
        }
        if index < self.prefix.capture_ranges().len() {
            let relative = self.prefix.capture_ranges().get(index)?;
            Some(
                matched.start.checked_add(relative.start)?
                    ..matched.start.checked_add(relative.end)?,
            )
        } else {
            self.body.capture_range(
                index - self.prefix.capture_ranges().len(),
                &(body_start..matched.end),
            )
        }
    }

    /// Earliest complete prefix start with greedy/lazy asserted repetition endpoints.
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
        let body_start = start.checked_add(self.prefix.atom_count())?;
        let matched = self.body.find_filtered(
            input,
            body_start,
            sticky,
            |position| {
                let Some(prefix_start) = position.checked_sub(self.prefix.atom_count()) else {
                    return false;
                };
                leading.accepts(units, prefix_start, multiline)
                    && self.prefix.find(input, prefix_start, true).is_some()
            },
            |position| trailing.accepts(units, position, multiline),
        )?;
        Some(matched.start.checked_sub(self.prefix.atom_count())?..matched.end)
    }

    /// Conservative passes for fixed-prefix checks and complete repeated-body search.
    pub fn search_passes(&self, sticky: bool) -> usize {
        self.body
            .search_passes(sticky)
            .saturating_add(self.prefix.search_passes(false).saturating_mul(2))
    }
}

fn repeated_start(units: &[u16]) -> Option<usize> {
    let mut index = 0;
    while let Some(&unit) = units.get(index) {
        match unit {
            92 => index = index.checked_add(2)?,
            91 => {
                index += 1;
                while let Some(&unit) = units.get(index) {
                    index += 1;
                    if unit == 92 {
                        index = index.checked_add(1)?;
                    } else if unit == 93 {
                        break;
                    }
                }
            }
            40 => {
                let end = index
                    .checked_add(crate::regexp_repeated_literal::group_end(&units[index..])?)?;
                if crate::regexp_quantified::quantifier(&units[end..]).is_some() {
                    return (index > 0).then_some(index);
                }
                index = end;
            }
            _ => index += 1,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn fixed_prefix_repeated_groups_snapshot() {
        let mut rows = String::new();
        for source in [
            "x(ab)+",
            "x(ab)+c",
            "x(ab)+?ab",
            "x(ab)*c",
            "x(ab)*?c",
            "x(ab){1,2}c",
            "(a)(aa)+(ab)",
            "(a)(aa){1,2}(ab)",
            "([ab])([ab][ab])+([ab]b)",
            "([ab])(([ab])([ab])()){2}([ab]b)()",
            "()(ab)+",
            "a()*(b)",
            "a()+(b)",
            "a(){999999999999999999999999999999}(b)",
            r"(\b)(a)(a\B)+b",
            r"(^)(a)(a())+($)(\n)",
            r"(^)(a)(^a$\n)+b",
            r"a(^){2}b",
            r"a(\b\B)*b",
            r"a(\b\B)+b",
            r"\b(a)(aa)*\b",
            r"\b(a)(aa)*?\b",
            r"^x(ab)+c$",
            r"\((ab)+c",
            r"\x28(ab)+c",
            r"[(](ab)+c",
            r"[()](ab)+[.$]",
            "(µ)([µ][µ])+(x)",
            r"\uDCA9(ab)+(\uDCA9)",
            "💩(ab)+(c)",
            "(.)(.a)+([ab])",
            "x(?:ab)+(?:c)",
            "x(ab)+(ab)()",
            "x(ab)+()",
            "x(ab)+$",
            "x(ab)+^c",
            "(x(ab)+c)",
            "(x)(ab)+c",
            "a+b(ab)+c",
            "x(ab|a)+c",
            "x(ab)+(c)+",
            "x(?<n>ab)+c",
            r"x(ab)+\1",
            "x((ab)+)c",
            "(ab)+c",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (false, true, false),
                (true, true, false),
                (false, true, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(matcher) =
                    RegExpRepeatedPrefixedMatcher::compile(&JsString::from(source), i, m, s)
                {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for text in [
                        "",
                        "ab",
                        "abc",
                        "xabc",
                        "xababc",
                        "xabababc",
                        "xababxc",
                        "aaab",
                        "aaaaab",
                        "aaaaaab",
                        "XABabc",
                        "aaa\n",
                        "x\naaaa\n",
                        "a\na\nb",
                        "(ababc",
                        "abab.",
                        "µΜµx",
                        "💩ababc",
                        "\n\naab",
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
    fn fixed_prefix_candidates_and_captures_agree_with_independent_count_order() {
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
                    "([ab])(([ab])([ab])()){quantifier}{}([ab]b)()",
                    if greedy { "" } else { "?" }
                );
                for anchored in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpRepeatedPrefixedMatcher::compile(
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
                        for length in 0..=7u32 {
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
                                                if ![97, 98].contains(units.get(candidate)?) {
                                                    return None;
                                                }
                                                let body = candidate + 1;
                                                let limit = max
                                                    .unwrap_or((units.len() - body) / 2)
                                                    .min((units.len() - body) / 2);
                                                let mut counts: Vec<_> = (min..=limit).collect();
                                                if greedy {
                                                    counts.reverse();
                                                }
                                                counts.into_iter().find_map(|count| {
                                                    let tail = body + count * 2;
                                                    let finish = tail + 2;
                                                    let suffix = units.get(tail..finish)?;
                                                    if !units[body..tail]
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
                                                        Some(candidate..body),
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
    fn long_prefix_runs_deep_captures_clones_and_optional_work_keep_fixed_bounds() {
        let input = JsString::from(format!("{}b", "a".repeat(200_000)).as_str());
        let matcher = RegExpRepeatedPrefixedMatcher::compile(
            &JsString::from("(a)(aa)+(ab)"),
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(matcher.clone().find(&input, 0, false), Some(0..200_001));
        assert_eq!(matcher.find(&input, 0, true), Some(0..200_001));
        assert_eq!(matcher.capture_range(0, &(0..200_001)), Some(0..1));
        assert_eq!(
            matcher.capture_range(1, &(0..200_001)),
            Some(199_997..199_999)
        );
        assert_eq!(
            matcher.capture_range(2, &(0..200_001)),
            Some(199_999..200_001)
        );
        assert_eq!(matcher.search_passes(true), 8);
        assert_eq!(matcher.search_passes(false), 9);
        assert_eq!(matcher.find(&input, usize::MAX, false), None);
        let source = format!("{}a{}(aa)+(ab)", "(".repeat(100_000), ")".repeat(100_000));
        let deep = RegExpRepeatedPrefixedMatcher::compile(
            &JsString::from(source.as_str()),
            false,
            false,
            false,
        )
        .unwrap();
        let matched = deep.find(&input, 0, false).unwrap();
        assert_eq!(deep.capture_count(), 100_002);
        assert_eq!(deep.capture_range(99_999, &matched), Some(0..1));
        assert_eq!(
            deep.capture_range(100_000, &matched),
            Some(199_997..199_999)
        );
        assert_eq!(
            deep.capture_range(100_001, &matched),
            Some(199_999..200_001)
        );
        assert_eq!(
            RegExpRepeatedPrefixedMatcher::compile_with_work(
                &JsString::from(r"(\d)(ab)+(c)"),
                false,
                false,
                false,
                |work| if work == 65536 { Err("host") } else { Ok(()) }
            )
            .unwrap_err(),
            "host"
        );
        assert!(
            RegExpRepeatedPrefixedMatcher::compile_with_work(
                &JsString::from("[ab](ab|a)+(c)"),
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
