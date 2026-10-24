//! Erased whole-Pattern class iterations (ECMA-262 22.2.2.3.1).

use crate::{JsString, regexp_literal::unicode_start, regexp_zero_count_group::zero_count};
use std::ops::Range;

/// An exactly-zero repetition of one complete, validated character class.
///
/// The caller must validate the entire Pattern and flags, including class
/// grammar and early errors, before compilation. RepeatMatcher with max zero
/// never queries class membership (22.2.2.3.1). The class therefore needs no
/// matching implementation, including data for string-valued properties.
/// Character classes contain no captures.
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
}
