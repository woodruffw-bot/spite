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
/// Ideographic/Ideo uses UCD 18 ideographic membership.
/// Deprecated/Dep uses UCD 18 deprecated membership.
/// Diacritic/Dia uses UCD 18 diacritic membership.
/// Soft_Dotted/SD uses UCD 18 soft-dotted membership.
/// Grapheme_Extend/Gr_Ext uses UCD 18 grapheme-extend membership.
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
            48 => 25,
            50 => 9,
            52 => 230,
            54 => 43,
            56 => 393,
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
        for kind in 0..58 {
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
                    48..=49 => 100,
                    50..=51 => 36,
                    52..=53 => 920,
                    54..=55 => 172,
                    56..=57 => 1572,
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
        [0x49, 0x64, 0x65, 0x6f]
        | [
            0x49,
            0x64,
            0x65,
            0x6f,
            0x67,
            0x72,
            0x61,
            0x70,
            0x68,
            0x69,
            0x63,
        ] => 48,
        [0x44, 0x65, 0x70] | [0x44, 0x65, 0x70, 0x72, 0x65, 0x63, 0x61, 0x74, 0x65, 0x64] => 50,
        [0x44, 0x69, 0x61] | [0x44, 0x69, 0x61, 0x63, 0x72, 0x69, 0x74, 0x69, 0x63] => 52,
        [0x53, 0x44]
        | [
            0x53,
            0x6f,
            0x66,
            0x74,
            0x5f,
            0x44,
            0x6f,
            0x74,
            0x74,
            0x65,
            0x64,
        ] => 54,
        [0x47, 0x72, 0x5f, 0x45, 0x78, 0x74]
        | [
            0x47,
            0x72,
            0x61,
            0x70,
            0x68,
            0x65,
            0x6d,
            0x65,
            0x5f,
            0x45,
            0x78,
            0x74,
            0x65,
            0x6e,
            0x64,
        ] => 56,
        _ => return None,
    })
}

// These interval preimages give the same canonical membership as the standalone
// property plan. Under v+i, complements exclude additional fold preimages
// before union and the shared fold closure. Under u+i, complement the raw set.
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
        // UCD 18 Ideographic: 123244 members, invariant under C/S folding.
        48 => &[
            (0x3006, 0x3007),
            (0x3021, 0x3029),
            (0x3038, 0x303a),
            (0x3400, 0x4dbf),
            (0x4e00, 0x9fff),
            (0xf900, 0xfa6d),
            (0xfa70, 0xfad9),
            (0x16fe4, 0x16fe4),
            (0x16ff2, 0x16ff6),
            (0x17000, 0x18cda),
            (0x18cff, 0x18d20),
            (0x18d80, 0x18df2),
            (0x18e00, 0x19191),
            (0x191a0, 0x191d2),
            (0x1b170, 0x1b2fb),
            (0x20000, 0x2a6df),
            (0x2a700, 0x2b81e),
            (0x2b820, 0x2cead),
            (0x2ceb0, 0x2ebe0),
            (0x2ebf0, 0x2ee5d),
            (0x2f800, 0x2fa1d),
            (0x30000, 0x3134a),
            (0x31350, 0x33479),
            (0x3d000, 0x3fc3f),
        ],
        49 => &[
            (0x0, 0x3005),
            (0x3008, 0x3020),
            (0x302a, 0x3037),
            (0x303b, 0x33ff),
            (0x4dc0, 0x4dff),
            (0xa000, 0xf8ff),
            (0xfa6e, 0xfa6f),
            (0xfada, 0x16fe3),
            (0x16fe5, 0x16ff1),
            (0x16ff7, 0x16fff),
            (0x18cdb, 0x18cfe),
            (0x18d21, 0x18d7f),
            (0x18df3, 0x18dff),
            (0x19192, 0x1919f),
            (0x191d3, 0x1b16f),
            (0x1b2fc, 0x1ffff),
            (0x2a6e0, 0x2a6ff),
            (0x2b81f, 0x2b81f),
            (0x2ceae, 0x2ceaf),
            (0x2ebe1, 0x2ebef),
            (0x2ee5e, 0x2f7ff),
            (0x2fa1e, 0x2ffff),
            (0x3134b, 0x3134f),
            (0x3347a, 0x3cfff),
            (0x3fc40, 0x10ffff),
        ],
        // UCD 18 Deprecated: 15 members, invariant under C/S folding.
        50 => &[
            (0x149, 0x149),
            (0x673, 0x673),
            (0xf77, 0xf77),
            (0xf79, 0xf79),
            (0x17a3, 0x17a4),
            (0x206a, 0x206f),
            (0x2329, 0x232a),
            (0xe0001, 0xe0001),
        ],
        51 => &[
            (0x0, 0x148),
            (0x14a, 0x672),
            (0x674, 0xf76),
            (0xf78, 0xf78),
            (0xf7a, 0x17a2),
            (0x17a5, 0x2069),
            (0x2070, 0x2328),
            (0x232b, 0xe0000),
            (0xe0002, 0x10ffff),
        ],
        // UCD 18 Diacritic: 1334 members; U+0345 folds to nonmember U+03B9.
        52 => &[
            (0x5e, 0x5e),
            (0x60, 0x60),
            (0xa8, 0xa8),
            (0xaf, 0xaf),
            (0xb4, 0xb4),
            (0xb7, 0xb8),
            (0x2b0, 0x34e),
            (0x350, 0x357),
            (0x35d, 0x362),
            (0x374, 0x375),
            (0x37a, 0x37a),
            (0x384, 0x385),
            (0x483, 0x487),
            (0x558, 0x559),
            (0x58b, 0x58c),
            (0x591, 0x5bd),
            (0x5bf, 0x5bf),
            (0x5c1, 0x5c2),
            (0x5c4, 0x5c5),
            (0x5c7, 0x5c9),
            (0x64b, 0x652),
            (0x656, 0x658),
            (0x6df, 0x6e0),
            (0x6e2, 0x6e2),
            (0x6e5, 0x6e6),
            (0x6ea, 0x6ec),
            (0x730, 0x74a),
            (0x7a6, 0x7b0),
            (0x7eb, 0x7f5),
            (0x818, 0x819),
            (0x898, 0x89f),
            (0x8c9, 0x8d3),
            (0x8e3, 0x8fe),
            (0x93c, 0x93c),
            (0x94d, 0x94d),
            (0x951, 0x954),
            (0x971, 0x971),
            (0x9bc, 0x9bc),
            (0x9cd, 0x9cd),
            (0xa3c, 0xa3c),
            (0xa4d, 0xa4d),
            (0xabc, 0xabc),
            (0xacd, 0xacd),
            (0xafd, 0xaff),
            (0xb3c, 0xb3c),
            (0xb4d, 0xb4d),
            (0xb53, 0xb55),
            (0xbcd, 0xbcd),
            (0xc3c, 0xc3c),
            (0xc4d, 0xc4d),
            (0xcbc, 0xcbc),
            (0xccd, 0xccd),
            (0xd3b, 0xd3c),
            (0xd4d, 0xd4d),
            (0xdca, 0xdca),
            (0xe3a, 0xe3a),
            (0xe47, 0xe4c),
            (0xe4e, 0xe4e),
            (0xeba, 0xeba),
            (0xec8, 0xecc),
            (0xf18, 0xf19),
            (0xf35, 0xf35),
            (0xf37, 0xf37),
            (0xf39, 0xf39),
            (0xf3e, 0xf3f),
            (0xf82, 0xf84),
            (0xf86, 0xf87),
            (0xfc6, 0xfc6),
            (0x1037, 0x1037),
            (0x1039, 0x103a),
            (0x1063, 0x1064),
            (0x1069, 0x106d),
            (0x1087, 0x108d),
            (0x108f, 0x108f),
            (0x109a, 0x109b),
            (0x135d, 0x135f),
            (0x1714, 0x1715),
            (0x1734, 0x1734),
            (0x17c9, 0x17d3),
            (0x17dd, 0x17dd),
            (0x1939, 0x193b),
            (0x1a60, 0x1a60),
            (0x1a75, 0x1a7c),
            (0x1a7f, 0x1a7f),
            (0x1ab0, 0x1abe),
            (0x1ac1, 0x1acb),
            (0x1acf, 0x1af0),
            (0x1b34, 0x1b34),
            (0x1b44, 0x1b44),
            (0x1b6b, 0x1b73),
            (0x1baa, 0x1bab),
            (0x1be6, 0x1be6),
            (0x1bf2, 0x1bf3),
            (0x1c36, 0x1c37),
            (0x1c78, 0x1c7d),
            (0x1cd0, 0x1ce8),
            (0x1ced, 0x1ced),
            (0x1cf4, 0x1cf4),
            (0x1cf7, 0x1cf9),
            (0x1d2c, 0x1d6a),
            (0x1d9b, 0x1dbe),
            (0x1dc4, 0x1dcf),
            (0x1df5, 0x1dff),
            (0x1fbd, 0x1fbd),
            (0x1fbf, 0x1fc1),
            (0x1fcd, 0x1fcf),
            (0x1fdd, 0x1fdf),
            (0x1fed, 0x1fef),
            (0x1ffd, 0x1ffe),
            (0x208f, 0x208f),
            (0x2cef, 0x2cf1),
            (0x2e2f, 0x2e2f),
            (0x302a, 0x302f),
            (0x3099, 0x309c),
            (0x30fc, 0x30fc),
            (0xa66f, 0xa66f),
            (0xa67c, 0xa67d),
            (0xa67f, 0xa67f),
            (0xa69c, 0xa69d),
            (0xa6f0, 0xa6f1),
            (0xa700, 0xa721),
            (0xa788, 0xa78a),
            (0xa7f1, 0xa7f1),
            (0xa7f8, 0xa7f9),
            (0xa806, 0xa806),
            (0xa82c, 0xa82c),
            (0xa8c4, 0xa8c4),
            (0xa8e0, 0xa8f1),
            (0xa92b, 0xa92e),
            (0xa953, 0xa953),
            (0xa9b3, 0xa9b3),
            (0xa9c0, 0xa9c0),
            (0xa9e5, 0xa9e5),
            (0xaa7b, 0xaa7d),
            (0xaabf, 0xaac2),
            (0xaaf6, 0xaaf6),
            (0xab5b, 0xab5f),
            (0xab69, 0xab6b),
            (0xabec, 0xabed),
            (0xfb1e, 0xfb1e),
            (0xfe20, 0xfe2f),
            (0xff3e, 0xff3e),
            (0xff40, 0xff40),
            (0xff70, 0xff70),
            (0xff9e, 0xff9f),
            (0xffe3, 0xffe3),
            (0x102e0, 0x102e0),
            (0x10780, 0x10785),
            (0x10787, 0x107b0),
            (0x107b2, 0x107ba),
            (0x10a38, 0x10a3a),
            (0x10a3f, 0x10a3f),
            (0x10ae5, 0x10ae6),
            (0x10d22, 0x10d27),
            (0x10d4e, 0x10d4e),
            (0x10d69, 0x10d6d),
            (0x10ecb, 0x10ecf),
            (0x10ef0, 0x10ef7),
            (0x10ef9, 0x10efa),
            (0x10efd, 0x10eff),
            (0x10f46, 0x10f50),
            (0x10f82, 0x10f85),
            (0x11046, 0x11046),
            (0x11070, 0x11070),
            (0x110b9, 0x110ba),
            (0x11133, 0x11134),
            (0x11173, 0x11173),
            (0x111c0, 0x111c0),
            (0x111ca, 0x111cc),
            (0x11235, 0x11236),
            (0x112e9, 0x112ea),
            (0x1133b, 0x1133c),
            (0x1134d, 0x1134d),
            (0x11366, 0x1136c),
            (0x11370, 0x11374),
            (0x113ce, 0x113d0),
            (0x113d2, 0x113d3),
            (0x113e1, 0x113e2),
            (0x11442, 0x11442),
            (0x11446, 0x11446),
            (0x114c2, 0x114c3),
            (0x115bf, 0x115c0),
            (0x1163f, 0x1163f),
            (0x116b6, 0x116b7),
            (0x1172b, 0x1172b),
            (0x11839, 0x1183a),
            (0x1193d, 0x1193e),
            (0x11943, 0x11943),
            (0x119e0, 0x119e0),
            (0x11a34, 0x11a34),
            (0x11a47, 0x11a47),
            (0x11a99, 0x11a99),
            (0x11c3f, 0x11c3f),
            (0x11d42, 0x11d42),
            (0x11d44, 0x11d45),
            (0x11d97, 0x11d97),
            (0x11dd9, 0x11dd9),
            (0x11f41, 0x11f42),
            (0x11f5a, 0x11f5a),
            (0x13447, 0x13455),
            (0x1612f, 0x1612f),
            (0x16af0, 0x16af4),
            (0x16b30, 0x16b36),
            (0x16d6b, 0x16d6c),
            (0x16f8f, 0x16f9f),
            (0x16ff0, 0x16ff1),
            (0x1aff0, 0x1aff3),
            (0x1aff5, 0x1affb),
            (0x1affd, 0x1affe),
            (0x1cf00, 0x1cf2d),
            (0x1cf30, 0x1cf46),
            (0x1d167, 0x1d169),
            (0x1d16d, 0x1d172),
            (0x1d17b, 0x1d182),
            (0x1d185, 0x1d18b),
            (0x1d1aa, 0x1d1ad),
            (0x1d250, 0x1d252),
            (0x1d25b, 0x1d25c),
            (0x1dfcd, 0x1dfcf),
            (0x1dfd1, 0x1dfff),
            (0x1e030, 0x1e06d),
            (0x1e130, 0x1e136),
            (0x1e2ae, 0x1e2ae),
            (0x1e2ec, 0x1e2ef),
            (0x1e5ee, 0x1e5ef),
            (0x1e8d0, 0x1e8d6),
            (0x1e944, 0x1e946),
            (0x1e948, 0x1e94a),
        ],
        // v+i complements after folding; exclude every Greek iota preimage.
        53 if unicode_sets && ignore_case => &[
            (0x0, 0x5d),
            (0x5f, 0x5f),
            (0x61, 0xa7),
            (0xa9, 0xae),
            (0xb0, 0xb3),
            (0xb5, 0xb6),
            (0xb9, 0x2af),
            (0x34f, 0x34f),
            (0x358, 0x35c),
            (0x363, 0x373),
            (0x376, 0x379),
            (0x37b, 0x383),
            (0x386, 0x398),
            (0x39a, 0x3b8),
            (0x3ba, 0x482),
            (0x488, 0x557),
            (0x55a, 0x58a),
            (0x58d, 0x590),
            (0x5be, 0x5be),
            (0x5c0, 0x5c0),
            (0x5c3, 0x5c3),
            (0x5c6, 0x5c6),
            (0x5ca, 0x64a),
            (0x653, 0x655),
            (0x659, 0x6de),
            (0x6e1, 0x6e1),
            (0x6e3, 0x6e4),
            (0x6e7, 0x6e9),
            (0x6ed, 0x72f),
            (0x74b, 0x7a5),
            (0x7b1, 0x7ea),
            (0x7f6, 0x817),
            (0x81a, 0x897),
            (0x8a0, 0x8c8),
            (0x8d4, 0x8e2),
            (0x8ff, 0x93b),
            (0x93d, 0x94c),
            (0x94e, 0x950),
            (0x955, 0x970),
            (0x972, 0x9bb),
            (0x9bd, 0x9cc),
            (0x9ce, 0xa3b),
            (0xa3d, 0xa4c),
            (0xa4e, 0xabb),
            (0xabd, 0xacc),
            (0xace, 0xafc),
            (0xb00, 0xb3b),
            (0xb3d, 0xb4c),
            (0xb4e, 0xb52),
            (0xb56, 0xbcc),
            (0xbce, 0xc3b),
            (0xc3d, 0xc4c),
            (0xc4e, 0xcbb),
            (0xcbd, 0xccc),
            (0xcce, 0xd3a),
            (0xd3d, 0xd4c),
            (0xd4e, 0xdc9),
            (0xdcb, 0xe39),
            (0xe3b, 0xe46),
            (0xe4d, 0xe4d),
            (0xe4f, 0xeb9),
            (0xebb, 0xec7),
            (0xecd, 0xf17),
            (0xf1a, 0xf34),
            (0xf36, 0xf36),
            (0xf38, 0xf38),
            (0xf3a, 0xf3d),
            (0xf40, 0xf81),
            (0xf85, 0xf85),
            (0xf88, 0xfc5),
            (0xfc7, 0x1036),
            (0x1038, 0x1038),
            (0x103b, 0x1062),
            (0x1065, 0x1068),
            (0x106e, 0x1086),
            (0x108e, 0x108e),
            (0x1090, 0x1099),
            (0x109c, 0x135c),
            (0x1360, 0x1713),
            (0x1716, 0x1733),
            (0x1735, 0x17c8),
            (0x17d4, 0x17dc),
            (0x17de, 0x1938),
            (0x193c, 0x1a5f),
            (0x1a61, 0x1a74),
            (0x1a7d, 0x1a7e),
            (0x1a80, 0x1aaf),
            (0x1abf, 0x1ac0),
            (0x1acc, 0x1ace),
            (0x1af1, 0x1b33),
            (0x1b35, 0x1b43),
            (0x1b45, 0x1b6a),
            (0x1b74, 0x1ba9),
            (0x1bac, 0x1be5),
            (0x1be7, 0x1bf1),
            (0x1bf4, 0x1c35),
            (0x1c38, 0x1c77),
            (0x1c7e, 0x1ccf),
            (0x1ce9, 0x1cec),
            (0x1cee, 0x1cf3),
            (0x1cf5, 0x1cf6),
            (0x1cfa, 0x1d2b),
            (0x1d6b, 0x1d9a),
            (0x1dbf, 0x1dc3),
            (0x1dd0, 0x1df4),
            (0x1e00, 0x1fbc),
            (0x1fc2, 0x1fcc),
            (0x1fd0, 0x1fdc),
            (0x1fe0, 0x1fec),
            (0x1ff0, 0x1ffc),
            (0x1fff, 0x208e),
            (0x2090, 0x2cee),
            (0x2cf2, 0x2e2e),
            (0x2e30, 0x3029),
            (0x3030, 0x3098),
            (0x309d, 0x30fb),
            (0x30fd, 0xa66e),
            (0xa670, 0xa67b),
            (0xa67e, 0xa67e),
            (0xa680, 0xa69b),
            (0xa69e, 0xa6ef),
            (0xa6f2, 0xa6ff),
            (0xa722, 0xa787),
            (0xa78b, 0xa7f0),
            (0xa7f2, 0xa7f7),
            (0xa7fa, 0xa805),
            (0xa807, 0xa82b),
            (0xa82d, 0xa8c3),
            (0xa8c5, 0xa8df),
            (0xa8f2, 0xa92a),
            (0xa92f, 0xa952),
            (0xa954, 0xa9b2),
            (0xa9b4, 0xa9bf),
            (0xa9c1, 0xa9e4),
            (0xa9e6, 0xaa7a),
            (0xaa7e, 0xaabe),
            (0xaac3, 0xaaf5),
            (0xaaf7, 0xab5a),
            (0xab60, 0xab68),
            (0xab6c, 0xabeb),
            (0xabee, 0xfb1d),
            (0xfb1f, 0xfe1f),
            (0xfe30, 0xff3d),
            (0xff3f, 0xff3f),
            (0xff41, 0xff6f),
            (0xff71, 0xff9d),
            (0xffa0, 0xffe2),
            (0xffe4, 0x102df),
            (0x102e1, 0x1077f),
            (0x10786, 0x10786),
            (0x107b1, 0x107b1),
            (0x107bb, 0x10a37),
            (0x10a3b, 0x10a3e),
            (0x10a40, 0x10ae4),
            (0x10ae7, 0x10d21),
            (0x10d28, 0x10d4d),
            (0x10d4f, 0x10d68),
            (0x10d6e, 0x10eca),
            (0x10ed0, 0x10eef),
            (0x10ef8, 0x10ef8),
            (0x10efb, 0x10efc),
            (0x10f00, 0x10f45),
            (0x10f51, 0x10f81),
            (0x10f86, 0x11045),
            (0x11047, 0x1106f),
            (0x11071, 0x110b8),
            (0x110bb, 0x11132),
            (0x11135, 0x11172),
            (0x11174, 0x111bf),
            (0x111c1, 0x111c9),
            (0x111cd, 0x11234),
            (0x11237, 0x112e8),
            (0x112eb, 0x1133a),
            (0x1133d, 0x1134c),
            (0x1134e, 0x11365),
            (0x1136d, 0x1136f),
            (0x11375, 0x113cd),
            (0x113d1, 0x113d1),
            (0x113d4, 0x113e0),
            (0x113e3, 0x11441),
            (0x11443, 0x11445),
            (0x11447, 0x114c1),
            (0x114c4, 0x115be),
            (0x115c1, 0x1163e),
            (0x11640, 0x116b5),
            (0x116b8, 0x1172a),
            (0x1172c, 0x11838),
            (0x1183b, 0x1193c),
            (0x1193f, 0x11942),
            (0x11944, 0x119df),
            (0x119e1, 0x11a33),
            (0x11a35, 0x11a46),
            (0x11a48, 0x11a98),
            (0x11a9a, 0x11c3e),
            (0x11c40, 0x11d41),
            (0x11d43, 0x11d43),
            (0x11d46, 0x11d96),
            (0x11d98, 0x11dd8),
            (0x11dda, 0x11f40),
            (0x11f43, 0x11f59),
            (0x11f5b, 0x13446),
            (0x13456, 0x1612e),
            (0x16130, 0x16aef),
            (0x16af5, 0x16b2f),
            (0x16b37, 0x16d6a),
            (0x16d6d, 0x16f8e),
            (0x16fa0, 0x16fef),
            (0x16ff2, 0x1afef),
            (0x1aff4, 0x1aff4),
            (0x1affc, 0x1affc),
            (0x1afff, 0x1ceff),
            (0x1cf2e, 0x1cf2f),
            (0x1cf47, 0x1d166),
            (0x1d16a, 0x1d16c),
            (0x1d173, 0x1d17a),
            (0x1d183, 0x1d184),
            (0x1d18c, 0x1d1a9),
            (0x1d1ae, 0x1d24f),
            (0x1d253, 0x1d25a),
            (0x1d25d, 0x1dfcc),
            (0x1dfd0, 0x1dfd0),
            (0x1e000, 0x1e02f),
            (0x1e06e, 0x1e12f),
            (0x1e137, 0x1e2ad),
            (0x1e2af, 0x1e2eb),
            (0x1e2f0, 0x1e5ed),
            (0x1e5f0, 0x1e8cf),
            (0x1e8d7, 0x1e943),
            (0x1e947, 0x1e947),
            (0x1e94b, 0x10ffff),
        ],
        53 => &[
            (0x0, 0x5d),
            (0x5f, 0x5f),
            (0x61, 0xa7),
            (0xa9, 0xae),
            (0xb0, 0xb3),
            (0xb5, 0xb6),
            (0xb9, 0x2af),
            (0x34f, 0x34f),
            (0x358, 0x35c),
            (0x363, 0x373),
            (0x376, 0x379),
            (0x37b, 0x383),
            (0x386, 0x482),
            (0x488, 0x557),
            (0x55a, 0x58a),
            (0x58d, 0x590),
            (0x5be, 0x5be),
            (0x5c0, 0x5c0),
            (0x5c3, 0x5c3),
            (0x5c6, 0x5c6),
            (0x5ca, 0x64a),
            (0x653, 0x655),
            (0x659, 0x6de),
            (0x6e1, 0x6e1),
            (0x6e3, 0x6e4),
            (0x6e7, 0x6e9),
            (0x6ed, 0x72f),
            (0x74b, 0x7a5),
            (0x7b1, 0x7ea),
            (0x7f6, 0x817),
            (0x81a, 0x897),
            (0x8a0, 0x8c8),
            (0x8d4, 0x8e2),
            (0x8ff, 0x93b),
            (0x93d, 0x94c),
            (0x94e, 0x950),
            (0x955, 0x970),
            (0x972, 0x9bb),
            (0x9bd, 0x9cc),
            (0x9ce, 0xa3b),
            (0xa3d, 0xa4c),
            (0xa4e, 0xabb),
            (0xabd, 0xacc),
            (0xace, 0xafc),
            (0xb00, 0xb3b),
            (0xb3d, 0xb4c),
            (0xb4e, 0xb52),
            (0xb56, 0xbcc),
            (0xbce, 0xc3b),
            (0xc3d, 0xc4c),
            (0xc4e, 0xcbb),
            (0xcbd, 0xccc),
            (0xcce, 0xd3a),
            (0xd3d, 0xd4c),
            (0xd4e, 0xdc9),
            (0xdcb, 0xe39),
            (0xe3b, 0xe46),
            (0xe4d, 0xe4d),
            (0xe4f, 0xeb9),
            (0xebb, 0xec7),
            (0xecd, 0xf17),
            (0xf1a, 0xf34),
            (0xf36, 0xf36),
            (0xf38, 0xf38),
            (0xf3a, 0xf3d),
            (0xf40, 0xf81),
            (0xf85, 0xf85),
            (0xf88, 0xfc5),
            (0xfc7, 0x1036),
            (0x1038, 0x1038),
            (0x103b, 0x1062),
            (0x1065, 0x1068),
            (0x106e, 0x1086),
            (0x108e, 0x108e),
            (0x1090, 0x1099),
            (0x109c, 0x135c),
            (0x1360, 0x1713),
            (0x1716, 0x1733),
            (0x1735, 0x17c8),
            (0x17d4, 0x17dc),
            (0x17de, 0x1938),
            (0x193c, 0x1a5f),
            (0x1a61, 0x1a74),
            (0x1a7d, 0x1a7e),
            (0x1a80, 0x1aaf),
            (0x1abf, 0x1ac0),
            (0x1acc, 0x1ace),
            (0x1af1, 0x1b33),
            (0x1b35, 0x1b43),
            (0x1b45, 0x1b6a),
            (0x1b74, 0x1ba9),
            (0x1bac, 0x1be5),
            (0x1be7, 0x1bf1),
            (0x1bf4, 0x1c35),
            (0x1c38, 0x1c77),
            (0x1c7e, 0x1ccf),
            (0x1ce9, 0x1cec),
            (0x1cee, 0x1cf3),
            (0x1cf5, 0x1cf6),
            (0x1cfa, 0x1d2b),
            (0x1d6b, 0x1d9a),
            (0x1dbf, 0x1dc3),
            (0x1dd0, 0x1df4),
            (0x1e00, 0x1fbc),
            (0x1fbe, 0x1fbe),
            (0x1fc2, 0x1fcc),
            (0x1fd0, 0x1fdc),
            (0x1fe0, 0x1fec),
            (0x1ff0, 0x1ffc),
            (0x1fff, 0x208e),
            (0x2090, 0x2cee),
            (0x2cf2, 0x2e2e),
            (0x2e30, 0x3029),
            (0x3030, 0x3098),
            (0x309d, 0x30fb),
            (0x30fd, 0xa66e),
            (0xa670, 0xa67b),
            (0xa67e, 0xa67e),
            (0xa680, 0xa69b),
            (0xa69e, 0xa6ef),
            (0xa6f2, 0xa6ff),
            (0xa722, 0xa787),
            (0xa78b, 0xa7f0),
            (0xa7f2, 0xa7f7),
            (0xa7fa, 0xa805),
            (0xa807, 0xa82b),
            (0xa82d, 0xa8c3),
            (0xa8c5, 0xa8df),
            (0xa8f2, 0xa92a),
            (0xa92f, 0xa952),
            (0xa954, 0xa9b2),
            (0xa9b4, 0xa9bf),
            (0xa9c1, 0xa9e4),
            (0xa9e6, 0xaa7a),
            (0xaa7e, 0xaabe),
            (0xaac3, 0xaaf5),
            (0xaaf7, 0xab5a),
            (0xab60, 0xab68),
            (0xab6c, 0xabeb),
            (0xabee, 0xfb1d),
            (0xfb1f, 0xfe1f),
            (0xfe30, 0xff3d),
            (0xff3f, 0xff3f),
            (0xff41, 0xff6f),
            (0xff71, 0xff9d),
            (0xffa0, 0xffe2),
            (0xffe4, 0x102df),
            (0x102e1, 0x1077f),
            (0x10786, 0x10786),
            (0x107b1, 0x107b1),
            (0x107bb, 0x10a37),
            (0x10a3b, 0x10a3e),
            (0x10a40, 0x10ae4),
            (0x10ae7, 0x10d21),
            (0x10d28, 0x10d4d),
            (0x10d4f, 0x10d68),
            (0x10d6e, 0x10eca),
            (0x10ed0, 0x10eef),
            (0x10ef8, 0x10ef8),
            (0x10efb, 0x10efc),
            (0x10f00, 0x10f45),
            (0x10f51, 0x10f81),
            (0x10f86, 0x11045),
            (0x11047, 0x1106f),
            (0x11071, 0x110b8),
            (0x110bb, 0x11132),
            (0x11135, 0x11172),
            (0x11174, 0x111bf),
            (0x111c1, 0x111c9),
            (0x111cd, 0x11234),
            (0x11237, 0x112e8),
            (0x112eb, 0x1133a),
            (0x1133d, 0x1134c),
            (0x1134e, 0x11365),
            (0x1136d, 0x1136f),
            (0x11375, 0x113cd),
            (0x113d1, 0x113d1),
            (0x113d4, 0x113e0),
            (0x113e3, 0x11441),
            (0x11443, 0x11445),
            (0x11447, 0x114c1),
            (0x114c4, 0x115be),
            (0x115c1, 0x1163e),
            (0x11640, 0x116b5),
            (0x116b8, 0x1172a),
            (0x1172c, 0x11838),
            (0x1183b, 0x1193c),
            (0x1193f, 0x11942),
            (0x11944, 0x119df),
            (0x119e1, 0x11a33),
            (0x11a35, 0x11a46),
            (0x11a48, 0x11a98),
            (0x11a9a, 0x11c3e),
            (0x11c40, 0x11d41),
            (0x11d43, 0x11d43),
            (0x11d46, 0x11d96),
            (0x11d98, 0x11dd8),
            (0x11dda, 0x11f40),
            (0x11f43, 0x11f59),
            (0x11f5b, 0x13446),
            (0x13456, 0x1612e),
            (0x16130, 0x16aef),
            (0x16af5, 0x16b2f),
            (0x16b37, 0x16d6a),
            (0x16d6d, 0x16f8e),
            (0x16fa0, 0x16fef),
            (0x16ff2, 0x1afef),
            (0x1aff4, 0x1aff4),
            (0x1affc, 0x1affc),
            (0x1afff, 0x1ceff),
            (0x1cf2e, 0x1cf2f),
            (0x1cf47, 0x1d166),
            (0x1d16a, 0x1d16c),
            (0x1d173, 0x1d17a),
            (0x1d183, 0x1d184),
            (0x1d18c, 0x1d1a9),
            (0x1d1ae, 0x1d24f),
            (0x1d253, 0x1d25a),
            (0x1d25d, 0x1dfcc),
            (0x1dfd0, 0x1dfd0),
            (0x1e000, 0x1e02f),
            (0x1e06e, 0x1e12f),
            (0x1e137, 0x1e2ad),
            (0x1e2af, 0x1e2eb),
            (0x1e2f0, 0x1e5ed),
            (0x1e5f0, 0x1e8cf),
            (0x1e8d7, 0x1e943),
            (0x1e947, 0x1e947),
            (0x1e94b, 0x10ffff),
        ],
        // UCD 18 Soft_Dotted: 54 members; twelve nonmembers fold to members.
        54 => &[
            (0x69, 0x6a),
            (0x12f, 0x12f),
            (0x249, 0x249),
            (0x268, 0x268),
            (0x29d, 0x29d),
            (0x2b2, 0x2b2),
            (0x3f3, 0x3f3),
            (0x456, 0x456),
            (0x458, 0x458),
            (0x1d62, 0x1d62),
            (0x1d96, 0x1d96),
            (0x1da4, 0x1da4),
            (0x1da8, 0x1da8),
            (0x1e2d, 0x1e2d),
            (0x1ecb, 0x1ecb),
            (0x2071, 0x2071),
            (0x2148, 0x2149),
            (0x2c7c, 0x2c7c),
            (0x1d422, 0x1d423),
            (0x1d456, 0x1d457),
            (0x1d48a, 0x1d48b),
            (0x1d4be, 0x1d4bf),
            (0x1d4f2, 0x1d4f3),
            (0x1d526, 0x1d527),
            (0x1d55a, 0x1d55b),
            (0x1d58e, 0x1d58f),
            (0x1d5c2, 0x1d5c3),
            (0x1d5f6, 0x1d5f7),
            (0x1d62a, 0x1d62b),
            (0x1d65e, 0x1d65f),
            (0x1d692, 0x1d693),
            (0x1df1a, 0x1df1a),
            (0x1df6f, 0x1df71),
            (0x1dfd9, 0x1dfd9),
            (0x1e04c, 0x1e04d),
            (0x1e068, 0x1e068),
        ],
        // v+i complements after folding; exclude all additional simple/common-fold preimages.
        55 if unicode_sets && ignore_case => &[
            (0x0, 0x48),
            (0x4b, 0x68),
            (0x6b, 0x12d),
            (0x130, 0x196),
            (0x198, 0x247),
            (0x24a, 0x267),
            (0x269, 0x29c),
            (0x29e, 0x2b1),
            (0x2b3, 0x37e),
            (0x380, 0x3f2),
            (0x3f4, 0x405),
            (0x407, 0x407),
            (0x409, 0x455),
            (0x457, 0x457),
            (0x459, 0x1d61),
            (0x1d63, 0x1d95),
            (0x1d97, 0x1da3),
            (0x1da5, 0x1da7),
            (0x1da9, 0x1e2b),
            (0x1e2e, 0x1ec9),
            (0x1ecc, 0x2070),
            (0x2072, 0x2147),
            (0x214a, 0x2c7b),
            (0x2c7d, 0xa7b1),
            (0xa7b3, 0x1d421),
            (0x1d424, 0x1d455),
            (0x1d458, 0x1d489),
            (0x1d48c, 0x1d4bd),
            (0x1d4c0, 0x1d4f1),
            (0x1d4f4, 0x1d525),
            (0x1d528, 0x1d559),
            (0x1d55c, 0x1d58d),
            (0x1d590, 0x1d5c1),
            (0x1d5c4, 0x1d5f5),
            (0x1d5f8, 0x1d629),
            (0x1d62c, 0x1d65d),
            (0x1d660, 0x1d691),
            (0x1d694, 0x1df19),
            (0x1df1b, 0x1df6d),
            (0x1df72, 0x1dfd8),
            (0x1dfda, 0x1e04b),
            (0x1e04e, 0x1e067),
            (0x1e069, 0x10ffff),
        ],
        55 => &[
            (0x0, 0x68),
            (0x6b, 0x12e),
            (0x130, 0x248),
            (0x24a, 0x267),
            (0x269, 0x29c),
            (0x29e, 0x2b1),
            (0x2b3, 0x3f2),
            (0x3f4, 0x455),
            (0x457, 0x457),
            (0x459, 0x1d61),
            (0x1d63, 0x1d95),
            (0x1d97, 0x1da3),
            (0x1da5, 0x1da7),
            (0x1da9, 0x1e2c),
            (0x1e2e, 0x1eca),
            (0x1ecc, 0x2070),
            (0x2072, 0x2147),
            (0x214a, 0x2c7b),
            (0x2c7d, 0x1d421),
            (0x1d424, 0x1d455),
            (0x1d458, 0x1d489),
            (0x1d48c, 0x1d4bd),
            (0x1d4c0, 0x1d4f1),
            (0x1d4f4, 0x1d525),
            (0x1d528, 0x1d559),
            (0x1d55c, 0x1d58d),
            (0x1d590, 0x1d5c1),
            (0x1d5c4, 0x1d5f5),
            (0x1d5f8, 0x1d629),
            (0x1d62c, 0x1d65d),
            (0x1d660, 0x1d691),
            (0x1d694, 0x1df19),
            (0x1df1b, 0x1df6e),
            (0x1df72, 0x1dfd8),
            (0x1dfda, 0x1e04b),
            (0x1e04e, 0x1e067),
            (0x1e069, 0x10ffff),
        ],
        // UCD 18 Grapheme_Extend: 2269 members; U+0345 folds to nonmember U+03B9.
        56 => &[
            (0x300, 0x36f),
            (0x483, 0x489),
            (0x591, 0x5bd),
            (0x5bf, 0x5bf),
            (0x5c1, 0x5c2),
            (0x5c4, 0x5c5),
            (0x5c7, 0x5c9),
            (0x610, 0x61a),
            (0x64b, 0x65f),
            (0x670, 0x670),
            (0x6d6, 0x6dc),
            (0x6df, 0x6e4),
            (0x6e7, 0x6e8),
            (0x6ea, 0x6ed),
            (0x711, 0x711),
            (0x730, 0x74a),
            (0x7a6, 0x7b0),
            (0x7eb, 0x7f3),
            (0x7fd, 0x7fd),
            (0x816, 0x819),
            (0x81b, 0x823),
            (0x825, 0x827),
            (0x829, 0x82d),
            (0x859, 0x85b),
            (0x897, 0x89f),
            (0x8ca, 0x8e1),
            (0x8e3, 0x902),
            (0x93a, 0x93a),
            (0x93c, 0x93c),
            (0x941, 0x948),
            (0x94d, 0x94d),
            (0x951, 0x957),
            (0x962, 0x963),
            (0x981, 0x981),
            (0x9bc, 0x9bc),
            (0x9be, 0x9be),
            (0x9c1, 0x9c4),
            (0x9cd, 0x9cd),
            (0x9d7, 0x9d7),
            (0x9e2, 0x9e3),
            (0x9fe, 0x9fe),
            (0xa01, 0xa02),
            (0xa3c, 0xa3c),
            (0xa41, 0xa42),
            (0xa47, 0xa48),
            (0xa4b, 0xa4d),
            (0xa51, 0xa51),
            (0xa70, 0xa71),
            (0xa75, 0xa75),
            (0xa81, 0xa82),
            (0xabc, 0xabc),
            (0xac1, 0xac5),
            (0xac7, 0xac8),
            (0xacd, 0xacd),
            (0xae2, 0xae3),
            (0xafa, 0xaff),
            (0xb01, 0xb01),
            (0xb3c, 0xb3c),
            (0xb3e, 0xb3f),
            (0xb41, 0xb44),
            (0xb4d, 0xb4d),
            (0xb53, 0xb57),
            (0xb62, 0xb63),
            (0xb82, 0xb82),
            (0xbbe, 0xbbe),
            (0xbc0, 0xbc0),
            (0xbcd, 0xbcd),
            (0xbd7, 0xbd7),
            (0xc00, 0xc00),
            (0xc04, 0xc04),
            (0xc3c, 0xc3c),
            (0xc3e, 0xc40),
            (0xc46, 0xc48),
            (0xc4a, 0xc4d),
            (0xc55, 0xc56),
            (0xc62, 0xc63),
            (0xc81, 0xc81),
            (0xcbc, 0xcbc),
            (0xcbf, 0xcc0),
            (0xcc2, 0xcc2),
            (0xcc6, 0xcc8),
            (0xcca, 0xccd),
            (0xcd5, 0xcd6),
            (0xce2, 0xce3),
            (0xd00, 0xd01),
            (0xd3b, 0xd3c),
            (0xd3e, 0xd3e),
            (0xd41, 0xd44),
            (0xd4d, 0xd4d),
            (0xd57, 0xd57),
            (0xd62, 0xd63),
            (0xd81, 0xd81),
            (0xdca, 0xdca),
            (0xdcf, 0xdcf),
            (0xdd2, 0xdd4),
            (0xdd6, 0xdd6),
            (0xddf, 0xddf),
            (0xe31, 0xe31),
            (0xe34, 0xe3a),
            (0xe47, 0xe4e),
            (0xeb1, 0xeb1),
            (0xeb4, 0xebc),
            (0xec8, 0xece),
            (0xf18, 0xf19),
            (0xf35, 0xf35),
            (0xf37, 0xf37),
            (0xf39, 0xf39),
            (0xf71, 0xf7e),
            (0xf80, 0xf84),
            (0xf86, 0xf87),
            (0xf8d, 0xf97),
            (0xf99, 0xfbc),
            (0xfc6, 0xfc6),
            (0x102d, 0x1030),
            (0x1032, 0x1037),
            (0x1039, 0x103a),
            (0x103d, 0x103e),
            (0x1058, 0x1059),
            (0x105e, 0x1060),
            (0x1071, 0x1074),
            (0x1082, 0x1082),
            (0x1085, 0x1086),
            (0x108d, 0x108d),
            (0x109d, 0x109d),
            (0x135d, 0x135f),
            (0x1712, 0x1715),
            (0x1732, 0x1734),
            (0x1752, 0x1753),
            (0x1772, 0x1773),
            (0x17b4, 0x17b5),
            (0x17b7, 0x17bd),
            (0x17c6, 0x17c6),
            (0x17c9, 0x17d3),
            (0x17dd, 0x17dd),
            (0x180b, 0x180d),
            (0x180f, 0x180f),
            (0x1885, 0x1886),
            (0x18a9, 0x18a9),
            (0x1920, 0x1922),
            (0x1927, 0x1928),
            (0x1932, 0x1932),
            (0x1939, 0x193b),
            (0x1a17, 0x1a18),
            (0x1a1b, 0x1a1b),
            (0x1a56, 0x1a56),
            (0x1a58, 0x1a5e),
            (0x1a60, 0x1a60),
            (0x1a62, 0x1a62),
            (0x1a65, 0x1a6c),
            (0x1a73, 0x1a7c),
            (0x1a7f, 0x1a7f),
            (0x1ab0, 0x1af0),
            (0x1b00, 0x1b03),
            (0x1b34, 0x1b3d),
            (0x1b42, 0x1b44),
            (0x1b6b, 0x1b73),
            (0x1b80, 0x1b81),
            (0x1ba2, 0x1ba5),
            (0x1ba8, 0x1bad),
            (0x1be6, 0x1be6),
            (0x1be8, 0x1be9),
            (0x1bed, 0x1bed),
            (0x1bef, 0x1bf3),
            (0x1c2c, 0x1c33),
            (0x1c36, 0x1c37),
            (0x1cd0, 0x1cd2),
            (0x1cd4, 0x1ce0),
            (0x1ce2, 0x1ce8),
            (0x1ced, 0x1ced),
            (0x1cf4, 0x1cf4),
            (0x1cf8, 0x1cf9),
            (0x1dc0, 0x1dff),
            (0x200c, 0x200c),
            (0x20d0, 0x20f0),
            (0x2cef, 0x2cf1),
            (0x2d7f, 0x2d7f),
            (0x2de0, 0x2dff),
            (0x302a, 0x302f),
            (0x3099, 0x309a),
            (0xa66f, 0xa672),
            (0xa674, 0xa67d),
            (0xa69e, 0xa69f),
            (0xa6f0, 0xa6f1),
            (0xa802, 0xa802),
            (0xa806, 0xa806),
            (0xa80b, 0xa80b),
            (0xa825, 0xa826),
            (0xa82c, 0xa82c),
            (0xa8c4, 0xa8c5),
            (0xa8e0, 0xa8f1),
            (0xa8ff, 0xa8ff),
            (0xa926, 0xa92d),
            (0xa947, 0xa951),
            (0xa953, 0xa953),
            (0xa980, 0xa982),
            (0xa9b3, 0xa9b3),
            (0xa9b6, 0xa9b9),
            (0xa9bc, 0xa9bd),
            (0xa9c0, 0xa9c0),
            (0xa9e5, 0xa9e5),
            (0xaa29, 0xaa2e),
            (0xaa31, 0xaa32),
            (0xaa35, 0xaa36),
            (0xaa43, 0xaa43),
            (0xaa4c, 0xaa4c),
            (0xaa7c, 0xaa7c),
            (0xaab0, 0xaab0),
            (0xaab2, 0xaab4),
            (0xaab7, 0xaab8),
            (0xaabe, 0xaabf),
            (0xaac1, 0xaac1),
            (0xaaec, 0xaaed),
            (0xaaf6, 0xaaf6),
            (0xabe5, 0xabe5),
            (0xabe8, 0xabe8),
            (0xabed, 0xabed),
            (0xfb1e, 0xfb1e),
            (0xfe00, 0xfe0f),
            (0xfe20, 0xfe2f),
            (0xff9e, 0xff9f),
            (0x101fd, 0x101fd),
            (0x102e0, 0x102e0),
            (0x10376, 0x1037a),
            (0x10a01, 0x10a03),
            (0x10a05, 0x10a06),
            (0x10a0c, 0x10a0f),
            (0x10a38, 0x10a3a),
            (0x10a3f, 0x10a3f),
            (0x10ae5, 0x10ae6),
            (0x10d24, 0x10d27),
            (0x10d69, 0x10d6d),
            (0x10eab, 0x10eac),
            (0x10ecb, 0x10ecf),
            (0x10ef0, 0x10eff),
            (0x10f46, 0x10f50),
            (0x10f82, 0x10f85),
            (0x11001, 0x11001),
            (0x11038, 0x11046),
            (0x11070, 0x11070),
            (0x11073, 0x11074),
            (0x1107f, 0x11081),
            (0x110b3, 0x110b6),
            (0x110b9, 0x110ba),
            (0x110c2, 0x110c2),
            (0x11100, 0x11102),
            (0x11127, 0x1112b),
            (0x1112d, 0x11134),
            (0x11173, 0x11173),
            (0x11180, 0x11181),
            (0x111b6, 0x111be),
            (0x111c0, 0x111c0),
            (0x111c9, 0x111cc),
            (0x111cf, 0x111cf),
            (0x1122f, 0x11231),
            (0x11234, 0x11237),
            (0x1123e, 0x1123e),
            (0x11241, 0x11241),
            (0x112df, 0x112df),
            (0x112e3, 0x112ea),
            (0x11300, 0x11301),
            (0x1133b, 0x1133c),
            (0x1133e, 0x1133e),
            (0x11340, 0x11340),
            (0x1134d, 0x1134d),
            (0x11357, 0x11357),
            (0x11366, 0x1136c),
            (0x11370, 0x11374),
            (0x113b8, 0x113b8),
            (0x113bb, 0x113c0),
            (0x113c2, 0x113c2),
            (0x113c5, 0x113c5),
            (0x113c7, 0x113c9),
            (0x113ce, 0x113d0),
            (0x113d2, 0x113d2),
            (0x113e1, 0x113e2),
            (0x11438, 0x1143f),
            (0x11442, 0x11444),
            (0x11446, 0x11446),
            (0x1145e, 0x1145e),
            (0x114b0, 0x114b0),
            (0x114b3, 0x114b8),
            (0x114ba, 0x114ba),
            (0x114bd, 0x114bd),
            (0x114bf, 0x114c0),
            (0x114c2, 0x114c3),
            (0x115af, 0x115af),
            (0x115b2, 0x115b5),
            (0x115bc, 0x115bd),
            (0x115bf, 0x115c0),
            (0x115dc, 0x115dd),
            (0x11633, 0x1163a),
            (0x1163d, 0x1163d),
            (0x1163f, 0x11640),
            (0x116ab, 0x116ab),
            (0x116ad, 0x116ad),
            (0x116b0, 0x116b7),
            (0x1171d, 0x1171d),
            (0x1171f, 0x1171f),
            (0x11722, 0x11725),
            (0x11727, 0x1172b),
            (0x1182f, 0x11837),
            (0x11839, 0x1183a),
            (0x11930, 0x11930),
            (0x1193b, 0x1193e),
            (0x11943, 0x11943),
            (0x119d4, 0x119d7),
            (0x119da, 0x119db),
            (0x119e0, 0x119e0),
            (0x11a01, 0x11a0a),
            (0x11a33, 0x11a38),
            (0x11a3b, 0x11a3e),
            (0x11a47, 0x11a47),
            (0x11a51, 0x11a56),
            (0x11a59, 0x11a5b),
            (0x11a8a, 0x11a96),
            (0x11a98, 0x11a99),
            (0x11b60, 0x11b60),
            (0x11b62, 0x11b64),
            (0x11b66, 0x11b66),
            (0x11c30, 0x11c36),
            (0x11c38, 0x11c3d),
            (0x11c3f, 0x11c3f),
            (0x11c92, 0x11ca7),
            (0x11caa, 0x11cb0),
            (0x11cb2, 0x11cb3),
            (0x11cb5, 0x11cb6),
            (0x11d31, 0x11d36),
            (0x11d3a, 0x11d3a),
            (0x11d3c, 0x11d3d),
            (0x11d3f, 0x11d45),
            (0x11d47, 0x11d47),
            (0x11d90, 0x11d91),
            (0x11d95, 0x11d95),
            (0x11d97, 0x11d97),
            (0x11df0, 0x11df0),
            (0x11ef3, 0x11ef4),
            (0x11f00, 0x11f01),
            (0x11f36, 0x11f3a),
            (0x11f40, 0x11f42),
            (0x11f5a, 0x11f5a),
            (0x13440, 0x13440),
            (0x13447, 0x13455),
            (0x1611e, 0x16129),
            (0x1612d, 0x1612f),
            (0x16af0, 0x16af4),
            (0x16b30, 0x16b36),
            (0x16f4f, 0x16f4f),
            (0x16f8f, 0x16f92),
            (0x16fe4, 0x16fe4),
            (0x16ff0, 0x16ff1),
            (0x1bc9d, 0x1bc9e),
            (0x1cf00, 0x1cf2d),
            (0x1cf30, 0x1cf46),
            (0x1d127, 0x1d128),
            (0x1d165, 0x1d169),
            (0x1d16d, 0x1d172),
            (0x1d17b, 0x1d182),
            (0x1d185, 0x1d18b),
            (0x1d1aa, 0x1d1ad),
            (0x1d242, 0x1d244),
            (0x1d250, 0x1d252),
            (0x1d25b, 0x1d25c),
            (0x1d25f, 0x1d25f),
            (0x1d280, 0x1d281),
            (0x1da00, 0x1da36),
            (0x1da3b, 0x1da6c),
            (0x1da75, 0x1da75),
            (0x1da84, 0x1da84),
            (0x1da9b, 0x1da9f),
            (0x1daa1, 0x1daaf),
            (0x1e000, 0x1e006),
            (0x1e008, 0x1e018),
            (0x1e01b, 0x1e021),
            (0x1e023, 0x1e024),
            (0x1e026, 0x1e02a),
            (0x1e08f, 0x1e08f),
            (0x1e130, 0x1e136),
            (0x1e2ae, 0x1e2ae),
            (0x1e2ec, 0x1e2ef),
            (0x1e4ec, 0x1e4ef),
            (0x1e5ee, 0x1e5ef),
            (0x1e6e3, 0x1e6e3),
            (0x1e6e6, 0x1e6e6),
            (0x1e6ee, 0x1e6ef),
            (0x1e6f5, 0x1e6f5),
            (0x1e8d0, 0x1e8d6),
            (0x1e944, 0x1e94a),
            (0xe0020, 0xe007f),
            (0xe0100, 0xe01ef),
        ],
        // v+i complements after folding; exclude every Greek iota preimage.
        57 if unicode_sets && ignore_case => &[
            (0x0, 0x2ff),
            (0x370, 0x398),
            (0x39a, 0x3b8),
            (0x3ba, 0x482),
            (0x48a, 0x590),
            (0x5be, 0x5be),
            (0x5c0, 0x5c0),
            (0x5c3, 0x5c3),
            (0x5c6, 0x5c6),
            (0x5ca, 0x60f),
            (0x61b, 0x64a),
            (0x660, 0x66f),
            (0x671, 0x6d5),
            (0x6dd, 0x6de),
            (0x6e5, 0x6e6),
            (0x6e9, 0x6e9),
            (0x6ee, 0x710),
            (0x712, 0x72f),
            (0x74b, 0x7a5),
            (0x7b1, 0x7ea),
            (0x7f4, 0x7fc),
            (0x7fe, 0x815),
            (0x81a, 0x81a),
            (0x824, 0x824),
            (0x828, 0x828),
            (0x82e, 0x858),
            (0x85c, 0x896),
            (0x8a0, 0x8c9),
            (0x8e2, 0x8e2),
            (0x903, 0x939),
            (0x93b, 0x93b),
            (0x93d, 0x940),
            (0x949, 0x94c),
            (0x94e, 0x950),
            (0x958, 0x961),
            (0x964, 0x980),
            (0x982, 0x9bb),
            (0x9bd, 0x9bd),
            (0x9bf, 0x9c0),
            (0x9c5, 0x9cc),
            (0x9ce, 0x9d6),
            (0x9d8, 0x9e1),
            (0x9e4, 0x9fd),
            (0x9ff, 0xa00),
            (0xa03, 0xa3b),
            (0xa3d, 0xa40),
            (0xa43, 0xa46),
            (0xa49, 0xa4a),
            (0xa4e, 0xa50),
            (0xa52, 0xa6f),
            (0xa72, 0xa74),
            (0xa76, 0xa80),
            (0xa83, 0xabb),
            (0xabd, 0xac0),
            (0xac6, 0xac6),
            (0xac9, 0xacc),
            (0xace, 0xae1),
            (0xae4, 0xaf9),
            (0xb00, 0xb00),
            (0xb02, 0xb3b),
            (0xb3d, 0xb3d),
            (0xb40, 0xb40),
            (0xb45, 0xb4c),
            (0xb4e, 0xb52),
            (0xb58, 0xb61),
            (0xb64, 0xb81),
            (0xb83, 0xbbd),
            (0xbbf, 0xbbf),
            (0xbc1, 0xbcc),
            (0xbce, 0xbd6),
            (0xbd8, 0xbff),
            (0xc01, 0xc03),
            (0xc05, 0xc3b),
            (0xc3d, 0xc3d),
            (0xc41, 0xc45),
            (0xc49, 0xc49),
            (0xc4e, 0xc54),
            (0xc57, 0xc61),
            (0xc64, 0xc80),
            (0xc82, 0xcbb),
            (0xcbd, 0xcbe),
            (0xcc1, 0xcc1),
            (0xcc3, 0xcc5),
            (0xcc9, 0xcc9),
            (0xcce, 0xcd4),
            (0xcd7, 0xce1),
            (0xce4, 0xcff),
            (0xd02, 0xd3a),
            (0xd3d, 0xd3d),
            (0xd3f, 0xd40),
            (0xd45, 0xd4c),
            (0xd4e, 0xd56),
            (0xd58, 0xd61),
            (0xd64, 0xd80),
            (0xd82, 0xdc9),
            (0xdcb, 0xdce),
            (0xdd0, 0xdd1),
            (0xdd5, 0xdd5),
            (0xdd7, 0xdde),
            (0xde0, 0xe30),
            (0xe32, 0xe33),
            (0xe3b, 0xe46),
            (0xe4f, 0xeb0),
            (0xeb2, 0xeb3),
            (0xebd, 0xec7),
            (0xecf, 0xf17),
            (0xf1a, 0xf34),
            (0xf36, 0xf36),
            (0xf38, 0xf38),
            (0xf3a, 0xf70),
            (0xf7f, 0xf7f),
            (0xf85, 0xf85),
            (0xf88, 0xf8c),
            (0xf98, 0xf98),
            (0xfbd, 0xfc5),
            (0xfc7, 0x102c),
            (0x1031, 0x1031),
            (0x1038, 0x1038),
            (0x103b, 0x103c),
            (0x103f, 0x1057),
            (0x105a, 0x105d),
            (0x1061, 0x1070),
            (0x1075, 0x1081),
            (0x1083, 0x1084),
            (0x1087, 0x108c),
            (0x108e, 0x109c),
            (0x109e, 0x135c),
            (0x1360, 0x1711),
            (0x1716, 0x1731),
            (0x1735, 0x1751),
            (0x1754, 0x1771),
            (0x1774, 0x17b3),
            (0x17b6, 0x17b6),
            (0x17be, 0x17c5),
            (0x17c7, 0x17c8),
            (0x17d4, 0x17dc),
            (0x17de, 0x180a),
            (0x180e, 0x180e),
            (0x1810, 0x1884),
            (0x1887, 0x18a8),
            (0x18aa, 0x191f),
            (0x1923, 0x1926),
            (0x1929, 0x1931),
            (0x1933, 0x1938),
            (0x193c, 0x1a16),
            (0x1a19, 0x1a1a),
            (0x1a1c, 0x1a55),
            (0x1a57, 0x1a57),
            (0x1a5f, 0x1a5f),
            (0x1a61, 0x1a61),
            (0x1a63, 0x1a64),
            (0x1a6d, 0x1a72),
            (0x1a7d, 0x1a7e),
            (0x1a80, 0x1aaf),
            (0x1af1, 0x1aff),
            (0x1b04, 0x1b33),
            (0x1b3e, 0x1b41),
            (0x1b45, 0x1b6a),
            (0x1b74, 0x1b7f),
            (0x1b82, 0x1ba1),
            (0x1ba6, 0x1ba7),
            (0x1bae, 0x1be5),
            (0x1be7, 0x1be7),
            (0x1bea, 0x1bec),
            (0x1bee, 0x1bee),
            (0x1bf4, 0x1c2b),
            (0x1c34, 0x1c35),
            (0x1c38, 0x1ccf),
            (0x1cd3, 0x1cd3),
            (0x1ce1, 0x1ce1),
            (0x1ce9, 0x1cec),
            (0x1cee, 0x1cf3),
            (0x1cf5, 0x1cf7),
            (0x1cfa, 0x1dbf),
            (0x1e00, 0x1fbd),
            (0x1fbf, 0x200b),
            (0x200d, 0x20cf),
            (0x20f1, 0x2cee),
            (0x2cf2, 0x2d7e),
            (0x2d80, 0x2ddf),
            (0x2e00, 0x3029),
            (0x3030, 0x3098),
            (0x309b, 0xa66e),
            (0xa673, 0xa673),
            (0xa67e, 0xa69d),
            (0xa6a0, 0xa6ef),
            (0xa6f2, 0xa801),
            (0xa803, 0xa805),
            (0xa807, 0xa80a),
            (0xa80c, 0xa824),
            (0xa827, 0xa82b),
            (0xa82d, 0xa8c3),
            (0xa8c6, 0xa8df),
            (0xa8f2, 0xa8fe),
            (0xa900, 0xa925),
            (0xa92e, 0xa946),
            (0xa952, 0xa952),
            (0xa954, 0xa97f),
            (0xa983, 0xa9b2),
            (0xa9b4, 0xa9b5),
            (0xa9ba, 0xa9bb),
            (0xa9be, 0xa9bf),
            (0xa9c1, 0xa9e4),
            (0xa9e6, 0xaa28),
            (0xaa2f, 0xaa30),
            (0xaa33, 0xaa34),
            (0xaa37, 0xaa42),
            (0xaa44, 0xaa4b),
            (0xaa4d, 0xaa7b),
            (0xaa7d, 0xaaaf),
            (0xaab1, 0xaab1),
            (0xaab5, 0xaab6),
            (0xaab9, 0xaabd),
            (0xaac0, 0xaac0),
            (0xaac2, 0xaaeb),
            (0xaaee, 0xaaf5),
            (0xaaf7, 0xabe4),
            (0xabe6, 0xabe7),
            (0xabe9, 0xabec),
            (0xabee, 0xfb1d),
            (0xfb1f, 0xfdff),
            (0xfe10, 0xfe1f),
            (0xfe30, 0xff9d),
            (0xffa0, 0x101fc),
            (0x101fe, 0x102df),
            (0x102e1, 0x10375),
            (0x1037b, 0x10a00),
            (0x10a04, 0x10a04),
            (0x10a07, 0x10a0b),
            (0x10a10, 0x10a37),
            (0x10a3b, 0x10a3e),
            (0x10a40, 0x10ae4),
            (0x10ae7, 0x10d23),
            (0x10d28, 0x10d68),
            (0x10d6e, 0x10eaa),
            (0x10ead, 0x10eca),
            (0x10ed0, 0x10eef),
            (0x10f00, 0x10f45),
            (0x10f51, 0x10f81),
            (0x10f86, 0x11000),
            (0x11002, 0x11037),
            (0x11047, 0x1106f),
            (0x11071, 0x11072),
            (0x11075, 0x1107e),
            (0x11082, 0x110b2),
            (0x110b7, 0x110b8),
            (0x110bb, 0x110c1),
            (0x110c3, 0x110ff),
            (0x11103, 0x11126),
            (0x1112c, 0x1112c),
            (0x11135, 0x11172),
            (0x11174, 0x1117f),
            (0x11182, 0x111b5),
            (0x111bf, 0x111bf),
            (0x111c1, 0x111c8),
            (0x111cd, 0x111ce),
            (0x111d0, 0x1122e),
            (0x11232, 0x11233),
            (0x11238, 0x1123d),
            (0x1123f, 0x11240),
            (0x11242, 0x112de),
            (0x112e0, 0x112e2),
            (0x112eb, 0x112ff),
            (0x11302, 0x1133a),
            (0x1133d, 0x1133d),
            (0x1133f, 0x1133f),
            (0x11341, 0x1134c),
            (0x1134e, 0x11356),
            (0x11358, 0x11365),
            (0x1136d, 0x1136f),
            (0x11375, 0x113b7),
            (0x113b9, 0x113ba),
            (0x113c1, 0x113c1),
            (0x113c3, 0x113c4),
            (0x113c6, 0x113c6),
            (0x113ca, 0x113cd),
            (0x113d1, 0x113d1),
            (0x113d3, 0x113e0),
            (0x113e3, 0x11437),
            (0x11440, 0x11441),
            (0x11445, 0x11445),
            (0x11447, 0x1145d),
            (0x1145f, 0x114af),
            (0x114b1, 0x114b2),
            (0x114b9, 0x114b9),
            (0x114bb, 0x114bc),
            (0x114be, 0x114be),
            (0x114c1, 0x114c1),
            (0x114c4, 0x115ae),
            (0x115b0, 0x115b1),
            (0x115b6, 0x115bb),
            (0x115be, 0x115be),
            (0x115c1, 0x115db),
            (0x115de, 0x11632),
            (0x1163b, 0x1163c),
            (0x1163e, 0x1163e),
            (0x11641, 0x116aa),
            (0x116ac, 0x116ac),
            (0x116ae, 0x116af),
            (0x116b8, 0x1171c),
            (0x1171e, 0x1171e),
            (0x11720, 0x11721),
            (0x11726, 0x11726),
            (0x1172c, 0x1182e),
            (0x11838, 0x11838),
            (0x1183b, 0x1192f),
            (0x11931, 0x1193a),
            (0x1193f, 0x11942),
            (0x11944, 0x119d3),
            (0x119d8, 0x119d9),
            (0x119dc, 0x119df),
            (0x119e1, 0x11a00),
            (0x11a0b, 0x11a32),
            (0x11a39, 0x11a3a),
            (0x11a3f, 0x11a46),
            (0x11a48, 0x11a50),
            (0x11a57, 0x11a58),
            (0x11a5c, 0x11a89),
            (0x11a97, 0x11a97),
            (0x11a9a, 0x11b5f),
            (0x11b61, 0x11b61),
            (0x11b65, 0x11b65),
            (0x11b67, 0x11c2f),
            (0x11c37, 0x11c37),
            (0x11c3e, 0x11c3e),
            (0x11c40, 0x11c91),
            (0x11ca8, 0x11ca9),
            (0x11cb1, 0x11cb1),
            (0x11cb4, 0x11cb4),
            (0x11cb7, 0x11d30),
            (0x11d37, 0x11d39),
            (0x11d3b, 0x11d3b),
            (0x11d3e, 0x11d3e),
            (0x11d46, 0x11d46),
            (0x11d48, 0x11d8f),
            (0x11d92, 0x11d94),
            (0x11d96, 0x11d96),
            (0x11d98, 0x11def),
            (0x11df1, 0x11ef2),
            (0x11ef5, 0x11eff),
            (0x11f02, 0x11f35),
            (0x11f3b, 0x11f3f),
            (0x11f43, 0x11f59),
            (0x11f5b, 0x1343f),
            (0x13441, 0x13446),
            (0x13456, 0x1611d),
            (0x1612a, 0x1612c),
            (0x16130, 0x16aef),
            (0x16af5, 0x16b2f),
            (0x16b37, 0x16f4e),
            (0x16f50, 0x16f8e),
            (0x16f93, 0x16fe3),
            (0x16fe5, 0x16fef),
            (0x16ff2, 0x1bc9c),
            (0x1bc9f, 0x1ceff),
            (0x1cf2e, 0x1cf2f),
            (0x1cf47, 0x1d126),
            (0x1d129, 0x1d164),
            (0x1d16a, 0x1d16c),
            (0x1d173, 0x1d17a),
            (0x1d183, 0x1d184),
            (0x1d18c, 0x1d1a9),
            (0x1d1ae, 0x1d241),
            (0x1d245, 0x1d24f),
            (0x1d253, 0x1d25a),
            (0x1d25d, 0x1d25e),
            (0x1d260, 0x1d27f),
            (0x1d282, 0x1d9ff),
            (0x1da37, 0x1da3a),
            (0x1da6d, 0x1da74),
            (0x1da76, 0x1da83),
            (0x1da85, 0x1da9a),
            (0x1daa0, 0x1daa0),
            (0x1dab0, 0x1dfff),
            (0x1e007, 0x1e007),
            (0x1e019, 0x1e01a),
            (0x1e022, 0x1e022),
            (0x1e025, 0x1e025),
            (0x1e02b, 0x1e08e),
            (0x1e090, 0x1e12f),
            (0x1e137, 0x1e2ad),
            (0x1e2af, 0x1e2eb),
            (0x1e2f0, 0x1e4eb),
            (0x1e4f0, 0x1e5ed),
            (0x1e5f0, 0x1e6e2),
            (0x1e6e4, 0x1e6e5),
            (0x1e6e7, 0x1e6ed),
            (0x1e6f0, 0x1e6f4),
            (0x1e6f6, 0x1e8cf),
            (0x1e8d7, 0x1e943),
            (0x1e94b, 0xe001f),
            (0xe0080, 0xe00ff),
            (0xe01f0, 0x10ffff),
        ],
        57 => &[
            (0x0, 0x2ff),
            (0x370, 0x482),
            (0x48a, 0x590),
            (0x5be, 0x5be),
            (0x5c0, 0x5c0),
            (0x5c3, 0x5c3),
            (0x5c6, 0x5c6),
            (0x5ca, 0x60f),
            (0x61b, 0x64a),
            (0x660, 0x66f),
            (0x671, 0x6d5),
            (0x6dd, 0x6de),
            (0x6e5, 0x6e6),
            (0x6e9, 0x6e9),
            (0x6ee, 0x710),
            (0x712, 0x72f),
            (0x74b, 0x7a5),
            (0x7b1, 0x7ea),
            (0x7f4, 0x7fc),
            (0x7fe, 0x815),
            (0x81a, 0x81a),
            (0x824, 0x824),
            (0x828, 0x828),
            (0x82e, 0x858),
            (0x85c, 0x896),
            (0x8a0, 0x8c9),
            (0x8e2, 0x8e2),
            (0x903, 0x939),
            (0x93b, 0x93b),
            (0x93d, 0x940),
            (0x949, 0x94c),
            (0x94e, 0x950),
            (0x958, 0x961),
            (0x964, 0x980),
            (0x982, 0x9bb),
            (0x9bd, 0x9bd),
            (0x9bf, 0x9c0),
            (0x9c5, 0x9cc),
            (0x9ce, 0x9d6),
            (0x9d8, 0x9e1),
            (0x9e4, 0x9fd),
            (0x9ff, 0xa00),
            (0xa03, 0xa3b),
            (0xa3d, 0xa40),
            (0xa43, 0xa46),
            (0xa49, 0xa4a),
            (0xa4e, 0xa50),
            (0xa52, 0xa6f),
            (0xa72, 0xa74),
            (0xa76, 0xa80),
            (0xa83, 0xabb),
            (0xabd, 0xac0),
            (0xac6, 0xac6),
            (0xac9, 0xacc),
            (0xace, 0xae1),
            (0xae4, 0xaf9),
            (0xb00, 0xb00),
            (0xb02, 0xb3b),
            (0xb3d, 0xb3d),
            (0xb40, 0xb40),
            (0xb45, 0xb4c),
            (0xb4e, 0xb52),
            (0xb58, 0xb61),
            (0xb64, 0xb81),
            (0xb83, 0xbbd),
            (0xbbf, 0xbbf),
            (0xbc1, 0xbcc),
            (0xbce, 0xbd6),
            (0xbd8, 0xbff),
            (0xc01, 0xc03),
            (0xc05, 0xc3b),
            (0xc3d, 0xc3d),
            (0xc41, 0xc45),
            (0xc49, 0xc49),
            (0xc4e, 0xc54),
            (0xc57, 0xc61),
            (0xc64, 0xc80),
            (0xc82, 0xcbb),
            (0xcbd, 0xcbe),
            (0xcc1, 0xcc1),
            (0xcc3, 0xcc5),
            (0xcc9, 0xcc9),
            (0xcce, 0xcd4),
            (0xcd7, 0xce1),
            (0xce4, 0xcff),
            (0xd02, 0xd3a),
            (0xd3d, 0xd3d),
            (0xd3f, 0xd40),
            (0xd45, 0xd4c),
            (0xd4e, 0xd56),
            (0xd58, 0xd61),
            (0xd64, 0xd80),
            (0xd82, 0xdc9),
            (0xdcb, 0xdce),
            (0xdd0, 0xdd1),
            (0xdd5, 0xdd5),
            (0xdd7, 0xdde),
            (0xde0, 0xe30),
            (0xe32, 0xe33),
            (0xe3b, 0xe46),
            (0xe4f, 0xeb0),
            (0xeb2, 0xeb3),
            (0xebd, 0xec7),
            (0xecf, 0xf17),
            (0xf1a, 0xf34),
            (0xf36, 0xf36),
            (0xf38, 0xf38),
            (0xf3a, 0xf70),
            (0xf7f, 0xf7f),
            (0xf85, 0xf85),
            (0xf88, 0xf8c),
            (0xf98, 0xf98),
            (0xfbd, 0xfc5),
            (0xfc7, 0x102c),
            (0x1031, 0x1031),
            (0x1038, 0x1038),
            (0x103b, 0x103c),
            (0x103f, 0x1057),
            (0x105a, 0x105d),
            (0x1061, 0x1070),
            (0x1075, 0x1081),
            (0x1083, 0x1084),
            (0x1087, 0x108c),
            (0x108e, 0x109c),
            (0x109e, 0x135c),
            (0x1360, 0x1711),
            (0x1716, 0x1731),
            (0x1735, 0x1751),
            (0x1754, 0x1771),
            (0x1774, 0x17b3),
            (0x17b6, 0x17b6),
            (0x17be, 0x17c5),
            (0x17c7, 0x17c8),
            (0x17d4, 0x17dc),
            (0x17de, 0x180a),
            (0x180e, 0x180e),
            (0x1810, 0x1884),
            (0x1887, 0x18a8),
            (0x18aa, 0x191f),
            (0x1923, 0x1926),
            (0x1929, 0x1931),
            (0x1933, 0x1938),
            (0x193c, 0x1a16),
            (0x1a19, 0x1a1a),
            (0x1a1c, 0x1a55),
            (0x1a57, 0x1a57),
            (0x1a5f, 0x1a5f),
            (0x1a61, 0x1a61),
            (0x1a63, 0x1a64),
            (0x1a6d, 0x1a72),
            (0x1a7d, 0x1a7e),
            (0x1a80, 0x1aaf),
            (0x1af1, 0x1aff),
            (0x1b04, 0x1b33),
            (0x1b3e, 0x1b41),
            (0x1b45, 0x1b6a),
            (0x1b74, 0x1b7f),
            (0x1b82, 0x1ba1),
            (0x1ba6, 0x1ba7),
            (0x1bae, 0x1be5),
            (0x1be7, 0x1be7),
            (0x1bea, 0x1bec),
            (0x1bee, 0x1bee),
            (0x1bf4, 0x1c2b),
            (0x1c34, 0x1c35),
            (0x1c38, 0x1ccf),
            (0x1cd3, 0x1cd3),
            (0x1ce1, 0x1ce1),
            (0x1ce9, 0x1cec),
            (0x1cee, 0x1cf3),
            (0x1cf5, 0x1cf7),
            (0x1cfa, 0x1dbf),
            (0x1e00, 0x200b),
            (0x200d, 0x20cf),
            (0x20f1, 0x2cee),
            (0x2cf2, 0x2d7e),
            (0x2d80, 0x2ddf),
            (0x2e00, 0x3029),
            (0x3030, 0x3098),
            (0x309b, 0xa66e),
            (0xa673, 0xa673),
            (0xa67e, 0xa69d),
            (0xa6a0, 0xa6ef),
            (0xa6f2, 0xa801),
            (0xa803, 0xa805),
            (0xa807, 0xa80a),
            (0xa80c, 0xa824),
            (0xa827, 0xa82b),
            (0xa82d, 0xa8c3),
            (0xa8c6, 0xa8df),
            (0xa8f2, 0xa8fe),
            (0xa900, 0xa925),
            (0xa92e, 0xa946),
            (0xa952, 0xa952),
            (0xa954, 0xa97f),
            (0xa983, 0xa9b2),
            (0xa9b4, 0xa9b5),
            (0xa9ba, 0xa9bb),
            (0xa9be, 0xa9bf),
            (0xa9c1, 0xa9e4),
            (0xa9e6, 0xaa28),
            (0xaa2f, 0xaa30),
            (0xaa33, 0xaa34),
            (0xaa37, 0xaa42),
            (0xaa44, 0xaa4b),
            (0xaa4d, 0xaa7b),
            (0xaa7d, 0xaaaf),
            (0xaab1, 0xaab1),
            (0xaab5, 0xaab6),
            (0xaab9, 0xaabd),
            (0xaac0, 0xaac0),
            (0xaac2, 0xaaeb),
            (0xaaee, 0xaaf5),
            (0xaaf7, 0xabe4),
            (0xabe6, 0xabe7),
            (0xabe9, 0xabec),
            (0xabee, 0xfb1d),
            (0xfb1f, 0xfdff),
            (0xfe10, 0xfe1f),
            (0xfe30, 0xff9d),
            (0xffa0, 0x101fc),
            (0x101fe, 0x102df),
            (0x102e1, 0x10375),
            (0x1037b, 0x10a00),
            (0x10a04, 0x10a04),
            (0x10a07, 0x10a0b),
            (0x10a10, 0x10a37),
            (0x10a3b, 0x10a3e),
            (0x10a40, 0x10ae4),
            (0x10ae7, 0x10d23),
            (0x10d28, 0x10d68),
            (0x10d6e, 0x10eaa),
            (0x10ead, 0x10eca),
            (0x10ed0, 0x10eef),
            (0x10f00, 0x10f45),
            (0x10f51, 0x10f81),
            (0x10f86, 0x11000),
            (0x11002, 0x11037),
            (0x11047, 0x1106f),
            (0x11071, 0x11072),
            (0x11075, 0x1107e),
            (0x11082, 0x110b2),
            (0x110b7, 0x110b8),
            (0x110bb, 0x110c1),
            (0x110c3, 0x110ff),
            (0x11103, 0x11126),
            (0x1112c, 0x1112c),
            (0x11135, 0x11172),
            (0x11174, 0x1117f),
            (0x11182, 0x111b5),
            (0x111bf, 0x111bf),
            (0x111c1, 0x111c8),
            (0x111cd, 0x111ce),
            (0x111d0, 0x1122e),
            (0x11232, 0x11233),
            (0x11238, 0x1123d),
            (0x1123f, 0x11240),
            (0x11242, 0x112de),
            (0x112e0, 0x112e2),
            (0x112eb, 0x112ff),
            (0x11302, 0x1133a),
            (0x1133d, 0x1133d),
            (0x1133f, 0x1133f),
            (0x11341, 0x1134c),
            (0x1134e, 0x11356),
            (0x11358, 0x11365),
            (0x1136d, 0x1136f),
            (0x11375, 0x113b7),
            (0x113b9, 0x113ba),
            (0x113c1, 0x113c1),
            (0x113c3, 0x113c4),
            (0x113c6, 0x113c6),
            (0x113ca, 0x113cd),
            (0x113d1, 0x113d1),
            (0x113d3, 0x113e0),
            (0x113e3, 0x11437),
            (0x11440, 0x11441),
            (0x11445, 0x11445),
            (0x11447, 0x1145d),
            (0x1145f, 0x114af),
            (0x114b1, 0x114b2),
            (0x114b9, 0x114b9),
            (0x114bb, 0x114bc),
            (0x114be, 0x114be),
            (0x114c1, 0x114c1),
            (0x114c4, 0x115ae),
            (0x115b0, 0x115b1),
            (0x115b6, 0x115bb),
            (0x115be, 0x115be),
            (0x115c1, 0x115db),
            (0x115de, 0x11632),
            (0x1163b, 0x1163c),
            (0x1163e, 0x1163e),
            (0x11641, 0x116aa),
            (0x116ac, 0x116ac),
            (0x116ae, 0x116af),
            (0x116b8, 0x1171c),
            (0x1171e, 0x1171e),
            (0x11720, 0x11721),
            (0x11726, 0x11726),
            (0x1172c, 0x1182e),
            (0x11838, 0x11838),
            (0x1183b, 0x1192f),
            (0x11931, 0x1193a),
            (0x1193f, 0x11942),
            (0x11944, 0x119d3),
            (0x119d8, 0x119d9),
            (0x119dc, 0x119df),
            (0x119e1, 0x11a00),
            (0x11a0b, 0x11a32),
            (0x11a39, 0x11a3a),
            (0x11a3f, 0x11a46),
            (0x11a48, 0x11a50),
            (0x11a57, 0x11a58),
            (0x11a5c, 0x11a89),
            (0x11a97, 0x11a97),
            (0x11a9a, 0x11b5f),
            (0x11b61, 0x11b61),
            (0x11b65, 0x11b65),
            (0x11b67, 0x11c2f),
            (0x11c37, 0x11c37),
            (0x11c3e, 0x11c3e),
            (0x11c40, 0x11c91),
            (0x11ca8, 0x11ca9),
            (0x11cb1, 0x11cb1),
            (0x11cb4, 0x11cb4),
            (0x11cb7, 0x11d30),
            (0x11d37, 0x11d39),
            (0x11d3b, 0x11d3b),
            (0x11d3e, 0x11d3e),
            (0x11d46, 0x11d46),
            (0x11d48, 0x11d8f),
            (0x11d92, 0x11d94),
            (0x11d96, 0x11d96),
            (0x11d98, 0x11def),
            (0x11df1, 0x11ef2),
            (0x11ef5, 0x11eff),
            (0x11f02, 0x11f35),
            (0x11f3b, 0x11f3f),
            (0x11f43, 0x11f59),
            (0x11f5b, 0x1343f),
            (0x13441, 0x13446),
            (0x13456, 0x1611d),
            (0x1612a, 0x1612c),
            (0x16130, 0x16aef),
            (0x16af5, 0x16b2f),
            (0x16b37, 0x16f4e),
            (0x16f50, 0x16f8e),
            (0x16f93, 0x16fe3),
            (0x16fe5, 0x16fef),
            (0x16ff2, 0x1bc9c),
            (0x1bc9f, 0x1ceff),
            (0x1cf2e, 0x1cf2f),
            (0x1cf47, 0x1d126),
            (0x1d129, 0x1d164),
            (0x1d16a, 0x1d16c),
            (0x1d173, 0x1d17a),
            (0x1d183, 0x1d184),
            (0x1d18c, 0x1d1a9),
            (0x1d1ae, 0x1d241),
            (0x1d245, 0x1d24f),
            (0x1d253, 0x1d25a),
            (0x1d25d, 0x1d25e),
            (0x1d260, 0x1d27f),
            (0x1d282, 0x1d9ff),
            (0x1da37, 0x1da3a),
            (0x1da6d, 0x1da74),
            (0x1da76, 0x1da83),
            (0x1da85, 0x1da9a),
            (0x1daa0, 0x1daa0),
            (0x1dab0, 0x1dfff),
            (0x1e007, 0x1e007),
            (0x1e019, 0x1e01a),
            (0x1e022, 0x1e022),
            (0x1e025, 0x1e025),
            (0x1e02b, 0x1e08e),
            (0x1e090, 0x1e12f),
            (0x1e137, 0x1e2ad),
            (0x1e2af, 0x1e2eb),
            (0x1e2f0, 0x1e4eb),
            (0x1e4f0, 0x1e5ed),
            (0x1e5f0, 0x1e6e2),
            (0x1e6e4, 0x1e6e5),
            (0x1e6e7, 0x1e6ed),
            (0x1e6f0, 0x1e6f4),
            (0x1e6f6, 0x1e8cf),
            (0x1e8d7, 0x1e943),
            (0x1e94b, 0xe001f),
            (0xe0080, 0xe00ff),
            (0xe01f0, 0x10ffff),
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
    #[test]
    fn unicode_ideographic_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (12294usize, 12294usize),
            (12295usize, 12295usize),
            (12321usize, 12329usize),
            (12344usize, 12346usize),
            (13312usize, 19903usize),
            (19968usize, 40959usize),
            (63744usize, 64109usize),
            (64112usize, 64217usize),
            (94180usize, 94180usize),
            (94194usize, 94195usize),
            (94196usize, 94198usize),
            (94208usize, 101594usize),
            (101631usize, 101664usize),
            (101760usize, 101874usize),
            (101888usize, 102801usize),
            (102816usize, 102866usize),
            (110960usize, 111355usize),
            (131072usize, 173791usize),
            (173824usize, 178206usize),
            (178208usize, 183981usize),
            (183984usize, 191456usize),
            (191472usize, 192093usize),
            (194560usize, 195101usize),
            (196608usize, 201546usize),
            (201552usize, 210041usize),
            (249856usize, 261183usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Ideographic}",
                    r"\P{Ideo}",
                    r"[\p{Ideo}]",
                    r"[^\p{Ideographic}]",
                    r"[\P{Ideo}K]",
                    r"[\p{Ideo}K]",
                    r"[\p{Ideo}\P{Ideographic}]",
                    r"[^\P{Ideo}K]",
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
                        let ideographic = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Ideographic}" | r"[\p{Ideo}]" => ideographic,
                            r"\P{Ideo}" | r"[^\p{Ideographic}]" => !ideographic,
                            r"[\P{Ideo}K]" => !ideographic || k,
                            r"[\p{Ideo}K]" => ideographic || k,
                            r"[\p{Ideo}\P{Ideographic}]" => true,
                            r"[^\P{Ideo}K]" => ideographic && !k,
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
    fn unicode_ideographic_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Ideo}",
            r"\P{Ideographic}",
            r"^[\p{Ideographic}]$",
            r"[\P{Ideo}K]",
            r"[^\P{Ideo}K]",
            r"[\p{Ideographic}]",
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
                            "😀\u{3005}\u{3006}\u{3007}\u{3008}\u{3020}\u{3021}\u{3029}\u{302a}\u{3037}\u{3038}\u{303a}\u{303b}\u{33ff}\u{3400}\u{4dbf}\u{4dc0}\u{4dff}\u{4e00}\u{9fff}\u{a000}\u{f8ff}\u{f900}\u{fa6d}\u{fa6e}\u{fa6f}\u{fa70}\u{fad9}\u{fada}\u{16fe3}\u{16fe4}\u{16fe5}\u{16ff1}\u{16ff2}\u{16ff6}\u{16ff7}\u{16fff}\u{17000}\u{18cdb}\u{18cfe}\u{18cff}\u{18d21}\u{18d7f}\u{18d80}\u{18df2}\u{18df3}\u{18dff}\u{19192}\u{1919f}\u{191d3}\u{1b16f}\u{1b170}\u{1b2fb}\u{1b2fc}\u{1ffff}\u{20000}\u{2a6df}\u{2a6e0}\u{2a6ff}\u{2a700}\u{2b81f}\u{2b820}\u{2cead}\u{2ceae}\u{2ceaf}\u{2ceb0}\u{2ebe0}\u{2ebe1}\u{2ebef}\u{2ebf0}\u{2ee5d}\u{2ee5e}\u{2f7ff}\u{2f800}\u{2fa1d}\u{2fa1e}\u{2ffff}\u{30000}\u{3134a}\u{3134b}\u{3134f}\u{31350}\u{33479}\u{3347a}\u{3cfff}\u{3fc40}\u{feff}ſK\n",
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
    fn unicode_ideographic_property_work_deduplication_and_contracts() {
        // New tags 48/49 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{Ideo}]"),
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
                &JsString::from(r"[^\P{ASCII}\p{Ideo}\P{Bidi_M}]"),
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

        let source =
            JsString::from(format!("[{}]", r"\p{Ideo}\P{Ideographic}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Ideo}$", r"^[\P{Ideographic}K]$"] {
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
            r"[\p{Ideo}--A]",
            r"[\p{Ideo}&&A]",
            r"[[\p{Ideo}]]",
            r"[A-\p{Ideo}]",
            r"[\p{Ideo}-A]",
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
                &JsString::from(r"[\p{Ideo}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_deprecated_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (329usize, 329usize),
            (1651usize, 1651usize),
            (3959usize, 3959usize),
            (3961usize, 3961usize),
            (6051usize, 6052usize),
            (8298usize, 8303usize),
            (9001usize, 9001usize),
            (9002usize, 9002usize),
            (917505usize, 917505usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Deprecated}",
                    r"\P{Dep}",
                    r"[\p{Dep}]",
                    r"[^\p{Deprecated}]",
                    r"[\P{Dep}K]",
                    r"[\p{Dep}K]",
                    r"[\p{Dep}\P{Deprecated}]",
                    r"[^\P{Dep}K]",
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
                        let deprecated = membership[point as usize];
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Deprecated}" | r"[\p{Dep}]" => deprecated,
                            r"\P{Dep}" | r"[^\p{Deprecated}]" => !deprecated,
                            r"[\P{Dep}K]" => !deprecated || k,
                            r"[\p{Dep}K]" => deprecated || k,
                            r"[\p{Dep}\P{Deprecated}]" => true,
                            r"[^\P{Dep}K]" => deprecated && !k,
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
    fn unicode_deprecated_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Dep}",
            r"\P{Deprecated}",
            r"^[\p{Deprecated}]$",
            r"[\P{Dep}K]",
            r"[^\P{Dep}K]",
            r"[\p{Deprecated}]",
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
                            "😀\u{148}\u{149}\u{14a}\u{672}\u{673}\u{674}\u{f76}\u{f77}\u{f78}\u{f79}\u{f7a}\u{17a2}\u{17a3}\u{17a4}\u{17a5}\u{17a3}\u{206a}\u{206f}\u{2070}\u{2328}\u{2329}\u{232a}\u{232b}\u{e0000}\u{e0001}\u{e0002}\u{feff}ſK\n",
                            "X\r\n\u{673}\u{2028}\u{17a3}\u{2029}",
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
    fn unicode_deprecated_property_work_deduplication_and_contracts() {
        // New tags 50/51 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{Dep}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x673, true),
                (0x17a3, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{Dep}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x673, false),
                (0x17a3, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source =
            JsString::from(format!("[{}]", r"\p{Dep}\P{Deprecated}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 1);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Dep}$", r"^[\P{Deprecated}K]$"] {
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
                plan.find(&JsString::from("\u{673}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Dep}--A]",
            r"[\p{Dep}&&A]",
            r"[[\p{Dep}]]",
            r"[A-\p{Dep}]",
            r"[\p{Dep}-A]",
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
                &JsString::from(r"[\p{Dep}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_diacritic_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (94usize, 94usize),
            (96usize, 96usize),
            (168usize, 168usize),
            (175usize, 175usize),
            (180usize, 180usize),
            (183usize, 183usize),
            (184usize, 184usize),
            (688usize, 705usize),
            (706usize, 709usize),
            (710usize, 721usize),
            (722usize, 735usize),
            (736usize, 740usize),
            (741usize, 747usize),
            (748usize, 748usize),
            (749usize, 749usize),
            (750usize, 750usize),
            (751usize, 767usize),
            (768usize, 846usize),
            (848usize, 855usize),
            (861usize, 866usize),
            (884usize, 884usize),
            (885usize, 885usize),
            (890usize, 890usize),
            (900usize, 901usize),
            (1155usize, 1159usize),
            (1368usize, 1369usize),
            (1419usize, 1420usize),
            (1425usize, 1469usize),
            (1471usize, 1471usize),
            (1473usize, 1474usize),
            (1476usize, 1477usize),
            (1479usize, 1481usize),
            (1611usize, 1618usize),
            (1622usize, 1624usize),
            (1759usize, 1760usize),
            (1762usize, 1762usize),
            (1765usize, 1766usize),
            (1770usize, 1772usize),
            (1840usize, 1866usize),
            (1958usize, 1968usize),
            (2027usize, 2035usize),
            (2036usize, 2037usize),
            (2072usize, 2073usize),
            (2200usize, 2207usize),
            (2249usize, 2249usize),
            (2250usize, 2259usize),
            (2275usize, 2302usize),
            (2364usize, 2364usize),
            (2381usize, 2381usize),
            (2385usize, 2388usize),
            (2417usize, 2417usize),
            (2492usize, 2492usize),
            (2509usize, 2509usize),
            (2620usize, 2620usize),
            (2637usize, 2637usize),
            (2748usize, 2748usize),
            (2765usize, 2765usize),
            (2813usize, 2815usize),
            (2876usize, 2876usize),
            (2893usize, 2893usize),
            (2899usize, 2901usize),
            (3021usize, 3021usize),
            (3132usize, 3132usize),
            (3149usize, 3149usize),
            (3260usize, 3260usize),
            (3277usize, 3277usize),
            (3387usize, 3388usize),
            (3405usize, 3405usize),
            (3530usize, 3530usize),
            (3642usize, 3642usize),
            (3655usize, 3660usize),
            (3662usize, 3662usize),
            (3770usize, 3770usize),
            (3784usize, 3788usize),
            (3864usize, 3865usize),
            (3893usize, 3893usize),
            (3895usize, 3895usize),
            (3897usize, 3897usize),
            (3902usize, 3903usize),
            (3970usize, 3972usize),
            (3974usize, 3975usize),
            (4038usize, 4038usize),
            (4151usize, 4151usize),
            (4153usize, 4154usize),
            (4195usize, 4196usize),
            (4201usize, 4205usize),
            (4231usize, 4236usize),
            (4237usize, 4237usize),
            (4239usize, 4239usize),
            (4250usize, 4251usize),
            (4957usize, 4959usize),
            (5908usize, 5908usize),
            (5909usize, 5909usize),
            (5940usize, 5940usize),
            (6089usize, 6099usize),
            (6109usize, 6109usize),
            (6457usize, 6459usize),
            (6752usize, 6752usize),
            (6773usize, 6780usize),
            (6783usize, 6783usize),
            (6832usize, 6845usize),
            (6846usize, 6846usize),
            (6849usize, 6859usize),
            (6863usize, 6896usize),
            (6964usize, 6964usize),
            (6980usize, 6980usize),
            (7019usize, 7027usize),
            (7082usize, 7082usize),
            (7083usize, 7083usize),
            (7142usize, 7142usize),
            (7154usize, 7155usize),
            (7222usize, 7223usize),
            (7288usize, 7293usize),
            (7376usize, 7378usize),
            (7379usize, 7379usize),
            (7380usize, 7392usize),
            (7393usize, 7393usize),
            (7394usize, 7400usize),
            (7405usize, 7405usize),
            (7412usize, 7412usize),
            (7415usize, 7415usize),
            (7416usize, 7417usize),
            (7468usize, 7530usize),
            (7579usize, 7614usize),
            (7620usize, 7631usize),
            (7669usize, 7679usize),
            (8125usize, 8125usize),
            (8127usize, 8129usize),
            (8141usize, 8143usize),
            (8157usize, 8159usize),
            (8173usize, 8175usize),
            (8189usize, 8190usize),
            (8335usize, 8335usize),
            (11503usize, 11505usize),
            (11823usize, 11823usize),
            (12330usize, 12333usize),
            (12334usize, 12335usize),
            (12441usize, 12442usize),
            (12443usize, 12444usize),
            (12540usize, 12540usize),
            (42607usize, 42607usize),
            (42620usize, 42621usize),
            (42623usize, 42623usize),
            (42652usize, 42653usize),
            (42736usize, 42737usize),
            (42752usize, 42774usize),
            (42775usize, 42783usize),
            (42784usize, 42785usize),
            (42888usize, 42888usize),
            (42889usize, 42890usize),
            (42993usize, 42993usize),
            (43000usize, 43001usize),
            (43014usize, 43014usize),
            (43052usize, 43052usize),
            (43204usize, 43204usize),
            (43232usize, 43249usize),
            (43307usize, 43309usize),
            (43310usize, 43310usize),
            (43347usize, 43347usize),
            (43443usize, 43443usize),
            (43456usize, 43456usize),
            (43493usize, 43493usize),
            (43643usize, 43643usize),
            (43644usize, 43644usize),
            (43645usize, 43645usize),
            (43711usize, 43711usize),
            (43712usize, 43712usize),
            (43713usize, 43713usize),
            (43714usize, 43714usize),
            (43766usize, 43766usize),
            (43867usize, 43867usize),
            (43868usize, 43871usize),
            (43881usize, 43881usize),
            (43882usize, 43883usize),
            (44012usize, 44012usize),
            (44013usize, 44013usize),
            (64286usize, 64286usize),
            (65056usize, 65071usize),
            (65342usize, 65342usize),
            (65344usize, 65344usize),
            (65392usize, 65392usize),
            (65438usize, 65439usize),
            (65507usize, 65507usize),
            (66272usize, 66272usize),
            (67456usize, 67461usize),
            (67463usize, 67504usize),
            (67506usize, 67514usize),
            (68152usize, 68154usize),
            (68159usize, 68159usize),
            (68325usize, 68326usize),
            (68898usize, 68899usize),
            (68900usize, 68903usize),
            (68942usize, 68942usize),
            (68969usize, 68973usize),
            (69323usize, 69327usize),
            (69360usize, 69367usize),
            (69369usize, 69370usize),
            (69373usize, 69375usize),
            (69446usize, 69456usize),
            (69506usize, 69509usize),
            (69702usize, 69702usize),
            (69744usize, 69744usize),
            (69817usize, 69818usize),
            (69939usize, 69940usize),
            (70003usize, 70003usize),
            (70080usize, 70080usize),
            (70090usize, 70092usize),
            (70197usize, 70197usize),
            (70198usize, 70198usize),
            (70377usize, 70378usize),
            (70459usize, 70460usize),
            (70477usize, 70477usize),
            (70502usize, 70508usize),
            (70512usize, 70516usize),
            (70606usize, 70606usize),
            (70607usize, 70607usize),
            (70608usize, 70608usize),
            (70610usize, 70610usize),
            (70611usize, 70611usize),
            (70625usize, 70626usize),
            (70722usize, 70722usize),
            (70726usize, 70726usize),
            (70850usize, 70851usize),
            (71103usize, 71104usize),
            (71231usize, 71231usize),
            (71350usize, 71350usize),
            (71351usize, 71351usize),
            (71467usize, 71467usize),
            (71737usize, 71738usize),
            (71997usize, 71997usize),
            (71998usize, 71998usize),
            (72003usize, 72003usize),
            (72160usize, 72160usize),
            (72244usize, 72244usize),
            (72263usize, 72263usize),
            (72345usize, 72345usize),
            (72767usize, 72767usize),
            (73026usize, 73026usize),
            (73028usize, 73029usize),
            (73111usize, 73111usize),
            (73177usize, 73177usize),
            (73537usize, 73537usize),
            (73538usize, 73538usize),
            (73562usize, 73562usize),
            (78919usize, 78933usize),
            (90415usize, 90415usize),
            (92912usize, 92916usize),
            (92976usize, 92982usize),
            (93547usize, 93548usize),
            (94095usize, 94098usize),
            (94099usize, 94111usize),
            (94192usize, 94193usize),
            (110576usize, 110579usize),
            (110581usize, 110587usize),
            (110589usize, 110590usize),
            (118528usize, 118573usize),
            (118576usize, 118598usize),
            (119143usize, 119145usize),
            (119149usize, 119154usize),
            (119163usize, 119170usize),
            (119173usize, 119179usize),
            (119210usize, 119213usize),
            (119376usize, 119378usize),
            (119387usize, 119388usize),
            (122829usize, 122831usize),
            (122833usize, 122879usize),
            (122928usize, 122989usize),
            (123184usize, 123190usize),
            (123566usize, 123566usize),
            (123628usize, 123631usize),
            (124398usize, 124399usize),
            (125136usize, 125142usize),
            (125252usize, 125254usize),
            (125256usize, 125258usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Diacritic}",
                    r"\P{Dia}",
                    r"[\p{Dia}]",
                    r"[^\p{Diacritic}]",
                    r"[\P{Dia}K]",
                    r"[\p{Dia}K]",
                    r"[\p{Dia}\P{Diacritic}]",
                    r"[^\P{Dia}K]",
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
                        let raw = membership[point as usize];
                        let greek = matches!(point, 0x345 | 0x399 | 0x3b9 | 0x1fbe);
                        let diacritic = raw || (ignore_case && greek);
                        let negative = if ignore_case && greek { !sets } else { !raw };
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Diacritic}" | r"[\p{Dia}]" => diacritic,
                            r"\P{Dia}" => negative,
                            r"[^\p{Diacritic}]" => !diacritic,
                            r"[\P{Dia}K]" => negative || k,
                            r"[\p{Dia}K]" => diacritic || k,
                            r"[\p{Dia}\P{Diacritic}]" => true,
                            r"[^\P{Dia}K]" => !negative && !k,
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
    fn unicode_diacritic_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Dia}",
            r"\P{Diacritic}",
            r"^[\p{Diacritic}]$",
            r"[\P{Dia}K]",
            r"[^\P{Dia}K]",
            r"[\p{Diacritic}]",
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
                            "😀\u{5d}\u{5e}\u{5f}\u{60}\u{61}\u{a7}\u{a8}\u{a9}\u{ae}\u{af}\u{b0}\u{b3}\u{b4}\u{b5}\u{b6}\u{b7}\u{b8}\u{b9}\u{2af}\u{2b0}\u{345}\u{34e}\u{34f}\u{350}\u{357}\u{358}\u{35c}\u{35d}\u{362}\u{363}\u{373}\u{374}\u{375}\u{376}\u{379}\u{37a}\u{37b}\u{383}\u{384}\u{385}\u{386}\u{399}\u{3b9}\u{482}\u{483}\u{487}\u{488}\u{557}\u{559}\u{55a}\u{58a}\u{58d}\u{590}\u{591}\u{5bd}\u{5be}\u{5bf}\u{5c0}\u{5c1}\u{5c2}\u{5c3}\u{5c4}\u{5c5}\u{5c6}\u{5c7}\u{5ca}\u{64a}\u{64b}\u{652}\u{653}\u{655}\u{658}\u{659}\u{6de}\u{6df}\u{6e0}\u{6e1}\u{6e3}\u{6e4}\u{6e5}\u{6e6}\u{6e7}\u{6e9}\u{6ea}\u{6ec}\u{6ed}\u{72f}\u{730}\u{74a}\u{74b}\u{7a5}\u{7a6}\u{7b0}\u{7b1}\u{7ea}\u{7eb}\u{7f5}\u{7f6}\u{817}\u{818}\u{819}\u{81a}\u{897}\u{898}\u{89f}\u{8a0}\u{8c8}\u{8c9}\u{8d4}\u{8e2}\u{8e3}\u{8fe}\u{8ff}\u{93b}\u{93c}\u{93d}\u{94c}\u{94d}\u{94e}\u{950}\u{951}\u{954}\u{955}\u{970}\u{971}\u{972}\u{9bb}\u{9bc}\u{9bd}\u{9cc}\u{9cd}\u{9ce}\u{a3b}\u{a3c}\u{a3d}\u{a4c}\u{a4d}\u{a4e}\u{abb}\u{abc}\u{abd}\u{acc}\u{acd}\u{ace}\u{afc}\u{afd}\u{aff}\u{b00}\u{b3b}\u{b3c}\u{b3d}\u{b4c}\u{b4d}\u{b4e}\u{b52}\u{b55}\u{b56}\u{bcc}\u{bcd}\u{bce}\u{c3b}\u{c3c}\u{c3d}\u{c4c}\u{c4d}\u{c4e}\u{cbb}\u{cbc}\u{cbd}\u{ccc}\u{ccd}\u{cce}\u{d3a}\u{d3b}\u{d3c}\u{d3d}\u{d4c}\u{d4d}\u{d4e}\u{dc9}\u{dca}\u{dcb}\u{e39}\u{e3a}\u{e3b}\u{e46}\u{e47}\u{e4c}\u{e4d}\u{e4e}\u{e4f}\u{eb9}\u{eba}\u{ebb}\u{ec7}\u{ec8}\u{ecc}\u{ecd}\u{f17}\u{f18}\u{f19}\u{f1a}\u{f34}\u{f35}\u{f36}\u{f37}\u{f38}\u{f39}\u{f3a}\u{f3d}\u{f3e}\u{f3f}\u{f40}\u{f81}\u{f82}\u{f84}\u{f85}\u{f86}\u{f87}\u{f88}\u{fc5}\u{fc6}\u{fc7}\u{1036}\u{1037}\u{1038}\u{1039}\u{103a}\u{103b}\u{1062}\u{1063}\u{1064}\u{1065}\u{1068}\u{1069}\u{106d}\u{106e}\u{1086}\u{1087}\u{108d}\u{108e}\u{108f}\u{1090}\u{1099}\u{109a}\u{109b}\u{109c}\u{135c}\u{135d}\u{135f}\u{1360}\u{1713}\u{1714}\u{1715}\u{1716}\u{1733}\u{1734}\u{1735}\u{17c8}\u{17c9}\u{17d3}\u{17d4}\u{17dc}\u{17dd}\u{17de}\u{1938}\u{1939}\u{193b}\u{193c}\u{1a5f}\u{1a60}\u{1a61}\u{1a74}\u{1a75}\u{1a7c}\u{1a7d}\u{1a7e}\u{1a7f}\u{1a80}\u{1aaf}\u{1ab0}\u{1abe}\u{1abf}\u{1ac0}\u{1ac1}\u{1acb}\u{1acc}\u{1ace}\u{1acf}\u{1af1}\u{1b33}\u{1b34}\u{1b35}\u{1b43}\u{1b44}\u{1b45}\u{1b6a}\u{1b6b}\u{1b73}\u{1b74}\u{1ba9}\u{1baa}\u{1bab}\u{1bac}\u{1be5}\u{1be6}\u{1be7}\u{1bf1}\u{1bf2}\u{1bf3}\u{1bf4}\u{1c35}\u{1c36}\u{1c37}\u{1c38}\u{1c77}\u{1c78}\u{1c7d}\u{1c7e}\u{1ccf}\u{1cd0}\u{1ce8}\u{1ce9}\u{1cec}\u{1ced}\u{1cee}\u{1cf3}\u{1cf4}\u{1cf5}\u{1cf6}\u{1cf7}\u{1cf9}\u{1cfa}\u{1d2b}\u{1d2c}\u{1d6a}\u{1d6b}\u{1d9a}\u{1d9b}\u{1dbe}\u{1dbf}\u{1dc3}\u{1dc4}\u{1dcf}\u{1dd0}\u{1df4}\u{1df5}\u{1dff}\u{1e00}\u{1fbc}\u{1fbd}\u{1fbe}\u{1fbf}\u{1fc1}\u{1fc2}\u{1fcc}\u{1fcd}\u{1fcf}\u{1fd0}\u{1fdc}\u{1fdd}\u{1fdf}\u{1fe0}\u{1fec}\u{1fed}\u{1fef}\u{1ff0}\u{1ffc}\u{1ffd}\u{1ffe}\u{1fff}\u{208e}\u{2090}\u{2cee}\u{2cef}\u{2cf1}\u{2cf2}\u{2e2e}\u{2e2f}\u{2e30}\u{3029}\u{302a}\u{302f}\u{3030}\u{3098}\u{3099}\u{309c}\u{309d}\u{30fb}\u{30fc}\u{30fd}\u{a66e}\u{a66f}\u{a670}\u{a67b}\u{a67c}\u{a67d}\u{a67e}\u{a67f}\u{a680}\u{a69b}\u{a69c}\u{a69d}\u{a69e}\u{a6ef}\u{a6f0}\u{a6f1}\u{a6f2}\u{a6ff}\u{a700}\u{a721}\u{a722}\u{a787}\u{a788}\u{a78a}\u{a78b}\u{a7f0}\u{a7f1}\u{a7f2}\u{a7f7}\u{a7f8}\u{a7f9}\u{a7fa}\u{a805}\u{a806}\u{a807}\u{a82b}\u{a82c}\u{a82d}\u{a8c3}\u{a8c4}\u{a8c5}\u{a8df}\u{a8e0}\u{a8f1}\u{a8f2}\u{a92a}\u{a92b}\u{a92e}\u{a92f}\u{a952}\u{a953}\u{a954}\u{a9b2}\u{a9b3}\u{a9b4}\u{a9bf}\u{a9c0}\u{a9c1}\u{a9e4}\u{a9e5}\u{a9e6}\u{aa7a}\u{aa7b}\u{aa7d}\u{aa7e}\u{aabe}\u{aabf}\u{aac2}\u{aac3}\u{aaf5}\u{aaf6}\u{aaf7}\u{ab5a}\u{ab5b}\u{ab5f}\u{ab60}\u{ab68}\u{ab69}\u{ab6b}\u{ab6c}\u{abeb}\u{abec}\u{abed}\u{abee}\u{fb1d}\u{fb1e}\u{fb1f}\u{fe1f}\u{fe20}\u{fe2f}\u{fe30}\u{ff3d}\u{ff3e}\u{ff3f}\u{ff40}\u{ff41}\u{ff6f}\u{ff70}\u{ff71}\u{ff9d}\u{ff9e}\u{ff9f}\u{ffa0}\u{ffe2}\u{ffe3}\u{ffe4}\u{102df}\u{102e0}\u{102e1}\u{1077f}\u{10780}\u{10785}\u{10786}\u{10787}\u{107b0}\u{107b1}\u{107b2}\u{107ba}\u{107bb}\u{10a37}\u{10a38}\u{10a3a}\u{10a3b}\u{10a3e}\u{10a3f}\u{10a40}\u{10ae4}\u{10ae5}\u{10ae6}\u{10ae7}\u{10d21}\u{10d22}\u{10d27}\u{10d28}\u{10d4d}\u{10d4e}\u{10d4f}\u{10d68}\u{10d69}\u{10d6d}\u{10d6e}\u{10eca}\u{10ed0}\u{10eef}\u{10ef8}\u{10efa}\u{10efb}\u{10efc}\u{10efd}\u{10eff}\u{10f00}\u{10f45}\u{10f46}\u{10f50}\u{10f51}\u{10f81}\u{10f82}\u{10f85}\u{10f86}\u{11045}\u{11046}\u{11047}\u{1106f}\u{11070}\u{11071}\u{110b8}\u{110b9}\u{110ba}\u{110bb}\u{11132}\u{11133}\u{11134}\u{11135}\u{11172}\u{11173}\u{11174}\u{111bf}\u{111c0}\u{111c1}\u{111c9}\u{111ca}\u{111cc}\u{111cd}\u{11234}\u{11235}\u{11236}\u{11237}\u{112e8}\u{112e9}\u{112ea}\u{112eb}\u{1133a}\u{1133b}\u{1133c}\u{1133d}\u{1134c}\u{1134d}\u{1134e}\u{11365}\u{11366}\u{1136c}\u{1136d}\u{1136f}\u{11370}\u{11374}\u{11375}\u{113cd}\u{113ce}\u{113d0}\u{113d1}\u{113d2}\u{113d3}\u{113d4}\u{113e0}\u{113e1}\u{113e2}\u{113e3}\u{11441}\u{11442}\u{11443}\u{11445}\u{11446}\u{11447}\u{114c1}\u{114c2}\u{114c3}\u{114c4}\u{115be}\u{115bf}\u{115c0}\u{115c1}\u{1163e}\u{1163f}\u{11640}\u{116b5}\u{116b6}\u{116b7}\u{116b8}\u{1172a}\u{1172b}\u{1172c}\u{11838}\u{11839}\u{1183a}\u{1183b}\u{1193c}\u{1193d}\u{1193e}\u{1193f}\u{11942}\u{11943}\u{11944}\u{119df}\u{119e0}\u{119e1}\u{11a33}\u{11a34}\u{11a35}\u{11a46}\u{11a47}\u{11a48}\u{11a98}\u{11a99}\u{11a9a}\u{11c3e}\u{11c3f}\u{11c40}\u{11d41}\u{11d42}\u{11d43}\u{11d44}\u{11d45}\u{11d46}\u{11d96}\u{11d97}\u{11d98}\u{11dd8}\u{11dd9}\u{11dda}\u{11f40}\u{11f41}\u{11f42}\u{11f43}\u{11f59}\u{11f5a}\u{11f5b}\u{13446}\u{13447}\u{13455}\u{13456}\u{1612e}\u{1612f}\u{16130}\u{16aef}\u{16af0}\u{16af4}\u{16af5}\u{16b2f}\u{16b30}\u{16b36}\u{16b37}\u{16d6a}\u{16d6b}\u{16d6c}\u{16d6d}\u{16f8e}\u{16f8f}\u{16f9f}\u{16fa0}\u{16fef}\u{16ff0}\u{16ff1}\u{16ff2}\u{1afef}\u{1aff0}\u{1aff3}\u{1aff4}\u{1aff5}\u{1affb}\u{1affc}\u{1affd}\u{1affe}\u{1afff}\u{1ceff}\u{1cf00}\u{1cf2d}\u{1cf2e}\u{1cf2f}\u{1cf30}\u{1cf46}\u{1cf47}\u{1d166}\u{1d167}\u{1d169}\u{1d16a}\u{1d16c}\u{1d16d}\u{1d172}\u{1d173}\u{1d17a}\u{1d17b}\u{1d182}\u{1d183}\u{1d184}\u{1d185}\u{1d18b}\u{1d18c}\u{1d1a9}\u{1d1aa}\u{1d1ad}\u{1d1ae}\u{1d24f}\u{1d253}\u{1d25a}\u{1d25d}\u{1dfcc}\u{1dfd0}\u{1e000}\u{1e02f}\u{1e030}\u{1e06d}\u{1e06e}\u{1e12f}\u{1e130}\u{1e136}\u{1e137}\u{1e2ad}\u{1e2ae}\u{1e2af}\u{1e2eb}\u{1e2ec}\u{1e2ef}\u{1e2f0}\u{1e5ed}\u{1e5ee}\u{1e5ef}\u{1e5f0}\u{1e8cf}\u{1e8d0}\u{1e8d6}\u{1e8d7}\u{1e943}\u{1e944}\u{1e946}\u{1e947}\u{1e948}\u{1e94a}\u{1e94b}\u{feff}ſK\n",
                            "X\r\n\u{300}\u{2028}\u{301}\u{2029}",
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
    fn unicode_diacritic_property_work_deduplication_and_contracts() {
        // New tags 52/53 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{Dia}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x300, true),
                (0x301, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{Dia}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x300, false),
                (0x301, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source =
            JsString::from(format!("[{}]", r"\p{Dia}\P{Diacritic}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 3);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Dia}$", r"^[\P{Diacritic}K]$"] {
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
                plan.find(&JsString::from("\u{300}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Dia}--A]",
            r"[\p{Dia}&&A]",
            r"[[\p{Dia}]]",
            r"[A-\p{Dia}]",
            r"[\p{Dia}-A]",
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
                &JsString::from(r"[\p{Dia}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_soft_dotted_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (105usize, 106usize),
            (303usize, 303usize),
            (585usize, 585usize),
            (616usize, 616usize),
            (669usize, 669usize),
            (690usize, 690usize),
            (1011usize, 1011usize),
            (1110usize, 1110usize),
            (1112usize, 1112usize),
            (7522usize, 7522usize),
            (7574usize, 7574usize),
            (7588usize, 7588usize),
            (7592usize, 7592usize),
            (7725usize, 7725usize),
            (7883usize, 7883usize),
            (8305usize, 8305usize),
            (8520usize, 8521usize),
            (11388usize, 11388usize),
            (119842usize, 119843usize),
            (119894usize, 119895usize),
            (119946usize, 119947usize),
            (119998usize, 119999usize),
            (120050usize, 120051usize),
            (120102usize, 120103usize),
            (120154usize, 120155usize),
            (120206usize, 120207usize),
            (120258usize, 120259usize),
            (120310usize, 120311usize),
            (120362usize, 120363usize),
            (120414usize, 120415usize),
            (120466usize, 120467usize),
            (122650usize, 122650usize),
            (122735usize, 122737usize),
            (122841usize, 122841usize),
            (122956usize, 122957usize),
            (122984usize, 122984usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Soft_Dotted}",
                    r"\P{SD}",
                    r"[\p{SD}]",
                    r"[^\p{Soft_Dotted}]",
                    r"[\P{SD}K]",
                    r"[\p{SD}K]",
                    r"[\p{SD}\P{Soft_Dotted}]",
                    r"[^\P{SD}K]",
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
                        let raw = membership[point as usize];
                        let greek = [
                            0x49u32, 0x4au32, 0x69u32, 0x6au32, 0x12eu32, 0x12fu32, 0x197u32,
                            0x248u32, 0x249u32, 0x268u32, 0x29du32, 0x37fu32, 0x3f3u32, 0x406u32,
                            0x408u32, 0x456u32, 0x458u32, 0x1e2cu32, 0x1e2du32, 0x1ecau32,
                            0x1ecbu32, 0xa7b2u32, 0x1df6eu32, 0x1df6fu32,
                        ]
                        .contains(&point);
                        let soft_dotted = raw || (ignore_case && greek);
                        let negative = if ignore_case && greek { !sets } else { !raw };
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Soft_Dotted}" | r"[\p{SD}]" => soft_dotted,
                            r"\P{SD}" => negative,
                            r"[^\p{Soft_Dotted}]" => !soft_dotted,
                            r"[\P{SD}K]" => negative || k,
                            r"[\p{SD}K]" => soft_dotted || k,
                            r"[\p{SD}\P{Soft_Dotted}]" => true,
                            r"[^\P{SD}K]" => !negative && !k,
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
    fn unicode_soft_dotted_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{SD}",
            r"\P{Soft_Dotted}",
            r"^[\p{Soft_Dotted}]$",
            r"[\P{SD}K]",
            r"[^\P{SD}K]",
            r"[\p{Soft_Dotted}]",
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
                            "😀\u{49}\u{4a}\u{68}\u{69}\u{6a}\u{6b}\u{12e}\u{12f}\u{130}\u{197}\u{248}\u{249}\u{24a}\u{267}\u{268}\u{269}\u{29c}\u{29d}\u{29e}\u{2b1}\u{2b2}\u{2b3}\u{37f}\u{3f2}\u{3f3}\u{3f4}\u{406}\u{408}\u{455}\u{456}\u{457}\u{458}\u{459}\u{1d61}\u{1d62}\u{1d63}\u{1d95}\u{1d96}\u{1d97}\u{1da3}\u{1da4}\u{1da5}\u{1da7}\u{1da8}\u{1da9}\u{1e2c}\u{1e2d}\u{1e2e}\u{1eca}\u{1ecb}\u{1ecc}\u{2070}\u{2071}\u{2072}\u{2147}\u{2148}\u{2149}\u{214a}\u{2c7b}\u{2c7c}\u{2c7d}\u{a7b2}\u{1d421}\u{1d422}\u{1d423}\u{1d424}\u{1d455}\u{1d456}\u{1d457}\u{1d458}\u{1d489}\u{1d48a}\u{1d48b}\u{1d48c}\u{1d4bd}\u{1d4be}\u{1d4bf}\u{1d4c0}\u{1d4f1}\u{1d4f2}\u{1d4f3}\u{1d4f4}\u{1d525}\u{1d526}\u{1d527}\u{1d528}\u{1d559}\u{1d55a}\u{1d55b}\u{1d55c}\u{1d58d}\u{1d58e}\u{1d58f}\u{1d590}\u{1d5c1}\u{1d5c2}\u{1d5c3}\u{1d5c4}\u{1d5f5}\u{1d5f6}\u{1d5f7}\u{1d5f8}\u{1d629}\u{1d62a}\u{1d62b}\u{1d62c}\u{1d65d}\u{1d65e}\u{1d65f}\u{1d660}\u{1d691}\u{1d692}\u{1d693}\u{1d694}\u{1df19}\u{1df1a}\u{1df1b}\u{1df72}\u{1dfd8}\u{1dfda}\u{1e04b}\u{1e04c}\u{1e04d}\u{1e04e}\u{1e067}\u{1e068}\u{1e069}\u{feff}ſK\n",
                            "X\r\n\u{69}\u{2028}\u{6a}\u{2029}",
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
    fn unicode_soft_dotted_property_work_deduplication_and_contracts() {
        // New tags 54/55 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{SD}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x69, true),
                (0x6a, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{SD}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x69, false),
                (0x6a, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source =
            JsString::from(format!("[{}]", r"\p{SD}\P{Soft_Dotted}".repeat(10000)).as_str());
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert!(!plan.ranges.is_empty());
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{SD}$", r"^[\P{Soft_Dotted}K]$"] {
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
                plan.find(&JsString::from("\u{69}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{SD}--A]",
            r"[\p{SD}&&A]",
            r"[[\p{SD}]]",
            r"[A-\p{SD}]",
            r"[\p{SD}-A]",
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
                &JsString::from(r"[\p{SD}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
    #[test]
    fn unicode_grapheme_extend_property_membership_over_every_code_point() {
        let mut membership = vec![false; 0x110000];
        for (a, b) in [
            (768usize, 879usize),
            (1155usize, 1159usize),
            (1160usize, 1161usize),
            (1425usize, 1469usize),
            (1471usize, 1471usize),
            (1473usize, 1474usize),
            (1476usize, 1477usize),
            (1479usize, 1481usize),
            (1552usize, 1562usize),
            (1611usize, 1631usize),
            (1648usize, 1648usize),
            (1750usize, 1756usize),
            (1759usize, 1764usize),
            (1767usize, 1768usize),
            (1770usize, 1773usize),
            (1809usize, 1809usize),
            (1840usize, 1866usize),
            (1958usize, 1968usize),
            (2027usize, 2035usize),
            (2045usize, 2045usize),
            (2070usize, 2073usize),
            (2075usize, 2083usize),
            (2085usize, 2087usize),
            (2089usize, 2093usize),
            (2137usize, 2139usize),
            (2199usize, 2207usize),
            (2250usize, 2273usize),
            (2275usize, 2306usize),
            (2362usize, 2362usize),
            (2364usize, 2364usize),
            (2369usize, 2376usize),
            (2381usize, 2381usize),
            (2385usize, 2391usize),
            (2402usize, 2403usize),
            (2433usize, 2433usize),
            (2492usize, 2492usize),
            (2494usize, 2494usize),
            (2497usize, 2500usize),
            (2509usize, 2509usize),
            (2519usize, 2519usize),
            (2530usize, 2531usize),
            (2558usize, 2558usize),
            (2561usize, 2562usize),
            (2620usize, 2620usize),
            (2625usize, 2626usize),
            (2631usize, 2632usize),
            (2635usize, 2637usize),
            (2641usize, 2641usize),
            (2672usize, 2673usize),
            (2677usize, 2677usize),
            (2689usize, 2690usize),
            (2748usize, 2748usize),
            (2753usize, 2757usize),
            (2759usize, 2760usize),
            (2765usize, 2765usize),
            (2786usize, 2787usize),
            (2810usize, 2815usize),
            (2817usize, 2817usize),
            (2876usize, 2876usize),
            (2878usize, 2878usize),
            (2879usize, 2879usize),
            (2881usize, 2884usize),
            (2893usize, 2893usize),
            (2899usize, 2902usize),
            (2903usize, 2903usize),
            (2914usize, 2915usize),
            (2946usize, 2946usize),
            (3006usize, 3006usize),
            (3008usize, 3008usize),
            (3021usize, 3021usize),
            (3031usize, 3031usize),
            (3072usize, 3072usize),
            (3076usize, 3076usize),
            (3132usize, 3132usize),
            (3134usize, 3136usize),
            (3142usize, 3144usize),
            (3146usize, 3149usize),
            (3157usize, 3158usize),
            (3170usize, 3171usize),
            (3201usize, 3201usize),
            (3260usize, 3260usize),
            (3263usize, 3263usize),
            (3264usize, 3264usize),
            (3266usize, 3266usize),
            (3270usize, 3270usize),
            (3271usize, 3272usize),
            (3274usize, 3275usize),
            (3276usize, 3277usize),
            (3285usize, 3286usize),
            (3298usize, 3299usize),
            (3328usize, 3329usize),
            (3387usize, 3388usize),
            (3390usize, 3390usize),
            (3393usize, 3396usize),
            (3405usize, 3405usize),
            (3415usize, 3415usize),
            (3426usize, 3427usize),
            (3457usize, 3457usize),
            (3530usize, 3530usize),
            (3535usize, 3535usize),
            (3538usize, 3540usize),
            (3542usize, 3542usize),
            (3551usize, 3551usize),
            (3633usize, 3633usize),
            (3636usize, 3642usize),
            (3655usize, 3662usize),
            (3761usize, 3761usize),
            (3764usize, 3772usize),
            (3784usize, 3790usize),
            (3864usize, 3865usize),
            (3893usize, 3893usize),
            (3895usize, 3895usize),
            (3897usize, 3897usize),
            (3953usize, 3966usize),
            (3968usize, 3972usize),
            (3974usize, 3975usize),
            (3981usize, 3991usize),
            (3993usize, 4028usize),
            (4038usize, 4038usize),
            (4141usize, 4144usize),
            (4146usize, 4151usize),
            (4153usize, 4154usize),
            (4157usize, 4158usize),
            (4184usize, 4185usize),
            (4190usize, 4192usize),
            (4209usize, 4212usize),
            (4226usize, 4226usize),
            (4229usize, 4230usize),
            (4237usize, 4237usize),
            (4253usize, 4253usize),
            (4957usize, 4959usize),
            (5906usize, 5908usize),
            (5909usize, 5909usize),
            (5938usize, 5939usize),
            (5940usize, 5940usize),
            (5970usize, 5971usize),
            (6002usize, 6003usize),
            (6068usize, 6069usize),
            (6071usize, 6077usize),
            (6086usize, 6086usize),
            (6089usize, 6099usize),
            (6109usize, 6109usize),
            (6155usize, 6157usize),
            (6159usize, 6159usize),
            (6277usize, 6278usize),
            (6313usize, 6313usize),
            (6432usize, 6434usize),
            (6439usize, 6440usize),
            (6450usize, 6450usize),
            (6457usize, 6459usize),
            (6679usize, 6680usize),
            (6683usize, 6683usize),
            (6742usize, 6742usize),
            (6744usize, 6750usize),
            (6752usize, 6752usize),
            (6754usize, 6754usize),
            (6757usize, 6764usize),
            (6771usize, 6780usize),
            (6783usize, 6783usize),
            (6832usize, 6845usize),
            (6846usize, 6846usize),
            (6847usize, 6896usize),
            (6912usize, 6915usize),
            (6964usize, 6964usize),
            (6965usize, 6965usize),
            (6966usize, 6970usize),
            (6971usize, 6971usize),
            (6972usize, 6972usize),
            (6973usize, 6973usize),
            (6978usize, 6978usize),
            (6979usize, 6980usize),
            (7019usize, 7027usize),
            (7040usize, 7041usize),
            (7074usize, 7077usize),
            (7080usize, 7081usize),
            (7082usize, 7082usize),
            (7083usize, 7085usize),
            (7142usize, 7142usize),
            (7144usize, 7145usize),
            (7149usize, 7149usize),
            (7151usize, 7153usize),
            (7154usize, 7155usize),
            (7212usize, 7219usize),
            (7222usize, 7223usize),
            (7376usize, 7378usize),
            (7380usize, 7392usize),
            (7394usize, 7400usize),
            (7405usize, 7405usize),
            (7412usize, 7412usize),
            (7416usize, 7417usize),
            (7616usize, 7679usize),
            (8204usize, 8204usize),
            (8400usize, 8412usize),
            (8413usize, 8416usize),
            (8417usize, 8417usize),
            (8418usize, 8420usize),
            (8421usize, 8432usize),
            (11503usize, 11505usize),
            (11647usize, 11647usize),
            (11744usize, 11775usize),
            (12330usize, 12333usize),
            (12334usize, 12335usize),
            (12441usize, 12442usize),
            (42607usize, 42607usize),
            (42608usize, 42610usize),
            (42612usize, 42621usize),
            (42654usize, 42655usize),
            (42736usize, 42737usize),
            (43010usize, 43010usize),
            (43014usize, 43014usize),
            (43019usize, 43019usize),
            (43045usize, 43046usize),
            (43052usize, 43052usize),
            (43204usize, 43205usize),
            (43232usize, 43249usize),
            (43263usize, 43263usize),
            (43302usize, 43309usize),
            (43335usize, 43345usize),
            (43347usize, 43347usize),
            (43392usize, 43394usize),
            (43443usize, 43443usize),
            (43446usize, 43449usize),
            (43452usize, 43453usize),
            (43456usize, 43456usize),
            (43493usize, 43493usize),
            (43561usize, 43566usize),
            (43569usize, 43570usize),
            (43573usize, 43574usize),
            (43587usize, 43587usize),
            (43596usize, 43596usize),
            (43644usize, 43644usize),
            (43696usize, 43696usize),
            (43698usize, 43700usize),
            (43703usize, 43704usize),
            (43710usize, 43711usize),
            (43713usize, 43713usize),
            (43756usize, 43757usize),
            (43766usize, 43766usize),
            (44005usize, 44005usize),
            (44008usize, 44008usize),
            (44013usize, 44013usize),
            (64286usize, 64286usize),
            (65024usize, 65039usize),
            (65056usize, 65071usize),
            (65438usize, 65439usize),
            (66045usize, 66045usize),
            (66272usize, 66272usize),
            (66422usize, 66426usize),
            (68097usize, 68099usize),
            (68101usize, 68102usize),
            (68108usize, 68111usize),
            (68152usize, 68154usize),
            (68159usize, 68159usize),
            (68325usize, 68326usize),
            (68900usize, 68903usize),
            (68969usize, 68973usize),
            (69291usize, 69292usize),
            (69323usize, 69327usize),
            (69360usize, 69375usize),
            (69446usize, 69456usize),
            (69506usize, 69509usize),
            (69633usize, 69633usize),
            (69688usize, 69702usize),
            (69744usize, 69744usize),
            (69747usize, 69748usize),
            (69759usize, 69761usize),
            (69811usize, 69814usize),
            (69817usize, 69818usize),
            (69826usize, 69826usize),
            (69888usize, 69890usize),
            (69927usize, 69931usize),
            (69933usize, 69940usize),
            (70003usize, 70003usize),
            (70016usize, 70017usize),
            (70070usize, 70078usize),
            (70080usize, 70080usize),
            (70089usize, 70092usize),
            (70095usize, 70095usize),
            (70191usize, 70193usize),
            (70196usize, 70196usize),
            (70197usize, 70197usize),
            (70198usize, 70199usize),
            (70206usize, 70206usize),
            (70209usize, 70209usize),
            (70367usize, 70367usize),
            (70371usize, 70378usize),
            (70400usize, 70401usize),
            (70459usize, 70460usize),
            (70462usize, 70462usize),
            (70464usize, 70464usize),
            (70477usize, 70477usize),
            (70487usize, 70487usize),
            (70502usize, 70508usize),
            (70512usize, 70516usize),
            (70584usize, 70584usize),
            (70587usize, 70592usize),
            (70594usize, 70594usize),
            (70597usize, 70597usize),
            (70599usize, 70601usize),
            (70606usize, 70606usize),
            (70607usize, 70607usize),
            (70608usize, 70608usize),
            (70610usize, 70610usize),
            (70625usize, 70626usize),
            (70712usize, 70719usize),
            (70722usize, 70724usize),
            (70726usize, 70726usize),
            (70750usize, 70750usize),
            (70832usize, 70832usize),
            (70835usize, 70840usize),
            (70842usize, 70842usize),
            (70845usize, 70845usize),
            (70847usize, 70848usize),
            (70850usize, 70851usize),
            (71087usize, 71087usize),
            (71090usize, 71093usize),
            (71100usize, 71101usize),
            (71103usize, 71104usize),
            (71132usize, 71133usize),
            (71219usize, 71226usize),
            (71229usize, 71229usize),
            (71231usize, 71232usize),
            (71339usize, 71339usize),
            (71341usize, 71341usize),
            (71344usize, 71349usize),
            (71350usize, 71350usize),
            (71351usize, 71351usize),
            (71453usize, 71453usize),
            (71455usize, 71455usize),
            (71458usize, 71461usize),
            (71463usize, 71467usize),
            (71727usize, 71735usize),
            (71737usize, 71738usize),
            (71984usize, 71984usize),
            (71995usize, 71996usize),
            (71997usize, 71997usize),
            (71998usize, 71998usize),
            (72003usize, 72003usize),
            (72148usize, 72151usize),
            (72154usize, 72155usize),
            (72160usize, 72160usize),
            (72193usize, 72202usize),
            (72243usize, 72248usize),
            (72251usize, 72254usize),
            (72263usize, 72263usize),
            (72273usize, 72278usize),
            (72281usize, 72283usize),
            (72330usize, 72342usize),
            (72344usize, 72345usize),
            (72544usize, 72544usize),
            (72546usize, 72548usize),
            (72550usize, 72550usize),
            (72752usize, 72758usize),
            (72760usize, 72765usize),
            (72767usize, 72767usize),
            (72850usize, 72871usize),
            (72874usize, 72880usize),
            (72882usize, 72883usize),
            (72885usize, 72886usize),
            (73009usize, 73014usize),
            (73018usize, 73018usize),
            (73020usize, 73021usize),
            (73023usize, 73029usize),
            (73031usize, 73031usize),
            (73104usize, 73105usize),
            (73109usize, 73109usize),
            (73111usize, 73111usize),
            (73200usize, 73200usize),
            (73459usize, 73460usize),
            (73472usize, 73473usize),
            (73526usize, 73530usize),
            (73536usize, 73536usize),
            (73537usize, 73537usize),
            (73538usize, 73538usize),
            (73562usize, 73562usize),
            (78912usize, 78912usize),
            (78919usize, 78933usize),
            (90398usize, 90409usize),
            (90413usize, 90415usize),
            (92912usize, 92916usize),
            (92976usize, 92982usize),
            (94031usize, 94031usize),
            (94095usize, 94098usize),
            (94180usize, 94180usize),
            (94192usize, 94193usize),
            (113821usize, 113822usize),
            (118528usize, 118573usize),
            (118576usize, 118598usize),
            (119079usize, 119080usize),
            (119141usize, 119142usize),
            (119143usize, 119145usize),
            (119149usize, 119154usize),
            (119163usize, 119170usize),
            (119173usize, 119179usize),
            (119210usize, 119213usize),
            (119362usize, 119364usize),
            (119376usize, 119378usize),
            (119387usize, 119388usize),
            (119391usize, 119391usize),
            (119424usize, 119425usize),
            (121344usize, 121398usize),
            (121403usize, 121452usize),
            (121461usize, 121461usize),
            (121476usize, 121476usize),
            (121499usize, 121503usize),
            (121505usize, 121519usize),
            (122880usize, 122886usize),
            (122888usize, 122904usize),
            (122907usize, 122913usize),
            (122915usize, 122916usize),
            (122918usize, 122922usize),
            (123023usize, 123023usize),
            (123184usize, 123190usize),
            (123566usize, 123566usize),
            (123628usize, 123631usize),
            (124140usize, 124143usize),
            (124398usize, 124399usize),
            (124643usize, 124643usize),
            (124646usize, 124646usize),
            (124654usize, 124655usize),
            (124661usize, 124661usize),
            (125136usize, 125142usize),
            (125252usize, 125258usize),
            (917536usize, 917631usize),
            (917760usize, 917999usize),
        ] {
            membership[a..=b].fill(true);
        }
        for sets in [false, true] {
            for ignore_case in [false, true] {
                for source in [
                    r"\p{Grapheme_Extend}",
                    r"\P{Gr_Ext}",
                    r"[\p{Gr_Ext}]",
                    r"[^\p{Grapheme_Extend}]",
                    r"[\P{Gr_Ext}K]",
                    r"[\p{Gr_Ext}K]",
                    r"[\p{Gr_Ext}\P{Grapheme_Extend}]",
                    r"[^\P{Gr_Ext}K]",
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
                        let raw = membership[point as usize];
                        let greek = matches!(point, 0x345 | 0x399 | 0x3b9 | 0x1fbe);
                        let grapheme_extend = raw || (ignore_case && greek);
                        let negative = if ignore_case && greek { !sets } else { !raw };
                        let k = point == 0x4b || (ignore_case && matches!(point, 0x6b | 0x212a));
                        let expected = match source {
                            r"\p{Grapheme_Extend}" | r"[\p{Gr_Ext}]" => grapheme_extend,
                            r"\P{Gr_Ext}" => negative,
                            r"[^\p{Grapheme_Extend}]" => !grapheme_extend,
                            r"[\P{Gr_Ext}K]" => negative || k,
                            r"[\p{Gr_Ext}K]" => grapheme_extend || k,
                            r"[\p{Gr_Ext}\P{Grapheme_Extend}]" => true,
                            r"[^\P{Gr_Ext}K]" => !negative && !k,
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
    fn unicode_grapheme_extend_property_ranges_snapshot() {
        use std::fmt::Write;
        let mut rows = String::new();
        for source in [
            r"\p{Gr_Ext}",
            r"\P{Grapheme_Extend}",
            r"^[\p{Grapheme_Extend}]$",
            r"[\P{Gr_Ext}K]",
            r"[^\P{Gr_Ext}K]",
            r"[\p{Grapheme_Extend}]",
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
                            "😀\u{2ff}\u{300}\u{345}\u{36f}\u{370}\u{399}\u{3b9}\u{482}\u{483}\u{489}\u{48a}\u{590}\u{591}\u{5bd}\u{5be}\u{5bf}\u{5c0}\u{5c1}\u{5c2}\u{5c3}\u{5c4}\u{5c5}\u{5c6}\u{5c7}\u{5ca}\u{60f}\u{610}\u{61a}\u{61b}\u{64a}\u{64b}\u{65f}\u{660}\u{66f}\u{670}\u{671}\u{6d5}\u{6d6}\u{6dc}\u{6dd}\u{6de}\u{6df}\u{6e4}\u{6e5}\u{6e6}\u{6e7}\u{6e8}\u{6e9}\u{6ea}\u{6ed}\u{6ee}\u{710}\u{711}\u{712}\u{72f}\u{730}\u{74a}\u{74b}\u{7a5}\u{7a6}\u{7b0}\u{7b1}\u{7ea}\u{7eb}\u{7f3}\u{7f4}\u{7fc}\u{7fd}\u{7fe}\u{815}\u{816}\u{819}\u{81a}\u{81b}\u{823}\u{824}\u{825}\u{827}\u{828}\u{829}\u{82d}\u{82e}\u{858}\u{859}\u{85b}\u{85c}\u{896}\u{897}\u{89f}\u{8a0}\u{8c9}\u{8ca}\u{8e1}\u{8e2}\u{8e3}\u{902}\u{903}\u{939}\u{93a}\u{93b}\u{93c}\u{93d}\u{940}\u{941}\u{948}\u{949}\u{94c}\u{94d}\u{94e}\u{950}\u{951}\u{957}\u{958}\u{961}\u{962}\u{963}\u{964}\u{980}\u{981}\u{982}\u{9bb}\u{9bc}\u{9bd}\u{9be}\u{9bf}\u{9c0}\u{9c1}\u{9c4}\u{9c5}\u{9cc}\u{9cd}\u{9ce}\u{9d6}\u{9d7}\u{9d8}\u{9e1}\u{9e2}\u{9e3}\u{9e4}\u{9fd}\u{9fe}\u{9ff}\u{a00}\u{a01}\u{a02}\u{a03}\u{a3b}\u{a3c}\u{a3d}\u{a40}\u{a41}\u{a42}\u{a43}\u{a46}\u{a47}\u{a48}\u{a49}\u{a4a}\u{a4b}\u{a4d}\u{a4e}\u{a50}\u{a51}\u{a52}\u{a6f}\u{a70}\u{a71}\u{a72}\u{a74}\u{a75}\u{a76}\u{a80}\u{a81}\u{a82}\u{a83}\u{abb}\u{abc}\u{abd}\u{ac0}\u{ac1}\u{ac5}\u{ac6}\u{ac7}\u{ac8}\u{ac9}\u{acc}\u{acd}\u{ace}\u{ae1}\u{ae2}\u{ae3}\u{ae4}\u{af9}\u{afa}\u{aff}\u{b00}\u{b01}\u{b02}\u{b3b}\u{b3c}\u{b3d}\u{b3e}\u{b3f}\u{b40}\u{b41}\u{b44}\u{b45}\u{b4c}\u{b4d}\u{b4e}\u{b52}\u{b57}\u{b58}\u{b61}\u{b62}\u{b63}\u{b64}\u{b81}\u{b82}\u{b83}\u{bbd}\u{bbe}\u{bbf}\u{bc0}\u{bc1}\u{bcc}\u{bcd}\u{bce}\u{bd6}\u{bd7}\u{bd8}\u{bff}\u{c00}\u{c01}\u{c03}\u{c04}\u{c05}\u{c3b}\u{c3c}\u{c3d}\u{c3e}\u{c40}\u{c41}\u{c45}\u{c46}\u{c48}\u{c49}\u{c4a}\u{c4d}\u{c4e}\u{c54}\u{c55}\u{c56}\u{c57}\u{c61}\u{c62}\u{c63}\u{c64}\u{c80}\u{c81}\u{c82}\u{cbb}\u{cbc}\u{cbd}\u{cbe}\u{cbf}\u{cc0}\u{cc1}\u{cc2}\u{cc3}\u{cc5}\u{cc6}\u{cc8}\u{cc9}\u{cca}\u{ccd}\u{cce}\u{cd4}\u{cd5}\u{cd6}\u{cd7}\u{ce1}\u{ce2}\u{ce3}\u{ce4}\u{cff}\u{d00}\u{d01}\u{d02}\u{d3a}\u{d3b}\u{d3c}\u{d3d}\u{d3e}\u{d3f}\u{d40}\u{d41}\u{d44}\u{d45}\u{d4c}\u{d4d}\u{d4e}\u{d56}\u{d57}\u{d58}\u{d61}\u{d62}\u{d63}\u{d64}\u{d80}\u{d81}\u{d82}\u{dc9}\u{dca}\u{dcb}\u{dce}\u{dcf}\u{dd0}\u{dd1}\u{dd2}\u{dd4}\u{dd5}\u{dd6}\u{dd7}\u{dde}\u{ddf}\u{de0}\u{e30}\u{e31}\u{e32}\u{e33}\u{e34}\u{e3a}\u{e3b}\u{e46}\u{e47}\u{e4e}\u{e4f}\u{eb0}\u{eb1}\u{eb2}\u{eb3}\u{eb4}\u{ebc}\u{ebd}\u{ec7}\u{ec8}\u{ece}\u{ecf}\u{f17}\u{f18}\u{f19}\u{f1a}\u{f34}\u{f35}\u{f36}\u{f37}\u{f38}\u{f39}\u{f3a}\u{f70}\u{f71}\u{f7e}\u{f7f}\u{f80}\u{f84}\u{f85}\u{f86}\u{f87}\u{f88}\u{f8c}\u{f8d}\u{f97}\u{f98}\u{f99}\u{fbc}\u{fbd}\u{fc5}\u{fc6}\u{fc7}\u{102c}\u{102d}\u{1030}\u{1031}\u{1032}\u{1037}\u{1038}\u{1039}\u{103a}\u{103b}\u{103c}\u{103d}\u{103e}\u{103f}\u{1057}\u{1058}\u{1059}\u{105a}\u{105d}\u{105e}\u{1060}\u{1061}\u{1070}\u{1071}\u{1074}\u{1075}\u{1081}\u{1082}\u{1083}\u{1084}\u{1085}\u{1086}\u{1087}\u{108c}\u{108d}\u{108e}\u{109c}\u{109d}\u{109e}\u{135c}\u{135d}\u{135f}\u{1360}\u{1711}\u{1712}\u{1715}\u{1716}\u{1731}\u{1732}\u{1734}\u{1735}\u{1751}\u{1752}\u{1753}\u{1754}\u{1771}\u{1772}\u{1773}\u{1774}\u{17b3}\u{17b4}\u{17b5}\u{17b6}\u{17b7}\u{17bd}\u{17be}\u{17c5}\u{17c6}\u{17c7}\u{17c8}\u{17c9}\u{17d3}\u{17d4}\u{17dc}\u{17dd}\u{17de}\u{180a}\u{180b}\u{180d}\u{180e}\u{180f}\u{1810}\u{1884}\u{1885}\u{1886}\u{1887}\u{18a8}\u{18a9}\u{18aa}\u{191f}\u{1920}\u{1922}\u{1923}\u{1926}\u{1927}\u{1928}\u{1929}\u{1931}\u{1932}\u{1933}\u{1938}\u{1939}\u{193b}\u{193c}\u{1a16}\u{1a17}\u{1a18}\u{1a19}\u{1a1a}\u{1a1b}\u{1a1c}\u{1a55}\u{1a56}\u{1a57}\u{1a58}\u{1a5e}\u{1a5f}\u{1a60}\u{1a61}\u{1a62}\u{1a63}\u{1a64}\u{1a65}\u{1a6c}\u{1a6d}\u{1a72}\u{1a73}\u{1a7c}\u{1a7d}\u{1a7e}\u{1a7f}\u{1a80}\u{1aaf}\u{1ab0}\u{1af1}\u{1aff}\u{1b00}\u{1b03}\u{1b04}\u{1b33}\u{1b34}\u{1b3d}\u{1b3e}\u{1b41}\u{1b42}\u{1b44}\u{1b45}\u{1b6a}\u{1b6b}\u{1b73}\u{1b74}\u{1b7f}\u{1b80}\u{1b81}\u{1b82}\u{1ba1}\u{1ba2}\u{1ba5}\u{1ba6}\u{1ba7}\u{1ba8}\u{1bad}\u{1bae}\u{1be5}\u{1be6}\u{1be7}\u{1be8}\u{1be9}\u{1bea}\u{1bec}\u{1bed}\u{1bee}\u{1bef}\u{1bf3}\u{1bf4}\u{1c2b}\u{1c2c}\u{1c33}\u{1c34}\u{1c35}\u{1c36}\u{1c37}\u{1c38}\u{1ccf}\u{1cd0}\u{1cd2}\u{1cd3}\u{1cd4}\u{1ce0}\u{1ce1}\u{1ce2}\u{1ce8}\u{1ce9}\u{1cec}\u{1ced}\u{1cee}\u{1cf3}\u{1cf4}\u{1cf5}\u{1cf7}\u{1cf8}\u{1cf9}\u{1cfa}\u{1dbf}\u{1dc0}\u{1dff}\u{1e00}\u{1fbe}\u{200b}\u{200c}\u{200d}\u{20cf}\u{20d0}\u{20f0}\u{20f1}\u{2cee}\u{2cef}\u{2cf1}\u{2cf2}\u{2d7e}\u{2d7f}\u{2d80}\u{2ddf}\u{2de0}\u{2dff}\u{2e00}\u{3029}\u{302a}\u{302f}\u{3030}\u{3098}\u{3099}\u{309a}\u{309b}\u{a66e}\u{a66f}\u{a672}\u{a673}\u{a674}\u{a67d}\u{a67e}\u{a69d}\u{a69e}\u{a69f}\u{a6a0}\u{a6ef}\u{a6f0}\u{a6f1}\u{a6f2}\u{a801}\u{a802}\u{a803}\u{a805}\u{a806}\u{a807}\u{a80a}\u{a80b}\u{a80c}\u{a824}\u{a825}\u{a826}\u{a827}\u{a82b}\u{a82c}\u{a82d}\u{a8c3}\u{a8c4}\u{a8c5}\u{a8c6}\u{a8df}\u{a8e0}\u{a8f1}\u{a8f2}\u{a8fe}\u{a8ff}\u{a900}\u{a925}\u{a926}\u{a92d}\u{a92e}\u{a946}\u{a947}\u{a951}\u{a952}\u{a953}\u{a954}\u{a97f}\u{a980}\u{a982}\u{a983}\u{a9b2}\u{a9b3}\u{a9b4}\u{a9b5}\u{a9b6}\u{a9b9}\u{a9ba}\u{a9bb}\u{a9bc}\u{a9bd}\u{a9be}\u{a9bf}\u{a9c0}\u{a9c1}\u{a9e4}\u{a9e5}\u{a9e6}\u{aa28}\u{aa29}\u{aa2e}\u{aa2f}\u{aa30}\u{aa31}\u{aa32}\u{aa33}\u{aa34}\u{aa35}\u{aa36}\u{aa37}\u{aa42}\u{aa43}\u{aa44}\u{aa4b}\u{aa4c}\u{aa4d}\u{aa7b}\u{aa7c}\u{aa7d}\u{aaaf}\u{aab0}\u{aab1}\u{aab2}\u{aab4}\u{aab5}\u{aab6}\u{aab7}\u{aab8}\u{aab9}\u{aabd}\u{aabe}\u{aabf}\u{aac0}\u{aac1}\u{aac2}\u{aaeb}\u{aaec}\u{aaed}\u{aaee}\u{aaf5}\u{aaf6}\u{aaf7}\u{abe4}\u{abe5}\u{abe6}\u{abe7}\u{abe8}\u{abe9}\u{abec}\u{abed}\u{abee}\u{fb1d}\u{fb1e}\u{fb1f}\u{fdff}\u{fe00}\u{fe0f}\u{fe10}\u{fe1f}\u{fe20}\u{fe2f}\u{fe30}\u{ff9d}\u{ff9e}\u{ff9f}\u{ffa0}\u{101fc}\u{101fd}\u{101fe}\u{102df}\u{102e0}\u{102e1}\u{10375}\u{10376}\u{1037a}\u{1037b}\u{10a00}\u{10a01}\u{10a03}\u{10a04}\u{10a05}\u{10a06}\u{10a07}\u{10a0b}\u{10a0c}\u{10a0f}\u{10a10}\u{10a37}\u{10a38}\u{10a3a}\u{10a3b}\u{10a3e}\u{10a3f}\u{10a40}\u{10ae4}\u{10ae5}\u{10ae6}\u{10ae7}\u{10d23}\u{10d24}\u{10d27}\u{10d28}\u{10d68}\u{10d69}\u{10d6d}\u{10d6e}\u{10eaa}\u{10eab}\u{10eac}\u{10ead}\u{10eca}\u{10ed0}\u{10eef}\u{10eff}\u{10f00}\u{10f45}\u{10f46}\u{10f50}\u{10f51}\u{10f81}\u{10f82}\u{10f85}\u{10f86}\u{11000}\u{11001}\u{11002}\u{11037}\u{11038}\u{11046}\u{11047}\u{1106f}\u{11070}\u{11071}\u{11072}\u{11073}\u{11074}\u{11075}\u{1107e}\u{1107f}\u{11081}\u{11082}\u{110b2}\u{110b3}\u{110b6}\u{110b7}\u{110b8}\u{110b9}\u{110ba}\u{110bb}\u{110c1}\u{110c2}\u{110c3}\u{110ff}\u{11100}\u{11102}\u{11103}\u{11126}\u{11127}\u{1112b}\u{1112c}\u{1112d}\u{11134}\u{11135}\u{11172}\u{11173}\u{11174}\u{1117f}\u{11180}\u{11181}\u{11182}\u{111b5}\u{111b6}\u{111be}\u{111bf}\u{111c0}\u{111c1}\u{111c8}\u{111c9}\u{111cc}\u{111cd}\u{111ce}\u{111cf}\u{111d0}\u{1122e}\u{1122f}\u{11231}\u{11232}\u{11233}\u{11234}\u{11237}\u{11238}\u{1123d}\u{1123e}\u{1123f}\u{11240}\u{11241}\u{11242}\u{112de}\u{112df}\u{112e0}\u{112e2}\u{112e3}\u{112ea}\u{112eb}\u{112ff}\u{11300}\u{11301}\u{11302}\u{1133a}\u{1133b}\u{1133c}\u{1133d}\u{1133e}\u{1133f}\u{11340}\u{11341}\u{1134c}\u{1134d}\u{1134e}\u{11356}\u{11357}\u{11358}\u{11365}\u{11366}\u{1136c}\u{1136d}\u{1136f}\u{11370}\u{11374}\u{11375}\u{113b7}\u{113b8}\u{113b9}\u{113ba}\u{113bb}\u{113c0}\u{113c1}\u{113c2}\u{113c3}\u{113c4}\u{113c5}\u{113c6}\u{113c7}\u{113c9}\u{113ca}\u{113cd}\u{113ce}\u{113d0}\u{113d1}\u{113d2}\u{113d3}\u{113e0}\u{113e1}\u{113e2}\u{113e3}\u{11437}\u{11438}\u{1143f}\u{11440}\u{11441}\u{11442}\u{11444}\u{11445}\u{11446}\u{11447}\u{1145d}\u{1145e}\u{1145f}\u{114af}\u{114b0}\u{114b1}\u{114b2}\u{114b3}\u{114b8}\u{114b9}\u{114ba}\u{114bb}\u{114bc}\u{114bd}\u{114be}\u{114bf}\u{114c0}\u{114c1}\u{114c2}\u{114c3}\u{114c4}\u{115ae}\u{115af}\u{115b0}\u{115b1}\u{115b2}\u{115b5}\u{115b6}\u{115bb}\u{115bc}\u{115bd}\u{115be}\u{115bf}\u{115c0}\u{115c1}\u{115db}\u{115dc}\u{115dd}\u{115de}\u{11632}\u{11633}\u{1163a}\u{1163b}\u{1163c}\u{1163d}\u{1163e}\u{1163f}\u{11640}\u{11641}\u{116aa}\u{116ab}\u{116ac}\u{116ad}\u{116ae}\u{116af}\u{116b0}\u{116b7}\u{116b8}\u{1171c}\u{1171d}\u{1171e}\u{1171f}\u{11720}\u{11721}\u{11722}\u{11725}\u{11726}\u{11727}\u{1172b}\u{1172c}\u{1182e}\u{1182f}\u{11837}\u{11838}\u{11839}\u{1183a}\u{1183b}\u{1192f}\u{11930}\u{11931}\u{1193a}\u{1193b}\u{1193e}\u{1193f}\u{11942}\u{11943}\u{11944}\u{119d3}\u{119d4}\u{119d7}\u{119d8}\u{119d9}\u{119da}\u{119db}\u{119dc}\u{119df}\u{119e0}\u{119e1}\u{11a00}\u{11a01}\u{11a0a}\u{11a0b}\u{11a32}\u{11a33}\u{11a38}\u{11a39}\u{11a3a}\u{11a3b}\u{11a3e}\u{11a3f}\u{11a46}\u{11a47}\u{11a48}\u{11a50}\u{11a51}\u{11a56}\u{11a57}\u{11a58}\u{11a59}\u{11a5b}\u{11a5c}\u{11a89}\u{11a8a}\u{11a96}\u{11a97}\u{11a98}\u{11a99}\u{11a9a}\u{11b5f}\u{11b60}\u{11b61}\u{11b62}\u{11b64}\u{11b65}\u{11b66}\u{11b67}\u{11c2f}\u{11c30}\u{11c36}\u{11c37}\u{11c38}\u{11c3d}\u{11c3e}\u{11c3f}\u{11c40}\u{11c91}\u{11c92}\u{11ca7}\u{11ca8}\u{11ca9}\u{11caa}\u{11cb0}\u{11cb1}\u{11cb2}\u{11cb3}\u{11cb4}\u{11cb5}\u{11cb6}\u{11cb7}\u{11d30}\u{11d31}\u{11d36}\u{11d37}\u{11d39}\u{11d3a}\u{11d3b}\u{11d3c}\u{11d3d}\u{11d3e}\u{11d3f}\u{11d45}\u{11d46}\u{11d47}\u{11d48}\u{11d8f}\u{11d90}\u{11d91}\u{11d92}\u{11d94}\u{11d95}\u{11d96}\u{11d97}\u{11d98}\u{11def}\u{11df1}\u{11ef2}\u{11ef3}\u{11ef4}\u{11ef5}\u{11eff}\u{11f00}\u{11f01}\u{11f02}\u{11f35}\u{11f36}\u{11f3a}\u{11f3b}\u{11f3f}\u{11f40}\u{11f42}\u{11f43}\u{11f59}\u{11f5a}\u{11f5b}\u{1343f}\u{13440}\u{13441}\u{13446}\u{13447}\u{13455}\u{13456}\u{1611d}\u{1611e}\u{16129}\u{1612a}\u{1612c}\u{1612d}\u{1612f}\u{16130}\u{16aef}\u{16af0}\u{16af4}\u{16af5}\u{16b2f}\u{16b30}\u{16b36}\u{16b37}\u{16f4e}\u{16f4f}\u{16f50}\u{16f8e}\u{16f8f}\u{16f92}\u{16f93}\u{16fe3}\u{16fe4}\u{16fe5}\u{16fef}\u{16ff0}\u{16ff1}\u{16ff2}\u{1bc9c}\u{1bc9d}\u{1bc9e}\u{1bc9f}\u{1ceff}\u{1cf00}\u{1cf2d}\u{1cf2e}\u{1cf2f}\u{1cf30}\u{1cf46}\u{1cf47}\u{1d126}\u{1d129}\u{1d164}\u{1d165}\u{1d169}\u{1d16a}\u{1d16c}\u{1d16d}\u{1d172}\u{1d173}\u{1d17a}\u{1d17b}\u{1d182}\u{1d183}\u{1d184}\u{1d185}\u{1d18b}\u{1d18c}\u{1d1a9}\u{1d1aa}\u{1d1ad}\u{1d1ae}\u{1d241}\u{1d242}\u{1d244}\u{1d245}\u{1d24f}\u{1d253}\u{1d25a}\u{1d25d}\u{1d25e}\u{1d260}\u{1d27f}\u{1d282}\u{1d9ff}\u{1da00}\u{1da36}\u{1da37}\u{1da3a}\u{1da3b}\u{1da6c}\u{1da6d}\u{1da74}\u{1da75}\u{1da76}\u{1da83}\u{1da84}\u{1da85}\u{1da9a}\u{1da9b}\u{1da9f}\u{1daa0}\u{1daa1}\u{1daaf}\u{1dab0}\u{1dfff}\u{1e000}\u{1e006}\u{1e007}\u{1e008}\u{1e018}\u{1e019}\u{1e01a}\u{1e01b}\u{1e021}\u{1e022}\u{1e023}\u{1e024}\u{1e025}\u{1e026}\u{1e02a}\u{1e02b}\u{1e08e}\u{1e08f}\u{1e090}\u{1e12f}\u{1e130}\u{1e136}\u{1e137}\u{1e2ad}\u{1e2ae}\u{1e2af}\u{1e2eb}\u{1e2ec}\u{1e2ef}\u{1e2f0}\u{1e4eb}\u{1e4ec}\u{1e4ef}\u{1e4f0}\u{1e5ed}\u{1e5ee}\u{1e5ef}\u{1e5f0}\u{1e6e2}\u{1e6e3}\u{1e6e4}\u{1e6e5}\u{1e6e6}\u{1e6e7}\u{1e6ed}\u{1e6ee}\u{1e6ef}\u{1e6f0}\u{1e6f4}\u{1e6f5}\u{1e6f6}\u{1e8cf}\u{1e8d0}\u{1e8d6}\u{1e8d7}\u{1e943}\u{1e944}\u{1e94a}\u{1e94b}\u{e001f}\u{e0020}\u{e007f}\u{e0080}\u{e00ff}\u{e0100}\u{e01ef}\u{e01f0}\u{feff}ſK\n",
                            "X\r\n\u{300}\u{2028}\u{301}\u{2029}",
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
    fn unicode_grapheme_extend_property_work_deduplication_and_contracts() {
        // New tags 56/57 must coexist with earlier low and high mask bits.
        for sets in [false, true] {
            let union = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[\p{ASCII}\p{Bidi_M}\p{Gr_Ext}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, true),
                (0x28, true),
                (0x300, true),
                (0x301, true),
                (0x1f600, false),
            ] {
                assert_eq!(union.matches(point), expected, "U+{point:X}");
            }
            let complement = RegExpUnicodeClassMatcher::compile_with_flags(
                &JsString::from(r"[^\P{ASCII}\p{Gr_Ext}\P{Bidi_M}]"),
                sets,
                false,
                false,
            )
            .unwrap();
            for (point, expected) in [
                (0x41, false),
                (0x28, true),
                (0x300, false),
                (0x301, false),
                (0x1f600, false),
            ] {
                assert_eq!(complement.matches(point), expected, "U+{point:X}");
            }
        }

        let source = JsString::from(
            format!("[{}]", r"\p{Gr_Ext}\P{Grapheme_Extend}".repeat(10000)).as_str(),
        );
        let plan =
            RegExpUnicodeClassMatcher::compile_with_flags(&source, true, true, false).unwrap();
        assert_eq!(plan.ranges.len(), 3);
        assert_eq!(
            plan.clone().find(&JsString::from("😀"), 1, true),
            Some(0..2)
        );
        for source in [r"^\P{Gr_Ext}$", r"^[\P{Grapheme_Extend}K]$"] {
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
                plan.find(&JsString::from("\u{300}\n😀\n"), 0, false),
                Some(2..4)
            );
        }
        for source in [
            r"[\p{Gr_Ext}--A]",
            r"[\p{Gr_Ext}&&A]",
            r"[[\p{Gr_Ext}]]",
            r"[A-\p{Gr_Ext}]",
            r"[\p{Gr_Ext}-A]",
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
                &JsString::from(r"[\p{Gr_Ext}]"),
                true,
                true,
                false
            )
            .is_none()
        );
    }
}
