//! Captures spanning part of one fixed repetition (CompileSubpattern, 22.2.2.3).

use crate::regexp_assertion::Assertions;
use crate::{JsString, RegExpRepeatedPrefixedMatcher, RegExpSequenceMatcher};
use std::{ops::Range, sync::Arc};

/// One repeated fixed group or character atom with partial enclosing captures.
///
/// Patterns must already be validated without `u` or `v`. Ordinary groups outside
/// the repetition use compact endpoints relative to the match start or end;
/// groups inside it keep the existing last-iteration/undefined semantics.
#[derive(Clone, Debug)]
pub struct RegExpRepeatedCaptureMatcher {
    body: RegExpRepeatedPrefixedMatcher,
    captures: Arc<[Capture]>,
}

#[derive(Debug)]
enum Capture {
    Fixed(Endpoint, Endpoint),
    Repeated(usize),
}

#[derive(Debug)]
enum Endpoint {
    Start(usize),
    End(usize),
}

impl Endpoint {
    fn resolve(&self, range: &Range<usize>) -> Option<usize> {
        match *self {
            Self::Start(offset) => range.start.checked_add(offset),
            Self::End(offset) => range.end.checked_sub(offset),
        }
    }
}

impl RegExpRepeatedCaptureMatcher {
    /// Compiles ordinary groups around one repeated group or character atom.
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

    /// Determines the complete capture layout before constructing character sets.
    pub fn compile_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        // Complete wrappers already have a scalar capture layout. Preserve their
        // existing body selection and optional work by leaving unwrapping to
        // the caller; this fallback handles groups around only part of a body.
        if crate::regexp_outer_group_body(source).is_some() {
            return Ok(None);
        }
        let Some(repeated) = repeated_atom(units, dot_all) else {
            return Ok(None);
        };
        let original_atom = JsString::from_code_units(units[repeated.atom.clone()].to_vec());
        let atom = if RegExpSequenceMatcher::repeated_atom_width(&original_atom, dot_all, true)
            .is_some()
        {
            original_atom
        } else if repeated.grouped {
            let Some(atom) = literal_choice_atom(original_atom.code_units()) else {
                return Ok(None);
            };
            JsString::from_code_units(atom)
        } else {
            return Ok(None);
        };
        if RegExpSequenceMatcher::repeated_atom_width(&atom, dot_all, true).is_none() {
            return Ok(None);
        }
        // A one-unit marker distinguishes boundaries before and after a possibly
        // empty repetition. Retaining all surrounding delimiters also preserves
        // lexical escape boundaries, including ordinary identity escapes.
        let mut layout = units[..repeated.source.start].to_vec();
        layout.extend_from_slice(&[40, 46, 41]);
        layout.extend_from_slice(&units[repeated.source.end..]);
        let Some((width, fixed)) = RegExpSequenceMatcher::fixed_capture_layout(
            &JsString::from_code_units(layout),
            dot_all,
        ) else {
            return Ok(None);
        };
        let Some(marker) = fixed.get(repeated.capture_index) else {
            return Ok(None);
        };
        if marker.end.checked_sub(marker.start) != Some(1) {
            return Ok(None);
        }
        let marker_start = marker.start;
        let Some(flat) = flatten(units, &repeated, dot_all, atom.code_units()) else {
            return Ok(None);
        };
        charge(units.len())?;
        charge(flat.len())?;
        charge(fixed.len())?;
        let Some(body) = RegExpRepeatedPrefixedMatcher::compile_with_work(
            &JsString::from_code_units(flat),
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )?
        else {
            return Ok(None);
        };
        let endpoint = |position| {
            if position <= marker_start {
                Endpoint::Start(position)
            } else {
                Endpoint::End(width - position)
            }
        };
        let Some(count) = fixed
            .len()
            .checked_sub(1)
            .and_then(|n| n.checked_add(body.capture_count()))
        else {
            return Ok(None);
        };
        charge(count)?;
        let mut captures = Vec::with_capacity(count);
        for (index, range) in fixed.into_iter().enumerate() {
            if index == repeated.capture_index {
                captures.extend((0..body.capture_count()).map(Capture::Repeated));
            } else {
                captures.push(Capture::Fixed(endpoint(range.start), endpoint(range.end)));
            }
        }
        Ok(Some(Self {
            body,
            captures: captures.into(),
        }))
    }

    /// Number of source-ordered ordinary capture slots.
    pub fn capture_count(&self) -> usize {
        self.captures.len()
    }

    /// Absolute partial-enclosing, fixed or final-iteration capture range.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        if matched.start > matched.end {
            return None;
        }
        match self.captures.get(index)? {
            Capture::Repeated(index) => self.body.capture_range(*index, matched),
            Capture::Fixed(start, end) => {
                let start = start.resolve(matched)?;
                let end = end.resolve(matched)?;
                (matched.start <= start && start <= end && end <= matched.end).then_some(start..end)
            }
        }
    }

    /// Earliest complete start with the repeated body's greedy/lazy endpoint order.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        self.body.find(input, start, sticky)
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
        self.body
            .find_asserted(input, start, sticky, leading, trailing, multiline)
    }

    /// Conservative passes inherited from the fixed-prefix repetition plan.
    pub fn search_passes(&self, sticky: bool) -> usize {
        self.body.search_passes(sticky)
    }
}

fn ordinary_group_end(units: &[u16], index: usize) -> Option<usize> {
    if units.get(index + 1) != Some(&63) {
        Some(index + 1)
    } else if units.get(index + 1..index + 3)? == [63, 58] {
        Some(index + 3)
    } else {
        None
    }
}

fn atom_end(units: &[u16], index: usize, dot_all: bool) -> Option<usize> {
    if let Some((_, width)) =
        crate::regexp_character::PreparedCharacter::parse(&units[index..], dot_all)
    {
        return index.checked_add(width);
    }
    if units[index] == 92 {
        if matches!(units.get(index + 1), Some(98 | 66)) {
            return Some(index + 2);
        }
        let mut end = index + 1;
        crate::regexp_literal::character_escape(units, &mut end)?;
        return Some(end);
    }
    Some(index + 1)
}

struct RepeatedAtom {
    source: Range<usize>,
    atom: Range<usize>,
    capture_index: usize,
    grouped: bool,
}

fn repeated_atom(units: &[u16], dot_all: bool) -> Option<RepeatedAtom> {
    let mut groups = Vec::new();
    let mut captures = 0usize;
    let mut repeated = None;
    let mut index = 0;
    while index < units.len() {
        match units[index] {
            40 => {
                let end = ordinary_group_end(units, index)?;
                groups.push((index, captures));
                if end == index + 1 {
                    captures = captures.checked_add(1)?;
                }
                index = end;
            }
            41 => {
                let (start, capture_index) = groups.pop()?;
                index += 1;
                if let Some((_, consumed)) = crate::regexp_quantified::quantifier(&units[index..]) {
                    if repeated.is_some() {
                        return None;
                    }
                    repeated = Some(RepeatedAtom {
                        source: start..index + consumed,
                        atom: start..index,
                        capture_index,
                        grouped: true,
                    });
                    index += consumed;
                }
            }
            _ => {
                let end = atom_end(units, index, dot_all)?;
                if let Some((_, consumed)) = crate::regexp_quantified::quantifier(&units[end..]) {
                    if repeated.is_some() {
                        return None;
                    }
                    repeated = Some(RepeatedAtom {
                        source: index..end + consumed,
                        atom: index..end,
                        capture_index: captures,
                        grouped: false,
                    });
                    index = end + consumed;
                } else {
                    index = end;
                }
            }
        }
    }
    groups.is_empty().then_some(repeated).flatten()
}

fn flatten(
    units: &[u16],
    repeated: &RepeatedAtom,
    dot_all: bool,
    atom: &[u16],
) -> Option<Vec<u16>> {
    // Empty noncapturing barriers preserve each removed group's lexical boundary:
    // removing delimiters directly could fuse `\0()1` into an octal escape.
    let barrier = [40, 63, 58, 41];
    let mut flat = barrier.to_vec();
    let mut index = 0;
    while index < units.len() {
        if index == repeated.source.start {
            if repeated.grouped {
                flat.extend_from_slice(atom);
                flat.extend_from_slice(&units[repeated.atom.end..repeated.source.end]);
            } else {
                // A synthetic noncapturing group lets the existing fixed-group
                // repetition plan handle a character atom without adding slots.
                flat.extend_from_slice(&[40, 63, 58]);
                flat.extend_from_slice(&units[repeated.atom.clone()]);
                flat.push(41);
                flat.extend_from_slice(&units[repeated.atom.end..repeated.source.end]);
            }
            index = repeated.source.end;
        } else if units[index] == 40 {
            index = ordinary_group_end(units, index)?;
            flat.extend_from_slice(&barrier);
        } else if units[index] == 41 {
            index += 1;
            flat.extend_from_slice(&barrier);
        } else {
            let end = atom_end(units, index, dot_all)?;
            flat.extend_from_slice(&units[index..end]);
            index = end;
        }
    }
    Some(flat)
}

pub(crate) fn literal_choice_atom(units: &[u16]) -> Option<Vec<u16>> {
    // Equal-width capture-free branches have identical endpoints. Their source
    // order therefore cannot change captures or subsequent endpoint selection;
    // one ordinary character set implements the same union of predicates.
    if units.first() != Some(&40) || units.last() != Some(&41) {
        return None;
    }
    let source = JsString::from_code_units(units.to_vec());
    let group = crate::regexp_outer_group_body(&source)?;
    let end = group.body.end;
    let mut index = group.body.start;
    let mut characters = Vec::new();
    while index < end {
        let unit = units[index];
        index += 1;
        let character = if unit == 92 {
            crate::regexp_literal::character_escape(units, &mut index)?
        } else if crate::regexp_literal::is_syntax(unit) {
            return None;
        } else {
            unit
        };
        characters.push(character);
        if index == end {
            break;
        }
        if units.get(index) != Some(&124) {
            return None;
        }
        index += 1;
        if index == end {
            return None;
        }
    }
    if characters.len() < 2 {
        return None;
    }
    let mut atom = if group.captures == 0 {
        vec![40, 63, 58]
    } else {
        vec![40; group.captures]
    };
    atom.push(91);
    for unit in characters {
        atom.extend_from_slice(&[92, 117]);
        for shift in [12, 8, 4, 0] {
            let digit = (unit >> shift) & 15;
            atom.push(if digit < 10 { 48 + digit } else { 87 + digit });
        }
    }
    atom.push(93);
    atom.extend(std::iter::repeat_n(41, group.captures.max(1)));
    Some(atom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn enclosing_repeated_unit_choice_captures_snapshot() {
        let mut rows = String::new();
        for source in [
            "((a|b))+",
            "((a|b))+?",
            "((a|b))*",
            "((a|b)){0}",
            "((a|b)){1,2}",
            "((a|b)){1,2}?",
            "((a|b))+(b)",
            "((a|b))+?(b)",
            "((a|b))*([ab]b)",
            "(((a|b)))+",
            "((?:a|b))+",
            "(?:(a|b))+",
            "(?:(?:a|b))+",
            "(x)(((a|b))+)(y)",
            "((x)((a|b))+)(y)",
            "(x)(((a|b))+(y))",
            "()((a|b))+()",
            "(())(((a|b))*)()c()",
            "x(?:(a|b))+(y)",
            r"((\x61|\u0062))+(c)",
            r"((\n|\r))+($)",
            r"(^)(((a|b))+)($)(\n)",
            r"(\b)(((a|b))+)(c)\b",
            "((µ|Μ))+(x)",
            "((ſ|S))+(x)",
            r"((\uD800|\uDC00))+(c)",
            r"((\uDCA9|a))+(b)",
            "((a|b)){999999999999999999999999999999}(c)",
            "((a|b)){0,999999999999999999999999999999}(c)",
            "((ab|a))+",
            "((a|))+",
            "((a)|(b))+",
            "((a|b)())+",
            "(x(a|b))+",
            "((a|[b]))+",
            "((a|.))+",
            r"((a|\b))+",
            "((a|b))+(c)+",
            "(?<n>(a|b))+",
            r"((a|b))+\1",
            "((a|b))+|c",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (false, true, false),
                (true, true, false),
                (false, true, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(matcher) =
                    RegExpRepeatedCaptureMatcher::compile(&JsString::from(source), i, m, s)
                {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for input in [
                        "", "a", "b", "ab", "abab", "abb", "ababc", "xay", "xababy", "xy", "c",
                        "x\nab\n", "\n\r", "µΜµx", "ſSsx", "💩aaab",
                    ] {
                        let input = JsString::from(input);
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let result = matcher.find(&input, start, sticky).map(|r| {
                                let captures: Vec<_> = (0..matcher.capture_count())
                                    .map(|i| matcher.capture_range(i, &r))
                                    .collect();
                                (r, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{result:?}").unwrap();
                        }
                    }
                } else {
                    write!(rows, " unsupported").unwrap();
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn deep_repeated_choice_wrappers_have_linear_preparation_and_shared_captures() {
        let source = format!("{}(a|b){}+(c)|x", "(".repeat(100_000), ")".repeat(100_000));
        let matcher =
            crate::RegExpDisjunctionMatcher::compile(&JsString::from(source.as_str()), false)
                .unwrap();
        let input = JsString::from(format!("{}c", "a".repeat(200_000)).as_str());
        let (branch, range) = matcher.clone().find_branch(&input, 0, false).unwrap();
        assert_eq!(range, 0..200_001);
        assert_eq!(matcher.capture_count(), 100_002);
        for index in [0, 99_999, 100_000] {
            assert_eq!(
                matcher.capture_range(branch, index, &range),
                Some(199_999..200_000)
            );
        }
        assert_eq!(
            matcher.capture_range(branch, 100_001, &range),
            Some(200_000..200_001)
        );
        let empty = RegExpRepeatedCaptureMatcher::compile(
            &JsString::from("(((a|b))){0}(c)"),
            false,
            false,
            false,
        )
        .unwrap();
        let range = empty.find(&JsString::from("c"), 0, true).unwrap();
        for index in 0..3 {
            assert_eq!(empty.capture_range(index, &range), None);
        }
        assert_eq!(empty.capture_range(3, &range), Some(0..1));
    }

    #[test]
    fn repeated_literal_unit_choices_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a|b)+",
            "(a|b)+?",
            "(a|b)*",
            "(a|b){0}",
            "(a|b){1,2}",
            "(a|b){1,2}?",
            "(a|b)+(b)",
            "(a|b)+?(b)",
            "(a|b)*([ab]b)",
            "(x)((a|b)+)(y)",
            "((x)(a|b)+)(y)",
            "(x)((a|b)+(y))",
            "()(a|b)+()",
            "(())((a|b)*)()c()",
            "(?:a|b)+",
            "x(?:a|b)+(y)",
            r"(\x61|\u0062)+(c)",
            r"(\n|\r)+($)",
            r"(^)((a|b)+)($)(\n)",
            r"(\b)((a|b)+)(c)\b",
            "(µ|Μ)+(x)",
            "(ſ|S)+(x)",
            r"(\uD800|\uDC00)+(c)",
            r"(\uDCA9|a)+(b)",
            r"(\x08|\t)+(x)",
            r"(\(|\))+(x)",
            r"(\0|1)+(x)",
            "(a|a|b|a)+(c)",
            "(a|b){999999999999999999999999999999}(c)",
            "(a|b){0,999999999999999999999999999999}(c)",
            "(ab|a)+",
            "(a|)+",
            "(|a)+",
            "((a)|(b))+",
            "((a|b))+",
            "(a|[b])+",
            "(a|.)+",
            r"(a|\b)+",
            "(a|b)+(c)+",
            "(?<n>a|b)+",
            r"(a|b)+\1",
            "(a|b)+|c",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (false, true, false),
                (true, true, false),
                (false, true, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(matcher) =
                    RegExpRepeatedCaptureMatcher::compile(&JsString::from(source), i, m, s)
                {
                    write!(rows, " captures={}", matcher.capture_count()).unwrap();
                    for input in [
                        "", "a", "b", "ab", "abab", "abb", "ababc", "xay", "xababy", "xy", "c",
                        "x\nab\n", "\n\r", "µΜµx", "ſSsx", "((x", "\x001x", "\x08\tx", "💩aaab",
                    ] {
                        let input = JsString::from(input);
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let result = matcher.find(&input, start, sticky).map(|r| {
                                let captures: Vec<_> = (0..matcher.capture_count())
                                    .map(|i| matcher.capture_range(i, &r))
                                    .collect();
                                (r, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{result:?}").unwrap();
                        }
                    }
                } else {
                    write!(rows, " unsupported").unwrap();
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn large_unit_choices_partial_captures_clones_and_opted_in_work_are_safe() {
        let source = format!("a{}(a|b)+{}(ab)", "(".repeat(100_000), ")".repeat(100_000));
        let matcher = RegExpRepeatedCaptureMatcher::compile(
            &JsString::from(source.as_str()),
            false,
            false,
            false,
        )
        .unwrap();
        let input = JsString::from(format!("{}b", "a".repeat(200_000)).as_str());
        let range = matcher.clone().find(&input, 0, true).unwrap();
        assert_eq!(range, 0..200_001);
        assert_eq!(matcher.capture_count(), 100_002);
        assert_eq!(matcher.capture_range(99_999, &range), Some(1..199_999));
        assert_eq!(
            matcher.capture_range(100_000, &range),
            Some(199_998..199_999)
        );
        assert_eq!(
            matcher.capture_range(100_001, &range),
            Some(199_999..200_001)
        );
        assert_eq!(matcher.find(&input, usize::MAX, false), None);
        let flat = format!("({}b)+c", "a|".repeat(100_000));
        let flat = RegExpRepeatedCaptureMatcher::compile(
            &JsString::from(flat.as_str()),
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(flat.find(&JsString::from("abc"), 0, false), Some(0..3));
        assert_eq!(flat.capture_range(0, &(0..3)), Some(1..2));
        let duplicate = RegExpRepeatedCaptureMatcher::compile(
            &JsString::from("(a|a|b|a)+c"),
            false,
            false,
            false,
        )
        .unwrap();
        let failed = JsString::from(format!("{}x", "a".repeat(200_000)).as_str());
        assert_eq!(duplicate.find(&failed, 0, false), None);
        assert_eq!(duplicate.find(&failed, 0, true), None);
        assert_eq!(
            RegExpRepeatedCaptureMatcher::compile_with_work(
                &JsString::from("(a|b)+c"),
                false,
                false,
                false,
                |work| if work == 1024 { Err("host") } else { Ok(()) }
            )
            .unwrap_err(),
            "host"
        );
        assert!(
            RegExpRepeatedCaptureMatcher::compile_with_work(
                &JsString::from("(ab|a)+c"),
                false,
                false,
                false,
                |_| Err("unexpected")
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn partial_quantified_atom_captures_snapshot() {
        let mut rows = String::new();
        for source in [
            "(a+)b",
            "(a+?)a",
            "x(a+)y",
            "(x(a+))y",
            "(x)((a+)(y))",
            "(x(a*))(y)",
            "(x(a{0}))(y)",
            "(x(a{1,2}?))(a)",
            "(())(a*)()b()",
            "((a+))b",
            "((a*))b",
            "(([ab])+)([ab]b)",
            "([ab]+)([ab]b)",
            "([ab]+?)([ab]b)",
            "(x([ab]+))(y)",
            "(x(\\d+))(y)",
            "(x(\\s{1,2}))(y)",
            "(.+)([ab])",
            "(.+?)([ab])",
            "((µ+))(x)",
            r"(\x61+)(b)",
            r"(\u0061+)(b)",
            r"(\cA+)(b)",
            r"(\0+)()1",
            r"\0()1(a+)b",
            r"(\(+)(a)",
            r"(\b(a+))(b)",
            r"((^)(a+))($)(\n)",
            r"(a+($))(\n)",
            "💩(a+)(b)",
            r"\uDCA9(a+)(\uDCA9)",
            "(a{999999999999999999999999999999})(b)",
            "(a{0,999999999999999999999999999999})(b)",
            "(a+)(b)+",
            "(a+b+)c",
            "((a|b)+)c",
            r"(a+)\1",
            "(?<n>a+)b",
            "((?=a)a+)b",
            "(a+)b|c",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (false, true, false),
                (true, true, false),
                (false, true, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(m) =
                    RegExpRepeatedCaptureMatcher::compile(&JsString::from(source), i, m, s)
                {
                    write!(rows, " captures={}", m.capture_count()).unwrap();
                    for input in [
                        "",
                        "a",
                        "b",
                        "ab",
                        "aaab",
                        "aaaaab",
                        "xay",
                        "xaaay",
                        "xaaab",
                        "x12y",
                        "x  y",
                        "\x01\x01b",
                        "\x001aaab",
                        "\x00\x001",
                        "((a",
                        "x\naaaa\n",
                        "aaa\nb",
                        "µΜµx",
                        "💩aaab",
                    ] {
                        let input = JsString::from(input);
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let matched = m.find(&input, start, sticky).map(|r| {
                                let captures: Vec<_> = (0..m.capture_count())
                                    .map(|i| m.capture_range(i, &r))
                                    .collect();
                                (r, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{matched:?}").unwrap();
                        }
                    }
                } else {
                    write!(rows, " unsupported").unwrap();
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn long_quantified_atom_partial_ranges_decoded_widths_and_work_remain_safe() {
        let source = format!("a{}a+{}(ab)", "(".repeat(100_000), ")".repeat(100_000));
        let matcher = RegExpRepeatedCaptureMatcher::compile(
            &JsString::from(source.as_str()),
            false,
            false,
            false,
        )
        .unwrap();
        let input = JsString::from(format!("{}b", "a".repeat(200_000)).as_str());
        let range = matcher.clone().find(&input, 0, true).unwrap();
        assert_eq!(range, 0..200_001);
        assert_eq!(matcher.capture_count(), 100_001);
        for index in [0, 99_999] {
            assert_eq!(matcher.capture_range(index, &range), Some(1..199_999));
        }
        assert_eq!(
            matcher.capture_range(100_000, &range),
            Some(199_999..200_001)
        );
        assert_eq!(matcher.find(&input, usize::MAX, false), None);
        assert_eq!(
            RegExpRepeatedCaptureMatcher::compile_with_work(
                &JsString::from(r"x(\d+)y"),
                false,
                false,
                false,
                |work| if work == 65536 { Err("host") } else { Ok(()) }
            )
            .unwrap_err(),
            "host"
        );
        assert!(
            RegExpRepeatedCaptureMatcher::compile_with_work(
                &JsString::from("([ab])(a+b+)(c)"),
                false,
                false,
                false,
                |_| Err("unexpected")
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn deep_partial_layouts_long_runs_clones_and_optional_work_remain_safe() {
        let source = format!("a{}(aa)+{}(ab)", "(".repeat(100_000), ")".repeat(100_000));
        let matcher = RegExpRepeatedCaptureMatcher::compile(
            &JsString::from(source.as_str()),
            false,
            false,
            false,
        )
        .unwrap();
        let input = JsString::from(format!("{}b", "a".repeat(200_000)).as_str());
        let copy = matcher.clone();
        drop(matcher);
        let range = copy.find(&input, 0, false).unwrap();
        assert_eq!(range, 0..200_001);
        assert_eq!(copy.find(&input, 0, true), Some(range.clone()));
        assert_eq!(copy.capture_count(), 100_002);
        for index in [0, 99_999] {
            assert_eq!(copy.capture_range(index, &range), Some(1..199_999));
        }
        assert_eq!(copy.capture_range(100_000, &range), Some(199_997..199_999));
        assert_eq!(copy.capture_range(100_001, &range), Some(199_999..200_001));
        assert_eq!(copy.capture_range(100_002, &range), None);
        assert_eq!(copy.find(&input, usize::MAX, false), None);
        assert_eq!(copy.search_passes(true), 8);
        assert_eq!(copy.search_passes(false), 9);
        assert_eq!(
            RegExpRepeatedCaptureMatcher::compile_with_work(
                &JsString::from(r"(\d)((ab)+)(c)"),
                false,
                false,
                false,
                |work| if work == 65536 { Err("host") } else { Ok(()) }
            )
            .unwrap_err(),
            "host"
        );
        assert!(
            RegExpRepeatedCaptureMatcher::compile_with_work(
                &JsString::from("([ab])((ab|a)+)(c)"),
                false,
                false,
                false,
                |_| Err("unexpected")
            )
            .unwrap()
            .is_none()
        );
    }

    #[test]
    fn partial_repetition_captures_snapshot() {
        let mut rows = String::new();
        for source in [
            "((ab)+)c",
            "((ab)+?)ab",
            "((ab)*)c",
            "((ab){0})c",
            "x((ab)+)c",
            "((x)(ab)+)(c)",
            "(x)((ab)+(c))",
            "((x)((ab)+))(c)",
            "(x)(((ab)+)(c))",
            "(x)((ab){1,2}?)(ab)",
            "(())((ab)+)()c()",
            "x(()*)(a)",
            "x(()+)(a)",
            "x((){999999999999999999999999999999})(a)",
            "x((\\b\\B)*)(a)",
            "x((\\b\\B)+)(a)",
            "(x)((a())+)(b)",
            "(x)((a\\B)+)(b)",
            "(x)((^a$\\n){2})(a)",
            "(^)((a)(a())+)($)(\\n)",
            "((a)(aa)+)(ab)",
            "([ab])((([ab])([ab])()){2})([ab]b)()",
            "[ab](([ab][ab])+)([ab]b)",
            "(([ab][ab]){0,2}?)([ab]b)",
            "(.a)+([ab])",
            "(.)((.a)+)([ab])",
            "(µ)(([µ][µ])+)(x)",
            "💩((ab)+)(c)",
            r"\uDCA9((ab)+)(\uDCA9)",
            r"\0()1((ab)+)c",
            r"\c()a((ab)+)c",
            r"\((ab)+c\)",
            "(?:x(?:ab)+)(c)",
            "x(?:(ab)+)c",
            "(?:(x)(ab)+)(c)",
            "((ab|a)+)c",
            "((ab)+)(c)+",
            "((a*)+)b",
            "((ab)+a*)b",
            "(?<n>(ab)+)c",
            r"((ab)+)\1",
            "((?=a)ab)+c",
            "((ab)+)|c",
        ] {
            for (i, m, s) in [
                (false, false, false),
                (false, true, false),
                (true, true, false),
                (false, true, true),
            ] {
                write!(rows, "{source:?} i={i} m={m} s={s}").unwrap();
                if let Some(m) =
                    RegExpRepeatedCaptureMatcher::compile(&JsString::from(source), i, m, s)
                {
                    write!(rows, " captures={}", m.capture_count()).unwrap();
                    for input in [
                        "",
                        "a",
                        "x",
                        "ab",
                        "abc",
                        "abab",
                        "ababc",
                        "xabc",
                        "xababc",
                        "aaaaaab",
                        "aaab",
                        "x\naaaa\n",
                        "a\na\na",
                        "x\na\na\na",
                        "µΜµx",
                        "\x001ababc",
                        r"\caababc",
                        "💩ababc",
                        "💩abab",
                    ] {
                        let input = JsString::from(input);
                        for (start, sticky) in [
                            (0, false),
                            (1, false),
                            (0, true),
                            (1, true),
                            (input.len(), true),
                        ] {
                            let matched = m.find(&input, start, sticky).map(|r| {
                                let captures: Vec<_> = (0..m.capture_count())
                                    .map(|i| m.capture_range(i, &r))
                                    .collect();
                                (r, captures)
                            });
                            write!(rows, " {input:?}@{start}/{sticky}:{matched:?}").unwrap();
                        }
                    }
                } else {
                    write!(rows, " unsupported").unwrap();
                }
                rows.push('\n');
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn partial_enclosing_and_fixed_captures_keep_source_order_and_empty_iterations() {
        for (source, input, ranges) in [
            ("((ab)+)c", "ababc", vec![Some(0..4), Some(2..4)]),
            ("x((ab)+)c", "xababc", vec![Some(1..5), Some(3..5)]),
            (
                "((x)(ab)+)(c)",
                "xababc",
                vec![Some(0..5), Some(0..1), Some(3..5), Some(5..6)],
            ),
            (
                "(x)((ab)+(c))",
                "xababc",
                vec![Some(0..1), Some(1..6), Some(3..5), Some(5..6)],
            ),
            ("x((ab)*)(c)", "xc", vec![Some(1..1), None, Some(1..2)]),
            ("x(()*)(c)", "xc", vec![Some(1..1), None, Some(1..2)]),
            ("x(()+)(c)", "xc", vec![Some(1..1), Some(1..1), Some(1..2)]),
            ("(?:x(?:ab)+)(c)", "xababc", vec![Some(5..6)]),
            (
                "(())((ab)+)()c()",
                "ababc",
                vec![
                    Some(0..0),
                    Some(0..0),
                    Some(0..4),
                    Some(2..4),
                    Some(4..4),
                    Some(5..5),
                ],
            ),
            (
                r"\0()1((ab)+)c",
                "\x001ababc",
                vec![Some(1..1), Some(2..6), Some(4..6)],
            ),
        ] {
            let matcher =
                RegExpRepeatedCaptureMatcher::compile(&JsString::from(source), false, false, false)
                    .unwrap();
            let input = JsString::from(input);
            let range = matcher.find(&input, 0, false).unwrap();
            assert_eq!(range, 0..input.len(), "{source}");
            assert_eq!(matcher.capture_count(), ranges.len(), "{source}");
            for (index, expected) in ranges.into_iter().enumerate() {
                assert_eq!(
                    matcher.capture_range(index, &range),
                    expected,
                    "{source}#{index}"
                );
            }
        }
    }
    #[test]
    fn partial_ranges_agree_with_independent_candidate_count_and_capture_order() {
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
                    "((([ab])(([ab])([ab])()){quantifier}{})([ab]b))()",
                    if greedy { "" } else { "?" }
                );
                for anchored in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpRepeatedCaptureMatcher::compile(
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
                                                    captures.insert(0, Some(candidate..tail));
                                                    captures.insert(0, Some(candidate..finish));
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
    fn partial_atom_ranges_agree_with_independent_candidate_count_and_capture_order() {
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
                    "((([ab])[ab]{quantifier}{})([ab]b))()",
                    if greedy { "" } else { "?" }
                );
                for anchored in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpRepeatedCaptureMatcher::compile(
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
                                                    .unwrap_or(units.len() - body)
                                                    .min(units.len() - body);
                                                let mut counts: Vec<_> = (min..=limit).collect();
                                                if greedy {
                                                    counts.reverse();
                                                }
                                                counts.into_iter().find_map(|count| {
                                                    let tail = body + count;
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
                                                        Some(tail..finish),
                                                        Some(finish..finish),
                                                    ];
                                                    captures.insert(0, Some(candidate..tail));
                                                    captures.insert(0, Some(candidate..finish));
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
    fn unit_choice_ranges_agree_with_independent_candidate_count_and_capture_order() {
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
                    "((([ab])(a|b){quantifier}{})([ab]b))()",
                    if greedy { "" } else { "?" }
                );
                for anchored in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpRepeatedCaptureMatcher::compile(
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
                                                    .unwrap_or(units.len() - body)
                                                    .min(units.len() - body);
                                                let mut counts: Vec<_> = (min..=limit).collect();
                                                if greedy {
                                                    counts.reverse();
                                                }
                                                counts.into_iter().find_map(|count| {
                                                    let tail = body + count;
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
                                                        (count > 0).then(|| tail - 1..tail),
                                                        Some(tail..finish),
                                                        Some(finish..finish),
                                                    ];
                                                    captures.insert(0, Some(candidate..tail));
                                                    captures.insert(0, Some(candidate..finish));
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
    fn enclosing_choice_ranges_agree_with_independent_candidate_count_and_capture_order() {
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
                    "((([ab])((a|b)){quantifier}{})([ab]b))()",
                    if greedy { "" } else { "?" }
                );
                for anchored in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpRepeatedCaptureMatcher::compile(
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
                                                    .unwrap_or(units.len() - body)
                                                    .min(units.len() - body);
                                                let mut counts: Vec<_> = (min..=limit).collect();
                                                if greedy {
                                                    counts.reverse();
                                                }
                                                counts.into_iter().find_map(|count| {
                                                    let tail = body + count;
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
                                                        (count > 0).then(|| tail - 1..tail),
                                                        (count > 0).then(|| tail - 1..tail),
                                                        Some(tail..finish),
                                                        Some(finish..finish),
                                                    ];
                                                    captures.insert(0, Some(candidate..tail));
                                                    captures.insert(0, Some(candidate..finish));
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
}
