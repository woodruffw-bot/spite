//! ECMAScript identifier character properties.

use crate::unicode_data::{ID_CONTINUE, ID_START};
use std::cmp::Ordering;

/// Returns whether a scalar can start an ECMAScript IdentifierName.
///
/// Implements IdentifierStartChar from ECMA-262 section 12.7 using the pinned
/// Unicode ID_Start property plus `$` and `_`.
pub fn is_identifier_start(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphabetic() || matches!(c, '$' | '_')
    } else {
        contains(ID_START, c as u32)
    }
}

/// Returns whether a scalar can continue an ECMAScript IdentifierName.
///
/// Implements IdentifierPartChar from ECMA-262 section 12.7 using the pinned
/// Unicode ID_Continue property plus `$`. ID_Continue includes `_` and joiners.
pub fn is_identifier_part(c: char) -> bool {
    if c.is_ascii() {
        c.is_ascii_alphanumeric() || matches!(c, '$' | '_')
    } else {
        contains(ID_CONTINUE, c as u32)
    }
}

fn contains(ranges: &[(u32, u32)], cp: u32) -> bool {
    ranges
        .binary_search_by(|&(start, end)| {
            if cp < start {
                Ordering::Greater
            } else if cp > end {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        })
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_identifier_boundaries() {
        for c in [
            'a',
            'Z',
            '$',
            '_',
            'π',
            '字',
            '\u{10400}',
            '\u{2118}',
            '\u{212e}',
            '\u{309b}',
            '\u{1885}',
            '\u{18e00}',
            '\u{3d000}',
        ] {
            assert!(is_identifier_start(c), "{c:?}");
            assert!(is_identifier_part(c), "{c:?}");
        }
        for c in [
            '0',
            '\u{0300}',
            '\u{0660}',
            '\u{00b7}',
            '\u{200c}',
            '\u{200d}',
            '\u{203f}',
            '\u{e0100}',
        ] {
            assert!(!is_identifier_start(c), "{c:?}");
            assert!(is_identifier_part(c), "{c:?}");
        }
        for c in [
            '\0',
            ' ',
            '-',
            '💩',
            '\u{0378}',
            '\u{2e2f}',
            '\u{feff}',
            '\u{10ffff}',
        ] {
            assert!(!is_identifier_start(c), "{c:?}");
            assert!(!is_identifier_part(c), "{c:?}");
        }
    }

    #[test]
    fn lookup_matches_ranges_for_every_unicode_scalar() {
        for ranges in [ID_START, ID_CONTINUE] {
            assert!(ranges.iter().all(|&(a, b)| a <= b && b <= 0x10ffff));
            assert!(ranges.windows(2).all(|w| w[0].1 + 1 < w[1].0));
            let mut cursor = 0;
            for cp in (0..=0x10ffff).filter(|cp| char::from_u32(*cp).is_some()) {
                while cursor < ranges.len() && ranges[cursor].1 < cp {
                    cursor += 1;
                }
                let expected = ranges.get(cursor).is_some_and(|&(start, _)| start <= cp);
                assert_eq!(contains(ranges, cp), expected, "U+{cp:04X}");
            }
        }
    }

    #[test]
    fn unicode_18_property_counts_and_subset() {
        // Counts from the pinned UCD, with ECMAScript's ASCII additions.
        let mut starts = 0;
        let mut parts = 0;
        for c in (0..=0x10ffff).filter_map(char::from_u32) {
            let start = is_identifier_start(c);
            let part = is_identifier_part(c);
            assert!(!start || part, "{c:?}");
            starts += usize::from(start);
            parts += usize::from(part);
        }
        assert_eq!(starts, 158739 + 2);
        assert_eq!(parts, 162100 + 1);
    }
}
