//! Single Unicode dot and CharacterClassEscape atoms (22.2.2.7.1, 22.2.2.9).

use crate::{JsString, is_line_terminator, is_whitespace, regexp_literal::unicode_start};
use std::ops::Range;

/// Immutable matcher for one case-sensitive Unicode dot or character escape.
///
/// The caller must validate the complete Pattern with `u` or `v` and without `i`.
/// Dot and `\d`, `\D`, `\s`, `\S`, `\w`, `\W` are the complete admitted bodies.
/// Optional leading ^ and/or trailing $ are admitted by the assertion entry points.
/// Classes, groups, concatenations and quantifiers remain separate.
/// Matching consumes complete code points, including isolated surrogates, while
/// successful ranges retain UTF-16 indices. No input-sized storage is allocated.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeCharacterMatcher {
    atom: Atom,
    start_anchor: bool,
    end_anchor: bool,
    multiline: bool,
}

#[derive(Clone, Copy, Debug)]
enum Atom {
    Dot(bool),
    Escape(u16),
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
        }))
    }

    /// Bounds the input scan and each optional assertion check for opt-in work.
    pub fn search_passes(&self) -> usize {
        1 + usize::from(self.start_anchor) + usize::from(self.end_anchor)
    }

    /// Finds the first complete matching character, or only the sticky character.
    ///
    /// An initial offset inside a pair uses its leading boundary, following the
    /// approved Node/V8 resolution of the edition-17 inconsistency (22.2.7.2).
    /// An offset at/past the input end cannot match a consuming atom.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let units = input.code_units();
        let mut cursor = unicode_start(input, start)?;
        while let Some(&first) = units.get(cursor) {
            let (value, width) = unicode_input_character(units, cursor, first);
            let end = cursor + width;
            let start_ok = !self.start_anchor
                || cursor == 0
                || (self.multiline
                    && char::from_u32(u32::from(units[cursor - 1]))
                        .is_some_and(is_line_terminator));
            let end_ok = !self.end_anchor
                || end == units.len()
                || (self.multiline
                    && char::from_u32(u32::from(units[end])).is_some_and(is_line_terminator));
            if start_ok && end_ok && self.matches(value) {
                return Some(cursor..cursor + width);
            }
            if sticky {
                return None;
            }
            cursor += width;
        }
        None
    }

    fn matches(&self, value: u32) -> bool {
        match self.atom {
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
}
