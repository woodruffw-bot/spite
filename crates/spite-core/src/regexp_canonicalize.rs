//! RegExp character Canonicalize (22.2.2.7.3), using pinned Unicode data.

use crate::{case_data::SIMPLE_CASE_FOLD, unicode_case_mapping};

/// Returns the character used for RegExp comparisons under the supplied flags.
///
/// `unicode_mode` is true when either `u` or `v` is present. Characters are code
/// points in that mode and UTF-16 code units otherwise. Surrogates retain their
/// values. Unicode ignore-case uses simple/common folding; ordinary ignore-case
/// uses single-unit uppercase conversion and preserves non-ASCII-to-ASCII cases.
pub fn regexp_canonicalize_character(point: u32, ignore_case: bool, unicode_mode: bool) -> u32 {
    if !ignore_case {
        return point;
    }
    if unicode_mode {
        return SIMPLE_CASE_FOLD
            .binary_search_by_key(&point, |&(source, _)| source)
            .map_or(point, |index| SIMPLE_CASE_FOLD[index].1);
    }
    let Some(mapped) = char::from_u32(point).and_then(|point| unicode_case_mapping(point, true))
    else {
        return point;
    };
    let [unit] = mapped else {
        return point;
    };
    if point >= 128 && *unit < 128 {
        return point;
    }
    u32::from(*unit)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_characters_snapshot() {
        let points = [
            0x0000, 0x0041, 0x0061, 0x007a, 0x007f, 0x0080, 0x00b5, 0x00df, 0x00ff, 0x0130, 0x0131,
            0x017f, 0x039c, 0x03bc, 0x03a3, 0x03c2, 0x03c3, 0x03a9, 0x03c9, 0x10d0, 0x13a0, 0x1c89,
            0x1c8a, 0x1c90, 0x1e96, 0x1e9e, 0x2126, 0x212a, 0xab70, 0xd800, 0xdc00, 0xfb00, 0xfb03,
            0xffff, 0x10400, 0x10428, 0x10d50, 0x10d70, 0x10ffff,
        ];
        let rows: Vec<_> = points
            .into_iter()
            .map(|point| {
                assert_eq!(regexp_canonicalize_character(point, false, true), point);
                let ordinary = if point <= 0xffff {
                    assert_eq!(regexp_canonicalize_character(point, false, false), point);
                    format!("{:06X}", regexp_canonicalize_character(point, true, false))
                } else {
                    "------".into()
                };
                format!(
                    "{point:06X}  exact={point:06X}  i={ordinary}  iu/iv={:06X}",
                    regexp_canonicalize_character(point, true, true)
                )
            })
            .collect();
        insta::assert_snapshot!(rows.join("\n"));
    }

    #[test]
    fn every_code_unit_preserves_legacy_range_ascii_barriers_and_idempotence() {
        for point in 0..=u32::from(u16::MAX) {
            let folded = regexp_canonicalize_character(point, true, false);
            assert!(folded <= u32::from(u16::MAX));
            assert!(point < 128 || folded >= 128);
            assert_eq!(regexp_canonicalize_character(folded, true, false), folded);
            assert_eq!(regexp_canonicalize_character(point, false, false), point);
        }
    }

    #[test]
    fn simple_folding_is_single_scalar_idempotent_and_excludes_full_and_turkic_rules() {
        assert_eq!(SIMPLE_CASE_FOLD.len(), 1533);
        for &(source, target) in SIMPLE_CASE_FOLD {
            assert!(char::from_u32(source).is_some() && char::from_u32(target).is_some());
            assert_eq!(regexp_canonicalize_character(source, true, true), target);
            assert_eq!(regexp_canonicalize_character(target, true, true), target);
            assert_eq!(regexp_canonicalize_character(source, false, true), source);
        }
        for (point, expected) in [
            (0x0049, 0x0069),
            (0x0130, 0x0130),
            (0x0131, 0x0131),
            (0x00df, 0x00df),
            (0x1e9e, 0x00df),
            (0xfb00, 0xfb00),
            (0xd800, 0xd800),
            (0xdc00, 0xdc00),
            (0x10ffff, 0x10ffff),
        ] {
            assert_eq!(regexp_canonicalize_character(point, true, true), expected);
        }
    }
}
