//! Unicode literal/dot/escape atoms and assertions (22.2.2.4, 22.2.2.7.1).

use crate::{
    JsString,
    case_data::SIMPLE_CASE_FOLD,
    is_line_terminator, is_whitespace, regexp_canonicalize_character,
    regexp_literal::{unicode_literal_atom, unicode_start},
};
use std::ops::Range;

/// Immutable matcher for one Unicode atom or zero-width assertion.
///
/// The caller must validate the complete Pattern with `u` or `v`. Existing bare
/// and assertion entry points require no i; flag-aware entry points also admit i.
/// Dot and `\d`, `\D`, `\s`, `\S`, `\w`, `\W` are the complete admitted bodies.
/// Optional leading ^ and/or trailing $ are admitted by the assertion entry points.
/// The separate literal entry point admits one decoded literal atom. Classes,
/// groups, concatenations and quantifiers remain separate.
/// Matching consumes complete code points, including isolated surrogates, while
/// successful ranges retain UTF-16 indices. Separate standalone word/line
/// assertion entry points consume no characters. No input-sized storage is allocated.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeCharacterMatcher {
    atom: Atom,
    start_anchor: bool,
    end_anchor: bool,
    multiline: bool,
    fold_input: bool,
    start_word: Option<bool>,
    end_word: Option<bool>,
    word_ignore_case: bool,
}

#[derive(Clone, Copy, Debug)]
enum Atom {
    Dot(bool),
    Escape(u16),
    Literal(u32),
    Empty,
}

impl RegExpUnicodeCharacterMatcher {
    /// Compiles exactly one admitted atom from a completely validated u/v Pattern.
    pub fn compile(source: &JsString, dot_all: bool) -> Option<Self> {
        Self::compile_with_work(source, dot_all, |_| Ok::<(), std::convert::Infallible>(()))
            .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same subset with fallible opt-in construction work accounting.
    ///
    /// The atom occupies constant storage; work is charged before creating it.
    /// Rejected syntax and an independent host charge failure remain distinct.
    pub fn compile_with_work<E>(
        source: &JsString,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(atom) = parse_atom(source.code_units(), dot_all) else {
            return Ok(None);
        };
        charge(source.len())?;
        Ok(Some(Self {
            atom,
            start_anchor: false,
            end_anchor: false,
            multiline: false,
            fold_input: false,
            start_word: None,
            end_word: None,
            word_ignore_case: false,
        }))
    }

    /// Compiles one admitted atom with an optional leading ^ and/or trailing $.
    ///
    /// Complete u/v validation and case-sensitive matching remain prerequisites.
    /// Multiline assertions inspect the preceding/following complete character;
    /// non-multiline $ succeeds only at the actual input end (22.2.2.4).
    pub fn compile_with_assertions(
        source: &JsString,
        multiline: bool,
        dot_all: bool,
    ) -> Option<Self> {
        Self::compile_with_assertions_and_work(source, multiline, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same anchored subset with fallible opt-in construction work.
    pub fn compile_with_assertions_and_work<E>(
        source: &JsString,
        multiline: bool,
        dot_all: bool,
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_flags_and_work(source, false, multiline, dot_all, charge)
    }

    /// Compiles one dot/escape atom with optional assertions and Unicode i.
    ///
    /// The complete Pattern must be validated in u/v mode. Unicode word escapes
    /// use pinned simple/common case folding; dot, digit and space sets are
    /// invariant under folding. Full and Turkic folding do not apply.
    pub fn compile_with_flags(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
    ) -> Option<Self> {
        Self::compile_with_flags_and_work(source, ignore_case, multiline, dot_all, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same flag-aware subset with fallible opt-in work accounting.
    pub fn compile_with_flags_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let Some(body) =
            units.get(usize::from(start_anchor)..units.len() - usize::from(end_anchor))
        else {
            return Ok(None);
        };
        let Some(atom) = parse_atom(body, dot_all) else {
            return Ok(None);
        };
        charge(source.len())?;
        Ok(Some(Self {
            atom,
            start_anchor,
            end_anchor,
            multiline,
            fold_input: ignore_case && matches!(atom, Atom::Escape(0x77 | 0x57)),
            start_word: None,
            end_word: None,
            word_ignore_case: false,
        }))
    }

    /// Compiles one literal Unicode atom with optional boundary assertions.
    ///
    /// Complete u/v validation is required. This entry point shares atom decoding
    /// with Unicode literal concatenations, while i uses pinned simple/common
    /// folding and consumes the original input width (22.2.2.7.3). Bare dot/escape
    /// APIs keep their original subset. Groups and concatenations are excluded.
    pub fn compile_literal_with_flags(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
    ) -> Option<Self> {
        Self::compile_literal_with_flags_and_work(source, ignore_case, multiline, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same literal subset with fallible opt-in work accounting.
    pub fn compile_literal_with_flags_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        // Braced escapes may contain source-sized runs of leading zeroes.
        // Charge before decoding, including rejected literal bodies.
        charge(source.len())?;
        let units = source.code_units();
        let start_anchor = units.first() == Some(&0x5e);
        let mut index = usize::from(start_anchor);
        let Some(&first) = units.get(index) else {
            return Ok(None);
        };
        index += 1;
        let Some(value) = unicode_literal_atom(units, &mut index, first) else {
            return Ok(None);
        };
        let end_anchor = units.get(index) == Some(&0x24);
        index += usize::from(end_anchor);
        if index != units.len() {
            return Ok(None);
        }
        let value = if ignore_case {
            charge(2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize)?;
            regexp_canonicalize_character(value, true, true)
        } else {
            value
        };
        Ok(Some(Self {
            atom: Atom::Literal(value),
            start_anchor,
            end_anchor,
            multiline,
            fold_input: ignore_case,
            start_word: None,
            end_word: None,
            word_ignore_case: false,
        }))
    }

    /// Compiles optional word assertions around one Unicode literal/dot/escape atom.
    ///
    /// Requires complete u/v validation and at least one leading or trailing
    /// \b/\B. Optional ^/$ retain m context. Atom and neighbor folding are
    /// independent: dot/digit/space atoms do not need folding, but their word
    /// assertions still include the Unicode i aliases (22.2.2.4, 22.2.2.7.1).
    /// Groups and other compositions remain caller proofs.
    pub fn compile_with_word_boundaries_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        dot_all: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let matcher = Self::compile_with_flags_and_work(
            &assertions.source,
            ignore_case,
            multiline,
            dot_all,
            &mut charge,
        )?;
        let matcher = match matcher {
            Some(matcher) => Some(matcher),
            None => Self::compile_literal_with_flags_and_work(
                &assertions.source,
                ignore_case,
                multiline,
                charge,
            )?,
        };
        Ok(matcher.map(|mut matcher| {
            matcher.start_word = assertions.start;
            matcher.end_word = assertions.end;
            matcher.word_ignore_case = ignore_case;
            matcher
        }))
    }

    /// Compiles one zero-width Unicode word assertion, with optional ^/$ and i/m.
    ///
    /// Complete u/v validation is required. Bare and consuming-atom entry points
    /// retain their subsets. Repeated assertions and other bodies remain separate.
    pub fn compile_word_assertion_with_flags_and_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len())?;
        let units = source.code_units();
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let Some(body) =
            units.get(usize::from(start_anchor)..units.len() - usize::from(end_anchor))
        else {
            return Ok(None);
        };
        let [0x5c, kind @ (0x62 | 0x42)] = body else {
            return Ok(None);
        };
        Ok(Some(Self {
            atom: Atom::Empty,
            start_anchor,
            end_anchor,
            multiline,
            fold_input: false,
            start_word: Some(*kind == 0x62),
            end_word: None,
            word_ignore_case: ignore_case,
        }))
    }

    /// Compiles standalone Unicode ^, $ or ^$ assertions, with optional m.
    ///
    /// Requires complete u/v validation. IgnoreCase does not affect line
    /// assertions. Other entry points and Pattern compositions retain their own
    /// subsets (22.2.2.4).
    pub fn compile_line_assertions_with_work<E>(
        source: &JsString,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len())?;
        let (start_anchor, end_anchor) = match source.code_units() {
            [0x5e] => (true, false),
            [0x24] => (false, true),
            [0x5e, 0x24] => (true, true),
            _ => return Ok(None),
        };
        Ok(Some(Self {
            atom: Atom::Empty,
            start_anchor,
            end_anchor,
            multiline,
            fold_input: false,
            start_word: None,
            end_word: None,
            word_ignore_case: false,
        }))
    }

    /// Compiles a complete sequence of at least two Unicode ^/$/\b/\B assertions.
    ///
    /// Complete u/v validation is required. All assertions inspect the same
    /// unchanged position, so repeated conditions can be deduplicated. Both
    /// opposite word assertions retain a failing conjunction (22.2.2.4).
    /// Captures, lookaround and consuming bodies remain separate caller proofs.
    pub fn compile_assertion_sequence_with_work<E>(
        source: &JsString,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len())?;
        let units = source.code_units();
        let mut index = 0;
        let mut count = 0;
        let mut start_anchor = false;
        let mut end_anchor = false;
        let mut positive_word = false;
        let mut negative_word = false;
        while let Some(&unit) = units.get(index) {
            match unit {
                0x5e => {
                    start_anchor = true;
                    index += 1;
                }
                0x24 => {
                    end_anchor = true;
                    index += 1;
                }
                0x5c => {
                    match units.get(index + 1) {
                        Some(0x62) => positive_word = true,
                        Some(0x42) => negative_word = true,
                        _ => return Ok(None),
                    }
                    index += 2;
                }
                _ => return Ok(None),
            }
            count += 1;
        }
        if count < 2 {
            return Ok(None);
        }
        Ok(Some(Self {
            atom: Atom::Empty,
            start_anchor,
            end_anchor,
            multiline,
            fold_input: false,
            start_word: positive_word.then_some(true),
            end_word: negative_word.then_some(false),
            word_ignore_case: ignore_case,
        }))
    }

    /// Bounds the input scan and each optional assertion check for opt-in work.
    pub fn search_passes(&self) -> usize {
        1 + usize::from(self.start_anchor)
            + usize::from(self.end_anchor)
            + 8 * (usize::from(self.start_word.is_some()) + usize::from(self.end_word.is_some()))
            + if self.fold_input {
                // Bound pinned-table binary search, including final membership.
                2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize
            } else {
                0
            }
    }

    /// Finds the first complete matching character, or only the sticky character.
    ///
    /// An initial offset inside a pair uses its leading boundary, following the
    /// approved Node/V8 resolution of the edition-17 inconsistency (22.2.7.2).
    /// An offset at/past the input end cannot match a consuming atom. Zero-width
    /// assertions can match the input end without advancing lastIndex.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let units = input.code_units();
        let mut cursor = unicode_start(input, start)?;
        loop {
            let (value, width) = if matches!(self.atom, Atom::Empty) {
                (0, 0)
            } else {
                let &first = units.get(cursor)?;
                unicode_input_character(units, cursor, first)
            };
            let end = cursor + width;
            if unicode_assertions_match(
                units,
                cursor,
                end,
                self.start_anchor,
                self.end_anchor,
                self.multiline,
            ) && self.start_word.is_none_or(|expected| {
                unicode_word_boundary(units, cursor, self.word_ignore_case) == expected
            }) && self.end_word.is_none_or(|expected| {
                unicode_word_boundary(units, end, self.word_ignore_case) == expected
            }) && self.matches(value)
            {
                return Some(cursor..end);
            }
            if sticky {
                return None;
            }
            // Unicode search candidates are complete-character boundaries
            // (22.2.7.2), including for standalone zero-width assertions.
            cursor += if width == 0 {
                let &first = units.get(cursor)?;
                unicode_input_character(units, cursor, first).1
            } else {
                width
            };
        }
    }

    fn matches(&self, value: u32) -> bool {
        let value = if self.fold_input {
            regexp_canonicalize_character(value, true, true)
        } else {
            value
        };
        match self.atom {
            Atom::Empty => true,
            Atom::Literal(expected) => value == expected,
            Atom::Dot(all) => all || !char::from_u32(value).is_some_and(is_line_terminator),
            Atom::Escape(kind) => {
                let included = match kind | 0x20 {
                    0x64 => (0x30..=0x39).contains(&value),
                    0x77 => matches!(value, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a),
                    0x73 => char::from_u32(value)
                        .is_some_and(|c| is_whitespace(c) || is_line_terminator(c)),
                    _ => unreachable!("validated character escape"),
                };
                included != (kind < 0x60)
            }
        }
    }
}

pub(crate) struct UnicodeWordAssertions {
    pub(crate) source: JsString,
    pub(crate) start: Option<bool>,
    pub(crate) end: Option<bool>,
}

// Strips only optional outside word assertions. A caller must subsequently prove
// the complete retained body; line assertions keep their original order/context.
pub(crate) fn unicode_word_assertions_with_work<E>(
    source: &JsString,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<UnicodeWordAssertions>, E> {
    let units = source.code_units();
    charge(units.len().saturating_mul(2))?;
    let start_anchor = units.first() == Some(&0x5e);
    let end_anchor = units.last() == Some(&0x24);
    let mut begin = usize::from(start_anchor);
    let mut end = units.len() - usize::from(end_anchor);
    let start_word = match units.get(begin..end).and_then(|body| body.get(..2)) {
        Some([0x5c, kind @ (0x62 | 0x42)]) => {
            begin += 2;
            Some(*kind == 0x62)
        }
        _ => None,
    };
    let end_word = if end >= begin + 2 {
        match &units[end - 2..end] {
            [0x5c, kind @ (0x62 | 0x42)]
                if units[begin..end - 1]
                    .iter()
                    .rev()
                    .take_while(|&&unit| unit == 0x5c)
                    .count()
                    % 2
                    == 1 =>
            {
                end -= 2;
                Some(*kind == 0x62)
            }
            _ => None,
        }
    } else {
        None
    };
    if start_word.is_none() && end_word.is_none() {
        return Ok(None);
    }
    let length = end - begin + usize::from(start_anchor) + usize::from(end_anchor);
    charge(length)?;
    let mut body = Vec::with_capacity(length);
    if start_anchor {
        body.push(0x5e);
    }
    body.extend_from_slice(&units[begin..end]);
    if end_anchor {
        body.push(0x24);
    }
    Ok(Some(UnicodeWordAssertions {
        source: JsString::from_code_units(body),
        start: start_word,
        end: end_word,
    }))
}

// WordCharacters adds only the two pinned non-ASCII aliases with Unicode i.
// Complete-character endpoints mean raw neighboring units suffice: surrogate
// units cannot be word characters (22.2.2.4, 22.2.2.7.1).
pub(crate) fn unicode_word_boundary(units: &[u16], at: usize, ignore_case: bool) -> bool {
    let word = |unit: Option<&u16>| {
        unit.is_some_and(|&unit| {
            matches!(unit, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a)
                || (ignore_case && matches!(unit, 0x17f | 0x212a))
        })
    };
    word(at.checked_sub(1).and_then(|position| units.get(position))) != word(units.get(at))
}

// Caller supplies a complete consuming-character range within the input.
pub(crate) fn unicode_assertions_match(
    units: &[u16],
    start: usize,
    end: usize,
    start_anchor: bool,
    end_anchor: bool,
    multiline: bool,
) -> bool {
    let start_ok = !start_anchor
        || start == 0
        || (multiline
            && char::from_u32(u32::from(units[start - 1])).is_some_and(is_line_terminator));
    let end_ok = !end_anchor
        || end == units.len()
        || (multiline && char::from_u32(u32::from(units[end])).is_some_and(is_line_terminator));
    start_ok && end_ok
}

// Caller supplies the in-bounds first unit at a complete-character boundary.
pub(crate) fn unicode_input_character(units: &[u16], cursor: usize, first: u16) -> (u32, usize) {
    if (0xd800..=0xdbff).contains(&first)
        && units
            .get(cursor + 1)
            .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
    {
        (
            0x10000 + ((u32::from(first) - 0xd800) << 10) + u32::from(units[cursor + 1]) - 0xdc00,
            2,
        )
    } else {
        (u32::from(first), 1)
    }
}

fn parse_atom(units: &[u16], dot_all: bool) -> Option<Atom> {
    match units {
        [0x2e] => Some(Atom::Dot(dot_all)),
        [0x5c, kind @ (0x64 | 0x44 | 0x73 | 0x53 | 0x77 | 0x57)] => Some(Atom::Escape(*kind)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_literal_character_folding_ranges_snapshot() {
        let input = JsString::from_code_units(vec![
            0xd83d, 0xde00, 0x17f, 0x212a, 0x41, 0x61, 0x24, 0x5e, 0x2e, 0xa, 0xd800, 0xdc01,
            0xd800,
        ]);
        let mut rows = String::new();
        for body in [
            "a",
            "A",
            "s",
            "k",
            "ſ",
            "K",
            r"\$",
            r"\^",
            r"\.",
            r"\u{1f600}",
            r"\uD83D\uDE00",
            r"\uD800",
            r"\u{10400}",
            "ß",
        ] {
            for (left, right) in [("", ""), ("^", "$")] {
                let source = JsString::from(format!("{left}{body}{right}").as_str());
                for ignore_case in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpUnicodeCharacterMatcher::compile_literal_with_flags(
                            &source,
                            ignore_case,
                            multiline,
                        )
                        .unwrap();
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                writeln!(rows,"{source:?} ignore_case={ignore_case} multiline={multiline} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_literal_character_folds_keep_independent_sets_and_original_widths() {
        for (source, members) in [
            ("A", &[0x41, 0x61][..]),
            ("ß", &[0xdf, 0x1e9e, 0x1df95][..]),
            (r"\u{10400}", &[0x10400, 0x10428][..]),
        ] {
            let matcher = RegExpUnicodeCharacterMatcher::compile_literal_with_flags(
                &JsString::from(source),
                true,
                false,
            )
            .unwrap();
            for value in 0..=0x10ffff {
                let units = if value <= 0xffff {
                    vec![value as u16]
                } else {
                    char::from_u32(value)
                        .unwrap()
                        .encode_utf16(&mut [0; 2])
                        .to_vec()
                };
                let len = units.len();
                let input = JsString::from_code_units(units);
                let expected = members.contains(&value).then_some(0..len);
                assert_eq!(
                    matcher.find(&input, 0, true),
                    expected,
                    "{source} U+{value:x}"
                );
                if len == 2 {
                    assert_eq!(
                        matcher.find(&input, 1, true),
                        expected,
                        "interior {source} U+{value:x}"
                    );
                }
            }
        }
        for &(source, target) in SIMPLE_CASE_FOLD {
            let point = |value: u32| {
                JsString::from_code_units(if value <= 0xffff {
                    vec![value as u16]
                } else {
                    char::from_u32(value)
                        .unwrap()
                        .encode_utf16(&mut [0; 2])
                        .to_vec()
                })
            };
            let pattern = point(source);
            let input = point(target);
            let matcher =
                RegExpUnicodeCharacterMatcher::compile_literal_with_flags(&pattern, true, false)
                    .unwrap();
            assert_eq!(
                matcher.find(&input, 0, true),
                Some(0..input.len()),
                "U+{source:x} -> U+{target:x}"
            );
            let matcher =
                RegExpUnicodeCharacterMatcher::compile_literal_with_flags(&input, true, false)
                    .unwrap();
            assert_eq!(
                matcher.find(&pattern, 0, true),
                Some(0..pattern.len()),
                "reverse U+{target:x}"
            );
        }
    }

    #[test]
    fn unicode_literal_character_contracts_clones_and_fallible_work() {
        for source in [
            "",
            ".",
            r"\w",
            "[a]",
            "(a)",
            "ab",
            "a+",
            "a|b",
            r"\uD800()\uDC00",
            r"\u{D800}\u{DC00}",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_literal_with_flags(
                    &JsString::from(source),
                    true,
                    false
                )
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeCharacterMatcher::compile_with_flags(
                &JsString::from("a"),
                true,
                false,
                false
            )
            .is_none()
        );
        for (source, anchored) in [
            (r"\$", false),
            (r"^\$$", true),
            (r"\\$", true),
            (r"\u0024", false),
        ] {
            let matcher = RegExpUnicodeCharacterMatcher::compile_literal_with_flags(
                &JsString::from(source),
                true,
                false,
            )
            .unwrap();
            assert_eq!(
                matcher.search_passes(),
                14 + usize::from(anchored) + usize::from(source.starts_with('^'))
            );
        }
        let source = JsString::from(r"^\u{00000061}$");
        let mut charges = Vec::new();
        let matcher = RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
            &source,
            true,
            true,
            |n| {
                charges.push(n);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(charges, vec![source.len(), 13]);
        assert_eq!(
            matcher.clone().find(&JsString::from("A"), 0, true),
            Some(0..1)
        );
        for failure in 0..charges.len() {
            let mut calls = 0;
            let result = RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
                &source,
                true,
                true,
                |_| {
                    let call = calls;
                    calls += 1;
                    if call == failure { Err(17) } else { Ok(()) }
                },
            );
            assert!(matches!(result, Err(17)));
            assert_eq!(calls, failure + 1);
        }
        let source = JsString::from(format!("\\u{{{}61}}", "0".repeat(100000)).as_str());
        assert!(matches!(
            RegExpUnicodeCharacterMatcher::compile_literal_with_flags_and_work(
                &source,
                true,
                false,
                |_| Err(17)
            ),
            Err(17)
        ));
        assert_eq!(
            RegExpUnicodeCharacterMatcher::compile_literal_with_flags(&source, true, false)
                .unwrap()
                .find(&JsString::from("A"), 0, true),
            Some(0..1)
        );
    }
    use std::fmt::Write;

    const PLANS: [(&str, bool); 8] = [
        (".", false),
        (".", true),
        (r"\d", false),
        (r"\D", false),
        (r"\s", false),
        (r"\S", false),
        (r"\w", false),
        (r"\W", false),
    ];

    fn reference_membership(source: &str, all: bool, value: u32) -> bool {
        let space = matches!(value, 0x9..=0xd | 0x20 | 0xa0 | 0x1680 | 0x2000..=0x200a
            | 0x2028..=0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff);
        let word = matches!(value, 0x30..=0x39 | 0x41..=0x5a | 0x5f | 0x61..=0x7a);
        let digit = (0x30..=0x39).contains(&value);
        match source {
            "." => all || !matches!(value, 0xa | 0xd | 0x2028 | 0x2029),
            r"\d" => digit,
            r"\D" => !digit,
            r"\s" => space,
            r"\S" => !space,
            r"\w" => word,
            r"\W" => !word,
            _ => unreachable!(),
        }
    }

    #[test]
    fn unicode_dot_and_escape_ranges_snapshot() {
        let input = JsString::from_code_units(vec![
            0xd83d, 0xde00, 0xd800, 0x61, 0xa, 0x2028, 0xdc00, 0x39, 0xfeff, 0x5f,
        ]);
        let mut rows = String::new();
        for (source, all) in PLANS {
            let matcher =
                RegExpUnicodeCharacterMatcher::compile(&JsString::from(source), all).unwrap();
            for start in 0..=input.len() + 1 {
                for sticky in [false, true] {
                    writeln!(
                        rows,
                        "{source:?} dotAll={all} start={start} sticky={sticky} {:?}",
                        matcher.find(&input, start, sticky)
                    )
                    .unwrap();
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn every_unicode_code_point_agrees_with_character_escape_definitions() {
        let plans = PLANS.map(|(source, all)| {
            (
                source,
                all,
                RegExpUnicodeCharacterMatcher::compile(&JsString::from(source), all).unwrap(),
            )
        });
        for value in 0..=0x10ffff {
            let units = if value <= 0xffff {
                vec![u16::try_from(value).unwrap()]
            } else {
                let mut buffer = [0; 2];
                char::from_u32(value)
                    .unwrap()
                    .encode_utf16(&mut buffer)
                    .to_vec()
            };
            let input = JsString::from_code_units(units);
            for (source, all, matcher) in &plans {
                let expected = reference_membership(source, *all, value).then_some(0..input.len());
                assert_eq!(
                    matcher.find(&input, 0, true),
                    expected,
                    "{source} {value:x}"
                );
                if input.len() == 2 {
                    assert_eq!(
                        matcher.find(&input, 1, true),
                        expected,
                        "inside pair {source} {value:x}"
                    );
                }
            }
        }
    }

    #[test]
    fn unicode_character_search_agrees_with_independent_utf16_decoding() {
        let alphabet = [
            0x61, 0x39, 0x20, 0xa, 0x2028, 0xd800, 0xdbff, 0xdc00, 0xdfff,
        ];
        let plans = PLANS.map(|(source, all)| {
            (
                source,
                all,
                RegExpUnicodeCharacterMatcher::compile(&JsString::from(source), all).unwrap(),
            )
        });
        for length in 0..=4 {
            for mut ordinal in 0..alphabet.len().pow(length) {
                let units: Vec<u16> = (0..length)
                    .map(|_| {
                        let unit = alphabet[ordinal % alphabet.len()];
                        ordinal /= alphabet.len();
                        unit
                    })
                    .collect();
                let mut cursor = 0;
                let decoded: Vec<_> = char::decode_utf16(units.iter().copied())
                    .map(|decoded| {
                        let (value, width) = match decoded {
                            Ok(c) => (u32::from(c), c.len_utf16()),
                            Err(e) => (u32::from(e.unpaired_surrogate()), 1),
                        };
                        let range = cursor..cursor + width;
                        cursor += width;
                        (value, range)
                    })
                    .collect();
                let input = JsString::from_code_units(units);
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        for (source, all, matcher) in &plans {
                            let candidates = decoded.iter().filter(|(_, range)| range.end > start);
                            let expected = candidates
                                .take(if sticky { 1 } else { decoded.len() })
                                .find(|(value, _)| reference_membership(source, *all, *value))
                                .map(|(_, range)| range.clone());
                            assert_eq!(
                                matcher.find(&input, start, sticky),
                                expected,
                                "{input:?} {source} dotAll={all} start={start} sticky={sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_character_compilation_contracts_and_opted_work() {
        for source in [
            "",
            "a",
            "..",
            "(.)",
            "[a]",
            "[^a]",
            r"\p{ASCII}",
            r"\b",
            r"\.",
            r"\d+",
            r"\D|a",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile(&JsString::from(source), false).is_none()
            );
        }
        for (source, all) in PLANS {
            let source = JsString::from(source);
            let mut charged = Vec::new();
            let matcher = RegExpUnicodeCharacterMatcher::compile_with_work(&source, all, |work| {
                charged.push(work);
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
            assert_eq!(charged, vec![source.len()]);
            assert!(
                RegExpUnicodeCharacterMatcher::compile_with_work(&source, all, |_| Err(17))
                    .is_err()
            );
            let input = JsString::from("😀");
            assert_eq!(
                matcher.clone().find(&input, 1, true),
                matcher.find(&input, 1, true)
            );
            assert!(matcher.find(&input, input.len(), false).is_none());
            assert!(matcher.find(&input, usize::MAX, false).is_none());
        }
    }
    #[test]
    fn unicode_character_assertion_ranges_snapshot() {
        let mut rows = String::new();
        for source in ["^.$", "^.", ".$", r"^\D$", r"\W$", r"^\s", r"^\d$"] {
            for multiline in [false, true] {
                for all in [false, true] {
                    let matcher = RegExpUnicodeCharacterMatcher::compile_with_assertions(
                        &JsString::from(source),
                        multiline,
                        all,
                    )
                    .unwrap();
                    for text in ["", "😀", "a\n", "\na\r\n😀\u{2028}", "9", "\u{2029}x"] {
                        let input = JsString::from(text);
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                writeln!(rows, "{source:?} multiline={multiline} dotAll={all} input={input:?} start={start} sticky={sticky} {:?}", matcher.find(&input, start, sticky)).unwrap();
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_character_assertions_agree_with_independent_character_contexts() {
        let alphabet = [0x61, 0x39, 0xa, 0xd, 0x2028, 0xd800, 0xdc00];
        let mut plans = Vec::new();
        for (source, body) in [
            ("^.$", "."),
            ("^.", "."),
            (".$", "."),
            (r"^\D$", r"\D"),
            (r"\W$", r"\W"),
            (r"^\s", r"\s"),
            (r"^\d$", r"\d"),
        ] {
            for multiline in [false, true] {
                for all in [false, true] {
                    plans.push((
                        source,
                        body,
                        multiline,
                        all,
                        RegExpUnicodeCharacterMatcher::compile_with_assertions(
                            &JsString::from(source),
                            multiline,
                            all,
                        )
                        .unwrap(),
                    ));
                }
            }
        }
        for length in 0..=4 {
            for mut ordinal in 0..alphabet.len().pow(length) {
                let units: Vec<u16> = (0..length)
                    .map(|_| {
                        let unit = alphabet[ordinal % alphabet.len()];
                        ordinal /= alphabet.len();
                        unit
                    })
                    .collect();
                let mut cursor = 0;
                let decoded: Vec<_> = char::decode_utf16(units.iter().copied())
                    .map(|decoded| {
                        let (value, width) = match decoded {
                            Ok(c) => (u32::from(c), c.len_utf16()),
                            Err(e) => (u32::from(e.unpaired_surrogate()), 1),
                        };
                        let range = cursor..cursor + width;
                        cursor += width;
                        (value, range)
                    })
                    .collect();
                let input = JsString::from_code_units(units);
                let line = |value: u32| matches!(value, 0xa | 0xd | 0x2028 | 0x2029);
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        for (source, body, multiline, all, matcher) in &plans {
                            let candidates = decoded
                                .iter()
                                .enumerate()
                                .filter(|(_, (_, range))| range.end > start);
                            let expected = candidates
                                .take(if sticky { 1 } else { decoded.len() })
                                .find(|(i, (value, range))| {
                                    reference_membership(body, *all, *value)
                                        && (!source.starts_with('^')
                                            || *i == 0
                                            || (*multiline && line(decoded[i - 1].0)))
                                        && (!source.ends_with('$')
                                            || range.end == input.len()
                                            || (*multiline && line(decoded[i + 1].0)))
                                })
                                .map(|(_, (_, range))| range.clone());
                            assert_eq!(
                                matcher.find(&input, start, sticky),
                                expected,
                                "{input:?} {source} multiline={multiline} dotAll={all} start={start} sticky={sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_character_assertion_compilation_and_optional_work_contracts() {
        for source in [
            "", "^", "$", "^$", "^^.$", "^.$$", "^a$", "^(.)$", r"^\D+$", "^[a]$",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_with_assertions(
                    &JsString::from(source),
                    true,
                    true
                )
                .is_none()
            );
        }
        for source in ["^.$", "^.", ".$", r"^\D$", r"\W$", r"^\s"] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile(&JsString::from(source), true).is_none()
            );
            let mut charges = Vec::new();
            let matcher = RegExpUnicodeCharacterMatcher::compile_with_assertions_and_work(
                &JsString::from(source),
                true,
                true,
                |work| {
                    charges.push(work);
                    Ok::<(), ()>(())
                },
            )
            .unwrap()
            .unwrap();
            assert_eq!(charges, vec![source.len()]);
            assert_eq!(
                matcher.search_passes(),
                1 + usize::from(source.starts_with('^')) + usize::from(source.ends_with('$'))
            );
            assert!(
                RegExpUnicodeCharacterMatcher::compile_with_assertions_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    |_| Err(17)
                )
                .is_err()
            );
        }
    }
    #[test]
    fn unicode_character_folding_ranges_snapshot() {
        let input = JsString::from_code_units(vec![
            0xd83d, 0xde00, 0x17f, 0x212a, 9, 0x39, 0x41, 0xfeff, 0xd800, 0xa, 0xdc00,
        ]);
        let mut rows = String::new();
        for (body, all) in PLANS {
            for (left, right) in [("", ""), ("^", ""), ("", "$"), ("^", "$")] {
                let source = JsString::from(format!("{left}{body}{right}").as_str());
                for ignore_case in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpUnicodeCharacterMatcher::compile_with_flags(
                            &source,
                            ignore_case,
                            multiline,
                            all,
                        )
                        .unwrap();
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                writeln!(rows, "{source:?} ignore_case={ignore_case} multiline={multiline} dot_all={all} start={start} sticky={sticky} {:?}", matcher.find(&input, start, sticky)).unwrap();
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_ignore_case_characters_agree_over_every_code_point_and_pair_offset() {
        let plans = PLANS.map(|(source, all)| {
            (
                source,
                all,
                RegExpUnicodeCharacterMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    false,
                    all,
                )
                .unwrap(),
            )
        });
        for value in 0..=0x10ffff {
            let units = if value <= 0xffff {
                vec![value as u16]
            } else {
                let mut buffer = [0; 2];
                char::from_u32(value)
                    .unwrap()
                    .encode_utf16(&mut buffer)
                    .to_vec()
            };
            let input = JsString::from_code_units(units);
            for (source, all, matcher) in &plans {
                // WordCharacters adds the only two non-ASCII characters whose
                // pinned simple/common fold is an ASCII word character.
                let word = matches!(value,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a|0x17f|0x212a);
                let expected = match *source {
                    r"\w" => word,
                    r"\W" => !word,
                    _ => reference_membership(source, *all, value),
                };
                assert_eq!(
                    matcher.matches(value),
                    expected,
                    "source={source} value={value:x}"
                );
                assert_eq!(
                    matcher.find(&input, 0, true),
                    expected.then_some(0..input.len())
                );
                if input.len() == 2 {
                    assert_eq!(
                        matcher.find(&input, 1, true),
                        expected.then_some(0..2),
                        "source={source} value={value:x}"
                    );
                }
            }
        }
    }

    #[test]
    fn unicode_character_folding_contracts_and_opted_work() {
        for source in [
            "[a]",
            "(a)",
            "(. )",
            "a",
            "b",
            r"\p{ASCII}",
            r"\b",
            r"\w+",
            r"\w\d",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    true
                )
                .is_none(),
                "{source}"
            );
        }
        for (source, _) in PLANS {
            let plain = RegExpUnicodeCharacterMatcher::compile_with_flags(
                &JsString::from(source),
                false,
                false,
                false,
            )
            .unwrap();
            let folded = RegExpUnicodeCharacterMatcher::compile_with_flags(
                &JsString::from(source),
                true,
                false,
                false,
            )
            .unwrap();
            assert_eq!(
                folded.search_passes() > plain.search_passes(),
                matches!(source, r"\w" | r"\W")
            );
        }
        let source = JsString::from(r"^\w$");
        let mut charges = Vec::new();
        RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
            &source,
            true,
            true,
            false,
            |work| {
                charges.push(work);
                Ok::<(), usize>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(charges, [source.len()]);
        let result = RegExpUnicodeCharacterMatcher::compile_with_flags_and_work(
            &source,
            true,
            true,
            false,
            |_| Err::<(), _>(17),
        );
        assert!(matches!(result, Err(17)));
    }

    fn word_plan(source: &str, i: bool, m: bool, dot_all: bool) -> RegExpUnicodeCharacterMatcher {
        RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
            &JsString::from(source),
            i,
            m,
            dot_all,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_character_word_boundary_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\bA\b",
            r"\B.\B",
            r"^\b\d\b$",
            r"\b\D\B",
            r"\b\s\B",
            r"\B\S\b",
            r"\b\w\b",
            r"\b\W\b",
            r"\b\u{1f600}\b",
            r"\b\uD800\b",
            r"\b\$\B",
            r"\b\\\B",
            r"\bA",
            r".\B",
            r"^\B\u{10400}\B$",
            r"\b\u{000041}\b",
        ] {
            for i in [false, true] {
                for m in [false, true] {
                    for dot_all in [false, true] {
                        let plan = word_plan(source, i, m, dot_all);
                        for input in [
                            JsString::from("A😀B a_ſK\n"),
                            JsString::from("x\r\n8\u{2028}𐐨\u{2029}"),
                            JsString::from("A$!A\\!A \u{85}\u{feff}"),
                            JsString::from_code_units(vec![0x41, 0xd800, 0x5f, 0xdc00, 0x42]),
                            JsString::from(""),
                        ] {
                            for start in [0, 1, 2, input.len(), input.len() + 1] {
                                for sticky in [false, true] {
                                    writeln!(rows,"{:?} i={i} m={m} s={dot_all} input={input:?} start={start} sticky={sticky} {:?}",JsString::from(source),plan.find(&input,start,sticky)).unwrap();
                                }
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_character_word_boundaries_agree_with_independent_raw_candidates() {
        let alphabet = [
            0x41, 0x61, 0x5f, 8, 10, 0x20, 0x17f, 0x212a, 0xd83d, 0xde00, 0x38,
        ];
        let cases = [
            (r"\bA\b", 1, Some(true), Some(true), false, false),
            (r"\B.\B", 2, Some(false), Some(false), false, false),
            (r"^\b\d\b$", 3, Some(true), Some(true), true, true),
            (r"\b\W\b", 4, Some(true), Some(true), false, false),
            (r"\b\s\B", 5, Some(true), Some(false), false, false),
            (r"\B\S\b", 6, Some(false), Some(true), false, false),
            (r"\b\w\b", 7, Some(true), Some(true), false, false),
            (r"\b\u{1f600}\b", 8, Some(true), Some(true), false, false),
            (r"\bA", 1, Some(true), None, false, false),
            (r".\B", 2, None, Some(false), false, false),
        ];
        let word = |v: Option<&u16>, i| {
            v.is_some_and(|&v| {
                char::from_u32(u32::from(v)).is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                    || (i && [0x17f, 0x212a].contains(&v))
            })
        };
        let line = |v: u16| [10, 13, 0x2028, 0x2029].contains(&v);
        for i in [false, true] {
            for m in [false, true] {
                for dot_all in [false, true] {
                    let plans: Vec<_> = cases
                        .iter()
                        .map(|c| word_plan(c.0, i, m, dot_all))
                        .collect();
                    for len in 0..=3 {
                        for ordinal in 0..alphabet.len().pow(len) {
                            let units: Vec<_> = (0..len)
                                .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                                .collect();
                            let input = JsString::from_code_units(units.clone());
                            for start in 0..=units.len() + 1 {
                                for sticky in [false, true] {
                                    for (case, plan) in cases.iter().zip(&plans) {
                                        let mut expected = None;
                                        if start <= units.len() {
                                            let mut cursor = start;
                                            if cursor > 0
                                                && cursor < units.len()
                                                && (0xdc00..=0xdfff).contains(&units[cursor])
                                                && (0xd800..=0xdbff).contains(&units[cursor - 1])
                                            {
                                                cursor -= 1;
                                            }
                                            while cursor < units.len() {
                                                let first = u32::from(units[cursor]);
                                                let pair = (0xd800..=0xdbff).contains(&first)
                                                    && units.get(cursor + 1).is_some_and(|v| {
                                                        (0xdc00..=0xdfff).contains(v)
                                                    });
                                                let (point, width) = if pair {
                                                    (
                                                        (first - 0xd800) * 1024
                                                            + u32::from(units[cursor + 1])
                                                            - 0xdc00
                                                            + 65536,
                                                        2,
                                                    )
                                                } else {
                                                    (first, 1)
                                                };
                                                let end = cursor + width;
                                                let is_word = point <= 0xffff
                                                    && word(Some(&(point as u16)), i);
                                                let space = [
                                                    9, 10, 11, 12, 13, 0x20, 0xa0, 0x1680, 0x2028,
                                                    0x2029, 0x202f, 0x205f, 0x3000, 0xfeff,
                                                ]
                                                .contains(&point)
                                                    || (0x2000..=0x200a).contains(&point);
                                                let member = match case.1 {
                                                    1 => point == 0x41 || (i && point == 0x61),
                                                    2 => {
                                                        dot_all
                                                            || ![10, 13, 0x2028, 0x2029]
                                                                .contains(&point)
                                                    }
                                                    3 => (48..=57).contains(&point),
                                                    4 => !is_word,
                                                    5 => space,
                                                    6 => !space,
                                                    7 => is_word,
                                                    8 => point == 0x1f600,
                                                    _ => unreachable!(),
                                                };
                                                let at = |pos: usize| {
                                                    word(
                                                        pos.checked_sub(1)
                                                            .and_then(|j| units.get(j)),
                                                        i,
                                                    ) != word(units.get(pos), i)
                                                };
                                                let begin_ok = !case.4
                                                    || cursor == 0
                                                    || (m && line(units[cursor - 1]));
                                                let end_ok = !case.5
                                                    || end == units.len()
                                                    || (m && line(units[end]));
                                                if member
                                                    && begin_ok
                                                    && end_ok
                                                    && case.2.is_none_or(|b| at(cursor) == b)
                                                    && case.3.is_none_or(|b| at(end) == b)
                                                {
                                                    expected = Some(cursor..end);
                                                    break;
                                                }
                                                if sticky {
                                                    break;
                                                }
                                                cursor = end;
                                            }
                                        }
                                        assert_eq!(
                                            plan.find(&input, start, sticky),
                                            expected,
                                            "{} i={i} m={m} s={dot_all} {input:?} start={start} sticky={sticky}",
                                            case.0
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
    fn unicode_character_word_boundaries_work_clones_and_complete_body_contracts() {
        let source = JsString::from(format!(r"\b\u{{{}41}}\b", "0".repeat(10000)).as_str());
        let plan = RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
            &source,
            true,
            true,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            plan.clone().find(&JsString::from("x\na\n"), 0, false),
            Some(2..3)
        );
        let source = JsString::from(r"^\B\u{10400}\B$");
        let mut calls = 0;
        let plan = RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
            &source,
            true,
            true,
            false,
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
                RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                    &source,
                    true,
                    true,
                    false,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        assert_eq!(plan.find(&JsString::from("x\n𐐨\n"), 0, false), Some(2..4));
        for source in [
            r"A",
            r"\b[A]\b",
            r"\b(A)\b",
            r"\bA+\b",
            r"\bAB\b",
            r"\bA\\b",
            r"\b\bA\b",
            r"\b\B",
            r"\b\p{Script_Extensions=Latin}\b",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeCharacterMatcher::compile_literal_with_flags(
                &JsString::from(r"\bA\b"),
                true,
                true
            )
            .is_none()
        );
        assert!(
            RegExpUnicodeCharacterMatcher::compile_with_flags(
                &JsString::from(r"\b.\b"),
                true,
                true,
                true
            )
            .is_none()
        );
        // Pinned Unicode 18 folding changes original widths; word neighbors stay
        // non-ASCII non-word on either representation, independently of widths.
        let plan = word_plan(r"\Bß\B", true, false, false);
        assert_eq!(plan.find(&JsString::from("\u{1df95}"), 0, true), Some(0..2));
        assert_eq!(plan.find(&JsString::from("ß"), 0, true), Some(0..1));
    }

    fn zero_word_plan(source: &str, i: bool, m: bool) -> RegExpUnicodeCharacterMatcher {
        RegExpUnicodeCharacterMatcher::compile_word_assertion_with_flags_and_work(
            &JsString::from(source),
            i,
            m,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_word_assertion_zero_width_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\b", r"\B", r"^\b", r"^\B", r"\b$", r"\B$", r"^\b$", r"^\B$",
        ] {
            for i in [false, true] {
                for m in [false, true] {
                    let plan = zero_word_plan(source, i, m);
                    for input in [
                        JsString::from("A😀B a_ſK\n"),
                        JsString::from("x\r\n \u{2028}A\u{2029}"),
                        JsString::from("😀"),
                        JsString::from_code_units(vec![0x41, 0xd800, 0x5f, 0xdc00, 0x42]),
                        JsString::from(""),
                    ] {
                        for start in [0, 1, 2, input.len(), input.len() + 1] {
                            for sticky in [false, true] {
                                writeln!(rows,"{:?} i={i} m={m} input={input:?} start={start} sticky={sticky} {:?}",JsString::from(source),plan.find(&input,start,sticky)).unwrap();
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_word_assertions_agree_with_independent_raw_zero_width_candidates() {
        let alphabet = [0x41, 0x5f, 8, 10, 13, 0x2028, 0x17f, 0x212a, 0xd83d, 0xde00];
        let cases = [
            (r"\b", true, false, false),
            (r"\B", false, false, false),
            (r"^\b", true, true, false),
            (r"^\B", false, true, false),
            (r"\b$", true, false, true),
            (r"\B$", false, false, true),
            (r"^\b$", true, true, true),
            (r"^\B$", false, true, true),
        ];
        let line = |v: u16| [10, 13, 0x2028, 0x2029].contains(&v);
        for i in [false, true] {
            let word = |v: Option<&u16>| {
                v.is_some_and(|&v| {
                    char::from_u32(u32::from(v))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                        || (i && [0x17f, 0x212a].contains(&v))
                })
            };
            for m in [false, true] {
                let plans: Vec<_> = cases.iter().map(|c| zero_word_plan(c.0, i, m)).collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let input = JsString::from_code_units(units.clone());
                        let mut boundaries = vec![0];
                        for (position, &unit) in units.iter().enumerate() {
                            if position > 0
                                && (0xdc00..=0xdfff).contains(&unit)
                                && (0xd800..=0xdbff).contains(&units[position - 1])
                            {
                                continue;
                            }
                            if position > 0 {
                                boundaries.push(position);
                            }
                        }
                        if !units.is_empty() {
                            boundaries.push(units.len());
                        }
                        for start in 0..=units.len() + 1 {
                            for sticky in [false, true] {
                                for (case, plan) in cases.iter().zip(&plans) {
                                    let mut expected = None;
                                    if start <= units.len() {
                                        let normalized = if start > 0
                                            && start < units.len()
                                            && (0xdc00..=0xdfff).contains(&units[start])
                                            && (0xd800..=0xdbff).contains(&units[start - 1])
                                        {
                                            start - 1
                                        } else {
                                            start
                                        };
                                        for &at in &boundaries {
                                            if at < normalized || (sticky && at != normalized) {
                                                continue;
                                            }
                                            let left =
                                                word(at.checked_sub(1).and_then(|j| units.get(j)));
                                            let right = word(units.get(at));
                                            let begin =
                                                !case.2 || at == 0 || (m && line(units[at - 1]));
                                            let end = !case.3
                                                || at == units.len()
                                                || (m && line(units[at]));
                                            if (left != right) == case.1 && begin && end {
                                                expected = Some(at..at);
                                                break;
                                            }
                                        }
                                    }
                                    assert_eq!(
                                        plan.find(&input, start, sticky),
                                        expected,
                                        "{} i={i} m={m} {input:?} start={start} sticky={sticky}",
                                        case.0
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
    fn unicode_word_assertions_work_clones_endpoints_and_complete_contracts() {
        let source = JsString::from(r"^\B$");
        let plan = RegExpUnicodeCharacterMatcher::compile_word_assertion_with_flags_and_work(
            &source,
            true,
            true,
            |work| {
                assert_eq!(work, source.len());
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.clone().find(&JsString::from(""), 0, true), Some(0..0));
        assert_eq!(plan.find(&JsString::from("\n"), 1, true), Some(1..1));
        assert!(
            RegExpUnicodeCharacterMatcher::compile_word_assertion_with_flags_and_work(
                &source,
                true,
                true,
                |_| Err(())
            )
            .is_err()
        );
        for source in [
            "", r"^$", r"\b\B", r"\b\b", r"\bA", r"\B+", r"(\b)", r"\b|\B", r"\b(?=A)",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_word_assertion_with_flags_and_work(
                    &JsString::from(source),
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeCharacterMatcher::compile_with_word_boundaries_and_work(
                &JsString::from(r"\b"),
                true,
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeCharacterMatcher::compile_with_flags(
                &JsString::from(r"\B"),
                true,
                true,
                true
            )
            .is_none()
        );
    }

    fn zero_line_plan(source: &str, m: bool) -> RegExpUnicodeCharacterMatcher {
        RegExpUnicodeCharacterMatcher::compile_line_assertions_with_work(
            &JsString::from(source),
            m,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_line_assertion_zero_width_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in ["^", "$", "^$"] {
            for m in [false, true] {
                let plan = zero_line_plan(source, m);
                for input in [
                    JsString::from("A😀B\n"),
                    JsString::from("A\r\nB\u{2028}\u{2029}"),
                    JsString::from("😀"),
                    JsString::from_code_units(vec![0xd800, 10, 0xdc00]),
                    JsString::from(""),
                ] {
                    for start in [0, 1, 2, input.len(), input.len() + 1] {
                        for sticky in [false, true] {
                            writeln!(
                                rows,
                                "{:?} m={m} input={input:?} start={start} sticky={sticky} {:?}",
                                JsString::from(source),
                                plan.find(&input, start, sticky)
                            )
                            .unwrap();
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_line_assertions_agree_with_independent_raw_zero_width_candidates() {
        let alphabet = [0x41, 0x5f, 8, 10, 13, 0x2028, 0x2029, 0xd83d, 0xde00];
        let line = |v: u16| [10, 13, 0x2028, 0x2029].contains(&v);
        for m in [false, true] {
            let plans = [
                zero_line_plan("^", m),
                zero_line_plan("$", m),
                zero_line_plan("^$", m),
            ];
            for len in 0..=4 {
                for ordinal in 0..alphabet.len().pow(len) {
                    let units: Vec<_> = (0..len)
                        .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                        .collect();
                    let input = JsString::from_code_units(units.clone());
                    let boundaries: Vec<_> = (0..=units.len())
                        .filter(|&at| {
                            !(at > 0
                                && at < units.len()
                                && (0xdc00..=0xdfff).contains(&units[at])
                                && (0xd800..=0xdbff).contains(&units[at - 1]))
                        })
                        .collect();
                    for start in 0..=units.len() + 1 {
                        for sticky in [false, true] {
                            for (kind, plan) in plans.iter().enumerate() {
                                let mut expected = None;
                                if start <= units.len() {
                                    let normalized = if start > 0
                                        && start < units.len()
                                        && !boundaries.contains(&start)
                                    {
                                        start - 1
                                    } else {
                                        start
                                    };
                                    for &at in &boundaries {
                                        if at < normalized || (sticky && at != normalized) {
                                            continue;
                                        }
                                        let begin =
                                            kind == 1 || at == 0 || (m && line(units[at - 1]));
                                        let end = kind == 0
                                            || at == units.len()
                                            || (m && line(units[at]));
                                        if begin && end {
                                            expected = Some(at..at);
                                            break;
                                        }
                                    }
                                }
                                assert_eq!(
                                    plan.find(&input, start, sticky),
                                    expected,
                                    "kind={kind} m={m} {input:?} start={start} sticky={sticky}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_line_assertions_work_clones_endpoints_and_complete_contracts() {
        let source = JsString::from("^$");
        let plan = RegExpUnicodeCharacterMatcher::compile_line_assertions_with_work(
            &source,
            true,
            |work| {
                assert_eq!(work, 2);
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.clone().find(&JsString::from(""), 0, true), Some(0..0));
        assert_eq!(plan.find(&JsString::from("A\r\nB"), 0, false), Some(2..2));
        assert!(
            RegExpUnicodeCharacterMatcher::compile_line_assertions_with_work(&source, true, |_| {
                Err(())
            })
            .is_err()
        );
        for source in [
            "", r"\^", r"\$", r"\b", r"\B", r"^^", r"$$", r"$^", r"(^)", r"^|$", r"(?=^)",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_line_assertions_with_work(
                    &JsString::from(source),
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeCharacterMatcher::compile_word_assertion_with_flags_and_work(
                &source,
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeCharacterMatcher::compile_with_flags(&source, true, true, true).is_none()
        );
    }

    fn zero_assertion_plan(source: &str, i: bool, m: bool) -> RegExpUnicodeCharacterMatcher {
        RegExpUnicodeCharacterMatcher::compile_assertion_sequence_with_work(
            &JsString::from(source),
            i,
            m,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_assertion_sequence_zero_width_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"^^", r"$$", r"$^", r"\b\b", r"\B\B", r"\b\B", r"\B^", r"$\B", r"\b^$\b", r"\B$^\B",
            r"^\b\B$", r"$\b^",
        ] {
            for i in [false, true] {
                for m in [false, true] {
                    let plan = zero_assertion_plan(source, i, m);
                    for input in [
                        JsString::from("A😀B a_ſK\n"),
                        JsString::from("A\r\n \u{2028}\u{2029}"),
                        JsString::from("😀"),
                        JsString::from_code_units(vec![0x41, 0xd800, 0x5f, 0xdc00, 0x42]),
                        JsString::from(""),
                    ] {
                        for start in [0, 1, 2, input.len(), input.len() + 1] {
                            for sticky in [false, true] {
                                writeln!(rows,"{:?} i={i} m={m} input={input:?} start={start} sticky={sticky} {:?}",JsString::from(source),plan.find(&input,start,sticky)).unwrap();
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_assertion_sequences_agree_with_independent_raw_conjunctions() {
        let alphabet = [0x41, 0x5f, 8, 10, 13, 0x2028, 0x17f, 0x212a, 0xd83d, 0xde00];
        // Conditions derived from the literal test Patterns, independent of parsing.
        let cases = [
            (r"^^", true, false, false, false),
            (r"$$", false, true, false, false),
            (r"$^", true, true, false, false),
            (r"\b\b", false, false, true, false),
            (r"\B\B", false, false, false, true),
            (r"\b\B", false, false, true, true),
            (r"\B^", true, false, false, true),
            (r"$\B", false, true, false, true),
            (r"\b^$\b", true, true, true, false),
            (r"\B$^\B", true, true, false, true),
            (r"^\b\B$", true, true, true, true),
            (r"$\b^", true, true, true, false),
        ];
        let line = |v: u16| [10, 13, 0x2028, 0x2029].contains(&v);
        for i in [false, true] {
            let word = |v: Option<&u16>| {
                v.is_some_and(|&v| {
                    char::from_u32(u32::from(v))
                        .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                        || (i && [0x17f, 0x212a].contains(&v))
                })
            };
            for m in [false, true] {
                let plans: Vec<_> = cases
                    .iter()
                    .map(|c| zero_assertion_plan(c.0, i, m))
                    .collect();
                for len in 0..=4 {
                    for ordinal in 0..alphabet.len().pow(len) {
                        let units: Vec<_> = (0..len)
                            .map(|j| alphabet[ordinal / alphabet.len().pow(j) % alphabet.len()])
                            .collect();
                        let input = JsString::from_code_units(units.clone());
                        let boundaries: Vec<_> = (0..=units.len())
                            .filter(|&at| {
                                !(at > 0
                                    && at < units.len()
                                    && (0xdc00..=0xdfff).contains(&units[at])
                                    && (0xd800..=0xdbff).contains(&units[at - 1]))
                            })
                            .collect();
                        for start in 0..=units.len() + 1 {
                            for sticky in [false, true] {
                                for (case, plan) in cases.iter().zip(&plans) {
                                    let mut expected = None;
                                    if start <= units.len() {
                                        let normalized = if start > 0
                                            && start < units.len()
                                            && !boundaries.contains(&start)
                                        {
                                            start - 1
                                        } else {
                                            start
                                        };
                                        for &at in &boundaries {
                                            if at < normalized || (sticky && at != normalized) {
                                                continue;
                                            }
                                            let boundary =
                                                word(at.checked_sub(1).and_then(|j| units.get(j)))
                                                    != word(units.get(at));
                                            let begin =
                                                !case.1 || at == 0 || (m && line(units[at - 1]));
                                            let end = !case.2
                                                || at == units.len()
                                                || (m && line(units[at]));
                                            if begin
                                                && end
                                                && (!case.3 || boundary)
                                                && (!case.4 || !boundary)
                                            {
                                                expected = Some(at..at);
                                                break;
                                            }
                                        }
                                    }
                                    assert_eq!(
                                        plan.find(&input, start, sticky),
                                        expected,
                                        "{} i={i} m={m} {input:?} start={start} sticky={sticky}",
                                        case.0
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
    fn unicode_assertion_sequences_work_deduplication_clones_and_complete_contracts() {
        let source = JsString::from(r"\B$^".repeat(10000).as_str());
        let plan = RegExpUnicodeCharacterMatcher::compile_assertion_sequence_with_work(
            &source,
            true,
            true,
            |work| {
                assert_eq!(work, source.len());
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.search_passes(), 11);
        assert_eq!(
            plan.clone().find(&JsString::from("A\r\nB"), 0, false),
            Some(2..2)
        );
        assert!(
            RegExpUnicodeCharacterMatcher::compile_assertion_sequence_with_work(
                &source,
                true,
                true,
                |_| Err(())
            )
            .is_err()
        );
        assert_eq!(
            zero_assertion_plan(r"\b\B", true, true).find(&JsString::from("A😀B\n"), 0, false),
            None
        );
        for source in [
            "",
            r"^",
            r"$",
            r"\b",
            r"\B",
            r"\^\$",
            r"^A$",
            r"\b[A]\b",
            r"(\b)(\B)",
            r"(?:^)+",
            r"^|$",
            r"^(?=A)",
        ] {
            assert!(
                RegExpUnicodeCharacterMatcher::compile_assertion_sequence_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpUnicodeCharacterMatcher::compile_line_assertions_with_work(
                &JsString::from("^^"),
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
        assert!(
            RegExpUnicodeCharacterMatcher::compile_word_assertion_with_flags_and_work(
                &JsString::from(r"\b\b"),
                true,
                true,
                |_| Ok::<(), ()>(())
            )
            .unwrap()
            .is_none()
        );
    }
}
