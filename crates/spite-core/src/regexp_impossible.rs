//! Root alternatives blocked by an always-failing assertion (22.2.2.4).

use crate::{
    JsString, RegExpZeroCountSequenceMatcher, regexp_zero_count_group::enclosing_group_body,
};
use std::ops::Range;

/// A validated root sequence containing an always-failing negative assertion.
///
/// The complete original Pattern/flags must be validated and named capture
/// heads normalized first. A direct root negative lookahead/lookbehind whose
/// body always succeeds empty must fail at every position (22.2.2.4). Other
/// validated terms cannot rescue the sequence; no matching data is needed.
#[derive(Clone, Debug)]
pub struct RegExpImpossibleMatcher {
    captures: usize,
}

impl RegExpImpossibleMatcher {
    /// Proves a root sequence blocked by a negative assertion over an empty tree.
    ///
    /// Root choices and assertions inside enclosing groups require other proofs.
    /// A flat mode-aware scan counts every original capture. Projected assertion
    /// bodies keep original bindings; only independently proved empty bodies
    /// qualify. Source/copy work precedes allocation; retained storage is constant.
    pub fn compile_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut cursor = 0;
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        let mut captures = 0;
        let mut negative_body = None;
        let mut impossible = false;
        while let Some(&unit) = units.get(cursor) {
            if unit == 0x5c {
                cursor += 2;
                continue;
            }
            if class_depth != 0 {
                if unit == 0x5d {
                    class_depth -= 1;
                } else if sets && unit == 0x5b {
                    class_depth += 1;
                }
            } else {
                match unit {
                    0x5b => class_depth = 1,
                    0x28 => {
                        if depth == 0 {
                            negative_body = if units[cursor..].starts_with(&[0x28, 0x3f, 0x21]) {
                                Some(cursor + 3)
                            } else if units[cursor..].starts_with(&[0x28, 0x3f, 0x3c, 0x21]) {
                                Some(cursor + 4)
                            } else {
                                None
                            };
                        }
                        depth += 1;
                        captures += usize::from(units.get(cursor + 1) != Some(&0x3f));
                    }
                    0x29 => {
                        let Some(next) = depth.checked_sub(1) else {
                            return Ok(None);
                        };
                        depth = next;
                        if depth == 0 {
                            if let Some(start) = negative_body.take().filter(|_| !impossible) {
                                let body = JsString::from_code_units(units[start..cursor].to_vec());
                                impossible |= RegExpZeroCountSequenceMatcher::compile_surrogate_tree_validated_with_work(
                                    &body, unicode, sets, &mut charge,
                                )?.is_some();
                            }
                        }
                    }
                    0x7c if depth == 0 => return Ok(None),
                    _ => {}
                }
            }
            cursor += 1;
        }
        Ok((impossible && depth == 0 && class_depth == 0).then_some(Self { captures }))
    }

    /// Full original lexical capture count, including unobservable slots.
    pub fn capture_count(&self) -> usize {
        self.captures
    }

    /// Proves mandatory ordinary groups enclosing an impossible root sequence.
    ///
    /// Full original validation and normalized capture heads are required.
    /// The flat scope proof excludes quantified/modifier wrappers and outside
    /// terms. Every enclosing capture remains in the original inventory even
    /// though no match can expose it. References keep their original bindings.
    pub fn compile_enclosing_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let Some(scope) = enclosing_group_body(source.code_units(), sets) else {
            return Ok(None);
        };
        let body = JsString::from_code_units(source.code_units()[scope.body].to_vec());
        Ok(
            Self::compile_negative_validated_with_work(&body, unicode, sets, charge)?.map(|body| {
                Self {
                    captures: scope.captures + body.captures,
                }
            }),
        )
    }

    /// Proves a root choice only when every alternative is independently impossible.
    ///
    /// Requires full original validation and normalized capture heads. Empty or
    /// unproved alternatives prevent this proof. Each projected arm retains its
    /// original binding context and uses the direct or mandatory-enclosing proof.
    /// Source/copy work precedes allocation; branch scans remain source-sized.
    pub fn compile_alternatives_negative_validated_with_work<E>(
        source: &JsString,
        unicode: bool,
        sets: bool,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        charge(source.len().saturating_mul(8).saturating_add(16))?;
        if sets && !unicode {
            return Ok(None);
        }
        let units = source.code_units();
        let mut ranges = Vec::new();
        let mut start = 0;
        let mut cursor = 0;
        let mut depth = 0usize;
        let mut class_depth = 0usize;
        while let Some(&unit) = units.get(cursor) {
            if unit == 0x5c {
                cursor += 2;
                continue;
            }
            if class_depth != 0 {
                if unit == 0x5d {
                    class_depth -= 1;
                } else if sets && unit == 0x5b {
                    class_depth += 1;
                }
            } else {
                match unit {
                    0x5b => class_depth = 1,
                    0x28 => depth += 1,
                    0x29 => {
                        let Some(next) = depth.checked_sub(1) else {
                            return Ok(None);
                        };
                        depth = next;
                    }
                    0x7c if depth == 0 => {
                        ranges.push(start..cursor);
                        start = cursor + 1;
                    }
                    _ => {}
                }
            }
            cursor += 1;
        }
        if ranges.is_empty() || depth != 0 || class_depth != 0 {
            return Ok(None);
        }
        ranges.push(start..units.len());
        let mut captures = 0;
        for range in ranges {
            if range.is_empty() {
                return Ok(None);
            }
            let arm = JsString::from_code_units(units[range].to_vec());
            let mut plan =
                Self::compile_negative_validated_with_work(&arm, unicode, sets, &mut charge)?;
            if plan.is_none() {
                plan = Self::compile_enclosing_negative_validated_with_work(
                    &arm,
                    unicode,
                    sets,
                    &mut charge,
                )?;
            }
            let Some(plan) = plan else {
                return Ok(None);
            };
            captures += plan.captures;
        }
        Ok(Some(Self { captures }))
    }

    /// Returns no match, charging actual work before any possible input access.
    pub fn find_with_work<E>(
        &self,
        _input: &JsString,
        _start: usize,
        mut charge: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<Option<Range<usize>>, E> {
        charge(1)?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile(source: &str, unicode: bool, sets: bool) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_negative_validated_with_work(
            &JsString::from(source),
            unicode,
            sets,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn negative_empty_assertions_block_arbitrary_root_terms_and_count_every_capture() {
        for (source, count) in [
            (r"(?!)", 0),
            (r"(?<!a{0})", 0),
            (r"a(?!(b{0}))c", 1),
            (r"[\p{RGI_Emoji}]+(?!(a{0}))\1", 1),
            (r"(?<!(\uD83D\uDE00{0}))(\p{RGI_Emoji})+", 2),
            (r"(a|b)(?!(?=(c{0}))(?<=(d{0})))(e)", 4),
            (r"[[\q{a|ab}]&&\p{RGI_Emoji}](?!(\uD800{0}))", 1),
            (r"\((?!(a{0}))\)", 1),
        ] {
            assert_eq!(
                compile(source, true, true).unwrap().capture_count(),
                count,
                "{source}"
            );
        }
        assert!(compile("(?!a{0})", false, false).is_some());
    }

    #[test]
    fn root_choices_nested_assertions_and_conditional_bodies_need_other_proofs() {
        for source in [
            "",
            "(?=a{0})",
            "(?!a)",
            "(?<!a)",
            "(?!)|a{0}",
            "a{0}|(?!)",
            "(?:(?!a{0}))?",
            "(?:(?!a{0}))",
            "[()?!]",
        ] {
            assert!(compile(source, true, true).is_none(), "{source}");
        }
        assert!(compile("(?!)", false, true).is_none());
    }

    #[test]
    fn deep_prefixes_and_many_assertions_have_flat_source_work_and_constant_retained_state() {
        let source = format!(
            "{}a{}(?!(b{{0}}))",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        let plan = compile(&source, true, true).unwrap();
        assert_eq!(plan.capture_count(), 100_001);
        let source = "(?!a{0})".repeat(20_000);
        let mut work = 0;
        assert!(
            RegExpImpossibleMatcher::compile_negative_validated_with_work(
                &JsString::from(source.as_str()),
                true,
                true,
                |n| {
                    work += n;
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert!(work <= source.len() * 9 + 256);
    }

    #[test]
    fn constructor_abort_propagates_before_source_and_each_projected_body_step() {
        let source = JsString::from("(?!a)(?!(b{0}))(c)");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work.len(), 3);
        assert_eq!(work[0], source.len() * 8 + 16);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_negative_validated_with_work(
                &source, true, true, |n| {seen.push(n); if seen.len()==stop+1 {Err(stop)}else{Ok(())}}
            ), Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    #[test]
    fn no_match_work_is_constant_and_aborts_before_any_input_access() {
        let plan = compile(r"[\p{RGI_Emoji}]+(?!(a{0}))", true, true).unwrap();
        for input in ["", "a😀ab!", "a\r\nb"] {
            let input = JsString::from(input);
            for start in [0, 1, input.len(), input.len() + 1, usize::MAX] {
                let mut work = 0;
                assert_eq!(
                    plan.find_with_work(&input, start, |n| {
                        work += n;
                        Ok::<(), ()>(())
                    }),
                    Ok(None)
                );
                assert_eq!(work, 1);
                assert_eq!(plan.find_with_work(&input, start, |_| Err(53)), Err(53));
            }
        }
    }

    fn enclosing(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn mandatory_enclosing_impossible_groups_keep_all_original_capture_slots() {
        for (source, count) in [
            (r"([\p{RGI_Emoji}]+(?!(a{0})))", 2),
            (r"(?:((?<!\uD800{0})(a)))", 2),
            (r"((a|b)(?!(c{0}))(d))", 4),
            (r"(((?!(\2){0})[\p{RGI_Emoji}]+))", 3),
        ] {
            assert_eq!(
                enclosing(source).unwrap().capture_count(),
                count,
                "{source}"
            );
        }
    }

    #[test]
    fn enclosing_no_match_scope_excludes_optional_groups_siblings_and_outer_choices() {
        for source in [
            "(?!a{0})",
            "((?!a{0}))?",
            "((?!a{0})){0}",
            "(?i:(?!a{0}))",
            "((?!a{0}))|b{0}",
            "((?!a{0}))(x)",
            "x((?!a{0}))",
        ] {
            assert!(enclosing(source).is_none(), "{source}");
        }
    }

    #[test]
    fn hundred_thousand_enclosing_impossible_groups_need_no_native_recursion() {
        let source = format!(
            "{}a(?!(b{{0}})){}",
            "(".repeat(100_000),
            ")".repeat(100_000)
        );
        assert_eq!(enclosing(&source).unwrap().capture_count(), 100_001);
    }

    #[test]
    fn enclosing_impossible_work_aborts_before_each_projection_or_operand_step() {
        let source = JsString::from(r"(([\p{RGI_Emoji}]+(?!(a{0}))))");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert_eq!(work.len(), 3);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_enclosing_negative_validated_with_work(
                &source, true, true, |n| {seen.push(n); if seen.len()==stop+1 {Err(stop)}else{Ok(())}}
            ), Err(n) if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }

    fn alternatives(source: &str) -> Option<RegExpImpossibleMatcher> {
        RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
            &JsString::from(source),
            true,
            true,
            |_| Ok::<(), ()>(()),
        )
        .unwrap()
    }

    #[test]
    fn impossible_alternatives_reserve_all_original_slots_without_rebinding_arms() {
        for (source, captures) in [
            (r"(?!)|(?<!a{0})", 0),
            (r"[\p{RGI_Emoji}]+(?!(a{0}))|((?<!b{0})(c))", 3),
            (r"((?!(\3){0})[\p{RGI_Emoji}]+)|(?<!(\uD800{0}))(a)", 4),
            (r"\|(?!(a{0}))|[\q{a|ab}](?<!b{0})", 1),
        ] {
            assert_eq!(
                alternatives(source).unwrap().capture_count(),
                captures,
                "{source}"
            );
        }
    }

    #[test]
    fn any_empty_possible_or_unproved_alternative_prevents_no_match_proof() {
        for source in [
            "(?!)",
            "(?!)|a{0}",
            "a|(?!)",
            "(?!)|",
            "|(?!)",
            "(?!)|(?:(?!a{0}))?",
            "(?!(a))|(?!)",
            "(?:(?!)|a{0})",
        ] {
            assert!(alternatives(source).is_none(), "{source}");
        }
    }

    #[test]
    fn many_impossible_alternatives_have_linear_source_work_and_constant_retained_state() {
        let source = std::iter::repeat_n("(?!a{0})", 20_000)
            .collect::<Vec<_>>()
            .join("|");
        let mut work = 0;
        let plan = RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
            &JsString::from(source.as_str()),
            true,
            true,
            |n| {
                work += n;
                Ok::<(), ()>(())
            },
        )
        .unwrap()
        .unwrap();
        assert_eq!(plan.capture_count(), 0);
        assert!(work <= source.len() * 128);
    }

    #[test]
    fn impossible_alternative_work_aborts_before_each_arm_projection_and_operand_step() {
        let source = JsString::from("(?!a{0})|((?<!b{0}))");
        let mut work = Vec::new();
        assert!(
            RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
                &source,
                true,
                true,
                |n| {
                    work.push(n);
                    Ok::<(), ()>(())
                }
            )
            .unwrap()
            .is_some()
        );
        assert!(work.len() >= 5);
        for stop in 0..work.len() {
            let mut seen = Vec::new();
            assert!(
                matches!(RegExpImpossibleMatcher::compile_alternatives_negative_validated_with_work(
                &source,true,true,|n|{seen.push(n);if seen.len()==stop+1{Err(stop)}else{Ok(())}}
            ),Err(n)if n==stop)
            );
            assert_eq!(seen, work[..=stop]);
        }
    }
}
