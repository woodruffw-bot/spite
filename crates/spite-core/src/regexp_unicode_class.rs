//! Case-sensitive flat Unicode character-set classes (22.2.2.9).
use crate::{
    JsString,
    regexp_literal::{unicode_literal_atom, unicode_start},
    regexp_unicode_character::{unicode_assertions_match, unicode_input_character},
};
use std::{ops::Range, sync::Arc};

/// Immutable intervals for one case-sensitive flat u/v character set.
///
/// The caller must validate the complete Pattern in its u/v mode without i.
/// Unions, ranges and inversion admit all Unicode code points, including lone
/// surrogates. Property/string escapes, nested v sets and set operators
/// remain separate proofs. Matching always consumes complete input characters.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassMatcher {
    ranges: Arc<[(u32, u32)]>,
    inverted: bool,
    start_anchor: bool,
    end_anchor: bool,
    multiline: bool,
}

impl RegExpUnicodeClassMatcher {
    /// Compiles one complete class from a fully validated case-sensitive Pattern.
    pub fn compile(source: &JsString, unicode_sets: bool) -> Option<Self> {
        Self::compile_with_work(source, unicode_sets, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same subset with fallible opt-in construction work.
    ///
    /// Precharges parsing, sorting and merging before each corresponding pass.
    /// Storage scales with source size; ranges are never expanded into members.
    pub fn compile_with_work<E>(
        source: &JsString,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        if units.first() != Some(&0x5b) || units.last() != Some(&0x5d) {
            return Ok(None);
        }
        charge(units.len())?;
        Self::compile_units_with_work(units, unicode_sets, false, false, false, charge)
    }

    /// Compiles a flat class with an optional leading ^ and/or trailing $.
    ///
    /// Requires full case-sensitive u/v validation. Non-multiline $ succeeds
    /// only at the actual input end; m uses all four line terminators.
    pub fn compile_with_assertions(
        source: &JsString,
        unicode_sets: bool,
        multiline: bool,
    ) -> Option<Self> {
        Self::compile_with_assertions_and_work(source, unicode_sets, multiline, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same anchored subset with fallible opt-in construction work.
    pub fn compile_with_assertions_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        multiline: bool,
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
        if body.first() != Some(&0x5b) || body.last() != Some(&0x5d) {
            return Ok(None);
        }
        charge(units.len())?;
        Self::compile_units_with_work(
            body,
            unicode_sets,
            start_anchor,
            end_anchor,
            multiline,
            charge,
        )
    }

    fn compile_units_with_work<E>(
        units: &[u16],
        unicode_sets: bool,
        start_anchor: bool,
        end_anchor: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let inverted = units.get(1) == Some(&0x5e);
        let mut index = 1 + usize::from(inverted);
        let mut ranges = Vec::new();
        let mut escapes = 0u8;
        while index < units.len() - 1 {
            let Some(first) = class_atom(units, &mut index, unicode_sets) else {
                return Ok(None);
            };
            if units.get(index) == Some(&0x2d) && index + 1 < units.len() - 1 {
                // Only a raw hyphen introduces a range. A raw second hyphen
                // belongs to unproved v subtraction, even at small endpoints.
                if unicode_sets && units.get(index + 1) == Some(&0x2d) {
                    return Ok(None);
                }
                index += 1;
                let Some(ClassAtom::Point(last)) = class_atom(units, &mut index, unicode_sets)
                else {
                    return Ok(None);
                };
                let ClassAtom::Point(first) = first else {
                    return Ok(None);
                };
                if first > last {
                    return Ok(None);
                }
                ranges.push((first, last));
            } else {
                match first {
                    ClassAtom::Point(point) => ranges.push((point, point)),
                    ClassAtom::Escape(kind) => escapes |= 1 << kind,
                }
            }
            if index > units.len() - 1 {
                return Ok(None);
            }
        }
        // Repeated class escapes contribute each fixed set only once.
        for kind in 0..6 {
            if escapes & (1 << kind) != 0 {
                // At most ten fixed-range visits and eleven appends.
                charge(24)?;
                append_escape(&mut ranges, kind);
            }
        }
        if index != units.len() - 1 {
            return Ok(None);
        }
        if ranges.len() > 1 {
            // Stable byte passes sort 21-bit Unicode values by start with
            // explicit linear work bounds and source-sized temporary storage.
            charge(ranges.len())?;
            let mut buffer = vec![(0, 0); ranges.len()];
            for shift in [0, 8, 16] {
                charge(256)?;
                let mut positions = [0usize; 256];
                charge(ranges.len())?;
                for &(first, _) in &ranges {
                    positions[((first >> shift) & 255) as usize] += 1;
                }
                charge(256)?;
                let mut offset = 0;
                for position in &mut positions {
                    let count = *position;
                    *position = offset;
                    offset += count;
                }
                charge(ranges.len())?;
                for &range in &ranges {
                    let bucket = ((range.0 >> shift) & 255) as usize;
                    buffer[positions[bucket]] = range;
                    positions[bucket] += 1;
                }
                std::mem::swap(&mut ranges, &mut buffer);
            }
        }
        charge(ranges.len())?;
        let mut write = 0;
        for read in 0..ranges.len() {
            let range = ranges[read];
            if write > 0 && range.0 <= ranges[write - 1].1 + 1 {
                ranges[write - 1].1 = ranges[write - 1].1.max(range.1);
            } else {
                ranges[write] = range;
                write += 1;
            }
        }
        ranges.truncate(write);
        charge(ranges.len())?;
        Ok(Some(Self {
            ranges: ranges.into(),
            inverted,
            start_anchor,
            end_anchor,
            multiline,
        }))
    }

    /// Bounds the input scan and binary membership lookup for opt-in search work.
    pub fn search_passes(&self) -> usize {
        2 + lookup_levels(self.ranges.len())
            + usize::from(self.start_anchor)
            + usize::from(self.end_anchor)
    }

    /// Finds a complete character, normalizing an initial offset inside a pair.
    pub fn find(&self, input: &JsString, start: usize, sticky: bool) -> Option<Range<usize>> {
        let mut cursor = unicode_start(input, start)?;
        let units = input.code_units();
        while let Some(&first) = units.get(cursor) {
            let (value, width) = unicode_input_character(units, cursor, first);
            let end = cursor + width;
            if unicode_assertions_match(
                units,
                cursor,
                end,
                self.start_anchor,
                self.end_anchor,
                self.multiline,
            ) && self.matches(value)
            {
                return Some(cursor..end);
            }
            if sticky {
                return None;
            }
            cursor += width;
        }
        None
    }

    fn matches(&self, value: u32) -> bool {
        let next = self.ranges.partition_point(|&(first, _)| first <= value);
        let contains = next > 0 && value <= self.ranges[next - 1].1;
        contains != self.inverted
    }
}

fn lookup_levels(length: usize) -> usize {
    (usize::BITS - length.leading_zeros()) as usize
}

enum ClassAtom {
    Point(u32),
    Escape(u8),
}

fn class_atom(source: &[u16], index: &mut usize, unicode_sets: bool) -> Option<ClassAtom> {
    let unit = *source.get(*index)?;
    *index += 1;
    let point = match unit {
        0x5d => return None,
        0x5b if unicode_sets => return None,
        0x26 if unicode_sets && source.get(*index) == Some(&0x26) => return None,
        0x2d if unicode_sets && source.get(*index) == Some(&0x2d) => return None,
        0x5c if source.get(*index) == Some(&0x62) => {
            *index += 1;
            8
        }
        0x5c => {
            if let Some(kind) = [0x64, 0x44, 0x77, 0x57, 0x73, 0x53]
                .iter()
                .position(|unit| source.get(*index) == Some(unit))
            {
                *index += 1;
                return Some(ClassAtom::Escape(kind as u8));
            }
            unicode_literal_atom(source, index, unit)?
        }
        0xd800..=0xdbff => unicode_literal_atom(source, index, unit)?,
        _ => u32::from(unit),
    };
    Some(ClassAtom::Point(point))
}

fn append_escape(ranges: &mut Vec<(u32, u32)>, kind: u8) {
    let positive: &[(u32, u32)] = match kind / 2 {
        0 => &[(0x30, 0x39)],
        1 => &[(0x30, 0x39), (0x41, 0x5a), (0x5f, 0x5f), (0x61, 0x7a)],
        2 => &[
            (9, 13),
            (0x20, 0x20),
            (0xa0, 0xa0),
            (0x1680, 0x1680),
            (0x2000, 0x200a),
            (0x2028, 0x2029),
            (0x202f, 0x202f),
            (0x205f, 0x205f),
            (0x3000, 0x3000),
            (0xfeff, 0xfeff),
        ],
        _ => unreachable!("validated class escape"),
    };
    if kind & 1 == 0 {
        ranges.extend_from_slice(positive);
    } else {
        let mut cursor = 0;
        for &(start, end) in positive {
            if cursor < start {
                ranges.push((cursor, start - 1));
            }
            cursor = end + 1;
        }
        if cursor <= 0x10ffff {
            ranges.push((cursor, 0x10ffff));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;
    const CASES: [(&str, u8); 34] = [
        ("[]", 0),
        ("[^]", 1),
        ("[a-b]", 2),
        ("[😀]", 3),
        (r"[\uD800-\uDFFF]", 4),
        (r"[^\uD800-\uDFFF]", 5),
        (r"[\u{10000}-\u{10FFFF}]", 6),
        (r"[^\u{10000}-\u{10FFFF}]", 7),
        (r"[\d]", 8),
        (r"[\D]", 9),
        (r"[\w]", 10),
        (r"[\W]", 11),
        (r"[\s]", 12),
        (r"[\S]", 13),
        (r"[\d\D]", 1),
        (r"[^\d\D]", 0),
        (r"[\u{D800}\u{DC00}]", 14),
        (r"[\uD800\uDC00]", 15),
        (r"[\uDBFF\uDFFF]", 16),
        ("[^a😀]", 17),
        (r"[\u{0}-\u{10FFFF}]", 1),
        (r"[\x2d\u005d\u{5b}]", 18),
        (r"[\uD800\u{DC00}]", 14),
        (r"[\u{D800}\uDC00]", 14),
        (r"[\u{D7FF}-\u{E000}]", 19),
        (r"[^\uD7FF-\uE000]", 20),
        (r"[a-c\u0062-\u{65}]", 21),
        (r"[\w\u{1f600}]", 22),
        (r"[^\s\u{1f600}]", 23),
        (r"[\u{100000}-\u{10ffff}\u{ff}-\u{101}]", 24),
        (r"[^\D]", 8),
        (r"[\S\s]", 1),
        (r"[\b\cA\x61\0]", 25),
        (r"[\u{0000001f600}]", 3),
    ];
    fn reference(case: u8, value: u32) -> bool {
        let digit = (0x30..=0x39).contains(&value);
        let word = matches!(value,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a);
        let space = matches!(value,9..=13|0x20|0xa0|0x1680|0x2000..=0x200a|0x2028..=0x2029|0x202f|0x205f|0x3000|0xfeff);
        match case {
            0 => false,
            1 => true,
            2 => (0x61..=0x62).contains(&value),
            3 => value == 0x1f600,
            4 => (0xd800..=0xdfff).contains(&value),
            5 => !(0xd800..=0xdfff).contains(&value),
            6 => value >= 0x10000,
            7 => value < 0x10000,
            8 => digit,
            9 => !digit,
            10 => word,
            11 => !word,
            12 => space,
            13 => !space,
            14 => matches!(value, 0xd800 | 0xdc00),
            15 => value == 0x10000,
            16 => value == 0x10ffff,
            17 => !matches!(value, 0x61 | 0x1f600),
            18 => matches!(value, 0x2d | 0x5b | 0x5d),
            19 => (0xd7ff..=0xe000).contains(&value),
            20 => !(0xd7ff..=0xe000).contains(&value),
            21 => (0x61..=0x65).contains(&value),
            22 => word || value == 0x1f600,
            23 => !space && value != 0x1f600,
            24 => value >= 0x100000 || (0xff..=0x101).contains(&value),
            25 => matches!(value, 0 | 1 | 8 | 0x61),
            _ => unreachable!(),
        }
    }

    #[test]
    fn unicode_class_ranges_snapshot() {
        let input = JsString::from_code_units(vec![
            0xd83d, 0xde00, 0xd800, 0xa, 0xdc00, 0x39, 0x61, 0xfeff, 0xdbff, 0xdfff,
        ]);
        let mut rows = String::new();
        for (source, _) in CASES {
            for sets in [false, true] {
                let matcher =
                    RegExpUnicodeClassMatcher::compile(&JsString::from(source), sets).unwrap();
                for start in 0..=input.len() + 1 {
                    for sticky in [false, true] {
                        writeln!(
                            rows,
                            "{:?} sets={sets} start={start} sticky={sticky} {:?}",
                            JsString::from(source),
                            matcher.find(&input, start, sticky)
                        )
                        .unwrap();
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn every_code_point_agrees_with_independent_class_membership_definitions() {
        let plans = CASES.map(|(source, case)| {
            (
                case,
                RegExpUnicodeClassMatcher::compile(&JsString::from(source), true).unwrap(),
            )
        });
        for value in 0..=0x10ffff {
            for (case, matcher) in &plans {
                assert_eq!(
                    matcher.matches(value),
                    reference(*case, value),
                    "case={case} value={value:x}"
                );
            }
        }
    }

    #[test]
    fn unicode_class_search_agrees_with_independent_complete_character_decoding() {
        let alphabet = [0x61, 0x39, 0xa, 0xfeff, 0xd800, 0xdbff, 0xdc00, 0xdfff];
        let plans = CASES.map(|(source, case)| {
            (
                case,
                RegExpUnicodeClassMatcher::compile(&JsString::from(source), true).unwrap(),
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
                        for (case, matcher) in &plans {
                            let expected = decoded
                                .iter()
                                .filter(|(_, range)| range.end > start)
                                .take(if sticky { 1 } else { decoded.len() })
                                .find(|(value, _)| reference(*case, *value))
                                .map(|(_, range)| range.clone());
                            assert_eq!(
                                matcher.find(&input, start, sticky),
                                expected,
                                "case={case} input={input:?} start={start} sticky={sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_class_escaped_punctuation_and_unsupported_proofs_remain_distinct() {
        let matcher =
            RegExpUnicodeClassMatcher::compile(&JsString::from(r"[\&\&\-\-]"), true).unwrap();
        for text in ["&", "-"] {
            assert_eq!(matcher.find(&JsString::from(text), 0, true), Some(0..1));
        }
        let matcher = RegExpUnicodeClassMatcher::compile(&JsString::from(r"[\&&]"), true).unwrap();
        assert_eq!(matcher.find(&JsString::from("&"), 0, true), Some(0..1));
        let matcher = RegExpUnicodeClassMatcher::compile(&JsString::from("[a&&b]"), false).unwrap();
        assert_eq!(matcher.find(&JsString::from("&"), 0, true), Some(0..1));
        for source in [
            "[a&&b]",
            r"[\x00--0]",
            r"[\x00&&0]",
            "[[a]]",
            r"[\q{a|b}]",
            r"[\p{ASCII}]",
            r"[^\q{a|b}]",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile(&JsString::from(source), true).is_none(),
                "{source}"
            );
        }
        for source in [
            "",
            "a",
            ".",
            "[a]+",
            "[a]b",
            "([a])",
            "^[a]$",
            r"[\p{ASCII}]",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile(&JsString::from(source), false).is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn unicode_class_large_sources_masks_clones_and_fallible_work() {
        let source = JsString::from(format!("[{}\\u{{1f600}}]", r"\d".repeat(100000)).as_str());
        let matcher = RegExpUnicodeClassMatcher::compile(&source, true).unwrap();
        assert_eq!(&*matcher.ranges, &[(0x30, 0x39), (0x1f600, 0x1f600)]);
        assert_eq!(
            matcher.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        let source = JsString::from(r"[\W\u{1000}-\u{ffff}a😀]");
        let mut charges = Vec::new();
        RegExpUnicodeClassMatcher::compile_with_work(&source, true, |work| {
            charges.push(work);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for failure in 0..charges.len() {
            let mut calls = 0;
            let result = RegExpUnicodeClassMatcher::compile_with_work(&source, true, |_| {
                let index = calls;
                calls += 1;
                if index == failure { Err(17) } else { Ok(()) }
            });
            assert!(matches!(result, Err(17)));
            assert_eq!(calls, failure + 1);
        }
    }
    #[test]
    fn unicode_class_assertion_ranges_snapshot() {
        let input = JsString::from_code_units(vec![
            0xd83d, 0xde00, 0xa, 0x61, 0xd, 0xa, 0xd800, 0x2028, 0xdc00, 0x2029, 0x24,
        ]);
        let mut rows = String::new();
        for body in [
            "[😀]",
            "[^]",
            r"[\D]",
            r"[\uD800-\uDFFF]",
            r"[\r\n\u2028\u2029]",
            r"[\$\^]",
        ] {
            for (left, right) in [("", ""), ("^", ""), ("", "$"), ("^", "$")] {
                let source = JsString::from(format!("{left}{body}{right}").as_str());
                for sets in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpUnicodeClassMatcher::compile_with_assertions(
                            &source, sets, multiline,
                        )
                        .unwrap();
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                writeln!(rows, "{source:?} sets={sets} multiline={multiline} start={start} sticky={sticky} {:?}", matcher.find(&input, start, sticky)).unwrap();
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    #[test]
    fn unicode_class_assertions_agree_with_independent_character_contexts() {
        let alphabet = [0x61, 0x62, 0xa, 0xd, 0x2028, 0x2029, 0xd800, 0xdc00, 0x24];
        let bodies = [
            ("[a-b]", 2),
            ("[^]", 1),
            (r"[\uD800-\uDFFF]", 4),
            (r"[\s]", 12),
        ];
        let mut plans = Vec::new();
        for (body, case) in bodies {
            for (left, right) in [("", ""), ("^", ""), ("", "$"), ("^", "$")] {
                for multiline in [false, true] {
                    let source = JsString::from(format!("{left}{body}{right}").as_str());
                    plans.push((
                        case,
                        !left.is_empty(),
                        !right.is_empty(),
                        multiline,
                        RegExpUnicodeClassMatcher::compile_with_assertions(
                            &source, true, multiline,
                        )
                        .unwrap(),
                    ));
                }
            }
        }
        for length in 0..=4 {
            for mut ordinal in 0..alphabet.len().pow(length) {
                let units: Vec<_> = (0..length)
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
                        for (case, left, right, multiline, matcher) in &plans {
                            let first = decoded.iter().position(|(_, range)| range.end > start);
                            let expected = first.and_then(|first| {
                                decoded
                                    .iter()
                                    .enumerate()
                                    .skip(first)
                                    .take(if sticky { 1 } else { decoded.len() })
                                    .find(|(i, (value, range))| {
                                        let line =
                                            |value| matches!(value, 0xa | 0xd | 0x2028 | 0x2029);
                                        let left_ok = !left
                                            || *i == 0
                                            || (*multiline && line(decoded[i - 1].0));
                                        let right_ok = !right
                                            || range.end == input.len()
                                            || (*multiline && line(decoded[i + 1].0));
                                        left_ok && right_ok && reference(*case, *value)
                                    })
                                    .map(|(_, (_, range))| range.clone())
                            });
                            assert_eq!(
                                matcher.find(&input, start, sticky),
                                expected,
                                "case={case} left={left} right={right} multiline={multiline} input={input:?} start={start} sticky={sticky}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_class_assertion_contracts_and_fallible_work() {
        for source in ["^[a]", "[a]$", "^[a]$"] {
            assert!(RegExpUnicodeClassMatcher::compile(&JsString::from(source), true).is_none());
            let plan = RegExpUnicodeClassMatcher::compile_with_assertions(
                &JsString::from(source),
                true,
                true,
            )
            .unwrap();
            assert_eq!(
                plan.search_passes(),
                3 + usize::from(source.starts_with('^')) + usize::from(source.ends_with('$'))
            );
        }
        for source in [
            "^",
            "$",
            "^$",
            "^^[a]",
            "[a]$$",
            "^([a])$",
            "^[a]+$",
            "^[a]b$",
            "^[a&&b]$",
            r"^[\p{ASCII}]$",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_assertions(
                    &JsString::from(source),
                    true,
                    true
                )
                .is_none(),
                "{source}"
            );
        }
        let source = JsString::from(r"^[\W\u{1000}-\u{ffff}a😀]$");
        let mut charges = Vec::new();
        RegExpUnicodeClassMatcher::compile_with_assertions_and_work(&source, true, true, |work| {
            charges.push(work);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        assert_eq!(charges[0], source.len());
        for failure in 0..charges.len() {
            let mut calls = 0;
            let result = RegExpUnicodeClassMatcher::compile_with_assertions_and_work(
                &source,
                true,
                true,
                |_| {
                    let index = calls;
                    calls += 1;
                    if index == failure { Err(17) } else { Ok(()) }
                },
            );
            assert!(matches!(result, Err(17)));
            assert_eq!(calls, failure + 1);
        }
    }
}
