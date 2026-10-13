//! Flat Unicode character-set classes (22.2.2.9).
use crate::{
    JsString,
    case_data::SIMPLE_CASE_FOLD,
    regexp_canonicalize_character,
    regexp_literal::{unicode_literal_atom, unicode_start},
    regexp_unicode_character::{
        unicode_assertions_match, unicode_input_character, unicode_word_assertions_with_work,
        unicode_word_boundary,
    },
};
use std::{ops::Range, sync::Arc};

/// Immutable intervals for one flat u/v character set.
///
/// The caller must validate the complete Pattern in its u/v mode. Bare/assertion
/// entry points require no i; flag-aware entry points admit simple/common folding.
/// Unions, ranges and inversion admit all Unicode code points, including lone
/// surrogates. Fixed ASCII, Any, hex-digit, White_Space, Join_Control,
/// noncharacter, Bidi_Control, Quotation_Mark, Pattern_White_Space and
/// Logical_Order_Exception, Variation_Selector and Dash
/// properties are admitted in flat unions
/// and through the standalone binary-property API.
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
    start_word: Option<bool>,
    end_word: Option<bool>,
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
    /// Admits ASCII, Any, ASCII_Hex_Digit/AHex, Hex_Digit/Hex, White_Space/space
    /// Join_Control/Join_C, Noncharacter_Code_Point/NChar, Bidi_Control/Bidi_C
    /// Quotation_Mark/QMark, Pattern_White_Space/Pat_WS, Logical_Order_Exception/LOE
    /// Variation_Selector/VS and Dash.
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
        // Fixed properties use up to eighteen positive or complementary intervals.
        // Charge the bounded append before it can allocate.
        charge(match kind {
            4 => 4,
            6 => 7,
            8 => 11,
            10 => 2,
            12 => 18,
            14 => 5,
            16 => 14,
            18 => 6,
            20 => 8,
            22 => 5,
            24 => 25,
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

    /// Compiles optional word assertions around one flat class or fixed property.
    ///
    /// Requires complete u/v validation. At least one leading or trailing \b/\B
    /// assertion is required; optional ^/$ and m retain their usual context.
    /// Capturing wrappers and other Pattern compositions remain caller proofs.
    pub fn compile_with_word_boundaries_and_work<E>(
        source: &JsString,
        unicode_sets: bool,
        ignore_case: bool,
        multiline: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(assertions) = unicode_word_assertions_with_work(source, &mut charge)? else {
            return Ok(None);
        };
        let matcher = Self::compile_with_flags_and_work(
            &assertions.source,
            unicode_sets,
            ignore_case,
            multiline,
            &mut charge,
        )?;
        let matcher = match matcher {
            Some(matcher) => Some(matcher),
            None => Self::compile_binary_property_with_flags_and_work(
                &assertions.source,
                unicode_sets,
                ignore_case,
                multiline,
                charge,
            )?,
        };
        Ok(matcher.map(|mut matcher| {
            matcher.start_word = assertions.start;
            matcher.end_word = assertions.end;
            matcher
        }))
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
        let mut properties = 0u32;
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
        for kind in 0..26 {
            if properties & (1 << kind) != 0 {
                // Appending may move the original intervals if capacity grows.
                // Charge that move and the fixed set before allocating.
                charge(ranges.len().saturating_add(match kind {
                    12..=13 | 16..=17 => 40,
                    8..=11 | 14..=15 | 18..=19 | 22..=23 => 24,
                    20..=21 => 32,
                    24..=25 => 100,
                    _ => 8,
                }))?;
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
            start_word: None,
            end_word: None,
        }))
    }

    /// Bounds the input scan and binary membership lookup for opt-in search work.
    pub fn search_passes(&self) -> usize {
        2 + lookup_levels(self.ranges.len())
            + usize::from(self.start_anchor)
            + usize::from(self.end_anchor)
            + 8 * (usize::from(self.start_word.is_some()) + usize::from(self.end_word.is_some()))
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
            ) && self.start_word.is_none_or(|expected| {
                unicode_word_boundary(units, cursor, self.ignore_case) == expected
            }) && self.end_word.is_none_or(|expected| {
                unicode_word_boundary(units, end, self.ignore_case) == expected
            }) && self.matches(value)
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
        [0x4a, 0x6f, 0x69, 0x6e, 0x5f, 0x43]
        | [
            0x4a,
            0x6f,
            0x69,
            0x6e,
            0x5f,
            0x43,
            0x6f,
            0x6e,
            0x74,
            0x72,
            0x6f,
            0x6c,
        ] => 10,
        [0x4e, 0x43, 0x68, 0x61, 0x72]
        | [
            0x4e,
            0x6f,
            0x6e,
            0x63,
            0x68,
            0x61,
            0x72,
            0x61,
            0x63,
            0x74,
            0x65,
            0x72,
            0x5f,
            0x43,
            0x6f,
            0x64,
            0x65,
            0x5f,
            0x50,
            0x6f,
            0x69,
            0x6e,
            0x74,
        ] => 12,
        [0x42, 0x69, 0x64, 0x69, 0x5f, 0x43]
        | [
            0x42,
            0x69,
            0x64,
            0x69,
            0x5f,
            0x43,
            0x6f,
            0x6e,
            0x74,
            0x72,
            0x6f,
            0x6c,
        ] => 14,
        [0x51, 0x4d, 0x61, 0x72, 0x6b]
        | [
            0x51,
            0x75,
            0x6f,
            0x74,
            0x61,
            0x74,
            0x69,
            0x6f,
            0x6e,
            0x5f,
            0x4d,
            0x61,
            0x72,
            0x6b,
        ] => 16,
        [0x50, 0x61, 0x74, 0x5f, 0x57, 0x53]
        | [
            0x50,
            0x61,
            0x74,
            0x74,
            0x65,
            0x72,
            0x6e,
            0x5f,
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
        ] => 18,
        [0x4c, 0x4f, 0x45]
        | [
            0x4c,
            0x6f,
            0x67,
            0x69,
            0x63,
            0x61,
            0x6c,
            0x5f,
            0x4f,
            0x72,
            0x64,
            0x65,
            0x72,
            0x5f,
            0x45,
            0x78,
            0x63,
            0x65,
            0x70,
            0x74,
            0x69,
            0x6f,
            0x6e,
        ] => 20,
        [0x56, 0x53]
        | [
            0x56,
            0x61,
            0x72,
            0x69,
            0x61,
            0x74,
            0x69,
            0x6f,
            0x6e,
            0x5f,
            0x53,
            0x65,
            0x6c,
            0x65,
            0x63,
            0x74,
            0x6f,
            0x72,
        ] => 22,
        [0x44, 0x61, 0x73, 0x68] => 24,
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
        // Pinned UCD Join_Control: ZWNJ and ZWJ, both invariant under C/S folding.
        10 => &[(0x200c, 0x200d)],
        11 => &[(0, 0x200b), (0x200e, 0x10ffff)],
        // Pinned UCD noncharacters: FDD0..FDEF and the final two points of every plane.
        12 => &[
            (0xfdd0, 0xfdef),
            (0xfffe, 0xffff),
            (0x1fffe, 0x1ffff),
            (0x2fffe, 0x2ffff),
            (0x3fffe, 0x3ffff),
            (0x4fffe, 0x4ffff),
            (0x5fffe, 0x5ffff),
            (0x6fffe, 0x6ffff),
            (0x7fffe, 0x7ffff),
            (0x8fffe, 0x8ffff),
            (0x9fffe, 0x9ffff),
            (0xafffe, 0xaffff),
            (0xbfffe, 0xbffff),
            (0xcfffe, 0xcffff),
            (0xdfffe, 0xdffff),
            (0xefffe, 0xeffff),
            (0xffffe, 0xfffff),
            (0x10fffe, 0x10ffff),
        ],
        13 => &[
            (0x0, 0xfdcf),
            (0xfdf0, 0xfffd),
            (0x10000, 0x1fffd),
            (0x20000, 0x2fffd),
            (0x30000, 0x3fffd),
            (0x40000, 0x4fffd),
            (0x50000, 0x5fffd),
            (0x60000, 0x6fffd),
            (0x70000, 0x7fffd),
            (0x80000, 0x8fffd),
            (0x90000, 0x9fffd),
            (0xa0000, 0xafffd),
            (0xb0000, 0xbfffd),
            (0xc0000, 0xcfffd),
            (0xd0000, 0xdfffd),
            (0xe0000, 0xefffd),
            (0xf0000, 0xffffd),
            (0x100000, 0x10fffd),
        ],
        // UCD 18 Bidi_Control: twelve controls, invariant under C/S folding.
        14 => &[
            (0x61c, 0x61c),
            (0x200e, 0x200f),
            (0x202a, 0x202e),
            (0x2066, 0x2069),
        ],
        15 => &[
            (0, 0x61b),
            (0x61d, 0x200d),
            (0x2010, 0x2029),
            (0x202f, 0x2065),
            (0x206a, 0x10ffff),
        ],
        // UCD 18 Quotation_Mark: thirty members, invariant under C/S folding.
        16 => &[
            (0x22, 0x22),
            (0x27, 0x27),
            (0xab, 0xab),
            (0xbb, 0xbb),
            (0x2018, 0x201f),
            (0x2039, 0x203a),
            (0x2e42, 0x2e42),
            (0x300c, 0x300f),
            (0x301d, 0x301f),
            (0xfe41, 0xfe44),
            (0xff02, 0xff02),
            (0xff07, 0xff07),
            (0xff62, 0xff63),
        ],
        17 => &[
            (0x0, 0x21),
            (0x23, 0x26),
            (0x28, 0xaa),
            (0xac, 0xba),
            (0xbc, 0x2017),
            (0x2020, 0x2038),
            (0x203b, 0x2e41),
            (0x2e43, 0x300b),
            (0x3010, 0x301c),
            (0x3020, 0xfe40),
            (0xfe45, 0xff01),
            (0xff03, 0xff06),
            (0xff08, 0xff61),
            (0xff64, 0x10ffff),
        ],
        // UCD 18 Pattern_White_Space: eleven members, invariant under C/S folding.
        18 => &[
            (0x9, 0xd),
            (0x20, 0x20),
            (0x85, 0x85),
            (0x200e, 0x200f),
            (0x2028, 0x2029),
        ],
        19 => &[
            (0x0, 0x8),
            (0xe, 0x1f),
            (0x21, 0x84),
            (0x86, 0x200d),
            (0x2010, 0x2027),
            (0x202a, 0x10ffff),
        ],
        // UCD 18 Logical_Order_Exception: nineteen members, invariant under C/S folding.
        20 => &[
            (0xe40, 0xe44),
            (0xec0, 0xec4),
            (0x19b5, 0x19b7),
            (0x19ba, 0x19ba),
            (0xaab5, 0xaab6),
            (0xaab9, 0xaab9),
            (0xaabb, 0xaabc),
        ],
        21 => &[
            (0x0, 0xe3f),
            (0xe45, 0xebf),
            (0xec5, 0x19b4),
            (0x19b8, 0x19b9),
            (0x19bb, 0xaab4),
            (0xaab7, 0xaab8),
            (0xaaba, 0xaaba),
            (0xaabd, 0x10ffff),
        ],
        // UCD 18 Variation_Selector: 260 members, invariant under C/S folding.
        22 => &[
            (0x180b, 0x180d),
            (0x180f, 0x180f),
            (0xfe00, 0xfe0f),
            (0xe0100, 0xe01ef),
        ],
        23 => &[
            (0x0, 0x180a),
            (0x180e, 0x180e),
            (0x1810, 0xfdff),
            (0xfe10, 0xe00ff),
            (0xe01f0, 0x10ffff),
        ],
        // UCD 18 Dash: 31 members, invariant under C/S folding.
        24 => &[
            (0x2d, 0x2d),
            (0x58a, 0x58a),
            (0x5be, 0x5be),
            (0x1400, 0x1400),
            (0x1806, 0x1806),
            (0x2010, 0x2015),
            (0x2053, 0x2053),
            (0x207b, 0x207b),
            (0x208b, 0x208b),
            (0x2212, 0x2212),
            (0x2e17, 0x2e17),
            (0x2e1a, 0x2e1a),
            (0x2e3a, 0x2e3b),
            (0x2e40, 0x2e40),
            (0x2e5d, 0x2e5d),
            (0x301c, 0x301c),
            (0x3030, 0x3030),
            (0x30a0, 0x30a0),
            (0xfe31, 0xfe32),
            (0xfe58, 0xfe58),
            (0xfe63, 0xfe63),
            (0xff0d, 0xff0d),
            (0x10d6e, 0x10d6e),
            (0x10ead, 0x10ead),
        ],
        25 => &[
            (0x0, 0x2c),
            (0x2e, 0x589),
            (0x58b, 0x5bd),
            (0x5bf, 0x13ff),
            (0x1401, 0x1805),
            (0x1807, 0x200f),
            (0x2016, 0x2052),
            (0x2054, 0x207a),
            (0x207c, 0x208a),
            (0x208c, 0x2211),
            (0x2213, 0x2e16),
            (0x2e18, 0x2e19),
            (0x2e1b, 0x2e39),
            (0x2e3c, 0x2e3f),
            (0x2e41, 0x2e5c),
            (0x2e5e, 0x301b),
            (0x301d, 0x302f),
            (0x3031, 0x309f),
            (0x30a1, 0xfe30),
            (0xfe33, 0xfe57),
            (0xfe59, 0xfe62),
            (0xfe64, 0xff0c),
            (0xff0e, 0x10d6d),
            (0x10d6f, 0x10eac),
            (0x10eae, 0x10ffff),
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
    fn word_plan(source: &str, sets: bool, i: bool, m: bool) -> RegExpUnicodeClassMatcher {
        RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
            &JsString::from(source),
            sets,
            i,
            m,
            |_| Ok::<(), std::convert::Infallible>(()),
        )
        .unwrap()
        .unwrap()
    }

    #[test]
    fn unicode_class_word_boundary_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\b[A]\b",
            r"\B[\p{Any}]\B",
            r"^\b[\p{AHex}K]\b$",
            r"\b\p{space}\B",
            r"\B\P{space}\b",
            r"\b[\b]\B",
            r"\b[A]",
            r"[\p{Any}]\B",
        ] {
            for sets in [false, true] {
                for i in [false, true] {
                    for m in [false, true] {
                        let plan = word_plan(source, sets, i, m);
                        for text in [
                            "A😀B a_ſK\n",
                            "x\r\nK\u{2028}A\u{2029}",
                            "A\u{8}!\u{85} ",
                            "\u{d7ff}",
                            "",
                        ] {
                            let input = JsString::from(text);
                            for start in [0, 1, 2, input.len(), input.len() + 1] {
                                for sticky in [false, true] {
                                    writeln!(rows,"{:?} v={sets} i={i} m={m} input={input:?} start={start} sticky={sticky} {:?}",JsString::from(source),plan.find(&input,start,sticky)).unwrap();
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
    fn unicode_class_word_boundaries_agree_with_independent_raw_candidates() {
        let alphabet = [
            0x41, 0x61, 0x5f, 8, 10, 0x20, 0x85, 0x17f, 0x212a, 0xd83d, 0xde00,
        ];
        let cases = [
            (r"\b[A]\b", 1, Some(true), Some(true), false, false),
            (r"\B[\p{Any}]\B", 2, Some(false), Some(false), false, false),
            (r"^\b[\p{AHex}K]\b$", 3, Some(true), Some(true), true, true),
            (r"\b\P{Any}\b", 4, Some(true), Some(true), false, false),
            (r"\b\p{space}\B", 5, Some(true), Some(false), false, false),
            (r"\B\P{space}\b", 6, Some(false), Some(true), false, false),
            (r"\b[\b]\B", 7, Some(true), Some(false), false, false),
            (r"\b[A]", 1, Some(true), None, false, false),
            (r"[\p{Any}]\B", 2, None, Some(false), false, false),
        ];
        let word = |value: Option<&u16>, i| {
            value.is_some_and(|&v| {
                char::from_u32(u32::from(v)).is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
                    || (i && [0x17f, 0x212a].contains(&v))
            })
        };
        let line = |v: u16| [10, 13, 0x2028, 0x2029].contains(&v);
        for sets in [false, true] {
            for i in [false, true] {
                for m in [false, true] {
                    let plans: Vec<_> = cases
                        .iter()
                        .map(|case| word_plan(case.0, sets, i, m))
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
                                                let space = [
                                                    9, 10, 11, 12, 13, 0x20, 0x85, 0xa0, 0x1680,
                                                    0x2028, 0x2029, 0x202f, 0x205f, 0x3000,
                                                ]
                                                .contains(&point)
                                                    || (0x2000..=0x200a).contains(&point);
                                                let member = match case.1 {
                                                    1 => point == 0x41 || (i && point == 0x61),
                                                    2 => true,
                                                    3 => {
                                                        matches!(point,0x30..=0x39|0x41..=0x46|0x61..=0x66|0x4b)
                                                            || (i
                                                                && [0x6b, 0x212a].contains(&point))
                                                    }
                                                    4 => false,
                                                    5 => space,
                                                    6 => !space,
                                                    7 => point == 8,
                                                    _ => unreachable!(),
                                                };
                                                let at = |position: usize| {
                                                    word(
                                                        position
                                                            .checked_sub(1)
                                                            .and_then(|j| units.get(j)),
                                                        i,
                                                    ) != word(units.get(position), i)
                                                };
                                                let begin_ok = !case.4
                                                    || cursor == 0
                                                    || (m && line(units[cursor - 1]));
                                                let end_ok = !case.5
                                                    || end == units.len()
                                                    || (m && line(units[end]));
                                                let word_ok =
                                                    case.2.is_none_or(|b| at(cursor) == b)
                                                        && case.3.is_none_or(|b| at(end) == b);
                                                if member && begin_ok && end_ok && word_ok {
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
                                            "{} v={sets} i={i} m={m} {input:?} start={start} sticky={sticky}",
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
    fn unicode_class_word_boundaries_work_clones_and_complete_body_contracts() {
        let source = JsString::from(format!(r"\b[{}]\B", r"\p{Any}".repeat(10000)).as_str());
        let plan = RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
            &source,
            true,
            true,
            false,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from(" A_"), 0, false),
            Some(1..2)
        );
        let source = JsString::from(r"^\b[\p{AHex}K]\b$");
        let mut calls = 0;
        let plan = RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
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
                RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
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
        for source in [
            r"[A]",
            r"\bA\b",
            r"\b([A])\b",
            r"\b[A]+\b",
            r"\b[[A]]\b",
            r"\b[A]--[B]\b",
            r"\b[A]\\b",
            r"\b\b[A]\b",
            r"\b\B",
            r"\b[\p{Assigned}]\b",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_word_boundaries_and_work(
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
            RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"\b[A]\b"),
                true,
                true,
                true
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_join_control_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Join_Control}",
                    r"\P{Join_C}",
                    r"[\p{Join_C}]",
                    r"[^\p{Join_Control}]",
                    r"[\P{Join_C}K]",
                    r"[\p{Join_C}K]",
                    r"[\p{Join_C}\P{Join_Control}]",
                    r"[^\P{Join_C}K]",
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
                        let whitespace = matches!(point, 0x200c..=0x200d);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Join_Control}" | r"[\p{Join_C}]" => whitespace,
                            r"\P{Join_C}" | r"[^\p{Join_Control}]" => !whitespace,
                            r"[\P{Join_C}K]" => !whitespace || k,
                            r"[\p{Join_C}K]" => whitespace || k,
                            r"[\p{Join_C}\P{Join_Control}]" => true,
                            r"[^\P{Join_C}K]" => whitespace && !k,
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
    fn unicode_join_control_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Join_C}",
            r"\P{Join_Control}",
            r"^[\p{Join_Control}]$",
            r"[\P{Join_C}K]",
            r"[^\P{Join_C}K]",
            r"[\p{Join_Control}]",
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
                            "😀\u{200b}\u{200c}\u{200d}\u{200e}ſK\n",
                            "X\r\n\u{200c}\u{2028}\u{200d}\u{2029}",
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
    fn unicode_join_control_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{Join_C}\P{Join_Control}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Join_C}$", r"^[\P{Join_Control}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{200c}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Join_C}--A]",
            r"[\p{Join_C}&&A]",
            r"[[\p{Join_C}]]",
            r"[A-\p{Join_C}]",
            r"[\p{Join_C}-A]",
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
                &JsString::from(r"[\p{Join_C}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_noncharacter_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Noncharacter_Code_Point}",
                    r"\P{NChar}",
                    r"[\p{NChar}]",
                    r"[^\p{Noncharacter_Code_Point}]",
                    r"[\P{NChar}K]",
                    r"[\p{NChar}K]",
                    r"[\p{NChar}\P{Noncharacter_Code_Point}]",
                    r"[^\P{NChar}K]",
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
                        let whitespace =
                            matches!(point, 0xfdd0..=0xfdef) || (point & 0xffff) >= 0xfffe;
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Noncharacter_Code_Point}" | r"[\p{NChar}]" => whitespace,
                            r"\P{NChar}" | r"[^\p{Noncharacter_Code_Point}]" => !whitespace,
                            r"[\P{NChar}K]" => !whitespace || k,
                            r"[\p{NChar}K]" => whitespace || k,
                            r"[\p{NChar}\P{Noncharacter_Code_Point}]" => true,
                            r"[^\P{NChar}K]" => whitespace && !k,
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
    fn unicode_noncharacter_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{NChar}",
            r"\P{Noncharacter_Code_Point}",
            r"^[\p{Noncharacter_Code_Point}]$",
            r"[\P{NChar}K]",
            r"[^\P{NChar}K]",
            r"[\p{Noncharacter_Code_Point}]",
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
                            "😀\u{fdcf}\u{fdd0}\u{fdef}\u{fdf0}\u{fffe}\u{ffff}ſK\n",
                            "X\r\n\u{1fffe}\u{2028}\u{10ffff}\u{2029}",
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
    fn unicode_noncharacter_property_work_deduplication_and_contracts() {
        let source = JsString::from(
            format!(
                "[{}]",
                r"\p{NChar}\P{Noncharacter_Code_Point}".repeat(10000)
            )
            .as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{NChar}$", r"^[\P{Noncharacter_Code_Point}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{fdd0}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{NChar}--A]",
            r"[\p{NChar}&&A]",
            r"[[\p{NChar}]]",
            r"[A-\p{NChar}]",
            r"[\p{NChar}-A]",
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
                &JsString::from(r"[\p{NChar}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_bidi_control_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Bidi_Control}",
                    r"\P{Bidi_C}",
                    r"[\p{Bidi_C}]",
                    r"[^\p{Bidi_Control}]",
                    r"[\P{Bidi_C}K]",
                    r"[\p{Bidi_C}K]",
                    r"[\p{Bidi_C}\P{Bidi_Control}]",
                    r"[^\P{Bidi_C}K]",
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
                        let bidi_control = [
                            0x61c, 0x200e, 0x200f, 0x202a, 0x202b, 0x202c, 0x202d, 0x202e, 0x2066,
                            0x2067, 0x2068, 0x2069,
                        ]
                        .contains(&point);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Bidi_Control}" | r"[\p{Bidi_C}]" => bidi_control,
                            r"\P{Bidi_C}" | r"[^\p{Bidi_Control}]" => !bidi_control,
                            r"[\P{Bidi_C}K]" => !bidi_control || k,
                            r"[\p{Bidi_C}K]" => bidi_control || k,
                            r"[\p{Bidi_C}\P{Bidi_Control}]" => true,
                            r"[^\P{Bidi_C}K]" => bidi_control && !k,
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
    fn unicode_bidi_control_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Bidi_C}",
            r"\P{Bidi_Control}",
            r"^[\p{Bidi_Control}]$",
            r"[\P{Bidi_C}K]",
            r"[^\P{Bidi_C}K]",
            r"[\p{Bidi_Control}]",
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
                            "😀\u{61b}\u{61c}\u{61d}\u{200d}\u{200e}\u{200f}\u{2010}\u{2029}\u{202a}\u{202b}\u{202c}\u{202d}\u{202e}\u{202f}\u{2065}\u{2066}\u{2067}\u{2068}\u{2069}\u{206a}ſK\n",
                            "X\r\n\u{200e}\u{2028}\u{2069}\u{2029}",
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
    fn unicode_bidi_control_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{Bidi_C}\P{Bidi_Control}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Bidi_C}$", r"^[\P{Bidi_Control}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{200e}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Bidi_C}--A]",
            r"[\p{Bidi_C}&&A]",
            r"[[\p{Bidi_C}]]",
            r"[A-\p{Bidi_C}]",
            r"[\p{Bidi_C}-A]",
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
                &JsString::from(r"[\p{Bidi_C}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_quotation_mark_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Quotation_Mark}",
                    r"\P{QMark}",
                    r"[\p{QMark}]",
                    r"[^\p{Quotation_Mark}]",
                    r"[\P{QMark}K]",
                    r"[\p{QMark}K]",
                    r"[\p{QMark}\P{Quotation_Mark}]",
                    r"[^\P{QMark}K]",
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
                        let quotation_mark = [
                            0x22, 0x27, 0xab, 0xbb, 0x2018, 0x2019, 0x201a, 0x201b, 0x201c, 0x201d,
                            0x201e, 0x201f, 0x2039, 0x203a, 0x2e42, 0x300c, 0x300d, 0x300e, 0x300f,
                            0x301d, 0x301e, 0x301f, 0xfe41, 0xfe42, 0xfe43, 0xfe44, 0xff02, 0xff07,
                            0xff62, 0xff63,
                        ]
                        .contains(&point);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Quotation_Mark}" | r"[\p{QMark}]" => quotation_mark,
                            r"\P{QMark}" | r"[^\p{Quotation_Mark}]" => !quotation_mark,
                            r"[\P{QMark}K]" => !quotation_mark || k,
                            r"[\p{QMark}K]" => quotation_mark || k,
                            r"[\p{QMark}\P{Quotation_Mark}]" => true,
                            r"[^\P{QMark}K]" => quotation_mark && !k,
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
    fn unicode_quotation_mark_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{QMark}",
            r"\P{Quotation_Mark}",
            r"^[\p{Quotation_Mark}]$",
            r"[\P{QMark}K]",
            r"[^\P{QMark}K]",
            r"[\p{Quotation_Mark}]",
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
                            "😀\u{21}\u{22}\u{23}\u{26}\u{27}\u{28}\u{ab}\u{bb}\u{2017}\u{2018}\u{2019}\u{201a}\u{201b}\u{201c}\u{201d}\u{201e}\u{201f}\u{2039}\u{203a}\u{2e42}\u{300c}\u{300d}\u{300e}\u{300f}\u{301d}\u{301e}\u{301f}\u{fe41}\u{fe42}\u{fe43}\u{fe44}\u{ff02}\u{ff07}\u{ff62}\u{ff63}\u{2010}\u{300b}\u{3010}\u{ff64}ſK\n",
                            "X\r\n\u{201c}\u{2028}\u{301f}\u{2029}",
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
    fn unicode_quotation_mark_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{QMark}\P{Quotation_Mark}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{QMark}$", r"^[\P{Quotation_Mark}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{201c}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{QMark}--A]",
            r"[\p{QMark}&&A]",
            r"[[\p{QMark}]]",
            r"[A-\p{QMark}]",
            r"[\p{QMark}-A]",
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
                &JsString::from(r"[\p{QMark}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_pattern_white_space_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Pattern_White_Space}",
                    r"\P{Pat_WS}",
                    r"[\p{Pat_WS}]",
                    r"[^\p{Pattern_White_Space}]",
                    r"[\P{Pat_WS}K]",
                    r"[\p{Pat_WS}K]",
                    r"[\p{Pat_WS}\P{Pattern_White_Space}]",
                    r"[^\P{Pat_WS}K]",
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
                        let pattern_white_space = [
                            0x9, 0xa, 0xb, 0xc, 0xd, 0x20, 0x85, 0x200e, 0x200f, 0x2028, 0x2029,
                        ]
                        .contains(&point);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Pattern_White_Space}" | r"[\p{Pat_WS}]" => pattern_white_space,
                            r"\P{Pat_WS}" | r"[^\p{Pattern_White_Space}]" => !pattern_white_space,
                            r"[\P{Pat_WS}K]" => !pattern_white_space || k,
                            r"[\p{Pat_WS}K]" => pattern_white_space || k,
                            r"[\p{Pat_WS}\P{Pattern_White_Space}]" => true,
                            r"[^\P{Pat_WS}K]" => pattern_white_space && !k,
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
    fn unicode_pattern_white_space_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Pat_WS}",
            r"\P{Pattern_White_Space}",
            r"^[\p{Pattern_White_Space}]$",
            r"[\P{Pat_WS}K]",
            r"[^\P{Pat_WS}K]",
            r"[\p{Pattern_White_Space}]",
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
                            "😀\u{9}\u{a}\u{b}\u{c}\u{d}\u{20}\u{85}\u{200e}\u{200f}\u{2028}\u{2029}\u{8}\u{e}\u{1f}\u{21}\u{84}\u{86}\u{200d}\u{2010}\u{2027}\u{202a}\u{feff}ſK\n",
                            "X\r\n\u{200e}\u{2028}\u{200f}\u{2029}",
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
    fn unicode_pattern_white_space_property_work_deduplication_and_contracts() {
        let source = JsString::from(
            format!("[{}]", r"\p{Pat_WS}\P{Pattern_White_Space}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Pat_WS}$", r"^[\P{Pattern_White_Space}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{200e}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Pat_WS}--A]",
            r"[\p{Pat_WS}&&A]",
            r"[[\p{Pat_WS}]]",
            r"[A-\p{Pat_WS}]",
            r"[\p{Pat_WS}-A]",
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
                &JsString::from(r"[\p{Pat_WS}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_logical_order_exception_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Logical_Order_Exception}",
                    r"\P{LOE}",
                    r"[\p{LOE}]",
                    r"[^\p{Logical_Order_Exception}]",
                    r"[\P{LOE}K]",
                    r"[\p{LOE}K]",
                    r"[\p{LOE}\P{Logical_Order_Exception}]",
                    r"[^\P{LOE}K]",
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
                        let logical_order_exception = [
                            0xe40, 0xe41, 0xe42, 0xe43, 0xe44, 0xec0, 0xec1, 0xec2, 0xec3, 0xec4,
                            0x19b5, 0x19b6, 0x19b7, 0x19ba, 0xaab5, 0xaab6, 0xaab9, 0xaabb, 0xaabc,
                        ]
                        .contains(&point);
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Logical_Order_Exception}" | r"[\p{LOE}]" => {
                                logical_order_exception
                            }
                            r"\P{LOE}" | r"[^\p{Logical_Order_Exception}]" => {
                                !logical_order_exception
                            }
                            r"[\P{LOE}K]" => !logical_order_exception || k,
                            r"[\p{LOE}K]" => logical_order_exception || k,
                            r"[\p{LOE}\P{Logical_Order_Exception}]" => true,
                            r"[^\P{LOE}K]" => logical_order_exception && !k,
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
    fn unicode_logical_order_exception_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{LOE}",
            r"\P{Logical_Order_Exception}",
            r"^[\p{Logical_Order_Exception}]$",
            r"[\P{LOE}K]",
            r"[^\P{LOE}K]",
            r"[\p{Logical_Order_Exception}]",
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
                            "😀\u{e40}\u{e41}\u{e42}\u{e43}\u{e44}\u{ec0}\u{ec1}\u{ec2}\u{ec3}\u{ec4}\u{19b5}\u{19b6}\u{19b7}\u{19ba}\u{aab5}\u{aab6}\u{aab9}\u{aabb}\u{aabc}\u{e3f}\u{e45}\u{ebf}\u{ec5}\u{19b4}\u{19b8}\u{19b9}\u{19bb}\u{aab4}\u{aab7}\u{aab8}\u{aaba}\u{aabd}\u{feff}ſK\n",
                            "X\r\n\u{e40}\u{2028}\u{ec0}\u{2029}",
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
    fn unicode_logical_order_exception_property_work_deduplication_and_contracts() {
        let source = JsString::from(
            format!("[{}]", r"\p{LOE}\P{Logical_Order_Exception}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{LOE}$", r"^[\P{Logical_Order_Exception}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{e40}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{LOE}--A]",
            r"[\p{LOE}&&A]",
            r"[[\p{LOE}]]",
            r"[A-\p{LOE}]",
            r"[\p{LOE}-A]",
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
                &JsString::from(r"[\p{LOE}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_variation_selector_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Variation_Selector}",
                    r"\P{VS}",
                    r"[\p{VS}]",
                    r"[^\p{Variation_Selector}]",
                    r"[\P{VS}K]",
                    r"[\p{VS}K]",
                    r"[\p{VS}\P{Variation_Selector}]",
                    r"[^\P{VS}K]",
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
                        let variation_selector = [
                            0x180b, 0x180c, 0x180d, 0x180f, 0xfe00, 0xfe01, 0xfe02, 0xfe03, 0xfe04,
                            0xfe05, 0xfe06, 0xfe07, 0xfe08, 0xfe09, 0xfe0a, 0xfe0b, 0xfe0c, 0xfe0d,
                            0xfe0e, 0xfe0f, 0xe0100, 0xe0101, 0xe0102, 0xe0103, 0xe0104, 0xe0105,
                            0xe0106, 0xe0107, 0xe0108, 0xe0109, 0xe010a, 0xe010b, 0xe010c, 0xe010d,
                            0xe010e, 0xe010f, 0xe0110, 0xe0111, 0xe0112, 0xe0113, 0xe0114, 0xe0115,
                            0xe0116, 0xe0117, 0xe0118, 0xe0119, 0xe011a, 0xe011b, 0xe011c, 0xe011d,
                            0xe011e, 0xe011f, 0xe0120, 0xe0121, 0xe0122, 0xe0123, 0xe0124, 0xe0125,
                            0xe0126, 0xe0127, 0xe0128, 0xe0129, 0xe012a, 0xe012b, 0xe012c, 0xe012d,
                            0xe012e, 0xe012f, 0xe0130, 0xe0131, 0xe0132, 0xe0133, 0xe0134, 0xe0135,
                            0xe0136, 0xe0137, 0xe0138, 0xe0139, 0xe013a, 0xe013b, 0xe013c, 0xe013d,
                            0xe013e, 0xe013f, 0xe0140, 0xe0141, 0xe0142, 0xe0143, 0xe0144, 0xe0145,
                            0xe0146, 0xe0147, 0xe0148, 0xe0149, 0xe014a, 0xe014b, 0xe014c, 0xe014d,
                            0xe014e, 0xe014f, 0xe0150, 0xe0151, 0xe0152, 0xe0153, 0xe0154, 0xe0155,
                            0xe0156, 0xe0157, 0xe0158, 0xe0159, 0xe015a, 0xe015b, 0xe015c, 0xe015d,
                            0xe015e, 0xe015f, 0xe0160, 0xe0161, 0xe0162, 0xe0163, 0xe0164, 0xe0165,
                            0xe0166, 0xe0167, 0xe0168, 0xe0169, 0xe016a, 0xe016b, 0xe016c, 0xe016d,
                            0xe016e, 0xe016f, 0xe0170, 0xe0171, 0xe0172, 0xe0173, 0xe0174, 0xe0175,
                            0xe0176, 0xe0177, 0xe0178, 0xe0179, 0xe017a, 0xe017b, 0xe017c, 0xe017d,
                            0xe017e, 0xe017f, 0xe0180, 0xe0181, 0xe0182, 0xe0183, 0xe0184, 0xe0185,
                            0xe0186, 0xe0187, 0xe0188, 0xe0189, 0xe018a, 0xe018b, 0xe018c, 0xe018d,
                            0xe018e, 0xe018f, 0xe0190, 0xe0191, 0xe0192, 0xe0193, 0xe0194, 0xe0195,
                            0xe0196, 0xe0197, 0xe0198, 0xe0199, 0xe019a, 0xe019b, 0xe019c, 0xe019d,
                            0xe019e, 0xe019f, 0xe01a0, 0xe01a1, 0xe01a2, 0xe01a3, 0xe01a4, 0xe01a5,
                            0xe01a6, 0xe01a7, 0xe01a8, 0xe01a9, 0xe01aa, 0xe01ab, 0xe01ac, 0xe01ad,
                            0xe01ae, 0xe01af, 0xe01b0, 0xe01b1, 0xe01b2, 0xe01b3, 0xe01b4, 0xe01b5,
                            0xe01b6, 0xe01b7, 0xe01b8, 0xe01b9, 0xe01ba, 0xe01bb, 0xe01bc, 0xe01bd,
                            0xe01be, 0xe01bf, 0xe01c0, 0xe01c1, 0xe01c2, 0xe01c3, 0xe01c4, 0xe01c5,
                            0xe01c6, 0xe01c7, 0xe01c8, 0xe01c9, 0xe01ca, 0xe01cb, 0xe01cc, 0xe01cd,
                            0xe01ce, 0xe01cf, 0xe01d0, 0xe01d1, 0xe01d2, 0xe01d3, 0xe01d4, 0xe01d5,
                            0xe01d6, 0xe01d7, 0xe01d8, 0xe01d9, 0xe01da, 0xe01db, 0xe01dc, 0xe01dd,
                            0xe01de, 0xe01df, 0xe01e0, 0xe01e1, 0xe01e2, 0xe01e3, 0xe01e4, 0xe01e5,
                            0xe01e6, 0xe01e7, 0xe01e8, 0xe01e9, 0xe01ea, 0xe01eb, 0xe01ec, 0xe01ed,
                            0xe01ee, 0xe01ef,
                        ]
                        .binary_search(&point)
                        .is_ok();
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Variation_Selector}" | r"[\p{VS}]" => variation_selector,
                            r"\P{VS}" | r"[^\p{Variation_Selector}]" => !variation_selector,
                            r"[\P{VS}K]" => !variation_selector || k,
                            r"[\p{VS}K]" => variation_selector || k,
                            r"[\p{VS}\P{Variation_Selector}]" => true,
                            r"[^\P{VS}K]" => variation_selector && !k,
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
    fn unicode_variation_selector_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{VS}",
            r"\P{Variation_Selector}",
            r"^[\p{Variation_Selector}]$",
            r"[\P{VS}K]",
            r"[^\P{VS}K]",
            r"[\p{Variation_Selector}]",
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
                            "😀\u{180b}\u{180c}\u{180d}\u{180f}\u{fe00}\u{fe01}\u{fe02}\u{fe03}\u{fe04}\u{fe05}\u{fe06}\u{fe07}\u{fe08}\u{fe09}\u{fe0a}\u{fe0b}\u{fe0c}\u{fe0d}\u{fe0e}\u{fe0f}\u{e0100}\u{e0101}\u{e0102}\u{e0103}\u{e0104}\u{e0105}\u{e0106}\u{e0107}\u{e0108}\u{e0109}\u{e010a}\u{e010b}\u{e010c}\u{e010d}\u{e010e}\u{e010f}\u{e0110}\u{e0111}\u{e0112}\u{e0113}\u{e0114}\u{e0115}\u{e0116}\u{e0117}\u{e0118}\u{e0119}\u{e011a}\u{e011b}\u{e011c}\u{e011d}\u{e011e}\u{e011f}\u{e0120}\u{e0121}\u{e0122}\u{e0123}\u{e0124}\u{e0125}\u{e0126}\u{e0127}\u{e0128}\u{e0129}\u{e012a}\u{e012b}\u{e012c}\u{e012d}\u{e012e}\u{e012f}\u{e0130}\u{e0131}\u{e0132}\u{e0133}\u{e0134}\u{e0135}\u{e0136}\u{e0137}\u{e0138}\u{e0139}\u{e013a}\u{e013b}\u{e013c}\u{e013d}\u{e013e}\u{e013f}\u{e0140}\u{e0141}\u{e0142}\u{e0143}\u{e0144}\u{e0145}\u{e0146}\u{e0147}\u{e0148}\u{e0149}\u{e014a}\u{e014b}\u{e014c}\u{e014d}\u{e014e}\u{e014f}\u{e0150}\u{e0151}\u{e0152}\u{e0153}\u{e0154}\u{e0155}\u{e0156}\u{e0157}\u{e0158}\u{e0159}\u{e015a}\u{e015b}\u{e015c}\u{e015d}\u{e015e}\u{e015f}\u{e0160}\u{e0161}\u{e0162}\u{e0163}\u{e0164}\u{e0165}\u{e0166}\u{e0167}\u{e0168}\u{e0169}\u{e016a}\u{e016b}\u{e016c}\u{e016d}\u{e016e}\u{e016f}\u{e0170}\u{e0171}\u{e0172}\u{e0173}\u{e0174}\u{e0175}\u{e0176}\u{e0177}\u{e0178}\u{e0179}\u{e017a}\u{e017b}\u{e017c}\u{e017d}\u{e017e}\u{e017f}\u{e0180}\u{e0181}\u{e0182}\u{e0183}\u{e0184}\u{e0185}\u{e0186}\u{e0187}\u{e0188}\u{e0189}\u{e018a}\u{e018b}\u{e018c}\u{e018d}\u{e018e}\u{e018f}\u{e0190}\u{e0191}\u{e0192}\u{e0193}\u{e0194}\u{e0195}\u{e0196}\u{e0197}\u{e0198}\u{e0199}\u{e019a}\u{e019b}\u{e019c}\u{e019d}\u{e019e}\u{e019f}\u{e01a0}\u{e01a1}\u{e01a2}\u{e01a3}\u{e01a4}\u{e01a5}\u{e01a6}\u{e01a7}\u{e01a8}\u{e01a9}\u{e01aa}\u{e01ab}\u{e01ac}\u{e01ad}\u{e01ae}\u{e01af}\u{e01b0}\u{e01b1}\u{e01b2}\u{e01b3}\u{e01b4}\u{e01b5}\u{e01b6}\u{e01b7}\u{e01b8}\u{e01b9}\u{e01ba}\u{e01bb}\u{e01bc}\u{e01bd}\u{e01be}\u{e01bf}\u{e01c0}\u{e01c1}\u{e01c2}\u{e01c3}\u{e01c4}\u{e01c5}\u{e01c6}\u{e01c7}\u{e01c8}\u{e01c9}\u{e01ca}\u{e01cb}\u{e01cc}\u{e01cd}\u{e01ce}\u{e01cf}\u{e01d0}\u{e01d1}\u{e01d2}\u{e01d3}\u{e01d4}\u{e01d5}\u{e01d6}\u{e01d7}\u{e01d8}\u{e01d9}\u{e01da}\u{e01db}\u{e01dc}\u{e01dd}\u{e01de}\u{e01df}\u{e01e0}\u{e01e1}\u{e01e2}\u{e01e3}\u{e01e4}\u{e01e5}\u{e01e6}\u{e01e7}\u{e01e8}\u{e01e9}\u{e01ea}\u{e01eb}\u{e01ec}\u{e01ed}\u{e01ee}\u{e01ef}\u{180a}\u{180e}\u{1810}\u{fdff}\u{fe10}\u{e00ff}\u{e01f0}\u{feff}ſK\n",
                            "X\r\n\u{fe00}\u{2028}\u{fe0f}\u{2029}",
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
    fn unicode_variation_selector_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{VS}\P{Variation_Selector}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{VS}$", r"^[\P{Variation_Selector}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{fe00}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{VS}--A]",
            r"[\p{VS}&&A]",
            r"[[\p{VS}]]",
            r"[A-\p{VS}]",
            r"[\p{VS}-A]",
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
                &JsString::from(r"[\p{VS}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_dash_property_membership_over_every_code_point() {
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Dash}",
                    r"\P{Dash}",
                    r"[\p{Dash}]",
                    r"[^\p{Dash}]",
                    r"[\P{Dash}K]",
                    r"[\p{Dash}K]",
                    r"[\p{Dash}\P{Dash}]",
                    r"[^\P{Dash}K]",
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
                        let dash = [
                            0x2d, 0x58a, 0x5be, 0x1400, 0x1806, 0x2010, 0x2011, 0x2012, 0x2013,
                            0x2014, 0x2015, 0x2053, 0x207b, 0x208b, 0x2212, 0x2e17, 0x2e1a, 0x2e3a,
                            0x2e3b, 0x2e40, 0x2e5d, 0x301c, 0x3030, 0x30a0, 0xfe31, 0xfe32, 0xfe58,
                            0xfe63, 0xff0d, 0x10d6e, 0x10ead,
                        ]
                        .binary_search(&point)
                        .is_ok();
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Dash}" | r"[\p{Dash}]" => dash,
                            r"\P{Dash}" | r"[^\p{Dash}]" => !dash,
                            r"[\P{Dash}K]" => !dash || k,
                            r"[\p{Dash}K]" => dash || k,
                            r"[\p{Dash}\P{Dash}]" => true,
                            r"[^\P{Dash}K]" => dash && !k,
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
    fn unicode_dash_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Dash}",
            r"\P{Dash}",
            r"^[\p{Dash}]$",
            r"[\P{Dash}K]",
            r"[^\P{Dash}K]",
            r"[\p{Dash}]",
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
                            "😀\u{2d}\u{58a}\u{5be}\u{1400}\u{1806}\u{2010}\u{2011}\u{2012}\u{2013}\u{2014}\u{2015}\u{2053}\u{207b}\u{208b}\u{2212}\u{2e17}\u{2e1a}\u{2e3a}\u{2e3b}\u{2e40}\u{2e5d}\u{301c}\u{3030}\u{30a0}\u{fe31}\u{fe32}\u{fe58}\u{fe63}\u{ff0d}\u{10d6e}\u{10ead}\u{2c}\u{2e}\u{589}\u{58b}\u{5bd}\u{5bf}\u{13ff}\u{1401}\u{1805}\u{1807}\u{200f}\u{2016}\u{2052}\u{2054}\u{207a}\u{207c}\u{208a}\u{208c}\u{2211}\u{2213}\u{2e16}\u{2e18}\u{2e19}\u{2e1b}\u{2e39}\u{2e3c}\u{2e3f}\u{2e41}\u{2e5c}\u{2e5e}\u{301b}\u{301d}\u{302f}\u{3031}\u{309f}\u{30a1}\u{fe30}\u{fe33}\u{fe57}\u{fe59}\u{fe62}\u{fe64}\u{ff0c}\u{ff0e}\u{10d6d}\u{10d6f}\u{10eac}\u{10eae}\u{feff}ſK\n",
                            "X\r\n\u{2010}\u{2028}\u{2015}\u{2029}",
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
    fn unicode_dash_property_work_deduplication_and_contracts() {
        let source = JsString::from(format!("[{}]", r"\p{Dash}\P{Dash}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Dash}$", r"^[\P{Dash}K]$"] {
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
            assert_eq!(
                plan.find(&JsString::from("\u{2010}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Dash}--A]",
            r"[\p{Dash}&&A]",
            r"[[\p{Dash}]]",
            r"[A-\p{Dash}]",
            r"[\p{Dash}-A]",
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
                &JsString::from(r"[\p{Dash}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
}
