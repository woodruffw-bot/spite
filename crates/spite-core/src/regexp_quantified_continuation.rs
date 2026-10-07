//! A quantified consuming atom followed by a fixed literal continuation (22.2.2.3).

use crate::{JsString, RegExpLiteralMatcher, RegExpQuantifiedMatcher};
use std::{ops::Range, sync::Arc};

/// Immutable ordinary-mode repeated atom with a capture-free literal continuation.
///
/// The complete Pattern must already be validated without `u` or `v`. The prefix
/// uses one quantifier, with ordinary capturing/noncapturing wrappers. The
/// remainder compiles completely as literal characters and noncapturing groups.
/// Search is linear without allocation, expanded repetitions or native recursion.
#[derive(Clone, Debug)]
pub struct RegExpQuantifiedContinuationMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    prefix: RegExpQuantifiedMatcher,
    suffix: RegExpLiteralMatcher,
}

impl RegExpQuantifiedContinuationMatcher {
    /// Number of ordinary capturing wrappers around the repeated prefix.
    pub fn capture_count(&self) -> usize {
        self.0.prefix.capture_count()
    }

    /// Absolute capture range in a successful match from this plan.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        let end = matched.end.checked_sub(self.0.suffix.matched_len())?;
        self.0.prefix.capture_range(index, &(matched.start..end))
    }

    /// An empty continuation exposes all repetition endpoints to outer anchors.
    pub(crate) fn from_quantified(prefix: RegExpQuantifiedMatcher) -> Self {
        let suffix = RegExpLiteralMatcher::compile(&JsString::from(""), false)
            .expect("empty literal continuation");
        Self(Arc::new(Program { prefix, suffix }))
    }

    /// Compiles the complete repeated-prefix/fixed-continuation subset.
    pub fn compile(source: &JsString, ignore_case: bool, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Requires a complete literal continuation before constructing a prefix set.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(end) = RegExpQuantifiedMatcher::prefix_end(source, dot_all) else {
            return Ok(None);
        };
        if end == source.len() {
            return Ok(None);
        }
        let suffix_source = JsString::from_code_units(source.code_units()[end..].to_vec());
        let Some(suffix) = RegExpLiteralMatcher::compile(&suffix_source, ignore_case) else {
            return Ok(None);
        };
        if !suffix.capture_ranges().is_empty() {
            return Ok(None);
        }
        charge(source.len())?;
        charge(source.len())?;
        let prefix_source = JsString::from_code_units(source.code_units()[..end].to_vec());
        let prefix = RegExpQuantifiedMatcher::compile_with_work(
            &prefix_source,
            ignore_case,
            dot_all,
            charge,
        )?
        .expect("accepted complete quantified prefix");
        Ok(Some(Self(Arc::new(Program { prefix, suffix }))))
    }

    /// Selects the earliest start and its longest greedy or shortest lazy prefix.
    ///
    /// Literal continuation matches arrive in increasing position order. For
    /// each position, the earliest admissible prefix start is the maximum of
    /// `start`, the preceding atom-run start and the maximum-repetition boundary.
    /// All three are monotone, so a later continuation cannot produce an earlier
    /// whole match. Lazy search returns the first eligible position; greedy search
    /// retains the last position with the same earliest start. Prefix membership
    /// and the literal prefix-failure scan each visit the input only linearly.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.find_with(input, start, sticky, Some, |_| true)
    }

    /// Outer assertions constrain starts and endpoints before choosing repetition
    /// order (22.2.2.3.1, 22.2.2.4). The next permitted start is monotone, so its
    /// boundary cursor also visits the input only linearly.
    pub(crate) fn find_anchored(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        at_start: bool,
        at_end: bool,
        multiline: bool,
    ) -> Option<Range<usize>> {
        let units = input.code_units();
        let mut boundary = start;
        self.find_with(
            input,
            start,
            sticky,
            |candidate| {
                if !at_start {
                    return Some(candidate);
                }
                if !multiline {
                    return (candidate == 0).then_some(0);
                }
                boundary = boundary.max(candidate);
                while boundary <= units.len() {
                    if boundary == 0 || is_line_terminator(units[boundary - 1]) {
                        return Some(boundary);
                    }
                    if boundary == units.len() {
                        break;
                    }
                    boundary += 1;
                }
                None
            },
            |end| !at_end || end == units.len() || (multiline && is_line_terminator(units[end])),
        )
    }

    fn find_with(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        mut next_start: impl FnMut(usize) -> Option<usize>,
        mut accept_end: impl FnMut(usize) -> bool,
    ) -> Option<Range<usize>> {
        let units = input.code_units();
        units.get(start..)?;
        let (min, max, greedy) = self.0.prefix.bounds();
        let min = min?;
        if min > units.len() - start {
            return None;
        }
        let mut scanned = start;
        let mut run_start = start;
        let mut best: Option<Range<usize>> = None;
        self.0.suffix.find_if(input, start, false, |suffix| {
            for (offset, &unit) in units[scanned..suffix.start].iter().enumerate() {
                if !self.0.prefix.matches(unit) {
                    run_start = scanned + offset + 1;
                }
            }
            scanned = suffix.start;
            let lower = start
                .max(run_start)
                .max(max.map_or(start, |max| suffix.start.saturating_sub(max)));
            let Some(candidate) = next_start(lower) else {
                return true;
            };
            if sticky && candidate > start
                || best.as_ref().is_some_and(|best| candidate > best.start)
            {
                return true;
            }
            if candidate <= suffix.start
                && suffix.start - candidate >= min
                && accept_end(suffix.end)
            {
                best = Some(candidate..suffix.end);
                return !greedy;
            }
            false
        });
        best
    }
}

fn is_line_terminator(unit: u16) -> bool {
    matches!(unit, 0x0a | 0x0d | 0x2028 | 0x2029)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn quantified_literal_continuation_snapshot() {
        let mut rows = String::new();
        for source in [
            "a*a",
            "a*?a",
            "a+aa",
            "a+?aa",
            "a{0}a",
            "a{1,3}a",
            "a{1,3}?a",
            "[ab]*ab",
            "[ab]*?ab",
            "[ab]{2,3}b",
            "[ab]{2,3}?b",
            "[Nn]?evermore",
            "[]*x",
            "[]+x",
            "[^]*x",
            "[^]*?x",
            ".*x",
            ".*?x",
            r"\d+x",
            "(?:a)+?(?:aa)",
            "(?:a+)b",
            "a+(?:)",
            "a+",
            "a+b+",
            "a+(b)",
            "(?:a+b)",
            "a+|b",
            "^a+b$",
            "a+[b]",
        ] {
            for (i, s) in [(false, false), (true, false), (false, true)] {
                write!(rows, "{source:?} i={i} s={s}").unwrap();
                if let Some(m) = matcher(source, i, s) {
                    for input in [
                        "",
                        "a",
                        "aaaa",
                        "baaab",
                        "abab",
                        "ABab",
                        "evermore",
                        "Nevermore",
                        "xaxbxx",
                        "12x",
                        "a\nx",
                    ] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            m.find(&input, 0, false),
                            m.find(&input, 1, true)
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

    fn matcher(source: &str, i: bool, s: bool) -> Option<RegExpQuantifiedContinuationMatcher> {
        RegExpQuantifiedContinuationMatcher::compile(&JsString::from(source), i, s)
    }

    #[test]
    fn exhaustive_greedy_lazy_continuations_match_independent_candidate_oracle() {
        for (quant, min, max, greedy) in [
            ("*", 0, usize::MAX, true),
            ("*?", 0, usize::MAX, false),
            ("+", 1, usize::MAX, true),
            ("+?", 1, usize::MAX, false),
            ("{0}", 0, 0, true),
            ("{2}", 2, 2, true),
            ("{1,3}", 1, 3, true),
            ("{1,3}?", 1, 3, false),
        ] {
            for suffix in ["", "a", "aa", "ab", "b", "aba"] {
                let source = format!("[ab]{quant}(?:{suffix})");
                let m = matcher(&source, false, false).unwrap();
                let suffix: Vec<u16> = suffix.encode_utf16().collect();
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
                                        let mut lengths: Vec<_> =
                                            (min..=max.min(units.len() - candidate)).collect();
                                        if greedy {
                                            lengths.reverse();
                                        }
                                        lengths.into_iter().find_map(|count| {
                                            (units[candidate..candidate + count]
                                                .iter()
                                                .all(|u| matches!(u, 97 | 98))
                                                && units.get(
                                                    candidate + count
                                                        ..candidate + count + suffix.len(),
                                                ) == Some(suffix.as_slice()))
                                            .then_some(candidate..candidate + count + suffix.len())
                                        })
                                    });
                                assert_eq!(
                                    m.find(&input, start, sticky),
                                    expected,
                                    "{source} {input:?} {start} {sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn long_failed_prefixes_compact_bounds_and_construction_failures_are_safe() {
        let input = JsString::from_code_units(vec![97; 300_000]);
        let m = matcher("[a]*aaaaab", false, false).unwrap();
        assert_eq!(m.clone().find(&input, 0, false), None);
        let m = matcher("[a]*aaaaa", false, false).unwrap();
        assert_eq!(m.find(&input, 0, false), Some(0..300_000));
        let m = matcher("[a]*?aaaaa", false, false).unwrap();
        assert_eq!(m.find(&input, 0, true), Some(0..5));
        assert_eq!(
            matcher("a{0,999999999999999999999999999}a", false, false)
                .unwrap()
                .find(&input, 0, false),
            Some(0..300_000)
        );
        let rejected = RegExpQuantifiedContinuationMatcher::compile_with_work(
            &JsString::from(r"\d+(a)"),
            false,
            false,
            |_| Err("must not charge"),
        );
        assert!(matches!(rejected, Ok(None)));
        let abort = RegExpQuantifiedContinuationMatcher::compile_with_work(
            &JsString::from(r"\d+x"),
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
}
