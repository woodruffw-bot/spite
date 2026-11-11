//! RegExp character Canonicalize (22.2.2.7.3), using pinned Unicode data.

use crate::case_data::SIMPLE_CASE_FOLD;
use crate::unicode_case_mapping;

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
        insta::assert_snapshot!(rows.join("\n"), @r#"
            000000  exact=000000  i=000000  iu/iv=000000
            000041  exact=000041  i=000041  iu/iv=000061
            000061  exact=000061  i=000041  iu/iv=000061
            00007A  exact=00007A  i=00005A  iu/iv=00007A
            00007F  exact=00007F  i=00007F  iu/iv=00007F
            000080  exact=000080  i=000080  iu/iv=000080
            0000B5  exact=0000B5  i=00039C  iu/iv=0003BC
            0000DF  exact=0000DF  i=0000DF  iu/iv=0000DF
            0000FF  exact=0000FF  i=000178  iu/iv=0000FF
            000130  exact=000130  i=000130  iu/iv=000130
            000131  exact=000131  i=000131  iu/iv=000131
            00017F  exact=00017F  i=00017F  iu/iv=000073
            00039C  exact=00039C  i=00039C  iu/iv=0003BC
            0003BC  exact=0003BC  i=00039C  iu/iv=0003BC
            0003A3  exact=0003A3  i=0003A3  iu/iv=0003C3
            0003C2  exact=0003C2  i=0003A3  iu/iv=0003C3
            0003C3  exact=0003C3  i=0003A3  iu/iv=0003C3
            0003A9  exact=0003A9  i=0003A9  iu/iv=0003C9
            0003C9  exact=0003C9  i=0003A9  iu/iv=0003C9
            0010D0  exact=0010D0  i=001C90  iu/iv=0010D0
            0013A0  exact=0013A0  i=0013A0  iu/iv=0013A0
            001C89  exact=001C89  i=001C89  iu/iv=001C8A
            001C8A  exact=001C8A  i=001C89  iu/iv=001C8A
            001C90  exact=001C90  i=001C90  iu/iv=0010D0
            001E96  exact=001E96  i=001E96  iu/iv=001E96
            001E9E  exact=001E9E  i=001E9E  iu/iv=0000DF
            002126  exact=002126  i=002126  iu/iv=0003C9
            00212A  exact=00212A  i=00212A  iu/iv=00006B
            00AB70  exact=00AB70  i=0013A0  iu/iv=0013A0
            00D800  exact=00D800  i=00D800  iu/iv=00D800
            00DC00  exact=00DC00  i=00DC00  iu/iv=00DC00
            00FB00  exact=00FB00  i=00FB00  iu/iv=00FB00
            00FB03  exact=00FB03  i=00FB03  iu/iv=00FB03
            00FFFF  exact=00FFFF  i=00FFFF  iu/iv=00FFFF
            010400  exact=010400  i=------  iu/iv=010428
            010428  exact=010428  i=------  iu/iv=010428
            010D50  exact=010D50  i=------  iu/iv=010D70
            010D70  exact=010D70  i=------  iu/iv=010D70
            10FFFF  exact=10FFFF  i=------  iu/iv=10FFFF
            "#);
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
