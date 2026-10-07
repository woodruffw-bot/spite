//! Ordinary consuming sequences with outer boundary assertions (22.2.2.4).

use crate::regexp_assertion::{Assertions, split_outer_assertions};
use crate::{
    JsString, RegExpLiteralMatcher, RegExpPrefixedMatcher, RegExpQuantifiedContinuationMatcher,
    RegExpQuantifiedMatcher, RegExpSequenceMatcher, regexp_outer_group_body,
};
use std::ops::Range;

/// A supported ordinary sequence with outer `^`, `$`, `\b` or `\B` assertions.
///
/// Patterns must already be validated without `u` or `v`. Input/line assertions
/// embedded within consuming bodies remain unsupported. Capture ranges retain the body's
/// relative UTF-16 offsets; quantified capture ranges depend on the match.
/// Compilation is iterative; literal and quantified search remain linear.
/// Fixed class bodies retain the sequence candidate search bound.
#[derive(Clone, Debug)]
pub struct RegExpAnchoredMatcher {
    body: Body,
    leading: Assertions,
    trailing: Assertions,
    multiline: bool,
    enclosing_captures: usize,
    capture_count: usize,
}

#[derive(Clone, Debug)]
enum Body {
    Literal(RegExpLiteralMatcher),
    Sequence(RegExpSequenceMatcher),
    Quantified(RegExpQuantifiedContinuationMatcher),
    Prefixed(RegExpPrefixedMatcher),
}

impl RegExpAnchoredMatcher {
    /// Number of ordinary captures in the complete consuming body.
    pub fn capture_count(&self) -> usize {
        self.capture_count
    }

    /// Absolute capture range in a successful match from this plan.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        if index >= self.capture_count || matched.start > matched.end {
            return None;
        }
        if index < self.enclosing_captures {
            return Some(matched.clone());
        }
        let index = index - self.enclosing_captures;
        if let Body::Quantified(matcher) = &self.body {
            return matcher.capture_range(index, matched);
        }
        if let Body::Prefixed(matcher) = &self.body {
            return matcher.capture_range(index, matched);
        }
        let relative = self.body.capture_ranges().get(index)?;
        Some(matched.start.checked_add(relative.start)?..matched.start.checked_add(relative.end)?)
    }

    /// Compiles a sequence with outer assertions and DotAll disabled, preserving escapes.
    pub fn compile(source: &JsString, ignore_case: bool, multiline: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, multiline, false, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles with explicit DotAll and optional set construction accounting.
    ///
    /// Bodies reuse their complete existing consuming plans. Unsupported syntax
    /// and host charge failures remain distinct. Callers account for the outer
    /// source scan/copy separately.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let Some((body_range, leading, trailing)) = split_outer_assertions(units) else {
            return Ok(None);
        };
        let units = &units[body_range];
        let body = JsString::from_code_units(units.to_vec());
        // Preserve fixed relative layouts before removing complete wrappers.
        let (body, enclosing_captures) = if let Some(matcher) =
            compile_body(&body, ignore_case, dot_all, &mut charge)?
        {
            (matcher, 0)
        } else if body.code_units().first() == Some(&40) {
            charge(body.len())?;
            charge(body.len())?;
            charge(body.len())?;
            let Some(group) = regexp_outer_group_body(&body) else {
                return Ok(None);
            };
            charge(group.body.len())?;
            let source = JsString::from_code_units(body.code_units()[group.body].to_vec());
            charge(source.len())?;
            charge(source.len())?;
            let Some(matcher) = compile_body(&source, ignore_case, dot_all, &mut charge)? else {
                return Ok(None);
            };
            (matcher, group.captures)
        } else {
            return Ok(None);
        };
        let Some(capture_count) = enclosing_captures.checked_add(body.capture_count()) else {
            return Ok(None);
        };
        Ok(Some(Self {
            body,
            leading,
            trailing,
            multiline,
            enclosing_captures,
            capture_count,
        }))
    }

    /// Fixed relative captures; quantified or enclosing captures return an empty slice.
    /// Use `capture_count` and `capture_range` to resolve every supported body.
    pub fn capture_ranges(&self) -> &[Range<usize>] {
        if self.enclosing_captures == 0 {
            self.body.capture_ranges()
        } else {
            &[]
        }
    }

    /// Conservative optional passes covering consuming terms and outer boundaries.
    pub fn search_passes(&self, sticky: bool) -> usize {
        let consuming = match &self.body {
            Body::Literal(_) => 2,
            Body::Quantified(m) => m.search_passes().saturating_add(1),
            Body::Prefixed(m) => m.search_passes(sticky).saturating_add(1),
            Body::Sequence(matcher) => matcher.search_passes(sticky).saturating_add(1),
        };
        consuming.saturating_add(
            2 * (usize::from(self.leading.has_word_boundary())
                + usize::from(self.trailing.has_word_boundary())),
        )
    }

    /// Repetition may inspect the entire remaining input even at one sticky start.
    pub fn requires_full_suffix(&self) -> bool {
        matches!(self.body, Body::Quantified(_) | Body::Prefixed(_))
    }

    /// Finds the earliest match whose outer assertions all succeed.
    ///
    /// Sticky matching never reinterprets `^` as the requested start. Only the
    /// complete input's beginning, or a preceding LineTerminator in multiline
    /// mode, satisfies it. `$` without multiline requires the input's exact end.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let units = input.code_units();
        let accept = |range: &Range<usize>| {
            self.leading.accepts(units, range.start, self.multiline)
                && self.trailing.accepts(units, range.end, self.multiline)
        };
        match &self.body {
            Body::Literal(matcher) => matcher.find_if(input, start, sticky, accept),
            Body::Sequence(matcher) => matcher.find_if(input, start, sticky, accept),
            Body::Prefixed(matcher) => matcher.find_asserted(
                input,
                start,
                sticky,
                self.leading,
                self.trailing,
                self.multiline,
            ),
            Body::Quantified(matcher) => matcher.find_asserted(
                input,
                start,
                sticky,
                self.leading,
                self.trailing,
                self.multiline,
            ),
        }
    }
}

impl Body {
    fn capture_count(&self) -> usize {
        match self {
            Self::Quantified(m) => m.capture_count(),
            Self::Prefixed(m) => m.capture_count(),
            _ => self.capture_ranges().len(),
        }
    }
    fn capture_ranges(&self) -> &[Range<usize>] {
        match self {
            Self::Literal(m) => m.capture_ranges(),
            Self::Sequence(m) => m.capture_ranges(),
            Self::Quantified(_) | Self::Prefixed(_) => &[],
        }
    }
}

fn compile_body<E>(
    source: &JsString,
    ignore_case: bool,
    dot_all: bool,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<Body>, E> {
    let body = if let Some(m) = RegExpLiteralMatcher::compile(source, ignore_case) {
        Body::Literal(m)
    } else if let Some(m) =
        RegExpSequenceMatcher::compile_with_work(source, ignore_case, dot_all, &mut *charge)?
    {
        Body::Sequence(m)
    } else if let Some(m) =
        RegExpQuantifiedMatcher::compile_with_work(source, ignore_case, dot_all, &mut *charge)?
    {
        Body::Quantified(RegExpQuantifiedContinuationMatcher::from_quantified(m))
    } else if let Some(m) = RegExpQuantifiedContinuationMatcher::compile_with_work(
        source,
        ignore_case,
        dot_all,
        &mut *charge,
    )? {
        Body::Quantified(m)
    } else if let Some(m) =
        RegExpPrefixedMatcher::compile_with_work(source, ignore_case, dot_all, &mut *charge)?
    {
        Body::Prefixed(m)
    } else {
        return Ok(None);
    };
    Ok(Some(body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn word_boundary_snapshot() {
        let mut rows = String::new();
        for source in [
            r"\b",
            r"\B",
            r"\b\b",
            r"\b\B",
            r"^$\B",
            r"^^a$$",
            r"$a^",
            r"\bfoo\b",
            r"\Bfoo\B",
            r"\ba*\b",
            r"\ba*?\b",
            r"\Ba+",
            r"a+\b",
            r"a+\B",
            r"\ba{1,3}\b",
            r"\b([ab])+([ab])\b",
            r"\B([x])([ab])*?([ab])\b",
            r"\b(a*)()\b",
            r"\b(a)*()\b",
            r"\b([a-z][a-z])\b",
            r"\b.\b",
            r"\B[^]\B",
            r"^\b(a+)\b$",
            r"\b((?:a)+)\b",
            r"\b\\b\B",
            r"\B[\b]\B",
            r"^\x24\B",
            r"\bµ\b",
            r"\ba\b",
            r"\bſ\b",
            r"\bK\b",
            r"\B\uD800\B",
            r"a\bb",
            r"(\ba\b)",
            r"\b(ab)+\b",
            r"\ba+b+\b",
            r"\b(?<n>a)\b",
            r"\ba|b\b",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
            ] {
                let matcher = RegExpAnchoredMatcher::compile_with_work(
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
                        " captures={} passes={}/{} full_suffix={}",
                        matcher.capture_count(),
                        matcher.search_passes(false),
                        matcher.search_passes(true),
                        matcher.requires_full_suffix()
                    )
                    .unwrap();
                    for input in [
                        "",
                        "a",
                        "A",
                        "aaa",
                        "aaab",
                        "xaaab",
                        " foo ",
                        "xfooy",
                        "ab",
                        "aa",
                        "xaaa",
                        "a\nb",
                        "\r\n",
                        "x\naaa\ny",
                        "µa",
                        "ſa",
                        "Ka",
                        "💩a",
                        "\\b",
                        "\u{0008}",
                    ] {
                        let input = JsString::from(input);
                        let matches: Vec<_> = [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ]
                        .into_iter()
                        .map(|(start, sticky)| {
                            matcher.find(&input, start, sticky).map(|r| {
                                let captures: Vec<_> = (0..matcher.capture_count())
                                    .map(|n| matcher.capture_range(n, &r))
                                    .collect();
                                (r, captures)
                            })
                        })
                        .collect();
                        write!(rows, " {input:?}:{matches:?}").unwrap();
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
    fn word_assertions_select_repetition_endpoints_against_an_independent_oracle() {
        fn boundary(units: &[u16], p: usize) -> bool {
            fn word(c: u16) -> bool {
                (b'A' as u16..=b'Z' as u16).contains(&c)
                    || (b'a' as u16..=b'z' as u16).contains(&c)
                    || (b'0' as u16..=b'9' as u16).contains(&c)
                    || c == b'_' as u16
            }
            let before = p > 0 && word(units[p - 1]);
            let after = p < units.len() && word(units[p]);
            before != after
        }
        for leading in [r"\b", r"\B"] {
            for trailing in [r"\b", r"\B"] {
                for (quantifier, minimum, maximum) in [
                    ("*", 0, usize::MAX),
                    ("+", 1, usize::MAX),
                    ("{0,2}", 0, 2),
                    ("{2,3}", 2, 3),
                ] {
                    for lazy in [false, true] {
                        let source = format!(
                            "{leading}((a){quantifier}{})(){trailing}",
                            if lazy { "?" } else { "" }
                        );
                        let matcher = RegExpAnchoredMatcher::compile(
                            &JsString::from(source.as_str()),
                            false,
                            false,
                        )
                        .unwrap();
                        for length in 0..=5usize {
                            for encoded in 0..4usize.pow(length as u32) {
                                let mut n = encoded;
                                let units: Vec<_> = (0..length)
                                    .map(|_| {
                                        let c = [97, 98, 32, 0xd800][n % 4];
                                        n /= 4;
                                        c
                                    })
                                    .collect();
                                let input = JsString::from_code_units(units.clone());
                                for start in 0..=length + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=length)
                                            .filter(|&p| !sticky || p == start)
                                            .filter(|&p| boundary(&units, p) == (leading == r"\b"))
                                            .find_map(|p| {
                                                let run = units[p..]
                                                    .iter()
                                                    .take_while(|&&c| c == 97)
                                                    .count()
                                                    .min(maximum);
                                                let endpoints: Vec<_> = (minimum..=run)
                                                    .filter(|&count| {
                                                        boundary(&units, p + count)
                                                            == (trailing == r"\b")
                                                    })
                                                    .collect();
                                                let count = if lazy {
                                                    endpoints.first()
                                                } else {
                                                    endpoints.last()
                                                }?;
                                                Some(p..p + count)
                                            });
                                        let actual = matcher.find(&input, start, sticky);
                                        assert_eq!(
                                            actual, expected,
                                            "{source} {units:?} {start} {sticky}"
                                        );
                                        if let Some(r) = actual {
                                            assert_eq!(
                                                matcher.capture_range(0, &r),
                                                Some(r.clone())
                                            );
                                            assert_eq!(
                                                matcher.capture_range(1, &r),
                                                (r.start != r.end).then(|| r.end - 1..r.end)
                                            );
                                            assert_eq!(
                                                matcher.capture_range(2, &r),
                                                Some(r.end..r.end)
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
    }

    #[test]
    fn large_flat_assertion_sequences_and_surrogate_positions_need_no_limits() {
        let source = JsString::from(
            format!("{}(a+){}", r"\b".repeat(100_000), r"\b".repeat(100_000)).as_str(),
        );
        let m = RegExpAnchoredMatcher::compile(&source, false, false)
            .unwrap()
            .clone();
        let input = JsString::from(format!(" {} ", "a".repeat(100_000)).as_str());
        let matched = m.find(&input, 0, false).unwrap();
        assert_eq!(matched, 1..100_001);
        assert_eq!(m.capture_range(0, &matched), Some(matched.clone()));
        assert_eq!(m.search_passes(false), 7);
        assert_eq!(m.find(&input, 2, true), None);
        let input = JsString::from_code_units(vec![0xd800, 0xdc00, 97]);
        let word = RegExpAnchoredMatcher::compile(&JsString::from(r"\b"), true, false).unwrap();
        let nonword = RegExpAnchoredMatcher::compile(&JsString::from(r"\B"), true, false).unwrap();
        assert_eq!(word.find(&input, 0, false), Some(2..2));
        assert_eq!(nonword.find(&input, 1, true), Some(1..1));
        assert_eq!(word.find(&input, usize::MAX, false), None);
    }

    #[test]
    fn quantified_anchor_snapshot() {
        let mut rows = String::new();
        for source in [
            "^a*$",
            "^a*?$",
            "a+$",
            "a+?$",
            "^a+",
            "^a+?",
            "^a{1,3}$",
            "a{1,3}$",
            "a{1,3}?$",
            "^a{0}$",
            "^[ab]*$",
            "^[ab]*?$",
            "^[^]*$",
            "^[^]*?$",
            "^.*$",
            "^.*?$",
            "^.+$",
            "^[ab]*ab$",
            "^[ab]*?ab$",
            "[ab]*ab$",
            "[ab]*?ab$",
            "^[Nn]?evermore$",
            "^(?:a)+$",
            "^(?:a+)$",
            "^[]*$",
            "^[]+$",
            r"^\d+x$",
            r"^a*\$$",
            r"^a*\\$",
            "^a{999999999999999999999999999}$",
            "^a+[b]$",
            "^(a)+$",
            "^(a+)$",
            "^a+b+$",
            "^(?:a+b)$",
            "^^a*$",
            "a*^",
            "^a*|b$",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
            ] {
                let matcher = RegExpAnchoredMatcher::compile_with_work(
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
                        " captures={:?} full_suffix={} passes={}/{}",
                        matcher.capture_ranges(),
                        matcher.requires_full_suffix(),
                        matcher.search_passes(false),
                        matcher.search_passes(true)
                    )
                    .unwrap();
                    if matcher.capture_count() != matcher.capture_ranges().len() {
                        write!(rows, " dynamic_captures={}", matcher.capture_count()).unwrap();
                    }
                    for input in [
                        "",
                        "a",
                        "A",
                        "aaaa",
                        "baaa",
                        "abab",
                        "a\nb",
                        "a\r\nb",
                        "\r\n",
                        "x\naaa\ry",
                        "aa\u{2028}a",
                        "\u{2029}",
                        "aax",
                        "12x",
                        "Nevermore",
                        "evermore",
                        "aaa$",
                        "aaa\\",
                    ] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            matcher.find(&input, 0, false),
                            matcher.find(&input, 1, true)
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
    fn quantified_boundaries_agree_with_independent_candidate_repetition_order() {
        let alphabet = [97, 98, 10, 13, 0xd800];
        for (quantifier, min, max) in [
            ("*", 0, None),
            ("+", 1, None),
            ("{0,2}", 0, Some(2)),
            ("{2,3}", 2, Some(3)),
            ("{0}", 0, Some(0)),
        ] {
            for suffix in ["", "a", "ab"] {
                for (begin, end) in [(true, false), (false, true), (true, true)] {
                    for greedy in [false, true] {
                        let source = format!(
                            "{}[ab]{quantifier}{}{suffix}{}",
                            if begin { "^" } else { "" },
                            if greedy { "" } else { "?" },
                            if end { "$" } else { "" }
                        );
                        for multiline in [false, true] {
                            let matcher = RegExpAnchoredMatcher::compile(
                                &JsString::from(source.as_str()),
                                false,
                                multiline,
                            )
                            .unwrap();
                            for length in 0..=4u32 {
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
                                                        || begin
                                                            && candidate != 0
                                                            && !(multiline
                                                                && [10, 13].contains(
                                                                    &units[candidate - 1],
                                                                ))
                                                    {
                                                        return None;
                                                    }
                                                    let limit = max
                                                        .unwrap_or(units.len() - candidate)
                                                        .min(units.len() - candidate);
                                                    let counts: Vec<_> = if greedy {
                                                        (min..=limit).rev().collect()
                                                    } else {
                                                        (min..=limit).collect()
                                                    };
                                                    counts.into_iter().find_map(|count| {
                                                        let tail = candidate + count;
                                                        let finish =
                                                            tail.checked_add(suffix.len())?;
                                                        let segment = units.get(tail..finish)?;
                                                        if !units[candidate..tail]
                                                            .iter()
                                                            .all(|u| [97, 98].contains(u))
                                                            || !segment
                                                                .iter()
                                                                .copied()
                                                                .eq(suffix.bytes().map(u16::from))
                                                        {
                                                            return None;
                                                        }
                                                        if end
                                                            && finish != units.len()
                                                            && !(multiline
                                                                && [10, 13]
                                                                    .contains(&units[finish]))
                                                        {
                                                            return None;
                                                        }
                                                        Some(candidate..finish)
                                                    })
                                                });
                                            assert_eq!(
                                                matcher.find(&input, start, sticky),
                                                expected,
                                                "{source} {units:?} start={start} sticky={sticky} m={multiline}"
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
    }

    #[test]
    fn quantified_anchor_runs_are_linear_compact_and_charged_before_execution() {
        let units = JsString::from("a".repeat(300000).as_str());
        for source in ["^[^]*$", "^[^]*?$"] {
            let matcher =
                RegExpAnchoredMatcher::compile(&JsString::from(source), false, false).unwrap();
            assert_eq!(matcher.find(&units, 0, false), Some(0..units.len()));
            assert_eq!(matcher.clone().find(&units, 1, true), None);
            assert_eq!(matcher.search_passes(true), 3);
            assert!(matcher.requires_full_suffix());
        }
        let failed =
            RegExpAnchoredMatcher::compile(&JsString::from("^[a]*aaaaab$"), false, false).unwrap();
        assert_eq!(failed.find(&units, 0, false), None);
        let long = JsString::from(format!("{}b", "a\n".repeat(150000)).as_str());
        assert_eq!(failed.find(&long, 0, false), None);
        let overflow = RegExpAnchoredMatcher::compile(
            &JsString::from("^a{999999999999999999999999999}$"),
            false,
            false,
        )
        .unwrap();
        assert_eq!(overflow.find(&units, 0, false), None);
        let fixed = RegExpAnchoredMatcher::compile(&JsString::from("^[a]$"), false, false).unwrap();
        assert!(!fixed.requires_full_suffix());
        let result = RegExpAnchoredMatcher::compile_with_work(
            &JsString::from(r"^\d+$"),
            false,
            false,
            false,
            |work| {
                if work == 65536 {
                    Err("host abort")
                } else {
                    Ok(())
                }
            },
        );
        assert_eq!(result.unwrap_err(), "host abort");
        let branches =
            crate::RegExpDisjunctionMatcher::compile(&JsString::from("^a*$|([x])"), false).unwrap();
        assert!(branches.requires_full_suffix());
        assert_eq!(branches.search_passes(true), 4);
    }

    #[test]
    fn fixed_class_sequence_anchor_snapshot() {
        let mut rows = String::new();
        for source in [
            "^([a])()$",
            "[a]b$",
            "^[^b]a",
            r"^[\d]$",
            "^[.$]$",
            "^[a]$",
            "^.$",
            r"^\.$",
            r"^[a]\$$",
            r"^[a]\\$",
            "[a]^b",
            "(^[a])",
            "^[a]|b$",
            "^[a]*$",
            "^^[a]",
        ] {
            for (ignore_case, multiline, dot_all) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
            ] {
                let matcher = RegExpAnchoredMatcher::compile_with_work(
                    &JsString::from(source),
                    ignore_case,
                    multiline,
                    dot_all,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap();
                write!(rows, "{source:?} i={ignore_case} m={multiline} s={dot_all}").unwrap();
                if let Some(matcher) = matcher {
                    write!(rows, " captures={:?}", matcher.capture_ranges()).unwrap();
                    for input in [
                        "",
                        "a",
                        "A",
                        "ba",
                        "aba",
                        "ab\n",
                        "x\na\ry",
                        "\r\n",
                        "x\u{2028}a\u{2029}y",
                        "a$",
                        "a\\",
                        "\n",
                        ".",
                    ] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            matcher.find(&input, 0, false),
                            matcher.find(&input, 1, true)
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
    fn class_candidate_boundaries_agree_with_an_independent_position_oracle() {
        let alphabet = [97, 98, 10, 13, 0xd800];
        for (begin, end) in [(true, false), (false, true), (true, true)] {
            let source = format!(
                "{}([ab])([a]){}",
                if begin { "^" } else { "" },
                if end { "$" } else { "" }
            );
            for multiline in [false, true] {
                let matcher = RegExpAnchoredMatcher::compile(
                    &JsString::from(source.as_str()),
                    false,
                    multiline,
                )
                .unwrap();
                assert_eq!(matcher.capture_ranges(), [0..1, 1..2]);
                for length in 0..=4u32 {
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
                                    let end_offset = offset.checked_add(2)?;
                                    let pair = units.get(offset..end_offset)?;
                                    if !matches!(pair[0], 97 | 98) || pair[1] != 97 {
                                        return None;
                                    }
                                    let begins = !begin
                                        || offset == 0
                                        || (multiline && [10, 13].contains(&units[offset - 1]));
                                    let ends = !end
                                        || end_offset == units.len()
                                        || (multiline && [10, 13].contains(&units[end_offset]));
                                    (begins && ends).then_some(offset..end_offset)
                                });
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
                                    expected,
                                    "{source} {units:?} start={start} sticky={sticky} m={multiline}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn anchored_set_construction_preserves_host_abort_and_unsupported_distinction() {
        let result = RegExpAnchoredMatcher::compile_with_work(
            &JsString::from(r"^\d$"),
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
        assert!(
            RegExpAnchoredMatcher::compile_with_work(
                &JsString::from("^[a]+[b]+$"),
                false,
                false,
                false,
                |_| -> Result<(), ()> {
                    panic!("unsupported complete body must be rejected before set preparation")
                }
            )
            .unwrap()
            .is_none()
        );
        let matcher =
            RegExpAnchoredMatcher::compile(&JsString::from("^[a]a$"), false, false).unwrap();
        assert_eq!(matcher.search_passes(false), 3);
        assert_eq!(matcher.search_passes(true), 2);
    }

    #[test]
    fn literal_input_and_line_anchor_snapshot() {
        let mut rows = String::new();
        for source in [
            "^", "$", "^$", "^a", "a$", "^a$", "^(a)()$", r"^a\$", r"a\\$", r"\^a$", "^(?:a)$",
            "a", "^a|b$", "(^a)", "a$b", "^a*", "^^a",
        ] {
            for multiline in [false, true] {
                let source = JsString::from(source);
                write!(rows, "{source:?} m={multiline}").unwrap();
                if let Some(matcher) = RegExpAnchoredMatcher::compile(&source, false, multiline) {
                    write!(rows, " captures={:?}", matcher.capture_ranges()).unwrap();
                    for input in [
                        "",
                        "a",
                        "ba",
                        "a\n",
                        "x\na\ry",
                        "\r\n",
                        "x\u{2028}a\u{2029}y",
                        "a$",
                        "a\\",
                        "^a",
                    ] {
                        let input = JsString::from(input);
                        write!(
                            rows,
                            " {input:?}:{:?}/{:?}",
                            matcher.find(&input, 0, false),
                            matcher.find(&input, 1, true)
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
    fn rejected_overlapping_candidates_continue_the_linear_scan() {
        let matcher =
            RegExpAnchoredMatcher::compile(&JsString::from("aba$"), false, false).unwrap();
        assert_eq!(matcher.find(&JsString::from("ababa"), 0, false), Some(2..5));
        let source = format!("{}$", "a".repeat(60_000));
        let matcher =
            RegExpAnchoredMatcher::compile(&JsString::from(source.as_str()), false, false).unwrap();
        assert_eq!(
            matcher.find(&JsString::from("a".repeat(120_000).as_str()), 0, false),
            Some(60_000..120_000)
        );
        let matcher = RegExpAnchoredMatcher::compile(&JsString::from("^()$"), false, true).unwrap();
        assert_eq!(
            matcher.find(&JsString::from("x\r\ny"), 0, false),
            Some(2..2)
        );
    }

    #[test]
    fn anchors_agree_with_an_independent_position_and_boundary_oracle() {
        let mut inputs = vec![Vec::new()];
        let mut previous = vec![Vec::new()];
        for _ in 0..6 {
            let mut next = Vec::new();
            for word in previous {
                for unit in [u16::from(b'a'), 0x0a, 0x0d] {
                    let mut word = word.clone();
                    word.push(unit);
                    inputs.push(word.clone());
                    next.push(word);
                }
            }
            previous = next;
        }
        for body in ["", "a", "aa", "aaa"] {
            for (begin, end) in [(true, false), (false, true), (true, true)] {
                let source = format!(
                    "{}{body}{}",
                    if begin { "^" } else { "" },
                    if end { "$" } else { "" }
                );
                for multiline in [false, true] {
                    let matcher = RegExpAnchoredMatcher::compile(
                        &JsString::from(source.as_str()),
                        false,
                        multiline,
                    )
                    .unwrap();
                    for units in &inputs {
                        let input = JsString::from_code_units(units.clone());
                        for start in 0..=units.len() + 1 {
                            for sticky in [false, true] {
                                let expected = (start..=units.len()).find_map(|offset| {
                                    if sticky && offset != start {
                                        return None;
                                    }
                                    let end_offset = offset.checked_add(body.len())?;
                                    let candidate = units.get(offset..end_offset)?;
                                    if !candidate.iter().copied().eq(body.bytes().map(u16::from)) {
                                        return None;
                                    }
                                    let begin_ok = !begin
                                        || offset == 0
                                        || (multiline && [0x0a, 0x0d].contains(&units[offset - 1]));
                                    let end_ok = !end
                                        || end_offset == units.len()
                                        || (multiline && [0x0a, 0x0d].contains(&units[end_offset]));
                                    (begin_ok && end_ok).then_some(offset..end_offset)
                                });
                                assert_eq!(
                                    matcher.find(&input, start, sticky),
                                    expected,
                                    "{source:?} {input:?} start={start} m={multiline} y={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn complete_anchored_group_snapshot() {
        let mut rows = String::new();
        for source in [
            "^(?:a+b)$",
            "^(a+b)$",
            "^((a+b))$",
            "^(?:(a+b))$",
            "^(a+?aa)$",
            "^(a+aa)$",
            "((a)+b)$",
            "^((a)+b)",
            "^((a)+(b))$",
            "^((a)*())$",
            "^((a*)())$",
            "^((?:a)*x)$",
            "^([ab]*?ab)$",
            "^([ab]*ab)$",
            "^((ab))$",
            "^([ab](c))$",
            "^((?:))$",
            "^([ab]*)$",
            r"^(\d+x)$",
            r"^((.)+\uD800)$",
            "^(µ+x)$",
            r"^(a*\$)$",
            "^(a+[b])$",
            "^(a+b)*$",
            "^a(a+b)$",
            "^(a|b)$",
            "^((a|b))$",
            "^(?<n>a)$",
            "^((^a))$",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (true, true, false),
                (false, false, true),
            ] {
                let matcher = RegExpAnchoredMatcher::compile_with_work(
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
                        " captures={} full_suffix={} passes={}/{}",
                        matcher.capture_count(),
                        matcher.requires_full_suffix(),
                        matcher.search_passes(false),
                        matcher.search_passes(true)
                    )
                    .unwrap();
                    for text in [
                        "",
                        "a",
                        "ab",
                        "aaaa",
                        "aaab",
                        "abab",
                        "ABab",
                        "12x",
                        "a\nb",
                        "x\naaab\ny",
                        "\r\nab\r\n",
                        "a$",
                        "µµx",
                    ] {
                        let input = JsString::from(text);
                        for (start, sticky) in [(0, false), (1, true)] {
                            let result = matcher.find(&input, start, sticky).map(|r| {
                                let caps = (0..matcher.capture_count())
                                    .map(|i| matcher.capture_range(i, &r))
                                    .collect::<Vec<_>>();
                                (r, caps)
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
    fn enclosing_ranges_follow_independent_anchor_and_repetition_order() {
        let alphabet = [97, 98, 120, 10, 13, 0x2028, 0x2029];
        for (quantifier, min) in [("*", 0), ("+", 1)] {
            for greedy in [false, true] {
                for (begin, end) in [(true, false), (false, true), (true, true)] {
                    for multiline in [false, true] {
                        let source = format!(
                            "{}(([ab]){quantifier}{}(ab)()){}",
                            if begin { "^" } else { "" },
                            if greedy { "" } else { "?" },
                            if end { "$" } else { "" }
                        );
                        let matcher = RegExpAnchoredMatcher::compile(
                            &JsString::from(source.as_str()),
                            false,
                            multiline,
                        )
                        .unwrap();
                        assert_eq!(matcher.capture_count(), 4);
                        for len in 0..=3u32 {
                            for mut encoded in 0..alphabet.len().pow(len) {
                                let mut units = Vec::new();
                                for _ in 0..len {
                                    units.push(alphabet[encoded % alphabet.len()]);
                                    encoded /= alphabet.len();
                                }
                                let input = JsString::from_code_units(units.clone());
                                for start in 0..=units.len() + 1 {
                                    for sticky in [false, true] {
                                        let expected = (start..=units.len())
                                            .take(if sticky { 1 } else { usize::MAX })
                                            .find_map(|candidate| {
                                                if begin
                                                    && candidate != 0
                                                    && !(multiline
                                                        && [10, 13, 0x2028, 0x2029]
                                                            .contains(&units[candidate - 1]))
                                                {
                                                    return None;
                                                }
                                                let counts: Vec<_> = if greedy {
                                                    (min..=units.len() - candidate).rev().collect()
                                                } else {
                                                    (min..=units.len() - candidate).collect()
                                                };
                                                counts.into_iter().find_map(|count| {
                                                    let tail = candidate + count;
                                                    let finish = tail + 2;
                                                    if !units[candidate..tail]
                                                        .iter()
                                                        .all(|u| [97, 98].contains(u))
                                                        || units.get(tail..finish)
                                                            != Some(&[97, 98])
                                                    {
                                                        return None;
                                                    }
                                                    if end
                                                        && finish != units.len()
                                                        && !(multiline
                                                            && [10, 13, 0x2028, 0x2029]
                                                                .contains(&units[finish]))
                                                    {
                                                        return None;
                                                    }
                                                    Some((
                                                        candidate..finish,
                                                        vec![
                                                            Some(candidate..finish),
                                                            (count > 0).then(|| tail - 1..tail),
                                                            Some(tail..finish),
                                                            Some(finish..finish),
                                                        ],
                                                    ))
                                                })
                                            });
                                        let actual = matcher.find(&input, start, sticky).map(|r| {
                                            let caps = (0..4)
                                                .map(|i| matcher.capture_range(i, &r))
                                                .collect::<Vec<_>>();
                                            (r, caps)
                                        });
                                        assert_eq!(
                                            actual, expected,
                                            "{source} {units:?} {start} {sticky} m={multiline}"
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
    fn deep_anchored_groups_preserve_static_layouts_and_optional_work() {
        let source = format!("^{}a+b{}$", "(".repeat(100_000), ")".repeat(100_000));
        let matcher =
            RegExpAnchoredMatcher::compile(&JsString::from(source.as_str()), false, false).unwrap();
        let copy = matcher.clone();
        let input = JsString::from(format!("{}b", "a".repeat(300_000)).as_str());
        let r = copy.find(&input, 0, false).unwrap();
        assert_eq!(copy.capture_count(), 100_000);
        assert_eq!(copy.capture_range(0, &r), Some(0..300_001));
        assert_eq!(copy.capture_range(99_999, &r), Some(0..300_001));
        assert_eq!(copy.capture_range(100_000, &r), None);
        assert!(copy.capture_ranges().is_empty());
        assert_eq!(copy.search_passes(true), 3);
        assert!(copy.requires_full_suffix());
        assert_eq!(
            copy.find(&JsString::from("a".repeat(300_000).as_str()), 0, false),
            None
        );
        let fixed =
            RegExpAnchoredMatcher::compile(&JsString::from("^((ab))$"), false, false).unwrap();
        assert_eq!(fixed.capture_ranges(), &[0..2, 0..2]);
        assert_eq!(
            RegExpAnchoredMatcher::compile_with_work(
                &JsString::from("^(a+b)$"),
                false,
                false,
                false,
                |_| Err("abort")
            )
            .unwrap_err(),
            "abort"
        );
    }
}
