//! Fixed ordinary prefixes constrain a single quantified continuation (22.2.2.3).

use crate::regexp_assertion::Assertions;
use crate::{
    JsString, RegExpLiteralMatcher, RegExpQuantifiedContinuationMatcher, RegExpQuantifiedMatcher,
    RegExpSequenceMatcher, regexp_character::PreparedCharacter,
};
use std::{ops::Range, sync::Arc};

/// An ordinary fixed-width prefix before one quantified atom and suffix.
///
/// The complete Pattern must already be validated without `u` or `v`. The prefix
/// contains literal characters, sets, dots, word assertions and unquantified groups,
/// including empty captures; the body reuses the complete quantified
/// atom/fixed-continuation grammar. Search streams prefix occurrences and
/// continuation candidates in monotone order, without allocation or recursion.
/// Literal prefixes search linearly; fixed sequences inspect consuming terms and
/// assertion offsets at each input candidate. Sticky search checks one prefix.
#[derive(Clone, Debug)]
pub struct RegExpPrefixedMatcher(Arc<Program>);

#[derive(Debug)]
struct Program {
    prefix: Prefix,
    body: RegExpQuantifiedContinuationMatcher,
    capture_count: usize,
}

#[derive(Debug)]
enum Prefix {
    Literal(RegExpLiteralMatcher),
    Sequence(RegExpSequenceMatcher),
}

impl Prefix {
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
    fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        match self {
            Self::Literal(m) => m.find(input, start, sticky),
            Self::Sequence(m) => m.find(input, start, sticky),
        }
    }
    fn matches_from<'a>(&'a self, input: &'a [u16], start: usize) -> Option<PrefixMatches<'a>> {
        match self {
            Self::Literal(m) => m.matches_from(input, start).map(PrefixMatches::Literal),
            Self::Sequence(m) => m.matches_from(input, start).map(PrefixMatches::Sequence),
        }
    }
}

enum PrefixMatches<'a> {
    Literal(crate::regexp_literal::LiteralMatches<'a>),
    Sequence(crate::regexp_sequence::SequenceMatches<'a>),
}

impl Iterator for PrefixMatches<'_> {
    type Item = Range<usize>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Literal(m) => m.next(),
            Self::Sequence(m) => m.next(),
        }
    }
}

impl RegExpPrefixedMatcher {
    /// Total captures in source order: prefix, quantified atom, and continuation.
    pub fn capture_count(&self) -> usize {
        self.0.capture_count
    }

    /// Absolute capture range, including fixed prefix and dynamic body captures.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        if index >= self.capture_count() {
            return None;
        }
        let start = matched.start.checked_add(self.0.prefix.matched_len())?;
        if start > matched.end {
            return None;
        }
        let prefix_captures = self.0.prefix.capture_ranges();
        if let Some(range) = prefix_captures.get(index) {
            return Some(
                matched.start.checked_add(range.start)?..matched.start.checked_add(range.end)?,
            );
        }
        self.0
            .body
            .capture_range(index - prefix_captures.len(), &(start..matched.end))
    }

    /// Compiles the complete fixed-prefix/quantified-body subset.
    pub fn compile(source: &JsString, ignore_case: bool, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, ignore_case, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Charges preparation before copying and compiling each accepted component.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(split) = prefix_end(source.code_units()) else {
            return Ok(None);
        };
        if split == 0 {
            return Ok(None);
        }
        charge(source.len())?;
        charge(source.len())?;
        let prefix = JsString::from_code_units(source.code_units()[..split].to_vec());
        let prefix = if let Some(m) = RegExpLiteralMatcher::compile(&prefix, ignore_case) {
            Prefix::Literal(m)
        } else if let Some(m) =
            RegExpSequenceMatcher::compile_with_work(&prefix, ignore_case, dot_all, &mut charge)?
        {
            Prefix::Sequence(m)
        } else {
            return Ok(None);
        };
        let body = JsString::from_code_units(source.code_units()[split..].to_vec());
        let body = if let Some(q) =
            RegExpQuantifiedMatcher::compile_with_work(&body, ignore_case, dot_all, &mut charge)?
        {
            RegExpQuantifiedContinuationMatcher::from_quantified(q)
        } else if let Some(q) = RegExpQuantifiedContinuationMatcher::compile_with_work(
            &body,
            ignore_case,
            dot_all,
            &mut charge,
        )? {
            q
        } else {
            return Ok(None);
        };
        let Some(capture_count) = prefix
            .capture_ranges()
            .len()
            .checked_add(body.capture_count())
        else {
            return Ok(None);
        };
        Ok(Some(Self(Arc::new(Program {
            prefix,
            body,
            capture_count,
        }))))
    }

    /// Conservative consuming passes for optional work accounting. Literal
    /// prefixes retain linear KMP search; sequences inspect every prefix atom at
    /// most once per input candidate, alongside the body and continuation passes.
    pub fn search_passes(&self, sticky: bool) -> usize {
        let prefix_passes = match &self.0.prefix {
            Prefix::Literal(_) => 1,
            Prefix::Sequence(m) => m.search_passes(sticky),
        };
        prefix_passes.saturating_add(self.0.body.search_passes())
    }

    /// Finds the earliest whole prefix match and its greedy/lazy body endpoint.
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

    /// Whole-prefix assertions constrain starts and body endpoints before repetition
    /// selection. The prefix stream and repeated-body cursors remain monotone.
    pub(crate) fn find_asserted(
        &self,
        input: &JsString,
        start: usize,
        sticky: bool,
        leading: Assertions,
        trailing: Assertions,
        multiline: bool,
    ) -> Option<Range<usize>> {
        let unconstrained_empty = match &self.0.prefix {
            Prefix::Literal(m) => m.matched_len() == 0,
            Prefix::Sequence(m) => m.atom_count() == 0 && !m.has_word_assertions(),
        };
        if unconstrained_empty {
            return self
                .0
                .body
                .find_asserted(input, start, sticky, leading, trailing, multiline);
        }
        let units = input.code_units();
        let allowed_start = |position| leading.accepts(units, position, multiline);
        let allowed_end = |position| trailing.accepts(units, position, multiline);
        let len = self.0.prefix.matched_len();
        let body_start = start.checked_add(len)?;
        let matched = if sticky {
            let prefix = self.0.prefix.find(input, start, true)?;
            if !allowed_start(prefix.start) {
                return None;
            }
            self.0
                .body
                .find_with(input, prefix.end, true, Some, allowed_end)?
        } else {
            let mut prefixes = self.0.prefix.matches_from(input.code_units(), start)?;
            let mut next = prefixes.next();
            self.0.body.find_with(
                input,
                body_start,
                false,
                |minimum| {
                    loop {
                        let prefix = next.as_ref()?;
                        if prefix.end >= minimum && allowed_start(prefix.start) {
                            return Some(prefix.end);
                        }
                        next = prefixes.next();
                    }
                },
                allowed_end,
            )?
        };
        Some(matched.start.checked_sub(len)?..matched.end)
    }
}

/// Find the complete fixed-width prefix before the first quantified
/// Atom. Quantifiers inside a group retain the entire top-level group in the
/// body; a quantifier after a group applies to that complete group (22.2.2.3).
/// Only scalar nesting state is needed; fixed capture compilation stays shared.
fn prefix_end(units: &[u16]) -> Option<usize> {
    let mut index = 0;
    let mut last = None;
    let mut depth = 0usize;
    let mut group_start = 0;
    while let Some(&unit) = units.get(index) {
        let start = index;
        match unit {
            40 => {
                if depth == 0 {
                    group_start = start;
                }
                depth = depth.checked_add(1)?;
                index += 1;
                if units.get(index) == Some(&63) {
                    if units.get(index..index + 2)? != [63, 58] {
                        return None;
                    }
                    index += 2;
                }
                continue;
            }
            41 => {
                depth = depth.checked_sub(1)?;
                index += 1;
                if depth == 0 {
                    last = Some(group_start);
                }
                continue;
            }
            46 | 91 => {
                let (_, consumed) = PreparedCharacter::parse(&units[index..], false)?;
                index += consumed;
            }
            42 | 43 | 63 | 123 => return if depth == 0 { last } else { Some(group_start) },
            92 if units
                .get(index + 1)
                .is_some_and(|&unit| unit == 98 || unit == 66) =>
            {
                index += 2;
            }
            92 => {
                index += 1;
                if crate::regexp_literal::character_escape(units, &mut index).is_none() {
                    let (_, consumed) = PreparedCharacter::parse(&units[start..], false)?;
                    index = start + consumed;
                }
            }
            _ if crate::regexp_literal::is_syntax(unit) || unit == 124 => return None,
            _ => index += 1,
        }
        if depth == 0 {
            last = Some(start);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn literal_prefix_snapshot() {
        let mut rows = String::new();
        for source in [
            "ab+",
            "ab+?",
            "ab*",
            "ab*?",
            "aa+a",
            "aa+?a",
            "aaa*",
            "aaa*?",
            "ab{1,3}",
            "ab{0,2}?b",
            "a([ab])*ab",
            "a([ab])*?ab",
            "x(a)+(b)",
            "a((a)+)b",
            "x(a*)y",
            "x(a)*y",
            r"a\d+x",
            r"\u0061b+",
            r"a\u0062+",
            "µa+",
            "x(.)+",
            "x(.)*",
            "a[^]+",
            "a[]*",
            "a[]+",
            "ab{999999999999999999999999999}",
            "a(?:a)+b",
            "a(?:(?:a)+)b",
            "ab+((c)())",
            "ab+()",
            r"a\(b+",
            "💩+",
            "(a)b+",
            "(?:a)b+",
            "a.b+",
            "a[b]c+",
            "ab+c+",
            "ab+(c|d)",
            "ab+[c]",
            "ab+(?<n>c)",
            "a+",
            "ab",
            "^ab+",
            "ab+$",
        ] {
            for (i, s) in [(false, false), (true, false), (false, true)] {
                let matcher = RegExpPrefixedMatcher::compile(&JsString::from(source), i, s);
                write!(rows, "{source:?} i={i} s={s}").unwrap();
                if let Some(matcher) = matcher {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for text in [
                        "", "a", "ab", "abb", "abbb", "aaa", "aaaa", "aab", "aabab", "xaab", "xay",
                        "xy", "12x", "a12x", "ΜAA", "x\n", "abc",
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
    fn overlapping_prefixes_agree_with_independent_candidate_and_repetition_order() {
        for prefix in ["a", "aa", "ab", "aba"] {
            for (quantifier, min, max) in [
                ("*", 0, None),
                ("+", 1, None),
                ("{0,2}", 0, Some(2)),
                ("{2,3}", 2, Some(3)),
            ] {
                for suffix in ["", "a", "ab"] {
                    for greedy in [false, true] {
                        let text = format!(
                            "{prefix}([ab]){quantifier}{}{suffix}",
                            if greedy { "" } else { "?" }
                        );
                        let matcher = RegExpPrefixedMatcher::compile(
                            &JsString::from(text.as_str()),
                            false,
                            false,
                        )
                        .unwrap();
                        assert_eq!(matcher.capture_count(), 1);
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
                                                let begin = candidate + prefix.len();
                                                let segment = units.get(candidate..begin)?;
                                                if !segment
                                                    .iter()
                                                    .copied()
                                                    .eq(prefix.bytes().map(u16::from))
                                                {
                                                    return None;
                                                }
                                                let limit = max
                                                    .unwrap_or(units.len() - begin)
                                                    .min(units.len() - begin);
                                                let counts: Vec<_> = if greedy {
                                                    (min..=limit).rev().collect()
                                                } else {
                                                    (min..=limit).collect()
                                                };
                                                counts.into_iter().find_map(|count| {
                                                    let end = begin + count;
                                                    let finish = end + suffix.len();
                                                    if !units[begin..end]
                                                        .iter()
                                                        .all(|u| [97, 98].contains(u))
                                                        || !units
                                                            .get(end..finish)?
                                                            .iter()
                                                            .copied()
                                                            .eq(suffix.bytes().map(u16::from))
                                                    {
                                                        return None;
                                                    }
                                                    Some((
                                                        candidate..finish,
                                                        (count > 0).then(|| end - 1..end),
                                                    ))
                                                })
                                            });
                                        let actual = matcher.find(&input, start, sticky).map(|r| {
                                            let cap = matcher.capture_range(0, &r);
                                            (r, cap)
                                        });
                                        assert_eq!(
                                            actual, expected,
                                            "{text} {units:?} {start} {sticky}"
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
    fn long_prefixes_failed_overlap_runs_clones_and_utf16_offsets_stay_compact() {
        let text = format!("{}b+", "a".repeat(100_000));
        let matcher =
            RegExpPrefixedMatcher::compile(&JsString::from(text.as_str()), false, false).unwrap();
        let copy = matcher.clone();
        let input = JsString::from(format!("{}bb", "a".repeat(300_000)).as_str());
        assert_eq!(copy.find(&input, 0, false), Some(200_000..300_002));
        let failed =
            RegExpPrefixedMatcher::compile(&JsString::from("aaa+b"), false, false).unwrap();
        let input = JsString::from("a".repeat(300_000).as_str());
        assert_eq!(failed.find(&input, 0, false), None);
        let source = JsString::from("💩+");
        let matcher = RegExpPrefixedMatcher::compile(&source, false, false).unwrap();
        let input = JsString::from_code_units(vec![0xd83d, 0xdca9, 0xdca9, 0xdca9]);
        assert_eq!(matcher.find(&input, 0, false), Some(0..4));
        let escaped =
            RegExpPrefixedMatcher::compile(&JsString::from(r"\u0061(b)+"), false, false).unwrap();
        let input = JsString::from("abb");
        let r = escaped.find(&input, 0, false).unwrap();
        assert_eq!(escaped.capture_range(0, &r), Some(2..3));
        assert_eq!(
            RegExpPrefixedMatcher::compile_with_work(
                &JsString::from("ab+"),
                false,
                false,
                |_| Err("abort")
            )
            .unwrap_err(),
            "abort"
        );
    }

    #[test]
    fn prefix_group_capture_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a)b+",
            "(?:a)b+",
            "((a))b+",
            "()(a)()b+",
            "()a+",
            "(?:)a+",
            "()([ab])*()",
            "()([ab])*?()",
            "(a)(b)+((c)())",
            "((ab))a*?a",
            "(a())(b*)()",
            "(a)(b)*()",
            "(a)[]*()",
            "()[]*()",
            "()[]+()",
            r"(\u0061)(b)+()",
            r"(\n)(a)+",
            "(µ)(a)+",
            "(💩)a+",
            "()(a){0,2}?()",
            "((?:a)())((b)+)(c)",
            "((a)b)+",
            "(a|b)c+",
            "([a])b+",
            "(a)b+c+",
            "(?<n>a)b+",
            "(a)(?=b)b+",
            "()a+u|x",
            "(a)b+[c]",
            "(a(b+))",
        ] {
            for (ignore_case, dot_all) in [(false, false), (true, false), (false, true)] {
                let matcher =
                    RegExpPrefixedMatcher::compile(&JsString::from(source), ignore_case, dot_all);
                write!(rows, "{source:?} i={ignore_case} s={dot_all}").unwrap();
                if let Some(matcher) = matcher {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for text in [
                        "", "a", "ab", "abb", "abbc", "aaaa", "xabbc", "x", "ΜAA", "💩aa", "\nAA",
                    ] {
                        for (start, sticky) in [(0, false), (0, true), (1, true)] {
                            let input = JsString::from(text);
                            let result = matcher.find(&input, start, sticky).map(|r| {
                                let caps = (0..matcher.capture_count())
                                    .map(|c| matcher.capture_range(c, &r))
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
    fn prefix_captures_and_empty_groups_agree_with_independent_repetition_order() {
        for prefix in ["", "a", "aa", "ab"] {
            for style in 0..3 {
                let prefix_source = match style {
                    0 => format!("({prefix})"),
                    1 => format!("(?:(?:({prefix})))"),
                    _ => format!("()(?:({prefix}))()"),
                };
                for (quantifier, min, max) in
                    [("*", 0, None), ("+", 1, None), ("{0,2}", 0, Some(2))]
                {
                    for suffix in ["", "a"] {
                        for greedy in [false, true] {
                            let source = format!(
                                "{prefix_source}([ab]){quantifier}{}({suffix})()",
                                if greedy { "" } else { "?" }
                            );
                            let matcher = RegExpPrefixedMatcher::compile(
                                &JsString::from(source.as_str()),
                                false,
                                false,
                            )
                            .unwrap();
                            let prefix_caps = if style == 2 { 3 } else { 1 };
                            assert_eq!(matcher.capture_count(), prefix_caps + 3);
                            for len in 0..=4u32 {
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
                                                    let begin = candidate + prefix.len();
                                                    if !units
                                                        .get(candidate..begin)?
                                                        .iter()
                                                        .copied()
                                                        .eq(prefix.bytes().map(u16::from))
                                                    {
                                                        return None;
                                                    }
                                                    let limit = max
                                                        .unwrap_or(units.len() - begin)
                                                        .min(units.len() - begin);
                                                    let counts: Vec<_> = if greedy {
                                                        (min..=limit).rev().collect()
                                                    } else {
                                                        (min..=limit).collect()
                                                    };
                                                    counts.into_iter().find_map(|count| {
                                                        let end = begin + count;
                                                        let finish = end + suffix.len();
                                                        if !units[begin..end]
                                                            .iter()
                                                            .all(|u| [97, 98].contains(u))
                                                            || !units
                                                                .get(end..finish)?
                                                                .iter()
                                                                .copied()
                                                                .eq(suffix.bytes().map(u16::from))
                                                        {
                                                            return None;
                                                        }
                                                        let mut caps = if style == 2 {
                                                            vec![
                                                                Some(candidate..candidate),
                                                                Some(candidate..begin),
                                                                Some(begin..begin),
                                                            ]
                                                        } else {
                                                            vec![Some(candidate..begin)]
                                                        };
                                                        caps.extend([
                                                            (count > 0).then(|| end - 1..end),
                                                            Some(end..finish),
                                                            Some(finish..finish),
                                                        ]);
                                                        Some((candidate..finish, caps))
                                                    })
                                                });
                                            let actual =
                                                matcher.find(&input, start, sticky).map(|r| {
                                                    let caps = (0..matcher.capture_count())
                                                        .map(|c| matcher.capture_range(c, &r))
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
        }
    }

    #[test]
    fn large_empty_prefix_capture_layouts_stay_flat_and_checked() {
        let source = JsString::from(format!("{}(a)+()", "()".repeat(100_000)).as_str());
        let matcher = RegExpPrefixedMatcher::compile(&source, false, false).unwrap();
        let copy = matcher.clone();
        assert_eq!(copy.capture_count(), 100_002);
        let r = copy.find(&JsString::from("xaa"), 0, false).unwrap();
        assert_eq!(r, 1..3);
        assert_eq!(copy.capture_range(0, &r), Some(1..1));
        assert_eq!(copy.capture_range(99_999, &r), Some(1..1));
        assert_eq!(copy.capture_range(100_000, &r), Some(2..3));
        assert_eq!(copy.capture_range(100_001, &r), Some(3..3));
        assert_eq!(copy.capture_range(100_002, &r), None);
        assert_eq!(copy.find(&JsString::from("aa"), 3, false), None);
        assert_eq!(copy.find(&JsString::from("xaa"), 0, true), None);
        let prefix =
            RegExpPrefixedMatcher::compile(&JsString::from("(a)b+"), false, false).unwrap();
        assert_eq!(prefix.capture_range(0, &(usize::MAX..usize::MAX)), None);
        assert_eq!(prefix.capture_range(0, &Range { start: 2, end: 1 }), None);
        assert_eq!(
            RegExpPrefixedMatcher::compile_with_work(&source, false, false, |_| Err("abort"))
                .unwrap_err(),
            "abort"
        );
    }

    #[test]
    fn sequence_prefix_snapshot() {
        let mut rows = String::new();
        for source in [
            "[ab]c+",
            "([ab])(c)+()",
            ".[ab]+",
            "a.b+",
            "a[b]c+",
            "([^x])a*?a",
            "([ab][ab])a+",
            "([])a*",
            "([^])a+",
            "((a[bc]))(d)+()",
            "(?:(.))a+",
            r"(\d)(a)+",
            r"(\w)(a)+",
            r"(\s)(a)+",
            r"(\D)(a)+",
            "([µ])(a)+",
            "([ſ])a+",
            "([💩])a+",
            "()[ab](c)*()",
            "([ab])c{0,2}?()",
            "[ab]c+()",
            "[ab](c*)()",
            "[ab](c)*()",
            "([ab])c+[d]",
            "([ab])c+d+",
            "([ab]c)+",
            "([ab]|c)d+",
            "(?<n>[ab])c+",
            "([ab])(?=c)c+",
            "^([ab])c+",
            "([ab])c+$",
        ] {
            for (ignore_case, dot_all) in [(false, false), (true, false), (false, true)] {
                let m =
                    RegExpPrefixedMatcher::compile(&JsString::from(source), ignore_case, dot_all);
                write!(rows, "{source:?} i={ignore_case} s={dot_all}").unwrap();
                if let Some(m) = m {
                    write!(rows, " captures={}", m.capture_count()).unwrap();
                    for text in [
                        "", "a", "aa", "ac", "acc", "abccc", "xbc", "xaabb", "1aa", " aa", "ΜAA",
                        "saa", "\naa", "💩aa",
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
    fn sequence_prefixes_agree_with_independent_candidate_and_capture_order() {
        for (prefix, width, kind) in [
            ("[ab]", 1, 0),
            ("a[ab]", 2, 1),
            ("[ab][ab]", 2, 2),
            (".", 1, 3),
            ("[^x]", 1, 4),
            ("[]", 1, 5),
        ] {
            for (quantifier, min, max) in [("*", 0, None), ("+", 1, None), ("{0,2}", 0, Some(2))] {
                for suffix in ["", "a"] {
                    for greedy in [false, true] {
                        let source = format!(
                            "({prefix})([ab]){quantifier}{}({suffix})()",
                            if greedy { "" } else { "?" }
                        );
                        let matcher = RegExpPrefixedMatcher::compile(
                            &JsString::from(source.as_str()),
                            false,
                            false,
                        )
                        .unwrap();
                        assert_eq!(matcher.capture_count(), 4);
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
                                                let begin = candidate + width;
                                                let p = units.get(candidate..begin)?;
                                                let accepts = match kind {
                                                    0 => [97, 98].contains(&p[0]),
                                                    1 => p[0] == 97 && [97, 98].contains(&p[1]),
                                                    2 => p.iter().all(|u| [97, 98].contains(u)),
                                                    3 => p[0] != 10,
                                                    4 => p[0] != 120,
                                                    _ => false,
                                                };
                                                if !accepts {
                                                    return None;
                                                }
                                                let limit = max
                                                    .unwrap_or(units.len() - begin)
                                                    .min(units.len() - begin);
                                                let counts: Vec<_> = if greedy {
                                                    (min..=limit).rev().collect()
                                                } else {
                                                    (min..=limit).collect()
                                                };
                                                counts.into_iter().find_map(|count| {
                                                    let end = begin + count;
                                                    let finish = end + suffix.len();
                                                    if !units[begin..end]
                                                        .iter()
                                                        .all(|u| [97, 98].contains(u))
                                                        || !units
                                                            .get(end..finish)?
                                                            .iter()
                                                            .copied()
                                                            .eq(suffix.bytes().map(u16::from))
                                                    {
                                                        return None;
                                                    }
                                                    Some((
                                                        candidate..finish,
                                                        vec![
                                                            Some(candidate..begin),
                                                            (count > 0).then(|| end - 1..end),
                                                            Some(end..finish),
                                                            Some(finish..finish),
                                                        ],
                                                    ))
                                                })
                                            });
                                        let actual = matcher.find(&input, start, sticky).map(|r| {
                                            let caps = (0..matcher.capture_count())
                                                .map(|i| matcher.capture_range(i, &r))
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
    }

    #[test]
    fn shared_large_sequence_prefixes_preserve_clones_and_optional_work_bounds() {
        let source = JsString::from(format!("{}(b)+()", "[a]".repeat(1000)).as_str());
        let mut remaining = 100_000usize;
        let m = RegExpPrefixedMatcher::compile_with_work(&source, false, false, |work| {
            remaining = remaining.checked_sub(work).ok_or("abort")?;
            Ok::<(), &str>(())
        })
        .unwrap()
        .unwrap();
        let copy = m.clone();
        let input = JsString::from(format!("{}bb", "a".repeat(1000)).as_str());
        let r = copy.find(&input, 0, true).unwrap();
        assert_eq!(r, 0..1002);
        assert_eq!(copy.capture_range(0, &r), Some(1001..1002));
        assert_eq!(copy.capture_range(1, &r), Some(1002..1002));
        assert_eq!(copy.search_passes(false), 1002);
        assert_eq!(copy.search_passes(true), 3);
        assert_eq!(
            copy.find(
                &JsString::from(format!("{}xbb", "a".repeat(999)).as_str()),
                0,
                true
            ),
            None
        );
        let literal =
            RegExpPrefixedMatcher::compile(&JsString::from("(aaa)b+"), false, false).unwrap();
        assert_eq!(literal.search_passes(false), 3);
        let mut charges = Vec::new();
        assert_eq!(
            RegExpPrefixedMatcher::compile_with_work(
                &JsString::from("([a])b+"),
                false,
                false,
                |work| {
                    charges.push(work);
                    if work == 1024 { Err("abort") } else { Ok(()) }
                }
            )
            .unwrap_err(),
            "abort"
        );
        assert!(charges.contains(&1024));
    }
}
