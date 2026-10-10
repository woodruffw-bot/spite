//! Flat Unicode character-set classes (22.2.2.9).
use crate::{
    JsString,
    case_data::SIMPLE_CASE_FOLD,
    regexp_canonicalize_character,
    regexp_literal::{unicode_literal_atom, unicode_start},
    regexp_unicode_character::{unicode_assertions_match, unicode_input_character},
};
use std::{ops::Range, sync::Arc};

/// Immutable intervals for one flat u/v character set.
///
/// The caller must validate the complete Pattern in its u/v mode. Bare/assertion
/// entry points require no i; flag-aware entry points admit simple/common folding.
/// Unions, ranges and inversion admit all Unicode code points, including lone
/// surrogates. Fixed ASCII, Any, hex-digit and White_Space properties
/// are admitted in flat unions and through the standalone binary-property API.
/// Other property/string sets and v operators remain separate proofs.
/// Matching always consumes complete input characters.
#[derive(Clone, Debug)]
pub struct RegExpUnicodeClassMatcher {
    ranges: Arc<[(u32, u32)]>,
    inverted: bool,
    start_anchor: bool,
    end_anchor: bool,
    multiline: bool,
    ignore_case: bool,
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
        Self::compile_units_with_work(units, unicode_sets, false, false, false, false, charge)
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
        charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        Self::compile_with_flags_and_work(source, unicode_sets, false, multiline, charge)
    }

    /// Compiles a validated flat Unicode class with i and optional assertions.
    ///
    /// Uses pinned simple/common folding while consuming original complete input
    /// characters. Outer inversion is applied after folded membership.
    pub fn compile_with_flags(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
    ) -> Option<Self> {
        Self::compile_with_flags_and_work(source, unicode_sets, ignore_case, multiline, |_| {
            Ok::<(), std::convert::Infallible>(())
        })
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same flag-aware subset with fallible opt-in work accounting.
    pub fn compile_with_flags_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
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
            ignore_case,
            charge,
        )
    }

    /// Compiles standalone fixed binary Unicode properties and boundaries.
    ///
    /// Admits ASCII, Any, ASCII_Hex_Digit/AHex, Hex_Digit/Hex and White_Space/space.
    ///
    /// Requires complete u/v validation with the supplied flags. Exact property
    /// names are required. In u with i, P complements before folding; in v with
    /// i, P complements folded membership. Other property sets remain excluded.
    pub fn compile_binary_property_with_flags(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
    ) -> Option<Self> {
        Self::compile_binary_property_with_flags_and_work(
            source,
            unicode_sets,
            ignore_case,
            multiline,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap_or_else(|never| match never {})
    }

    /// Compiles the same binary-property subset with fallible opt-in work.
    pub fn compile_binary_property_with_flags_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let units = source.code_units();
        charge(units.len())?;
        let start_anchor = units.first() == Some(&0x5e);
        let end_anchor = units.last() == Some(&0x24);
        let Some(body) =
            units.get(usize::from(start_anchor)..units.len() - usize::from(end_anchor))
        else {
            return Ok(None);
        };
        if body.len() < 5
            || body[0] != 0x5c
            || !matches!(body[1], 0x70 | 0x50)
            || body[2] != 0x7b
            || body.last() != Some(&0x7d)
        {
            return Ok(None);
        }
        let name = &body[3..body.len() - 1];
        let Some(kind) = fixed_property_kind(name) else {
            return Ok(None);
        };
        let negated = body[1] == 0x50;
        // CompileToCharSet complements u property escapes before Canonicalize.
        // UnicodeSets mode complements the canonical set instead. The shared
        // interval constructor folds only members of its original ranges.
        let complement_first = negated && ignore_case && !unicode_sets;
        // Fixed properties use up to ten positive or eleven complementary intervals.
        // Charge the bounded append before it can allocate.
        charge(match kind {
            4 => 4,
            6 => 7,
            8 => 11,
            _ => 1,
        })?;
        let mut ranges = Vec::new();
        append_property(
            &mut ranges,
            kind + u8::from(complement_first),
            unicode_sets,
            ignore_case,
        );
        Self::compile_ranges_with_work(
            ranges,
            negated && !complement_first,
            start_anchor,
            end_anchor,
            multiline,
            ignore_case,
            charge,
        )
    }

    fn compile_units_with_work<E>(
        units: &[u16],
        unicode_sets: bool,
        start_anchor: bool,
        end_anchor: bool,
        multiline: bool,
        ignore_case: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let inverted = units.get(1) == Some(&0x5e);
        let mut index = 1 + usize::from(inverted);
        let mut ranges = Vec::new();
        let mut escapes = 0u8;
        let mut properties = 0u16;
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
                    ClassAtom::Property(kind) => properties |= 1 << kind,
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
                append_escape(&mut ranges, kind, ignore_case);
            }
        }
        // Fixed properties are appended once even when repeated in the source.
        for kind in 0..10 {
            if properties & (1 << kind) != 0 {
                // Appending may move the original intervals if capacity grows.
                // Charge that move and the fixed set before allocating.
                charge(ranges.len().saturating_add(if kind >= 8 { 24 } else { 8 }))?;
                append_property(&mut ranges, kind, unicode_sets, ignore_case);
            }
        }
        if index != units.len() - 1 {
            return Ok(None);
        }
        Self::compile_ranges_with_work(
            ranges,
            inverted,
            start_anchor,
            end_anchor,
            multiline,
            ignore_case,
            charge,
        )
    }

    fn compile_ranges_with_work<E>(
        mut ranges: Vec<(u32, u32)>,
        inverted: bool,
        start_anchor: bool,
        end_anchor: bool,
        multiline: bool,
        ignore_case: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        ranges = normalize_ranges_with_work(ranges, &mut charge)?;
        if ignore_case {
            // Keep original intervals: canonical input cannot reach mapped
            // source values. Add only targets of members of the original set.
            // Lookup is over the normalized original intervals, not new targets.
            charge(
                SIMPLE_CASE_FOLD
                    .len()
                    .saturating_mul(3 + lookup_levels(ranges.len())),
            )?;
            let mut targets = Vec::new();
            for &(source, target) in SIMPLE_CASE_FOLD {
                if interval_contains(&ranges, source) {
                    targets.push((target, target));
                }
            }
            // Extension can move the original intervals if capacity grows.
            charge(ranges.len().saturating_add(targets.len()))?;
            ranges.extend(targets);
            ranges = normalize_ranges_with_work(ranges, &mut charge)?;
        }
        charge(ranges.len())?;
        Ok(Some(Self {
            ranges: ranges.into(),
            inverted,
            start_anchor,
            end_anchor,
            multiline,
            ignore_case,
        }))
    }

    /// Bounds the input scan and binary membership lookup for opt-in search work.
    pub fn search_passes(&self) -> usize {
        2 + lookup_levels(self.ranges.len())
            + usize::from(self.start_anchor)
            + usize::from(self.end_anchor)
            + if self.ignore_case {
                2 + (usize::BITS - SIMPLE_CASE_FOLD.len().leading_zeros()) as usize
            } else {
                0
            }
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
        let value = if self.ignore_case {
            regexp_canonicalize_character(value, true, true)
        } else {
            value
        };
        interval_contains(&self.ranges, value) != self.inverted
    }
}

fn interval_contains(ranges: &[(u32, u32)], value: u32) -> bool {
    let next = ranges.partition_point(|&(first, _)| first <= value);
    next > 0 && value <= ranges[next - 1].1
}

fn normalize_ranges_with_work<E>(
    mut ranges: Vec<(u32, u32)>,
    charge: &mut impl FnMut(usize) -> Result<(), E>,
) -> Result<Vec<(u32, u32)>, E> {
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
    Ok(ranges)
}

fn lookup_levels(length: usize) -> usize {
    (usize::BITS - length.leading_zeros()) as usize
}

enum ClassAtom {
    Point(u32),
    Escape(u8),
    Property(u8),
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
            if matches!(source.get(*index), Some(0x70 | 0x50)) {
                let negative = source[*index] == 0x50;
                *index += 1;
                if source.get(*index) != Some(&0x7b) {
                    return None;
                }
                *index += 1;
                let name = *index;
                while source.get(*index).is_some_and(|&unit| unit != 0x7d) {
                    *index += 1;
                }
                let kind = fixed_property_kind(source.get(name..*index)?)?;
                if source.get(*index) != Some(&0x7d) {
                    return None;
                }
                *index += 1;
                return Some(ClassAtom::Property(kind + u8::from(negative)));
            }
            unicode_literal_atom(source, index, unit)?
        }
        0xd800..=0xdbff => unicode_literal_atom(source, index, unit)?,
        _ => u32::from(unit),
    };
    Some(ClassAtom::Point(point))
}

fn fixed_property_kind(name: &[u16]) -> Option<u8> {
    Some(match name {
        [0x41, 0x53, 0x43, 0x49, 0x49] => 0,
        [0x41, 0x6e, 0x79] => 2,
        [0x41, 0x48, 0x65, 0x78]
        | [
            0x41,
            0x53,
            0x43,
            0x49,
            0x49,
            0x5f,
            0x48,
            0x65,
            0x78,
            0x5f,
            0x44,
            0x69,
            0x67,
            0x69,
            0x74,
        ] => 4,
        [0x48, 0x65, 0x78] | [0x48, 0x65, 0x78, 0x5f, 0x44, 0x69, 0x67, 0x69, 0x74] => 6,
        [0x73, 0x70, 0x61, 0x63, 0x65]
        | [
            0x57,
            0x68,
            0x69,
            0x74,
            0x65,
            0x5f,
            0x53,
            0x70,
            0x61,
            0x63,
            0x65,
        ] => 8,
        _ => return None,
    })
}

// These interval preimages give the same canonical membership as the standalone
// property plan. Only v+i P{ASCII} must remove the two non-ASCII ASCII aliases
// before union and the shared fold closure. u+i complements the original set.
// Both hex properties have no additional simple/common-fold preimages: uppercase
// members already have lowercase partners. Both u/v complement orders agree.
fn append_property(ranges: &mut Vec<(u32, u32)>, kind: u8, unicode_sets: bool, ignore_case: bool) {
    let intervals: &[(u32, u32)] = match kind {
        0 => &[(0, 0x7f)],
        1 if unicode_sets && ignore_case => &[(0x80, 0x17e), (0x180, 0x2129), (0x212b, 0x10ffff)],
        1 => &[(0x80, 0x10ffff)],
        2 => &[(0, 0x10ffff)],
        3 => &[],
        4 => &[(0x30, 0x39), (0x41, 0x46), (0x61, 0x66)],
        5 => &[(0, 0x2f), (0x3a, 0x40), (0x47, 0x60), (0x67, 0x10ffff)],
        6 => &[
            (0x30, 0x39),
            (0x41, 0x46),
            (0x61, 0x66),
            (0xff10, 0xff19),
            (0xff21, 0xff26),
            (0xff41, 0xff46),
        ],
        7 => &[
            (0, 0x2f),
            (0x3a, 0x40),
            (0x47, 0x60),
            (0x67, 0xff0f),
            (0xff1a, 0xff20),
            (0xff27, 0xff40),
            (0xff47, 0x10ffff),
        ],
        // UCD White_Space includes NEL (0085) and excludes BOM (FEFF).
        8 => &[
            (9, 13),
            (0x20, 0x20),
            (0x85, 0x85),
            (0xa0, 0xa0),
            (0x1680, 0x1680),
            (0x2000, 0x200a),
            (0x2028, 0x2029),
            (0x202f, 0x202f),
            (0x205f, 0x205f),
            (0x3000, 0x3000),
        ],
        9 => &[
            (0, 8),
            (14, 0x1f),
            (0x21, 0x84),
            (0x86, 0x9f),
            (0xa1, 0x167f),
            (0x1681, 0x1fff),
            (0x200b, 0x2027),
            (0x202a, 0x202e),
            (0x2030, 0x205e),
            (0x2060, 0x2fff),
            (0x3001, 0x10ffff),
        ],
        _ => unreachable!("validated binary property kind"),
    };
    ranges.extend_from_slice(intervals);
}

fn append_escape(ranges: &mut Vec<(u32, u32)>, kind: u8, ignore_case: bool) {
    let positive: &[(u32, u32)] = match kind / 2 {
        0 => &[(0x30, 0x39)],
        // WordCharacters includes the pinned non-ASCII ASCII-word aliases
        // before complement. Exhaustive tests verify these two additions.
        1 if ignore_case => &[
            (0x30, 0x39),
            (0x41, 0x5a),
            (0x5f, 0x5f),
            (0x61, 0x7a),
            (0x17f, 0x17f),
            (0x212a, 0x212a),
        ],
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
            r"[\p{Assigned}]",
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
            r"[\p{Assigned}]",
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
            r"^[\p{Assigned}]$",
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
    #[test]
    fn unicode_class_folding_ranges_snapshot() {
        let input = JsString::from_code_units(vec![
            0xd83d, 0xde00, 0x17f, 0x212a, 0x41, 0x61, 0x1fd3, 0x390, 0xd800, 0xa, 0xdc00, 0xfeff,
        ]);
        let mut rows = String::new();
        for (body, _) in CASES {
            for (left, right) in [("", ""), ("^", "$")] {
                let source = JsString::from(format!("{left}{body}{right}").as_str());
                for sets in [false, true] {
                    for multiline in [false, true] {
                        let matcher = RegExpUnicodeClassMatcher::compile_with_flags(
                            &source, sets, true, multiline,
                        )
                        .unwrap();
                        for start in 0..=input.len() + 1 {
                            for sticky in [false, true] {
                                writeln!(rows,"{source:?} sets={sets} multiline={multiline} start={start} sticky={sticky} {:?}",matcher.find(&input,start,sticky)).unwrap();
                            }
                        }
                    }
                }
            }
        }
        insta::assert_snapshot!(rows);
    }

    fn folding_reference_member(case: u8, value: u32) -> bool {
        let word = matches!(value,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a|0x17f|0x212a);
        match case {
            10 => word,
            11 => !word,
            22 => word || value == 0x1f600,
            _ => reference(case, value),
        }
    }

    #[test]
    fn every_unicode_class_fold_agrees_with_independent_set_preimages() {
        use std::collections::HashMap;
        let folds: HashMap<u32, u32> = SIMPLE_CASE_FOLD.iter().copied().collect();
        let mut preimages: HashMap<u32, Vec<u32>> = HashMap::new();
        for &(source, target) in SIMPLE_CASE_FOLD {
            preimages.entry(target).or_default().push(source);
        }
        let plans = CASES.map(|(source, case)| {
            (
                case,
                source.starts_with("[^"),
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    false,
                )
                .unwrap(),
            )
        });
        for value in 0..=0x10ffff {
            let canonical = folds.get(&value).copied().unwrap_or(value);
            for (case, inverted, matcher) in &plans {
                let member = |point| folding_reference_member(*case, point) != *inverted;
                let included = member(canonical)
                    || preimages
                        .get(&canonical)
                        .is_some_and(|points| points.iter().copied().any(member));
                assert_eq!(
                    matcher.matches(value),
                    included != *inverted,
                    "case={case} value={value:x}"
                );
            }
        }
    }

    #[test]
    fn unicode_class_word_complements_and_cross_plane_widths() {
        for source in [r"[\w]", r"[^\W]"] {
            let matcher = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(source),
                true,
                true,
                false,
            )
            .unwrap();
            for point in [0x41, 0x61, 0x17f, 0x212a] {
                assert!(matcher.matches(point));
            }
            for point in [0xdf, 0x1df95, 0x1f600, 0xd800] {
                assert!(!matcher.matches(point));
            }
        }
        for source in [r"[\W]", r"[^\w]"] {
            let matcher = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(source),
                true,
                true,
                false,
            )
            .unwrap();
            for point in [0x41, 0x61, 0x17f, 0x212a] {
                assert!(!matcher.matches(point));
            }
            for point in [0xdf, 0x1df95, 0x1f600, 0xd800] {
                assert!(matcher.matches(point));
            }
        }
        // Unicode 18 has a supplementary simple-fold source with a BMP target.
        assert!(SIMPLE_CASE_FOLD.contains(&(0x1df95, 0xdf)));
        let mut buffer = [0; 2];
        let wide = JsString::from_code_units(
            char::from_u32(0x1df95)
                .unwrap()
                .encode_utf16(&mut buffer)
                .to_vec(),
        );
        for sets in [false, true] {
            let matcher = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from("[ß]"),
                sets,
                true,
                false,
            )
            .unwrap();
            assert_eq!(matcher.find(&wide, 0, true), Some(0..2));
            assert_eq!(matcher.find(&wide, 1, true), Some(0..2));
            let matcher = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\u{1df95}]"),
                sets,
                true,
                false,
            )
            .unwrap();
            assert_eq!(matcher.find(&JsString::from("ß"), 0, true), Some(0..1));
            let inverted = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\u{10000}-\u{10ffff}]"),
                sets,
                true,
                false,
            )
            .unwrap();
            assert_eq!(inverted.find(&JsString::from("ß"), 0, true), None);
        }
    }

    #[test]
    fn unicode_class_folding_contracts_large_masks_and_fallible_work() {
        for source in [
            r"[a&&b]",
            r"[[a]]",
            r"[\p{Assigned}]",
            r"[\q{a|b}]",
            r"([a])",
            r"[a]+",
            r"[a]b",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    true
                )
                .is_none(),
                "{source}"
            );
        }
        let source = JsString::from(format!("[{}A]", r"\w".repeat(100000)).as_str());
        let matcher =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(matcher.find(&JsString::from("K"), 0, true), Some(0..1));
        let source = JsString::from(r"^[\W\u{1000}-\u{ffff}A😀]$");
        let mut charges = Vec::new();
        RegExpUnicodeClassMatcher::compile_with_flags_and_work(&source, true, true, true, |work| {
            charges.push(work);
            Ok::<(), usize>(())
        })
        .unwrap()
        .unwrap();
        for failure in 0..charges.len() {
            let mut calls = 0;
            let result = RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                &source,
                true,
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

    #[test]
    fn unicode_binary_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{ASCII}",
            r"\P{ASCII}",
            r"\p{Any}",
            r"\P{Any}",
            r"^\p{ASCII}$",
            r"^\P{ASCII}$",
            r"\p{Any}$",
            r"^\p{Any}",
        ] {
            for sets in [false, true] {
                for ignore_case in [false, true] {
                    for multiline in [false, true] {
                        let plan = RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                            &JsString::from(source),
                            sets,
                            ignore_case,
                            multiline,
                        )
                        .unwrap();
                        for input in ["aSkſK😀\n", "\r\n\u{2028}\u{2029}", "A\nſ\n😀", ""] {
                            let input = JsString::from(input);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    writeln!(rows,"{:?} v={sets} i={ignore_case} m={multiline} input={input:?} start={start} sticky={sticky} {:?}",JsString::from(source),plan.find(&input,start,sticky)).unwrap();
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
    fn unicode_binary_properties_match_independent_membership_for_all_code_points() {
        let settings = [
            (r"\p{ASCII}", false, false),
            (r"\P{ASCII}", false, false),
            (r"\p{ASCII}", false, true),
            (r"\p{ASCII}", true, true),
            (r"\P{ASCII}", false, true),
            (r"\P{ASCII}", true, true),
            (r"\p{Any}", false, true),
            (r"\P{Any}", true, true),
        ];
        let plans: Vec<_> = settings
            .iter()
            .map(|(source, sets, i)| {
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                    &JsString::from(*source),
                    *sets,
                    *i,
                    false,
                )
                .unwrap()
            })
            .collect();
        for point in 0..=0x10ffffu32 {
            let units = if point <= 0xffff {
                vec![point as u16]
            } else {
                vec![
                    ((point - 65536) / 1024 + 0xd800) as u16,
                    ((point - 65536) % 1024 + 0xdc00) as u16,
                ]
            };
            let width = units.len();
            let input = JsString::from_code_units(units);
            for ((source, sets, i), plan) in settings.iter().zip(&plans) {
                let ascii = point <= 127;
                let alias = matches!(point, 0x17f | 0x212a);
                let expected = match *source {
                    r"\p{Any}" => true,
                    r"\P{Any}" => false,
                    r"\p{ASCII}" => ascii || (*i && alias),
                    r"\P{ASCII}" if !i => !ascii,
                    r"\P{ASCII}" if *sets => !ascii && !alias,
                    r"\P{ASCII}" => !ascii || matches!(point, 0x4b | 0x6b | 0x53 | 0x73),
                    _ => unreachable!(),
                };
                assert_eq!(
                    plan.find(&input, 0, false),
                    expected.then_some(0..width),
                    "{source} v={sets} i={i} U+{point:X}"
                );
            }
        }
    }

    #[test]
    fn unicode_binary_property_contracts_clones_source_bounds_and_work() {
        for source in [r"\p{ASCII}", r"\P{Any}", r"^\p{ASCII}$"] {
            assert!(RegExpUnicodeClassMatcher::compile(&JsString::from(source), true).is_none());
            assert!(
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    true
                )
                .is_none()
            );
        }
        for source in [
            r"\p{ascii}",
            r"\p{Assigned}",
            r"\p{Script=Han}",
            r"\p{RGI_Emoji}",
            r"[\p{ASCII}]",
            r"(\p{ASCII})",
            r"\p{ASCII}+",
            r"\p{ASCII}x",
            r"\p{ASCII}|x",
            r"^$",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    true
                )
                .is_none()
            );
        }
        let source = JsString::from(r"^\P{ASCII}$");
        let mut calls = 0;
        let plan = RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
            &source,
            false,
            true,
            true,
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
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &source,
                    false,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        let input = JsString::from("x\n😀\nS");
        assert_eq!(plan.clone().find(&input, 3, false), Some(2..4));
        assert_eq!(plan.find(&input, 5, true), Some(5..6));
        let literal = RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
            &JsString::from(r"^\p{ASCII}$"),
            false,
            false,
            false,
        )
        .unwrap();
        assert_eq!(literal.find(&JsString::from("A\n"), 0, false), None);
    }

    #[test]
    fn unicode_class_property_union_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"[\p{ASCII}]",
            r"[^\p{ASCII}]",
            r"[\P{ASCII}K]",
            r"[^\P{ASCII}K]",
            r"[\p{ASCII}\P{ASCII}]",
            r"[\P{Any}K]",
            r"[^\P{Any}K]",
            r"[\p{Any}]",
            r"[\P{ASCII}\w]",
            r"^[\P{ASCII}K]$",
            r"^[\p{ASCII}]$",
        ] {
            for sets in [false, true] {
                for ignore_case in [false, true] {
                    for multiline in [false, true] {
                        let plan = RegExpUnicodeClassMatcher::compile_with_flags(
                            &JsString::from(source),
                            sets,
                            ignore_case,
                            multiline,
                        )
                        .unwrap();
                        for input in ["aSkſK😀\n", "\r\n\u{2028}\u{2029}", "A\nſ\n😀", ""] {
                            let input = JsString::from(input);
                            for start in 0..=input.len() + 1 {
                                for sticky in [false, true] {
                                    writeln!(rows,"{:?} v={sets} i={ignore_case} m={multiline} input={input:?} start={start} sticky={sticky} {:?}",JsString::from(source),plan.find(&input,start,sticky)).unwrap();
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
    fn unicode_class_property_unions_match_independent_membership_for_all_code_points() {
        let sources = [
            r"[\p{ASCII}]",
            r"[^\p{ASCII}]",
            r"[\P{ASCII}K]",
            r"[^\P{ASCII}K]",
            r"[\p{ASCII}\P{ASCII}]",
            r"[\P{Any}K]",
            r"[^\P{Any}K]",
            r"[\P{ASCII}\w]",
        ];
        let mut plans = Vec::new();
        for source in sources {
            for sets in [false, true] {
                for i in [false, true] {
                    plans.push((
                        source,
                        sets,
                        i,
                        RegExpUnicodeClassMatcher::compile_with_flags(
                            &JsString::from(source),
                            sets,
                            i,
                            false,
                        )
                        .unwrap(),
                    ));
                }
            }
        }
        for point in 0..=0x10ffffu32 {
            let units = if point <= 0xffff {
                vec![point as u16]
            } else {
                vec![
                    ((point - 65536) / 1024 + 0xd800) as u16,
                    ((point - 65536) % 1024 + 0xdc00) as u16,
                ]
            };
            let width = units.len();
            let input = JsString::from_code_units(units);
            for (source, sets, i, plan) in &plans {
                let alias = matches!(point, 0x17f | 0x212a);
                let ascii = point <= 127 || (*i && alias);
                let nonascii = if !i {
                    point > 127
                } else if *sets {
                    point > 127 && !alias
                } else {
                    point > 127 || matches!(point, 0x4b | 0x6b | 0x53 | 0x73)
                };
                let k = point == 0x4b || (*i && matches!(point, 0x6b | 0x212a));
                let word =
                    matches!(point,0x30..=0x39|0x41..=0x5a|0x5f|0x61..=0x7a) || (*i && alias);
                let expected = match *source {
                    r"[\p{ASCII}]" => ascii,
                    r"[^\p{ASCII}]" => !ascii,
                    r"[\P{ASCII}K]" => nonascii || k,
                    r"[^\P{ASCII}K]" => !(nonascii || k),
                    r"[\p{ASCII}\P{ASCII}]" => ascii || nonascii,
                    r"[\P{Any}K]" => k,
                    r"[^\P{Any}K]" => !k,
                    r"[\P{ASCII}\w]" => nonascii || word,
                    _ => unreachable!(),
                };
                assert_eq!(
                    plan.find(&input, 0, false),
                    expected.then_some(0..width),
                    "{source} v={sets} i={i} U+{point:X}"
                );
            }
        }
    }

    #[test]
    fn unicode_class_property_repeated_sets_clones_contracts_and_work() {
        let source = JsString::from(
            format!("[{}]", r"\P{ASCII}\p{Any}\P{Any}\p{ASCII}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        let source = JsString::from(r"^[\P{ASCII}K]$");
        let mut calls = 0;
        let plan = RegExpUnicodeClassMatcher::compile_with_flags_and_work(
            &source,
            true,
            true,
            true,
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
                RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                    &source,
                    true,
                    true,
                    true,
                    |_| {
                        n += 1;
                        if n == fail { Err(()) } else { Ok(()) }
                    }
                )
                .is_err()
            );
        }
        assert_eq!(plan.find(&JsString::from("x\nK\n"), 0, false), Some(2..3));
        assert_eq!(plan.find(&JsString::from("x\nſ\n"), 0, false), None);
        for source in [
            r"[\p{Assigned}]",
            r"[\p{Script=Han}]",
            r"[\p{RGI_Emoji}]",
            r"[\p{ASCII}--K]",
            r"[\p{ASCII}&&K]",
            r"[[\p{ASCII}]]",
            r"[K-\p{ASCII}]",
            r"[\p{ASCII}-K]",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    true
                )
                .is_none()
            );
        }
        assert!(
            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                &JsString::from(r"[\p{ASCII}]"),
                true,
                true,
                true
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_ascii_hex_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{ASCII_Hex_Digit}",
                    r"\P{AHex}",
                    r"[\p{AHex}]",
                    r"[^\p{ASCII_Hex_Digit}]",
                    r"[\P{AHex}K]",
                    r"[\p{AHex}K]",
                    r"[\p{AHex}\P{ASCII_Hex_Digit}]",
                    r"[^\P{AHex}K]",
                ] {
                    let pattern = JsString::from(source);
                    let plan = if source.starts_with('[') {
                        RegExpUnicodeClassMatcher::compile_with_flags(
                            &pattern,
                            sets,
                            ignore_case,
                            false,
                        )
                    } else {
                        RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                            &pattern,
                            sets,
                            ignore_case,
                            false,
                        )
                    }
                    .unwrap();
                    for point in 0..=0x10ffff {
                        let hex = matches!(point, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{ASCII_Hex_Digit}" | r"[\p{AHex}]" => hex,
                            r"\P{AHex}" | r"[^\p{ASCII_Hex_Digit}]" => !hex,
                            r"[\P{AHex}K]" => !hex || k,
                            r"[\p{AHex}K]" => hex || k,
                            r"[\p{AHex}\P{ASCII_Hex_Digit}]" => true,
                            r"[^\P{AHex}K]" => hex && !k,
                            _ => unreachable!(),
                        };
                        assert_eq!(
                            plan.matches(point),
                            expected,
                            "{source} v={sets} i={ignore_case} U+{point:X}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_ascii_hex_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{AHex}",
            r"\P{ASCII_Hex_Digit}",
            r"^[\p{ASCII_Hex_Digit}]$",
            r"[\P{AHex}K]",
            r"[^\P{AHex}K]",
        ] {
            for sets in [false, true] {
                for ignore_case in [false, true] {
                    for multiline in [false, true] {
                        let pattern = JsString::from(source);
                        let plan = if source.contains('[') {
                            RegExpUnicodeClassMatcher::compile_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                multiline,
                            )
                        } else {
                            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                multiline,
                            )
                        }
                        .unwrap();
                        for input in [
                            "😀0FaGſK\n",
                            "x\r\nF\u{2028}a\u{2029}",
                            "\u{ff10}\u{ff26}",
                            "",
                        ] {
                            let input = JsString::from(input);
                            for start in [0, 1, 2, input.len(), input.len() + 1] {
                                for sticky in [false, true] {
                                    writeln!(rows,"{pattern:?} v={sets} i={ignore_case} m={multiline} input={input:?} start={start} sticky={sticky} {:?}",plan.find(&input,start,sticky)).unwrap();
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
    fn unicode_ascii_hex_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{AHex}\P{ASCII_Hex_Digit}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{AHex}$", r"^[\P{ASCII_Hex_Digit}K]$"] {
            let source = JsString::from(source);
            let class = source.code_units().contains(&0x5b);
            let mut calls = 0;
            let charge = |_| {
                calls += 1;
                Ok::<(), ()>(())
            };
            let plan = if class {
                RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                    &source, true, true, true, charge,
                )
            } else {
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &source, true, true, true, charge,
                )
            }
            .unwrap()
            .unwrap();
            let count = calls;
            for fail in 1..=count {
                let mut n = 0;
                let charge = |_| {
                    n += 1;
                    if fail == n { Err(()) } else { Ok(()) }
                };
                let result = if class {
                    RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                        &source, true, true, true, charge,
                    )
                } else {
                    RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                        &source, true, true, true, charge,
                    )
                };
                assert!(result.is_err());
            }
            assert_eq!(plan.find(&JsString::from("F\n😀\n"), 0, false), Some(2..4));
        }
        for source in [
            r"[\p{AHex}--A]",
            r"[\p{AHex}&&A]",
            r"[[\p{AHex}]]",
            r"[A-\p{AHex}]",
            r"[\p{AHex}-A]",
            r"[\p{Assigned}]",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    false
                )
                .is_none()
            );
        }
        assert!(
            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                &JsString::from(r"[\p{AHex}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_hex_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Hex_Digit}",
                    r"\P{Hex}",
                    r"[\p{Hex}]",
                    r"[^\p{Hex_Digit}]",
                    r"[\P{Hex}K]",
                    r"[\p{Hex}K]",
                    r"[\p{Hex}\P{Hex_Digit}]",
                    r"[^\P{Hex}K]",
                ] {
                    let pattern = JsString::from(source);
                    let plan = if source.starts_with('[') {
                        RegExpUnicodeClassMatcher::compile_with_flags(
                            &pattern,
                            sets,
                            ignore_case,
                            false,
                        )
                    } else {
                        RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                            &pattern,
                            sets,
                            ignore_case,
                            false,
                        )
                    }
                    .unwrap();
                    for point in 0..=0x10ffff {
                        let hex = matches!(point, 0x30..=0x39 | 0x41..=0x46 | 0x61..=0x66 | 0xff10..=0xff19 | 0xff21..=0xff26 | 0xff41..=0xff46);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Hex_Digit}" | r"[\p{Hex}]" => hex,
                            r"\P{Hex}" | r"[^\p{Hex_Digit}]" => !hex,
                            r"[\P{Hex}K]" => !hex || k,
                            r"[\p{Hex}K]" => hex || k,
                            r"[\p{Hex}\P{Hex_Digit}]" => true,
                            r"[^\P{Hex}K]" => hex && !k,
                            _ => unreachable!(),
                        };
                        assert_eq!(
                            plan.matches(point),
                            expected,
                            "{source} v={sets} i={ignore_case} U+{point:X}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_hex_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Hex}",
            r"\P{Hex_Digit}",
            r"^[\p{Hex_Digit}]$",
            r"[\P{Hex}K]",
            r"[^\P{Hex}K]",
            r"[\p{Hex_Digit}]",
        ] {
            for sets in [false, true] {
                for ignore_case in [false, true] {
                    for multiline in [false, true] {
                        let pattern = JsString::from(source);
                        let plan = if source.contains('[') {
                            RegExpUnicodeClassMatcher::compile_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                multiline,
                            )
                        } else {
                            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                multiline,
                            )
                        }
                        .unwrap();
                        for input in [
                            "😀0FaGſK\n",
                            "x\r\nF\u{2028}a\u{2029}",
                            "\u{ff10}\u{ff26}",
                            "",
                        ] {
                            let input = JsString::from(input);
                            for start in [0, 1, 2, input.len(), input.len() + 1] {
                                for sticky in [false, true] {
                                    writeln!(rows,"{pattern:?} v={sets} i={ignore_case} m={multiline} input={input:?} start={start} sticky={sticky} {:?}",plan.find(&input,start,sticky)).unwrap();
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
    fn unicode_hex_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{Hex}\P{Hex_Digit}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Hex}$", r"^[\P{Hex_Digit}K]$"] {
            let source = JsString::from(source);
            let class = source.code_units().contains(&0x5b);
            let mut calls = 0;
            let charge = |_| {
                calls += 1;
                Ok::<(), ()>(())
            };
            let plan = if class {
                RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                    &source, true, true, true, charge,
                )
            } else {
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &source, true, true, true, charge,
                )
            }
            .unwrap()
            .unwrap();
            let count = calls;
            for fail in 1..=count {
                let mut n = 0;
                let charge = |_| {
                    n += 1;
                    if fail == n { Err(()) } else { Ok(()) }
                };
                let result = if class {
                    RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                        &source, true, true, true, charge,
                    )
                } else {
                    RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                        &source, true, true, true, charge,
                    )
                };
                assert!(result.is_err());
            }
            assert_eq!(plan.find(&JsString::from("F\n😀\n"), 0, false), Some(2..4));
        }
        for source in [
            r"[\p{Hex}--A]",
            r"[\p{Hex}&&A]",
            r"[[\p{Hex}]]",
            r"[A-\p{Hex}]",
            r"[\p{Hex}-A]",
            r"[\p{Assigned}]",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    false
                )
                .is_none()
            );
        }
        assert!(
            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                &JsString::from(r"[\p{Hex}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_whitespace_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{White_Space}",
                    r"\P{space}",
                    r"[\p{space}]",
                    r"[^\p{White_Space}]",
                    r"[\P{space}K]",
                    r"[\p{space}K]",
                    r"[\p{space}\P{White_Space}]",
                    r"[^\P{space}K]",
                ] {
                    let pattern = JsString::from(source);
                    let plan = if source.starts_with('[') {
                        RegExpUnicodeClassMatcher::compile_with_flags(
                            &pattern,
                            sets,
                            ignore_case,
                            false,
                        )
                    } else {
                        RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                            &pattern,
                            sets,
                            ignore_case,
                            false,
                        )
                    }
                    .unwrap();
                    for point in 0..=0x10ffff {
                        let whitespace = matches!(point, 9..=13 | 0x20 | 0x85 | 0xa0 | 0x1680 | 0x2000..=0x200a | 0x2028..=0x2029 | 0x202f | 0x205f | 0x3000);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{White_Space}" | r"[\p{space}]" => whitespace,
                            r"\P{space}" | r"[^\p{White_Space}]" => !whitespace,
                            r"[\P{space}K]" => !whitespace || k,
                            r"[\p{space}K]" => whitespace || k,
                            r"[\p{space}\P{White_Space}]" => true,
                            r"[^\P{space}K]" => whitespace && !k,
                            _ => unreachable!(),
                        };
                        assert_eq!(
                            plan.matches(point),
                            expected,
                            "{source} v={sets} i={ignore_case} U+{point:X}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn unicode_whitespace_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{space}",
            r"\P{White_Space}",
            r"^[\p{White_Space}]$",
            r"[\P{space}K]",
            r"[^\P{space}K]",
            r"[\p{White_Space}]",
        ] {
            for sets in [false, true] {
                for ignore_case in [false, true] {
                    for multiline in [false, true] {
                        let pattern = JsString::from(source);
                        let plan = if source.contains('[') {
                            RegExpUnicodeClassMatcher::compile_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                multiline,
                            )
                        } else {
                            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                multiline,
                            )
                        }
                        .unwrap();
                        for input in [
                            "😀\t\u{85}\u{feff} \u{a0}X\n",
                            "X\r\n \u{2028}\u{85}\u{2029}",
                            "\u{ff10}\u{ff26}",
                            "",
                        ] {
                            let input = JsString::from(input);
                            for start in [0, 1, 2, input.len(), input.len() + 1] {
                                for sticky in [false, true] {
                                    writeln!(rows,"{pattern:?} v={sets} i={ignore_case} m={multiline} input={input:?} start={start} sticky={sticky} {:?}",plan.find(&input,start,sticky)).unwrap();
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
    fn unicode_whitespace_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{space}\P{White_Space}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{space}$", r"^[\P{White_Space}K]$"] {
            let source = JsString::from(source);
            let class = source.code_units().contains(&0x5b);
            let mut calls = 0;
            let charge = |_| {
                calls += 1;
                Ok::<(), ()>(())
            };
            let plan = if class {
                RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                    &source, true, true, true, charge,
                )
            } else {
                RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                    &source, true, true, true, charge,
                )
            }
            .unwrap()
            .unwrap();
            let count = calls;
            for fail in 1..=count {
                let mut n = 0;
                let charge = |_| {
                    n += 1;
                    if fail == n { Err(()) } else { Ok(()) }
                };
                let result = if class {
                    RegExpUnicodeClassMatcher::compile_with_flags_and_work(
                        &source, true, true, true, charge,
                    )
                } else {
                    RegExpUnicodeClassMatcher::compile_binary_property_with_flags_and_work(
                        &source, true, true, true, charge,
                    )
                };
                assert!(result.is_err());
            }
            assert_eq!(plan.find(&JsString::from(" \n😀\n"), 0, false), Some(2..4));
        }
        for source in [
            r"[\p{space}--A]",
            r"[\p{space}&&A]",
            r"[[\p{space}]]",
            r"[A-\p{space}]",
            r"[\p{space}-A]",
            r"[\p{Assigned}]",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    false
                )
                .is_none()
            );
        }
        assert!(
            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                &JsString::from(r"[\p{space}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
}
