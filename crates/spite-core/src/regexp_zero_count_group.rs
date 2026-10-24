//! Erased whole-Pattern group iterations (ECMA-262 22.2.2.3.1).

use crate::{JsString, regexp_literal::unicode_start};
use std::ops::Range;

/// An exactly-zero repetition of a complete, validated Pattern group.
///
/// The caller must first validate the complete Pattern and flags, including
/// early errors, and normalize named capture heads to ordinary capture heads.
/// Only the outer structural scope is proved here. RepeatMatcher with max zero
/// invokes its continuation without calling the body (22.2.2.3.1); consequently
/// the body needs no matching implementation and every capture is undefined.
#[derive(Clone, Debug)]
pub struct RegExpZeroCountGroupMatcher {
    captures: usize,
    unicode: bool,
}

impl RegExpZeroCountGroupMatcher {
    /// Proves a whole group followed by `{0}` or `{0,0}`, optionally lazy.
    ///
    /// Padded zero counts are accepted without numeric expansion. The flat
    /// scan charges before reading source and treats escaped units and classes
    /// as opaque. `unicode_sets` selects nested class syntax and requires
    /// `unicode`; both arguments must agree with the validated flags.
    pub fn compile_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(3).saturating_add(1))?;
        let units = source.code_units();
        if (unicode_sets && !unicode) || units.first() != Some(&0x28) {
            return Ok(None);
        }
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        let mut captures = 0usize;
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
                        if depth == 0 {
                            return Ok(zero_count(&units[index + 1..])
                                .then_some(Self { captures, unicode }));
                        }
                    }
                    _ => {}
                }
            }
            index += 1;
        }
        Ok(None)
    }

    /// Proves complete mandatory ordinary wrappers around a zero-count group.
    ///
    /// Validation and flag requirements are the same as for
    /// [`Self::compile_validated_with_work`]. The returned count is the number
    /// of enclosing captures, preceding all captures in the retained matcher.
    /// Those enclosing captures select its empty match span; the retained
    /// captures remain undefined (22.2.2.2, 22.2.2.7). Quantified wrappers,
    /// outside terms and alternatives require a different proof.
    pub fn compile_enclosing_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<(Self, usize)>, E> {
        charge(source.len().saturating_mul(6).saturating_add(1))?;
        if unicode_sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut opening_end = 0;
        let mut leading = 0;
        while let Some(width) = ordinary_opener(units, opening_end) {
            opening_end += width;
            leading += 1;
        }
        if leading == 0 {
            return Ok(None);
        }
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        let mut complete = 0;
        let mut remaining = leading;
        let mut index = 0;
        while let Some(&unit) = units.get(index) {
            index += 1;
            if unit == 0x5c {
                index += 1;
                continue;
            }
            if class_depth != 0 {
                if unit == 0x5d {
                    class_depth -= 1;
                } else if unicode_sets && unit == 0x5b {
                    class_depth += 1;
                }
                continue;
            }
            match unit {
                0x5b => class_depth = 1,
                0x28 => depth += 1,
                0x29 => {
                    let Some(next) = depth.checked_sub(1) else {
                        return Ok(None);
                    };
                    depth = next;
                    // Only the first closure at a leading wrapper's depth
                    // belongs to it; a later sibling must not replace it.
                    if depth < remaining {
                        remaining = depth;
                        if index == units.len() - depth {
                            complete = complete.max(depth + 1);
                        }
                    }
                }
                _ => {}
            }
        }
        if complete == 0 || depth != 0 || class_depth != 0 {
            return Ok(None);
        }
        let mut body_start = 0;
        let mut captures = 0;
        for _ in 0..complete {
            let Some(width) = ordinary_opener(units, body_start) else {
                return Ok(None);
            };
            captures += usize::from(width == 1);
            body_start += width;
        }
        let body = JsString::from_code_units(units[body_start..units.len() - complete].to_vec());
        // Original Pattern validation also covers references to enclosing
        // slots. Such references stay unexecuted; the projected body must not
        // be revalidated or assigned new lexical bindings in isolation.
        Ok(
            Self::compile_validated_with_work(&body, unicode, unicode_sets, charge)?
                .map(|matcher| (matcher, captures)),
        )
    }

    /// Number of lexical captures, all undefined (22.2.7.2).
    pub fn capture_count(&self) -> usize {
        self.captures
    }

    /// Finds the empty match at the requested boundary, with constant work.
    ///
    /// Unicode starts inside a surrogate pair use its leading boundary, as
    /// specified by the project's approved RegExpBuiltinExec interpretation.
    /// Sticky and searching execution have the same first empty match.
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

fn ordinary_opener(units: &[u16], start: usize) -> Option<usize> {
    let suffix = units.get(start..)?;
    if suffix.starts_with(&[0x28, 0x3f, 0x3a]) {
        Some(3)
    } else if suffix.first() == Some(&0x28) && suffix.get(1) != Some(&0x3f) {
        Some(1)
    } else {
        None
    }
}

fn zero_count(mut units: &[u16]) -> bool {
    if units.last() == Some(&0x3f) {
        units = &units[..units.len() - 1];
    }
    let Some(count) = units
        .strip_prefix(&[0x7b])
        .and_then(|s| s.strip_suffix(&[0x7d]))
    else {
        return false;
    };
    let mut halves = count.split(|&unit| unit == 0x2c);
    let zero = |half: &[u16]| !half.is_empty() && half.iter().all(|&unit| unit == 0x30);
    halves.next().is_some_and(zero) && halves.next().is_none_or(zero) && halves.next().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str, unicode: bool, sets: bool) -> Option<RegExpZeroCountGroupMatcher> {
        RegExpZeroCountGroupMatcher::compile_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn structural_scan_preserves_class_modes_escapes_and_capture_slots() {
        for (source, unicode, sets, captures) in [
            (r"((?=(a))(?<=(b))\2|[()\]](c)){0}", true, false, 4),
            (r"(?:[[a]--[b]](a)|[\q{\(\)|ab}]){000,000}?", true, true, 1),
            (r"([[](a)){0,0}", false, false, 2),
            (r"(?i:(a)(?-i:(b))){0}", true, false, 2),
            (r"(\((a)\)){0}?", false, false, 2),
        ] {
            assert_eq!(
                compile(source, unicode, sets).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
        for source in [
            "a(a){0}",
            "(a){0}b",
            "(a){0}|",
            "((a){0})",
            "(a){0,}",
            "(a){0,1}",
            "(a){1,0}",
            "(a){0}??",
            "(a){,0}",
            "(a){0,0,0}",
        ] {
            assert!(compile(source, true, false).is_none(), "{source}");
        }
        assert!(compile("(a){0}", false, true).is_none());
    }

    #[test]
    fn empty_matches_use_mode_boundaries_and_abort_before_inspection() {
        let input = JsString::from("A😀!");
        for unicode in [false, true] {
            let matcher = compile("((?=(a))b){0}", unicode, false).unwrap();
            for start in 0..=input.len() + 1 {
                let mut work = 0;
                let found = matcher
                    .find_with_work(&input, start, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    })
                    .unwrap();
                let expected = (start <= input.len()).then(|| {
                    let p = if unicode && start == 2 { 1 } else { start };
                    p..p
                });
                assert_eq!(found, expected);
                assert_eq!(work, 1);
                assert_eq!(matcher.find_with_work(&input, start, |_| Err(19)), Err(19));
            }
        }
        assert!(
            RegExpZeroCountGroupMatcher::compile_validated_with_work(
                &JsString::from("((a)){0}"),
                true,
                false,
                |_| Err(23)
            )
            .is_err()
        );
    }

    #[test]
    fn deep_body_and_decimal_counts_are_scanned_flat_without_matching_work() {
        let depth = 100_000;
        let source = JsString::from(
            format!(
                "{}(?=(a)){}{{{},{}}}",
                "(".repeat(depth),
                ")".repeat(depth),
                "0".repeat(depth),
                "0".repeat(depth),
            )
            .as_str(),
        );
        let mut work = 0;
        let matcher =
            RegExpZeroCountGroupMatcher::compile_validated_with_work(&source, true, false, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, source.len() * 3 + 1);
        assert_eq!(matcher.capture_count(), depth + 1);
        let clone = matcher.clone();
        let mut work = 0;
        assert_eq!(
            clone.find_with_work(&JsString::from(""), 0, |n| {
                work += n;
                Ok::<(), ()>(())
            }),
            Ok(Some(0..0))
        );
        assert_eq!(work, 1);
    }

    #[test]
    fn enclosing_groups_return_only_mandatory_capture_prefixes() {
        for (source, unicode, sets, outer, erased) in [
            (r"(((?=(a))b){0})", true, false, 1, 2),
            (r"(?:(?:(((?=(a))b){0})))", true, false, 1, 2),
            (r"(((?:(([[a]--[b]](c)){000,000}?))))", true, true, 3, 2),
            (r"((([[](a)){0}))", false, false, 2, 2),
            (r"(?:(?i:(a)(b)){0})", true, false, 0, 2),
            (r"(((((?=\4)a){0})))", true, false, 3, 1),
        ] {
            let source = JsString::from(source);
            let (matcher, captures) =
                RegExpZeroCountGroupMatcher::compile_enclosing_validated_with_work(
                    &source,
                    unicode,
                    sets,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(
                (captures, matcher.capture_count()),
                (outer, erased),
                "{source:?}"
            );
            assert!(
                RegExpZeroCountGroupMatcher::compile_validated_with_work(
                    &source,
                    unicode,
                    sets,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
        for source in [
            "((a){0})x",
            "x((a){0})",
            "((a){0})|((b){0})",
            "((a){0}|b)",
            "((a){0}){1}",
            "((a){0})?",
            "(a){0}",
            "((a){0,1})",
        ] {
            assert!(
                RegExpZeroCountGroupMatcher::compile_enclosing_validated_with_work(
                    &JsString::from(source),
                    true,
                    false,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
    }

    #[test]
    fn deep_enclosing_groups_and_every_construction_abort_stay_flat() {
        let depth = 100_000;
        let source = JsString::from(
            format!("{}((?=(a))b){{0}}{}", "(".repeat(depth), ")".repeat(depth)).as_str(),
        );
        let mut trace = Vec::new();
        let (matcher, captures) =
            RegExpZeroCountGroupMatcher::compile_enclosing_validated_with_work(
                &source,
                true,
                false,
                |n| {
                    trace.push(n);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(captures, depth);
        assert_eq!(matcher.capture_count(), 2);
        assert_eq!(trace.len(), 2);
        assert!(trace.iter().sum::<usize>() <= source.len() * 10);
        for fail in 0..trace.len() {
            let mut calls = 0;
            let result = RegExpZeroCountGroupMatcher::compile_enclosing_validated_with_work(
                &source,
                true,
                false,
                |_| {
                    let at = calls;
                    calls += 1;
                    if at == fail { Err(fail) } else { Ok(()) }
                },
            );
            assert!(matches!(result,Err(at) if at==fail));
            assert_eq!(calls, fail + 1);
        }
        let mut work = 0;
        assert_eq!(
            matcher
                .find_with_work(&JsString::from("A😀!"), 2, |n| {
                    work += n;
                    Ok::<(), ()>(())
                })
                .unwrap(),
            Some(1..1)
        );
        assert_eq!(work, 1);
    }
}
