//! A quantified consuming atom followed by a fixed continuation (22.2.2.3).

use crate::{JsString, RegExpLiteralMatcher, RegExpQuantifiedMatcher, RegExpSequenceMatcher};
use std::{ops::Range, sync::Arc};

/// Immutable ordinary-mode repeated atom with a fixed continuation.
///
/// The complete Pattern must already be validated without `u` or `v`. The prefix
/// uses one quantifier, with ordinary capturing/noncapturing wrappers. The
/// remainder uses fixed characters, sets, dots and ordinary capturing/noncapturing groups.
/// Literal continuations search linearly; fixed sets inspect at most their atom
/// count per input candidate, without allocation, expanded repetitions or recursion.
#[derive(Clone, Debug)]
pub struct RegExpQuantifiedContinuationMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    prefix: RegExpQuantifiedMatcher,
    suffix: Suffix,
    capture_count: usize,
}

#[derive(Debug)]
enum Suffix {
    Literal(RegExpLiteralMatcher),
    Sequence(RegExpSequenceMatcher),
}

impl Suffix {
    fn matched_len(&self) -> usize {
        match self {
            Self::Literal(m) => m.matched_len(),
            Self::Sequence(m) => m.atom_count(),
        }
    }
    fn capture_ranges(&self) -> &[Range<usize>] {
        match self {
            Self::Literal(m) => m.capture_ranges(),
            Self::Sequence(m) => m.capture_ranges(),
        }
    }
    fn find_if(
        &self,
        input: &JsString,
        start: usize,
        accept: impl FnMut(&Range<usize>) -> bool,
    ) -> Option<Range<usize>> {
        match self {
            Self::Literal(m) => m.find_if(input, start, false, accept),
            Self::Sequence(m) => m.find_if(input, start, false, accept),
        }
    }
}

impl RegExpQuantifiedContinuationMatcher {
    /// Number of ordinary captures in the repeated prefix and fixed continuation.
    pub fn capture_count(&self) -> usize {
        self.0.capture_count
    }

    /// Absolute capture range in a successful match from this plan.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        let end = matched.end.checked_sub(self.0.suffix.matched_len())?;
        if end < matched.start {
            return None;
        }
        let offset = self.0.prefix.capture_count();
        if index < offset {
            self.0.prefix.capture_range(index, &(matched.start..end))
        } else {
            let relative = self.0.suffix.capture_ranges().get(index - offset)?;
            Some(end.checked_add(relative.start)?..end.checked_add(relative.end)?)
        }
    }

    /// An empty continuation exposes all repetition endpoints to outer anchors.
    pub(crate) fn from_quantified(prefix: RegExpQuantifiedMatcher) -> Self {
        let suffix = RegExpLiteralMatcher::compile(&JsString::from(""), false)
            .expect("empty fixed continuation");
        let capture_count = prefix.capture_count();
        Self(Arc::new(Program {
            prefix,
            suffix: Suffix::Literal(suffix),
            capture_count,
        }))
    }

    /// Compiles the complete repeated-prefix/fixed-continuation subset.
    pub fn compile(source: &JsString, ignore_case: bool, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Requires a complete fixed continuation before constructing a prefix set.
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
        let suffix = if let Some(m) = RegExpLiteralMatcher::compile(&suffix_source, ignore_case) {
            Suffix::Literal(m)
        } else if let Some(m) = RegExpSequenceMatcher::compile_with_work(
            &suffix_source,
            ignore_case,
            dot_all,
            &mut charge,
        )? {
            Suffix::Sequence(m)
        } else {
            return Ok(None);
        };
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
        let Some(capture_count) = prefix
            .capture_count()
            .checked_add(suffix.capture_ranges().len())
        else {
            return Ok(None);
        };
        Ok(Some(Self(Arc::new(Program {
            prefix,
            suffix,
            capture_count,
        }))))
    }

    /// Conservative consuming passes for optional work accounting. Sticky
    /// repetition still scans continuation endpoints across the complete suffix.
    pub fn search_passes(&self) -> usize {
        match &self.0.suffix {
            Suffix::Literal(_) => 2,
            Suffix::Sequence(m) => m.atom_count().saturating_add(1),
        }
    }

    /// Selects the earliest start and its longest greedy or shortest lazy prefix.
    ///
    /// Fixed continuation matches arrive in increasing position order. For
    /// each position, the earliest admissible prefix start is the maximum of
    /// `start`, the preceding atom-run start and the maximum-repetition boundary.
    /// All three are monotone, so a later continuation cannot produce an earlier
    /// whole match. Lazy search returns the first eligible position; greedy search
    /// retains the last position with the same earliest start. Prefix membership
    /// visits the input linearly. Literal continuations share the prefix-failure
    /// scan; set/dot continuations check each fixed-width candidate once.
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

    pub(crate) fn find_with(
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
        self.0.suffix.find_if(input, start, |suffix| {
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
    fn captured_literal_continuation_snapshot() {
        let mut rows = String::new();
        for source in [
            "a+(b)",
            "(a)+(b)",
            "(a+)(b)",
            "((a)+)(b)",
            "([ab])*?(ab)",
            "([ab])*(ab)",
            "([ab]+?)(ab)",
            "a*()",
            "(a)*()",
            "(a*)()",
            "(a{0})(b)",
            "(a){0}(b)",
            "[]*(x)",
            "[^]*?(x)",
            "[^]*(x)",
            "a+((b)())",
            "a+((?:b))",
            "(?:a)+((b))",
            "a+(µ)",
            r"\d+(x)",
            r"a+(\uD800)",
            "(a+)(?:)()",
            "a+((?:))",
            "a+(b|c)",
            "a+([b])",
            "(a+b)",
            "a+(b+)",
            "a+(?<x>b)",
            r"a+(b)\1",
        ] {
            for (i, s) in [(false, false), (true, false), (false, true)] {
                write!(rows, "{source:?} i={i} s={s}").unwrap();
                if let Some(m) =
                    RegExpQuantifiedContinuationMatcher::compile(&JsString::from(source), i, s)
                {
                    write!(rows, " captures={}", m.capture_count()).unwrap();
                    for input in [
                        "",
                        "a",
                        "aaab",
                        "abab",
                        "ab",
                        "b",
                        "ABab",
                        "12x",
                        "a\nb",
                        "x",
                        "aaµ",
                        "\u{10000}",
                    ] {
                        let input = JsString::from(input);
                        for (start, sticky) in [(0, false), (1, true)] {
                            let range = m.find(&input, start, sticky);
                            let captures = range.as_ref().map(|r| {
                                (0..m.capture_count())
                                    .map(|c| m.capture_range(c, r))
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
    fn prefix_and_fixed_suffix_captures_agree_with_independent_repetition_order() {
        for (source, min, greedy) in [
            ("((a)*)((ab)())", 0, true),
            ("((a)*?)((ab)())", 0, false),
            ("((a)+)((ab)())", 1, true),
            ("((a)+?)((ab)())", 1, false),
        ] {
            let m =
                RegExpQuantifiedContinuationMatcher::compile(&JsString::from(source), false, false)
                    .unwrap();
            assert_eq!(m.capture_count(), 5);
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
                                    let counts: Vec<_> = if greedy {
                                        (min..=units.len() - candidate).rev().collect()
                                    } else {
                                        (min..=units.len() - candidate).collect()
                                    };
                                    counts.into_iter().find_map(|count| {
                                        let end = candidate + count;
                                        (units[candidate..end].iter().all(|u| *u == 97)
                                            && units.get(end..end + 2) == Some(&[97, 98]))
                                        .then_some((candidate..end + 2, count))
                                    })
                                });
                            assert_eq!(
                                m.find(&input, start, sticky),
                                expected.as_ref().map(|(r, _)| r.clone()),
                                "{source} {units:?} {start} {sticky}"
                            );
                            if let Some((range, count)) = expected {
                                let end = range.start + count;
                                let captures = [
                                    Some(range.start..end),
                                    (count > 0).then(|| end - 1..end),
                                    Some(end..end + 2),
                                    Some(end..end + 2),
                                    Some(end + 2..end + 2),
                                ];
                                for (index, expected) in captures.into_iter().enumerate() {
                                    assert_eq!(
                                        m.capture_range(index, &range),
                                        expected,
                                        "{source} capture={index}"
                                    );
                                }
                                assert_eq!(m.capture_range(5, &range), None);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn literal_suffix_capture_storage_clones_and_anchored_choices_preserve_layouts() {
        let source =
            JsString::from(format!("a+{}b{}", "(".repeat(100000), ")".repeat(100000)).as_str());
        let m = RegExpQuantifiedContinuationMatcher::compile(&source, false, false).unwrap();
        assert_eq!(m.capture_count(), 100000);
        let input = JsString::from("aaab");
        let range = m.find(&input, 0, false).unwrap();
        assert_eq!(m.capture_range(0, &range), Some(3..4));
        assert_eq!(m.clone().capture_range(99999, &range), Some(3..4));
        let anchored =
            crate::RegExpAnchoredMatcher::compile(&JsString::from("^((a)+)((b)())$"), false, false)
                .unwrap();
        assert_eq!(anchored.capture_count(), 5);
        let range = anchored.find(&input, 0, false).unwrap();
        assert_eq!(anchored.capture_range(4, &range), Some(4..4));
        let choices = crate::RegExpDisjunctionMatcher::compile(
            &JsString::from("([x])|((a)+)((b)())|([y])"),
            false,
        )
        .unwrap();
        assert_eq!(choices.capture_count(), 7);
        let (branch, range) = choices.find_branch(&input, 0, false).unwrap();
        assert_eq!(choices.capture_range(branch, 0, &range), None);
        assert_eq!(choices.capture_range(branch, 2, &range), Some(2..3));
        assert_eq!(choices.capture_range(branch, 3, &range), Some(3..4));
        assert_eq!(choices.capture_range(branch, 5, &range), Some(4..4));
        assert_eq!(choices.capture_range(branch, 6, &range), None);
        assert!(
            RegExpQuantifiedContinuationMatcher::compile_with_work(
                &JsString::from(r"\d+([x])+"),
                false,
                false,
                |_| -> Result<(), ()> { panic!("unsupported suffix before set preparation") }
            )
            .unwrap()
            .is_none()
        );
    }

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
            &JsString::from(r"\d+([a])+"),
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

    #[test]
    fn fixed_sequence_continuation_snapshot() {
        let mut rows = String::new();
        for source in [
            "a+[b]",
            "(a)+([b])",
            "(a+)(.)",
            "(a)*([ab])()",
            "(a*)[ab]",
            "[ab]+[ab]",
            "[ab]+?[ab]",
            "[ab]*([ab][ab])",
            "[ab]*?([ab][ab])",
            "[ab]{0,2}?([^x])",
            "[]*([ab])",
            "[]+([ab])",
            "[^]*?(.)",
            "[^]*(.)",
            "a+([])",
            "a+([^])",
            r"(a)+(\d)",
            r"(a)+(\s)",
            "(a)+([µ])",
            "(a)+([ſ])",
            "(a)+([💩])",
            "((a)+)(([b])())",
            "a+((?:.))",
            "(?:a)+([b])",
            "a+([b])c",
            "a+([b])+",
            "(a+[b])",
            "a+([b]|c)",
            "a+(?<n>[b])",
            r"a+([b])\1",
            "a+[b]$",
            "^a+[b]",
            "a+",
        ] {
            for (ignore_case, dot_all) in [(false, false), (true, false), (false, true)] {
                let m = RegExpQuantifiedContinuationMatcher::compile(
                    &JsString::from(source),
                    ignore_case,
                    dot_all,
                );
                write!(rows, "{source:?} i={ignore_case} s={dot_all}").unwrap();
                if let Some(m) = m {
                    write!(rows, " captures={}", m.capture_count()).unwrap();
                    for text in [
                        "", "a", "aa", "ab", "aaab", "abab", "baaa", "aaa1", "aa ", "aa\n", "aaΜ",
                        "aas", "aa💩", "xabb",
                    ] {
                        for (start, sticky) in [(0, false), (0, true), (1, true)] {
                            let r = m.find(&JsString::from(text), start, sticky).map(|r| {
                                let caps = (0..m.capture_count())
                                    .map(|i| m.capture_range(i, &r))
                                    .collect::<Vec<_>>();
                                (r, caps)
                            });
                            write!(rows, " {text:?}@{start}/{sticky}:{r:?}").unwrap();
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
    fn set_continuations_agree_with_independent_repetition_and_capture_order() {
        for (suffix, width, kind) in [
            ("[a]", 1, 0),
            ("[ab]", 1, 1),
            ("[ab][ab]", 2, 2),
            (".", 1, 3),
            ("[^a]", 1, 4),
            ("[]", 1, 5),
        ] {
            for (quantifier, min, max) in [
                ("*", 0, None),
                ("+", 1, None),
                ("{0,2}", 0, Some(2)),
                ("{2,3}", 2, Some(3)),
            ] {
                for greedy in [false, true] {
                    let source = format!(
                        "((([ab]){quantifier}{}))({suffix})()",
                        if greedy { "" } else { "?" }
                    );
                    let m = RegExpQuantifiedContinuationMatcher::compile(
                        &JsString::from(source.as_str()),
                        false,
                        false,
                    )
                    .unwrap();
                    assert_eq!(m.capture_count(), 5);
                    for len in 0..=4u32 {
                        for mut encoded in 0..4usize.pow(len) {
                            let mut units = Vec::new();
                            for _ in 0..len {
                                units.push([97, 98, 120, 10][encoded % 4]);
                                encoded /= 4;
                            }
                            let input = JsString::from_code_units(units.clone());
                            for start in 0..=units.len() + 1 {
                                for sticky in [false, true] {
                                    let expected = (start..=units.len())
                                        .take(if sticky { 1 } else { usize::MAX })
                                        .find_map(|candidate| {
                                            let limit = max
                                                .unwrap_or(units.len() - candidate)
                                                .min(units.len() - candidate);
                                            let counts: Vec<_> = if greedy {
                                                (min..=limit).rev().collect()
                                            } else {
                                                (min..=limit).collect()
                                            };
                                            counts.into_iter().find_map(|count| {
                                                let end = candidate + count;
                                                let finish = end + width;
                                                if !units[candidate..end]
                                                    .iter()
                                                    .all(|u| [97, 98].contains(u))
                                                {
                                                    return None;
                                                }
                                                let p = units.get(end..finish)?;
                                                let accepts = match kind {
                                                    0 => p[0] == 97,
                                                    1 => [97, 98].contains(&p[0]),
                                                    2 => p.iter().all(|u| [97, 98].contains(u)),
                                                    3 => p[0] != 10,
                                                    4 => p[0] != 97,
                                                    _ => false,
                                                };
                                                if !accepts {
                                                    return None;
                                                }
                                                Some((
                                                    candidate..finish,
                                                    vec![
                                                        Some(candidate..end),
                                                        Some(candidate..end),
                                                        (count > 0).then(|| end - 1..end),
                                                        Some(end..finish),
                                                        Some(finish..finish),
                                                    ],
                                                ))
                                            })
                                        });
                                    let actual = m.find(&input, start, sticky).map(|r| {
                                        let caps = (0..m.capture_count())
                                            .map(|i| m.capture_range(i, &r))
                                            .collect::<Vec<_>>();
                                        (r, caps)
                                    });
                                    assert_eq!(
                                        actual, expected,
                                        "{source} {units:?} {start} {sticky}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn large_shared_continuations_keep_static_captures_and_conservative_work_bounds() {
        let source = JsString::from(format!("(b)+{}", "([a])".repeat(1000)).as_str());
        let mut remaining = 100_000usize;
        let m =
            RegExpQuantifiedContinuationMatcher::compile_with_work(&source, false, false, |work| {
                remaining = remaining.checked_sub(work).ok_or("abort")?;
                Ok::<(), &str>(())
            })
            .unwrap()
            .unwrap();
        let copy = m.clone();
        assert_eq!(copy.capture_count(), 1001);
        assert_eq!(copy.search_passes(), 1001);
        let input = JsString::from(format!("bbb{}", "a".repeat(1000)).as_str());
        let r = copy.find(&input, 0, true).unwrap();
        assert_eq!(r, 0..1003);
        assert_eq!(copy.capture_range(0, &r), Some(2..3));
        assert_eq!(copy.capture_range(1, &r), Some(3..4));
        assert_eq!(copy.capture_range(1000, &r), Some(1002..1003));
        assert_eq!(copy.capture_range(1001, &r), None);
        assert_eq!(
            copy.find(
                &JsString::from(format!("bbb{}x", "a".repeat(999)).as_str()),
                0,
                true
            ),
            None
        );
        let literal =
            RegExpQuantifiedContinuationMatcher::compile(&JsString::from("a+b"), false, false)
                .unwrap();
        assert_eq!(literal.search_passes(), 2);
        assert_eq!(
            RegExpQuantifiedContinuationMatcher::compile_with_work(
                &JsString::from("a+[b]"),
                false,
                false,
                |work| if work == 1024 { Err("abort") } else { Ok(()) }
            )
            .unwrap_err(),
            "abort"
        );
    }
}
