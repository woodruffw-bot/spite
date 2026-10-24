//! Empty class/property group spines (ECMA-262 22.2.2.3.1, 22.2.2.7).

use crate::{JsString, regexp_literal::unicode_start};
use std::ops::Range;

/// A validated class/property atom erased by a zero bound on its group spine.
///
/// Requires full original Pattern/flag validation and normalized capture heads.
/// The Pattern contains one class or Unicode property escape, surrounded only
/// by ordinary or ims modifier groups and their validated quantifiers. At least
/// one bound must have max zero, proving that the entire Pattern matches empty.
/// No membership data is needed. Captures before the first optional group are
/// defined empty; captures in that group and its descendants remain undefined.
#[derive(Clone, Debug)]
pub struct RegExpZeroCountSpineMatcher {
    unicode: bool,
    captures: usize,
    defined: usize,
}

impl RegExpZeroCountSpineMatcher {
    /// Compiles a complete validated zero-erased class/property group spine.
    ///
    /// Numeric bounds are scanned without expansion. A required empty iteration
    /// may define captures; an optional empty iteration fails its continuation
    /// and rolls its capture writes back (22.2.2.3.1). Modifier flags have no
    /// observable effect on the erased body's matching. Work is charged before
    /// scanning and temporary frame allocation. Retained storage is constant.
    pub fn compile_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(12).saturating_add(16))?;
        if unicode_sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut index = 0;
        let mut captures = 0;
        let mut frames = Vec::new();
        while let Some((width, capturing)) = opener(units, index) {
            frames.push(captures);
            captures += usize::from(capturing);
            index += width;
        }
        let Some(end) = atom_end(units, index, unicode, unicode_sets) else {
            return Ok(None);
        };
        index = end;
        let Some((end, _, mut erased)) = quantifier(units, index) else {
            return Ok(None);
        };
        index = end;
        let mut defined = captures;
        while let Some(before) = frames.pop() {
            if units.get(index) != Some(&0x29) {
                return Ok(None);
            }
            index += 1;
            let Some((end, minimum_zero, maximum_zero)) = quantifier(units, index) else {
                return Ok(None);
            };
            if minimum_zero {
                defined = defined.min(before);
            }
            erased |= maximum_zero;
            index = end;
        }
        Ok((erased && index == units.len()).then_some(Self {
            unicode,
            captures,
            defined,
        }))
    }

    /// Number of lexical capture slots, including skipped groups (22.2.7.2).
    pub fn capture_count(&self) -> usize {
        self.captures
    }

    /// Absolute empty span for a participating group, or an undefined slot.
    pub fn capture_range(&self, index: usize, matched: &Range<usize>) -> Option<Range<usize>> {
        (index < self.defined).then(|| matched.clone())
    }

    /// Matches empty at the requested boundary, charging before input access.
    ///
    /// Unicode offsets inside a surrogate pair use its leading boundary under
    /// the project's approved RegExpBuiltinExec interpretation. Searching and
    /// sticky execution share the same first empty match.
    pub fn find_with_work<E>(
        &self,
        input: &JsString,
        start: usize,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        let start = if self.unicode {
            unicode_start(input, start)
        } else {
            (start <= input.len()).then_some(start)
        };
        Ok(start.map(|start| start..start))
    }
}

fn opener(units: &[u16], index: usize) -> Option<(usize, bool)> {
    if units.get(index) != Some(&0x28) {
        return None;
    }
    if units.get(index + 1) != Some(&0x3f) {
        return Some((1, true));
    }
    let mut end = index + 2;
    while matches!(units.get(end), Some(0x69 | 0x6d | 0x73 | 0x2d)) {
        end += 1;
    }
    (units.get(end) == Some(&0x3a)).then_some((end + 1 - index, false))
}

fn atom_end(units: &[u16], index: usize, unicode: bool, sets: bool) -> Option<usize> {
    if units.get(index) == Some(&0x5b) {
        let mut depth = 1usize;
        let mut end = index + 1;
        while let Some(&unit) = units.get(end) {
            if unit == 0x5c {
                end += 2;
                continue;
            }
            if unit == 0x5d {
                depth -= 1;
                if depth == 0 {
                    return Some(end + 1);
                }
            } else if sets && unit == 0x5b {
                depth += 1;
            }
            end += 1;
        }
        return None;
    }
    let suffix = units.get(index..)?;
    if unicode
        && (suffix.starts_with(&[0x5c, 0x70, 0x7b]) || suffix.starts_with(&[0x5c, 0x50, 0x7b]))
    {
        let end = suffix[3..].iter().position(|&unit| unit == 0x7d)?;
        return (end != 0).then_some(index + end + 4);
    }
    None
}

// Validated quantifier end, whether its minimum is zero, and whether max is zero.
// An absent quantifier has the single-iteration bounds of an ordinary Term.
fn quantifier(units: &[u16], index: usize) -> Option<(usize, bool, bool)> {
    let (mut end, minimum_zero, maximum_zero) = match units.get(index) {
        Some(0x3f | 0x2a) => (index + 1, true, false),
        Some(0x2b) => (index + 1, false, false),
        Some(0x7b) => {
            let mut end = index + 1;
            let lower = end;
            let mut minimum_zero = true;
            while let Some(&unit @ 0x30..=0x39) = units.get(end) {
                minimum_zero &= unit == 0x30;
                end += 1;
            }
            if end == lower {
                return None;
            }
            let maximum_zero = if units.get(end) == Some(&0x2c) {
                end += 1;
                let upper = end;
                let mut zero = true;
                while let Some(&unit @ 0x30..=0x39) = units.get(end) {
                    zero &= unit == 0x30;
                    end += 1;
                }
                zero && end != upper
            } else {
                minimum_zero
            };
            if units.get(end) != Some(&0x7d) {
                return None;
            }
            (end + 1, minimum_zero, maximum_zero)
        }
        _ => return Some((index, false, false)),
    };
    if units.get(end) == Some(&0x3f) {
        end += 1;
    }
    Some((end, minimum_zero, maximum_zero))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str, unicode: bool, sets: bool) -> Option<RegExpZeroCountSpineMatcher> {
        RegExpZeroCountSpineMatcher::compile_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn native_capture_roles_distinguish_required_empty_and_skipped_iterations() {
        for (source, unicode, sets, expected) in [
            (r"([\p{RGI_Emoji}]{0})?", true, true, &[false][..]),
            (r"([\p{RGI_Emoji}]{0})+", true, true, &[true][..]),
            (r"(([\p{RGI_Emoji}]{0})?)+", true, true, &[true, false][..]),
            (
                r"((?i:([\p{RGI_Emoji}]{0})+))*",
                true,
                true,
                &[false, false][..],
            ),
            (
                r"(?i:(([\p{RGI_Emoji}]){0})+)",
                true,
                true,
                &[true, false][..],
            ),
            (r"(([a]{0}){2,3}){2,3}?", false, false, &[true, true][..]),
            (r"(([\p{RGI_Emoji}]{0})+)?", true, true, &[false, false][..]),
            (
                r"(?-i:([\p{RGI_Emoji}]{0}){0002,0003}?)",
                true,
                true,
                &[true][..],
            ),
            (
                r"((\p{RGI_Emoji}{0})?){2,3}",
                true,
                true,
                &[true, false][..],
            ),
            (r"(?i:(\p{RGI_Emoji}){0})+", true, true, &[false][..]),
            (r"([[]){0}", false, false, &[false][..]),
            (r"(?:[a]{0}){0,}", false, false, &[][..]),
        ] {
            let matcher = compile(source, unicode, sets).unwrap();
            assert_eq!(matcher.capture_count(), expected.len(), "{source}");
            let range = if unicode { 1..1 } else { 2..2 };
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("A😀!"), 2, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(range.clone())
            );
            for (index, &defined) in expected.iter().enumerate() {
                assert_eq!(
                    matcher.capture_range(index, &range),
                    defined.then(|| range.clone()),
                    "{source} capture {index}"
                );
            }
            assert_eq!(matcher.capture_range(expected.len(), &range), None);
        }
    }

    #[test]
    fn scopes_requiring_membership_assertions_or_other_terms_remain_outside_the_proof() {
        for source in [
            r"([\p{RGI_Emoji}])+",
            r"([\p{RGI_Emoji}])*",
            r"([\p{RGI_Emoji}]{0})a",
            r"a([\p{RGI_Emoji}]{0})",
            r"([\p{RGI_Emoji}]{0})|x",
            r"((?=[a])[\p{RGI_Emoji}]{0})",
            r"([\p{RGI_Emoji}]{0}\b)",
            r"\p{RGI_Emoji}{0,1}",
        ] {
            assert!(compile(source, true, true).is_none(), "{source}");
        }
        assert!(compile("[a]{0}", false, true).is_none());
        assert!(compile(r"(\p{ASCII}){0}", false, false).is_none());
    }

    #[test]
    fn deep_quantified_spines_and_unrepresentable_minima_are_symbolic_and_flat() {
        let depth = 100_000;
        let source = JsString::from(
            format!(
                r"{}[\p{{RGI_Emoji}}]{{0}}{}",
                "(".repeat(depth),
                ")+".repeat(depth)
            )
            .as_str(),
        );
        let mut trace = Vec::new();
        let matcher =
            RegExpZeroCountSpineMatcher::compile_validated_with_work(&source, true, true, |n| {
                trace.push(n);
                Ok::<(), usize>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(trace, vec![source.len() * 12 + 16]);
        assert_eq!(matcher.capture_count(), depth);
        let mut work = 0;
        let range = matcher
            .find_with_work(&JsString::from("A😀!"), 2, |n| {
                work += n;
                Ok::<(), usize>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(range, 1..1);
        assert_eq!(work, 1);
        assert_eq!(matcher.capture_range(depth - 1, &range), Some(range));
        assert!(
            RegExpZeroCountSpineMatcher::compile_validated_with_work(&source, true, true, |_| Err(
                29
            ))
            .is_err()
        );
        let huge = "9".repeat(20_000);
        let matcher = compile(
            &format!(r"(([\p{{RGI_Emoji}}]{{0}})?){{{huge}}}"),
            true,
            true,
        )
        .unwrap();
        assert_eq!(matcher.capture_count(), 2);
        assert_eq!(matcher.capture_range(0, &(0..0)), Some(0..0));
        assert_eq!(matcher.capture_range(1, &(0..0)), None);
    }

    #[test]
    fn matches_abort_before_input_access_and_reject_out_of_bounds_starts() {
        for unicode in [false, true] {
            let matcher = compile("(([a]{0})?)+", unicode, false).unwrap();
            let input = JsString::from("A😀!");
            for start in 0..=input.len() + 1 {
                assert_eq!(matcher.find_with_work(&input, start, |_| Err(31)), Err(31));
                let found = matcher
                    .find_with_work(&input, start, |_| Ok::<(), ()>(()))
                    .unwrap();
                let expected = (start <= input.len()).then(|| {
                    let at = if unicode && start == 2 { 1 } else { start };
                    at..at
                });
                assert_eq!(found, expected);
            }
            let clone = matcher.clone();
            assert_eq!(clone.capture_count(), 2);
        }
    }
}
