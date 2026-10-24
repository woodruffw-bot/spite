//! Composable empty Pattern trees (ECMA-262 22.2.2.2, 22.2.2.3.1, 22.2.2.7).

use crate::{
    JsString,
    regexp_zero_count_spine::{atom_end, opener, quantifier},
};

struct Frame {
    supported: bool,
    all_empty: bool,
    capture_start: usize,
    later_arms: Option<usize>,
}

impl Frame {
    fn new(supported: bool, capture_start: usize) -> Self {
        Self {
            supported,
            all_empty: true,
            capture_start,
            later_arms: None,
        }
    }
}

// Each active alternative must be unconditionally empty under backtracking.
// A first empty arm alone is insufficient inside RepeatMatcher (22.2.2.3.1).
pub(crate) fn compile_with_work<E>(
    source: &JsString,
    unicode: bool,
    sets: bool,
    mut charge: impl FnMut(usize) -> Result<(), E>,
) -> Result<Option<Box<[bool]>>, E> {
    charge(source.len().saturating_mul(64).saturating_add(16))?;
    if sets && !unicode {
        return Ok(None);
    }
    let units = source.code_units();
    let mut frames = vec![Frame::new(true, 0)];
    // A marker stores the furthest exclusive end of a cleared capture interval.
    // Prefix maxima apply overlapping/nested clears once, after the flat parse.
    let mut clear_ends = Vec::new();
    let mut cursor = 0;
    while let Some(&unit) = units.get(cursor) {
        match unit {
            0x28 => {
                let (width, capturing, supported) =
                    if let Some((width, capturing)) = opener(units, cursor) {
                        (width, capturing, true)
                    } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x3d])
                        || units[cursor..].starts_with(&[0x28, 0x3f, 0x21])
                    {
                        (3, false, false)
                    } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x3c, 0x3d])
                        || units[cursor..].starts_with(&[0x28, 0x3f, 0x3c, 0x21])
                    {
                        (4, false, false)
                    } else {
                        return Ok(None);
                    };
                let before = clear_ends.len();
                if capturing {
                    clear_ends.push(0);
                }
                frames.push(Frame::new(supported, before));
                cursor += width;
            }
            0x29 => {
                if frames.len() == 1 {
                    return Ok(None);
                }
                let frame = frames.pop().expect("non-root frame");
                if let Some(start) = frame.later_arms {
                    clear(&mut clear_ends, start);
                }
                let Some((end, minimum_zero, maximum_zero)) = quantifier(units, cursor + 1) else {
                    return Ok(None);
                };
                let empty = maximum_zero || (frame.supported && frame.all_empty);
                if empty && minimum_zero {
                    clear(&mut clear_ends, frame.capture_start);
                }
                frames.last_mut().expect("root frame").all_empty &= empty;
                cursor = end;
            }
            0x7c => {
                frames
                    .last_mut()
                    .expect("root frame")
                    .later_arms
                    .get_or_insert(clear_ends.len());
                cursor += 1;
            }
            _ => {
                let Some((end, erasable)) = leaf(units, cursor, unicode, sets) else {
                    return Ok(None);
                };
                let Some((end, _, maximum_zero)) = quantifier(units, end) else {
                    return Ok(None);
                };
                frames.last_mut().expect("root frame").all_empty &= erasable && maximum_zero;
                cursor = end;
            }
        }
    }
    if frames.len() != 1 {
        return Ok(None);
    }
    let root = frames.pop().expect("root frame");
    if !root.all_empty {
        return Ok(None);
    }
    if let Some(start) = root.later_arms {
        clear(&mut clear_ends, start);
    }
    let mut cleared_until = 0;
    Ok(Some(
        clear_ends
            .into_iter()
            .enumerate()
            .map(|(index, end)| {
                cleared_until = cleared_until.max(end);
                index >= cleared_until
            })
            .collect(),
    ))
}

fn clear(ends: &mut [usize], start: usize) {
    let end = ends.len();
    if let Some(marker) = ends.get_mut(start) {
        *marker = (*marker).max(end);
    }
}

// Unknown escapes stay unproved even with a following zero bound: legacy
// escapes can span more than one atom. An erased enclosing group can still
// suppress them without executing or reinterpreting their original bindings.
fn leaf(units: &[u16], cursor: usize, unicode: bool, sets: bool) -> Option<(usize, bool)> {
    if let Some(end) = atom_end(units, cursor, unicode, sets) {
        return Some((end, true));
    }
    let unit = *units.get(cursor)?;
    if unit == 0x5b {
        return None;
    }
    if unit == 0x5c {
        let escaped = *units.get(cursor + 1)?;
        let safe = matches!(
            escaped,
            0x5e | 0x24
                | 0x5c
                | 0x2e
                | 0x2a
                | 0x2b
                | 0x3f
                | 0x28
                | 0x29
                | 0x5b
                | 0x5d
                | 0x7b
                | 0x7d
                | 0x7c
                | 0x2f
                | 0x74
                | 0x6e
                | 0x72
                | 0x66
                | 0x76
                | 0x64
                | 0x44
                | 0x73
                | 0x53
                | 0x77
                | 0x57
        ) || (escaped == 0x30 && !matches!(units.get(cursor + 2), Some(0x30..=0x39)));
        return Some((cursor + 2, safe));
    }
    let erasable = !matches!(unit, 0x5e | 0x24 | 0x2a | 0x2b | 0x3f | 0x5d | 0x7b | 0x7d);
    let pair = unicode
        && (0xd800..=0xdbff).contains(&unit)
        && units
            .get(cursor + 1)
            .is_some_and(|u| (0xdc00..=0xdfff).contains(u));
    Some((cursor + 1 + usize::from(pair), erasable))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str, unicode: bool, sets: bool) -> Option<Box<[bool]>> {
        compile_with_work(&JsString::from(source), unicode, sets, |_| Ok::<(), ()>(())).unwrap()
    }

    #[test]
    fn native_empty_tree_masks_compose_every_active_alternative_and_capture_clear() {
        for (source, expected) in [
            (r"a{0}b{0}", &[][..]),
            (r"((a){0}|([b]{0})+)(c{0})", &[true, false, false, true][..]),
            (r"((a{0})?|(b{0})+)+", &[true, false, false][..]),
            (r"(((a){0})+(b{0})?){2,3}", &[true, true, false, false][..]),
            (
                r"(?i:((?:[\p{RGI_Emoji}]+|(?=(a))){0}|(b{0})))",
                &[true, false, false][..],
            ),
            (
                r"(?-i:((x{0})+|(\p{RGI_Emoji}{0})?)){0,2}",
                &[false, false, false][..],
            ),
            (
                r"((?:(a)|(b)){0})+((c{0})?(d{0})+)?",
                &[true, false, false, false, false, false][..],
            ),
            (r"\d{0}(a{0}|b{0})(?:\p{RGI_Emoji}{0}.){0}", &[true][..]),
            (r"((?:^|$|\b|\B|(a))){0}(😀{0})", &[false, false, true][..]),
            (r"([\p{RGI_Emoji}]{0}|(a){0})+", &[true, false][..]),
            (r"((a){0}|(b){0})*", &[false, false, false][..]),
        ] {
            assert_eq!(
                compile(source, true, true).as_deref(),
                Some(expected),
                "{source}"
            );
        }
    }

    #[test]
    fn consuming_alternatives_assertions_and_unproved_escapes_never_compose() {
        for source in [
            "a",
            "a?",
            "a*?",
            "|a",
            "(|a)+",
            "(|a)*",
            "(|a){2,3}",
            "([a]{0}|b)+",
            "(?i:(|[\\p{RGI_Emoji}]+))+",
            "^a{0}",
            "a{0}$",
            r"\b",
            r"(a{0})\1",
            r"\x61{0}",
            r"\u0061{0}",
            "(?=a{0})",
            "(?<=a{0})",
            "(?!a)",
            "(?<!a)",
        ] {
            assert!(compile(source, true, true).is_none(), "{source}");
        }
        assert!(compile("a{0}", false, true).is_none());
    }

    #[test]
    fn lexical_modes_safe_escapes_and_unicode_literal_atoms_keep_original_widths() {
        for (source, unicode, sets, expected) in [
            (r"([[]{0})([\(\)]{0})", false, false, &[true, true][..]),
            (r"([\q{\(|\)|\|}]{0})\P{ASCII}{0}", true, true, &[true][..]),
            (
                r"(\n{0})(\t{0})(\0{0})(\d{0})(\/{0})(.{0})",
                true,
                false,
                &[true; 6][..],
            ),
            ("(😀{0})", true, false, &[true][..]),
            ("😀{0}", false, false, &[][..]),
            (
                r"((?=(a))\x61\u0061\1){0}",
                true,
                false,
                &[false, false][..],
            ),
        ] {
            let got = compile(source, unicode, sets);
            if source == "😀{0}" && !unicode {
                assert!(got.is_none()); // Only the trailing code unit is erased.
            } else {
                assert_eq!(got.as_deref(), Some(expected), "{source}");
            }
        }
    }

    #[test]
    fn deep_tree_capture_clears_and_large_counts_use_linear_flat_storage() {
        let depth = 100_000;
        let source = format!(
            "{}(a){{0}}|(b){{0}}{}",
            "(".repeat(depth),
            "){2,3}".repeat(depth)
        );
        let mask = compile(&source, true, true).unwrap();
        assert_eq!(mask.len(), depth + 2);
        assert!(mask[..depth].iter().all(|&defined| defined));
        assert_eq!(&mask[depth..], &[false, false]);
        let source = format!("((a){{0}}|(b){{0}}){{{}}}", "9".repeat(20_000));
        assert_eq!(
            compile(&source, true, true).as_deref(),
            Some(&[true, false, false][..])
        );
        let source = format!(
            "{}((a){{0}}|(b){{0}})?{}",
            "(".repeat(depth),
            ")".repeat(depth)
        );
        let mask = compile(&source, true, true).unwrap();
        assert!(mask[..depth].iter().all(|&defined| defined));
        assert_eq!(&mask[depth..], &[false, false, false]);
    }

    #[test]
    fn source_and_frame_work_abort_before_parsing_or_allocation() {
        let source = JsString::from(r"((a){0}|([b]{0})+)(c{0})");
        let mut seen = Vec::new();
        assert!(matches!(
            compile_with_work(&source, true, true, |n| {
                seen.push(n);
                Err(37)
            }),
            Err(37)
        ));
        assert_eq!(seen, [source.len() * 64 + 16]);
        let mut seen = Vec::new();
        assert!(
            compile_with_work(&source, true, true, |n| {
                seen.push(n);
                Ok::<(), ()>(())
            })
            .unwrap()
            .is_some()
        );
        assert_eq!(seen, [source.len() * 64 + 16]);
    }
}
