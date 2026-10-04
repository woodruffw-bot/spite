//! Versioned Unicode normalization data and algorithmic Hangul mappings (UAX #15).

use crate::normalization_data::{COMBINING_CLASSES, COMPOSITIONS, DECOMPOSITIONS};
use std::cmp::Ordering;

const S_BASE: u32 = 0xac00;
const L_BASE: u32 = 0x1100;
const V_BASE: u32 = 0x1161;
const T_BASE: u32 = 0x11a7;
const L_COUNT: u32 = 19;
const V_COUNT: u32 = 21;
const T_COUNT: u32 = 28;
const N_COUNT: u32 = V_COUNT * T_COUNT;
const S_COUNT: u32 = L_COUNT * N_COUNT;

/// Returns a Unicode decomposition mapping, excluding algorithmic Hangul.
///
/// Canonical mappings are always included. Compatibility mappings are included
/// only when requested. Unmapped values, including surrogate code points, return
/// `None`. Callers recursively decompose the returned points before reordering.
pub fn unicode_decomposition(point: u32, compatibility: bool) -> Option<&'static [u32]> {
    let index = DECOMPOSITIONS
        .binary_search_by_key(&point, |&(cp, _, _)| cp)
        .ok()?;
    let (_, is_compatibility, mapping) = DECOMPOSITIONS[index];
    (!is_compatibility || compatibility).then_some(mapping)
}

/// Returns an algorithmic Hangul decomposition and its length (two or three).
pub fn hangul_decomposition(point: u32) -> Option<([u32; 3], usize)> {
    let index = point.checked_sub(S_BASE).filter(|&index| index < S_COUNT)?;
    let trailing = index % T_COUNT;
    Some((
        [
            L_BASE + index / N_COUNT,
            V_BASE + (index % N_COUNT) / T_COUNT,
            T_BASE + trailing,
        ],
        if trailing == 0 { 2 } else { 3 },
    ))
}

/// Returns the pinned Unicode canonical combining class, or zero for other values.
pub fn canonical_combining_class(point: u32) -> u8 {
    COMBINING_CLASSES
        .binary_search_by(|&(start, end, _)| {
            if point < start {
                Ordering::Greater
            } else if point > end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .map_or(0, |index| COMBINING_CLASSES[index].2)
}

/// Returns a canonical composite, applying composition exclusions and Hangul rules.
///
/// Callers apply the canonical blocking rule before composing these points.
pub fn canonical_composition(first: u32, second: u32) -> Option<u32> {
    if (L_BASE..L_BASE + L_COUNT).contains(&first) && (V_BASE..V_BASE + V_COUNT).contains(&second) {
        return Some(S_BASE + ((first - L_BASE) * V_COUNT + second - V_BASE) * T_COUNT);
    }
    if (S_BASE..S_BASE + S_COUNT).contains(&first)
        && (first - S_BASE) % T_COUNT == 0
        && (T_BASE + 1..T_BASE + T_COUNT).contains(&second)
    {
        return Some(first + second - T_BASE);
    }
    COMPOSITIONS
        .binary_search_by_key(&(first, second), |&(pair, _)| pair)
        .ok()
        .map(|index| COMPOSITIONS[index].1)
}
