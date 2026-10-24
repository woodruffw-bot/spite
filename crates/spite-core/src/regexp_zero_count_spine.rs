//! Empty class/property group spines (ECMA-262 22.2.2.3.1, 22.2.2.7).

use crate::{JsString, regexp_literal::unicode_start};
use std::ops::Range;

/// A validated atom erased by a zero bound on its complete group spine.
///
/// Requires full original Pattern/flag validation and normalized capture heads.
/// The class/property constructor requires one such atom; the group constructor
/// admits an arbitrary body inside an erased group. Outside groups must form
/// one spine of ordinary or ims modifier groups and validated quantifiers. A
/// bound with max zero proves that the entire Pattern matches empty.
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

    /// Proves an arbitrary zero-count group inside a quantified outside spine.
    ///
    /// Requires full original grammar/flag validation and normalized capture
    /// heads. The erased group's body needs no matching implementation, but all
    /// its lexical slots are reserved as undefined. Outside required captures
    /// are defined empty; outside optional groups clear their entire suffix.
    /// One flat scan records each leading group's first closure and counts
    /// every capture. A linear suffix pass excludes outside terms and choices.
    pub fn compile_group_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(32).saturating_add(16))?;
        if unicode_sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut opening_end = 0;
        let mut outside_captures = 0;
        let mut frames = Vec::new();
        while let Some((width, capturing)) = opener(units, opening_end) {
            frames.push(GroupFrame {
                before: outside_captures,
                close: 0,
                end: 0,
                minimum_zero: false,
                maximum_zero: false,
            });
            outside_captures += usize::from(capturing);
            opening_end += width;
        }
        if frames.is_empty() {
            return Ok(None);
        }
        let mut remaining = frames.len();
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        let mut captures = 0;
        let mut index = 0;
        while let Some(&unit) = units.get(index) {
            if unit == 0x5c {
                index += 2;
                continue;
            }
            if class_depth != 0 {
                if unit == 0x5d {
                    class_depth -= 1;
                } else if unicode_sets && unit == 0x5b {
                    class_depth += 1;
                }
            } else {
                match unit {
                    0x5b => class_depth = 1,
                    0x28 => {
                        depth += 1;
                        captures += usize::from(units.get(index + 1) != Some(&0x3f));
                    }
                    0x29 => {
                        let Some(next) = depth.checked_sub(1) else {
                            return Ok(None);
                        };
                        depth = next;
                        if depth < remaining {
                            remaining = depth;
                            let Some((end, minimum_zero, maximum_zero)) =
                                quantifier(units, index + 1)
                            else {
                                return Ok(None);
                            };
                            frames[depth].close = index;
                            frames[depth].end = end;
                            frames[depth].minimum_zero = minimum_zero;
                            frames[depth].maximum_zero = maximum_zero;
                        }
                    }
                    _ => {}
                }
            }
            index += 1;
        }
        if depth != 0 || class_depth != 0 || remaining != 0 {
            return Ok(None);
        }
        let mut outside_defined = captures;
        for (index, frame) in frames.iter().enumerate() {
            let expected_end = if index == 0 {
                units.len()
            } else {
                frames[index - 1].close
            };
            if frame.end != expected_end {
                break;
            }
            if frame.maximum_zero {
                return Ok(Some(Self {
                    unicode,
                    captures,
                    defined: outside_defined.min(frame.before),
                }));
            }
            if frame.minimum_zero {
                outside_defined = outside_defined.min(frame.before);
            }
        }
        Ok(None)
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

struct GroupFrame {
    before: usize,
    close: usize,
    end: usize,
    minimum_zero: bool,
    maximum_zero: bool,
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

    #[test]
    fn arbitrary_erased_bodies_preserve_native_outside_and_inside_capture_roles() {
        for (source, unicode, sets, expected) in [
            (r"((a|(?=(b))c){0})+", true, true, &[true, false, false][..]),
            (
                r"(?i:(([\p{RGI_Emoji}])(z)?){0})+",
                true,
                true,
                &[false, false, false][..],
            ),
            (
                r"((?:(a)(?<=b(c))|\p{RGI_Emoji}+){0}){2,3}",
                true,
                true,
                &[true, false, false][..],
            ),
            (r"(((a)\2){0})?", true, true, &[false, false, false][..]),
            (
                r"(?-i:((?=a)(b)){0}){2,3}?",
                true,
                true,
                &[false, false][..],
            ),
            (r"((a){0}x){0}", false, false, &[false, false][..]),
            (r"(?i:((?:x)(a))){0}", true, true, &[false, false][..]),
            (
                r"(?:(?:((a){0}){2,3}){0,2}){1}",
                true,
                true,
                &[false, false][..],
            ),
            (
                r"((((a){0})?)+)",
                true,
                true,
                &[true, true, false, false][..],
            ),
            (r"(([\q{\(|\)}]z?){0})+", true, true, &[true, false][..]),
            (r"(([()\]])(a)){0}", true, false, &[false, false, false][..]),
            (r"((?=(a))\1){0}", true, true, &[false, false][..]),
        ] {
            let matcher = RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                &JsString::from(source),
                unicode,
                sets,
                |_| Ok::<(), ()>(()),
            )
            .unwrap()
            .unwrap();
            assert_eq!(matcher.capture_count(), expected.len(), "{source}");
            for (index, &defined) in expected.iter().enumerate() {
                assert_eq!(
                    matcher.capture_range(index, &(1..1)),
                    defined.then_some(1..1),
                    "{source} slot {index}"
                );
            }
        }
    }

    #[test]
    fn erased_group_suffixes_exclude_other_outside_terms_and_later_sibling_closures() {
        for source in [
            r"((a){0}x)+",
            r"((a){0}|x)+",
            r"((a){0})+x",
            r"(a)((b){0})+",
            r"(?=(a){0})",
            r"((a){0})+|",
            r"[a]{0}",
            r"((a){0,1})+",
        ] {
            assert!(
                RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                    &JsString::from(source),
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_arbitrary_group_erasure_and_rejected_suffixes_have_source_sized_work() {
        let depth = 100_000;
        let source = JsString::from(
            format!(
                r"{}((a)(?=b(c))|\p{{RGI_Emoji}}+){{0}}{}",
                "(".repeat(depth),
                ")+".repeat(depth)
            )
            .as_str(),
        );
        let mut work = 0;
        let matcher = RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
            &source,
            true,
            true,
            |n| {
                work += n;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(work, source.len() * 32 + 16);
        assert_eq!(matcher.capture_count(), depth + 3);
        assert_eq!(matcher.capture_range(depth - 1, &(0..0)), Some(0..0));
        for index in depth..depth + 3 {
            assert_eq!(matcher.capture_range(index, &(0..0)), None);
        }
        assert!(
            RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                &source,
                true,
                true,
                |_| Err(47)
            )
            .is_err()
        );
        let rejected = JsString::from(
            format!("({}a{}x)+", "(".repeat(20_000), "){0}".repeat(20_000)).as_str(),
        );
        let mut work = 0;
        assert!(
            RegExpZeroCountSpineMatcher::compile_group_validated_with_work(
                &rejected,
                true,
                true,
                |n| {
                    work += n;
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_none()
        );
        assert_eq!(work, rejected.len() * 32 + 16);
    }
}
