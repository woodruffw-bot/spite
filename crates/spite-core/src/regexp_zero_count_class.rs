//! Erased whole-Pattern class iterations (ECMA-262 22.2.2.3.1).

use crate::{
    JsString,
    regexp_literal::unicode_start,
    regexp_zero_count_group::{enclosing_group_body, zero_count},
};
use std::ops::Range;

/// An exactly-zero repetition of a validated class or Unicode property escape.
///
/// The caller must validate the entire Pattern and flags, including class
/// grammar and early errors, before compilation. RepeatMatcher with max zero
/// never queries class membership (22.2.2.3.1). The class therefore needs no
/// matching implementation, including data for string-valued properties.
/// Character classes and property escapes contain no captures.
#[derive(Clone, Debug)]
pub struct RegExpZeroCountClassMatcher {
    unicode: bool,
}

impl RegExpZeroCountClassMatcher {
    /// Proves a complete class followed by `{0}` or `{0,0}`, optionally lazy.
    ///
    /// Padded counts are scanned without numeric expansion. Escapes are opaque;
    /// `unicode_sets` selects nested classes and requires `unicode`. Both mode
    /// arguments must agree with the validated flags. Construction charges
    /// source-sized work before scanning and retains constant storage.
    pub fn compile_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        unicode_sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(3).saturating_add(1))?;
        let units = source.code_units();
        if (unicode_sets && !unicode) || units.first() != Some(&0x5b) {
            return Ok(None);
        }
        let mut depth = 1usize;
        let mut index = 1;
        while let Some(&unit) = units.get(index) {
            if unit == 0x5c {
                index += 2;
                continue;
            }
            if unit == 0x5d {
                depth -= 1;
                if depth == 0 {
                    return Ok(zero_count(&units[index + 1..]).then_some(Self { unicode }));
                }
            } else if unicode_sets && unit == 0x5b {
                depth += 1;
            }
            index += 1;
        }
        Ok(None)
    }

    /// Proves a complete Unicode property escape with an exact zero bound.
    ///
    /// The caller must validate the complete Pattern with `u` or `v`, including
    /// the property's name and any string-property restrictions. This separate
    /// constructor accepts `\p{...}` and `\P{...}` followed only by `{0}` or
    /// `{0,0}`, with padded zeros and an optional lazy marker. Property names
    /// remain opaque: RepeatMatcher never queries their data (22.2.2.3.1).
    pub fn compile_unicode_property_validated_with_work<E>(
        source: &JsString,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(3).saturating_add(1))?;
        let units = source.code_units();
        if !units.starts_with(&[0x5c, 0x70, 0x7b]) && !units.starts_with(&[0x5c, 0x50, 0x7b]) {
            return Ok(None);
        }
        let Some(end) = units[3..].iter().position(|&unit| unit == 0x7d) else {
            return Ok(None);
        };
        if end == 0 {
            return Ok(None);
        }
        Ok(zero_count(&units[end + 4..]).then_some(Self { unicode: true }))
    }

    /// Proves complete mandatory ordinary wrappers around a zero-count class.
    ///
    /// The complete original Pattern and flags must already be validated and
    /// named capture heads normalized. The returned count gives every capture:
    /// all are enclosing wrappers and select the empty match span (22.2.2.7).
    /// The shared flat scan excludes continuations, choices and quantified
    /// wrappers, charging source work before temporary body allocation.
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
        let Some(group) = enclosing_group_body(source.code_units(), unicode_sets) else {
            return Ok(None);
        };
        let body = JsString::from_code_units(source.code_units()[group.body].to_vec());
        Ok(
            Self::compile_validated_with_work(&body, unicode, unicode_sets, charge)?
                .map(|matcher| (matcher, group.captures)),
        )
    }

    /// Returns the empty match at the requested boundary with constant work.
    ///
    /// Searching and sticky execution share the same first empty match.
    /// Unicode starts within a surrogate pair use its leading boundary under
    /// the project's approved RegExpBuiltinExec interpretation.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str, unicode: bool, sets: bool) -> Option<RegExpZeroCountClassMatcher> {
        RegExpZeroCountClassMatcher::compile_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn class_scope_handles_nested_sets_escaped_brackets_and_ordinary_left_brackets() {
        for (source, unicode, sets) in [
            (r"[\p{RGI_Emoji}]{0}", true, true),
            (r"[[\q{ab}]&&\p{RGI_Emoji}]{000,000}?", true, true),
            (r"[\q{\[\]|a}]{0}?", true, true),
            (r"[()\]]{0,0}", true, false),
            (r"[[]{0}", false, false),
            (r"[^]{0000}", true, false),
        ] {
            assert!(compile(source, unicode, sets).is_some(), "{source}");
        }
        for source in [
            "a[a]{0}",
            "[a]{0}b",
            "[a]{0}|",
            "([a]){0}",
            "[a]{0,}",
            "[a]{0,1}",
            "[a]{0}??",
            "[a]{0,0,0}",
        ] {
            assert!(compile(source, true, false).is_none(), "{source}");
        }
        assert!(compile("[a]{0}", false, true).is_none());
    }

    #[test]
    fn boundaries_end_of_input_and_work_aborts_do_not_query_class_data() {
        let input = JsString::from("A😀!");
        for unicode in [false, true] {
            let matcher = compile("[a]{0}", unicode, false).unwrap();
            for start in 0..=input.len() + 1 {
                let mut work = 0;
                let found = matcher
                    .find_with_work(&input, start, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    })
                    .unwrap();
                let expected = (start <= input.len()).then(|| {
                    let at = if unicode && start == 2 { 1 } else { start };
                    at..at
                });
                assert_eq!(found, expected);
                assert_eq!(work, 1);
                assert_eq!(matcher.find_with_work(&input, start, |_| Err(17)), Err(17));
            }
        }
        assert!(
            RegExpZeroCountClassMatcher::compile_validated_with_work(
                &JsString::from("[a]{0}"),
                true,
                false,
                |_| Err(23)
            )
            .is_err()
        );
    }

    #[test]
    fn deep_class_scopes_and_long_zero_bounds_stay_flat() {
        let depth = 100_000;
        let source = JsString::from(
            format!(
                "{}a{}{{{},{}}}",
                "[".repeat(depth),
                "]".repeat(depth),
                "0".repeat(depth),
                "0".repeat(depth)
            )
            .as_str(),
        );
        let mut work = 0;
        let matcher =
            RegExpZeroCountClassMatcher::compile_validated_with_work(&source, true, true, |n| {
                work += n;
                Ok::<(), ()>(())
            })
            .unwrap()
            .unwrap();
        assert_eq!(work, source.len() * 3 + 1);
        let clone = matcher.clone();
        assert_eq!(
            clone.find_with_work(&JsString::from(""), 0, |_| Ok::<(), ()>(())),
            Ok(Some(0..0))
        );
    }

    #[test]
    fn enclosing_class_proofs_return_the_complete_defined_capture_prefix() {
        for (source, unicode, sets, captures) in [
            (r"([\p{RGI_Emoji}]{0})", true, true, 1),
            (r"(?:(?:([\p{RGI_Emoji}]{0})))", true, true, 1),
            (
                r"((?:([[\q{ab}]&&\p{RGI_Emoji}]{000,000}?)))",
                true,
                true,
                2,
            ),
            (r"(([[]{0}))", false, false, 2),
            (r"((?:([()\]]{0})))", true, false, 2),
            (r"(?:[\p{RGI_Emoji}]{0})", true, true, 0),
        ] {
            let source = JsString::from(source);
            let (matcher, count) =
                RegExpZeroCountClassMatcher::compile_enclosing_validated_with_work(
                    &source,
                    unicode,
                    sets,
                    |_| Ok::<(), ()>(()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(count, captures, "{source:?}");
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("A😀!"), 2, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(if unicode { 1..1 } else { 2..2 })
            );
            assert!(
                RegExpZeroCountClassMatcher::compile_validated_with_work(
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
            r"([\p{RGI_Emoji}]{0})?",
            r"([\p{RGI_Emoji}]{0}){1}",
            r"([\p{RGI_Emoji}]{0})|x",
            r"([\p{RGI_Emoji}]{0}|x)",
            r"(a)([\p{RGI_Emoji}]{0})",
            r"([\p{RGI_Emoji}]{0})(b)",
            r"([\p{RGI_Emoji}]{0,1})",
            r"[\p{RGI_Emoji}]{0}",
        ] {
            assert!(
                RegExpZeroCountClassMatcher::compile_enclosing_validated_with_work(
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
    fn deep_class_enclosing_captures_and_each_construction_abort_are_flat() {
        let depth = 100_000;
        let source = JsString::from(
            format!(
                r"{}[\p{{RGI_Emoji}}]{{0}}{}",
                "(".repeat(depth),
                ")".repeat(depth)
            )
            .as_str(),
        );
        let mut trace = Vec::new();
        let (matcher, captures) =
            RegExpZeroCountClassMatcher::compile_enclosing_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    trace.push(n);
                    Ok::<(), usize>(())
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(captures, depth);
        assert_eq!(trace.len(), 2);
        assert!(trace.iter().sum::<usize>() <= source.len() * 10);
        for fail in 0..trace.len() {
            let mut calls = 0;
            let result = RegExpZeroCountClassMatcher::compile_enclosing_validated_with_work(
                &source,
                true,
                true,
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

    #[test]
    fn unicode_property_zero_bounds_are_separate_from_bracket_class_proofs() {
        for source in [
            r"\p{RGI_Emoji}{0}",
            r"\P{ASCII}{000,000}?",
            r"\p{General_Category=Lu}{0,0}",
            r"\p{Script=Greek}{0000}?",
        ] {
            let source = JsString::from(source);
            let mut work = 0;
            let matcher =
                RegExpZeroCountClassMatcher::compile_unicode_property_validated_with_work(
                    &source,
                    |n| {
                        work += n;
                        Ok::<(), ()>(())
                    },
                )
                .unwrap()
                .unwrap();
            assert_eq!(work, source.len() * 3 + 1);
            assert_eq!(
                matcher
                    .find_with_work(&JsString::from("A😀!"), 2, |_| Ok::<(), ()>(()))
                    .unwrap(),
                Some(1..1)
            );
            assert!(
                RegExpZeroCountClassMatcher::compile_validated_with_work(
                    &source,
                    true,
                    true,
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none()
            );
        }
        for source in [
            r"\p{RGI_Emoji}",
            r"\p{RGI_Emoji}{0,1}",
            r"\p{RGI_Emoji}{0}|x",
            r"\p{RGI_Emoji}{0}a",
            r"a\p{RGI_Emoji}{0}",
            r"(\p{RGI_Emoji}{0})",
            r"[\p{RGI_Emoji}]{0}",
        ] {
            assert!(
                RegExpZeroCountClassMatcher::compile_unicode_property_validated_with_work(
                    &JsString::from(source),
                    |_| Ok::<(), ()>(())
                )
                .unwrap()
                .is_none(),
                "{source}"
            );
        }
        assert!(
            RegExpZeroCountClassMatcher::compile_unicode_property_validated_with_work(
                &JsString::from(r"\p{RGI_Emoji}{0}"),
                |_| Err(41)
            )
            .is_err()
        );
    }
}
