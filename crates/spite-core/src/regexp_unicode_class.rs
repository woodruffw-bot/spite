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
/// Han/Hani with exact Script/sc names also reuses these fixed intervals.
/// Pattern_Syntax/Pat_Syn uses its exact fixed property intervals.
/// Bidi_Mirrored/Bidi_M uses UCD 18 mirrored membership.
/// Extender/Ext uses UCD 18 extender membership.
/// Sentence_Terminal/STerm uses UCD 18 sentence-terminal membership.
/// Terminal_Punctuation/Term uses UCD 18 terminal-punctuation membership.
/// IDS_Binary_Operator/IDSB uses UCD 18 binary-operator membership.
/// IDS_Trinary_Operator/IDST uses UCD 18 trinary-operator membership.
/// Radical uses UCD 18 radical membership.
/// Regional_Indicator/RI uses UCD 18 regional-indicator membership.
/// Unified_Ideograph/UIdeo uses UCD 18 unified-ideograph membership.
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

    /// Compiles standalone supported Unicode properties and boundaries.
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
        // Each fixed property has bounded positive and complementary interval lists.
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
            26 => 22,
            28 => 29,
            30 => 121,
            32 => 44,
            34 => 90,
            36 => 118,
            38 => 4,
            40 => 2,
            42 => 4,
            44 => 2,
            46 => 17,
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
        let mut properties = 0u64;
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
        for kind in 0..48 {
            if properties & (1 << kind) != 0 {
                // Appending may move the original intervals if capacity grows.
                // Charge that move and the fixed set before allocating.
                charge(ranges.len().saturating_add(match kind {
                    12..=13 | 16..=17 => 40,
                    8..=11 | 14..=15 | 18..=19 | 22..=23 => 24,
                    20..=21 => 32,
                    24..=25 => 100,
                    26..=27 => 88,
                    28..=29 => 116,
                    30..=31 => 484,
                    32..=33 => 176,
                    34..=35 => 360,
                    36..=37 => 472,
                    38..=39 => 16,
                    40..=41 => 8,
                    42..=43 => 16,
                    44..=45 => 8,
                    46..=47 => 68,
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
        [0x53, 0x63, 0x72, 0x69, 0x70, 0x74, 0x3d, 0x48, 0x61, 0x6e]
        | [
            0x53,
            0x63,
            0x72,
            0x69,
            0x70,
            0x74,
            0x3d,
            0x48,
            0x61,
            0x6e,
            0x69,
        ]
        | [0x73, 0x63, 0x3d, 0x48, 0x61, 0x6e]
        | [0x73, 0x63, 0x3d, 0x48, 0x61, 0x6e, 0x69] => 26,
        [0x50, 0x61, 0x74, 0x5f, 0x53, 0x79, 0x6e]
        | [
            0x50,
            0x61,
            0x74,
            0x74,
            0x65,
            0x72,
            0x6e,
            0x5f,
            0x53,
            0x79,
            0x6e,
            0x74,
            0x61,
            0x78,
        ] => 28,
        [0x42, 0x69, 0x64, 0x69, 0x5f, 0x4d]
        | [
            0x42,
            0x69,
            0x64,
            0x69,
            0x5f,
            0x4d,
            0x69,
            0x72,
            0x72,
            0x6f,
            0x72,
            0x65,
            0x64,
        ] => 30,
        [0x45, 0x78, 0x74] | [0x45, 0x78, 0x74, 0x65, 0x6e, 0x64, 0x65, 0x72] => 32,
        [0x53, 0x54, 0x65, 0x72, 0x6d]
        | [
            0x53,
            0x65,
            0x6e,
            0x74,
            0x65,
            0x6e,
            0x63,
            0x65,
            0x5f,
            0x54,
            0x65,
            0x72,
            0x6d,
            0x69,
            0x6e,
            0x61,
            0x6c,
        ] => 34,
        [0x54, 0x65, 0x72, 0x6d]
        | [
            0x54,
            0x65,
            0x72,
            0x6d,
            0x69,
            0x6e,
            0x61,
            0x6c,
            0x5f,
            0x50,
            0x75,
            0x6e,
            0x63,
            0x74,
            0x75,
            0x61,
            0x74,
            0x69,
            0x6f,
            0x6e,
        ] => 36,
        [0x49, 0x44, 0x53, 0x42]
        | [
            0x49,
            0x44,
            0x53,
            0x5f,
            0x42,
            0x69,
            0x6e,
            0x61,
            0x72,
            0x79,
            0x5f,
            0x4f,
            0x70,
            0x65,
            0x72,
            0x61,
            0x74,
            0x6f,
            0x72,
        ] => 38,
        [0x49, 0x44, 0x53, 0x54]
        | [
            0x49,
            0x44,
            0x53,
            0x5f,
            0x54,
            0x72,
            0x69,
            0x6e,
            0x61,
            0x72,
            0x79,
            0x5f,
            0x4f,
            0x70,
            0x65,
            0x72,
            0x61,
            0x74,
            0x6f,
            0x72,
        ] => 40,
        [0x52, 0x61, 0x64, 0x69, 0x63, 0x61, 0x6c] => 42,
        [0x52, 0x49]
        | [
            0x52,
            0x65,
            0x67,
            0x69,
            0x6f,
            0x6e,
            0x61,
            0x6c,
            0x5f,
            0x49,
            0x6e,
            0x64,
            0x69,
            0x63,
            0x61,
            0x74,
            0x6f,
            0x72,
        ] => 44,
        [0x55, 0x49, 0x64, 0x65, 0x6f]
        | [
            0x55,
            0x6e,
            0x69,
            0x66,
            0x69,
            0x65,
            0x64,
            0x5f,
            0x49,
            0x64,
            0x65,
            0x6f,
            0x67,
            0x72,
            0x61,
            0x70,
            0x68,
        ] => 46,
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
        // Unicode 18 Scripts.txt: Han/Hani, invariant under C/S folding.
        // Source: https://www.unicode.org/Public/18.0.0/ucd/Scripts.txt
        // Copyright 2026 Unicode, Inc. See third-party/unicode/LICENSE.
        // SHA-256: 0071fd81b6aeae25f6e8bce8efec3066a6476a91b49bdb2f52dc76e817862a6a
        26 => &[
            (0x2e80, 0x2e99),
            (0x2e9b, 0x2ef3),
            (0x2f00, 0x2fd5),
            (0x3005, 0x3005),
            (0x3007, 0x3007),
            (0x3021, 0x3029),
            (0x3038, 0x303b),
            (0x3400, 0x4dbf),
            (0x4e00, 0x9fff),
            (0xf900, 0xfa6d),
            (0xfa70, 0xfad9),
            (0x16fe2, 0x16fe3),
            (0x16ff0, 0x16ff6),
            (0x20000, 0x2a6df),
            (0x2a700, 0x2b81e),
            (0x2b820, 0x2cead),
            (0x2ceb0, 0x2ebe0),
            (0x2ebf0, 0x2ee5d),
            (0x2f800, 0x2fa1d),
            (0x30000, 0x3134a),
            (0x31350, 0x33479),
        ],
        27 => &[
            (0x0, 0x2e7f),
            (0x2e9a, 0x2e9a),
            (0x2ef4, 0x2eff),
            (0x2fd6, 0x3004),
            (0x3006, 0x3006),
            (0x3008, 0x3020),
            (0x302a, 0x3037),
            (0x303c, 0x33ff),
            (0x4dc0, 0x4dff),
            (0xa000, 0xf8ff),
            (0xfa6e, 0xfa6f),
            (0xfada, 0x16fe1),
            (0x16fe4, 0x16fef),
            (0x16ff7, 0x1ffff),
            (0x2a6e0, 0x2a6ff),
            (0x2b81f, 0x2b81f),
            (0x2ceae, 0x2ceaf),
            (0x2ebe1, 0x2ebef),
            (0x2ee5e, 0x2f7ff),
            (0x2fa1e, 0x2ffff),
            (0x3134b, 0x3134f),
            (0x3347a, 0x10ffff),
        ],
        // UCD 18 Pattern_Syntax: 2760 members, invariant under C/S folding.
        28 => &[
            (0x21, 0x2f),
            (0x3a, 0x40),
            (0x5b, 0x5e),
            (0x60, 0x60),
            (0x7b, 0x7e),
            (0xa1, 0xa7),
            (0xa9, 0xa9),
            (0xab, 0xac),
            (0xae, 0xae),
            (0xb0, 0xb1),
            (0xb6, 0xb6),
            (0xbb, 0xbb),
            (0xbf, 0xbf),
            (0xd7, 0xd7),
            (0xf7, 0xf7),
            (0x2010, 0x2027),
            (0x2030, 0x203e),
            (0x2041, 0x2053),
            (0x2055, 0x205e),
            (0x2190, 0x245f),
            (0x2500, 0x2775),
            (0x2794, 0x2bff),
            (0x2e00, 0x2e7f),
            (0x3001, 0x3003),
            (0x3008, 0x3020),
            (0x3030, 0x3030),
            (0xfd3e, 0xfd3f),
            (0xfe45, 0xfe46),
        ],
        29 => &[
            (0x0, 0x20),
            (0x30, 0x39),
            (0x41, 0x5a),
            (0x5f, 0x5f),
            (0x61, 0x7a),
            (0x7f, 0xa0),
            (0xa8, 0xa8),
            (0xaa, 0xaa),
            (0xad, 0xad),
            (0xaf, 0xaf),
            (0xb2, 0xb5),
            (0xb7, 0xba),
            (0xbc, 0xbe),
            (0xc0, 0xd6),
            (0xd8, 0xf6),
            (0xf8, 0x200f),
            (0x2028, 0x202f),
            (0x203f, 0x2040),
            (0x2054, 0x2054),
            (0x205f, 0x218f),
            (0x2460, 0x24ff),
            (0x2776, 0x2793),
            (0x2c00, 0x2dff),
            (0x2e80, 0x3000),
            (0x3004, 0x3007),
            (0x3021, 0x302f),
            (0x3031, 0xfd3d),
            (0xfd40, 0xfe44),
            (0xfe47, 0x10ffff),
        ],
        // UCD 18 Bidi_Mirrored: 570 members, invariant under C/S folding.
        30 => &[
            (0x28, 0x29),
            (0x3c, 0x3c),
            (0x3e, 0x3e),
            (0x5b, 0x5b),
            (0x5d, 0x5d),
            (0x7b, 0x7b),
            (0x7d, 0x7d),
            (0xab, 0xab),
            (0xbb, 0xbb),
            (0xf3a, 0xf3d),
            (0x169b, 0x169c),
            (0x2039, 0x203a),
            (0x2045, 0x2046),
            (0x207d, 0x207e),
            (0x208d, 0x208e),
            (0x2140, 0x2140),
            (0x2201, 0x2204),
            (0x2208, 0x220d),
            (0x2211, 0x2211),
            (0x2215, 0x2216),
            (0x221a, 0x221d),
            (0x221f, 0x2222),
            (0x2224, 0x2224),
            (0x2226, 0x2226),
            (0x222b, 0x2233),
            (0x2239, 0x2239),
            (0x223b, 0x224c),
            (0x2252, 0x2255),
            (0x225f, 0x2260),
            (0x2262, 0x2262),
            (0x2264, 0x226b),
            (0x226d, 0x228c),
            (0x228f, 0x2292),
            (0x2298, 0x2298),
            (0x22a2, 0x22a3),
            (0x22a6, 0x22b8),
            (0x22be, 0x22bf),
            (0x22c9, 0x22cd),
            (0x22d0, 0x22d1),
            (0x22d6, 0x22ed),
            (0x22f0, 0x22ff),
            (0x2308, 0x230b),
            (0x2320, 0x2321),
            (0x2329, 0x232a),
            (0x2768, 0x2775),
            (0x27c0, 0x27c0),
            (0x27c3, 0x27c6),
            (0x27c8, 0x27c9),
            (0x27cb, 0x27cd),
            (0x27d3, 0x27d6),
            (0x27dc, 0x27de),
            (0x27e2, 0x27ef),
            (0x2983, 0x2998),
            (0x299b, 0x29a0),
            (0x29a2, 0x29af),
            (0x29b8, 0x29b8),
            (0x29c0, 0x29c5),
            (0x29c9, 0x29c9),
            (0x29ce, 0x29d2),
            (0x29d4, 0x29d5),
            (0x29d8, 0x29dc),
            (0x29e1, 0x29e1),
            (0x29e3, 0x29e5),
            (0x29e8, 0x29e9),
            (0x29f4, 0x29f9),
            (0x29fc, 0x29fd),
            (0x2a0a, 0x2a1c),
            (0x2a1e, 0x2a21),
            (0x2a24, 0x2a24),
            (0x2a26, 0x2a26),
            (0x2a29, 0x2a29),
            (0x2a2b, 0x2a2e),
            (0x2a34, 0x2a35),
            (0x2a3c, 0x2a3e),
            (0x2a57, 0x2a58),
            (0x2a64, 0x2a65),
            (0x2a6a, 0x2a6d),
            (0x2a6f, 0x2a70),
            (0x2a73, 0x2a74),
            (0x2a79, 0x2aa3),
            (0x2aa6, 0x2aad),
            (0x2aaf, 0x2ad6),
            (0x2adc, 0x2adc),
            (0x2ade, 0x2ade),
            (0x2ae2, 0x2ae6),
            (0x2aec, 0x2aee),
            (0x2af3, 0x2af3),
            (0x2af7, 0x2afb),
            (0x2afd, 0x2afd),
            (0x2bfe, 0x2bfe),
            (0x2e02, 0x2e05),
            (0x2e09, 0x2e0a),
            (0x2e0c, 0x2e0d),
            (0x2e1c, 0x2e1d),
            (0x2e20, 0x2e29),
            (0x2e55, 0x2e5c),
            (0x2e62, 0x2e63),
            (0x3008, 0x3011),
            (0x3014, 0x301b),
            (0xfe59, 0xfe5e),
            (0xfe64, 0xfe65),
            (0xff08, 0xff09),
            (0xff1c, 0xff1c),
            (0xff1e, 0xff1e),
            (0xff3b, 0xff3b),
            (0xff3d, 0xff3d),
            (0xff5b, 0xff5b),
            (0xff5d, 0xff5d),
            (0xff5f, 0xff60),
            (0xff62, 0xff63),
            (0x1cedd, 0x1cedf),
            (0x1d6db, 0x1d6db),
            (0x1d715, 0x1d715),
            (0x1d74f, 0x1d74f),
            (0x1d789, 0x1d789),
            (0x1d7c3, 0x1d7c3),
            (0x1db03, 0x1db09),
            (0x1db10, 0x1db10),
            (0x1db17, 0x1db18),
            (0x1db1b, 0x1db1b),
        ],
        31 => &[
            (0x0, 0x27),
            (0x2a, 0x3b),
            (0x3d, 0x3d),
            (0x3f, 0x5a),
            (0x5c, 0x5c),
            (0x5e, 0x7a),
            (0x7c, 0x7c),
            (0x7e, 0xaa),
            (0xac, 0xba),
            (0xbc, 0xf39),
            (0xf3e, 0x169a),
            (0x169d, 0x2038),
            (0x203b, 0x2044),
            (0x2047, 0x207c),
            (0x207f, 0x208c),
            (0x208f, 0x213f),
            (0x2141, 0x2200),
            (0x2205, 0x2207),
            (0x220e, 0x2210),
            (0x2212, 0x2214),
            (0x2217, 0x2219),
            (0x221e, 0x221e),
            (0x2223, 0x2223),
            (0x2225, 0x2225),
            (0x2227, 0x222a),
            (0x2234, 0x2238),
            (0x223a, 0x223a),
            (0x224d, 0x2251),
            (0x2256, 0x225e),
            (0x2261, 0x2261),
            (0x2263, 0x2263),
            (0x226c, 0x226c),
            (0x228d, 0x228e),
            (0x2293, 0x2297),
            (0x2299, 0x22a1),
            (0x22a4, 0x22a5),
            (0x22b9, 0x22bd),
            (0x22c0, 0x22c8),
            (0x22ce, 0x22cf),
            (0x22d2, 0x22d5),
            (0x22ee, 0x22ef),
            (0x2300, 0x2307),
            (0x230c, 0x231f),
            (0x2322, 0x2328),
            (0x232b, 0x2767),
            (0x2776, 0x27bf),
            (0x27c1, 0x27c2),
            (0x27c7, 0x27c7),
            (0x27ca, 0x27ca),
            (0x27ce, 0x27d2),
            (0x27d7, 0x27db),
            (0x27df, 0x27e1),
            (0x27f0, 0x2982),
            (0x2999, 0x299a),
            (0x29a1, 0x29a1),
            (0x29b0, 0x29b7),
            (0x29b9, 0x29bf),
            (0x29c6, 0x29c8),
            (0x29ca, 0x29cd),
            (0x29d3, 0x29d3),
            (0x29d6, 0x29d7),
            (0x29dd, 0x29e0),
            (0x29e2, 0x29e2),
            (0x29e6, 0x29e7),
            (0x29ea, 0x29f3),
            (0x29fa, 0x29fb),
            (0x29fe, 0x2a09),
            (0x2a1d, 0x2a1d),
            (0x2a22, 0x2a23),
            (0x2a25, 0x2a25),
            (0x2a27, 0x2a28),
            (0x2a2a, 0x2a2a),
            (0x2a2f, 0x2a33),
            (0x2a36, 0x2a3b),
            (0x2a3f, 0x2a56),
            (0x2a59, 0x2a63),
            (0x2a66, 0x2a69),
            (0x2a6e, 0x2a6e),
            (0x2a71, 0x2a72),
            (0x2a75, 0x2a78),
            (0x2aa4, 0x2aa5),
            (0x2aae, 0x2aae),
            (0x2ad7, 0x2adb),
            (0x2add, 0x2add),
            (0x2adf, 0x2ae1),
            (0x2ae7, 0x2aeb),
            (0x2aef, 0x2af2),
            (0x2af4, 0x2af6),
            (0x2afc, 0x2afc),
            (0x2afe, 0x2bfd),
            (0x2bff, 0x2e01),
            (0x2e06, 0x2e08),
            (0x2e0b, 0x2e0b),
            (0x2e0e, 0x2e1b),
            (0x2e1e, 0x2e1f),
            (0x2e2a, 0x2e54),
            (0x2e5d, 0x2e61),
            (0x2e64, 0x3007),
            (0x3012, 0x3013),
            (0x301c, 0xfe58),
            (0xfe5f, 0xfe63),
            (0xfe66, 0xff07),
            (0xff0a, 0xff1b),
            (0xff1d, 0xff1d),
            (0xff1f, 0xff3a),
            (0xff3c, 0xff3c),
            (0xff3e, 0xff5a),
            (0xff5c, 0xff5c),
            (0xff5e, 0xff5e),
            (0xff61, 0xff61),
            (0xff64, 0x1cedc),
            (0x1cee0, 0x1d6da),
            (0x1d6dc, 0x1d714),
            (0x1d716, 0x1d74e),
            (0x1d750, 0x1d788),
            (0x1d78a, 0x1d7c2),
            (0x1d7c4, 0x1db02),
            (0x1db0a, 0x1db0f),
            (0x1db11, 0x1db16),
            (0x1db19, 0x1db1a),
            (0x1db1c, 0x10ffff),
        ],
        // UCD 18 Extender: 63 members, invariant under C/S folding.
        32 => &[
            (0xb7, 0xb7),
            (0x2d0, 0x2d1),
            (0x640, 0x640),
            (0x7fa, 0x7fa),
            (0xa71, 0xa71),
            (0xafb, 0xafb),
            (0xb54, 0xb55),
            (0xe46, 0xe46),
            (0xec6, 0xec6),
            (0x180a, 0x180a),
            (0x1843, 0x1843),
            (0x1aa7, 0x1aa7),
            (0x1c36, 0x1c36),
            (0x1c7b, 0x1c7b),
            (0x3005, 0x3005),
            (0x3031, 0x3035),
            (0x309d, 0x309e),
            (0x30fc, 0x30fe),
            (0xa015, 0xa015),
            (0xa60c, 0xa60c),
            (0xa9cf, 0xa9cf),
            (0xa9e6, 0xa9e6),
            (0xaa70, 0xaa70),
            (0xaadd, 0xaadd),
            (0xaaf3, 0xaaf4),
            (0xff70, 0xff70),
            (0x10781, 0x10782),
            (0x10d4e, 0x10d4e),
            (0x10d6a, 0x10d6a),
            (0x10d6f, 0x10d6f),
            (0x11237, 0x11237),
            (0x1135d, 0x1135d),
            (0x113d2, 0x113d3),
            (0x115c6, 0x115c8),
            (0x11a98, 0x11a98),
            (0x11dd9, 0x11dd9),
            (0x16b42, 0x16b43),
            (0x16fe0, 0x16fe1),
            (0x16fe3, 0x16fe3),
            (0x16ff2, 0x16ff3),
            (0x1e13c, 0x1e13d),
            (0x1e5ef, 0x1e5ef),
            (0x1e944, 0x1e946),
        ],
        33 => &[
            (0x0, 0xb6),
            (0xb8, 0x2cf),
            (0x2d2, 0x63f),
            (0x641, 0x7f9),
            (0x7fb, 0xa70),
            (0xa72, 0xafa),
            (0xafc, 0xb53),
            (0xb56, 0xe45),
            (0xe47, 0xec5),
            (0xec7, 0x1809),
            (0x180b, 0x1842),
            (0x1844, 0x1aa6),
            (0x1aa8, 0x1c35),
            (0x1c37, 0x1c7a),
            (0x1c7c, 0x3004),
            (0x3006, 0x3030),
            (0x3036, 0x309c),
            (0x309f, 0x30fb),
            (0x30ff, 0xa014),
            (0xa016, 0xa60b),
            (0xa60d, 0xa9ce),
            (0xa9d0, 0xa9e5),
            (0xa9e7, 0xaa6f),
            (0xaa71, 0xaadc),
            (0xaade, 0xaaf2),
            (0xaaf5, 0xff6f),
            (0xff71, 0x10780),
            (0x10783, 0x10d4d),
            (0x10d4f, 0x10d69),
            (0x10d6b, 0x10d6e),
            (0x10d70, 0x11236),
            (0x11238, 0x1135c),
            (0x1135e, 0x113d1),
            (0x113d4, 0x115c5),
            (0x115c9, 0x11a97),
            (0x11a99, 0x11dd8),
            (0x11dda, 0x16b41),
            (0x16b44, 0x16fdf),
            (0x16fe2, 0x16fe2),
            (0x16fe4, 0x16ff1),
            (0x16ff4, 0x1e13b),
            (0x1e13e, 0x1e5ee),
            (0x1e5f0, 0x1e943),
            (0x1e947, 0x10ffff),
        ],
        // UCD 18 Sentence_Terminal: 172 members, invariant under C/S folding.
        34 => &[
            (0x21, 0x21),
            (0x2e, 0x2e),
            (0x3f, 0x3f),
            (0x589, 0x589),
            (0x61d, 0x61f),
            (0x6d4, 0x6d4),
            (0x700, 0x702),
            (0x7f9, 0x7f9),
            (0x837, 0x837),
            (0x839, 0x839),
            (0x83d, 0x83e),
            (0x964, 0x965),
            (0x104a, 0x104b),
            (0x1362, 0x1362),
            (0x1367, 0x1368),
            (0x166e, 0x166e),
            (0x1735, 0x1736),
            (0x17d4, 0x17d5),
            (0x1803, 0x1803),
            (0x1809, 0x1809),
            (0x1944, 0x1945),
            (0x1aa8, 0x1aab),
            (0x1b4e, 0x1b4f),
            (0x1b5a, 0x1b5b),
            (0x1b5e, 0x1b5f),
            (0x1b7d, 0x1b7f),
            (0x1c3b, 0x1c3c),
            (0x1c7e, 0x1c7f),
            (0x2024, 0x2024),
            (0x203c, 0x203d),
            (0x2047, 0x2049),
            (0x2cf9, 0x2cfb),
            (0x2e2e, 0x2e2e),
            (0x2e3c, 0x2e3c),
            (0x2e53, 0x2e54),
            (0x2e60, 0x2e61),
            (0x3002, 0x3002),
            (0xa4ff, 0xa4ff),
            (0xa60e, 0xa60f),
            (0xa6f3, 0xa6f3),
            (0xa6f7, 0xa6f7),
            (0xa876, 0xa877),
            (0xa8ce, 0xa8cf),
            (0xa92f, 0xa92f),
            (0xa9c8, 0xa9c9),
            (0xaa5d, 0xaa5f),
            (0xaaf0, 0xaaf1),
            (0xabeb, 0xabeb),
            (0xfe12, 0xfe12),
            (0xfe15, 0xfe16),
            (0xfe52, 0xfe52),
            (0xfe56, 0xfe57),
            (0xff01, 0xff01),
            (0xff0e, 0xff0e),
            (0xff1f, 0xff1f),
            (0xff61, 0xff61),
            (0x10a56, 0x10a57),
            (0x10f55, 0x10f59),
            (0x10f86, 0x10f89),
            (0x11047, 0x11048),
            (0x110be, 0x110c1),
            (0x11141, 0x11143),
            (0x111c5, 0x111c6),
            (0x111cd, 0x111cd),
            (0x111de, 0x111df),
            (0x11238, 0x11239),
            (0x1123b, 0x1123c),
            (0x112a9, 0x112a9),
            (0x113d4, 0x113d5),
            (0x1144b, 0x1144c),
            (0x115c2, 0x115c3),
            (0x115c9, 0x115d7),
            (0x11641, 0x11642),
            (0x1173c, 0x1173e),
            (0x11944, 0x11944),
            (0x11946, 0x11946),
            (0x11a42, 0x11a43),
            (0x11a9b, 0x11a9c),
            (0x11c41, 0x11c42),
            (0x11ef7, 0x11ef8),
            (0x11f43, 0x11f44),
            (0x16a6e, 0x16a6f),
            (0x16af5, 0x16af5),
            (0x16b37, 0x16b38),
            (0x16b44, 0x16b44),
            (0x16d6e, 0x16d6f),
            (0x16e98, 0x16e98),
            (0x1bc9f, 0x1bc9f),
            (0x1da88, 0x1da88),
        ],
        35 => &[
            (0x0, 0x20),
            (0x22, 0x2d),
            (0x2f, 0x3e),
            (0x40, 0x588),
            (0x58a, 0x61c),
            (0x620, 0x6d3),
            (0x6d5, 0x6ff),
            (0x703, 0x7f8),
            (0x7fa, 0x836),
            (0x838, 0x838),
            (0x83a, 0x83c),
            (0x83f, 0x963),
            (0x966, 0x1049),
            (0x104c, 0x1361),
            (0x1363, 0x1366),
            (0x1369, 0x166d),
            (0x166f, 0x1734),
            (0x1737, 0x17d3),
            (0x17d6, 0x1802),
            (0x1804, 0x1808),
            (0x180a, 0x1943),
            (0x1946, 0x1aa7),
            (0x1aac, 0x1b4d),
            (0x1b50, 0x1b59),
            (0x1b5c, 0x1b5d),
            (0x1b60, 0x1b7c),
            (0x1b80, 0x1c3a),
            (0x1c3d, 0x1c7d),
            (0x1c80, 0x2023),
            (0x2025, 0x203b),
            (0x203e, 0x2046),
            (0x204a, 0x2cf8),
            (0x2cfc, 0x2e2d),
            (0x2e2f, 0x2e3b),
            (0x2e3d, 0x2e52),
            (0x2e55, 0x2e5f),
            (0x2e62, 0x3001),
            (0x3003, 0xa4fe),
            (0xa500, 0xa60d),
            (0xa610, 0xa6f2),
            (0xa6f4, 0xa6f6),
            (0xa6f8, 0xa875),
            (0xa878, 0xa8cd),
            (0xa8d0, 0xa92e),
            (0xa930, 0xa9c7),
            (0xa9ca, 0xaa5c),
            (0xaa60, 0xaaef),
            (0xaaf2, 0xabea),
            (0xabec, 0xfe11),
            (0xfe13, 0xfe14),
            (0xfe17, 0xfe51),
            (0xfe53, 0xfe55),
            (0xfe58, 0xff00),
            (0xff02, 0xff0d),
            (0xff0f, 0xff1e),
            (0xff20, 0xff60),
            (0xff62, 0x10a55),
            (0x10a58, 0x10f54),
            (0x10f5a, 0x10f85),
            (0x10f8a, 0x11046),
            (0x11049, 0x110bd),
            (0x110c2, 0x11140),
            (0x11144, 0x111c4),
            (0x111c7, 0x111cc),
            (0x111ce, 0x111dd),
            (0x111e0, 0x11237),
            (0x1123a, 0x1123a),
            (0x1123d, 0x112a8),
            (0x112aa, 0x113d3),
            (0x113d6, 0x1144a),
            (0x1144d, 0x115c1),
            (0x115c4, 0x115c8),
            (0x115d8, 0x11640),
            (0x11643, 0x1173b),
            (0x1173f, 0x11943),
            (0x11945, 0x11945),
            (0x11947, 0x11a41),
            (0x11a44, 0x11a9a),
            (0x11a9d, 0x11c40),
            (0x11c43, 0x11ef6),
            (0x11ef9, 0x11f42),
            (0x11f45, 0x16a6d),
            (0x16a70, 0x16af4),
            (0x16af6, 0x16b36),
            (0x16b39, 0x16b43),
            (0x16b45, 0x16d6d),
            (0x16d70, 0x16e97),
            (0x16e99, 0x1bc9e),
            (0x1bca0, 0x1da87),
            (0x1da89, 0x10ffff),
        ],
        // UCD 18 Terminal_Punctuation: 293 members, invariant under C/S folding.
        36 => &[
            (0x21, 0x21),
            (0x2c, 0x2c),
            (0x2e, 0x2e),
            (0x3a, 0x3b),
            (0x3f, 0x3f),
            (0x37e, 0x37e),
            (0x387, 0x387),
            (0x589, 0x589),
            (0x5c3, 0x5c3),
            (0x60c, 0x60c),
            (0x61b, 0x61b),
            (0x61d, 0x61f),
            (0x6d4, 0x6d4),
            (0x700, 0x70a),
            (0x70c, 0x70c),
            (0x7f8, 0x7f9),
            (0x830, 0x835),
            (0x837, 0x83e),
            (0x85e, 0x85e),
            (0x964, 0x965),
            (0xe5a, 0xe5b),
            (0xf08, 0xf08),
            (0xf0d, 0xf12),
            (0x104a, 0x104b),
            (0x1361, 0x1368),
            (0x166e, 0x166e),
            (0x16eb, 0x16ed),
            (0x1735, 0x1736),
            (0x17d4, 0x17d6),
            (0x17da, 0x17da),
            (0x1802, 0x1805),
            (0x1808, 0x1809),
            (0x1944, 0x1945),
            (0x1aa8, 0x1aab),
            (0x1b4e, 0x1b4f),
            (0x1b5a, 0x1b5b),
            (0x1b5d, 0x1b5f),
            (0x1b7d, 0x1b7f),
            (0x1c3b, 0x1c3f),
            (0x1c7e, 0x1c7f),
            (0x2024, 0x2024),
            (0x203c, 0x203d),
            (0x2047, 0x2049),
            (0x2cf9, 0x2cfb),
            (0x2e2e, 0x2e2e),
            (0x2e3c, 0x2e3c),
            (0x2e41, 0x2e41),
            (0x2e4c, 0x2e4c),
            (0x2e4e, 0x2e4f),
            (0x2e53, 0x2e54),
            (0x2e60, 0x2e61),
            (0x3001, 0x3002),
            (0xa4fe, 0xa4ff),
            (0xa60d, 0xa60f),
            (0xa6f3, 0xa6f7),
            (0xa876, 0xa877),
            (0xa8ce, 0xa8cf),
            (0xa92f, 0xa92f),
            (0xa9c7, 0xa9c9),
            (0xaa5d, 0xaa5f),
            (0xaadf, 0xaadf),
            (0xaaf0, 0xaaf1),
            (0xabeb, 0xabeb),
            (0xfe12, 0xfe12),
            (0xfe15, 0xfe16),
            (0xfe50, 0xfe52),
            (0xfe54, 0xfe57),
            (0xff01, 0xff01),
            (0xff0c, 0xff0c),
            (0xff0e, 0xff0e),
            (0xff1a, 0xff1b),
            (0xff1f, 0xff1f),
            (0xff61, 0xff61),
            (0xff64, 0xff64),
            (0x1039f, 0x1039f),
            (0x103d0, 0x103d0),
            (0x10857, 0x10857),
            (0x1091f, 0x1091f),
            (0x10a56, 0x10a57),
            (0x10af0, 0x10af5),
            (0x10b3a, 0x10b3f),
            (0x10b99, 0x10b9c),
            (0x10f55, 0x10f59),
            (0x10f86, 0x10f89),
            (0x11047, 0x1104d),
            (0x110be, 0x110c1),
            (0x11141, 0x11143),
            (0x111c5, 0x111c6),
            (0x111cd, 0x111cd),
            (0x111de, 0x111df),
            (0x11238, 0x1123c),
            (0x112a9, 0x112a9),
            (0x113d4, 0x113d5),
            (0x1144b, 0x1144d),
            (0x1145a, 0x1145b),
            (0x115c2, 0x115c5),
            (0x115c9, 0x115d7),
            (0x11641, 0x11642),
            (0x1173c, 0x1173e),
            (0x11944, 0x11944),
            (0x11946, 0x11946),
            (0x11a42, 0x11a43),
            (0x11a9b, 0x11a9c),
            (0x11aa1, 0x11aa2),
            (0x11c41, 0x11c43),
            (0x11c71, 0x11c71),
            (0x11ef7, 0x11ef8),
            (0x11f43, 0x11f44),
            (0x12470, 0x12474),
            (0x16a6e, 0x16a6f),
            (0x16af5, 0x16af5),
            (0x16b37, 0x16b39),
            (0x16b44, 0x16b44),
            (0x16d6e, 0x16d6f),
            (0x16e97, 0x16e98),
            (0x1bc9f, 0x1bc9f),
            (0x1da87, 0x1da8a),
        ],
        37 => &[
            (0x0, 0x20),
            (0x22, 0x2b),
            (0x2d, 0x2d),
            (0x2f, 0x39),
            (0x3c, 0x3e),
            (0x40, 0x37d),
            (0x37f, 0x386),
            (0x388, 0x588),
            (0x58a, 0x5c2),
            (0x5c4, 0x60b),
            (0x60d, 0x61a),
            (0x61c, 0x61c),
            (0x620, 0x6d3),
            (0x6d5, 0x6ff),
            (0x70b, 0x70b),
            (0x70d, 0x7f7),
            (0x7fa, 0x82f),
            (0x836, 0x836),
            (0x83f, 0x85d),
            (0x85f, 0x963),
            (0x966, 0xe59),
            (0xe5c, 0xf07),
            (0xf09, 0xf0c),
            (0xf13, 0x1049),
            (0x104c, 0x1360),
            (0x1369, 0x166d),
            (0x166f, 0x16ea),
            (0x16ee, 0x1734),
            (0x1737, 0x17d3),
            (0x17d7, 0x17d9),
            (0x17db, 0x1801),
            (0x1806, 0x1807),
            (0x180a, 0x1943),
            (0x1946, 0x1aa7),
            (0x1aac, 0x1b4d),
            (0x1b50, 0x1b59),
            (0x1b5c, 0x1b5c),
            (0x1b60, 0x1b7c),
            (0x1b80, 0x1c3a),
            (0x1c40, 0x1c7d),
            (0x1c80, 0x2023),
            (0x2025, 0x203b),
            (0x203e, 0x2046),
            (0x204a, 0x2cf8),
            (0x2cfc, 0x2e2d),
            (0x2e2f, 0x2e3b),
            (0x2e3d, 0x2e40),
            (0x2e42, 0x2e4b),
            (0x2e4d, 0x2e4d),
            (0x2e50, 0x2e52),
            (0x2e55, 0x2e5f),
            (0x2e62, 0x3000),
            (0x3003, 0xa4fd),
            (0xa500, 0xa60c),
            (0xa610, 0xa6f2),
            (0xa6f8, 0xa875),
            (0xa878, 0xa8cd),
            (0xa8d0, 0xa92e),
            (0xa930, 0xa9c6),
            (0xa9ca, 0xaa5c),
            (0xaa60, 0xaade),
            (0xaae0, 0xaaef),
            (0xaaf2, 0xabea),
            (0xabec, 0xfe11),
            (0xfe13, 0xfe14),
            (0xfe17, 0xfe4f),
            (0xfe53, 0xfe53),
            (0xfe58, 0xff00),
            (0xff02, 0xff0b),
            (0xff0d, 0xff0d),
            (0xff0f, 0xff19),
            (0xff1c, 0xff1e),
            (0xff20, 0xff60),
            (0xff62, 0xff63),
            (0xff65, 0x1039e),
            (0x103a0, 0x103cf),
            (0x103d1, 0x10856),
            (0x10858, 0x1091e),
            (0x10920, 0x10a55),
            (0x10a58, 0x10aef),
            (0x10af6, 0x10b39),
            (0x10b40, 0x10b98),
            (0x10b9d, 0x10f54),
            (0x10f5a, 0x10f85),
            (0x10f8a, 0x11046),
            (0x1104e, 0x110bd),
            (0x110c2, 0x11140),
            (0x11144, 0x111c4),
            (0x111c7, 0x111cc),
            (0x111ce, 0x111dd),
            (0x111e0, 0x11237),
            (0x1123d, 0x112a8),
            (0x112aa, 0x113d3),
            (0x113d6, 0x1144a),
            (0x1144e, 0x11459),
            (0x1145c, 0x115c1),
            (0x115c6, 0x115c8),
            (0x115d8, 0x11640),
            (0x11643, 0x1173b),
            (0x1173f, 0x11943),
            (0x11945, 0x11945),
            (0x11947, 0x11a41),
            (0x11a44, 0x11a9a),
            (0x11a9d, 0x11aa0),
            (0x11aa3, 0x11c40),
            (0x11c44, 0x11c70),
            (0x11c72, 0x11ef6),
            (0x11ef9, 0x11f42),
            (0x11f45, 0x1246f),
            (0x12475, 0x16a6d),
            (0x16a70, 0x16af4),
            (0x16af6, 0x16b36),
            (0x16b3a, 0x16b43),
            (0x16b45, 0x16d6d),
            (0x16d70, 0x16e96),
            (0x16e99, 0x1bc9e),
            (0x1bca0, 0x1da86),
            (0x1da8b, 0x10ffff),
        ],
        // UCD 18 IDS_Binary_Operator: 13 members, invariant under C/S folding.
        38 => &[(0x2ff0, 0x2ff1), (0x2ff4, 0x2ffd), (0x31ef, 0x31ef)],
        39 => &[
            (0x0, 0x2fef),
            (0x2ff2, 0x2ff3),
            (0x2ffe, 0x31ee),
            (0x31f0, 0x10ffff),
        ],
        // UCD 18 IDS_Trinary_Operator: 2 members, invariant under C/S folding.
        40 => &[(0x2ff2, 0x2ff3)],
        41 => &[(0x0, 0x2ff1), (0x2ff4, 0x10ffff)],
        // UCD 18 Radical: 329 members, invariant under C/S folding.
        42 => &[(0x2e80, 0x2e99), (0x2e9b, 0x2ef3), (0x2f00, 0x2fd5)],
        43 => &[
            (0x0, 0x2e7f),
            (0x2e9a, 0x2e9a),
            (0x2ef4, 0x2eff),
            (0x2fd6, 0x10ffff),
        ],
        // UCD 18 Regional_Indicator: 26 members, invariant under C/S folding.
        44 => &[(0x1f1e6, 0x1f1ff)],
        45 => &[(0x0, 0x1f1e5), (0x1f200, 0x10ffff)],
        // UCD 18 Unified_Ideograph: 101997 members, invariant under C/S folding.
        46 => &[
            (0x3400, 0x4dbf),
            (0x4e00, 0x9fff),
            (0xfa0e, 0xfa0f),
            (0xfa11, 0xfa11),
            (0xfa13, 0xfa14),
            (0xfa1f, 0xfa1f),
            (0xfa21, 0xfa21),
            (0xfa23, 0xfa24),
            (0xfa27, 0xfa29),
            (0x20000, 0x2a6df),
            (0x2a700, 0x2b81e),
            (0x2b820, 0x2cead),
            (0x2ceb0, 0x2ebe0),
            (0x2ebf0, 0x2ee5d),
            (0x30000, 0x3134a),
            (0x31350, 0x33479),
        ],
        47 => &[
            (0x0, 0x33ff),
            (0x4dc0, 0x4dff),
            (0xa000, 0xfa0d),
            (0xfa10, 0xfa10),
            (0xfa12, 0xfa12),
            (0xfa15, 0xfa1e),
            (0xfa20, 0xfa20),
            (0xfa22, 0xfa22),
            (0xfa25, 0xfa26),
            (0xfa2a, 0x1ffff),
            (0x2a6e0, 0x2a6ff),
            (0x2b81f, 0x2b81f),
            (0x2ceae, 0x2ceaf),
            (0x2ebe1, 0x2ebef),
            (0x2ee5e, 0x2ffff),
            (0x3134b, 0x3134f),
            (0x3347a, 0x10ffff),
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
            r"\p{Script=Greek}",
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
            r"[\p{Script=Greek}]",
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
    #[test]
    fn unicode_han_script_membership_over_every_code_point() {
        // Separate UCD raw records, rather than the normalized production ranges.
        let records: &[(u32, u32)] = &[
            (0x2e80, 0x2e99),
            (0x2e9b, 0x2ef3),
            (0x2f00, 0x2fd5),
            (0x3005, 0x3005),
            (0x3007, 0x3007),
            (0x3021, 0x3029),
            (0x3038, 0x303a),
            (0x303b, 0x303b),
            (0x3400, 0x4dbf),
            (0x4e00, 0x9fff),
            (0xf900, 0xfa6d),
            (0xfa70, 0xfad9),
            (0x16fe2, 0x16fe2),
            (0x16fe3, 0x16fe3),
            (0x16ff0, 0x16ff1),
            (0x16ff2, 0x16ff3),
            (0x16ff4, 0x16ff6),
            (0x20000, 0x2a6df),
            (0x2a700, 0x2b81e),
            (0x2b820, 0x2cead),
            (0x2ceb0, 0x2ebe0),
            (0x2ebf0, 0x2ee5d),
            (0x2f800, 0x2fa1d),
            (0x30000, 0x3134a),
            (0x31350, 0x33479),
        ];
        for name in ["Script=Han", "Script=Hani", "sc=Han", "sc=Hani"] {
            for sets in [false, true] {
                for ignore_case in [false, true] {
                    for form in 0..8 {
                        let source = match form {
                            0 => format!(r"\p{{{name}}}"),
                            1 => format!(r"\P{{{name}}}"),
                            2 => format!(r"[\p{{{name}}}]"),
                            3 => format!(r"[^\p{{{name}}}]"),
                            4 => format!(r"[\P{{{name}}}K]"),
                            5 => format!(r"[\p{{{name}}}K]"),
                            6 => format!(r"[\p{{{name}}}\P{{{name}}}]"),
                            _ => format!(r"[^\P{{{name}}}K]"),
                        };
                        let pattern = JsString::from(source.as_str());
                        let plan = if form < 2 {
                            RegExpUnicodeClassMatcher::compile_binary_property_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                false,
                            )
                        } else {
                            RegExpUnicodeClassMatcher::compile_with_flags(
                                &pattern,
                                sets,
                                ignore_case,
                                false,
                            )
                        }
                        .unwrap();
                        for point in 0..=0x10ffff {
                            let han = records.iter().any(|(a, b)| *a <= point && point <= *b);
                            let k =
                                point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                            let expected = match form {
                                0 | 2 => han,
                                1 | 3 => !han,
                                4 => !han || k,
                                5 => han || k,
                                6 => true,
                                _ => han && !k,
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
    }

    #[test]
    fn unicode_han_script_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Script=Han}",
            r"\P{sc=Hani}",
            r"^[\p{sc=Han}]$",
            r"[\P{Script=Hani}K]",
            r"[^\P{sc=Han}K]",
            r"[\p{Script=Hani}]",
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
                            "𠮷a一々〆〇⺀⻳⿕\u{2b81e}😀ſK\n",
                            "X\r\n𠮷\u{2028}一\u{2029}",
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
    fn unicode_han_script_charges_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{Script=Han}\P{sc=Hani}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("𠮷"), 1, true),
            Some(0..2)
        );
        for source in [r"^\p{sc=Han}$", r"^[\p{Script=Hani}K]$"] {
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
            for fail in 1..=calls {
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
            assert_eq!(plan.find(&JsString::from("😀\n𠮷\n"), 0, false), Some(3..5));
        }
        for source in [
            r"[\p{sc=Han}--A]",
            r"[\p{Script=Han}&&A]",
            r"[[\p{sc=Han}]]",
            r"[A-\p{sc=Han}]",
            r"[\p{sc=Han}-A]",
            r"[\p{Script=Latin}]",
            r"[\p{Script_Extensions=Han}]",
            r"[\p{sc=han}]",
        ] {
            assert!(
                RegExpUnicodeClassMatcher::compile_with_flags(
                    &JsString::from(source),
                    true,
                    true,
                    false
                )
                .is_none(),
                "{source}"
            );
        }
    }
    #[test]
    fn unicode_pattern_syntax_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (33usize, 35usize),
            (36usize, 36usize),
            (37usize, 39usize),
            (40usize, 40usize),
            (41usize, 41usize),
            (42usize, 42usize),
            (43usize, 43usize),
            (44usize, 44usize),
            (45usize, 45usize),
            (46usize, 47usize),
            (58usize, 59usize),
            (60usize, 62usize),
            (63usize, 64usize),
            (91usize, 91usize),
            (92usize, 92usize),
            (93usize, 93usize),
            (94usize, 94usize),
            (96usize, 96usize),
            (123usize, 123usize),
            (124usize, 124usize),
            (125usize, 125usize),
            (126usize, 126usize),
            (161usize, 161usize),
            (162usize, 165usize),
            (166usize, 166usize),
            (167usize, 167usize),
            (169usize, 169usize),
            (171usize, 171usize),
            (172usize, 172usize),
            (174usize, 174usize),
            (176usize, 176usize),
            (177usize, 177usize),
            (182usize, 182usize),
            (187usize, 187usize),
            (191usize, 191usize),
            (215usize, 215usize),
            (247usize, 247usize),
            (8208usize, 8213usize),
            (8214usize, 8215usize),
            (8216usize, 8216usize),
            (8217usize, 8217usize),
            (8218usize, 8218usize),
            (8219usize, 8220usize),
            (8221usize, 8221usize),
            (8222usize, 8222usize),
            (8223usize, 8223usize),
            (8224usize, 8231usize),
            (8240usize, 8248usize),
            (8249usize, 8249usize),
            (8250usize, 8250usize),
            (8251usize, 8254usize),
            (8257usize, 8259usize),
            (8260usize, 8260usize),
            (8261usize, 8261usize),
            (8262usize, 8262usize),
            (8263usize, 8273usize),
            (8274usize, 8274usize),
            (8275usize, 8275usize),
            (8277usize, 8286usize),
            (8592usize, 8596usize),
            (8597usize, 8601usize),
            (8602usize, 8603usize),
            (8604usize, 8607usize),
            (8608usize, 8608usize),
            (8609usize, 8610usize),
            (8611usize, 8611usize),
            (8612usize, 8613usize),
            (8614usize, 8614usize),
            (8615usize, 8621usize),
            (8622usize, 8622usize),
            (8623usize, 8653usize),
            (8654usize, 8655usize),
            (8656usize, 8657usize),
            (8658usize, 8658usize),
            (8659usize, 8659usize),
            (8660usize, 8660usize),
            (8661usize, 8691usize),
            (8692usize, 8959usize),
            (8960usize, 8967usize),
            (8968usize, 8968usize),
            (8969usize, 8969usize),
            (8970usize, 8970usize),
            (8971usize, 8971usize),
            (8972usize, 8991usize),
            (8992usize, 8993usize),
            (8994usize, 9000usize),
            (9001usize, 9001usize),
            (9002usize, 9002usize),
            (9003usize, 9083usize),
            (9084usize, 9084usize),
            (9085usize, 9114usize),
            (9115usize, 9139usize),
            (9140usize, 9179usize),
            (9180usize, 9185usize),
            (9186usize, 9257usize),
            (9258usize, 9279usize),
            (9280usize, 9290usize),
            (9291usize, 9311usize),
            (9472usize, 9654usize),
            (9655usize, 9655usize),
            (9656usize, 9664usize),
            (9665usize, 9665usize),
            (9666usize, 9719usize),
            (9720usize, 9727usize),
            (9728usize, 9838usize),
            (9839usize, 9839usize),
            (9840usize, 10087usize),
            (10088usize, 10088usize),
            (10089usize, 10089usize),
            (10090usize, 10090usize),
            (10091usize, 10091usize),
            (10092usize, 10092usize),
            (10093usize, 10093usize),
            (10094usize, 10094usize),
            (10095usize, 10095usize),
            (10096usize, 10096usize),
            (10097usize, 10097usize),
            (10098usize, 10098usize),
            (10099usize, 10099usize),
            (10100usize, 10100usize),
            (10101usize, 10101usize),
            (10132usize, 10175usize),
            (10176usize, 10180usize),
            (10181usize, 10181usize),
            (10182usize, 10182usize),
            (10183usize, 10213usize),
            (10214usize, 10214usize),
            (10215usize, 10215usize),
            (10216usize, 10216usize),
            (10217usize, 10217usize),
            (10218usize, 10218usize),
            (10219usize, 10219usize),
            (10220usize, 10220usize),
            (10221usize, 10221usize),
            (10222usize, 10222usize),
            (10223usize, 10223usize),
            (10224usize, 10239usize),
            (10240usize, 10495usize),
            (10496usize, 10626usize),
            (10627usize, 10627usize),
            (10628usize, 10628usize),
            (10629usize, 10629usize),
            (10630usize, 10630usize),
            (10631usize, 10631usize),
            (10632usize, 10632usize),
            (10633usize, 10633usize),
            (10634usize, 10634usize),
            (10635usize, 10635usize),
            (10636usize, 10636usize),
            (10637usize, 10637usize),
            (10638usize, 10638usize),
            (10639usize, 10639usize),
            (10640usize, 10640usize),
            (10641usize, 10641usize),
            (10642usize, 10642usize),
            (10643usize, 10643usize),
            (10644usize, 10644usize),
            (10645usize, 10645usize),
            (10646usize, 10646usize),
            (10647usize, 10647usize),
            (10648usize, 10648usize),
            (10649usize, 10711usize),
            (10712usize, 10712usize),
            (10713usize, 10713usize),
            (10714usize, 10714usize),
            (10715usize, 10715usize),
            (10716usize, 10747usize),
            (10748usize, 10748usize),
            (10749usize, 10749usize),
            (10750usize, 11007usize),
            (11008usize, 11055usize),
            (11056usize, 11076usize),
            (11077usize, 11078usize),
            (11079usize, 11084usize),
            (11085usize, 11123usize),
            (11124usize, 11125usize),
            (11126usize, 11263usize),
            (11776usize, 11777usize),
            (11778usize, 11778usize),
            (11779usize, 11779usize),
            (11780usize, 11780usize),
            (11781usize, 11781usize),
            (11782usize, 11784usize),
            (11785usize, 11785usize),
            (11786usize, 11786usize),
            (11787usize, 11787usize),
            (11788usize, 11788usize),
            (11789usize, 11789usize),
            (11790usize, 11798usize),
            (11799usize, 11799usize),
            (11800usize, 11801usize),
            (11802usize, 11802usize),
            (11803usize, 11803usize),
            (11804usize, 11804usize),
            (11805usize, 11805usize),
            (11806usize, 11807usize),
            (11808usize, 11808usize),
            (11809usize, 11809usize),
            (11810usize, 11810usize),
            (11811usize, 11811usize),
            (11812usize, 11812usize),
            (11813usize, 11813usize),
            (11814usize, 11814usize),
            (11815usize, 11815usize),
            (11816usize, 11816usize),
            (11817usize, 11817usize),
            (11818usize, 11822usize),
            (11823usize, 11823usize),
            (11824usize, 11833usize),
            (11834usize, 11835usize),
            (11836usize, 11839usize),
            (11840usize, 11840usize),
            (11841usize, 11841usize),
            (11842usize, 11842usize),
            (11843usize, 11855usize),
            (11856usize, 11857usize),
            (11858usize, 11860usize),
            (11861usize, 11861usize),
            (11862usize, 11862usize),
            (11863usize, 11863usize),
            (11864usize, 11864usize),
            (11865usize, 11865usize),
            (11866usize, 11866usize),
            (11867usize, 11867usize),
            (11868usize, 11868usize),
            (11869usize, 11869usize),
            (11870usize, 11871usize),
            (11872usize, 11873usize),
            (11874usize, 11874usize),
            (11875usize, 11875usize),
            (11876usize, 11903usize),
            (12289usize, 12291usize),
            (12296usize, 12296usize),
            (12297usize, 12297usize),
            (12298usize, 12298usize),
            (12299usize, 12299usize),
            (12300usize, 12300usize),
            (12301usize, 12301usize),
            (12302usize, 12302usize),
            (12303usize, 12303usize),
            (12304usize, 12304usize),
            (12305usize, 12305usize),
            (12306usize, 12307usize),
            (12308usize, 12308usize),
            (12309usize, 12309usize),
            (12310usize, 12310usize),
            (12311usize, 12311usize),
            (12312usize, 12312usize),
            (12313usize, 12313usize),
            (12314usize, 12314usize),
            (12315usize, 12315usize),
            (12316usize, 12316usize),
            (12317usize, 12317usize),
            (12318usize, 12319usize),
            (12320usize, 12320usize),
            (12336usize, 12336usize),
            (64830usize, 64830usize),
            (64831usize, 64831usize),
            (65093usize, 65094usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Pattern_Syntax}",
                    r"\P{Pat_Syn}",
                    r"[\p{Pat_Syn}]",
                    r"[^\p{Pattern_Syntax}]",
                    r"[\P{Pat_Syn}K]",
                    r"[\p{Pat_Syn}K]",
                    r"[\p{Pat_Syn}\P{Pattern_Syntax}]",
                    r"[^\P{Pat_Syn}K]",
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
                        let pattern_syntax = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Pattern_Syntax}" | r"[\p{Pat_Syn}]" => pattern_syntax,
                            r"\P{Pat_Syn}" | r"[^\p{Pattern_Syntax}]" => !pattern_syntax,
                            r"[\P{Pat_Syn}K]" => !pattern_syntax || k,
                            r"[\p{Pat_Syn}K]" => pattern_syntax || k,
                            r"[\p{Pat_Syn}\P{Pattern_Syntax}]" => true,
                            r"[^\P{Pat_Syn}K]" => pattern_syntax && !k,
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
    fn unicode_pattern_syntax_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Pat_Syn}",
            r"\P{Pattern_Syntax}",
            r"^[\p{Pattern_Syntax}]$",
            r"[\P{Pat_Syn}K]",
            r"[^\P{Pat_Syn}K]",
            r"[\p{Pattern_Syntax}]",
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
                            "😀\u{21}\u{2f}\u{3a}\u{40}\u{5b}\u{5e}\u{60}\u{7b}\u{7e}\u{a1}\u{a7}\u{a9}\u{ab}\u{ac}\u{ae}\u{b0}\u{b1}\u{b6}\u{bb}\u{bf}\u{d7}\u{f7}\u{2010}\u{2027}\u{2030}\u{203e}\u{2041}\u{2053}\u{2055}\u{205e}\u{2190}\u{245f}\u{2500}\u{2775}\u{2794}\u{2bff}\u{2e00}\u{2e7f}\u{3001}\u{3003}\u{3008}\u{3020}\u{3030}\u{fd3e}\u{fd3f}\u{fe45}\u{fe46}\u{20}\u{30}\u{39}\u{41}\u{5a}\u{5f}\u{61}\u{7a}\u{7f}\u{a0}\u{a8}\u{aa}\u{ad}\u{af}\u{b2}\u{b5}\u{b7}\u{ba}\u{bc}\u{be}\u{c0}\u{d6}\u{d8}\u{f6}\u{f8}\u{200f}\u{2028}\u{202f}\u{203f}\u{2040}\u{2054}\u{205f}\u{218f}\u{2460}\u{24ff}\u{2776}\u{2793}\u{2c00}\u{2dff}\u{2e80}\u{3000}\u{3004}\u{3007}\u{3021}\u{302f}\u{3031}\u{fd3d}\u{fd40}\u{fe44}\u{fe47}\u{feff}ſK\n",
                            "X\r\n\u{21}\u{2028}\u{2337}\u{2029}",
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
    fn unicode_pattern_syntax_property_work_deduplication_and_contracts() {
        let source = JsString::from(
            format!("[{}]", r"\p{Pat_Syn}\P{Pattern_Syntax}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Pat_Syn}$", r"^[\P{Pattern_Syntax}K]$"] {
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
                plan.find(&JsString::from("\u{21}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Pat_Syn}--A]",
            r"[\p{Pat_Syn}&&A]",
            r"[[\p{Pat_Syn}]]",
            r"[A-\p{Pat_Syn}]",
            r"[\p{Pat_Syn}-A]",
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
                &JsString::from(r"[\p{Pat_Syn}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_bidi_mirrored_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (40usize, 40usize),
            (41usize, 41usize),
            (60usize, 60usize),
            (62usize, 62usize),
            (91usize, 91usize),
            (93usize, 93usize),
            (123usize, 123usize),
            (125usize, 125usize),
            (171usize, 171usize),
            (187usize, 187usize),
            (3898usize, 3898usize),
            (3899usize, 3899usize),
            (3900usize, 3900usize),
            (3901usize, 3901usize),
            (5787usize, 5787usize),
            (5788usize, 5788usize),
            (8249usize, 8249usize),
            (8250usize, 8250usize),
            (8261usize, 8261usize),
            (8262usize, 8262usize),
            (8317usize, 8317usize),
            (8318usize, 8318usize),
            (8333usize, 8333usize),
            (8334usize, 8334usize),
            (8512usize, 8512usize),
            (8705usize, 8705usize),
            (8706usize, 8706usize),
            (8707usize, 8707usize),
            (8708usize, 8708usize),
            (8712usize, 8712usize),
            (8713usize, 8713usize),
            (8714usize, 8714usize),
            (8715usize, 8715usize),
            (8716usize, 8716usize),
            (8717usize, 8717usize),
            (8721usize, 8721usize),
            (8725usize, 8725usize),
            (8726usize, 8726usize),
            (8730usize, 8730usize),
            (8731usize, 8731usize),
            (8732usize, 8732usize),
            (8733usize, 8733usize),
            (8735usize, 8735usize),
            (8736usize, 8736usize),
            (8737usize, 8737usize),
            (8738usize, 8738usize),
            (8740usize, 8740usize),
            (8742usize, 8742usize),
            (8747usize, 8747usize),
            (8748usize, 8748usize),
            (8749usize, 8749usize),
            (8750usize, 8750usize),
            (8751usize, 8751usize),
            (8752usize, 8752usize),
            (8753usize, 8753usize),
            (8754usize, 8754usize),
            (8755usize, 8755usize),
            (8761usize, 8761usize),
            (8763usize, 8763usize),
            (8764usize, 8764usize),
            (8765usize, 8765usize),
            (8766usize, 8766usize),
            (8767usize, 8767usize),
            (8768usize, 8768usize),
            (8769usize, 8769usize),
            (8770usize, 8770usize),
            (8771usize, 8771usize),
            (8772usize, 8772usize),
            (8773usize, 8773usize),
            (8774usize, 8774usize),
            (8775usize, 8775usize),
            (8776usize, 8776usize),
            (8777usize, 8777usize),
            (8778usize, 8778usize),
            (8779usize, 8779usize),
            (8780usize, 8780usize),
            (8786usize, 8786usize),
            (8787usize, 8787usize),
            (8788usize, 8788usize),
            (8789usize, 8789usize),
            (8799usize, 8799usize),
            (8800usize, 8800usize),
            (8802usize, 8802usize),
            (8804usize, 8804usize),
            (8805usize, 8805usize),
            (8806usize, 8806usize),
            (8807usize, 8807usize),
            (8808usize, 8808usize),
            (8809usize, 8809usize),
            (8810usize, 8810usize),
            (8811usize, 8811usize),
            (8813usize, 8813usize),
            (8814usize, 8814usize),
            (8815usize, 8815usize),
            (8816usize, 8816usize),
            (8817usize, 8817usize),
            (8818usize, 8818usize),
            (8819usize, 8819usize),
            (8820usize, 8820usize),
            (8821usize, 8821usize),
            (8822usize, 8822usize),
            (8823usize, 8823usize),
            (8824usize, 8824usize),
            (8825usize, 8825usize),
            (8826usize, 8826usize),
            (8827usize, 8827usize),
            (8828usize, 8828usize),
            (8829usize, 8829usize),
            (8830usize, 8830usize),
            (8831usize, 8831usize),
            (8832usize, 8832usize),
            (8833usize, 8833usize),
            (8834usize, 8834usize),
            (8835usize, 8835usize),
            (8836usize, 8836usize),
            (8837usize, 8837usize),
            (8838usize, 8838usize),
            (8839usize, 8839usize),
            (8840usize, 8840usize),
            (8841usize, 8841usize),
            (8842usize, 8842usize),
            (8843usize, 8843usize),
            (8844usize, 8844usize),
            (8847usize, 8847usize),
            (8848usize, 8848usize),
            (8849usize, 8849usize),
            (8850usize, 8850usize),
            (8856usize, 8856usize),
            (8866usize, 8866usize),
            (8867usize, 8867usize),
            (8870usize, 8870usize),
            (8871usize, 8871usize),
            (8872usize, 8872usize),
            (8873usize, 8873usize),
            (8874usize, 8874usize),
            (8875usize, 8875usize),
            (8876usize, 8876usize),
            (8877usize, 8877usize),
            (8878usize, 8878usize),
            (8879usize, 8879usize),
            (8880usize, 8880usize),
            (8881usize, 8881usize),
            (8882usize, 8882usize),
            (8883usize, 8883usize),
            (8884usize, 8884usize),
            (8885usize, 8885usize),
            (8886usize, 8886usize),
            (8887usize, 8887usize),
            (8888usize, 8888usize),
            (8894usize, 8894usize),
            (8895usize, 8895usize),
            (8905usize, 8905usize),
            (8906usize, 8906usize),
            (8907usize, 8907usize),
            (8908usize, 8908usize),
            (8909usize, 8909usize),
            (8912usize, 8912usize),
            (8913usize, 8913usize),
            (8918usize, 8918usize),
            (8919usize, 8919usize),
            (8920usize, 8920usize),
            (8921usize, 8921usize),
            (8922usize, 8922usize),
            (8923usize, 8923usize),
            (8924usize, 8924usize),
            (8925usize, 8925usize),
            (8926usize, 8926usize),
            (8927usize, 8927usize),
            (8928usize, 8928usize),
            (8929usize, 8929usize),
            (8930usize, 8930usize),
            (8931usize, 8931usize),
            (8932usize, 8932usize),
            (8933usize, 8933usize),
            (8934usize, 8934usize),
            (8935usize, 8935usize),
            (8936usize, 8936usize),
            (8937usize, 8937usize),
            (8938usize, 8938usize),
            (8939usize, 8939usize),
            (8940usize, 8940usize),
            (8941usize, 8941usize),
            (8944usize, 8944usize),
            (8945usize, 8945usize),
            (8946usize, 8946usize),
            (8947usize, 8947usize),
            (8948usize, 8948usize),
            (8949usize, 8949usize),
            (8950usize, 8950usize),
            (8951usize, 8951usize),
            (8952usize, 8952usize),
            (8953usize, 8953usize),
            (8954usize, 8954usize),
            (8955usize, 8955usize),
            (8956usize, 8956usize),
            (8957usize, 8957usize),
            (8958usize, 8958usize),
            (8959usize, 8959usize),
            (8968usize, 8968usize),
            (8969usize, 8969usize),
            (8970usize, 8970usize),
            (8971usize, 8971usize),
            (8992usize, 8992usize),
            (8993usize, 8993usize),
            (9001usize, 9001usize),
            (9002usize, 9002usize),
            (10088usize, 10088usize),
            (10089usize, 10089usize),
            (10090usize, 10090usize),
            (10091usize, 10091usize),
            (10092usize, 10092usize),
            (10093usize, 10093usize),
            (10094usize, 10094usize),
            (10095usize, 10095usize),
            (10096usize, 10096usize),
            (10097usize, 10097usize),
            (10098usize, 10098usize),
            (10099usize, 10099usize),
            (10100usize, 10100usize),
            (10101usize, 10101usize),
            (10176usize, 10176usize),
            (10179usize, 10179usize),
            (10180usize, 10180usize),
            (10181usize, 10181usize),
            (10182usize, 10182usize),
            (10184usize, 10184usize),
            (10185usize, 10185usize),
            (10187usize, 10187usize),
            (10188usize, 10188usize),
            (10189usize, 10189usize),
            (10195usize, 10195usize),
            (10196usize, 10196usize),
            (10197usize, 10197usize),
            (10198usize, 10198usize),
            (10204usize, 10204usize),
            (10205usize, 10205usize),
            (10206usize, 10206usize),
            (10210usize, 10210usize),
            (10211usize, 10211usize),
            (10212usize, 10212usize),
            (10213usize, 10213usize),
            (10214usize, 10214usize),
            (10215usize, 10215usize),
            (10216usize, 10216usize),
            (10217usize, 10217usize),
            (10218usize, 10218usize),
            (10219usize, 10219usize),
            (10220usize, 10220usize),
            (10221usize, 10221usize),
            (10222usize, 10222usize),
            (10223usize, 10223usize),
            (10627usize, 10627usize),
            (10628usize, 10628usize),
            (10629usize, 10629usize),
            (10630usize, 10630usize),
            (10631usize, 10631usize),
            (10632usize, 10632usize),
            (10633usize, 10633usize),
            (10634usize, 10634usize),
            (10635usize, 10635usize),
            (10636usize, 10636usize),
            (10637usize, 10637usize),
            (10638usize, 10638usize),
            (10639usize, 10639usize),
            (10640usize, 10640usize),
            (10641usize, 10641usize),
            (10642usize, 10642usize),
            (10643usize, 10643usize),
            (10644usize, 10644usize),
            (10645usize, 10645usize),
            (10646usize, 10646usize),
            (10647usize, 10647usize),
            (10648usize, 10648usize),
            (10651usize, 10651usize),
            (10652usize, 10652usize),
            (10653usize, 10653usize),
            (10654usize, 10654usize),
            (10655usize, 10655usize),
            (10656usize, 10656usize),
            (10658usize, 10658usize),
            (10659usize, 10659usize),
            (10660usize, 10660usize),
            (10661usize, 10661usize),
            (10662usize, 10662usize),
            (10663usize, 10663usize),
            (10664usize, 10664usize),
            (10665usize, 10665usize),
            (10666usize, 10666usize),
            (10667usize, 10667usize),
            (10668usize, 10668usize),
            (10669usize, 10669usize),
            (10670usize, 10670usize),
            (10671usize, 10671usize),
            (10680usize, 10680usize),
            (10688usize, 10688usize),
            (10689usize, 10689usize),
            (10690usize, 10690usize),
            (10691usize, 10691usize),
            (10692usize, 10692usize),
            (10693usize, 10693usize),
            (10697usize, 10697usize),
            (10702usize, 10702usize),
            (10703usize, 10703usize),
            (10704usize, 10704usize),
            (10705usize, 10705usize),
            (10706usize, 10706usize),
            (10708usize, 10708usize),
            (10709usize, 10709usize),
            (10712usize, 10712usize),
            (10713usize, 10713usize),
            (10714usize, 10714usize),
            (10715usize, 10715usize),
            (10716usize, 10716usize),
            (10721usize, 10721usize),
            (10723usize, 10723usize),
            (10724usize, 10724usize),
            (10725usize, 10725usize),
            (10728usize, 10728usize),
            (10729usize, 10729usize),
            (10740usize, 10740usize),
            (10741usize, 10741usize),
            (10742usize, 10742usize),
            (10743usize, 10743usize),
            (10744usize, 10744usize),
            (10745usize, 10745usize),
            (10748usize, 10748usize),
            (10749usize, 10749usize),
            (10762usize, 10762usize),
            (10763usize, 10763usize),
            (10764usize, 10764usize),
            (10765usize, 10765usize),
            (10766usize, 10766usize),
            (10767usize, 10767usize),
            (10768usize, 10768usize),
            (10769usize, 10769usize),
            (10770usize, 10770usize),
            (10771usize, 10771usize),
            (10772usize, 10772usize),
            (10773usize, 10773usize),
            (10774usize, 10774usize),
            (10775usize, 10775usize),
            (10776usize, 10776usize),
            (10777usize, 10777usize),
            (10778usize, 10778usize),
            (10779usize, 10779usize),
            (10780usize, 10780usize),
            (10782usize, 10782usize),
            (10783usize, 10783usize),
            (10784usize, 10784usize),
            (10785usize, 10785usize),
            (10788usize, 10788usize),
            (10790usize, 10790usize),
            (10793usize, 10793usize),
            (10795usize, 10795usize),
            (10796usize, 10796usize),
            (10797usize, 10797usize),
            (10798usize, 10798usize),
            (10804usize, 10804usize),
            (10805usize, 10805usize),
            (10812usize, 10812usize),
            (10813usize, 10813usize),
            (10814usize, 10814usize),
            (10839usize, 10839usize),
            (10840usize, 10840usize),
            (10852usize, 10852usize),
            (10853usize, 10853usize),
            (10858usize, 10858usize),
            (10859usize, 10859usize),
            (10860usize, 10860usize),
            (10861usize, 10861usize),
            (10863usize, 10863usize),
            (10864usize, 10864usize),
            (10867usize, 10867usize),
            (10868usize, 10868usize),
            (10873usize, 10873usize),
            (10874usize, 10874usize),
            (10875usize, 10875usize),
            (10876usize, 10876usize),
            (10877usize, 10877usize),
            (10878usize, 10878usize),
            (10879usize, 10879usize),
            (10880usize, 10880usize),
            (10881usize, 10881usize),
            (10882usize, 10882usize),
            (10883usize, 10883usize),
            (10884usize, 10884usize),
            (10885usize, 10885usize),
            (10886usize, 10886usize),
            (10887usize, 10887usize),
            (10888usize, 10888usize),
            (10889usize, 10889usize),
            (10890usize, 10890usize),
            (10891usize, 10891usize),
            (10892usize, 10892usize),
            (10893usize, 10893usize),
            (10894usize, 10894usize),
            (10895usize, 10895usize),
            (10896usize, 10896usize),
            (10897usize, 10897usize),
            (10898usize, 10898usize),
            (10899usize, 10899usize),
            (10900usize, 10900usize),
            (10901usize, 10901usize),
            (10902usize, 10902usize),
            (10903usize, 10903usize),
            (10904usize, 10904usize),
            (10905usize, 10905usize),
            (10906usize, 10906usize),
            (10907usize, 10907usize),
            (10908usize, 10908usize),
            (10909usize, 10909usize),
            (10910usize, 10910usize),
            (10911usize, 10911usize),
            (10912usize, 10912usize),
            (10913usize, 10913usize),
            (10914usize, 10914usize),
            (10915usize, 10915usize),
            (10918usize, 10918usize),
            (10919usize, 10919usize),
            (10920usize, 10920usize),
            (10921usize, 10921usize),
            (10922usize, 10922usize),
            (10923usize, 10923usize),
            (10924usize, 10924usize),
            (10925usize, 10925usize),
            (10927usize, 10927usize),
            (10928usize, 10928usize),
            (10929usize, 10929usize),
            (10930usize, 10930usize),
            (10931usize, 10931usize),
            (10932usize, 10932usize),
            (10933usize, 10933usize),
            (10934usize, 10934usize),
            (10935usize, 10935usize),
            (10936usize, 10936usize),
            (10937usize, 10937usize),
            (10938usize, 10938usize),
            (10939usize, 10939usize),
            (10940usize, 10940usize),
            (10941usize, 10941usize),
            (10942usize, 10942usize),
            (10943usize, 10943usize),
            (10944usize, 10944usize),
            (10945usize, 10945usize),
            (10946usize, 10946usize),
            (10947usize, 10947usize),
            (10948usize, 10948usize),
            (10949usize, 10949usize),
            (10950usize, 10950usize),
            (10951usize, 10951usize),
            (10952usize, 10952usize),
            (10953usize, 10953usize),
            (10954usize, 10954usize),
            (10955usize, 10955usize),
            (10956usize, 10956usize),
            (10957usize, 10957usize),
            (10958usize, 10958usize),
            (10959usize, 10959usize),
            (10960usize, 10960usize),
            (10961usize, 10961usize),
            (10962usize, 10962usize),
            (10963usize, 10963usize),
            (10964usize, 10964usize),
            (10965usize, 10965usize),
            (10966usize, 10966usize),
            (10972usize, 10972usize),
            (10974usize, 10974usize),
            (10978usize, 10978usize),
            (10979usize, 10979usize),
            (10980usize, 10980usize),
            (10981usize, 10981usize),
            (10982usize, 10982usize),
            (10988usize, 10988usize),
            (10989usize, 10989usize),
            (10990usize, 10990usize),
            (10995usize, 10995usize),
            (10999usize, 10999usize),
            (11000usize, 11000usize),
            (11001usize, 11001usize),
            (11002usize, 11002usize),
            (11003usize, 11003usize),
            (11005usize, 11005usize),
            (11262usize, 11262usize),
            (11778usize, 11778usize),
            (11779usize, 11779usize),
            (11780usize, 11780usize),
            (11781usize, 11781usize),
            (11785usize, 11785usize),
            (11786usize, 11786usize),
            (11788usize, 11788usize),
            (11789usize, 11789usize),
            (11804usize, 11804usize),
            (11805usize, 11805usize),
            (11808usize, 11808usize),
            (11809usize, 11809usize),
            (11810usize, 11810usize),
            (11811usize, 11811usize),
            (11812usize, 11812usize),
            (11813usize, 11813usize),
            (11814usize, 11814usize),
            (11815usize, 11815usize),
            (11816usize, 11816usize),
            (11817usize, 11817usize),
            (11861usize, 11861usize),
            (11862usize, 11862usize),
            (11863usize, 11863usize),
            (11864usize, 11864usize),
            (11865usize, 11865usize),
            (11866usize, 11866usize),
            (11867usize, 11867usize),
            (11868usize, 11868usize),
            (11874usize, 11874usize),
            (11875usize, 11875usize),
            (12296usize, 12296usize),
            (12297usize, 12297usize),
            (12298usize, 12298usize),
            (12299usize, 12299usize),
            (12300usize, 12300usize),
            (12301usize, 12301usize),
            (12302usize, 12302usize),
            (12303usize, 12303usize),
            (12304usize, 12304usize),
            (12305usize, 12305usize),
            (12308usize, 12308usize),
            (12309usize, 12309usize),
            (12310usize, 12310usize),
            (12311usize, 12311usize),
            (12312usize, 12312usize),
            (12313usize, 12313usize),
            (12314usize, 12314usize),
            (12315usize, 12315usize),
            (65113usize, 65113usize),
            (65114usize, 65114usize),
            (65115usize, 65115usize),
            (65116usize, 65116usize),
            (65117usize, 65117usize),
            (65118usize, 65118usize),
            (65124usize, 65124usize),
            (65125usize, 65125usize),
            (65288usize, 65288usize),
            (65289usize, 65289usize),
            (65308usize, 65308usize),
            (65310usize, 65310usize),
            (65339usize, 65339usize),
            (65341usize, 65341usize),
            (65371usize, 65371usize),
            (65373usize, 65373usize),
            (65375usize, 65375usize),
            (65376usize, 65376usize),
            (65378usize, 65378usize),
            (65379usize, 65379usize),
            (118493usize, 118493usize),
            (118494usize, 118494usize),
            (118495usize, 118495usize),
            (120539usize, 120539usize),
            (120597usize, 120597usize),
            (120655usize, 120655usize),
            (120713usize, 120713usize),
            (120771usize, 120771usize),
            (121603usize, 121603usize),
            (121604usize, 121604usize),
            (121605usize, 121605usize),
            (121606usize, 121606usize),
            (121607usize, 121607usize),
            (121608usize, 121608usize),
            (121609usize, 121609usize),
            (121616usize, 121616usize),
            (121623usize, 121623usize),
            (121624usize, 121624usize),
            (121627usize, 121627usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Bidi_Mirrored}",
                    r"\P{Bidi_M}",
                    r"[\p{Bidi_M}]",
                    r"[^\p{Bidi_Mirrored}]",
                    r"[\P{Bidi_M}K]",
                    r"[\p{Bidi_M}K]",
                    r"[\p{Bidi_M}\P{Bidi_Mirrored}]",
                    r"[^\P{Bidi_M}K]",
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
                        let bidi_mirrored = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Bidi_Mirrored}" | r"[\p{Bidi_M}]" => bidi_mirrored,
                            r"\P{Bidi_M}" | r"[^\p{Bidi_Mirrored}]" => !bidi_mirrored,
                            r"[\P{Bidi_M}K]" => !bidi_mirrored || k,
                            r"[\p{Bidi_M}K]" => bidi_mirrored || k,
                            r"[\p{Bidi_M}\P{Bidi_Mirrored}]" => true,
                            r"[^\P{Bidi_M}K]" => bidi_mirrored && !k,
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
    fn unicode_bidi_mirrored_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Bidi_M}",
            r"\P{Bidi_Mirrored}",
            r"^[\p{Bidi_Mirrored}]$",
            r"[\P{Bidi_M}K]",
            r"[^\P{Bidi_M}K]",
            r"[\p{Bidi_Mirrored}]",
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
                            "😀\u{27}\u{28}\u{29}\u{2a}\u{3b}\u{3c}\u{3d}\u{3e}\u{3f}\u{5a}\u{5b}\u{5c}\u{5d}\u{5e}\u{7a}\u{7b}\u{7c}\u{7d}\u{7e}\u{aa}\u{ab}\u{ac}\u{ba}\u{bb}\u{bc}\u{f39}\u{f3a}\u{f3d}\u{f3e}\u{169a}\u{169b}\u{169c}\u{169d}\u{2038}\u{2039}\u{203a}\u{203b}\u{2044}\u{2045}\u{2046}\u{2047}\u{207c}\u{207d}\u{207e}\u{207f}\u{208c}\u{208d}\u{208e}\u{208f}\u{213f}\u{2140}\u{2141}\u{2200}\u{2201}\u{2204}\u{2205}\u{2207}\u{2208}\u{220d}\u{220e}\u{2210}\u{2211}\u{2212}\u{2214}\u{2215}\u{2216}\u{2217}\u{2219}\u{221a}\u{221d}\u{221e}\u{221f}\u{2222}\u{2223}\u{2224}\u{2225}\u{2226}\u{2227}\u{222a}\u{222b}\u{2233}\u{2234}\u{2238}\u{2239}\u{223a}\u{223b}\u{224c}\u{224d}\u{2251}\u{2252}\u{2255}\u{2256}\u{225e}\u{225f}\u{2260}\u{2261}\u{2262}\u{2263}\u{2264}\u{226b}\u{226c}\u{226d}\u{228c}\u{228d}\u{228e}\u{228f}\u{2292}\u{2293}\u{2297}\u{2298}\u{2299}\u{22a1}\u{22a2}\u{22a3}\u{22a4}\u{22a5}\u{22a6}\u{22b8}\u{22b9}\u{22bd}\u{22be}\u{22bf}\u{22c0}\u{22c8}\u{22c9}\u{22cd}\u{22ce}\u{22cf}\u{22d0}\u{22d1}\u{22d2}\u{22d5}\u{22d6}\u{22ed}\u{22ee}\u{22ef}\u{22f0}\u{22ff}\u{2300}\u{2307}\u{2308}\u{230b}\u{230c}\u{231f}\u{2320}\u{2321}\u{2322}\u{2328}\u{2329}\u{232a}\u{232b}\u{2767}\u{2768}\u{2775}\u{2776}\u{27bf}\u{27c0}\u{27c1}\u{27c2}\u{27c3}\u{27c6}\u{27c7}\u{27c8}\u{27c9}\u{27ca}\u{27cb}\u{27cd}\u{27ce}\u{27d2}\u{27d3}\u{27d6}\u{27d7}\u{27db}\u{27dc}\u{27de}\u{27df}\u{27e1}\u{27e2}\u{27ef}\u{27f0}\u{2982}\u{2983}\u{2998}\u{2999}\u{299a}\u{299b}\u{29a0}\u{29a1}\u{29a2}\u{29af}\u{29b0}\u{29b7}\u{29b8}\u{29b9}\u{29bf}\u{29c0}\u{29c5}\u{29c6}\u{29c8}\u{29c9}\u{29ca}\u{29cd}\u{29ce}\u{29d2}\u{29d3}\u{29d4}\u{29d5}\u{29d6}\u{29d7}\u{29d8}\u{29dc}\u{29dd}\u{29e0}\u{29e1}\u{29e2}\u{29e3}\u{29e5}\u{29e6}\u{29e7}\u{29e8}\u{29e9}\u{29ea}\u{29f3}\u{29f4}\u{29f9}\u{29fa}\u{29fb}\u{29fc}\u{29fd}\u{29fe}\u{2a09}\u{2a0a}\u{2a1c}\u{2a1d}\u{2a1e}\u{2a21}\u{2a22}\u{2a23}\u{2a24}\u{2a25}\u{2a26}\u{2a27}\u{2a28}\u{2a29}\u{2a2a}\u{2a2b}\u{2a2e}\u{2a2f}\u{2a33}\u{2a34}\u{2a35}\u{2a36}\u{2a3b}\u{2a3c}\u{2a3e}\u{2a3f}\u{2a56}\u{2a57}\u{2a58}\u{2a59}\u{2a63}\u{2a64}\u{2a65}\u{2a66}\u{2a69}\u{2a6a}\u{2a6d}\u{2a6e}\u{2a6f}\u{2a70}\u{2a71}\u{2a72}\u{2a73}\u{2a74}\u{2a75}\u{2a78}\u{2a79}\u{2aa3}\u{2aa4}\u{2aa5}\u{2aa6}\u{2aad}\u{2aae}\u{2aaf}\u{2ad6}\u{2ad7}\u{2adb}\u{2adc}\u{2add}\u{2ade}\u{2adf}\u{2ae1}\u{2ae2}\u{2ae6}\u{2ae7}\u{2aeb}\u{2aec}\u{2aee}\u{2aef}\u{2af2}\u{2af3}\u{2af4}\u{2af6}\u{2af7}\u{2afb}\u{2afc}\u{2afd}\u{2afe}\u{2bfd}\u{2bfe}\u{2bff}\u{2e01}\u{2e02}\u{2e05}\u{2e06}\u{2e08}\u{2e09}\u{2e0a}\u{2e0b}\u{2e0c}\u{2e0d}\u{2e0e}\u{2e1b}\u{2e1c}\u{2e1d}\u{2e1e}\u{2e1f}\u{2e20}\u{2e29}\u{2e2a}\u{2e54}\u{2e55}\u{2e5c}\u{2e5d}\u{2e61}\u{2e64}\u{3007}\u{3008}\u{3011}\u{3012}\u{3013}\u{3014}\u{301b}\u{301c}\u{fe58}\u{fe59}\u{fe5e}\u{fe5f}\u{fe63}\u{fe64}\u{fe65}\u{fe66}\u{ff07}\u{ff08}\u{ff09}\u{ff0a}\u{ff1b}\u{ff1c}\u{ff1d}\u{ff1e}\u{ff1f}\u{ff3a}\u{ff3b}\u{ff3c}\u{ff3d}\u{ff3e}\u{ff5a}\u{ff5b}\u{ff5c}\u{ff5d}\u{ff5e}\u{ff5f}\u{ff60}\u{ff61}\u{ff62}\u{ff63}\u{ff64}\u{1cedc}\u{1cee0}\u{1d6da}\u{1d6db}\u{1d6dc}\u{1d714}\u{1d715}\u{1d716}\u{1d74e}\u{1d74f}\u{1d750}\u{1d788}\u{1d789}\u{1d78a}\u{1d7c2}\u{1d7c3}\u{1d7c4}\u{1db02}\u{1db0a}\u{1db0f}\u{1db11}\u{1db16}\u{1db19}\u{1db1a}\u{1db1c}\u{feff}ſK\n",
                            "X\r\n\u{28}\u{2028}\u{ab}\u{2029}",
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
    fn unicode_bidi_mirrored_property_work_deduplication_and_contracts() {
        let source =
            JsString::from(format!("[{}]", r"\p{Bidi_M}\P{Bidi_Mirrored}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Bidi_M}$", r"^[\P{Bidi_Mirrored}K]$"] {
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
                plan.find(&JsString::from("\u{28}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Bidi_M}--A]",
            r"[\p{Bidi_M}&&A]",
            r"[[\p{Bidi_M}]]",
            r"[A-\p{Bidi_M}]",
            r"[\p{Bidi_M}-A]",
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
                &JsString::from(r"[\p{Bidi_M}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_extender_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (183usize, 183usize),
            (720usize, 721usize),
            (1600usize, 1600usize),
            (2042usize, 2042usize),
            (2673usize, 2673usize),
            (2811usize, 2811usize),
            (2900usize, 2901usize),
            (3654usize, 3654usize),
            (3782usize, 3782usize),
            (6154usize, 6154usize),
            (6211usize, 6211usize),
            (6823usize, 6823usize),
            (7222usize, 7222usize),
            (7291usize, 7291usize),
            (12293usize, 12293usize),
            (12337usize, 12341usize),
            (12445usize, 12446usize),
            (12540usize, 12542usize),
            (40981usize, 40981usize),
            (42508usize, 42508usize),
            (43471usize, 43471usize),
            (43494usize, 43494usize),
            (43632usize, 43632usize),
            (43741usize, 43741usize),
            (43763usize, 43764usize),
            (65392usize, 65392usize),
            (67457usize, 67458usize),
            (68942usize, 68942usize),
            (68970usize, 68970usize),
            (68975usize, 68975usize),
            (70199usize, 70199usize),
            (70493usize, 70493usize),
            (70610usize, 70610usize),
            (70611usize, 70611usize),
            (71110usize, 71112usize),
            (72344usize, 72344usize),
            (73177usize, 73177usize),
            (92994usize, 92995usize),
            (94176usize, 94177usize),
            (94179usize, 94179usize),
            (94194usize, 94195usize),
            (123196usize, 123197usize),
            (124399usize, 124399usize),
            (125252usize, 125254usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Extender}",
                    r"\P{Ext}",
                    r"[\p{Ext}]",
                    r"[^\p{Extender}]",
                    r"[\P{Ext}K]",
                    r"[\p{Ext}K]",
                    r"[\p{Ext}\P{Extender}]",
                    r"[^\P{Ext}K]",
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
                        let extender = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Extender}" | r"[\p{Ext}]" => extender,
                            r"\P{Ext}" | r"[^\p{Extender}]" => !extender,
                            r"[\P{Ext}K]" => !extender || k,
                            r"[\p{Ext}K]" => extender || k,
                            r"[\p{Ext}\P{Extender}]" => true,
                            r"[^\P{Ext}K]" => extender && !k,
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
    fn unicode_extender_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Ext}",
            r"\P{Extender}",
            r"^[\p{Extender}]$",
            r"[\P{Ext}K]",
            r"[^\P{Ext}K]",
            r"[\p{Extender}]",
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
                            "😀\u{b6}\u{b7}\u{b8}\u{2cf}\u{2d0}\u{2d1}\u{2d2}\u{63f}\u{640}\u{641}\u{7f9}\u{7fa}\u{7fb}\u{a70}\u{a71}\u{a72}\u{afa}\u{afb}\u{afc}\u{b53}\u{b55}\u{b56}\u{e45}\u{e46}\u{e47}\u{ec5}\u{ec6}\u{ec7}\u{1809}\u{180a}\u{180b}\u{1842}\u{1843}\u{1844}\u{1aa6}\u{1aa7}\u{1aa8}\u{1c35}\u{1c36}\u{1c37}\u{1c7a}\u{1c7b}\u{1c7c}\u{3004}\u{3005}\u{3006}\u{3030}\u{3031}\u{3035}\u{3036}\u{309c}\u{309d}\u{309e}\u{309f}\u{30fb}\u{30fc}\u{30fe}\u{30ff}\u{a014}\u{a015}\u{a016}\u{a60b}\u{a60c}\u{a60d}\u{a9ce}\u{a9cf}\u{a9d0}\u{a9e5}\u{a9e6}\u{a9e7}\u{aa6f}\u{aa70}\u{aa71}\u{aadc}\u{aadd}\u{aade}\u{aaf2}\u{aaf3}\u{aaf4}\u{aaf5}\u{ff6f}\u{ff70}\u{ff71}\u{10780}\u{10781}\u{10782}\u{10783}\u{10d4d}\u{10d4e}\u{10d4f}\u{10d69}\u{10d6a}\u{10d6b}\u{10d6e}\u{10d6f}\u{10d70}\u{11236}\u{11237}\u{11238}\u{1135c}\u{1135d}\u{1135e}\u{113d1}\u{113d2}\u{113d3}\u{113d4}\u{115c5}\u{115c6}\u{115c8}\u{115c9}\u{11a97}\u{11a98}\u{11a99}\u{11dd8}\u{11dd9}\u{11dda}\u{16b41}\u{16b42}\u{16b43}\u{16b44}\u{16fdf}\u{16fe0}\u{16fe1}\u{16fe2}\u{16fe3}\u{16fe4}\u{16ff1}\u{16ff2}\u{16ff3}\u{16ff4}\u{1e13b}\u{1e13c}\u{1e13d}\u{1e13e}\u{1e5ee}\u{1e5ef}\u{1e5f0}\u{1e943}\u{1e944}\u{1e946}\u{1e947}\u{feff}ſK\n",
                            "X\r\n\u{b7}\u{2028}\u{640}\u{2029}",
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
    fn unicode_extender_property_work_deduplication_and_contracts() {
        // New tags 32/33 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{Ext}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0xb54, true),
                (0x640, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{Ext}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0xb54, false),
                (0x640, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source = JsString::from(format!("[{}]", r"\p{Ext}\P{Extender}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Ext}$", r"^[\P{Extender}K]$"] {
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
                plan.find(&JsString::from("\u{b7}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Ext}--A]",
            r"[\p{Ext}&&A]",
            r"[[\p{Ext}]]",
            r"[A-\p{Ext}]",
            r"[\p{Ext}-A]",
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
                &JsString::from(r"[\p{Ext}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_sentence_terminal_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (33usize, 33usize),
            (46usize, 46usize),
            (63usize, 63usize),
            (1417usize, 1417usize),
            (1565usize, 1567usize),
            (1748usize, 1748usize),
            (1792usize, 1794usize),
            (2041usize, 2041usize),
            (2103usize, 2103usize),
            (2105usize, 2105usize),
            (2109usize, 2110usize),
            (2404usize, 2405usize),
            (4170usize, 4171usize),
            (4962usize, 4962usize),
            (4967usize, 4968usize),
            (5742usize, 5742usize),
            (5941usize, 5942usize),
            (6100usize, 6101usize),
            (6147usize, 6147usize),
            (6153usize, 6153usize),
            (6468usize, 6469usize),
            (6824usize, 6827usize),
            (6990usize, 6991usize),
            (7002usize, 7003usize),
            (7006usize, 7007usize),
            (7037usize, 7039usize),
            (7227usize, 7228usize),
            (7294usize, 7295usize),
            (8228usize, 8228usize),
            (8252usize, 8253usize),
            (8263usize, 8265usize),
            (11513usize, 11515usize),
            (11822usize, 11822usize),
            (11836usize, 11836usize),
            (11859usize, 11860usize),
            (11872usize, 11873usize),
            (12290usize, 12290usize),
            (42239usize, 42239usize),
            (42510usize, 42511usize),
            (42739usize, 42739usize),
            (42743usize, 42743usize),
            (43126usize, 43127usize),
            (43214usize, 43215usize),
            (43311usize, 43311usize),
            (43464usize, 43465usize),
            (43613usize, 43615usize),
            (43760usize, 43761usize),
            (44011usize, 44011usize),
            (65042usize, 65042usize),
            (65045usize, 65046usize),
            (65106usize, 65106usize),
            (65110usize, 65111usize),
            (65281usize, 65281usize),
            (65294usize, 65294usize),
            (65311usize, 65311usize),
            (65377usize, 65377usize),
            (68182usize, 68183usize),
            (69461usize, 69465usize),
            (69510usize, 69513usize),
            (69703usize, 69704usize),
            (69822usize, 69825usize),
            (69953usize, 69955usize),
            (70085usize, 70086usize),
            (70093usize, 70093usize),
            (70110usize, 70111usize),
            (70200usize, 70201usize),
            (70203usize, 70204usize),
            (70313usize, 70313usize),
            (70612usize, 70613usize),
            (70731usize, 70732usize),
            (71106usize, 71107usize),
            (71113usize, 71127usize),
            (71233usize, 71234usize),
            (71484usize, 71486usize),
            (72004usize, 72004usize),
            (72006usize, 72006usize),
            (72258usize, 72259usize),
            (72347usize, 72348usize),
            (72769usize, 72770usize),
            (73463usize, 73464usize),
            (73539usize, 73540usize),
            (92782usize, 92783usize),
            (92917usize, 92917usize),
            (92983usize, 92984usize),
            (92996usize, 92996usize),
            (93550usize, 93551usize),
            (93848usize, 93848usize),
            (113823usize, 113823usize),
            (121480usize, 121480usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Sentence_Terminal}",
                    r"\P{STerm}",
                    r"[\p{STerm}]",
                    r"[^\p{Sentence_Terminal}]",
                    r"[\P{STerm}K]",
                    r"[\p{STerm}K]",
                    r"[\p{STerm}\P{Sentence_Terminal}]",
                    r"[^\P{STerm}K]",
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
                        let sentence_terminal = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Sentence_Terminal}" | r"[\p{STerm}]" => sentence_terminal,
                            r"\P{STerm}" | r"[^\p{Sentence_Terminal}]" => !sentence_terminal,
                            r"[\P{STerm}K]" => !sentence_terminal || k,
                            r"[\p{STerm}K]" => sentence_terminal || k,
                            r"[\p{STerm}\P{Sentence_Terminal}]" => true,
                            r"[^\P{STerm}K]" => sentence_terminal && !k,
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
    fn unicode_sentence_terminal_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{STerm}",
            r"\P{Sentence_Terminal}",
            r"^[\p{Sentence_Terminal}]$",
            r"[\P{STerm}K]",
            r"[^\P{STerm}K]",
            r"[\p{Sentence_Terminal}]",
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
                            "😀\u{20}\u{21}\u{22}\u{2d}\u{2e}\u{2f}\u{3e}\u{3f}\u{40}\u{588}\u{589}\u{58a}\u{61c}\u{61d}\u{61f}\u{620}\u{6d3}\u{6d4}\u{6d5}\u{6ff}\u{700}\u{702}\u{703}\u{7f8}\u{7f9}\u{7fa}\u{836}\u{837}\u{838}\u{839}\u{83a}\u{83c}\u{83d}\u{83e}\u{83f}\u{963}\u{964}\u{965}\u{966}\u{1049}\u{104a}\u{104b}\u{104c}\u{1361}\u{1362}\u{1363}\u{1366}\u{1367}\u{1368}\u{1369}\u{166d}\u{166e}\u{166f}\u{1734}\u{1735}\u{1736}\u{1737}\u{17d3}\u{17d4}\u{17d5}\u{17d6}\u{1802}\u{1803}\u{1804}\u{1808}\u{1809}\u{180a}\u{1943}\u{1944}\u{1945}\u{1946}\u{1aa7}\u{1aa8}\u{1aab}\u{1aac}\u{1b4d}\u{1b4e}\u{1b4f}\u{1b50}\u{1b59}\u{1b5a}\u{1b5b}\u{1b5c}\u{1b5d}\u{1b5e}\u{1b5f}\u{1b60}\u{1b7c}\u{1b7d}\u{1b7f}\u{1b80}\u{1c3a}\u{1c3b}\u{1c3c}\u{1c3d}\u{1c7d}\u{1c7e}\u{1c7f}\u{1c80}\u{2023}\u{2024}\u{2025}\u{203b}\u{203c}\u{203d}\u{203e}\u{2046}\u{2047}\u{2049}\u{204a}\u{2cf8}\u{2cf9}\u{2cfb}\u{2cfc}\u{2e2d}\u{2e2e}\u{2e2f}\u{2e3b}\u{2e3c}\u{2e3d}\u{2e52}\u{2e53}\u{2e54}\u{2e55}\u{2e5f}\u{2e62}\u{3001}\u{3002}\u{3003}\u{a4fe}\u{a4ff}\u{a500}\u{a60d}\u{a60e}\u{a60f}\u{a610}\u{a6f2}\u{a6f3}\u{a6f4}\u{a6f6}\u{a6f7}\u{a6f8}\u{a875}\u{a876}\u{a877}\u{a878}\u{a8cd}\u{a8ce}\u{a8cf}\u{a8d0}\u{a92e}\u{a92f}\u{a930}\u{a9c7}\u{a9c8}\u{a9c9}\u{a9ca}\u{aa5c}\u{aa5d}\u{aa5f}\u{aa60}\u{aaef}\u{aaf0}\u{aaf1}\u{aaf2}\u{abea}\u{abeb}\u{abec}\u{fe11}\u{fe12}\u{fe13}\u{fe14}\u{fe15}\u{fe16}\u{fe17}\u{fe51}\u{fe52}\u{fe53}\u{fe55}\u{fe56}\u{fe57}\u{fe58}\u{ff00}\u{ff01}\u{ff02}\u{ff0d}\u{ff0e}\u{ff0f}\u{ff1e}\u{ff1f}\u{ff20}\u{ff60}\u{ff61}\u{ff62}\u{10a55}\u{10a56}\u{10a57}\u{10a58}\u{10f54}\u{10f55}\u{10f59}\u{10f5a}\u{10f85}\u{10f86}\u{10f89}\u{10f8a}\u{11046}\u{11047}\u{11048}\u{11049}\u{110bd}\u{110be}\u{110c1}\u{110c2}\u{11140}\u{11141}\u{11143}\u{11144}\u{111c4}\u{111c5}\u{111c6}\u{111c7}\u{111cc}\u{111cd}\u{111ce}\u{111dd}\u{111de}\u{111df}\u{111e0}\u{11237}\u{11238}\u{11239}\u{1123a}\u{1123b}\u{1123c}\u{1123d}\u{112a8}\u{112a9}\u{112aa}\u{113d3}\u{113d4}\u{113d5}\u{113d6}\u{1144a}\u{1144b}\u{1144c}\u{1144d}\u{115c1}\u{115c2}\u{115c3}\u{115c4}\u{115c8}\u{115c9}\u{115d7}\u{115d8}\u{11640}\u{11641}\u{11642}\u{11643}\u{1173b}\u{1173c}\u{1173e}\u{1173f}\u{11943}\u{11944}\u{11945}\u{11946}\u{11947}\u{11a41}\u{11a42}\u{11a43}\u{11a44}\u{11a9a}\u{11a9b}\u{11a9c}\u{11a9d}\u{11c40}\u{11c41}\u{11c42}\u{11c43}\u{11ef6}\u{11ef7}\u{11ef8}\u{11ef9}\u{11f42}\u{11f43}\u{11f44}\u{11f45}\u{16a6d}\u{16a6e}\u{16a6f}\u{16a70}\u{16af4}\u{16af5}\u{16af6}\u{16b36}\u{16b37}\u{16b38}\u{16b39}\u{16b43}\u{16b44}\u{16b45}\u{16d6d}\u{16d6e}\u{16d6f}\u{16d70}\u{16e97}\u{16e98}\u{16e99}\u{1bc9e}\u{1bc9f}\u{1bca0}\u{1da87}\u{1da88}\u{1da89}\u{feff}ſK\n",
                            "X\r\n\u{21}\u{2028}\u{3002}\u{2029}",
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
    fn unicode_sentence_terminal_property_work_deduplication_and_contracts() {
        // New tags 34/35 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{STerm}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x2e60, true),
                (0x3002, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{STerm}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x2e60, false),
                (0x3002, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source = JsString::from(
            format!("[{}]", r"\p{STerm}\P{Sentence_Terminal}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{STerm}$", r"^[\P{Sentence_Terminal}K]$"] {
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
                plan.find(&JsString::from("\u{21}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{STerm}--A]",
            r"[\p{STerm}&&A]",
            r"[[\p{STerm}]]",
            r"[A-\p{STerm}]",
            r"[\p{STerm}-A]",
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
                &JsString::from(r"[\p{STerm}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_terminal_punctuation_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (33usize, 33usize),
            (44usize, 44usize),
            (46usize, 46usize),
            (58usize, 59usize),
            (63usize, 63usize),
            (894usize, 894usize),
            (903usize, 903usize),
            (1417usize, 1417usize),
            (1475usize, 1475usize),
            (1548usize, 1548usize),
            (1563usize, 1563usize),
            (1565usize, 1567usize),
            (1748usize, 1748usize),
            (1792usize, 1802usize),
            (1804usize, 1804usize),
            (2040usize, 2041usize),
            (2096usize, 2101usize),
            (2103usize, 2110usize),
            (2142usize, 2142usize),
            (2404usize, 2405usize),
            (3674usize, 3675usize),
            (3848usize, 3848usize),
            (3853usize, 3858usize),
            (4170usize, 4171usize),
            (4961usize, 4968usize),
            (5742usize, 5742usize),
            (5867usize, 5869usize),
            (5941usize, 5942usize),
            (6100usize, 6102usize),
            (6106usize, 6106usize),
            (6146usize, 6149usize),
            (6152usize, 6153usize),
            (6468usize, 6469usize),
            (6824usize, 6827usize),
            (6990usize, 6991usize),
            (7002usize, 7003usize),
            (7005usize, 7007usize),
            (7037usize, 7039usize),
            (7227usize, 7231usize),
            (7294usize, 7295usize),
            (8228usize, 8228usize),
            (8252usize, 8253usize),
            (8263usize, 8265usize),
            (11513usize, 11515usize),
            (11822usize, 11822usize),
            (11836usize, 11836usize),
            (11841usize, 11841usize),
            (11852usize, 11852usize),
            (11854usize, 11855usize),
            (11859usize, 11860usize),
            (11872usize, 11873usize),
            (12289usize, 12290usize),
            (42238usize, 42239usize),
            (42509usize, 42511usize),
            (42739usize, 42743usize),
            (43126usize, 43127usize),
            (43214usize, 43215usize),
            (43311usize, 43311usize),
            (43463usize, 43465usize),
            (43613usize, 43615usize),
            (43743usize, 43743usize),
            (43760usize, 43761usize),
            (44011usize, 44011usize),
            (65042usize, 65042usize),
            (65045usize, 65046usize),
            (65104usize, 65106usize),
            (65108usize, 65111usize),
            (65281usize, 65281usize),
            (65292usize, 65292usize),
            (65294usize, 65294usize),
            (65306usize, 65307usize),
            (65311usize, 65311usize),
            (65377usize, 65377usize),
            (65380usize, 65380usize),
            (66463usize, 66463usize),
            (66512usize, 66512usize),
            (67671usize, 67671usize),
            (67871usize, 67871usize),
            (68182usize, 68183usize),
            (68336usize, 68341usize),
            (68410usize, 68415usize),
            (68505usize, 68508usize),
            (69461usize, 69465usize),
            (69510usize, 69513usize),
            (69703usize, 69709usize),
            (69822usize, 69825usize),
            (69953usize, 69955usize),
            (70085usize, 70086usize),
            (70093usize, 70093usize),
            (70110usize, 70111usize),
            (70200usize, 70204usize),
            (70313usize, 70313usize),
            (70612usize, 70613usize),
            (70731usize, 70733usize),
            (70746usize, 70747usize),
            (71106usize, 71109usize),
            (71113usize, 71127usize),
            (71233usize, 71234usize),
            (71484usize, 71486usize),
            (72004usize, 72004usize),
            (72006usize, 72006usize),
            (72258usize, 72259usize),
            (72347usize, 72348usize),
            (72353usize, 72354usize),
            (72769usize, 72771usize),
            (72817usize, 72817usize),
            (73463usize, 73464usize),
            (73539usize, 73540usize),
            (74864usize, 74868usize),
            (92782usize, 92783usize),
            (92917usize, 92917usize),
            (92983usize, 92985usize),
            (92996usize, 92996usize),
            (93550usize, 93551usize),
            (93847usize, 93848usize),
            (113823usize, 113823usize),
            (121479usize, 121482usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Terminal_Punctuation}",
                    r"\P{Term}",
                    r"[\p{Term}]",
                    r"[^\p{Terminal_Punctuation}]",
                    r"[\P{Term}K]",
                    r"[\p{Term}K]",
                    r"[\p{Term}\P{Terminal_Punctuation}]",
                    r"[^\P{Term}K]",
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
                        let terminal_punctuation = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Terminal_Punctuation}" | r"[\p{Term}]" => terminal_punctuation,
                            r"\P{Term}" | r"[^\p{Terminal_Punctuation}]" => !terminal_punctuation,
                            r"[\P{Term}K]" => !terminal_punctuation || k,
                            r"[\p{Term}K]" => terminal_punctuation || k,
                            r"[\p{Term}\P{Terminal_Punctuation}]" => true,
                            r"[^\P{Term}K]" => terminal_punctuation && !k,
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
    fn unicode_terminal_punctuation_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Term}",
            r"\P{Terminal_Punctuation}",
            r"^[\p{Terminal_Punctuation}]$",
            r"[\P{Term}K]",
            r"[^\P{Term}K]",
            r"[\p{Terminal_Punctuation}]",
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
                            "😀\u{20}\u{21}\u{22}\u{2b}\u{2c}\u{2d}\u{2e}\u{2f}\u{39}\u{3a}\u{3b}\u{3c}\u{3e}\u{3f}\u{40}\u{37d}\u{37e}\u{37f}\u{386}\u{387}\u{388}\u{588}\u{589}\u{58a}\u{5c2}\u{5c3}\u{5c4}\u{60b}\u{60c}\u{60d}\u{61a}\u{61b}\u{61c}\u{61d}\u{61f}\u{620}\u{6d3}\u{6d4}\u{6d5}\u{6ff}\u{700}\u{70a}\u{70b}\u{70c}\u{70d}\u{7f7}\u{7f8}\u{7f9}\u{7fa}\u{82f}\u{830}\u{835}\u{836}\u{837}\u{83e}\u{83f}\u{85d}\u{85e}\u{85f}\u{963}\u{964}\u{965}\u{966}\u{e59}\u{e5a}\u{e5b}\u{e5c}\u{f07}\u{f08}\u{f09}\u{f0c}\u{f0d}\u{f12}\u{f13}\u{1049}\u{104a}\u{104b}\u{104c}\u{1360}\u{1361}\u{1368}\u{1369}\u{166d}\u{166e}\u{166f}\u{16ea}\u{16eb}\u{16ed}\u{16ee}\u{1734}\u{1735}\u{1736}\u{1737}\u{17d3}\u{17d4}\u{17d6}\u{17d7}\u{17d9}\u{17da}\u{17db}\u{1801}\u{1802}\u{1805}\u{1806}\u{1807}\u{1808}\u{1809}\u{180a}\u{1943}\u{1944}\u{1945}\u{1946}\u{1aa7}\u{1aa8}\u{1aab}\u{1aac}\u{1b4d}\u{1b4e}\u{1b4f}\u{1b50}\u{1b59}\u{1b5a}\u{1b5b}\u{1b5c}\u{1b5d}\u{1b5f}\u{1b60}\u{1b7c}\u{1b7d}\u{1b7f}\u{1b80}\u{1c3a}\u{1c3b}\u{1c3f}\u{1c40}\u{1c7d}\u{1c7e}\u{1c7f}\u{1c80}\u{2023}\u{2024}\u{2025}\u{203b}\u{203c}\u{203d}\u{203e}\u{2046}\u{2047}\u{2049}\u{204a}\u{2cf8}\u{2cf9}\u{2cfb}\u{2cfc}\u{2e2d}\u{2e2e}\u{2e2f}\u{2e3b}\u{2e3c}\u{2e3d}\u{2e40}\u{2e41}\u{2e42}\u{2e4b}\u{2e4c}\u{2e4d}\u{2e4e}\u{2e4f}\u{2e50}\u{2e52}\u{2e53}\u{2e54}\u{2e55}\u{2e5f}\u{2e62}\u{3000}\u{3001}\u{3002}\u{3003}\u{a4fd}\u{a4fe}\u{a4ff}\u{a500}\u{a60c}\u{a60d}\u{a60f}\u{a610}\u{a6f2}\u{a6f3}\u{a6f7}\u{a6f8}\u{a875}\u{a876}\u{a877}\u{a878}\u{a8cd}\u{a8ce}\u{a8cf}\u{a8d0}\u{a92e}\u{a92f}\u{a930}\u{a9c6}\u{a9c7}\u{a9c9}\u{a9ca}\u{aa5c}\u{aa5d}\u{aa5f}\u{aa60}\u{aade}\u{aadf}\u{aae0}\u{aaef}\u{aaf0}\u{aaf1}\u{aaf2}\u{abea}\u{abeb}\u{abec}\u{fe11}\u{fe12}\u{fe13}\u{fe14}\u{fe15}\u{fe16}\u{fe17}\u{fe4f}\u{fe50}\u{fe52}\u{fe53}\u{fe54}\u{fe57}\u{fe58}\u{ff00}\u{ff01}\u{ff02}\u{ff0b}\u{ff0c}\u{ff0d}\u{ff0e}\u{ff0f}\u{ff19}\u{ff1a}\u{ff1b}\u{ff1c}\u{ff1e}\u{ff1f}\u{ff20}\u{ff60}\u{ff61}\u{ff62}\u{ff63}\u{ff64}\u{ff65}\u{1039e}\u{1039f}\u{103a0}\u{103cf}\u{103d0}\u{103d1}\u{10856}\u{10857}\u{10858}\u{1091e}\u{1091f}\u{10920}\u{10a55}\u{10a56}\u{10a57}\u{10a58}\u{10aef}\u{10af0}\u{10af5}\u{10af6}\u{10b39}\u{10b3a}\u{10b3f}\u{10b40}\u{10b98}\u{10b99}\u{10b9c}\u{10b9d}\u{10f54}\u{10f55}\u{10f59}\u{10f5a}\u{10f85}\u{10f86}\u{10f89}\u{10f8a}\u{11046}\u{11047}\u{1104d}\u{1104e}\u{110bd}\u{110be}\u{110c1}\u{110c2}\u{11140}\u{11141}\u{11143}\u{11144}\u{111c4}\u{111c5}\u{111c6}\u{111c7}\u{111cc}\u{111cd}\u{111ce}\u{111dd}\u{111de}\u{111df}\u{111e0}\u{11237}\u{11238}\u{1123c}\u{1123d}\u{112a8}\u{112a9}\u{112aa}\u{113d3}\u{113d4}\u{113d5}\u{113d6}\u{1144a}\u{1144b}\u{1144d}\u{1144e}\u{11459}\u{1145a}\u{1145b}\u{1145c}\u{115c1}\u{115c2}\u{115c5}\u{115c6}\u{115c8}\u{115c9}\u{115d7}\u{115d8}\u{11640}\u{11641}\u{11642}\u{11643}\u{1173b}\u{1173c}\u{1173e}\u{1173f}\u{11943}\u{11944}\u{11945}\u{11946}\u{11947}\u{11a41}\u{11a42}\u{11a43}\u{11a44}\u{11a9a}\u{11a9b}\u{11a9c}\u{11a9d}\u{11aa0}\u{11aa1}\u{11aa2}\u{11aa3}\u{11c40}\u{11c41}\u{11c43}\u{11c44}\u{11c70}\u{11c71}\u{11c72}\u{11ef6}\u{11ef7}\u{11ef8}\u{11ef9}\u{11f42}\u{11f43}\u{11f44}\u{11f45}\u{1246f}\u{12470}\u{12474}\u{12475}\u{16a6d}\u{16a6e}\u{16a6f}\u{16a70}\u{16af4}\u{16af5}\u{16af6}\u{16b36}\u{16b37}\u{16b39}\u{16b3a}\u{16b43}\u{16b44}\u{16b45}\u{16d6d}\u{16d6e}\u{16d6f}\u{16d70}\u{16e96}\u{16e97}\u{16e98}\u{16e99}\u{1bc9e}\u{1bc9f}\u{1bca0}\u{1da86}\u{1da87}\u{1da8a}\u{1da8b}\u{feff}ſK\n",
                            "X\r\n\u{21}\u{2028}\u{3002}\u{2029}",
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
    fn unicode_terminal_punctuation_property_work_deduplication_and_contracts() {
        // New tags 36/37 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{Term}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x2e60, true),
                (0x3002, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{Term}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x2e60, false),
                (0x3002, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source = JsString::from(
            format!("[{}]", r"\p{Term}\P{Terminal_Punctuation}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Term}$", r"^[\P{Terminal_Punctuation}K]$"] {
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
                plan.find(&JsString::from("\u{21}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Term}--A]",
            r"[\p{Term}&&A]",
            r"[[\p{Term}]]",
            r"[A-\p{Term}]",
            r"[\p{Term}-A]",
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
                &JsString::from(r"[\p{Term}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_ids_binary_operator_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (12272usize, 12273usize),
            (12276usize, 12285usize),
            (12783usize, 12783usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{IDS_Binary_Operator}",
                    r"\P{IDSB}",
                    r"[\p{IDSB}]",
                    r"[^\p{IDS_Binary_Operator}]",
                    r"[\P{IDSB}K]",
                    r"[\p{IDSB}K]",
                    r"[\p{IDSB}\P{IDS_Binary_Operator}]",
                    r"[^\P{IDSB}K]",
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
                        let ids_binary_operator = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{IDS_Binary_Operator}" | r"[\p{IDSB}]" => ids_binary_operator,
                            r"\P{IDSB}" | r"[^\p{IDS_Binary_Operator}]" => !ids_binary_operator,
                            r"[\P{IDSB}K]" => !ids_binary_operator || k,
                            r"[\p{IDSB}K]" => ids_binary_operator || k,
                            r"[\p{IDSB}\P{IDS_Binary_Operator}]" => true,
                            r"[^\P{IDSB}K]" => ids_binary_operator && !k,
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
    fn unicode_ids_binary_operator_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{IDSB}",
            r"\P{IDS_Binary_Operator}",
            r"^[\p{IDS_Binary_Operator}]$",
            r"[\P{IDSB}K]",
            r"[^\P{IDSB}K]",
            r"[\p{IDS_Binary_Operator}]",
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
                            "😀\u{2fef}\u{2ff0}\u{2ff1}\u{2ff2}\u{2ff3}\u{2ff4}\u{2ffd}\u{2ffe}\u{31ee}\u{31ef}\u{31f0}\u{feff}ſK\n",
                            "X\r\n\u{2ff0}\u{2028}\u{31ef}\u{2029}",
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
    fn unicode_ids_binary_operator_property_work_deduplication_and_contracts() {
        // New tags 38/39 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{IDSB}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x2ff0, true),
                (0x31ef, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{IDSB}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x2ff0, false),
                (0x31ef, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source = JsString::from(
            format!("[{}]", r"\p{IDSB}\P{IDS_Binary_Operator}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{IDSB}$", r"^[\P{IDS_Binary_Operator}K]$"] {
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
                plan.find(&JsString::from("\u{2ff0}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{IDSB}--A]",
            r"[\p{IDSB}&&A]",
            r"[[\p{IDSB}]]",
            r"[A-\p{IDSB}]",
            r"[\p{IDSB}-A]",
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
                &JsString::from(r"[\p{IDSB}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_ids_trinary_operator_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        membership[12274..=12275].fill(true);
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{IDS_Trinary_Operator}",
                    r"\P{IDST}",
                    r"[\p{IDST}]",
                    r"[^\p{IDS_Trinary_Operator}]",
                    r"[\P{IDST}K]",
                    r"[\p{IDST}K]",
                    r"[\p{IDST}\P{IDS_Trinary_Operator}]",
                    r"[^\P{IDST}K]",
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
                        let ids_trinary_operator = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{IDS_Trinary_Operator}" | r"[\p{IDST}]" => ids_trinary_operator,
                            r"\P{IDST}" | r"[^\p{IDS_Trinary_Operator}]" => !ids_trinary_operator,
                            r"[\P{IDST}K]" => !ids_trinary_operator || k,
                            r"[\p{IDST}K]" => ids_trinary_operator || k,
                            r"[\p{IDST}\P{IDS_Trinary_Operator}]" => true,
                            r"[^\P{IDST}K]" => ids_trinary_operator && !k,
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
    fn unicode_ids_trinary_operator_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{IDST}",
            r"\P{IDS_Trinary_Operator}",
            r"^[\p{IDS_Trinary_Operator}]$",
            r"[\P{IDST}K]",
            r"[^\P{IDST}K]",
            r"[\p{IDS_Trinary_Operator}]",
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
                            "😀\u{2ff1}\u{2ff2}\u{2ff3}\u{2ff4}\u{feff}ſK\n",
                            "X\r\n\u{2ff2}\u{2028}\u{2ff3}\u{2029}",
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
    fn unicode_ids_trinary_operator_property_work_deduplication_and_contracts() {
        // New tags 40/41 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{IDST}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x2ff2, true),
                (0x2ff3, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{IDST}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x2ff2, false),
                (0x2ff3, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source = JsString::from(
            format!("[{}]", r"\p{IDST}\P{IDS_Trinary_Operator}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{IDST}$", r"^[\P{IDS_Trinary_Operator}K]$"] {
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
                plan.find(&JsString::from("\u{2ff2}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{IDST}--A]",
            r"[\p{IDST}&&A]",
            r"[[\p{IDST}]]",
            r"[A-\p{IDST}]",
            r"[\p{IDST}-A]",
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
                &JsString::from(r"[\p{IDST}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_radical_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (11904usize, 11929usize),
            (11931usize, 12019usize),
            (12032usize, 12245usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Radical}",
                    r"\P{Radical}",
                    r"[\p{Radical}]",
                    r"[^\p{Radical}]",
                    r"[\P{Radical}K]",
                    r"[\p{Radical}K]",
                    r"[\p{Radical}\P{Radical}]",
                    r"[^\P{Radical}K]",
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
                        let radical = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Radical}" | r"[\p{Radical}]" => radical,
                            r"\P{Radical}" | r"[^\p{Radical}]" => !radical,
                            r"[\P{Radical}K]" => !radical || k,
                            r"[\p{Radical}K]" => radical || k,
                            r"[\p{Radical}\P{Radical}]" => true,
                            r"[^\P{Radical}K]" => radical && !k,
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
    fn unicode_radical_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Radical}",
            r"\P{Radical}",
            r"^[\p{Radical}]$",
            r"[\P{Radical}K]",
            r"[^\P{Radical}K]",
            r"[\p{Radical}]",
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
                            "😀\u{2e7f}\u{2e80}\u{2e99}\u{2e9a}\u{2e9b}\u{2ef3}\u{2ef4}\u{2eff}\u{2f00}\u{2fd5}\u{2fd6}\u{feff}ſK\n",
                            "X\r\n\u{2e80}\u{2028}\u{2fd5}\u{2029}",
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
    fn unicode_radical_property_work_deduplication_and_contracts() {
        // New tags 42/43 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{Radical}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x2e80, true),
                (0x2fd5, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{Radical}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x2e80, false),
                (0x2fd5, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source =
            JsString::from(format!("[{}]", r"\p{Radical}\P{Radical}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Radical}$", r"^[\P{Radical}K]$"] {
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
                plan.find(&JsString::from("\u{2e80}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Radical}--A]",
            r"[\p{Radical}&&A]",
            r"[[\p{Radical}]]",
            r"[A-\p{Radical}]",
            r"[\p{Radical}-A]",
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
                &JsString::from(r"[\p{Radical}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_regional_indicator_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        membership[127462..=127487].fill(true);
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Regional_Indicator}",
                    r"\P{RI}",
                    r"[\p{RI}]",
                    r"[^\p{Regional_Indicator}]",
                    r"[\P{RI}K]",
                    r"[\p{RI}K]",
                    r"[\p{RI}\P{Regional_Indicator}]",
                    r"[^\P{RI}K]",
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
                        let regional_indicator = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Regional_Indicator}" | r"[\p{RI}]" => regional_indicator,
                            r"\P{RI}" | r"[^\p{Regional_Indicator}]" => !regional_indicator,
                            r"[\P{RI}K]" => !regional_indicator || k,
                            r"[\p{RI}K]" => regional_indicator || k,
                            r"[\p{RI}\P{Regional_Indicator}]" => true,
                            r"[^\P{RI}K]" => regional_indicator && !k,
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
    fn unicode_regional_indicator_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{RI}",
            r"\P{Regional_Indicator}",
            r"^[\p{Regional_Indicator}]$",
            r"[\P{RI}K]",
            r"[^\P{RI}K]",
            r"[\p{Regional_Indicator}]",
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
                            "😀\u{1f1e5}\u{1f1e6}\u{1f1ff}\u{1f200}\u{feff}ſK\n",
                            "X\r\n\u{1f1e6}\u{2028}\u{1f1ff}\u{2029}",
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
    fn unicode_regional_indicator_property_work_deduplication_and_contracts() {
        // New tags 44/45 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{RI}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x1f1e6, true),
                (0x1f1ff, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{RI}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x1f1e6, false),
                (0x1f1ff, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source =
            JsString::from(format!("[{}]", r"\p{RI}\P{Regional_Indicator}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{RI}$", r"^[\P{Regional_Indicator}K]$"] {
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
                plan.find(&JsString::from("\u{1f1e6}\n😀\n"), 0, false),
                Some(3..5)
            );
        }
        for source in [
            r"[\p{RI}--A]",
            r"[\p{RI}&&A]",
            r"[[\p{RI}]]",
            r"[A-\p{RI}]",
            r"[\p{RI}-A]",
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
                &JsString::from(r"[\p{RI}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_unified_ideograph_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (13312usize, 19903usize),
            (19968usize, 40959usize),
            (64014usize, 64015usize),
            (64017usize, 64017usize),
            (64019usize, 64020usize),
            (64031usize, 64031usize),
            (64033usize, 64033usize),
            (64035usize, 64036usize),
            (64039usize, 64041usize),
            (131072usize, 173791usize),
            (173824usize, 178206usize),
            (178208usize, 183981usize),
            (183984usize, 191456usize),
            (191472usize, 192093usize),
            (196608usize, 201546usize),
            (201552usize, 210041usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Unified_Ideograph}",
                    r"\P{UIdeo}",
                    r"[\p{UIdeo}]",
                    r"[^\p{Unified_Ideograph}]",
                    r"[\P{UIdeo}K]",
                    r"[\p{UIdeo}K]",
                    r"[\p{UIdeo}\P{Unified_Ideograph}]",
                    r"[^\P{UIdeo}K]",
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
                        let unified_ideograph = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Unified_Ideograph}" | r"[\p{UIdeo}]" => unified_ideograph,
                            r"\P{UIdeo}" | r"[^\p{Unified_Ideograph}]" => !unified_ideograph,
                            r"[\P{UIdeo}K]" => !unified_ideograph || k,
                            r"[\p{UIdeo}K]" => unified_ideograph || k,
                            r"[\p{UIdeo}\P{Unified_Ideograph}]" => true,
                            r"[^\P{UIdeo}K]" => unified_ideograph && !k,
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
    fn unicode_unified_ideograph_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{UIdeo}",
            r"\P{Unified_Ideograph}",
            r"^[\p{Unified_Ideograph}]$",
            r"[\P{UIdeo}K]",
            r"[^\P{UIdeo}K]",
            r"[\p{Unified_Ideograph}]",
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
                            "😀\u{33ff}\u{3400}\u{4dbf}\u{4dc0}\u{4dff}\u{4e00}\u{9fff}\u{a000}\u{fa0d}\u{fa0e}\u{fa0f}\u{fa10}\u{fa11}\u{fa12}\u{fa13}\u{fa14}\u{fa15}\u{fa1e}\u{fa1f}\u{fa20}\u{fa21}\u{fa22}\u{fa23}\u{fa24}\u{fa25}\u{fa26}\u{fa27}\u{fa29}\u{fa2a}\u{1ffff}\u{20000}\u{2a6df}\u{2a6e0}\u{2a6ff}\u{2a700}\u{2b81f}\u{2b820}\u{2cead}\u{2ceae}\u{2ceaf}\u{2ceb0}\u{2ebe0}\u{2ebe1}\u{2ebef}\u{2ebf0}\u{2ee5d}\u{2ee5e}\u{2ffff}\u{30000}\u{3134a}\u{3134b}\u{3134f}\u{31350}\u{33479}\u{3347a}\u{feff}ſK\n",
                            "X\r\n\u{4e00}\u{2028}\u{4e09}\u{2029}",
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
    fn unicode_unified_ideograph_property_work_deduplication_and_contracts() {
        // New tags 46/47 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{UIdeo}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x4e00, true),
                (0x4e8c, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{UIdeo}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x4e00, false),
                (0x4e8c, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source = JsString::from(
            format!("[{}]", r"\p{UIdeo}\P{Unified_Ideograph}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{UIdeo}$", r"^[\P{Unified_Ideograph}K]$"] {
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
                plan.find(&JsString::from("\u{4e00}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{UIdeo}--A]",
            r"[\p{UIdeo}&&A]",
            r"[[\p{UIdeo}]]",
            r"[A-\p{UIdeo}]",
            r"[\p{UIdeo}-A]",
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
                &JsString::from(r"[\p{UIdeo}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
}
